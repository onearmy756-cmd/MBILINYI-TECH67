//! Notifications: the one place a message to a person is produced.
//!
//! Every notification the platform sends — a request moved, a contract is ready,
//! a payment landed — goes through here. The module decides which channels to
//! use (SMS to a phone, mail to an address), writes the audit row for each, and
//! hands the actual sending to the gateway in the background so no HTTP handler
//! waits on a provider.
//!
//! A notification is never claimed to have been sent. The log records what the
//! gateway replied, and an unconfigured channel says so.

use crate::db;
use crate::settings;
use crate::util;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// Send an SMS and log it. The row is written first, so the admin's SMS log
/// shows the message even if the gateway is slow or refuses it.
pub fn sms(phone: &str, message: &str, request_id: &str, kind: &str) {
    if phone.trim().is_empty() {
        return;
    }
    let id = util::new_id("SMS-");
    let _ = db::run(
        "INSERT INTO sms_log (id, phone, message, request_id, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(phone)), arg(json!(message)), arg(json!(request_id)),
            arg(json!(kind)), arg(json!("queued")), arg(json!(util::now())),
        ],
    );
    crate::sms::spawn_send(id, phone.to_string(), message.to_string());
}

/// Send an email and log it.
pub fn email(to: &str, subject: &str, body: &str, kind: &str) {
    if to.trim().is_empty() || !to.contains('@') {
        return;
    }
    crate::mail::spawn(to.to_string(), subject.to_string(), body.to_string(), kind);
}

/// Tell the admin something needs a decision — a new request, an inbound SMS.
/// The admin's own address and number are read from settings, so the CEO can
/// change where the alerts go without a rebuild.
pub fn admin_alert(subject: &str, message: &str, kind: &str) {
    let phone = settings::get("admin_alert_phone", "");
    if !phone.trim().is_empty() {
        sms(&phone, message, "", kind);
    }
    let to = settings::get("admin_alert_email", "");
    if !to.trim().is_empty() {
        self::email(&to, subject, message, kind);
    }
}

/// A new request arrived. The admin is told by SMS and by mail, and the client
/// is told their request was received.
pub fn request_created(req: &db::Row) {
    let tracking = util::s(req.get("tracking_id").unwrap_or(&J::Null));
    let service = util::s(req.get("service").unwrap_or(&J::Null));
    let title = util::s(req.get("title").unwrap_or(&J::Null));
    let name = util::s(req.get("name").unwrap_or(&J::Null));
    let client_phone = util::s(req.get("phone").unwrap_or(&J::Null));
    let client_email = util::s(req.get("email").unwrap_or(&J::Null));

    admin_alert(
        &format!("Request mpya {tracking} — {service}"),
        &format!(
            "Ombi jipya {tracking} ({service}) kutoka {name}: {title}. Fungua Admin → Requests kuidhinisha na kusaini mkataba."
        ),
        "admin_new_request",
    );
    sms(
        &client_phone,
        &format!(
            "Asante {name}! Ombi lako {tracking} limepokelewa. Tutakujulisha hatua inayofuata."
        ),
        &tracking,
        "request_received",
    );
    email(
        &client_email,
        &format!("Request {tracking} received — Mbilinyi Tech Solutions"),
        &format!(
            "Habari {name},<br><br>Tumepokea ombi lako <b>{tracking}</b> kwa huduma ya <b>{service}</b>.<br>\
             Unaweza kufuatilia maendeleo kwenye portal kwa kutumia tracking ID hiyo.<br><br>Asante,<br>Mbilinyi Tech Solutions"
        ),
        "request_received",
    );
}

/// A contract is ready. Both parties are told, and the admin is asked to sign.
pub fn contract_ready(contract: &db::Row) {
    let cid = util::s(contract.get("id").unwrap_or(&J::Null));
    let title = util::s(contract.get("project_title").unwrap_or(&J::Null));
    let tracking = util::s(contract.get("tracking_id").unwrap_or(&J::Null));
    let client = util::s(contract.get("client_name").unwrap_or(&J::Null));
    let client_email = db::row(
        "SELECT email, phone FROM requests WHERE tracking_id = ?",
        &[arg(json!(tracking))],
    );
    let to = client_email
        .as_ref()
        .map(|r| util::s(r.get("email").unwrap_or(&J::Null)))
        .unwrap_or_default();
    let phone = client_email
        .as_ref()
        .map(|r| util::s(r.get("phone").unwrap_or(&J::Null)))
        .unwrap_or_default();

    admin_alert(
        &format!("Contract {cid} ready to sign"),
        &format!(
            "Contract {cid} for {client} ({title}) is ready. Open Contracts to review and sign."
        ),
        "admin_contract_ready",
    );
    email(
        &to,
        &format!("Contract {cid} ready to sign"),
        &format!("Habari {client},<br><br>Mkataba <b>{cid}</b> wa mradi <b>{title}</b> umekamilika. Ingia portal kusoma na kupiga saini."),
        "contract_ready",
    );
    sms(
        &phone,
        &format!("Mkataba {cid} wa mradi {title} upo tayari. Ingia portal kuusoma na kusaini."),
        &tracking,
        "contract_ready",
    );
}

/// A payment settled. The client is thanked and told the money is held in
/// escrow until they accept the delivery.
pub fn payment_settled(payment: &db::Row) {
    let amount = payment
        .get("amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let currency = util::s(payment.get("currency").unwrap_or(&J::Null));
    let reference = util::s(payment.get("reference").unwrap_or(&J::Null));
    let payer_email = util::s(payment.get("payer_email").unwrap_or(&J::Null));
    let phone = util::s(payment.get("payer_phone").unwrap_or(&J::Null));
    let amount_s = if currency.eq_ignore_ascii_case("USD") {
        format!("USD {amount:.2}")
    } else {
        format!("{currency} {}", util::thousands(amount))
    };
    email(
        &payer_email,
        "Payment received — Mbilinyi Tech Solutions",
        &format!(
            "Tumepokea malipo yako ya <b>{amount_s}</b> (kumbukumbu {reference}).<br>\
             Fedha zimehifadhiwa (escrow) hadi utakapokubali kazi imekamilika."
        ),
        "payment_received",
    );
    sms(
        &phone,
        &format!("Malipo yako ya {amount_s} yamepokelewa (kumbukumbu {reference}). Fedha ziko escrow hadi ukubali kazi."),
        "",
        "payment_received",
    );
}

/// The alert destinations, for the admin panel.
pub fn status() -> J {
    json!({
        "adminPhone": settings::get("admin_alert_phone", ""),
        "adminEmail": settings::get("admin_alert_email", ""),
        "smsConfigured": crate::sms::configured(),
        "emailConfigured": crate::mail::status().get("configured").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

/// Unused guard.
#[allow(dead_code)]
fn _keep(_: fn(&str, &str, &str, &str)) {
    let _ = crate::settings::get;
}
