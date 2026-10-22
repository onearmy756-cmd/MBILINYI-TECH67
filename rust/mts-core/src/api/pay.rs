//! Payment endpoints.
//!
//! A buyer picks a gateway, the server records the attempt and pushes the charge
//! to the provider. Confirmation arrives either by the provider calling back
//! (webhook) or by the portal asking the provider directly (`verify`). Both
//! paths land on the same `settle` routine, so an invoice is marked paid once
//! and only once.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::payments::{escrow, gateways, verify as vcode, CATALOGUE};
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "gateways" => gateways_list(c),
        "pay" => start(c).await,
        "verify" => verify(c).await,
        "mine" => mine(c),
        "admin_list" => admin_list(c),
        "escrow" => escrow_info(c),
        "escrow_release" => escrow_release(c),
        "escrow_refund" => escrow_refund(c),
        "webhook" => webhook(c).await,
        "code_check" => code_check(c),
        _ => bad("Unknown payments action."),
    }
}

/// Which gateways to show, with live enabled/configured status.
fn gateways_list(_c: &Ctx) -> Response {
    ok(json!({
        "gateways": crate::payments::available(),
        "currency": settings::get("pay_currency", "TZS"),
    }))
}

/// Start a payment for an invoice.
async fn start(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let invoice_id = c.p("invoiceId");
    let gateway = c.p("gateway");
    if invoice_id.is_empty() || gateway.is_empty() {
        return bad("Choose an invoice and a payment method.");
    }
    let Some(def) = crate::payments::find(&gateway) else {
        return bad("That payment method is not available.");
    };
    if !crate::payments::enabled(def) {
        return bad("That payment method is not switched on yet. Please choose another, or contact us.");
    }
    let Some(inv) = db::row("SELECT * FROM invoices WHERE id = ?", &[arg(json!(invoice_id))]) else {
        return err("Invoice not found.", 404);
    };
    // A client may only pay their own invoice; an admin may pay any.
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let owner = inv.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    if role != "admin" && owner != uid {
        return err("Not your invoice.", 403);
    }
    if inv.get("status").and_then(|v| v.as_str()) == Some("Paid") {
        return bad("Invoice already paid.");
    }
    let amount = inv
        .get("amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let currency = {
        let want = c.p("currency");
        if want.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            want
        }
    };
    if !def.currencies.is_empty() && !def.currencies.contains(&currency.as_str()) {
        return bad(&format!("{} does not accept {currency}.", def.name));
    }
    let payer_name = {
        let n = c.p("payerName");
        if n.is_empty() {
            u.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string()
        } else {
            n
        }
    };
    let payer_phone = {
        let p = c.p("payerPhone");
        if p.is_empty() {
            u.get("phone").and_then(|v| v.as_str()).unwrap_or("").to_string()
        } else {
            p
        }
    };
    let payer_email = u.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let payment_id = crate::payments::create_payment(
        &invoice_id, &uid, &gateway, amount, &currency, &payer_name, &payer_phone, &payer_email,
    );
    let reference = crate::payments::verify::make_reference(
        &crate::payments::get_payment(&payment_id)
            .and_then(|p| p.get("verify_code").and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or_default(),
    );

    let invoice_currency = settings::get("pay_currency", "TZS");
    let outcome = gateways::initiate(
        &gateway, amount, &currency, &invoice_currency,
        &payer_phone, &payer_email, &payer_name, &reference,
    )
    .await;
    if !outcome.ok {
        crate::payments::mark(&payment_id, "Failed", &outcome.reference, &outcome.message);
        return bad(&outcome.message);
    }
    let _ = db::run(
        "UPDATE payments SET reference = ?, note = ? WHERE id = ?",
        &[arg(json!(outcome.reference)), arg(json!(outcome.message)), arg(json!(payment_id))],
    );
    let code = crate::payments::get_payment(&payment_id)
        .and_then(|p| p.get("verify_code").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let pay = crate::payments::get_payment(&payment_id);
    ok(json!({
        "payment": pay.as_ref().map(map::payment),
        "paymentId": payment_id,
        "reference": outcome.reference,
        "instructions": outcome.message,
        "redirect": outcome.redirect,
        "verifyCode": code,
        "gateway": { "key": def.key, "name": def.name, "icon": def.icon, "brand": def.brand },
    }))
}

/// Ask the provider whether the charge settled.
async fn verify(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let pid = c.p("paymentId");
    let Some(p) = crate::payments::get_payment(&pid) else {
        return err("Payment not found.", 404);
    };
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let owner = p.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if role != "admin" && owner != uid {
        return err("Not your payment.", 403);
    }
    let gateway = p.get("gateway").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let reference = p.get("reference").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let outcome = gateways::verify(&gateway, &reference).await;
    if outcome.ok {
        settle(&pid, &gateway, &reference);
    } else {
        crate::payments::mark(&pid, "Pending", &reference, &outcome.message);
    }
    ok(json!({
        "settled": outcome.ok,
        "message": outcome.message,
        "payment": crate::payments::get_payment(&pid).as_ref().map(map::payment),
    }))
}

/// Mark the invoice paid and hold the money in escrow when the invoice belongs
/// to a contract's deposit. Idempotent: a second call changes nothing.
fn settle(payment_id: &str, gateway: &str, reference: &str) {
    let Some(p) = crate::payments::get_payment(payment_id) else {
        return;
    };
    if p.get("status").and_then(|v| v.as_str()) == Some("Settled") {
        return;
    }
    let invoice_id = p.get("invoice_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let amount = p.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let currency = p.get("currency").and_then(|v| v.as_str()).unwrap_or("TZS").to_string();
    crate::payments::mark(payment_id, "Settled", reference, "Confirmed");
    crate::payments::settle_invoice(&invoice_id, gateway, reference);
    // If the invoice belongs to a contract, the money is held, not spent.
    if let Some(inv) = db::row("SELECT tracking_id FROM invoices WHERE id = ?", &[arg(json!(invoice_id))]) {
        let trk = inv.get("tracking_id").and_then(|v| v.as_str()).unwrap_or("");
        if !trk.is_empty() {
            if let Some(ct) = db::row(
                "SELECT id FROM contracts WHERE tracking_id = ?",
                &[arg(json!(trk))],
            ) {
                let cid = ct.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                escrow::hold(payment_id, &cid, amount, &currency);
            }
        }
    }
}

fn mine(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let items: Vec<J> = db::rows(
        "SELECT * FROM payments WHERE user_id = ? ORDER BY created_at DESC",
        &[arg(json!(uid))],
    )
    .into_iter()
    .map(|r| map::payment(&r))
    .collect();
    ok(json!({ "payments": items }))
}

fn admin_list(c: &Ctx) -> Response {
    let Ok(_a) = c.require_admin() else {
        return util::forbidden("Admin access required.");
    };
    let items: Vec<J> = db::rows("SELECT * FROM payments ORDER BY created_at DESC", &[])
        .into_iter()
        .map(|r| map::payment(&r))
        .collect();
    ok(json!({
        "payments": items,
        "escrowHeld": escrow::total_held(),
        "gateways": crate::payments::available(),
    }))
}

fn escrow_info(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let cid = c.p("contractId");
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    if role != "admin" {
        // A client may only inspect their own contract's escrow.
        let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let owned = db::row(
            "SELECT c.id FROM contracts c JOIN requests r ON r.tracking_id = c.tracking_id
             WHERE c.id = ? AND r.user_id = ?",
            &[arg(json!(cid)), arg(json!(uid))],
        );
        if owned.is_none() {
            return err("Not your contract.", 403);
        }
    }
    ok(json!({
        "balance": escrow::balance(&cid),
        "ledger": escrow::ledger(&cid),
    }))
}

fn escrow_release(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let cid = c.p("contractId");
    let amount = c.p("amount").parse::<f64>().unwrap_or(0.0);
    let currency = {
        let x = c.p("currency");
        if x.is_empty() { settings::get("pay_currency", "TZS") } else { x }
    };
    match escrow::release(&cid, amount, &currency, &c.p("note")) {
        Some(id) => ok(json!({ "released": true, "entryId": id, "balance": escrow::balance(&cid) })),
        None => bad("That amount is more than is held for this contract."),
    }
}

fn escrow_refund(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let cid = c.p("contractId");
    let amount = c.p("amount").parse::<f64>().unwrap_or(0.0);
    let currency = {
        let x = c.p("currency");
        if x.is_empty() { settings::get("pay_currency", "TZS") } else { x }
    };
    match escrow::refund(&cid, amount, &currency, &c.p("note")) {
        Some(id) => ok(json!({ "refunded": true, "entryId": id, "balance": escrow::balance(&cid) })),
        None => bad("That amount is more than is held for this contract."),
    }
}

/// The provider's callback. The payload is stored before it is acted on, so a
/// disputed settlement can be re-checked against exactly what was sent.
async fn webhook(c: &Ctx) -> Response {
    let gateway = c.p("gateway");
    let reference = {
        let r = c.p("reference");
        if r.is_empty() { c.p("orderReference") } else { r }
    };
    let _ = db::run(
        "INSERT INTO gateway_events (id, gateway, reference, payload, created_at) VALUES (?,?,?,?,?)",
        &[
            arg(json!(util::new_id("GE-"))), arg(json!(gateway)), arg(json!(reference)),
            arg(json!(c.body.to_string())), arg(json!(util::now())),
        ],
    );
    if reference.is_empty() {
        return bad("No reference in the callback.");
    }
    let Some(p) = db::row(
        "SELECT * FROM payments WHERE reference = ? OR verify_code = ?",
        &[arg(json!(reference)), arg(json!(reference))],
    ) else {
        return ok(json!({ "received": true, "matched": false }));
    };
    let pid = p.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    // Ask the provider rather than trusting the callback body: a webhook can be
    // forged, and only the provider can say whether the money really moved.
    let outcome = gateways::verify(&gateway, &reference).await;
    if outcome.ok {
        settle(&pid, &gateway, &reference);
    }
    ok(json!({ "received": true, "matched": true, "settled": outcome.ok }))
}

/// A payer can quote their verification code; this resolves it to the payment.
fn code_check(c: &Ctx) -> Response {
    let code = c.p("code");
    if code.is_empty() {
        return bad("Enter the verification code.");
    }
    match db::row("SELECT * FROM payments WHERE verify_code = ?", &[arg(json!(code))]) {
        Some(p) => ok(json!({
            "payment": map::payment(&p),
            "gateway": p.get("gateway").and_then(|v| v.as_str()).unwrap_or(""),
        })),
        None => err("No payment matches that code.", 404),
    }
}

/// Unused guard so the catalogue import is not flagged when a build omits it.
#[allow(dead_code)]
fn _catalogue_len() -> usize {
    CATALOGUE.len()
}

#[allow(dead_code)]
fn _vcode(a: &str, b: &str) -> bool {
    vcode::matches(a, b)
}
