//! Sessions and CSRF.
//!
//! PHP stored sessions in `storage/sessions`; this keeps the same cookie name
//! and the same guarantees (HttpOnly, SameSite=Lax, id regenerated on sign-in)
//! but holds them in a table, so the session survives a restart and the admin
//! can see who is signed in.

use crate::db;
use crate::util;
use axum::http::HeaderMap;
use rusqlite::types::Value;
use serde_json::json;

pub const COOKIE: &str = "mts_session";

pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub csrf: String,
}

pub fn init_table() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS sessions (
           id         VARCHAR(64) NOT NULL,
           user_id    VARCHAR(64) NULL,
           csrf       VARCHAR(64) NOT NULL,
           created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
           seen_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(&format!("{name}=")) {
            return Some(rest.to_string());
        }
    }
    None
}

/// Load the caller's session, creating one when absent so that a first-time
/// visitor already has a CSRF token to send with their first POST.
pub fn load(headers: &HeaderMap) -> Session {
    if let Some(id) = cookie_value(headers, COOKIE) {
        if let Some(r) = db::row(
            "SELECT id, user_id, csrf FROM sessions WHERE id = ?",
            &db::args(vec![json!(id)]),
        ) {
            let _ = db::run(
                "UPDATE sessions SET seen_at = ? WHERE id = ?",
                &db::args(vec![json!(util::now()), json!(id)]),
            );
            return Session {
                id,
                user_id: r.get("user_id").and_then(|v| v.as_str()).map(str::to_string),
                csrf: r.get("csrf").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            };
        }
    }
    new_session()
}

fn new_session() -> Session {
    let id = util::token_hex(32);
    let csrf = util::token_hex(32);
    let _ = db::run(
        "INSERT INTO sessions (id, user_id, csrf, created_at, seen_at) VALUES (?, NULL, ?, ?, ?)",
        &db::args(vec![json!(id), json!(csrf), json!(util::now()), json!(util::now())]),
    );
    Session { id, user_id: None, csrf }
}

/// Sign in: a fresh session id and a fresh CSRF token, exactly as
/// `login_user()` did, so a token captured before sign-in cannot be replayed.
/// Sign a user in from an API handler, returning the session id so the caller
/// can set the cookie. Used when a guest submission creates an account.
pub fn login_user(_c: &crate::api::ctx::Ctx, user: &db::Row) -> Session {
    let id = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
    login(id)
}

pub fn login(user_id: &str) -> Session {
    let id = util::token_hex(32);
    let csrf = util::token_hex(32);
    let _ = db::run(
        "INSERT INTO sessions (id, user_id, csrf, created_at, seen_at) VALUES (?, ?, ?, ?, ?)",
        &db::args(vec![
            json!(id),
            json!(user_id),
            json!(csrf),
            json!(util::now()),
            json!(util::now()),
        ]),
    );
    Session { id, user_id: Some(user_id.to_string()), csrf }
}

pub fn logout(id: &str) {
    let _ = db::run("DELETE FROM sessions WHERE id = ?", &db::args(vec![json!(id)]));
}

pub fn set_cookie(id: &str) -> String {
    format!("{COOKIE}={id}; Path=/; HttpOnly; SameSite=Lax")
}

pub fn clear_cookie() -> String {
    format!("{COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// The user row for a session, or None. A suspended account is refused here so
/// every caller inherits the rule.
pub fn current_user(sess: &Session) -> Option<db::Row> {
    let uid = sess.user_id.as_ref()?;
    let u = db::row("SELECT * FROM users WHERE id = ?", &db::args(vec![json!(uid)]))?;
    let status = u.get("status").and_then(|v| v.as_str()).unwrap_or("Active");
    if status != "Active" {
        return None;
    }
    Some(u)
}

/// CSRF check for POSTs. The token lives on the session, so a request without a
/// matching `X-CSRF-Token` is refused even if it carries a valid cookie.
pub fn check_csrf(sess: &Session, headers: &HeaderMap) -> bool {
    let Some(got) = headers.get("x-csrf-token").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    !sess.csrf.is_empty() && got == sess.csrf
}

/// Purge sessions untouched for a fortnight, so the table cannot grow forever.
pub fn sweep() {
    let cutoff = (chrono::Local::now() - chrono::Duration::days(14))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let _ = db::run("DELETE FROM sessions WHERE seen_at < ?", &db::args(vec![json!(cutoff)]));
}

/// Args helper re-exported so modules need only import `session`.
pub fn v(x: impl Into<String>) -> Value {
    Value::Text(x.into())
}
