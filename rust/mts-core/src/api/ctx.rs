//! The per-request context.
//!
//! `action_name()` and `param()` in the PHP app resolved a field by looking in
//! the JSON body first, then the query string. That order is what lets `app.js`
//! send GETs with an `action` in the query and POSTs with it in the body, so it
//! is preserved here rather than simplified.

use crate::db;
use crate::session::{self, Session};
use crate::util;
use axum::body::Body;
use axum::http::{HeaderMap, Request};
use serde_json::Value as J;
use std::collections::HashMap;

pub struct Ctx {
    pub session: Session,
    pub body: J,
    pub query: HashMap<String, String>,
    pub headers: HeaderMap,
}

impl Ctx {
    pub async fn build(req: Request<Body>) -> Ctx {
        let query = parse_query(req.uri().query().unwrap_or(""));
        let headers = req.headers().clone();
        let session = session::load(&headers);
        let body = read_body(req).await;
        Ctx { session, body, query, headers }
    }

    /// Body first, then query — the PHP `param()` order.
    pub fn p(&self, key: &str) -> String {
        if let Some(v) = self.body.get(key) {
            if !v.is_null() {
                let s = util::s(v);
                if !s.is_empty() {
                    return s;
                }
            }
        }
        self.query.get(key).cloned().unwrap_or_default()
    }

    /// Like `p`, but returns the raw JSON value, for arrays and objects.
    pub fn pv(&self, key: &str) -> J {
        if let Some(v) = self.body.get(key) {
            if !v.is_null() {
                return v.clone();
            }
        }
        match self.query.get(key) {
            Some(s) => J::String(s.clone()),
            None => J::Null,
        }
    }

    pub fn action(&self) -> String {
        self.p("action")
    }

    pub fn user(&self) -> Option<db::Row> {
        session::current_user(&self.session)
    }

    /// The authenticated user, or the error the frontend expects.
    pub fn require_user(&self) -> Result<db::Row, axum::response::Response> {
        match self.user() {
            Some(u) => Ok(u),
            // A suspended account is signed in but refused, and says so.
            None => {
                if self.session.user_id.is_some() {
                    Err(util::forbidden("This account is suspended. Contact 0796 752 645."))
                } else {
                    Err(util::unauth())
                }
            }
        }
    }

    pub fn require_admin(&self) -> Result<db::Row, axum::response::Response> {
        let u = self.require_user()?;
        if u.get("role").and_then(|v| v.as_str()) != Some("admin") {
            return Err(util::forbidden("Admin access required."));
        }
        Ok(u)
    }

    pub fn require_csrf(&self) -> Result<(), axum::response::Response> {
        if session::check_csrf(&self.session, &self.headers) {
            Ok(())
        } else {
            Err(util::forbidden(
                "Invalid or missing CSRF token. Reload the page and try again.",
            ))
        }
    }
}

fn parse_query(q: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in q.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        out.insert(urldecode(k), urldecode(v));
    }
    out
}

fn urldecode(s: &str) -> String {
    let plus = s.replace('+', " ");
    percent_encoding::percent_decode_str(&plus)
        .decode_utf8_lossy()
        .to_string()
}

async fn read_body(req: Request<Body>) -> J {
    let bytes = match axum::body::to_bytes(req.into_body(), 8 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return J::Null,
    };
    if bytes.is_empty() {
        return J::Null;
    }
    // JSON is what app.js sends; a form-encoded body is accepted too so a plain
    // HTML form still works.
    if let Ok(v) = serde_json::from_slice::<J>(&bytes) {
        return v;
    }
    let text = String::from_utf8_lossy(&bytes).to_string();
    let map = parse_query(text.trim_start_matches('?'));
    if map.is_empty() {
        J::Null
    } else {
        J::Object(map.into_iter().map(|(k, v)| (k, J::String(v))).collect())
    }
}

/// An authenticated (or not) caller, plus the JSON body.
pub async fn ctx(req: Request<Body>) -> Ctx {
    Ctx::build(req).await
}
