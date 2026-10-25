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
            // One retry on a transport error. The first outbound call from a
            // cold container can lose a DNS or connect race, and a false
            // "cannot be reached" on the Test button sends the CEO chasing a
            // fault that is not there. A read-only GET is safe to repeat.
            let probe = || {
                client()
                    .get(format!("{}/api/v1/users/me", base_url()))
                    .bearer_auth(&key)
                    .send()
            };
            let mut res = probe().await;
            if res.is_err() {
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                res = probe().await;
            }
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
            let (conversation, sandbox) = resolve_conversation(&job, &key).await;

            if conversation.is_empty() {
                return ok(json!({ "job": job_json(&job), "pending": true }));
            }

            let items = fetch_events(&key, &conversation).await;
            let (count, answer, status) = match items {
                Some(items) => read_events(&items),
                None => (
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

        // The real OpenHands transcript, event by event. This is the agent's own
        // record — the messages it wrote, the tools it called and what they
        // returned — not a summary the panel invented. The panel renders it as
        // the conversation view, and the same id opens the full OpenHands UI.
        "transcript" => {
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
            let (conversation, sandbox) = resolve_conversation(&job, &key).await;
            if conversation.is_empty() {
                return ok(json!({ "conversation": "", "pending": true, "events": [] }));
            }
            let items = fetch_events(&key, &conversation).await.unwrap_or_default();
            let _ = db::run(
                "UPDATE coding_jobs SET conversation=?, sandbox=?, updated_at=? WHERE id=?",
                &[
                    arg(json!(conversation)),
                    arg(json!(sandbox)),
                    arg(json!(util::now())),
                    arg(json!(id)),
                ],
            );
            ok(json!({
                "conversation": conversation,
                "sandbox": sandbox,
                "uiUrl": ui_url(&conversation),
                "events": items.iter().map(transcript_entry).collect::<Vec<_>>(),
            }))
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

        _ => bad(
            "Choose a coding action: status, save, clear_key, test, start, poll, transcript or jobs.",
        ),
    }
}

/// Resolve a job's conversation id and sandbox id.
///
/// The create call may return the conversation at once, or only a start-task.
/// The start-task is the exact handle for this job, so it is the fallback that
/// makes an older job still readable.
async fn resolve_conversation(job: &db::Row, key: &str) -> (String, String) {
    let conversation = util::s(job.get("conversation").unwrap_or(&J::Null));
    let sandbox = util::s(job.get("sandbox").unwrap_or(&J::Null));
    if !conversation.is_empty() {
        return (conversation, sandbox);
    }
    let start_id = util::s(job.get("start_task").unwrap_or(&J::Null));
    if start_id.is_empty() {
        return (String::new(), sandbox);
    }
    let Ok(r) = client()
        .get(format!(
            "{}/api/v1/app-conversations/start-tasks?ids={}",
            base_url(),
            start_id
        ))
        .bearer_auth(key)
        .send()
        .await
    else {
        return (String::new(), sandbox);
    };
    let Ok(v) = r.json::<J>().await else {
        return (String::new(), sandbox);
    };
    let items = v
        .get("items")
        .and_then(|i| i.as_array())
        .cloned()
        .unwrap_or_else(|| v.as_array().cloned().unwrap_or_default());
    match items.first() {
        Some(it) => {
            let c = util::s(it.get("app_conversation_id").unwrap_or(&J::Null));
            let s = if sandbox.is_empty() {
                util::s(it.get("sandbox_id").unwrap_or(&J::Null))
            } else {
                sandbox
            };
            (c, s)
        }
        None => (String::new(), sandbox),
    }
}

/// Read a conversation's raw events. `None` means the call failed, so the
/// caller keeps the last known state rather than blanking the job.
async fn fetch_events(key: &str, conversation: &str) -> Option<Vec<J>> {
    let r = client()
        .get(events_url(&base_url(), conversation))
        .bearer_auth(key)
        .send()
        .await
        .ok()?;
    if !r.status().is_success() {
        return None;
    }
    let v: J = r.json().await.ok()?;
    Some(
        v.get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default(),
    )
}

/// One event, shaped for the panel's conversation view.
///
/// The panel needs to tell a message from a tool call and to show each one in
/// the right voice, so the kind and the actor are carried through. An event we
/// do not recognise still becomes a readable line rather than a blank row.
fn transcript_entry(e: &J) -> J {
    let kind = util::s(e.get("kind").unwrap_or(&J::Null));
    let source = util::s(e.get("source").unwrap_or(&J::Null));
    let tool = util::s(e.get("tool_name").unwrap_or(&J::Null));
    let action = util::s(e.get("action").unwrap_or(&J::Null));
    let (role, text) = if kind == "MessageEvent" {
        let t = text_of(e.get("llm_message").unwrap_or(&J::Null));
        let role = if source == "user" { "user" } else { "agent" };
        (role, t)
    } else if kind == "ActionEvent" {
        let args = e
            .get("tool_call")
            .and_then(|t| t.get("arguments"))
            .cloned()
            .unwrap_or(J::Null);
        let detail = util::s(&args);
        let name = if tool.is_empty() {
            action
        } else {
            tool.clone()
        };
        ("tool", format!("{name}({detail})"))
    } else if kind == "ObservationEvent" {
        let content = e
            .get("observation")
            .and_then(|o| o.get("content"))
            .cloned()
            .unwrap_or(J::Null);
        let out = if content.is_array() {
            text_of(&json!({ "content": content }))
        } else {
            util::s(&content)
        };
        ("tool", out)
    } else if kind == "ConversationErrorEvent" {
        let detail = util::s(e.get("detail").unwrap_or(&J::Null));
        let code = util::s(e.get("code").unwrap_or(&J::Null));
        (
            "error",
            if detail.trim().is_empty() {
                format!("The agent stopped with an error ({code}).")
            } else {
                detail
            },
        )
    } else if kind == "ConversationStateUpdateEvent" {
        // Internal bookkeeping (the whole agent state as JSON). It is noise in
        // a conversation view, so it is carried with no text and dropped.
        ("state", String::new())
    } else {
        ("other", String::new())
    };
    json!({
        "kind": kind,
        "role": role,
        "tool": tool,
        "text": text,
        "at": util::s(e.get("timestamp").unwrap_or(&J::Null)),
    })
}

/// Read a conversation's events into (event count, result text, status).
///
/// A ConversationErrorEvent that is not retryable is a terminal failure. It
/// must be surfaced: no agent message ever follows it, so a poll that only
/// looks for an answer would report "running" forever.
fn read_events(items: &[J]) -> (i64, String, String) {
    let mut answer = String::new();
    let mut finished = false;
    let mut failure = String::new();
    for e in items.iter() {
        let kind = util::s(e.get("kind").unwrap_or(&J::Null));
        if kind == "MessageEvent" && util::s(e.get("source").unwrap_or(&J::Null)) == "agent" {
            let text = text_of(e.get("llm_message").unwrap_or(&J::Null));
            if !text.trim().is_empty() {
                answer = text;
            }
        } else if kind == "ActionEvent"
            && util::s(e.get("tool_name").unwrap_or(&J::Null)) == "finish"
        {
            finished = true;
        } else if kind == "ConversationErrorEvent" {
            let retryable = e
                .get("classification")
                .and_then(|c| c.get("retryable"))
                .and_then(|r| r.as_bool())
                .unwrap_or(false);
            if !retryable {
                let detail = util::s(e.get("detail").unwrap_or(&J::Null));
                let code = util::s(e.get("code").unwrap_or(&J::Null));
                failure = if detail.trim().is_empty() {
                    format!("The agent stopped with an error ({code}).")
                } else {
                    detail
                };
            }
        }
    }
    let status = if !failure.is_empty() {
        "failed"
    } else if finished || !answer.trim().is_empty() {
        "done"
    } else {
        "running"
    };
    let result = if failure.is_empty() { answer } else { failure };
    (items.len() as i64, result, status.to_string())
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

    fn msg(text: &str) -> J {
        json!({"kind":"MessageEvent","source":"agent","llm_message":{"content":[{"type":"text","text":text}]}})
    }

    #[test]
    fn an_agent_message_is_the_result_and_finishes_the_job() {
        let (n, result, status) = read_events(&[msg("MTS-OK")]);
        assert_eq!(n, 1);
        assert_eq!(result, "MTS-OK");
        assert_eq!(status, "done");
    }

    #[test]
    fn a_non_retryable_error_ends_the_job_instead_of_hanging() {
        // The shape OpenHands returns when the LLM key is rejected: the run
        // never produces an agent message, so this event is the only terminal
        // signal. Ignoring it leaves the job "running" forever.
        let items = vec![
            msg("working"),
            json!({"kind":"ConversationErrorEvent","code":"LLMAuthenticationError",
                   "detail":"Incorrect API key provided",
                   "classification":{"kind":"auth","retryable":false}}),
        ];
        let (n, result, status) = read_events(&items);
        assert_eq!(n, 2);
        assert_eq!(status, "failed");
        assert!(result.contains("Incorrect API key"));
    }

    #[test]
    fn a_retryable_error_does_not_end_the_job() {
        let items = vec![json!({"kind":"ConversationErrorEvent","code":"Transient",
                                "classification":{"kind":"network","retryable":true}})];
        let (_, _, status) = read_events(&items);
        assert_eq!(status, "running");
    }

    #[test]
    fn a_finish_action_alone_finishes_the_job() {
        let items = vec![json!({"kind":"ActionEvent","tool_name":"finish"})];
        let (_, _, status) = read_events(&items);
        assert_eq!(status, "done");
    }

    #[test]
    fn a_transcript_keeps_the_agent_and_user_voices_apart() {
        let agent = transcript_entry(&json!({
            "kind":"MessageEvent","source":"agent",
            "llm_message":{"role":"assistant","content":[{"type":"text","text":"Done."}]}
        }));
        assert_eq!(agent["role"], "agent");
        assert_eq!(agent["text"], "Done.");
        let user = transcript_entry(&json!({
            "kind":"MessageEvent","source":"user",
            "llm_message":{"role":"user","content":[{"type":"text","text":"Fix it"}]}
        }));
        assert_eq!(user["role"], "user");
        assert_eq!(user["text"], "Fix it");
    }

    #[test]
    fn a_transcript_shows_a_tool_call_and_its_output() {
        let call = transcript_entry(&json!({
            "kind":"ActionEvent","tool_name":"terminal",
            "tool_call":{"arguments":{"command":"ls -la"}}
        }));
        assert_eq!(call["role"], "tool");
        assert!(call["text"].as_str().unwrap().starts_with("terminal("));
        assert!(call["text"].as_str().unwrap().contains("ls -la"));

        let out = transcript_entry(&json!({
            "kind":"ObservationEvent","tool_name":"terminal",
            "observation":{"content":[{"type":"text","text":"total 4"}]}
        }));
        assert_eq!(out["role"], "tool");
        assert_eq!(out["text"], "total 4");
    }

    #[test]
    fn a_transcript_surfaces_an_error_and_never_goes_blank() {
        let err = transcript_entry(&json!({
            "kind":"ConversationErrorEvent","code":"LLMAuthenticationError",
            "detail":"Incorrect API key provided"
        }));
        assert_eq!(err["role"], "error");
        assert_eq!(err["text"], "Incorrect API key provided");
        // An event the panel does not know still yields a shaped row.
        let unknown = transcript_entry(&json!({"kind":"SomethingNew"}));
        assert_eq!(unknown["role"], "other");
        // Internal state updates are noise and must carry no text, so the panel
        // can drop them instead of printing the whole agent state as JSON.
        let state = transcript_entry(&json!({
            "kind":"ConversationStateUpdateEvent","value":{"agent":{"big":"json"}}
        }));
        assert_eq!(state["role"], "state");
        assert_eq!(state["text"], "");
    }
}
