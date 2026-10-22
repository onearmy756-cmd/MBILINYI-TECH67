//! The JSON envelope, CSRF, IDs and small helpers the frontend depends on.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rand::Rng;
use serde_json::{json, Value};

/// `{"ok":true,"data":{...}}` — the shape `apiHandle` in app.js unwraps.
pub fn ok(data: Value) -> Response {
    let body = json!({ "ok": true, "data": data });
    (
        StatusCode::OK,
        [
            ("Content-Type", "application/json; charset=utf-8"),
            ("Cache-Control", "no-store"),
        ],
        body.to_string(),
    )
        .into_response()
}

/// `{"ok":false,"error":"..."}` — the shape `apiHandle` throws on.
pub fn err(msg: &str, code: u16) -> Response {
    let body = json!({ "ok": false, "error": msg });
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);
    (
        status,
        [
            ("Content-Type", "application/json; charset=utf-8"),
            ("Cache-Control", "no-store"),
        ],
        body.to_string(),
    )
        .into_response()
}

pub fn bad(msg: &str) -> Response {
    err(msg, 400)
}

pub fn unauth() -> Response {
    err("You need to sign in first.", 401)
}

pub fn forbidden(msg: &str) -> Response {
    err(msg, 403)
}

pub fn not_found(msg: &str) -> Response {
    err(msg, 404)
}

pub fn server_error() -> Response {
    err("Server error. Please try again.", 500)
}

/// `new_id('U-')` in PHP: prefix + six uppercase hex characters.
pub fn new_id(prefix: &str) -> String {
    let n: u32 = rand::thread_rng().gen_range(0..0xFFFFFF);
    format!("{prefix}{n:06X}")
}

pub fn token_hex(bytes: usize) -> String {
    let mut b = vec![0u8; bytes];
    rand::thread_rng().fill(&mut b[..]);
    hex::encode(b)
}

/// MySQL DATETIME text, which is what every column stores.
pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// ISO-8601 for the frontend; the columns hold local-time DATETIME text.
pub fn iso(ts: Option<&str>) -> String {
    let Some(t) = ts else { return String::new() };
    if t.is_empty() {
        return String::new();
    }
    match chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
        Err(_) => t.to_string(),
    }
}

pub fn s(v: &Value) -> String {
    match v {
        Value::String(x) => x.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn f(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(x) => x.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

pub fn i(v: &Value) -> i64 {
    match v {
        Value::Number(n) => n.as_i64().unwrap_or(0),
        Value::String(x) => x.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// Escape for HTML text and attribute contexts.
pub fn h(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

/// A grouped number without a currency, for messages that name the currency
/// separately.
pub fn thousands(n: f64) -> String {
    let s = tzs(n);
    s.trim_start_matches("TZS ").to_string()
}

pub fn tzs(n: f64) -> String {
    let whole = n.round() as i64;
    let mut digits = whole.abs().to_string();
    let mut out = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        out = format!(",{}{}", &digits[split..], out);
        digits.truncate(split);
    }
    let sign = if whole < 0 { "-" } else { "" };
    format!("TZS {sign}{digits}{out}")
}
