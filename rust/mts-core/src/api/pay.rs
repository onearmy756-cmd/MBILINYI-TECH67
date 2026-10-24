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
        "rates" => rates(c),
        "rates_refresh" => rates_refresh(c).await,
        "pay" => start(c).await,
        "verify" => verify(c).await,
        "mine" => mine(c),
        "admin_list" => admin_list(c),
        "escrow" => escrow_info(c),
        "escrow_release" => escrow_release(c),
        "escrow_refund" => escrow_refund(c),
        "payslips" => payslips(c),
        "payslip_pay" => payslip_pay(c),
        "payroll_preview" => payroll_preview(c),
        "tax_return" => tax_return(c),
        "accept_delivery" => accept_delivery(c),
        "settle_manual" => settle_manual(c),
        "webhook" => webhook(c).await,
        "stripe_webhook" => stripe_webhook(c).await,
        "code_check" => code_check(c),
        _ => bad("Unknown payments action."),
    }
}

/// Which gateways to show, with live enabled/configured status.
fn gateways_list(_c: &Ctx) -> Response {
    ok(json!({
        "gateways": crate::payments::available(),
        "currency": settings::get("pay_currency", "TZS"),
        "fx": { "usd_tzs": crate::payments::usd_rate() },
    }))
}

/// The current exchange rates and where they came from, so the admin can see
/// whether they are live or hand-set.
fn rates(_c: &Ctx) -> Response {
    let mut out = Vec::new();
    for cur in crate::payments::QUOTE_CURRENCIES {
        out.push(json!({
            "currency": cur,
            "tzsPerUnit": crate::payments::rate_for(cur),
        }));
    }
    ok(json!({
        "base": "TZS",
        "rates": out,
        "source": settings::get("fx_source", "manual"),
        "updatedAt": settings::get("fx_updated_at", ""),
    }))
}

/// Pull live mid-market rates from Wise and store them. Admin-only: it writes
/// the figures every invoice is priced against.
async fn rates_refresh(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    match crate::payments::refresh_rates_from_wise().await {
        Ok(updated) => ok(json!({ "refreshed": true, "updated": updated })),
        Err(e) => err(&e, 502),
    }
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
        return bad(
            "That payment method is not switched on yet. Please choose another, or contact us.",
        );
    }
    let Some(inv) = db::row(
        "SELECT * FROM invoices WHERE id = ?",
        &[arg(json!(invoice_id))],
    ) else {
        return err("Invoice not found.", 404);
    };
    // A client may only pay their own invoice; an admin may pay any.
    let uid = u
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let owner = inv.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    if role != "admin" && owner != uid {
        return err("Not your invoice.", 403);
    }
    if inv.get("status").and_then(|v| v.as_str()) == Some("Paid") {
        return bad("Invoice already paid.");
    }
    let amount = inv.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
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
            u.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            n
        }
    };
    let payer_phone = {
        let p = c.p("payerPhone");
        if p.is_empty() {
            u.get("phone")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            p
        }
    };
    let payer_email = u
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // The invoice's own currency is the base, not the platform default, so an
    // invoice written in USD keeps its meaning after the default changes.
    let invoice_currency = {
        let stored = inv.get("currency").and_then(|v| v.as_str()).unwrap_or("");
        if stored.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            stored.to_string()
        }
    };
    let charge = if currency.eq_ignore_ascii_case(&invoice_currency) {
        amount
    } else {
        // Any quoted pair, pivoting through the shilling, so a USD invoice can
        // be settled in EUR or KES as well as TZS.
        match crate::payments::convert_any(amount, &invoice_currency, &currency) {
            Some(v) => crate::payments::round2(v),
            None => {
                return bad(&format!(
                    "Paying a {invoice_currency} invoice in {currency} needs an exchange rate. Ask the CEO to set it in Admin → Settings, or refresh the rates from Wise."
                ))
            }
        }
    };

    let payment_id = crate::payments::create_payment(
        &invoice_id,
        &uid,
        &gateway,
        charge,
        &currency,
        &payer_name,
        &payer_phone,
        &payer_email,
    );
    let reference = crate::payments::verify::make_reference(
        &crate::payments::get_payment(&payment_id)
            .and_then(|p| {
                p.get("verify_code")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_default(),
    );

    let outcome = gateways::initiate(
        &gateway,
        charge,
        &currency,
        &invoice_currency,
        &payer_phone,
        &payer_email,
        &payer_name,
        &reference,
    )
    .await;
    if !outcome.ok {
        crate::payments::mark(&payment_id, "Failed", &outcome.reference, &outcome.message);
        return bad(&outcome.message);
    }
    let _ = db::run(
        "UPDATE payments SET reference = ?, note = ? WHERE id = ?",
        &[
            arg(json!(outcome.reference)),
            arg(json!(outcome.message)),
            arg(json!(payment_id)),
        ],
    );
    let code = crate::payments::get_payment(&payment_id)
        .and_then(|p| {
            p.get("verify_code")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
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
    let gateway = p
        .get("gateway")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let reference = p
        .get("reference")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
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
    let invoice_id = p
        .get("invoice_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let amount = p.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let currency = p
        .get("currency")
        .and_then(|v| v.as_str())
        .unwrap_or("TZS")
        .to_string();
    crate::payments::mark(payment_id, "Settled", reference, "Confirmed");
    crate::payments::settle_invoice(&invoice_id, gateway, reference);
    // If the invoice belongs to a contract, the money is held, not spent.
    if let Some(inv) = db::row(
        "SELECT tracking_id FROM invoices WHERE id = ?",
        &[arg(json!(invoice_id))],
    ) {
        let trk = inv
            .get("tracking_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !trk.is_empty() {
            if let Some(ct) = db::row(
                "SELECT id FROM contracts WHERE tracking_id = ?",
                &[arg(json!(trk))],
            ) {
                let cid = ct
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
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

/// The client accepts the delivery. This is the moment the money leaves escrow
/// and lands in the company account: the client confirms the work, the held
/// funds are released, and the contract is closed. It is the client's decision,
/// not the admin's — which is what makes the escrow worth anything to them.
fn accept_delivery(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let cid = c.p("contractId");
    if cid.is_empty() {
        return bad("Name the contract you are accepting.");
    }
    let Some(ct) = db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(cid))]) else {
        return err("Contract not found.", 404);
    };
    // Only the client on the contract may accept it. An admin can do so on the
    // client's behalf only when the request is recorded, so the trail is clear.
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let tracking = ct.get("tracking_id").and_then(|v| v.as_str()).unwrap_or("");
    let owner = db::row(
        "SELECT user_id FROM requests WHERE tracking_id = ?",
        &[arg(json!(tracking))],
    )
    .and_then(|r| {
        r.get("user_id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    })
    .unwrap_or_default();
    if role != "admin" && owner != uid {
        return err("This contract is not yours.", 403);
    }
    if ct.get("status").and_then(|v| v.as_str()) == Some("Completed") {
        return bad("This delivery has already been accepted.");
    }
    let held = escrow::balance(&cid);
    if held <= 0.0 {
        return bad("Nothing is held in escrow for this contract, so there is nothing to release.");
    }
    let currency = {
        let x = ct.get("currency").and_then(|v| v.as_str()).unwrap_or("");
        if x.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            x.to_string()
        }
    };
    let who = u
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("the client");
    let note = format!("Accepted by {who}");
    // Release the whole balance to the company account. A release entry is a
    // row, so the escrow balance stays provable from its own history.
    let Some(entry) = escrow::release(&cid, held, &currency, &note) else {
        return bad("The release could not be recorded. Please try again.");
    };
    let _ = db::run(
        "UPDATE contracts SET status = ?, client_signed_at = COALESCE(client_signed_at, ?) WHERE id = ?",
        &[arg(json!("Completed")), arg(json!(util::now())), arg(json!(cid))],
    );
    // The request the contract belongs to is complete, and the client's portal
    // shows it. Their progress bar reaches 100%.
    let _ = db::run(
        "UPDATE requests SET status = 'Completed' WHERE tracking_id = ?",
        &[arg(json!(tracking))],
    );
    // Tell the client their acceptance was recorded, and the company that money
    // moved. Both are real sends when a gateway is configured.
    notify_client_accepted(tracking, held, &currency, &u);

    // Pay the worker who carried out the job. The gross is the worker's share
    // of what was released; PAYE, NSSF and WCF are deducted and the net is what
    // the worker takes home. The company keeps the remainder as its margin.
    let payslip = crate::payroll::raise_for_contract(&cid, held, "Client accepted delivery");

    ok(json!({
        "accepted": true,
        "entryId": entry,
        "released": held,
        "currency": currency,
        "balance": escrow::balance(&cid),
        "companyAccount": crate::security::company_account(),
        "payslip": payslip,
        "message": "Asante. Umechukua kazi; malipo yametolewa kutoka escrow kwenda kwenye akaunti ya kampuni.",
    }))
}

/// Tell the client, in their own language, that their acceptance landed.
fn notify_client_accepted(tracking: &str, amount: f64, currency: &str, client: &db::Row) {
    let phone = util::s(client.get("phone").unwrap_or(&J::Null));
    if phone.trim().is_empty() {
        return;
    }
    let amount_s = if currency.eq_ignore_ascii_case("USD") {
        format!("USD {:.2}", amount)
    } else {
        util::thousands(amount)
    };
    let text = format!(
        "Asante! Umekubali kazi {tracking}. Malipo ya {amount_s} {currency} yametolewa kutoka escrow. Karibu tena."
    );
    let log_id = util::new_id("SMS-");
    let _ = db::run(
        "INSERT INTO sms_log (id, phone, message, request_id, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(log_id)), arg(json!(phone)), arg(json!(text)), arg(json!(tracking)),
            arg(json!("delivery_accepted")), arg(json!("queued")), arg(json!(util::now())),
        ],
    );
    crate::sms::spawn_send(log_id, phone, text);
}

/// The Stripe callback. Unlike the generic `webhook`, this one is authenticated:
/// the signature Stripe sends is checked against the endpoint's signing secret
/// before anything is acted on, and the timestamp is bounded so a captured
/// callback cannot be replayed.
pub async fn stripe_webhook(c: &Ctx) -> Response {
    // The raw bytes, not a re-serialised copy: Stripe signs exactly what it
    // sent, and re-encoding the JSON would change the key order and break the
    // HMAC even for a genuine callback.
    let body = c.raw.clone();
    let sig_header = c
        .headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    // Record the raw payload first, so a disputed settlement can be re-checked
    // against exactly what was sent.
    let reference = stripe_event_reference(&body);
    let _ = db::run(
        "INSERT INTO gateway_events (id, gateway, reference, payload, created_at) VALUES (?,?,?,?,?)",
        &[
            arg(json!(util::new_id("GE-"))), arg(json!("stripe")), arg(json!(reference.clone())),
            arg(json!(body)), arg(json!(util::now())),
        ],
    );
    if let Err(e) = gateways::verify_stripe_webhook(&sig_header, &body) {
        return err(&e, 400);
    }
    if reference.is_empty() {
        return bad("The Stripe event carries no reference this platform recognises.");
    }
    let Some(p) = db::row(
        "SELECT * FROM payments WHERE reference = ? OR verify_code = ?",
        &[arg(json!(reference)), arg(json!(reference))],
    ) else {
        // A genuine event for something we do not know about; accept it so
        // Stripe stops retrying.
        return ok(json!({ "received": true, "matched": false }));
    };
    let pid = p
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // Ask Stripe rather than trusting the body: a webhook can be genuine and
    // still be about a session that never completed.
    let outcome = gateways::verify("stripe", &reference).await;
    if outcome.ok {
        settle(&pid, "stripe", &reference);
    }
    ok(json!({ "received": true, "matched": true, "settled": outcome.ok }))
}

/// Pull the checkout session id out of a Stripe event body. Handles both the
/// event envelope (`data.object.id`) and a bare session object.
fn stripe_event_reference(body: &str) -> String {
    let Ok(v) = serde_json::from_str::<J>(body) else {
        return String::new();
    };
    for path in [
        "data.object.id",
        "data.object.client_reference_id",
        "object.id",
        "client_reference_id",
        "id",
    ] {
        let mut cur = &v;
        let mut found = true;
        for seg in path.split('.') {
            match cur.get(seg) {
                Some(n) => cur = n,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(s) = cur.as_str() {
                if !s.is_empty() {
                    return s.to_string();
                }
            }
        }
    }
    String::new()
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
        if x.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            x
        }
    };
    match escrow::release(&cid, amount, &currency, &c.p("note")) {
        Some(id) => {
            // The worker who did the job is paid from what was released, with
            // the statutory deductions taken out. A payslip is raised for every
            // release, so the money-out trail is complete.
            let payslip = crate::payroll::raise_for_contract(&cid, amount, "Escrow release");
            ok(json!({
                "released": true,
                "entryId": id,
                "balance": escrow::balance(&cid),
                "payslip": payslip,
            }))
        }
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
        if x.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            x
        }
    };
    match escrow::refund(&cid, amount, &currency, &c.p("note")) {
        Some(id) => {
            ok(json!({ "refunded": true, "entryId": id, "balance": escrow::balance(&cid) }))
        }
        None => bad("That amount is more than is held for this contract."),
    }
}

/// Record that a payment arrived out-of-band — a bank transfer or a mobile-money
/// deposit the provider did not call back about. The admin names the reference
/// they matched against the statement. This is the honest way to settle the
/// gateways that are confirmed by hand: the money still lands in escrow, so the
/// client's protection is the same as for a card payment.
fn settle_manual(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let payment_id = c.p("paymentId");
    if payment_id.is_empty() {
        return bad("Name the payment you are confirming.");
    }
    let Some(p) = crate::payments::get_payment(&payment_id) else {
        return err("Payment not found.", 404);
    };
    if p.get("status").and_then(|v| v.as_str()) == Some("Settled") {
        return bad("This payment is already settled.");
    }
    let reference = {
        let r = c.p("reference");
        if r.is_empty() {
            p.get("reference")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            r
        }
    };
    let gateway = p
        .get("gateway")
        .and_then(|v| v.as_str())
        .unwrap_or("manual")
        .to_string();
    // `settle` marks the invoice paid and holds the money in escrow, exactly as
    // a webhook would. Reusing it keeps one money path, not two.
    settle(&payment_id, &gateway, &reference);
    let after = crate::payments::get_payment(&payment_id).unwrap_or_default();
    ok(json!({
        "settled": true,
        "paymentId": payment_id,
        "reference": reference,
        "status": after.get("status").and_then(|v| v.as_str()).unwrap_or("Settled"),
        "note": "Recorded against the statement. The funds are held in escrow until the client accepts.",
    }))
}

/* ------------------------------ payroll ------------------------------ */

/// Every payslip, or the ones belonging to one worker. The admin reads them
/// here to know who is owed what; a worker reads only their own.
fn payslips(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let employee_id = if role == "admin" {
        c.p("employeeId")
    } else {
        // A worker sees only their own payslips, keyed by the account they
        // signed in with. Anything else would leak a colleague's pay.
        let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
        crate::wafanyakazi::employee_for_account(uid).unwrap_or_default()
    };
    let rows = crate::payroll::list_for(&employee_id);
    ok(json!({
        "payslips": rows,
        "outstanding": crate::payroll::outstanding(),
        "taxWithheld": crate::payroll::tax_withheld(),
        "workerShare": crate::settings::num("payroll_worker_share", 0.70),
    }))
}

/// Mark a payslip as paid. The money itself moves outside the system (bank or
/// mobile money), but the record of the payment is what makes the payroll
/// auditable, so the action is admin-only and leaves a timestamp.
fn payslip_pay(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    if id.is_empty() {
        return bad("Name the payslip to mark as paid.");
    }
    if crate::payroll::mark_paid(&id) {
        ok(json!({ "paid": true, "id": id }))
    } else {
        err("Payslip not found.", 404)
    }
}

/// What a gross figure pays out, so the admin can see the deductions before
/// committing to a figure.
fn payroll_preview(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let gross = c.p("gross").parse::<f64>().unwrap_or(0.0);
    ok(json!({ "payslip": crate::payroll::compute(gross) }))
}

/// The monthly return to TRA: what the company withheld and must remit.
fn tax_return(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let withheld = crate::payroll::tax_withheld();
    let total = withheld["paye"].as_f64().unwrap_or(0.0)
        + withheld["nssf"].as_f64().unwrap_or(0.0)
        + withheld["wcf"].as_f64().unwrap_or(0.0);
    ok(json!({
        "withheld": withheld,
        "totalRemittable": total,
        "currency": settings::get("pay_currency", "TZS"),
        "note": "PAYE and WCF are remitted to TRA; NSSF to the social security fund.",
    }))
}

/// The provider's callback. The payload is stored before it is acted on, so a
/// disputed settlement can be re-checked against exactly what was sent.
///
/// For Stripe the signature is verified before anything is believed; for the
/// Tanzanian providers, whose callbacks are authenticated by the reference
/// alone, the charge is confirmed with the provider rather than trusted.
async fn webhook(c: &Ctx) -> Response {
    let gateway = c.p("gateway");
    let reference = {
        let r = c.p("reference");
        if r.is_empty() {
            c.p("orderReference")
        } else {
            r
        }
    };
    let _ = db::run(
        "INSERT INTO gateway_events (id, gateway, reference, payload, created_at) VALUES (?,?,?,?,?)",
        &[
            arg(json!(util::new_id("GE-"))), arg(json!(gateway)), arg(json!(reference)),
            arg(json!(c.raw)), arg(json!(util::now())),
        ],
    );
    if gateway.eq_ignore_ascii_case("stripe") {
        // A Stripe callback must prove it came from Stripe.
        let sig = c
            .headers
            .get("stripe-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if let Err(e) = gateways::verify_stripe_webhook(&sig, &c.raw) {
            return err(&e, 400);
        }
    }
    if reference.is_empty() {
        return bad("No reference in the callback.");
    }
    let Some(p) = db::row(
        "SELECT * FROM payments WHERE reference = ? OR verify_code = ?",
        &[arg(json!(reference)), arg(json!(reference))],
    ) else {
        return ok(json!({ "received": true, "matched": false }));
    };
    let pid = p
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
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
    match db::row(
        "SELECT * FROM payments WHERE verify_code = ?",
        &[arg(json!(code))],
    ) {
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
