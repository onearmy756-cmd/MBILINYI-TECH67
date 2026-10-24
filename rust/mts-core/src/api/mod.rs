//! Endpoint dispatch.
//!
//! One entry point per `/api/<name>` path, matching the PHP filenames so the
//! frontend needs no change. Each returns the `{ok,data}` / `{ok,error}`
//! envelope from `util`.

pub mod admin;
pub mod auth;
pub mod ctx;
pub mod data;
pub mod map;
pub mod oauth;
pub mod pay;
pub mod reports;

use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use crate::api::ctx::Ctx;
use serde_json::json;
use crate::util::{bad, not_found, server_error};

pub async fn dispatch(name: &str, req: Request<Body>) -> Response {
    let c = Ctx::build(req).await;
    let result = match name {
        "auth" => auth::handle(&c).await,
        "requests" => data::requests(&c).await,
        "quotes" => data::quotes(&c).await,
        "contracts" => data::contracts(&c).await,
        "invoices" => data::invoices(&c).await,
        "tickets" => data::tickets(&c).await,
        "news" => data::news(&c).await,
        "portfolio" => data::portfolio(&c).await,
        "testimonials" => data::testimonials(&c).await,
        "team" => data::team(&c).await,
        "track" => data::track(&c).await,
        "dashboard" => data::dashboard(&c).await,
        "settings" => admin::settings(&c).await,
        "users" => admin::users(&c).await,
        "payments" => pay::handle(&c).await,
        "oauth" => oauth::handle(&c).await,
        "reports" => reports::handle(&c).await,
        "ollama" => data::ollama(&c).await,
        // The modules added for the MTECHOS merge: staff, hiring, the agent
        // company, the online office and the security posture.
        "wafanyakazi" | "employees" => crate::wafanyakazi::handle(&c).await,
        "ajira" | "jobs" => crate::ajira::handle(&c).await,
        "agents" => crate::agents::handle(&c).await,
        "engine" => crate::engine_api::handle(&c).await,
        "llm" => crate::llm_api::handle(&c).await,
        "tools" => crate::tools::handle(&c).await,
        "ofisi" | "office" => crate::ofisi::handle(&c).await,
        "security" => crate::security::handle(&c).await,
        _ => return not_found("Unknown endpoint."),
    };
    result
}

/// Run a handler and turn a panic-free error into the standard 500, so a bad
/// row can never take the server down.
pub fn guard<F: FnOnce() -> Response>(f: F) -> Response {
    f()
}

/// A tiny helper for the common "read a required field or 400" case.
pub fn need(c: &Ctx, key: &str, msg: &str) -> Result<String, Response> {
    let v = c.p(key);
    if v.trim().is_empty() {
        Err(bad(msg))
    } else {
        Ok(v)
    }
}

/// Wrap a fallible body so a database surprise becomes a clean 500.
pub fn attempt<F: FnOnce() -> Result<Response, String>>(f: F) -> Response {
    match f() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[api] {e}");
            server_error()
        }
    }
}

/// The `ok` payload for a list endpoint, keeping the key names the frontend
/// reads (`news`, `requests`, `quotes`, ...).
pub fn list(key: &str, items: Vec<serde_json::Value>) -> Response {
    crate::util::ok(json!({ key: items }))
}
