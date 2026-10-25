//! Mbilinyi Coding — the real OpenHands agent, run from inside the platform.
//!
//! This is not a re-implementation. A job started here is handed to an actual
//! OpenHands conversation on the OpenHands Cloud API: a real sandbox is created,
//! a real agent runs, and the transcript is read back event by event. The panel
//! only renames it "Mbilinyi Coding" so the CEO sees the company's own product
//! name; underneath, the wire protocol, the sandbox and the agent are OpenHands.
//!
//! The admin configures it in one place (Admin -> Mbilinyi Coding):
//!
//!   * the API key (a platform setting, or the `OPENHANDS_API_KEY` env),
//!   * the model every agent uses — the same provider key the platform already
//!     holds for its own LLM layer, so "agents use the configured LLM" is a
//!     setting, not a code change,
//!   * the tools an agent may be given, from the platform's own catalogue.
//!
//! Every call is admin-only: a conversation costs money and can touch a
//! repository, so no visitor and no ordinary worker can start one.

use crate::api::ctx::Ctx;
use crate::db;
use crate::settings;
use crate::util::{self, bad, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

/// A JSON value as a SQLite parameter, the same helper `engine` uses.
fn arg(v: J) -> Value {
    db::from_json(v)
}

/// Where the OpenHands Cloud API lives. Overridable so a self-hosted
/// deployment, or a test, can point somewhere else.
fn base_url() -> String {
    let b = settings::get("coding_base_url", "");
    let b = if b.trim().is_empty() {
        std::env::var("OPENHANDS_BASE_URL").unwrap_or_else(|_| "https://app.all-hands.dev".into())
    } else {
        b
    };
    b.trim().trim_end_matches('/').to_string()
}

/// The API key. The admin setting wins so the CEO can rotate it without a
/// redeploy; the environment is the fallback for a container that holds it as a
/// Fly secret.
fn api_key() -> String {
    let from_settings = settings::get("coding_api_key", "");
    if !from_settings.trim().is_empty() {
        return from_settings.trim().to_string();
    }
    std::env::var("OPENHANDS_API_KEY")
        .or_else(|_| std::env::var("OPENHANDS_CLOUD_API_KEY"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// The model every coding agent uses. Defaults to the platform's own LLM
/// settings, so "agents use the configured LLM" holds out of the box; the CEO
/// can pin a different one for coding alone.
fn model() -> String {
    let m = settings::get("coding_model", "");
    if !m.trim().is_empty() {
        return m.trim().to_string();
    }
    let provider = settings::get("llm_provider", "");
    let model = settings::get("llm_model", "");
    if provider.trim().is_empty() || model.trim().is_empty() {
        String::new()
    } else {
        format!("{}/{}", provider.trim(), model.trim())
    }
}

pub fn configured() -> bool {
    !api_key().is_empty()
}

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS coding_jobs (
           id            VARCHAR(64)   NOT NULL,
           title         VARCHAR(200)  NOT NULL DEFAULT '',
           task          TEXT          NOT NULL DEFAULT '',
           repository    VARCHAR(200)  NOT NULL DEFAULT '',
           branch        VARCHAR(120)  NOT NULL DEFAULT '',
           model         VARCHAR(160)  NOT NULL DEFAULT '',
           agent_id      VARCHAR(64)   NOT NULL DEFAULT '',
           status        VARCHAR(30)   NOT NULL DEFAULT 'queued',
           conversation  VARCHAR(64)   NOT NULL DEFAULT '',
           sandbox       VARCHAR(64)   NOT NULL DEFAULT '',
           start_task    VARCHAR(64)   NOT NULL DEFAULT '',
           result        TEXT          NOT NULL DEFAULT '',
           events        INTEGER       NOT NULL DEFAULT 0,
           created_by    VARCHAR(64)   NOT NULL DEFAULT '',
           created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_coding_created ON coding_jobs(created_at);",
    );
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("MbilinyiTech-Coding/1.0 (+https://mbilinyitech.co.tz)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// The platform's tool catalogue, as the tool names OpenHands understands. The
/// catalogue is the source of truth for what an agent may be given, so a grant
/// made in Admin -> Agents carries over to the coding sandbox unchanged.
fn tool_name_of(code: &str) -> &'static str {
    match code {
        "terminal" | "code_execution" => "terminal",
        "file_ops" => "file_editor",
        "todo" => "task_tracker",
        "browser" | "web" => "browser",
        _ => "terminal",
    }
}

/// The tools a job's agent may use: its own grant, mapped to OpenHands tool
/// names. An agent with no grant gets the working set a coder needs, because a
/// coding job with no tools cannot do anything.
fn tools_for(agent_id: &str) -> Vec<J> {
    let mut names: Vec<&'static str> = Vec::new();
    if let Some(a) = db::row(
        "SELECT tools FROM agents WHERE id = ?",
        &[arg(json!(agent_id))],
    ) {
        for code in crate::tools::granted(&util::s(a.get("tools").unwrap_or(&J::Null))) {
            let n = tool_name_of(&code);
            if !names.contains(&n) {
                names.push(n);
            }
        }
    }
    if names.is_empty() {
        names = vec!["terminal", "file_editor", "task_tracker"];
    }
    names.into_iter().map(|n| json!({ "name": n })).collect()
}

fn job_json(r: &db::Row) -> J {
    json!({
        "id": util::s(r.get("id").unwrap_or(&J::Null)),
        "title": util::s(r.get("title").unwrap_or(&J::Null)),
        "task": util::s(r.get("task").unwrap_or(&J::Null)),
        "repository": util::s(r.get("repository").unwrap_or(&J::Null)),
        "branch": util::s(r.get("branch").unwrap_or(&J::Null)),
        "model": util::s(r.get("model").unwrap_or(&J::Null)),
        "agentId": util::s(r.get("agent_id").unwrap_or(&J::Null)),
        "status": util::s(r.get("status").unwrap_or(&J::Null)),
        "conversation": util::s(r.get("conversation").unwrap_or(&J::Null)),
        "sandbox": util::s(r.get("sandbox").unwrap_or(&J::Null)),
        "result": util::s(r.get("result").unwrap_or(&J::Null)),
        "events": util::i(r.get("events").unwrap_or(&J::Null)),
        "createdAt": util::s(r.get("created_at").unwrap_or(&J::Null)),
        "updatedAt": util::s(r.get("updated_at").unwrap_or(&J::Null)),
    })
}

/// OpenHands rejects a `limit` above 100 with HTTP 422, so the events read
/// uses exactly this and no more.
const EVENTS_LIMIT: u32 = 100;

/// The events-search URL for a conversation. Kept as its own function so the
/// limit it sends is checked by a test rather than only at runtime.
fn events_url(base: &str, conversation: &str) -> String {
    format!(
        "{}/api/v1/conversation/{}/events/search?limit={EVENTS_LIMIT}",
        base.trim_end_matches('/'),
        conversation
    )
}

/// The UI URL for a conversation, so the CEO can open it in OpenHands itself.
fn ui_url(conversation: &str) -> String {
    canvas_url(&base_url(), conversation)
}

/// The pure form, so the shape can be checked without a database.
fn canvas_url(base: &str, conversation: &str) -> String {
    if conversation.is_empty() {
        return String::new();
    }
    format!(
        "{}/canvas/conversations/{}",
        base.trim_end_matches('/'),
        conversation
    )
}

/// A request to start a coding job, from the panel or from the agent runtime.
pub struct StartRequest {
    pub title: String,
    pub task: String,
    pub repository: String,
    pub branch: String,
    pub agent_id: String,
    pub created_by: String,
}

/// Start one real OpenHands conversation and record it as a job.
///
/// This is the single place a job begins, so the panel and an agent that holds
/// the `mbilinyi_coding` tool go through exactly the same path — the same key,
/// the same model, the same grant check.
pub async fn start_job(req: StartRequest) -> Result<J, String> {
    let key = api_key();
    if key.is_empty() {
        return Err("No OpenHands API key is set. Add one in Admin -> Mbilinyi Coding.".into());
    }
    if req.task.trim().is_empty() {
        return Err("Describe the coding job.".into());
    }
    let title = if req.title.trim().is_empty() {
        req.task.chars().take(80).collect::<String>()
    } else {
        req.title.clone()
    };
    let model = model();

    let id = util::new_id("JOB-");
    let _ = db::run(
        "INSERT INTO coding_jobs (id, title, task, repository, branch, model, agent_id, status, created_by, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(title)), arg(json!(req.task)),
            arg(json!(req.repository)), arg(json!(req.branch)), arg(json!(model)),
            arg(json!(req.agent_id)), arg(json!("starting")),
            arg(json!(req.created_by)),
            arg(json!(util::now())), arg(json!(util::now())),
        ],
    );

    let mut payload = json!({
        "initial_message": {
            "role": "user",
            "content": [{ "type": "text", "text": req.task }],
        },
        "title": title,
        "run": true,
    });
    if !req.repository.trim().is_empty() {
        payload["selected_repository"] = json!(req.repository.trim());
    }
    if !req.branch.trim().is_empty() {
        payload["selected_branch"] = json!(req.branch.trim());
    }
    if !model.is_empty() {
        payload["llm_model"] = json!(model);
    }
    // The agent's granted tools, so the platform's grant check is the same one
    // that governs the sandbox.
    let tools = tools_for(&req.agent_id);
    if !tools.is_empty() {
        payload["agent"] = json!({ "tools": tools });
    }

    let res = client()
        .post(format!("{}/api/v1/app-conversations", base_url()))
        .bearer_auth(&key)
        .json(&payload)
        .send()
        .await;
    let body: J = match res {
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let v: J = serde_json::from_str(&text).unwrap_or(json!({ "raw": text }));
            if !status.is_success() {
                let detail = util::s(v.get("detail").unwrap_or(&J::Null));
                let msg = if detail.is_empty() {
                    format!("OpenHands refused the job (HTTP {}).", status.as_u16())
                } else {
                    detail
                };
                let _ = db::run(
                    "UPDATE coding_jobs SET status='failed', result=?, updated_at=? WHERE id=?",
                    &[arg(json!(msg)), arg(json!(util::now())), arg(json!(id))],
                );
                return Err(msg);
            }
            v
        }
        Err(e) => {
            let msg = format!("The OpenHands API could not be reached: {e}");
            let _ = db::run(
                "UPDATE coding_jobs SET status='failed', result=?, updated_at=? WHERE id=?",
                &[arg(json!(msg)), arg(json!(util::now())), arg(json!(id))],
            );
            return Err(msg);
        }
    };

    // The response is a start-task. It may carry the conversation id at once,
    // or it may not — then `poll` resolves it from the start-task. The row is
    // updated either way so the panel can follow the job.
    let start_id = util::s(body.get("id").unwrap_or(&J::Null));
    let conversation = util::s(body.get("app_conversation_id").unwrap_or(&J::Null));
    let _ = db::run(
        "UPDATE coding_jobs SET status='running', conversation=?, start_task=?, updated_at=? WHERE id=?",
        &[arg(json!(conversation)), arg(json!(start_id)), arg(json!(util::now())), arg(json!(id))],
    );
    let job = db::row("SELECT * FROM coding_jobs WHERE id = ?", &[arg(json!(id))]);
    Ok(json!({
        "job": job.as_ref().map(job_json).unwrap_or(J::Null),
        "jobId": id,
        "startTask": start_id,
        "conversation": conversation,
        "uiUrl": canvas_url(&base_url(), &conversation),
        "model": model,
        "tools": tools.len(),
    }))
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        // What is configured, and what the panel needs to render its form.
        "status" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = api_key();
            let agents = db::rows(
                "SELECT id, name, role, department FROM agents WHERE status != 'paused' ORDER BY name LIMIT 120",
                &[],
            );
            ok(json!({
                "configured": !key.is_empty(),
                "keyHint": if key.is_empty() { String::new() } else {
                    let tail: String = key.chars().rev().take(4).collect::<String>().chars().rev().collect();
                    format!("…{tail}")
                },
                "baseUrl": base_url(),
                "model": model(),
                "llmConfigured": crate::llm::ready(),
                "llmProvider": crate::llm::provider_name(),
                "uiUrl": base_url(),
                "agents": agents.iter().map(|a| json!({
                    "id": util::s(a.get("id").unwrap_or(&J::Null)),
                    "name": util::s(a.get("name").unwrap_or(&J::Null)),
                    "role": util::s(a.get("role").unwrap_or(&J::Null)),
                    "department": util::s(a.get("department").unwrap_or(&J::Null)),
                })).collect::<Vec<_>>(),
            }))
        }

        // Save the key, base URL and model. The key is never returned.
        "save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let mut pairs: Vec<(String, String)> = Vec::new();
            let key = c.p("apiKey");
            if !key.trim().is_empty() {
                pairs.push(("coding_api_key".into(), key.trim().to_string()));
            }
            pairs.push(("coding_base_url".into(), c.p("baseUrl").trim().to_string()));
            pairs.push(("coding_model".into(), c.p("model").trim().to_string()));
            settings::set_many(&pairs);
            ok(json!({ "saved": true, "configured": configured() }))
        }

        "clear_key" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            settings::set("coding_api_key", "");
            ok(json!({ "cleared": true }))
        }

        // Prove the key works by asking the API who we are. Cheap, no sandbox.
        "test" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = api_key();
            if key.is_empty() {
                return ok(json!({ "connected": false, "detail": "No API key is set." }));
            }
            let res = client()
                .get(format!("{}/api/v1/users/me", base_url()))
                .bearer_auth(&key)
                .send()
                .await;
            match res {
                Ok(r) => {
                    let status = r.status();
                    let body: J = r.json().await.unwrap_or(J::Null);
                    if status.is_success() {
                        ok(json!({
                            "connected": true,
                            "email": util::s(body.get("email").unwrap_or(&J::Null)),
                            "detail": "The OpenHands API answered. Mbilinyi Coding is ready.",
                        }))
                    } else {
                        ok(
                            json!({ "connected": false, "detail": format!("The API refused the key (HTTP {}).", status.as_u16()) }),
                        )
                    }
                }
                Err(e) => ok(
                    json!({ "connected": false, "detail": format!("The API could not be reached: {e}") }),
                ),
            }
        }

        // Start a real coding job: create the conversation, then follow it.
        "start" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let admin = match c.require_admin() {
                Ok(a) => a,
                Err(e) => return e,
            };
            let task = c.p("task");
            if task.trim().is_empty() {
                return bad("Describe the coding job.");
            }
            let created_by = util::s(admin.get("id").unwrap_or(&J::Null));
            match start_job(StartRequest {
                title: c.p("title"),
                task,
                repository: c.p("repository"),
                branch: c.p("branch"),
                agent_id: c.p("agentId"),
                created_by,
            })
            .await
            {
                Ok(v) => ok(v),
                Err(msg) => bad(&msg),
            }
        }

        // Follow a job: resolve its conversation if it was not known at start,
        // read the events, and store the agent's final answer.
        "poll" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let Some(job) = db::row("SELECT * FROM coding_jobs WHERE id = ?", &[arg(json!(id))])
            else {
                return bad("No such job.");
            };
            let key = api_key();
            if key.is_empty() {
                return bad("No OpenHands API key is set.");
            }
            let mut conversation = util::s(job.get("conversation").unwrap_or(&J::Null));
            let mut sandbox = util::s(job.get("sandbox").unwrap_or(&J::Null));

            // If the create call did not know the conversation yet, resolve it
            // from the start-task, which is the exact handle for this job.
            if conversation.is_empty() {
                let start_id = util::s(job.get("start_task").unwrap_or(&J::Null));
                if !start_id.is_empty() {
                    if let Ok(r) = client()
                        .get(format!(
                            "{}/api/v1/app-conversations/start-tasks?ids={}",
                            base_url(),
                            start_id
                        ))
                        .bearer_auth(&key)
                        .send()
                        .await
                    {
                        if let Ok(v) = r.json::<J>().await {
                            let items = v
                                .get("items")
                                .and_then(|i| i.as_array())
                                .cloned()
                                .unwrap_or_else(|| {
                                    if v.is_array() {
                                        v.as_array().cloned().unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    }
                                });
                            if let Some(it) = items.first() {
                                conversation =
                                    util::s(it.get("app_conversation_id").unwrap_or(&J::Null));
                                if sandbox.is_empty() {
                                    sandbox = util::s(it.get("sandbox_id").unwrap_or(&J::Null));
                                }
                            }
                        }
                    }
                }
            }

            if conversation.is_empty() {
                return ok(json!({ "job": job_json(&job), "pending": true }));
            }

            let events = client()
                .get(events_url(&base_url(), &conversation))
                .bearer_auth(&key)
                .send()
                .await;
            let (count, answer, status) = match events {
                Ok(r) if r.status().is_success() => {
                    let v: J = r.json().await.unwrap_or(json!({}));
                    let items = v
                        .get("items")
                        .and_then(|i| i.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let mut answer = String::new();
                    let mut finished = false;
                    for e in items.iter() {
                        let kind = util::s(e.get("kind").unwrap_or(&J::Null));
                        if kind == "MessageEvent"
                            && util::s(e.get("source").unwrap_or(&J::Null)) == "agent"
                        {
                            let text = text_of(e.get("llm_message").unwrap_or(&J::Null));
                            if !text.trim().is_empty() {
                                answer = text;
                            }
                        } else if kind == "ActionEvent"
                            && util::s(e.get("tool_name").unwrap_or(&J::Null)) == "finish"
                        {
                            finished = true;
                        }
                    }
                    let status = if finished || !answer.trim().is_empty() {
                        "done".to_string()
                    } else {
                        "running".to_string()
                    };
                    (items.len() as i64, answer, status)
                }
                Ok(_) | Err(_) => (
                    util::i(job.get("events").unwrap_or(&J::Null)),
                    util::s(job.get("result").unwrap_or(&J::Null)),
                    util::s(job.get("status").unwrap_or(&J::Null)),
                ),
            };

            let _ = db::run(
                "UPDATE coding_jobs SET conversation=?, sandbox=?, status=?, result=?, events=?, updated_at=? WHERE id=?",
                &[
                    arg(json!(conversation)), arg(json!(sandbox)), arg(json!(status)),
                    arg(json!(answer)), arg(json!(count)), arg(json!(util::now())), arg(json!(id)),
                ],
            );
            let job =
                db::row("SELECT * FROM coding_jobs WHERE id = ?", &[arg(json!(id))]).unwrap_or(job);
            ok(json!({ "job": job_json(&job), "uiUrl": ui_url(&conversation) }))
        }

        "jobs" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows(
                "SELECT * FROM coding_jobs ORDER BY created_at DESC LIMIT 50",
                &[],
            );
            ok(json!({ "jobs": rows.iter().map(job_json).collect::<Vec<_>>() }))
        }

        _ => bad("Choose a coding action: status, save, clear_key, test, start, poll or jobs."),
    }
}

/// The text of a MessageEvent, which arrives as a list of content parts.
fn text_of(msg: &J) -> String {
    if let Some(parts) = msg.get("content").and_then(|c| c.as_array()) {
        let joined = parts
            .iter()
            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        if !joined.trim().is_empty() {
            return joined;
        }
    }
    msg.get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_map_to_openhands_tools() {
        assert_eq!(tool_name_of("terminal"), "terminal");
        assert_eq!(tool_name_of("code_execution"), "terminal");
        assert_eq!(tool_name_of("file_ops"), "file_editor");
        assert_eq!(tool_name_of("todo"), "task_tracker");
        assert_eq!(tool_name_of("browser"), "browser");
        // An unknown code still yields a working tool rather than nothing.
        assert_eq!(tool_name_of("not_a_tool"), "terminal");
    }

    #[test]
    fn message_text_is_read_from_content_parts() {
        let msg = json!({
            "role": "assistant",
            "content": [{ "type": "text", "text": "line one" }, { "type": "text", "text": "line two" }]
        });
        assert_eq!(text_of(&msg), "line one\nline two");
    }

    #[test]
    fn message_text_handles_a_plain_string() {
        let msg = json!({ "role": "assistant", "content": "just text" });
        assert_eq!(text_of(&msg), "just text");
    }

    #[test]
    fn the_ui_url_points_at_the_conversation() {
        let u = canvas_url("https://app.all-hands.dev", "abc123");
        assert_eq!(u, "https://app.all-hands.dev/canvas/conversations/abc123");
        // A trailing slash on the base must not double up.
        let u2 = canvas_url("https://app.all-hands.dev/", "abc123");
        assert_eq!(u2, "https://app.all-hands.dev/canvas/conversations/abc123");
    }

    #[test]
    fn an_empty_conversation_has_no_ui_url() {
        assert_eq!(canvas_url("https://app.all-hands.dev", ""), "");
    }

    #[test]
    fn the_events_url_asks_for_no_more_than_openhands_allows() {
        // OpenHands answers HTTP 422 above 100, so a larger value silently
        // reads zero events and every job looks stuck forever.
        let u = events_url("https://app.all-hands.dev", "conv123");
        assert_eq!(
            u,
            "https://app.all-hands.dev/api/v1/conversation/conv123/events/search?limit=100"
        );
    }
}
