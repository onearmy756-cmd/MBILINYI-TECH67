//! Escrow.
//!
//! Money a client pays against a contract is held, not spent, until the work is
//! delivered. Each movement is a ledger row so the balance is always the sum of
//! its history: a mutable "balance" column would eventually disagree with the
//! rows that explain it.

use crate::db;
use crate::util;
use serde_json::json;

fn arg(v: serde_json::Value) -> rusqlite::types::Value {
    db::from_json(v)
}

pub const KINDS: [&str; 4] = ["hold", "release", "refund", "fee"];

/// Hold funds for a contract.
pub fn hold(payment_id: &str, contract_id: &str, amount: f64, currency: &str) -> String {
    let id = util::new_id("ESC-");
    let _ = db::run(
        "INSERT INTO escrow_ledger (id, payment_id, contract_id, kind, amount, currency, status, created_at)
         VALUES (?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(payment_id)), arg(json!(contract_id)),
            arg(json!("hold")), arg(json!(amount)), arg(json!(currency)),
            arg(json!("Held")), arg(json!(util::now())),
        ],
    );
    id
}

/// Release held funds to the platform once the client accepts the delivery.
pub fn release(contract_id: &str, amount: f64, currency: &str, note: &str) -> Option<String> {
    let held = balance(contract_id);
    if amount <= 0.0 || amount > held + 0.001 {
        return None;
    }
    let id = util::new_id("ESC-");
    let _ = db::run(
        "INSERT INTO escrow_ledger (id, payment_id, contract_id, kind, amount, currency, status, note, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!("")), arg(json!(contract_id)),
            arg(json!("release")), arg(json!(amount)), arg(json!(currency)),
            arg(json!("Released")), arg(json!(note)), arg(json!(util::now())),
        ],
    );
    Some(id)
}

/// Refund held funds to the client.
pub fn refund(contract_id: &str, amount: f64, currency: &str, note: &str) -> Option<String> {
    let held = balance(contract_id);
    if amount <= 0.0 || amount > held + 0.001 {
        return None;
    }
    let id = util::new_id("ESC-");
    let _ = db::run(
        "INSERT INTO escrow_ledger (id, payment_id, contract_id, kind, amount, currency, status, note, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!("")), arg(json!(contract_id)),
            arg(json!("refund")), arg(json!(amount)), arg(json!(currency)),
            arg(json!("Refunded")), arg(json!(note)), arg(json!(util::now())),
        ],
    );
    Some(id)
}

pub fn balance(contract_id: &str) -> f64 {
    let held: f64 = db::rows(
        "SELECT COALESCE(SUM(amount),0) AS t FROM escrow_ledger WHERE contract_id = ? AND kind = 'hold'",
        &[arg(json!(contract_id))],
    )
    .first()
    .and_then(|r| r.get("t"))
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0);
    let out: f64 = db::rows(
        "SELECT COALESCE(SUM(amount),0) AS t FROM escrow_ledger WHERE contract_id = ? AND kind IN ('release','refund')",
        &[arg(json!(contract_id))],
    )
    .first()
    .and_then(|r| r.get("t"))
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0);
    held - out
}

pub fn ledger(contract_id: &str) -> Vec<serde_json::Value> {
    db::rows(
        "SELECT * FROM escrow_ledger WHERE contract_id = ? ORDER BY created_at DESC",
        &[arg(json!(contract_id))],
    )
    .into_iter()
    .map(serde_json::Value::Object)
    .collect()
}

/// Total currently held across every contract — the figure the admin must not
/// treat as revenue.
pub fn total_held() -> f64 {
    let held: f64 = db::rows(
        "SELECT COALESCE(SUM(amount),0) AS t FROM escrow_ledger WHERE kind = 'hold'",
        &[],
    )
    .first()
    .and_then(|r| r.get("t"))
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0);
    let out: f64 = db::rows(
        "SELECT COALESCE(SUM(amount),0) AS t FROM escrow_ledger WHERE kind IN ('release','refund')",
        &[],
    )
    .first()
    .and_then(|r| r.get("t"))
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0);
    held - out
}
