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

mod api;
mod db;
mod pages;
mod payments;
mod schema_sql;
mod seed;
mod session;
mod settings;
mod util;

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
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
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
        return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, target)], "").into_response();
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
            return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, target)], "").into_response();
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
    crate::api::oauth::init_tables();
    settings::seed_defaults();
    seed::run();
    session::sweep();
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

    let app = Router::new()
        .route("/api/*name", get(api_dispatch).post(api_dispatch))
        .route("/*path", get(page_dispatch).post(page_dispatch))
        .fallback(get(page_dispatch).post(page_dispatch));

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
