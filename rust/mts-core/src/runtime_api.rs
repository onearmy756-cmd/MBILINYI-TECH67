//! The runtime endpoint: what the platform can actually run.
//!
//! The admin needs to see, without guessing, which languages are installed, how
//! to run a snippet, and how to run a project's tests. A capability that is not
//! reported here is not claimed anywhere else in the platform.

use crate::api::ctx::Ctx;
use crate::db;
use crate::util::{self, bad, ok};
use axum::response::Response;
use serde_json::json;

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        // The installed toolchains. Available to any signed-in user, because a
        // client asking "can you run my Rust project?" deserves a true answer.
        "status" => {
            if c.require_user().is_err() {
                return crate::util::unauth();
            }
            ok(crate::langrun::status())
        }
        "sandbox" => {
            if c.require_admin().is_err() {
                return crate::util::forbidden("Admin access required.");
            }
            ok(crate::sandbox::status())
        }
        // Run a snippet. This is real execution, so it is admin-only: a worker
        // reaches it through their granted tool, never directly.
        "run" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let lang = c.p("language");
            let code = c.p("code");
            if code.trim().is_empty() {
                return bad("Give the code to run.");
            }
            let r = crate::langrun::run_snippet(&lang, &code);
            ok(json!({ "ok": r.ok, "ms": r.ms, "output": r.output }))
        }
        // Run a project's own test suite, in place. Admin-only for the same
        // reason.
        "test" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let lang = c.p("language");
            let path = c.p("path");
            let r = crate::langrun::run_tests(&lang, &path);
            ok(json!({ "ok": r.ok, "ms": r.ms, "output": r.output }))
        }
        // The test runs recorded against client work, so the CEO can see that
        // testing actually happened rather than taking it on trust.
        "test_log" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows(
                "SELECT id, run_id, step, tool, ok, note, created_at FROM agent_steps
                 WHERE tool IN ('test_runner','code_studio','build_pipeline','deployer')
                 ORDER BY created_at DESC LIMIT 200",
                &[],
            );
            let items: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "id": util::s(r.get("id").unwrap_or(&serde_json::Value::Null)),
                        "runId": util::s(r.get("run_id").unwrap_or(&serde_json::Value::Null)),
                        "tool": util::s(r.get("tool").unwrap_or(&serde_json::Value::Null)),
                        "ok": util::i(r.get("ok").unwrap_or(&serde_json::Value::Null)) == 1,
                        "note": util::s(r.get("note").unwrap_or(&serde_json::Value::Null)),
                        "at": util::iso(r.get("created_at").and_then(|v| v.as_str())),
                    })
                })
                .collect();
            ok(json!({ "runs": items, "count": items.len() }))
        }
        _ => bad("Unknown runtime action."),
    }
}
