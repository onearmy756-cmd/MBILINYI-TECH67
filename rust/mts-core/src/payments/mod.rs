//! Payments: gateways, escrow and verification codes.
//!
//! Two rules shape this module.
//!
//! 1. A gateway is inert until the admin switches it on AND fills in the
//!    merchant credentials. Funds always settle to the account named in those
//!    credentials, which is why the admin must supply them — a gateway cannot
//!    route money into an account it has not been given.
//! 2. Every gateway is reached through the same two calls, `initiate` and
//!    `verify`, so adding one does not touch the rest of the system.

pub mod escrow;
pub mod gateways;
pub mod verify;
pub mod webauthn;

use crate::db;
use crate::settings;
use crate::util;
use serde_json::{json, Value as J};

/// Everything the UI needs to render a gateway, including whether it can
/// actually be used right now.
pub struct GatewayDef {
    pub key: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub icon: &'static str,
    pub brand: &'static str,
    pub currencies: &'static [&'static str],
    /// Settings that must be non-empty before the gateway will be offered.
    pub required: &'static [&'static str],
}

/// The full catalogue. `icon` is a Font Awesome class, which the site already
/// loads, so a real brand mark appears with no extra assets.
pub const CATALOGUE: &[GatewayDef] = &[
    GatewayDef { key: "clickpesa", name: "ClickPesa", kind: "mobile", icon: "fa-solid fa-mobile-screen-button", brand: "#00A651", currencies: &["TZS"], required: &["gw_clickpesa_key", "gw_clickpesa_secret", "gw_clickpesa_account"] },
    GatewayDef { key: "beem", name: "Beem Africa", kind: "mobile", icon: "fa-solid fa-tower-broadcast", brand: "#0B5FFF", currencies: &["TZS"], required: &["gw_beem_key", "gw_beem_secret", "gw_beem_account"] },
    GatewayDef { key: "selcom", name: "Selcom Pay", kind: "mobile", icon: "fa-solid fa-wallet", brand: "#E31E24", currencies: &["TZS"], required: &["gw_selcom_key", "gw_selcom_secret", "gw_selcom_vendor"] },
    GatewayDef { key: "mpesa", name: "M-Pesa (Vodacom)", kind: "mobile", icon: "fa-solid fa-mobile-retro", brand: "#E60000", currencies: &["TZS"], required: &["gw_mpesa_key", "gw_mpesa_secret", "gw_mpesa_shortcode"] },
    GatewayDef { key: "tigopesa", name: "Mixx by Yas (Tigo)", kind: "mobile", icon: "fa-solid fa-sim-card", brand: "#0057B8", currencies: &["TZS"], required: &["gw_tigo_key", "gw_tigo_secret", "gw_tigo_shortcode"] },
    GatewayDef { key: "airtel", name: "Airtel Money", kind: "mobile", icon: "fa-solid fa-signal", brand: "#FF0000", currencies: &["TZS"], required: &["gw_airtel_key", "gw_airtel_secret", "gw_airtel_shortcode"] },
    GatewayDef { key: "halopesa", name: "HaloPesa (Halotel)", kind: "mobile", icon: "fa-solid fa-mobile-screen", brand: "#F5A623", currencies: &["TZS"], required: &["gw_halo_key", "gw_halo_secret", "gw_halo_shortcode"] },
    GatewayDef { key: "bank_local", name: "Bank transfer (Tanzania)", kind: "bank", icon: "fa-solid fa-building-columns", brand: "#0B7285", currencies: &["TZS"], required: &["gw_bank_name", "gw_bank_account", "gw_bank_holder"] },
    GatewayDef { key: "stripe", name: "Stripe", kind: "card", icon: "fa-brands fa-stripe-s", brand: "#635BFF", currencies: &["USD", "EUR", "GBP", "TZS"], required: &["gw_stripe_publishable", "gw_stripe_secret", "gw_stripe_account"] },
    GatewayDef { key: "paypal", name: "PayPal", kind: "card", icon: "fa-brands fa-paypal", brand: "#003087", currencies: &["USD", "EUR", "GBP"], required: &["gw_paypal_client", "gw_paypal_secret", "gw_paypal_email"] },
    GatewayDef { key: "payoneer", name: "Payoneer", kind: "card", icon: "fa-brands fa-cc-mastercard", brand: "#FF4800", currencies: &["USD", "EUR"], required: &["gw_payoneer_id", "gw_payoneer_email"] },
    GatewayDef { key: "wise", name: "Wise Business", kind: "bank", icon: "fa-solid fa-globe", brand: "#9FE870", currencies: &["USD", "EUR", "GBP"], required: &["gw_wise_iban", "gw_wise_holder"] },
    GatewayDef { key: "swift", name: "International bank / SWIFT", kind: "bank", icon: "fa-solid fa-earth-americas", brand: "#1B4F72", currencies: &["USD", "EUR", "GBP"], required: &["gw_intl_bank_name", "gw_intl_iban", "gw_intl_swift", "gw_intl_holder"] },
];

pub fn find(key: &str) -> Option<&'static GatewayDef> {
    CATALOGUE.iter().find(|g| g.key == key)
}

/// A gateway is enabled when the admin switched it on and every required
/// setting has a value. The second half is what stops a half-configured
/// gateway from appearing and then failing at the till.
pub fn enabled(g: &GatewayDef) -> bool {
    if !settings::flag(&format!("gw_{}_enabled", g.key)) {
        return false;
    }
    g.required.iter().all(|k| !settings::get(k, "").trim().is_empty())
}

/// The gateways to show a buyer, each with its live configuration status.
pub fn available() -> Vec<J> {
    CATALOGUE
        .iter()
        .map(|g| {
            json!({
                "key": g.key,
                "name": g.name,
                "kind": g.kind,
                "icon": g.icon,
                "brand": g.brand,
                "currencies": g.currencies,
                "enabled": enabled(g),
                "configured": g.required.iter().all(|k| !settings::get(k, "").trim().is_empty()),
            })
        })
        .collect()
}

/// Every setting the admin must fill, with its label and whether it is secret.
/// The admin UI is generated from this, so a new gateway needs no frontend edit.
pub fn config_fields() -> Vec<J> {
    let mut out = Vec::new();
    for g in CATALOGUE {
        for k in g.required {
            out.push(json!({
                "key": k,
                "gateway": g.key,
                "secret": k.contains("secret") || k.contains("_key"),
            }));
        }
    }
    out
}

fn arg(v: J) -> rusqlite::types::Value {
    db::from_json(v)
}

/// Record a payment attempt. The row is the single source of truth for the
/// status a client sees, whether the confirmation arrives by webhook or by a
/// manual check, and it is what escrow later reads.
#[allow(clippy::too_many_arguments)]
pub fn create_payment(
    invoice_id: &str,
    user_id: &str,
    gateway: &str,
    amount: f64,
    currency: &str,
    payer_name: &str,
    payer_phone: &str,
    payer_email: &str,
) -> String {
    let id = util::new_id("PAY-");
    let code = verify::make_code();
    let _ = db::run(
        "INSERT INTO payments (id, invoice_id, user_id, gateway, amount, currency, status,
                               payer_name, payer_phone, payer_email, verify_code, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(invoice_id)), arg(json!(user_id)),
            arg(json!(gateway)), arg(json!(amount)), arg(json!(currency)),
            arg(json!("Pending")), arg(json!(payer_name)), arg(json!(payer_phone)),
            arg(json!(payer_email)), arg(json!(code)), arg(json!(util::now())),
        ],
    );
    id
}

pub fn get_payment(id: &str) -> Option<db::Row> {
    db::row("SELECT * FROM payments WHERE id = ?", &[arg(json!(id))])
}

pub fn mark(id: &str, status: &str, reference: &str, note: &str) {
    let _ = db::run(
        "UPDATE payments SET status = ?, reference = ?, note = ?, settled_at = ? WHERE id = ?",
        &[
            arg(json!(status)), arg(json!(reference)), arg(json!(note)),
            arg(json!(util::now())), arg(json!(id)),
        ],
    );
}

/// Mark the invoice paid once a payment settles, and release the contract's
/// deposit gate — the same side effect the admin's "mark paid" had.
pub fn settle_invoice(invoice_id: &str, gateway: &str, reference: &str) {
    if invoice_id.is_empty() {
        return;
    }
    let _ = db::run(
        "UPDATE invoices SET status = ?, method = ? WHERE id = ?",
        &[
            arg(json!("Paid")),
            arg(json!(format!("{gateway} • Ref {reference}"))),
            arg(json!(invoice_id)),
        ],
    );
}

/// Money held in escrow is not the platform's to spend. Each release is a row,
/// so the balance is always the sum of its history rather than a mutable field
/// that can drift.
pub fn escrow_hold(payment_id: &str, contract_id: &str, amount: f64, currency: &str) -> String {
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

/// Balance still held for a contract: holds minus releases and refunds.
pub fn escrow_balance(contract_id: &str) -> f64 {
    let held = db::scalar_i64(
        "SELECT COALESCE(SUM(amount),0) FROM escrow_ledger WHERE contract_id = ? AND kind = 'hold'",
        &[arg(json!(contract_id))],
    ) as f64;
    let out = db::scalar_i64(
        "SELECT COALESCE(SUM(amount),0) FROM escrow_ledger WHERE contract_id = ? AND kind IN ('release','refund')",
        &[arg(json!(contract_id))],
    ) as f64;
    held - out
}

/// The payment tables. `payments` records every attempt; `escrow_ledger` is the
/// append-only history that makes an escrow balance provable; `gateway_events`
/// keeps the raw webhook payloads so a disputed settlement can be re-checked
/// against what the provider actually sent.
pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS payments (
           id           VARCHAR(64)   NOT NULL,
           invoice_id   VARCHAR(64)   NOT NULL DEFAULT '',
           user_id      VARCHAR(64)   NOT NULL DEFAULT '',
           gateway      VARCHAR(40)   NOT NULL,
           amount       DECIMAL(14,2) NOT NULL DEFAULT 0,
           currency     VARCHAR(10)   NOT NULL DEFAULT 'TZS',
           status       VARCHAR(30)   NOT NULL DEFAULT 'Pending',
           reference    VARCHAR(120)  NOT NULL DEFAULT '',
           payer_name   VARCHAR(160)  NOT NULL DEFAULT '',
           payer_phone  VARCHAR(60)   NOT NULL DEFAULT '',
           payer_email  VARCHAR(190)  NOT NULL DEFAULT '',
           verify_code  VARCHAR(16)   NOT NULL DEFAULT '',
           note         VARCHAR(255)  NOT NULL DEFAULT '',
           settled_at   DATETIME      NULL,
           created_at   DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS escrow_ledger (
           id          VARCHAR(64)   NOT NULL,
           payment_id  VARCHAR(64)   NOT NULL DEFAULT '',
           contract_id VARCHAR(64)   NOT NULL DEFAULT '',
           kind        VARCHAR(20)   NOT NULL,
           amount      DECIMAL(14,2) NOT NULL DEFAULT 0,
           currency    VARCHAR(10)   NOT NULL DEFAULT 'TZS',
           status      VARCHAR(30)   NOT NULL DEFAULT 'Held',
           note        VARCHAR(255)  NOT NULL DEFAULT '',
           created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS gateway_events (
           id         VARCHAR(64)  NOT NULL,
           gateway    VARCHAR(40)  NOT NULL,
           reference  VARCHAR(120) NOT NULL DEFAULT '',
           payload    TEXT         NOT NULL,
           created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS webauthn_challenges (
           id         VARCHAR(64) NOT NULL,
           kind       VARCHAR(20) NOT NULL,
           challenge  VARCHAR(255) NOT NULL,
           created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id, kind)
         );
         CREATE TABLE IF NOT EXISTS oauth_links (
           id         VARCHAR(64)  NOT NULL,
           user_id    VARCHAR(64)  NOT NULL,
           provider   VARCHAR(30)  NOT NULL,
           subject    VARCHAR(190) NOT NULL,
           created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}

/* ---------------- shared helpers ---------------- */

pub fn random_bytes(n: usize) -> Vec<u8> {
    use rand::Rng;
    let mut b = vec![0u8; n];
    rand::thread_rng().fill(&mut b[..]);
    b
}

pub fn b64u_enc(raw: &[u8]) -> String {
    webauthn::b64u_enc(raw)
}

/// A WebAuthn challenge is a one-time value: it is stored when the ceremony
/// starts and removed the moment it is used, so a captured response cannot be
/// replayed against a later attempt.
pub fn store_challenge(session_id: &str, kind: &str, challenge: &str) {
    let _ = db::run(
        "INSERT INTO webauthn_challenges (id, kind, challenge, created_at) VALUES (?,?,?,?)",
        &[
            arg(json!(session_id)), arg(json!(kind)), arg(json!(challenge)),
            arg(json!(util::now())),
        ],
    );
}

pub fn take_challenge(session_id: &str, kind: &str) -> Option<String> {
    let r = db::row(
        "SELECT challenge FROM webauthn_challenges WHERE id = ? AND kind = ?",
        &[arg(json!(session_id)), arg(json!(kind))],
    )?;
    let _ = db::run(
        "DELETE FROM webauthn_challenges WHERE id = ? AND kind = ?",
        &[arg(json!(session_id)), arg(json!(kind))],
    );
    r.get("challenge").and_then(|v| v.as_str()).map(str::to_string)
}
