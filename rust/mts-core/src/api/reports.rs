//! The R analytics layer.
//!
//! Rust owns the data and serves it as CSV; `r/report.R` reads that CSV with
//! base R only — no packages to install — and prints a JSON result that this
//! module hands straight to the browser.
//!
//! Doing it this way keeps R as a real analytics engine (it does the grouping,
//! the summaries and the trend work) without adding a database driver to the R
//! side, which would be one more thing to install on the host. When R is not
//! present the endpoints say so plainly instead of failing.

use crate::api::ctx::Ctx;
use crate::db;
use crate::util::{self, bad, ok};
use axum::response::Response;
use serde_json::json;
use std::process::Command;

/// The reports that exist, with the CSV each one feeds to R.
fn catalogue() -> Vec<serde_json::Value> {
    vec![
        json!({
            "key": "revenue",
            "name": "Revenue & pipeline",
            "en": "Invoices by month with paid, unpaid and pipeline totals",
            "sw": "Ankara kwa mwezi na jumla ya zilizolipwa, hazijalipwa na bomba la mauzo",
        }),
        json!({
            "key": "requests",
            "name": "Requests by service and status",
            "en": "How many requests each service draws, and where they sit",
            "sw": "Maombi kwa huduma na hali yake",
        }),
        json!({
            "key": "payments",
            "name": "Payments by gateway",
            "en": "Which payment methods clients actually use, and settlement rates",
            "sw": "Njia za malipo zinazotumika, na kiwango cha kukamilika",
        }),
        json!({
            "key": "escrow",
            "name": "Escrow held per contract",
            "en": "Money held against each contract, and what has been released",
            "sw": "Pesa inayoshikiliwa kwa kila mkataba, na iliyotolewa",
        }),
    ]
}

fn rscript() -> Option<String> {
    // `MTS_RSCRIPT` wins, so a host with R somewhere unusual still works.
    if let Ok(p) = std::env::var("MTS_RSCRIPT") {
        if std::path::Path::new(&p).is_file() {
            return Some(p);
        }
    }
    for p in ["/usr/bin/Rscript", "/usr/local/bin/Rscript"] {
        if std::path::Path::new(p).is_file() {
            return Some(p.to_string());
        }
    }
    None
}

/// R lives in /usr/bin, which the session reset clears, so "is R installed" is
/// a live question rather than a startup constant.
fn r_available() -> bool {
    rscript().is_some()
}

/// The CSV for a report. Kept next to the R script's expectations.
fn csv_for(key: &str) -> Result<String, String> {
    let (header, rows): (&str, Vec<String>) = match key {
        "revenue" => {
            let rs = db::rows(
                "SELECT substr(created_at,1,7) AS month, status, amount FROM invoices ORDER BY month",
                &[],
            );
            (
                "month,status,amount",
                rs.iter()
                    .map(|r| {
                        format!(
                            "{},{},{}",
                            r.get("month").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        )
                    })
                    .collect(),
            )
        }
        "requests" => {
            let rs = db::rows("SELECT service, status, priority FROM requests", &[]);
            (
                "service,status,priority",
                rs.iter()
                    .map(|r| {
                        format!(
                            "\"{}\",\"{}\",\"{}\"",
                            r.get("service").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("priority").and_then(|v| v.as_str()).unwrap_or("")
                        )
                    })
                    .collect(),
            )
        }
        "payments" => {
            let rs = db::rows(
                "SELECT gateway, status, currency, amount FROM payments",
                &[],
            );
            (
                "gateway,status,currency,amount",
                rs.iter()
                    .map(|r| {
                        format!(
                            "{},{},{},{}",
                            r.get("gateway").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("currency").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        )
                    })
                    .collect(),
            )
        }
        "escrow" => {
            let rs = db::rows(
                "SELECT contract_id, kind, currency, amount FROM escrow_ledger",
                &[],
            );
            (
                "contract_id,kind,currency,amount",
                rs.iter()
                    .map(|r| {
                        format!(
                            "{},{},{},{}",
                            r.get("contract_id").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("kind").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("currency").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        )
                    })
                    .collect(),
            )
        }
        _ => return Err(format!("Unknown report: {key}")),
    };
    let mut out = String::from(header);
    out.push('\n');
    for row in rows {
        out.push_str(&row);
        out.push('\n');
    }
    Ok(out)
}

/// Where the R script lives, alongside the same fallbacks as the templates.
fn script_dir() -> String {
    if let Ok(d) = std::env::var("MTS_R_DIR") {
        if std::path::Path::new(&d).is_dir() {
            return d;
        }
    }
    for c in ["rust/mts-core/r", "r"] {
        if std::path::Path::new(c).is_dir() {
            return c.to_string();
        }
    }
    "rust/mts-core/r".to_string()
}

/// Run one report: export the CSV, hand it to R, return what R printed.
fn run_report(key: &str) -> Response {
    let csv = match csv_for(key) {
        Ok(c) => c,
        Err(e) => return bad(&e),
    };
    let Some(rscript) = rscript() else {
        return ok(json!({
            "report": key,
            "r": false,
            "note": "R is not installed on this host, so the analytics report cannot run. The CSV export below is the same data R would have read.",
            "csv": csv,
        }));
    };
    let script = format!("{}/report.R", script_dir());
    if !std::path::Path::new(&script).is_file() {
        return ok(json!({
            "report": key,
            "r": true,
            "note": format!("The R script is missing at {script}."),
            "csv": csv,
        }));
    }

    // The CSV goes in on stdin, so no temporary file is needed and nothing is
    // left behind on disk.
    let mut child = match Command::new(&rscript)
        .arg(&script)
        .arg(key)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return ok(json!({
                "report": key,
                "r": false,
                "note": format!("Could not start R: {e}"),
                "csv": csv,
            }))
        }
    };
    {
        use std::io::Write;
        if let Some(si) = child.stdin.as_mut() {
            let _ = si.write_all(csv.as_bytes());
        }
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return ok(json!({
                "report": key, "r": false,
                "note": format!("R did not finish: {e}"), "csv": csv,
            }))
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    // R prints the JSON result on the last line; anything before it is chatter.
    let json_line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'));
    match json_line.and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok()) {
        Some(v) => ok(json!({ "report": key, "r": true, "result": v })),
        None => ok(json!({
            "report": key,
            "r": true,
            "note": "R produced no result.",
            "stdout": stdout,
            "stderr": stderr,
        })),
    }
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => ok(json!({
            "reports": catalogue(),
            "r_available": r_available(),
            "r_path": rscript().unwrap_or_default(),
        })),
        "run" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = c.p("report");
            if key.is_empty() {
                return bad("Choose a report.");
            }
            run_report(&key)
        }
        "csv" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = c.p("report");
            match csv_for(&key) {
                Ok(csv) => {
                    let mut res = Response::new(axum::body::Body::from(csv));
                    res.headers_mut().insert(
                        axum::http::header::CONTENT_TYPE,
                        axum::http::HeaderValue::from_static("text/csv; charset=utf-8"),
                    );
                    res
                }
                Err(e) => bad(&e),
            }
        }
        _ => bad("Unknown reports action."),
    }
}

/// Unused guard.
#[allow(dead_code)]
fn _keep() {
    let _ = util::now();
}
