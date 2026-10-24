//! The agent engine: what turns a roster of names into a company that works.
//!
//! A run is one job handed to one agent. The engine loops:
//!
//!   1. build the tool specs from the agent's grant (nothing else is offered),
//!   2. ask the model what to do next,
//!   3. execute any tool call through `crate::runtime`, checking the grant first,
//!   4. feed the result back, until the model stops calling tools or the step
//!      budget runs out.
//!
//! Every step is written to `agent_runs` and `agent_steps`, so the CEO can open
//! any run and read exactly what the agent did, which tool it used and what came
//! back. That is the "admin sees everything" requirement, made real.
//!
//! The manager ("Magma Commander") is an ordinary agent whose job is to hand
//! work out. When a job arrives with no named agent, the engine asks the manager
//! to pick one from the roster.

use crate::db;
use crate::llm::{Call, Llm, Msg, Spec};
use crate::runtime;
use crate::tools;
use crate::util;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// How many model round trips one job may take before it is stopped. A small
/// budget keeps a runaway agent from burning a provider key.
const MAX_STEPS: usize = 12;

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS agent_runs (
           id          VARCHAR(64)   NOT NULL,
           agent_id    VARCHAR(64)   NOT NULL DEFAULT '',
           agent_name  VARCHAR(160)  NOT NULL DEFAULT '',
           request_id  VARCHAR(64)   NOT NULL DEFAULT '',
           task        TEXT          NOT NULL DEFAULT '',
           status      VARCHAR(30)   NOT NULL DEFAULT 'queued',
           steps       INTEGER       NOT NULL DEFAULT 0,
           result      TEXT          NOT NULL DEFAULT '',
           error       TEXT          NOT NULL DEFAULT '',
           provider    VARCHAR(40)   NOT NULL DEFAULT '',
           model       VARCHAR(120)  NOT NULL DEFAULT '',
           started_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           ended_at    DATETIME      NOT NULL DEFAULT '',
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS agent_steps (
           id          VARCHAR(64)   NOT NULL,
           run_id      VARCHAR(64)   NOT NULL,
           step_no     INTEGER       NOT NULL DEFAULT 0,
           kind        VARCHAR(30)   NOT NULL DEFAULT 'think',
           tool        VARCHAR(80)   NOT NULL DEFAULT '',
           input       TEXT          NOT NULL DEFAULT '',
           output      TEXT          NOT NULL DEFAULT '',
           ok          INTEGER       NOT NULL DEFAULT 1,
           created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_run_agent ON agent_runs(agent_id);
         CREATE INDEX IF NOT EXISTS idx_step_run ON agent_steps(run_id);",
    );
}

/// Pick the agent best suited to a request: one that holds the tool the service
/// needs, preferring an active agent in the right department. The manager's
/// choice, when the model is available, wins over this.
pub fn best_agent_for(service: &str, tool: &str) -> Option<db::Row> {
    let all = db::rows(
        "SELECT * FROM agents WHERE status='active' AND manager_id != ''",
        &[],
    );
    let want: Vec<String> = service
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .map(|s| s.to_string())
        .collect();
    let mut best: Option<(i64, db::Row)> = None;
    for a in all {
        let raw = util::s(a.get("tools").unwrap_or(&J::Null));
        let held = tools::granted(&raw);
        let mut score = 0i64;
        if !tool.is_empty() && held.iter().any(|t| t == tool) {
            score += 10;
        }
        let hay = format!(
            "{} {} {}",
            util::s(a.get("role").unwrap_or(&J::Null)),
            util::s(a.get("speciality").unwrap_or(&J::Null)),
            util::s(a.get("department").unwrap_or(&J::Null)),
        )
        .to_lowercase();
        for w in &want {
            if w.len() > 2 && hay.contains(w.as_str()) {
                score += 3;
            }
        }
        if util::s(a.get("autonomy").unwrap_or(&J::Null)) == "autonomous" {
            score += 1;
        }
        if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
            best = Some((score, a));
        }
    }
    best.map(|(_, a)| a)
}

/// The tools an agent may use, as model-facing specs. A grant that names an
/// unknown code is skipped rather than offered, so the catalogue stays the
/// single source of truth.
pub fn specs_for(grant_raw: &str) -> Vec<Spec> {
    tools::granted(grant_raw)
        .into_iter()
        .filter(|c| tools::known(c))
        .map(|c| {
            let mut s = runtime::spec(&c);
            s.name = runtime::spec_name(&c);
            s
        })
        .collect()
}

/// The system prompt that makes an agent behave like its role.
fn system_prompt(agent: &db::Row) -> String {
    let name = util::s(agent.get("name").unwrap_or(&J::Null));
    let role = util::s(agent.get("role").unwrap_or(&J::Null));
    let spec = util::s(agent.get("speciality").unwrap_or(&J::Null));
    let dept = util::s(agent.get("department").unwrap_or(&J::Null));
    format!(
        "You are {name}, the {role} in the {dept} department of Mbilinyi Tech Solutions \
         (MBILINYI-TECH67), a software, AI and security company. Your speciality: {spec}. \
         You do the work yourself, using only the tools you have been granted. \
         Work until the job is genuinely finished, then write the finished deliverable as your answer. \
         Never claim to have used a tool you were not given. \
         If a tool reports that it is not configured, say so plainly and continue with what you can do."
    )
}

/// A run, as the admin reads it back.
pub fn run_json(r: &db::Row) -> J {
    json!({
        "id": util::s(r.get("id").unwrap_or(&J::Null)),
        "agentId": util::s(r.get("agent_id").unwrap_or(&J::Null)),
        "agentName": util::s(r.get("agent_name").unwrap_or(&J::Null)),
        "requestId": util::s(r.get("request_id").unwrap_or(&J::Null)),
        "task": util::s(r.get("task").unwrap_or(&J::Null)),
        "status": util::s(r.get("status").unwrap_or(&J::Null)),
        "steps": util::i(r.get("steps").unwrap_or(&J::Null)),
        "result": util::s(r.get("result").unwrap_or(&J::Null)),
        "error": util::s(r.get("error").unwrap_or(&J::Null)),
        "provider": util::s(r.get("provider").unwrap_or(&J::Null)),
        "model": util::s(r.get("model").unwrap_or(&J::Null)),
        "startedAt": util::s(r.get("started_at").unwrap_or(&J::Null)),
        "endedAt": util::s(r.get("ended_at").unwrap_or(&J::Null)),
    })
}

fn step_json(r: &db::Row) -> J {
    json!({
        "step": util::i(r.get("step_no").unwrap_or(&J::Null)),
        "kind": util::s(r.get("kind").unwrap_or(&J::Null)),
        "tool": util::s(r.get("tool").unwrap_or(&J::Null)),
        "input": util::s(r.get("input").unwrap_or(&J::Null)),
        "output": util::s(r.get("output").unwrap_or(&J::Null)),
        "ok": util::i(r.get("ok").unwrap_or(&J::Null)) == 1,
        "at": util::s(r.get("created_at").unwrap_or(&J::Null)),
    })
}

fn log_step(
    run_id: &str,
    step_no: i64,
    kind: &str,
    tool: &str,
    input: &str,
    output: &str,
    ok: bool,
) {
    let _ = db::run(
        "INSERT INTO agent_steps (id, run_id, step_no, kind, tool, input, output, ok, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("STP-"))),
            arg(json!(run_id)),
            arg(json!(step_no)),
            arg(json!(kind)),
            arg(json!(tool)),
            arg(json!(truncate(input, 4000))),
            arg(json!(truncate(output, 8000))),
            arg(json!(if ok { 1 } else { 0 })),
            arg(json!(util::now())),
        ],
    );
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let cut: String = s.chars().take(n).collect();
    format!("{cut}… [truncated]")
}

/// Create the run row before the first model call, so the admin sees the job
/// the moment it is accepted rather than when it finishes.
fn open_run(agent: &db::Row, task: &str, request_id: &str, provider: &str, model: &str) -> String {
    let id = util::new_id("RUN-");
    let _ = db::run(
        "INSERT INTO agent_runs (id, agent_id, agent_name, request_id, task, status, provider, model, started_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(util::s(agent.get("id").unwrap_or(&J::Null)))),
            arg(json!(util::s(agent.get("name").unwrap_or(&J::Null)))),
            arg(json!(request_id)), arg(json!(task)), arg(json!("running")),
            arg(json!(provider)), arg(json!(model)), arg(json!(util::now())),
        ],
    );
    id
}

/// Run one job to completion. This is the whole engine.
///
/// It is called from a background task, so it takes ids rather than a `Ctx` and
/// touches the database only through short `db::` calls — no guard is ever held
/// across an `await`.
pub async fn run_job(agent_id: &str, task: &str, request_id: &str) -> J {
    run_job_with_grant(agent_id, task, request_id, None).await
}

/// Run a job, optionally constrained to the tools a named worker's job allows.
///
/// The worker is the one who was hired for the service, and their grant is what
/// their job needs. When a grant is given it is **intersected** with the agent's
/// own grant: an agent may use a tool only when both hold it. That is the rule
/// the CEO asked for — a worker may not use a tool outside their own job, and an
/// agent may not exceed what it was given either.
pub async fn run_job_with_grant(
    agent_id: &str,
    task: &str,
    request_id: &str,
    worker_grant: Option<Vec<String>>,
) -> J {
    let Some(agent) = db::row("SELECT * FROM agents WHERE id = ?", &[arg(json!(agent_id))]) else {
        return json!({ "started": false, "error": "Agent not found." });
    };
    if util::s(agent.get("status").unwrap_or(&J::Null)) == "paused" {
        return json!({ "started": false, "error": "That agent is paused." });
    }
    let llm = Llm::from_settings();
    let run_id = open_run(&agent, task, request_id, &llm.provider, &llm.model);

    if !llm.ready() {
        let msg = format!(
            "No AI provider is configured, so {} cannot start the job. \
             The CEO adds a provider key in Admin -> Settings.",
            util::s(agent.get("name").unwrap_or(&J::Null))
        );
        finish(&run_id, "blocked", "", &msg, 0);
        return json!({ "started": false, "runId": run_id, "error": msg });
    }

    let grant = effective_grant(
        &util::s(agent.get("tools").unwrap_or(&J::Null)),
        worker_grant.as_deref(),
    );
    let specs = specs_for(&grant);
    // A job that names tools, given to an agent that holds none of them, cannot
    // be done. Report that plainly rather than let the model write a report
    // about work it never carried out — a silent "done" is worse than a visible
    // refusal, because the CEO would read it as success.
    if specs.is_empty() {
        if let Some(w) = worker_grant.as_ref() {
            if !w.is_empty() && !w.iter().any(|x| x == "all") {
                let msg = format!(
                    "No agent holds the tools this job needs ({}). \
                     The CEO grants tools to an agent in Admin -> Agents.",
                    w.join(", ")
                );
                finish(&run_id, "blocked", "", &msg, 0);
                return json!({ "started": false, "runId": run_id, "error": msg });
            }
        }
    }
    // The model sees tool names with underscores; map back to catalogue codes.
    let code_of = |n: &str| {
        tools::CATALOGUE
            .iter()
            .find(|t| runtime::spec_name(t.code) == n)
            .map(|t| t.code.to_string())
    };

    let mut msgs = vec![
        Msg::System(system_prompt(&agent)),
        Msg::User(format!("Job to complete:\n{task}")),
    ];

    let mut steps = 0i64;
    let mut final_text = String::new();
    let mut failure = String::new();

    while steps < MAX_STEPS as i64 {
        steps += 1;
        let turn = match llm.chat(&msgs, &specs).await {
            Ok(t) => t,
            Err(e) => {
                failure = e;
                break;
            }
        };
        if !turn.text.trim().is_empty() {
            final_text = turn.text.clone();
            log_step(&run_id, steps, "say", "", "", &turn.text, true);
        }
        if turn.calls.is_empty() {
            break;
        }

        // Record what the model asked for, then run each call in order.
        let mut results: Vec<Msg> = Vec::new();
        let mut assistant_calls: Vec<Call> = Vec::new();
        for call in &turn.calls {
            let Some(code) = code_of(&call.name) else {
                let note = format!("There is no tool called {}.", call.name);
                log_step(
                    &run_id,
                    steps,
                    "refuse",
                    &call.name,
                    &call.args.to_string(),
                    &note,
                    false,
                );
                results.push(Msg::ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    content: note,
                });
                assistant_calls.push(call.clone());
                continue;
            };
            // The grant check: a tool the agent does not hold is refused here,
            // which is the promise "no-one works outside their own job".
            if !tools::may_use(&grant, &code) {
                let note = format!(
                    "{} does not hold the {} tool, so this call was refused.",
                    util::s(agent.get("name").unwrap_or(&J::Null)),
                    code
                );
                log_step(
                    &run_id,
                    steps,
                    "refuse",
                    &code,
                    &call.args.to_string(),
                    &note,
                    false,
                );
                results.push(Msg::ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    content: note,
                });
                assistant_calls.push(call.clone());
                continue;
            }
            let out = runtime::run(&code, &call.args).await;
            log_step(
                &run_id,
                steps,
                "tool",
                &code,
                &call.args.to_string(),
                &out.content,
                out.ok,
            );
            results.push(Msg::ToolResult {
                id: call.id.clone(),
                name: call.name.clone(),
                content: out.content,
            });
            assistant_calls.push(call.clone());
        }

        msgs.push(Msg::Assistant {
            text: turn.text.clone(),
            calls: assistant_calls,
        });
        for r in results {
            msgs.push(r);
        }
    }

    if !failure.is_empty() {
        finish(&run_id, "error", &final_text, &failure, steps);
        return json!({ "started": false, "runId": run_id, "error": failure });
    }
    if steps >= MAX_STEPS as i64 && final_text.trim().is_empty() {
        failure = format!("The job was stopped after {MAX_STEPS} steps without an answer.");
        finish(&run_id, "error", "", &failure, steps);
        return json!({ "started": false, "runId": run_id, "error": failure });
    }
    finish(&run_id, "done", &final_text, "", steps);
    let _ = db::run(
        "UPDATE agents SET tasks_done = tasks_done + 1 WHERE id = ?",
        &[arg(json!(agent_id))],
    );
    json!({ "started": true, "runId": run_id, "steps": steps, "result": final_text })
}

fn finish(run_id: &str, status: &str, result: &str, error: &str, steps: i64) {
    let _ = db::run(
        "UPDATE agent_runs SET status=?, result=?, error=?, steps=?, ended_at=? WHERE id=?",
        &[
            arg(json!(status)),
            arg(json!(truncate(result, 20000))),
            arg(json!(error)),
            arg(json!(steps)),
            arg(json!(util::now())),
            arg(json!(run_id)),
        ],
    );
}

/// Ask the manager which agent should take a job. Falls back to the local
/// scorer when no provider is configured or the model's answer names nobody.
pub async fn manager_pick(service: &str, tool: &str, task: &str) -> Option<db::Row> {
    let fallback = || best_agent_for(service, tool);
    let llm = Llm::from_settings();
    if !llm.ready() {
        return fallback();
    }
    let roster: Vec<String> = db::rows(
        "SELECT id, name, role, department, speciality FROM agents WHERE status='active' AND manager_id != ''",
        &[],
    )
    .into_iter()
    .map(|r| {
        format!(
            "{} | {} | {} | {}",
            util::s(r.get("id").unwrap_or(&J::Null)),
            util::s(r.get("name").unwrap_or(&J::Null)),
            util::s(r.get("role").unwrap_or(&J::Null)),
            util::s(r.get("speciality").unwrap_or(&J::Null)),
        )
    })
    .collect();
    if roster.is_empty() {
        return None;
    }
    let system =
        "You are Magma Commander, the manager of the agent company at Mbilinyi Tech Solutions. \
                  Pick the one agent from the roster who should take a new job. \
                  Reply with the agent id and nothing else.";
    let user = format!(
        "Roster:\n{}\n\nNew job service: {service}\nTool needed: {tool}\nJob: {task}\n\nWhich agent id?",
        roster.join("\n")
    );
    let msgs = vec![Msg::System(system.into()), Msg::User(user)];
    if let Ok(turn) = llm.chat(&msgs, &[]).await {
        let answer = turn.text.trim().to_string();
        if let Some(a) = db::row(
            "SELECT * FROM agents WHERE id = ? AND status='active' AND manager_id != ''",
            &[arg(json!(answer.clone()))],
        ) {
            return Some(a);
        }
        // The model may have written a sentence around the id; find any id in it.
        for r in db::rows(
            "SELECT * FROM agents WHERE status='active' AND manager_id != ''",
            &[],
        ) {
            let id = util::s(r.get("id").unwrap_or(&J::Null));
            if !id.is_empty() && answer.contains(&id) {
                return Some(r);
            }
        }
    }
    fallback()
}

/// Start a job in the background. The HTTP handler returns at once and the CEO
/// watches the run appear in the admin panel.
pub fn spawn(agent_id: String, task: String, request_id: String) {
    tokio::spawn(async move {
        let outcome = run_job(&agent_id, &task, &request_id).await;
        // A finished run with a result is offered to the client's request as an
        // internal note the CEO can review before it goes out.
        if outcome
            .get("started")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(result) = outcome.get("result").and_then(|v| v.as_str()) {
                if !request_id.is_empty() && !result.trim().is_empty() {
                    let _ = db::run(
                        "UPDATE requests SET admin_note = ? WHERE id = ?",
                        &[arg(json!(truncate(result, 4000))), arg(json!(request_id))],
                    );
                }
            }
        }
    });
}

/// Run a job for a named worker, constrained to the tools their job allows, and
/// write the outcome back onto the request so the CEO can review it before the
/// client sees anything. This is what the CEO's approval triggers.
pub fn spawn_with_tools(
    agent_id: String,
    task: String,
    request_id: String,
    worker_grant: Vec<String>,
) {
    tokio::spawn(async move {
        let outcome = run_job_with_grant(&agent_id, &task, &request_id, Some(worker_grant)).await;
        if outcome
            .get("started")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(result) = outcome.get("result").and_then(|v| v.as_str()) {
                if !request_id.is_empty() && !result.trim().is_empty() {
                    let _ = db::run(
                        "UPDATE requests SET admin_note = ? WHERE id = ?",
                        &[arg(json!(truncate(result, 4000))), arg(json!(request_id))],
                    );
                }
            }
            // Move the request along now that the work has been carried out; the
            // client is told, and the CEO still reviews before delivery.
            let _ = db::run(
                "UPDATE requests SET status = 'In Progress' WHERE id = ? AND status = 'Approved'",
                &[arg(json!(request_id))],
            );
        }
    });
}

/// The tools an agent may actually use: its own grant, intersected with the
/// worker's grant when the job came from a named worker. An empty intersection
/// is returned as-is, so the caller can see that nothing was permitted rather
/// than silently widening the grant to everything.
pub fn effective_grant(agent_grant: &str, worker_grant: Option<&[String]>) -> String {
    let held = tools::granted(agent_grant);
    let allowed: Vec<String> = match worker_grant {
        None => held,
        Some(w) => {
            if w.iter().any(|x| x == "all") {
                held
            } else {
                held.into_iter()
                    .filter(|t| w.iter().any(|x| x == t))
                    .collect()
            }
        }
    };
    serde_json::to_string(&allowed).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_only_offer_granted_and_known_tools() {
        let grant = json!(["web", "code_studio", "not_a_real_tool"]).to_string();
        let specs = specs_for(&grant);
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"web".to_string()));
        assert!(names.contains(&"code_studio".to_string()));
        assert!(!names.contains(&"not_a_real_tool".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn an_empty_grant_offers_nothing() {
        assert!(specs_for("[]").is_empty());
        assert!(specs_for("not json").is_empty());
    }

    #[test]
    fn long_output_is_truncated_for_the_log() {
        let long = "x".repeat(100);
        assert_eq!(truncate(&long, 10).chars().count() > 10, true);
        assert!(truncate(&long, 10).contains("truncated"));
        assert_eq!(truncate("short", 10), "short");
    }
}
