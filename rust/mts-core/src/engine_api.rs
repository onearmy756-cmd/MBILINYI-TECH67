//! The endpoints the admin uses to watch and steer the agent engine.
//!
//! `run` accepts a job and returns at once; the work continues in the
//! background so a long job never holds an HTTP connection open. `runs` and
//! `run` (get) are how the CEO reads back exactly what an agent did.

use crate::api::ctx::Ctx;
use crate::db;
use crate::engine;
use crate::runtime;
use crate::tools;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        // ---- Watching ----
        "runs" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let status = c.p("status");
            let rows = db::rows(
                "SELECT * FROM agent_runs WHERE (?1 = '' OR status = ?1) ORDER BY started_at DESC LIMIT 200",
                &[arg(json!(status))],
            );
            ok(json!({ "runs": rows.iter().map(engine::run_json).collect::<Vec<_>>() }))
        }
        "run" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let Some(r) = db::row("SELECT * FROM agent_runs WHERE id = ?", &[arg(json!(id))])
            else {
                return err("Run not found.", 404);
            };
            let steps = db::rows(
                "SELECT * FROM agent_steps WHERE run_id = ? ORDER BY step_no ASC, created_at ASC",
                &[arg(json!(id))],
            )
            .into_iter()
            .map(|s| {
                json!({
                    "step": util::i(s.get("step_no").unwrap_or(&J::Null)),
                    "kind": util::s(s.get("kind").unwrap_or(&J::Null)),
                    "tool": util::s(s.get("tool").unwrap_or(&J::Null)),
                    "input": util::s(s.get("input").unwrap_or(&J::Null)),
                    "output": util::s(s.get("output").unwrap_or(&J::Null)),
                    "ok": util::i(s.get("ok").unwrap_or(&J::Null)) == 1,
                    "at": util::s(s.get("created_at").unwrap_or(&J::Null)),
                })
            })
            .collect::<Vec<_>>();
            ok(json!({ "run": engine::run_json(&r), "steps": steps }))
        }
        "stats" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let by_status = db::rows(
                "SELECT status, COUNT(*) AS n FROM agent_runs GROUP BY status",
                &[],
            )
            .into_iter()
            .map(|r| {
                json!({
                    "status": util::s(r.get("status").unwrap_or(&J::Null)),
                    "count": util::i(r.get("n").unwrap_or(&J::Null)),
                })
            })
            .collect::<Vec<_>>();
            ok(json!({
                "total": db::scalar_i64("SELECT COUNT(*) FROM agent_runs", &[]),
                "running": db::scalar_i64("SELECT COUNT(*) FROM agent_runs WHERE status='running'", &[]),
                "done": db::scalar_i64("SELECT COUNT(*) FROM agent_runs WHERE status='done'", &[]),
                "byStatus": by_status,
            }))
        }
        // ---- Steering ----
        "run_job" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let task = c.p("task");
            if task.trim().is_empty() {
                return bad("Describe the job the agent should do.");
            }
            let request_id = c.p("requestId");
            // No agent named? Let the manager choose, which is the whole point
            // of the manager seat.
            let agent_id = {
                let named = c.p("agentId");
                if !named.trim().is_empty() {
                    named
                } else {
                    let tool = c.p("tool");
                    match engine::manager_pick(&c.p("service"), &tool, &task).await {
                        Some(a) => util::s(a.get("id").unwrap_or(&J::Null)),
                        None => return bad("There is no active agent to take this job."),
                    }
                }
            };
            let tool = c.p("tool");
            if !tool.is_empty() {
                let Some(a) = db::row(
                    "SELECT * FROM agents WHERE id = ?",
                    &[arg(json!(agent_id.clone()))],
                ) else {
                    return err("Agent not found.", 404);
                };
                let grant = util::s(a.get("tools").unwrap_or(&J::Null));
                if !tools::may_use(&grant, &tool) {
                    return bad("That agent does not hold the tool this job needs.");
                }
            }
            engine::spawn(agent_id.clone(), task.clone(), request_id.clone());
            let a = db::row(
                "SELECT name FROM agents WHERE id = ?",
                &[arg(json!(agent_id.clone()))],
            );
            ok(json!({
                "started": true,
                "agentId": agent_id,
                "agent": a.map(|r| util::s(r.get("name").unwrap_or(&J::Null))).unwrap_or_default(),
                "message": "Kazi imeanza. Utaiona kwenye Runs mara tu inapoendelea.",
            }))
        }
        "cancel" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let n = db::run(
                "UPDATE agent_runs SET status='cancelled', ended_at=? WHERE id=? AND status='running'",
                &[arg(json!(util::now())), arg(json!(id))],
            )
            .unwrap_or(0);
            ok(json!({ "cancelled": n > 0 }))
        }
        // ---- The scheduler's queue: what the company is doing, and what waits ----
        "queue" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let status = c.p("status");
            let limit = c.p("limit").parse::<i64>().unwrap_or(100).clamp(1, 500);
            ok(json!({
                "jobs": crate::scheduler::list(&status, limit),
                "stats": crate::scheduler::stats(),
            }))
        }
        "queue_stats" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(crate::scheduler::stats())
        }
        "queue_cancel" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("jobId");
            ok(json!({ "cancelled": crate::scheduler::cancel(&id), "jobId": id }))
        }
        "queue_retry" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("jobId");
            ok(json!({ "retried": crate::scheduler::retry(&id), "jobId": id }))
        }
        // The CEO's approval, for a job filed by hand. A held job runs only once
        // this is called — which is the whole point of the gate.
        "queue_release" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("jobId");
            ok(json!({ "released": crate::scheduler::release(&id), "jobId": id }))
        }
        // Put a job on the queue by hand. `hold: true` files it behind the
        // approval gate; the default is to let it run.
        "queue_add" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let task = c.p("task");
            if task.trim().is_empty() {
                return bad("Describe the job the agent should do.");
            }
            let tools: Vec<String> = crate::tools::granted(&c.p("tools"));
            let agent_id = {
                let named = c.p("agentId");
                if !named.trim().is_empty() {
                    named
                } else {
                    crate::agents::pick_for_tools(&tools)
                }
            };
            let held = c.p("hold") == "true" || c.p("hold") == "1";
            let job = crate::scheduler::enqueue(
                &agent_id,
                &task,
                &c.p("requestId"),
                &c.p("service"),
                &tools,
                "admin",
                held,
            );
            ok(json!({ "jobId": job, "agentId": agent_id, "held": held }))
        }
        // Kick the loop once, so the CEO can make the queue move without waiting
        // for the next tick. Returns how many jobs started.
        "queue_sweep" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(json!({ "started": crate::scheduler::sweep_once() }))
        }
        // ---- The runtime's own health ----
        "capabilities" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let llm = crate::llm::Llm::from_settings();
            let by_family = db::rows(
                "SELECT department, COUNT(*) AS n FROM agents GROUP BY department ORDER BY n DESC",
                &[],
            )
            .into_iter()
            .map(|r| {
                json!({
                    "department": util::s(r.get("department").unwrap_or(&J::Null)),
                    "count": util::i(r.get("n").unwrap_or(&J::Null)),
                })
            })
            .collect::<Vec<_>>();
            ok(json!({
                "provider": llm.provider,
                "model": llm.model,
                "ready": llm.ready(),
                "toolCount": runtime::executable().len(),
                "agents": db::scalar_i64("SELECT COUNT(*) FROM agents", &[]),
                "activeAgents": db::scalar_i64("SELECT COUNT(*) FROM agents WHERE status='active'", &[]),
                "departments": by_family,
            }))
        }
        _ => bad("Unknown engine action."),
    }
}
