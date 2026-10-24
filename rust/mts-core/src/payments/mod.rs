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
    GatewayDef {
        key: "clickpesa",
        name: "ClickPesa",
        kind: "mobile",
        icon: "fa-solid fa-mobile-screen-button",
        brand: "#00A651",
        currencies: &["TZS"],
        required: &[
            "gw_clickpesa_key",
            "gw_clickpesa_secret",
            "gw_clickpesa_account",
        ],
    },
    GatewayDef {
        key: "beem",
        name: "Beem Africa",
        kind: "mobile",
        icon: "fa-solid fa-tower-broadcast",
        brand: "#0B5FFF",
        currencies: &["TZS"],
        required: &["gw_beem_key", "gw_beem_secret", "gw_beem_account"],
    },
    GatewayDef {
        key: "selcom",
        name: "Selcom Pay",
        kind: "mobile",
        icon: "fa-solid fa-wallet",
        brand: "#E31E24",
        currencies: &["TZS"],
        required: &["gw_selcom_key", "gw_selcom_secret", "gw_selcom_vendor"],
    },
    GatewayDef {
        key: "mpesa",
        name: "M-Pesa (Vodacom)",
        kind: "mobile",
        icon: "fa-solid fa-mobile-retro",
        brand: "#E60000",
        currencies: &["TZS"],
        required: &["gw_mpesa_key", "gw_mpesa_secret", "gw_mpesa_shortcode"],
    },
    GatewayDef {
        key: "tigopesa",
        name: "Mixx by Yas (Tigo)",
        kind: "mobile",
        icon: "fa-solid fa-sim-card",
        brand: "#0057B8",
        currencies: &["TZS"],
        required: &["gw_tigo_key", "gw_tigo_secret", "gw_tigo_shortcode"],
    },
    GatewayDef {
        key: "airtel",
        name: "Airtel Money",
        kind: "mobile",
        icon: "fa-solid fa-signal",
        brand: "#FF0000",
        currencies: &["TZS"],
        required: &["gw_airtel_key", "gw_airtel_secret", "gw_airtel_shortcode"],
    },
    GatewayDef {
        key: "halopesa",
        name: "HaloPesa (Halotel)",
        kind: "mobile",
        icon: "fa-solid fa-mobile-screen",
        brand: "#F5A623",
        currencies: &["TZS"],
        required: &["gw_halo_key", "gw_halo_secret", "gw_halo_shortcode"],
    },
    GatewayDef {
        key: "bank_local",
        name: "Bank transfer (Tanzania)",
        kind: "bank",
        icon: "fa-solid fa-building-columns",
        brand: "#0B7285",
        currencies: &["TZS"],
        required: &["gw_bank_name", "gw_bank_account", "gw_bank_holder"],
    },
    GatewayDef {
        key: "stripe",
        name: "Stripe",
        kind: "card",
        icon: "fa-brands fa-stripe-s",
        brand: "#635BFF",
        currencies: &["USD", "EUR", "GBP", "TZS"],
        required: &["gw_stripe_publishable", "gw_stripe_secret"],
    },
    GatewayDef {
        key: "paypal",
        name: "PayPal",
        kind: "card",
        icon: "fa-brands fa-paypal",
        brand: "#003087",
        currencies: &["USD", "EUR", "GBP"],
        required: &["gw_paypal_client", "gw_paypal_secret", "gw_paypal_email"],
    },
    GatewayDef {
        key: "payoneer",
        name: "Payoneer",
        kind: "card",
        icon: "fa-brands fa-cc-mastercard",
        brand: "#FF4800",
        currencies: &["USD", "EUR"],
        required: &["gw_payoneer_id", "gw_payoneer_email"],
    },
    GatewayDef {
        key: "wise",
        name: "Wise Business",
        kind: "bank",
        icon: "fa-solid fa-globe",
        brand: "#9FE870",
        currencies: &["USD", "EUR", "GBP"],
        required: &["gw_wise_iban", "gw_wise_holder", "gw_wise_token"],
    },
    GatewayDef {
        key: "swift",
        name: "International bank / SWIFT",
        kind: "bank",
        icon: "fa-solid fa-earth-americas",
        brand: "#1B4F72",
        currencies: &["USD", "EUR", "GBP"],
        required: &[
            "gw_intl_bank_name",
            "gw_intl_iban",
            "gw_intl_swift",
            "gw_intl_holder",
        ],
    },
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
    g.required
        .iter()
        .all(|k| !settings::get(k, "").trim().is_empty())
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

/// The number of Tanzanian shillings in one US dollar, as the CEO sets it.
/// Held in settings so the figure can be corrected without a redeploy.
pub fn usd_rate() -> f64 {
    settings::num("fx_usd_tzs", 0.0)
}

/// Convert an amount between the invoice currency and the currency the payer
/// chose, at a given shillings-per-dollar rate. `None` means the pair cannot be
/// converted — the caller must refuse rather than charge the wrong figure. A
/// TZS invoice paid in USD must never send the shilling amount to Stripe as
/// dollars.
pub fn convert_at(amount: f64, from: &str, to: &str, rate: f64) -> Option<f64> {
    let (from, to) = (from.to_uppercase(), to.to_uppercase());
    if from == to {
        return Some(amount);
    }
    if rate <= 0.0 {
        return None;
    }
    match (from.as_str(), to.as_str()) {
        ("TZS", "USD") => Some(amount / rate),
        ("USD", "TZS") => Some(amount * rate),
        _ => None,
    }
}

/// `convert_at` at the rate the CEO configured.
pub fn convert(amount: f64, from: &str, to: &str) -> Option<f64> {
    convert_at(amount, from, to, usd_rate())
}

/// The currencies the platform quotes besides the shilling. Each has a rate
/// against TZS in settings, so a USD invoice can be paid in EUR without a
/// separate setting for every pair.
pub const QUOTE_CURRENCIES: [&str; 4] = ["USD", "EUR", "GBP", "KES"];

/// Shillings per one unit of `currency`. USD keeps its original setting name so
/// an existing installation needs no migration.
pub fn rate_for(currency: &str) -> f64 {
    match currency.to_uppercase().as_str() {
        "TZS" => 1.0,
        "USD" => settings::num("fx_usd_tzs", 0.0),
        other => settings::num(&format!("fx_{}_tzs", other.to_lowercase()), 0.0),
    }
}

/// Convert between any two currencies the platform quotes, pivoting through the
/// shilling. `None` means a rate is missing, and the caller must refuse rather
/// than charge a guessed figure. This is the pair-wise generalisation of
/// `convert_at`, which only knows the single dollar rate.
pub fn convert_any(amount: f64, from: &str, to: &str) -> Option<f64> {
    convert_any_at(amount, from, to, rate_for(from), rate_for(to))
}

/// Fetch live mid-market rates from the Wise API and store them as the TZS
/// rates. Requires a Wise API token (`gw_wise_token`); without one the manual
/// rates stand. Returns the number of currencies updated.
pub async fn refresh_rates_from_wise() -> Result<Vec<J>, String> {
    let token = settings::get("gw_wise_token", "");
    if token.trim().is_empty() {
        return Err(
            "No Wise API token is set (gw_wise_token), so live rates cannot be fetched.".into(),
        );
    }
    let targets = ["USD", "EUR", "GBP", "KES"];
    let source = "TZS";
    let url = format!(
        "https://api.transferwise.com/v1/rates?source={source}&target={}",
        targets.join(",")
    );
    let res = reqwest::Client::new()
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let code = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    if !(200..300).contains(&code) {
        return Err(format!("Wise refused the request (HTTP {code}): {text}"));
    }
    let parsed: J =
        serde_json::from_str(&text).map_err(|e| format!("Unreadable rate reply: {e}"))?;
    let mut updated = Vec::new();
    if let Some(arr) = parsed.as_array() {
        for entry in arr {
            let target = entry.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let rate = entry.get("rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if target.is_empty() || rate <= 0.0 {
                continue;
            }
            let key = if target.eq_ignore_ascii_case("USD") {
                "fx_usd_tzs".to_string()
            } else {
                format!("fx_{}_tzs", target.to_lowercase())
            };
            settings::set(&key, &format!("{rate:.6}"));
            updated.push(json!({ "currency": target, "tzsPerUnit": rate }));
        }
    }
    if updated.is_empty() {
        return Err("Wise returned no usable rates.".into());
    }
    settings::set("fx_source", "wise");
    settings::set("fx_updated_at", &util::now());
    Ok(updated)
}

/// Round money to two decimals, so a converted figure is a payable amount.
pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
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
            arg(json!(id)),
            arg(json!(invoice_id)),
            arg(json!(user_id)),
            arg(json!(gateway)),
            arg(json!(amount)),
            arg(json!(currency)),
            arg(json!("Pending")),
            arg(json!(payer_name)),
            arg(json!(payer_phone)),
            arg(json!(payer_email)),
            arg(json!(code)),
            arg(json!(util::now())),
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
            arg(json!(status)),
            arg(json!(reference)),
            arg(json!(note)),
            arg(json!(util::now())),
            arg(json!(id)),
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

/// Compare two byte strings without leaking, through timing, how many leading
/// bytes matched. Used for every shared secret the platform checks — an inbound
/// webhook, an API token — where an ordinary `==` would let an attacker learn
/// the correct prefix one byte at a time.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// `constant_time_eq` for the gateway module, which reads the inbound SMS secret.
pub fn gateways_constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    constant_time_eq(a, b)
}

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
            arg(json!(session_id)),
            arg(json!(kind)),
            arg(json!(challenge)),
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
    r.get("challenge")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_currency_needs_no_rate() {
        assert_eq!(convert_at(100.0, "USD", "USD", 0.0), Some(100.0));
        assert_eq!(convert_at(100.0, "TZS", "tzs", 0.0), Some(100.0));
    }

    #[test]
    fn a_tzs_invoice_paid_in_usd_divides_by_the_rate() {
        // 2,600,000 TZS at 2,600 TZS/USD is 1,000 USD — never 2,600,000 USD.
        assert_eq!(convert_at(2_600_000.0, "TZS", "USD", 2600.0), Some(1000.0));
        assert_eq!(convert_at(1000.0, "USD", "TZS", 2600.0), Some(2_600_000.0));
    }

    #[test]
    fn round2_settles_to_a_payable_figure() {
        assert_eq!(round2(1234.567), 1234.57);
        assert_eq!(round2(0.005), 0.01);
    }

    #[test]
    fn an_unconvertible_pair_is_refused_rather_than_guessed() {
        assert_eq!(convert_at(100.0, "TZS", "USD", 0.0), None);
        assert_eq!(convert_at(100.0, "TZS", "EUR", 2600.0), None);
        assert_eq!(convert_at(100.0, "GBP", "USD", 2600.0), None);
    }

    /// The general converter pivots through the shilling, so a pair with no
    /// direct rate still converts when both legs are known.
    #[test]
    fn convert_any_pivots_through_the_shilling() {
        // Both legs known: 1,000 USD -> 2,600,000 TZS -> 2,600,000/2,900 EUR.
        let eur = convert_any_at(1000.0, "USD", "EUR", 2600.0, 2900.0).unwrap();
        assert!((eur - 896.55).abs() < 0.01);
        // Same currency is returned unchanged, whatever the rates.
        assert_eq!(convert_any_at(500.0, "USD", "usd", 0.0, 0.0), Some(500.0));
        // A missing leg refuses rather than guesses.
        assert_eq!(convert_any_at(100.0, "USD", "EUR", 2600.0, 0.0), None);
        assert_eq!(convert_any_at(100.0, "GBP", "USD", 0.0, 2600.0), None);
    }
}

/// The pure half of `convert_any`, with both legs passed in so it can be tested
/// without a database.
fn convert_any_at(amount: f64, from: &str, to: &str, from_rate: f64, to_rate: f64) -> Option<f64> {
    let (from, to) = (from.to_uppercase(), to.to_uppercase());
    if from == to {
        return Some(amount);
    }
    if from_rate <= 0.0 || to_rate <= 0.0 {
        return None;
    }
    Some(round2(amount * from_rate / to_rate))
}
