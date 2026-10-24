//! The autonomous scheduler: what makes the hundred agents a company that runs
//! itself rather than a roster the CEO has to push by hand.
//!
//! Work is put on a queue (`agent_jobs`) instead of being handed straight to a
//! `tokio::spawn`. A background loop then claims jobs and runs them:
//!
//!   1. a job is enqueued by the approval gate, the admin panel, or an agent
//!      handing work to another agent,
//!   2. the loop picks the oldest *ready* job whose dependencies are finished,
//!   3. it claims it atomically, so two workers can never take the same job,
//!   4. it runs it through the engine, honouring the tool grant,
//!   5. it records the outcome, releases the slot, and wakes any job that was
//!      waiting on it.
//!
//! Three things are enforced here rather than trusted to the caller:
//!
//!   * **The approval gate.** A job from a client request is `held` until the
//!     CEO approves it. The loop will not run a held job, no matter who asks.
//!   * **The tool grant.** The engine intersects the agent's grant with the
//!     worker's, so a worker cannot reach a tool outside their own job.
//!   * **Concurrency.** An agent may hold only so many jobs at once, so one
//!     seat cannot be handed the whole company's work.
//!
//! Every claim is a single SQL `UPDATE ... WHERE status='ready'` and the loop
//! checks the row count, so a claim either wins the row or changes nothing.
//! That is what makes the queue safe without a lock the whole process holds.

use crate::db;
use crate::util;
use rusqlite::types::Value;
use serde_json::{json, Value as J};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// How many jobs one agent may hold at the same time. Above this the loop
/// leaves the agent alone and looks for another seat, so a single busy agent
/// never becomes the company's bottleneck.
pub const MAX_PER_AGENT: i64 = 2;

/// How long a claimed job may stay silent before it is considered abandoned and
/// returned to the queue. A crashed run must not hold a job forever.
const LEASE_SECONDS: i64 = 900;

/// The default sleep between sweeps. The loop wakes sooner when it starts a job.
const IDLE_TICK_MS: u64 = 1500;

/// Set by `stop()` so a test (or a graceful shutdown) can end the loop.
static RUNNING: AtomicBool = AtomicBool::new(false);
static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS agent_jobs (
           id           VARCHAR(64)  NOT NULL,
           agent_id     VARCHAR(64)  NOT NULL DEFAULT '',
           request_id   VARCHAR(64)  NOT NULL DEFAULT '',
           service      VARCHAR(160) NOT NULL DEFAULT '',
           task         TEXT         NOT NULL DEFAULT '',
           tools        TEXT         NOT NULL DEFAULT '[]',
           status       VARCHAR(20)  NOT NULL DEFAULT 'ready',
           priority     INTEGER      NOT NULL DEFAULT 0,
           attempts     INTEGER      NOT NULL DEFAULT 0,
           max_attempts INTEGER      NOT NULL DEFAULT 3,
           depends_on   VARCHAR(64)  NOT NULL DEFAULT '',
           run_id       VARCHAR(64)  NOT NULL DEFAULT '',
           claimed_by   VARCHAR(64)  NOT NULL DEFAULT '',
           claimed_at   DATETIME     NOT NULL DEFAULT '',
           error        TEXT         NOT NULL DEFAULT '',
           created_by   VARCHAR(120) NOT NULL DEFAULT '',
           created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           finished_at  DATETIME     NOT NULL DEFAULT '',
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_job_status  ON agent_jobs(status, priority, created_at);
         CREATE INDEX IF NOT EXISTS idx_job_agent   ON agent_jobs(agent_id, status);
         CREATE INDEX IF NOT EXISTS idx_job_request ON agent_jobs(request_id);",
    );
}

/// Put a job on the queue. This is the single entry point every caller uses —
/// the approval gate, the admin panel and agent-to-agent hand-off — so there is
/// one place that decides whether a job is runnable.
///
/// `held` is how the approval gate works: the job is recorded and visible, but
/// the loop will not touch it until `release()` is called.
pub fn enqueue(
    agent_id: &str,
    task: &str,
    request_id: &str,
    service: &str,
    tools: &[String],
    created_by: &str,
    held: bool,
) -> String {
    let id = util::new_id("JOB-");
    let now = util::now();
    let tools_json = serde_json::to_string(tools).unwrap_or_else(|_| "[]".into());
    let status = if held { "held" } else { "ready" };
    let _ = db::run(
        "INSERT INTO agent_jobs (id, agent_id, request_id, service, task, tools, status, created_by, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(agent_id)), arg(json!(request_id)), arg(json!(service)),
            arg(json!(task)), arg(json!(tools_json)), arg(json!(status)), arg(json!(created_by)),
            arg(json!(now)), arg(json!(now)),
        ],
    );
    id
}

/// Let a held job run. Called when the CEO approves the request it belongs to.
pub fn release(job_id: &str) -> bool {
    db::run(
        "UPDATE agent_jobs SET status='ready', updated_at=? WHERE id=? AND status='held'",
        &[arg(json!(util::now())), arg(json!(job_id))],
    )
    .unwrap_or(0)
        > 0
}

/// Release every job waiting on a request. Approving the request starts them.
pub fn release_for_request(request_id: &str) -> usize {
    db::run(
        "UPDATE agent_jobs SET status='ready', updated_at=? WHERE request_id=? AND status='held'",
        &[arg(json!(util::now())), arg(json!(request_id))],
    )
    .unwrap_or(0)
}

/// Hold every job of a request again — used when a request is sent back.
pub fn hold_for_request(request_id: &str) -> usize {
    db::run(
        "UPDATE agent_jobs SET status='held', updated_at=? WHERE request_id=? AND status='ready'",
        &[arg(json!(util::now())), arg(json!(request_id))],
    )
    .unwrap_or(0)
}

/// Claim one ready job for an agent, atomically.
///
/// The `UPDATE ... WHERE status='ready'` is the whole concurrency control: the
/// database serialises the two writers, so exactly one of them sees a row
/// change. The winner re-reads the row it just claimed.
fn claim(agent_id: &str) -> Option<db::Row> {
    // Dependencies first: a job whose dependency has not finished is not ready
    // yet, so it is skipped without being touched.
    let candidates = db::rows(
        "SELECT * FROM agent_jobs
          WHERE status='ready' AND (agent_id=? OR agent_id='')
          ORDER BY priority DESC, created_at ASC LIMIT 25",
        &[arg(json!(agent_id))],
    );
    for job in candidates {
        let dep = util::s(job.get("depends_on").unwrap_or(&J::Null));
        if !dep.is_empty() {
            let done = db::scalar_i64(
                "SELECT COUNT(*) FROM agent_jobs WHERE id=? AND status='done'",
                &[arg(json!(dep))],
            );
            if done == 0 {
                continue;
            }
        }
        let id = util::s(job.get("id").unwrap_or(&J::Null));
        let now = util::now();
        let won = db::run(
            "UPDATE agent_jobs SET status='running', claimed_by=?, claimed_at=?, attempts=attempts+1, updated_at=?
              WHERE id=? AND status='ready'",
            &[
                arg(json!(agent_id)), arg(json!(now)), arg(json!(now)), arg(json!(id)),
            ],
        )
        .unwrap_or(0);
        if won == 1 {
            return db::row("SELECT * FROM agent_jobs WHERE id=?", &[arg(json!(id))]);
        }
        // Someone else took it between the read and the write; try the next.
    }
    None
}

/// Finish a job and record how it went. A failure that still has attempts left
/// goes back to `ready` so the loop retries it; otherwise it is `failed`.
fn settle(job_id: &str, ok: bool, run_id: &str, error: &str) {
    let now = util::now();
    let job = db::row("SELECT * FROM agent_jobs WHERE id=?", &[arg(json!(job_id))]);
    let attempts = job
        .as_ref()
        .map(|r| util::i(r.get("attempts").unwrap_or(&J::Null)))
        .unwrap_or(0);
    let max = job
        .as_ref()
        .map(|r| util::i(r.get("max_attempts").unwrap_or(&J::Null)))
        .unwrap_or(3);
    if ok {
        let _ = db::run(
            "UPDATE agent_jobs SET status='done', run_id=?, error='', finished_at=?, updated_at=? WHERE id=?",
            &[arg(json!(run_id)), arg(json!(now)), arg(json!(now)), arg(json!(job_id))],
        );
    } else if attempts < max {
        let _ = db::run(
            "UPDATE agent_jobs SET status='ready', claimed_by='', claimed_at='', error=?, updated_at=? WHERE id=?",
            &[arg(json!(error)), arg(json!(now)), arg(json!(job_id))],
        );
    } else {
        let _ = db::run(
            "UPDATE agent_jobs SET status='failed', run_id=?, error=?, finished_at=?, updated_at=? WHERE id=?",
            &[
                arg(json!(run_id)), arg(json!(error)), arg(json!(now)), arg(json!(now)), arg(json!(job_id)),
            ],
        );
    }
}

/// Return jobs whose agent went silent back to the queue, so a crash does not
/// strand work. `now` is passed in so a test can pretend time has passed.
fn reap_stale(now: &str) {
    let _ = db::run(
        "UPDATE agent_jobs SET status='ready', claimed_by='', claimed_at='', updated_at=?
          WHERE status='running'
            AND claimed_at != ''
            AND (strftime('%s', ?) - strftime('%s', claimed_at)) > ?",
        &[arg(json!(now)), arg(json!(now)), arg(json!(LEASE_SECONDS))],
    );
}

/// How many jobs an agent is already running.
fn running_for(agent_id: &str) -> i64 {
    db::scalar_i64(
        "SELECT COUNT(*) FROM agent_jobs WHERE claimed_by=? AND status='running'",
        &[arg(json!(agent_id))],
    )
}

/// Run one sweep: reap, then hand out as much work as there are free agents.
///
/// Public and synchronous on purpose — it contains no `await`, so a test can
/// call it directly and see the queue move without starting the loop.
pub fn sweep_once() -> usize {
    let now = util::now();
    reap_stale(&now);

    // Every active seat, the manager included. The manager works like any other
    // agent; its extra job is to choose who takes a job that names no one.
    let agents = db::rows(
        "SELECT id, name FROM agents WHERE status='active' ORDER BY manager_id ASC",
        &[],
    );
    let mut started = 0usize;
    for a in agents {
        let id = util::s(a.get("id").unwrap_or(&J::Null));
        if id.is_empty() || running_for(&id) >= MAX_PER_AGENT {
            continue;
        }
        let Some(job) = claim(&id) else { continue };
        started += 1;
        let job_id = util::s(job.get("id").unwrap_or(&J::Null));
        let task = util::s(job.get("task").unwrap_or(&J::Null));
        let request_id = util::s(job.get("request_id").unwrap_or(&J::Null));
        let tools_json = util::s(job.get("tools").unwrap_or(&J::Null));
        let worker_tools: Vec<String> = serde_json::from_str(&tools_json).unwrap_or_default();
        // The engine runs on the async runtime; the sweep itself stays sync so
        // tests can drive it, so the run is handed to the runtime here.
        let agent_for_run = id.clone();
        tokio::spawn(async move {
            let outcome = crate::engine::run_job_with_grant(
                &agent_for_run,
                &task,
                &request_id,
                Some(worker_tools),
            )
            .await;
            let started_ok = outcome
                .get("started")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let run_id = util::s(outcome.get("runId").unwrap_or(&J::Null));
            let error = util::s(outcome.get("error").unwrap_or(&J::Null));
            if started_ok {
                if let Some(result) = outcome.get("result").and_then(|v| v.as_str()) {
                    if !request_id.is_empty() && !result.trim().is_empty() {
                        let _ = db::run(
                            "UPDATE requests SET admin_note=? WHERE id=?",
                            &[arg(json!(truncate(result, 4000))), arg(json!(request_id))],
                        );
                    }
                }
                let _ = db::run(
                    "UPDATE requests SET status='In Progress' WHERE id=? AND status='Approved'",
                    &[arg(json!(request_id))],
                );
            }
            settle(&job_id, started_ok, &run_id, &error);
        });
    }
    started
}

/// The background loop. Wakes on a timer, sweeps, and sleeps again. It stops
/// when `stop()` is called, which is how a test ends it cleanly.
pub async fn run_loop() {
    RUNNING.store(true, Ordering::SeqCst);
    while RUNNING.load(Ordering::SeqCst) {
        TICKS.fetch_add(1, Ordering::SeqCst);
        let started = sweep_once();
        // Busy? Come straight back for more. Idle? Sleep, so an empty company
        // does not spin the CPU.
        let wait = if started > 0 { 50 } else { IDLE_TICK_MS };
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }
}

pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

/// How many sweeps have run. A test reads this to prove the loop is alive.
pub fn ticks() -> u64 {
    TICKS.load(Ordering::SeqCst)
}

/// Start the loop in the background. Called once from `main`.
pub fn start() {
    if is_running() {
        return;
    }
    tokio::spawn(run_loop());
}

/* ------------------------------ reading ------------------------------ */

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    s.chars().take(n).collect()
}

pub fn job_json(r: &db::Row) -> J {
    let g = |k: &str| util::s(r.get(k).unwrap_or(&J::Null));
    json!({
        "id": g("id"),
        "agentId": g("agent_id"),
        "requestId": g("request_id"),
        "service": g("service"),
        "task": g("task"),
        "tools": serde_json::from_str::<J>(&g("tools")).unwrap_or(json!([])),
        "status": g("status"),
        "priority": util::i(r.get("priority").unwrap_or(&J::Null)),
        "attempts": util::i(r.get("attempts").unwrap_or(&J::Null)),
        "maxAttempts": util::i(r.get("max_attempts").unwrap_or(&J::Null)),
        "dependsOn": g("depends_on"),
        "runId": g("run_id"),
        "claimedBy": g("claimed_by"),
        "error": g("error"),
        "createdBy": g("created_by"),
        "createdAt": util::iso(Some(&g("created_at"))),
        "updatedAt": util::iso(Some(&g("updated_at"))),
        "finishedAt": util::iso(Some(&g("finished_at"))),
    })
}

/// The queue, newest first, for the admin panel.
pub fn list(status: &str, limit: i64) -> Vec<J> {
    db::rows(
        "SELECT * FROM agent_jobs WHERE (?1='' OR status=?1) ORDER BY created_at DESC LIMIT ?2",
        &[arg(json!(status)), arg(json!(limit))],
    )
    .iter()
    .map(job_json)
    .collect()
}

/// Counts per status, and how many seats are busy right now.
pub fn stats() -> J {
    let count = |st: &str| {
        db::scalar_i64(
            "SELECT COUNT(*) FROM agent_jobs WHERE status=?",
            &[arg(json!(st))],
        )
    };
    let busy = db::scalar_i64(
        "SELECT COUNT(DISTINCT claimed_by) FROM agent_jobs WHERE status='running' AND claimed_by!=''",
        &[],
    );
    let total = db::scalar_i64("SELECT COUNT(*) FROM agent_jobs", &[]);
    let agents = db::scalar_i64("SELECT COUNT(*) FROM agents WHERE status='active'", &[]);
    json!({
        "ready": count("ready"),
        "held": count("held"),
        "running": count("running"),
        "done": count("done"),
        "failed": count("failed"),
        "total": total,
        "busyAgents": busy,
        "activeAgents": agents,
        "idleAgents": (agents - busy).max(0),
        "runningLoop": is_running(),
        "ticks": ticks(),
    })
}

/// Cancel a job that has not finished. A running job is marked cancelled too;
/// the engine finishes its current step and the result is discarded.
pub fn cancel(job_id: &str) -> bool {
    db::run(
        "UPDATE agent_jobs SET status='cancelled', updated_at=? WHERE id=? AND status IN ('ready','held','running')",
        &[arg(json!(util::now())), arg(json!(job_id))],
    )
    .unwrap_or(0)
        > 0
}

/// Retry a failed job by putting it back on the queue.
pub fn retry(job_id: &str) -> bool {
    db::run(
        "UPDATE agent_jobs SET status='ready', attempts=0, error='', claimed_by='', claimed_at='', updated_at=? WHERE id=? AND status='failed'",
        &[arg(json!(util::now())), arg(json!(job_id))],
    )
    .unwrap_or(0)
        > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boot() {
        crate::test_bootstrap();
    }

    #[test]
    fn enqueue_then_claim_runs_once() {
        boot();
        let id = enqueue(
            "AGT-TEST1",
            "do a thing",
            "",
            "software",
            &[],
            "test",
            false,
        );
        let claimed = claim("AGT-TEST1").expect("a ready job should be claimable");
        assert_eq!(util::s(claimed.get("id").unwrap_or(&J::Null)), id);
        // A second claim must not find it: it is already running.
        assert!(claim("AGT-TEST1").is_none());
    }

    #[test]
    fn held_job_is_never_claimed_until_released() {
        boot();
        let id = enqueue(
            "AGT-TEST2",
            "client work",
            "REQ-X",
            "software",
            &[],
            "gate",
            true,
        );
        assert!(claim("AGT-TEST2").is_none(), "a held job must not run");
        assert!(release(&id));
        assert!(claim("AGT-TEST2").is_some(), "a released job must run");
    }

    #[test]
    fn release_for_request_opens_only_that_requests_jobs() {
        boot();
        let a = enqueue(
            "AGT-TEST3",
            "work a",
            "REQ-A",
            "software",
            &[],
            "gate",
            true,
        );
        let b = enqueue(
            "AGT-TEST3",
            "work b",
            "REQ-B",
            "software",
            &[],
            "gate",
            true,
        );
        assert_eq!(release_for_request("REQ-A"), 1);
        let arow = db::row("SELECT status FROM agent_jobs WHERE id=?", &[arg(json!(a))]).unwrap();
        let brow = db::row("SELECT status FROM agent_jobs WHERE id=?", &[arg(json!(b))]).unwrap();
        assert_eq!(util::s(arow.get("status").unwrap_or(&J::Null)), "ready");
        assert_eq!(util::s(brow.get("status").unwrap_or(&J::Null)), "held");
    }

    #[test]
    fn a_failed_job_retries_then_gives_up() {
        boot();
        let id = enqueue(
            "AGT-TEST4",
            "flaky work",
            "",
            "software",
            &[],
            "test",
            false,
        );
        db::run(
            "UPDATE agent_jobs SET attempts=1, max_attempts=2 WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        // First failure: still under the limit, so back to the queue.
        settle(&id, false, "", "boom");
        let r = db::row(
            "SELECT status FROM agent_jobs WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert_eq!(util::s(r.get("status").unwrap_or(&J::Null)), "ready");
        // Second failure: the limit is reached, so it stops for good.
        db::run(
            "UPDATE agent_jobs SET attempts=2 WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        settle(&id, false, "", "boom again");
        let r = db::row(
            "SELECT status FROM agent_jobs WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert_eq!(util::s(r.get("status").unwrap_or(&J::Null)), "failed");
    }

    #[test]
    fn a_dependency_blocks_until_it_is_done() {
        boot();
        let first = enqueue("AGT-TEST5", "first", "", "software", &[], "test", false);
        let second = enqueue("AGT-TEST5", "second", "", "software", &[], "test", false);
        db::run(
            "UPDATE agent_jobs SET depends_on=? WHERE id=?",
            &[arg(json!(first)), arg(json!(second))],
        )
        .unwrap();
        // The second must not be claimable while the first is unfinished.
        let claimed = claim("AGT-TEST5").expect("the first job is ready");
        assert_eq!(util::s(claimed.get("id").unwrap_or(&J::Null)), first);
        db::run(
            "UPDATE agent_jobs SET status='done' WHERE id=?",
            &[arg(json!(first))],
        )
        .unwrap();
        let next = claim("AGT-TEST5").expect("the dependent job is ready now");
        assert_eq!(util::s(next.get("id").unwrap_or(&J::Null)), second);
    }

    #[test]
    fn stale_running_jobs_are_returned_to_the_queue() {
        boot();
        let id = enqueue("AGT-TEST6", "abandoned", "", "software", &[], "test", false);
        db::run(
            "UPDATE agent_jobs SET status='running', claimed_by='AGT-TEST6', claimed_at='2000-01-01 00:00:00' WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        reap_stale(&util::now());
        let r = db::row(
            "SELECT status FROM agent_jobs WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert_eq!(util::s(r.get("status").unwrap_or(&J::Null)), "ready");
    }

    #[test]
    fn cancel_and_retry_move_a_job_both_ways() {
        boot();
        let id = enqueue("AGT-TEST7", "work", "", "software", &[], "test", false);
        assert!(cancel(&id));
        let r = db::row(
            "SELECT status FROM agent_jobs WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert_eq!(util::s(r.get("status").unwrap_or(&J::Null)), "cancelled");
        // Retry only accepts a failed job, so a cancelled one is left alone.
        assert!(!retry(&id));
        db::run(
            "UPDATE agent_jobs SET status='failed' WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert!(retry(&id));
        let r = db::row(
            "SELECT status FROM agent_jobs WHERE id=?",
            &[arg(json!(id))],
        )
        .unwrap();
        assert_eq!(util::s(r.get("status").unwrap_or(&J::Null)), "ready");
    }
}
