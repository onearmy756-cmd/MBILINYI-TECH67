//! Mbilinyi Tech Solutions — Rust core.
//!
//! This replaces the PHP backend. The frontend is untouched: the same pages are
//! served, and `/api/*` returns the same JSON envelope (`{ok, data}` /
//! `{ok, error}`) that `assets/app.js` already unwraps, so no JavaScript had to
//! change.
//!
//! Layout:
//!   db        SQLite access
//!   session   cookies, sessions, CSRF
//!   settings  the settings table (prices, payment accounts, gateway keys)
//!   payments  gateways, escrow, verification codes
//!   api       the JSON endpoints
//!   pages     the HTML pages

mod agents;
mod ajira;
mod api;
mod call;
mod db;
mod engine;
mod engine_api;
mod hardening;
mod langrun;
mod llm;
mod llm_api;
mod mail;
mod notify;
mod ofisi;
mod pages;
mod payroll;
mod payments;
mod pdf;
mod pqc;
mod runtime;
mod runtime_api;
mod sandbox;
mod scheduler;
mod schema_sql;
mod security;
mod seed;
mod session;
mod settings;
mod sms;
mod tools;
mod util;
mod wafanyakazi;

use axum::body::Body;
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

fn mime_of(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Serve a real file from disk (assets, icons, the service worker). Anything
/// under a private tree, and any dotfile, is refused rather than read.
async fn static_file(path: &str) -> Response {
    let clean = path.trim_start_matches('/');
    if clean.starts_with("storage/")
        || clean.starts_with("config/")
        || clean.starts_with("scripts/")
        || clean.starts_with("rust/")
        || clean.split('/').any(|seg| seg.starts_with('.'))
    {
        return (StatusCode::FORBIDDEN, "403 Forbidden").into_response();
    }
    match tokio::fs::read(clean).await {
        Ok(bytes) => {
            let ct = HeaderValue::from_static(mime_of(clean));
            let mut res = Response::new(Body::from(bytes));
            res.headers_mut().insert(header::CONTENT_TYPE, ct);
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=3600"),
            );
            res
        }
        Err(_) => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

/// `/api/<name>` and `/api/<name>.php` both reach the same handler, because
/// `app.js` uses the clean path while older links and the smoke test use the
/// `.php` form.
async fn api_dispatch(req: Request<Body>) -> Response {
    let name = req
        .uri()
        .path()
        .trim_start_matches("/api/")
        .trim_end_matches('/')
        .trim_end_matches(".php")
        .to_string();
    api::dispatch(&name, req).await
}

/// The query string, carried through a redirect so no parameter is lost.
fn query_of(uri: &axum::http::Uri) -> String {
    match uri.query() {
        Some(q) if !q.is_empty() => format!("?{q}"),
        _ => String::new(),
    }
}

/// The health endpoint: what a load balancer or an uptime monitor asks. It
/// reports the facts a monitor needs — the database answers, the schema is in
/// place, the process is up — and nothing that would help an attacker.
async fn health() -> Response {
    let db_ok = db::scalar_i64("SELECT 1", &[]) == 1;
    let tables = db::scalar_i64(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
        &[],
    );
    let ai = crate::llm::ready();
    let sms = crate::sms::configured();
    let mail_ok = crate::mail::status()
        .get("configured")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let status = if db_ok && tables > 20 {
        "ok"
    } else {
        "degraded"
    };
    let code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        [("Content-Type", "application/json; charset=utf-8")],
        serde_json::json!({
            "status": status,
            "database": db_ok,
            "tables": tables,
            "channels": { "ai": ai, "sms": sms, "email": mail_ok },
            "time": util::now(),
        })
        .to_string(),
    )
        .into_response()
}

/// The inbound SMS callback. The provider posts here; the payload is handed to
/// the office module, which checks the shared secret and files the message in
/// the admin's inbox.
async fn inbound_sms(req: Request<Body>) -> Response {
    let c = api::ctx::Ctx::build(req).await;
    crate::ofisi::inbound_webhook(&c)
}

/// The Stripe webhook. The signature is verified inside the payments module
/// before the event is acted on.
async fn stripe_webhook(req: Request<Body>) -> Response {
    let c = api::ctx::Ctx::build(req).await;
    crate::api::pay::stripe_webhook(&c).await
}

/// A generated PDF: `/api/pdf/<kind>/<id>`, where kind is `invoice`, `receipt`
/// or `contract`. The document is built from the record on every request, so it
/// always reflects the current state of the row.
async fn pdf_doc(req: Request<Body>) -> Response {
    let path = req.uri().path().to_string();
    let mut parts = path.trim_start_matches("/api/pdf/").splitn(2, '/');
    let kind = parts.next().unwrap_or("").to_string();
    let id = parts.next().unwrap_or("").to_string();
    let c = api::ctx::Ctx::build(req).await;

    // An invoice and a contract are private: the caller must own it or be admin.
    let doc = match kind.as_str() {
        "invoice" | "receipt" => {
            let Some(inv) = db::row(
                "SELECT user_id FROM invoices WHERE id = ?",
                &[db::from_json(serde_json::json!(id))],
            ) else {
                return (StatusCode::NOT_FOUND, "Invoice not found").into_response();
            };
            if !owns(
                &c,
                util::s(inv.get("user_id").unwrap_or(&serde_json::Value::Null)),
            ) {
                return (StatusCode::FORBIDDEN, "Not your invoice").into_response();
            }
            pdf::invoice(&id, kind == "receipt")
        }
        "contract" => {
            let Some(ct) = db::row(
                "SELECT tracking_id FROM contracts WHERE id = ?",
                &[db::from_json(serde_json::json!(id))],
            ) else {
                return (StatusCode::NOT_FOUND, "Contract not found").into_response();
            };
            let trk = util::s(ct.get("tracking_id").unwrap_or(&serde_json::Value::Null));
            let owner = db::row(
                "SELECT user_id FROM requests WHERE tracking_id = ?",
                &[db::from_json(serde_json::json!(trk))],
            )
            .map(|r| util::s(r.get("user_id").unwrap_or(&serde_json::Value::Null)))
            .unwrap_or_default();
            if !owns(&c, owner) {
                return (StatusCode::FORBIDDEN, "Not your contract").into_response();
            }
            pdf::contract(&id)
        }
        _ => None,
    };
    let Some(bytes) = doc else {
        return (StatusCode::NOT_FOUND, "No such document").into_response();
    };
    let filename = format!("{kind}-{id}.pdf");
    (
        StatusCode::OK,
        [
            ("Content-Type", "application/pdf"),
            (
                "Content-Disposition",
                Box::leak(format!("inline; filename=\"{filename}\"").into_boxed_str()) as &str,
            ),
            ("Cache-Control", "no-store"),
        ],
        bytes,
    )
        .into_response()
}

/// May the caller read a record owned by `owner_id`? An admin may; a signed-in
/// user may read their own; nobody else may.
fn owns(c: &api::ctx::Ctx, owner_id: String) -> bool {
    let Some(u) = c.user() else {
        return false;
    };
    if u.get("role").and_then(|v| v.as_str()) == Some("admin") {
        return true;
    }
    u.get("id").and_then(|v| v.as_str()) == Some(owner_id.as_str())
}

/// Any page the site serves. Mirrors `router.php`:
///   `/pricing/`   -> 301 `/pricing`
///   `/pricing.php`-> 301 `/pricing` (GET/HEAD only, so a POST keeps its body)
///   a real file   -> served as-is
///   `/pricing`    -> the pricing page
#[axum::debug_handler]
async fn page_dispatch(req: Request<Body>) -> Response {
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let method = req.method().clone();

    // Private trees and dotfiles are refused outright.
    let clean = path.trim_start_matches('/');
    if clean.starts_with("storage/")
        || clean.starts_with("config/")
        || clean.starts_with("scripts/")
        || clean.split('/').any(|seg| seg.starts_with('.'))
    {
        return (StatusCode::FORBIDDEN, "403 Forbidden").into_response();
    }

    // A trailing slash resolves to the clean URL.
    if path.len() > 1 && path.ends_with('/') {
        let target = format!("{}{}", path.trim_end_matches('/'), query_of(&uri));
        return (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, target)],
            "",
        )
            .into_response();
    }

    // `/page.php` resolves to the clean URL, but only for GET/HEAD: redirecting
    // a POST would turn it into a GET and drop the body, breaking every API call.
    if path != "/" && path.ends_with(".php") {
        if method == axum::http::Method::GET || method == axum::http::Method::HEAD {
            let stem = path.trim_end_matches(".php");
            let target = if stem.is_empty() || stem == "/index" {
                "/".to_string()
            } else {
                stem.to_string()
            };
            let target = format!("{target}{}", query_of(&uri));
            return (
                StatusCode::MOVED_PERMANENTLY,
                [(header::LOCATION, target)],
                "",
            )
                .into_response();
        }
    }

    // A real file (asset) wins over a page.
    if path != "/" && path.contains('.') {
        return static_file(&path).await;
    }

    let name = path.trim_matches('/');
    let name = if name.is_empty() { "index" } else { name };
    pages::render(name, req.headers()).await
}

/// Everything a database needs to be usable: the schema, the tables the Rust
/// side adds, the settings defaults and the demo data. Runs at boot, and again
/// if the database file is replaced while the server is up.
pub fn bootstrap() {
    if let Err(e) = db::exec(schema_sql::SCHEMA_SQL) {
        eprintln!("[fatal] schema failed: {e}");
        return;
    }
    session::init_table();
    payments::init_tables();
    // A column added to the schema after a database was first created needs an
    // explicit ALTER; CREATE TABLE IF NOT EXISTS will not add it.
    if let Err(e) = db::ensure_column("invoices", "currency", "VARCHAR(8) NOT NULL DEFAULT ''") {
        eprintln!("[warn] invoices.currency migration: {e}");
    }
    for (col, decl) in [
        ("country", "VARCHAR(80) NOT NULL DEFAULT ''"),
        ("region", "VARCHAR(80) NOT NULL DEFAULT ''"),
    ] {
        if let Err(e) = db::ensure_column("requests", col, decl) {
            eprintln!("[warn] requests.{col} migration: {e}");
        }
    }
    crate::api::oauth::init_tables();
    wafanyakazi::init_tables();
    ajira::init_tables();
    agents::init_tables();
    engine::init_tables();
    scheduler::init_tables();
    ofisi::init_tables();
    security::init_tables();
    payroll::ensure_schema();
    mail::init_tables();
    pqc::init_tables();
    pdf_init_ok();
    settings::seed_defaults();
    seed::run();
    agents::seed();
    ofisi::seed();
    session::sweep();
}

/// A database for the test process. `db::open` may only run once per process,
/// so a `Once` guards it and every test that touches a table calls this first.
#[cfg(test)]
pub fn test_bootstrap() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let path = std::env::temp_dir().join(format!("mts-test-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
        if let Err(e) = db::open(path.to_str().expect("temp path")) {
            eprintln!("[test] cannot open the test database: {e}");
            return;
        }
        bootstrap();
    });
}

/// Touch the PDF module so a broken layout crate fails loudly at boot rather
/// than the first time a client asks for a receipt.
fn pdf_init_ok() {
    let probe = pdf::build("probe", "", &[], "");
    if !probe.starts_with(b"%PDF") {
        eprintln!("[warn] the PDF writer did not produce a valid document at boot");
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("MTS_DB").unwrap_or_else(|_| "storage/mts.sqlite".to_string());
    let port = std::env::var("MTS_PORT").unwrap_or_else(|_| "12000".to_string());

    if let Err(e) = db::open(&db_path) {
        eprintln!("[fatal] cannot open {db_path}: {e}");
        std::process::exit(1);
    }
    bootstrap();

    // The autonomous scheduler: from here on the company works without being
    // pushed. It claims queued jobs, honours the CEO's approval gate, and keeps
    // every step on the record.
    scheduler::start();

    let app = Router::new()
        .route("/ws/call/:room", get(call::ws_call))
        .route("/api/*name", get(api_dispatch).post(api_dispatch))
        // The SMS provider's inbound callback, and Stripe's webhook: both are
        // called by a third party, so they are separate paths that carry their
        // own authentication rather than a session.
        .route("/api/inbound/sms", get(inbound_sms).post(inbound_sms))
        .route(
            "/api/webhook/stripe",
            get(stripe_webhook).post(stripe_webhook),
        )
        .route("/api/pdf/:kind/:id", get(pdf_doc))
        .route("/api/health", get(health))
        .route("/*path", get(page_dispatch).post(page_dispatch))
        .fallback(get(page_dispatch).post(page_dispatch))
        // Security headers on every response, and the rate limits on the
        // endpoints worth guessing at.
        .layer(axum::middleware::from_fn(hardening::layer));

    let addr = format!("0.0.0.0:{port}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[fatal] cannot bind {addr}: {e}");
            std::process::exit(1);
        }
    };
    println!("Mbilinyi Tech Solutions — Rust core on http://{addr}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[fatal] server stopped: {e}");
    }
}
