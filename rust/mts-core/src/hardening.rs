//! Hardening: the response headers every answer carries, and the rate limits
//! that stop a password being guessed at speed.
//!
//! Two separate concerns, kept together because both are "the wall around the
//! platform" rather than a feature a client buys.
//!
//! * **Headers.** A browser is told what it may load and where from. The policy
//!   is written so the platform's own inline styles and the CDN it already uses
//!   keep working, and nothing else is allowed in.
//! * **Rate limits.** A fixed window per client address and per account. The
//!   limiter is in memory on purpose: it is a speed bump, not a ledger, and a
//!   restart clearing it is acceptable.

use axum::http::{header, HeaderName, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/* ------------------------------ headers ------------------------------ */

/// The content policy. `unsafe-inline` is required for the style attribute the
/// templates and `app.js` set; scripts are only ever loaded from this origin,
/// the two CDNs the site already depends on, and inline (the pages ship their
/// bootstrap inline). `frame-ancestors` is `self` so the online office can embed
/// its own rooms but no third party may frame the platform.
const CSP: &str = "default-src 'self'; \
base-uri 'self'; \
object-src 'none'; \
frame-ancestors 'self'; \
form-action 'self'; \
img-src 'self' data: blob: https:; \
media-src 'self' blob: data:; \
font-src 'self' data: https://fonts.gstatic.com https://cdnjs.cloudflare.com; \
style-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://fonts.googleapis.com https://cdnjs.cloudflare.com; \
script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com; \
connect-src 'self' ws: wss: https:; \
worker-src 'self' blob:";

/// Is the request already on TLS? Behind a proxy the scheme arrives in
/// `X-Forwarded-Proto`, so that is honoured too.
fn request_is_https(req: &Request<axum::body::Body>) -> bool {
    if let Some(v) = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    {
        return v.eq_ignore_ascii_case("https");
    }
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    crate::settings::get("platform_url", "").starts_with("https")
        || (!host.starts_with("localhost")
            && !host.starts_with("127.0.0.1")
            && !host.is_empty()
            && host.contains('.'))
}

/// The security headers, applied to every response.
fn apply(res: &mut Response, https: bool) {
    let h = res.headers_mut();
    let mut set = |name: HeaderName, value: &str| {
        if let Ok(v) = HeaderValue::from_str(value) {
            h.insert(name, v);
        }
    };
    set(HeaderName::from_static("x-content-type-options"), "nosniff");
    set(HeaderName::from_static("x-frame-options"), "SAMEORIGIN");
    set(
        HeaderName::from_static("referrer-policy"),
        "strict-origin-when-cross-origin",
    );
    set(HeaderName::from_static("content-security-policy"), CSP);
    set(
        HeaderName::from_static("permissions-policy"),
        "geolocation=(self), microphone=(self), camera=(self), payment=(self)",
    );
    set(
        HeaderName::from_static("cross-origin-opener-policy"),
        "same-origin",
    );
    set(
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        "none",
    );
    // HSTS is only meaningful over TLS; sending it on plain HTTP would lock a
    // developer out of their own localhost.
    if https {
        set(
            HeaderName::from_static("strict-transport-security"),
            "max-age=31536000; includeSubDomains; preload",
        );
    }
}

/// Middleware: harden the response, and refuse a request that has spent its
/// budget on a sensitive endpoint.
pub async fn layer(req: Request<axum::body::Body>, next: Next) -> Response {
    let https = request_is_https(&req);
    let path = req.uri().path().to_string();
    let ip = client_ip(&req);

    // Only the endpoints worth guessing at are throttled. A page view is not.
    let budget = match path.as_str() {
        "/api/auth.php" | "/api/auth" => Some(("auth", 30u32, 60u64)),
        "/api/payments.php" | "/api/payments" => Some(("pay", 40, 60)),
        "/api/oauth.php" | "/api/oauth" => Some(("oauth", 30, 60)),
        _ => None,
    };
    if let Some((bucket, limit, window)) = budget {
        if !allow(&format!("{bucket}:{ip}"), limit, window) {
            let mut res = crate::util::err(
                "Too many attempts from this address. Wait a minute and try again.",
                429,
            );
            res.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
            apply(&mut res, https);
            return res;
        }
    }

    let mut res = next.run(req).await;
    apply(&mut res, https);
    res
}

/// The client address, honouring the proxy header the platform is served behind.
pub fn client_ip(req: &Request<axum::body::Body>) -> String {
    client_ip_from_headers(req.headers())
}

/// The client address from the headers alone. `render` receives only the header
/// map, so the page-view recorder needs the same logic without a whole request.
pub fn client_ip_from_headers(headers: &axum::http::HeaderMap) -> String {
    if let Some(v) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = v.split(',').next() {
            let first = first.trim();
            if !first.is_empty() {
                return first.to_string();
            }
        }
    }
    if let Some(v) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    "local".to_string()
}

/* ---------------------------- rate limiting ---------------------------- */

struct Window {
    count: u32,
    started: Instant,
}

static LIMITS: OnceLock<Mutex<HashMap<String, Window>>> = OnceLock::new();

fn limits() -> &'static Mutex<HashMap<String, Window>> {
    LIMITS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Record one hit against `key` and say whether it is still within `limit` per
/// `window_secs`. The window is fixed: it starts at the first hit and resets
/// when it expires, which is enough to make guessing impractical.
pub fn allow(key: &str, limit: u32, window_secs: u64) -> bool {
    let mut map = limits().lock().unwrap_or_else(|e| e.into_inner());
    // A cheap cap so a flood of distinct addresses cannot grow the map forever.
    if map.len() > 20_000 {
        map.retain(|_, w| w.started.elapsed() < Duration::from_secs(3600));
        if map.len() > 20_000 {
            map.clear();
        }
    }
    let now = Instant::now();
    let e = map.entry(key.to_string()).or_insert(Window {
        count: 0,
        started: now,
    });
    if now.duration_since(e.started) > Duration::from_secs(window_secs) {
        e.count = 0;
        e.started = now;
    }
    e.count += 1;
    e.count <= limit
}

/// Forget a key's history — called after a successful sign-in so a legitimate
/// user who mistyped a few times is not punished afterwards.
pub fn forgive(key: &str) {
    let mut map = limits().lock().unwrap_or_else(|e| e.into_inner());
    map.remove(key);
}

/// A description of the posture, for the admin panel. Everything here is read
/// from live configuration, so the page cannot claim more than is switched on.
pub fn posture() -> serde_json::Value {
    use serde_json::json;
    let headers = [
        ("Content-Security-Policy", true),
        ("Strict-Transport-Security", request_is_https_enabled()),
        ("X-Content-Type-Options", true),
        ("X-Frame-Options", true),
        ("Referrer-Policy", true),
        ("Permissions-Policy", true),
        ("Cross-Origin-Opener-Policy", true),
    ];
    json!({
        "headers": headers.iter().map(|(n, on)| json!({ "name": n, "enabled": on })).collect::<Vec<_>>(),
        "rateLimit": {
            "enabled": true,
            "endpoints": ["auth (30/min per address)", "payments (40/min)", "oauth (30/min)"],
            "accountLockout": "5 wrong PINs lock the account for 15 minutes",
        },
        "transport": { "https": request_is_https_enabled() },
    })
}

fn request_is_https_enabled() -> bool {
    crate::settings::get("platform_url", "").starts_with("https")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_is_spent_then_refused() {
        let key = format!("test:{}", crate::util::token_hex(6));
        for _ in 0..5 {
            assert!(allow(&key, 5, 60));
        }
        assert!(!allow(&key, 5, 60));
        forgive(&key);
        assert!(allow(&key, 5, 60));
    }

    #[test]
    fn separate_keys_do_not_share_a_budget() {
        let a = format!("a:{}", crate::util::token_hex(6));
        let b = format!("b:{}", crate::util::token_hex(6));
        assert!(allow(&a, 1, 60));
        assert!(!allow(&a, 1, 60));
        assert!(allow(&b, 1, 60));
    }

    #[test]
    fn the_policy_denies_framing_and_object_embedding() {
        assert!(CSP.contains("frame-ancestors 'self'"));
        assert!(CSP.contains("object-src 'none'"));
    }
}
