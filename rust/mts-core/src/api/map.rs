//! Response shapes.
//!
//! The frontend reads camelCase keys, so every row is mapped here rather than
//! sent raw. Some mappers also emit the short aliases the original PHP mappers
//! produced (`img` beside `imageUrl`, `s` beside `statLabel`, `r`/`t` beside
//! `role`/`text`), because `app.js` reads both spellings in different places.

use crate::db;
use crate::util;
use serde_json::{json, Value as J};

fn gs(r: &db::Row, k: &str) -> String {
    r.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn gf(r: &db::Row, k: &str) -> f64 {
    match r.get(k) {
        Some(J::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(J::String(s)) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn gi(r: &db::Row, k: &str) -> i64 {
    match r.get(k) {
        Some(J::Number(n)) => n.as_i64().unwrap_or(0),
        Some(J::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn iso_of(r: &db::Row, k: &str) -> String {
    util::iso(r.get(k).and_then(|v| v.as_str()))
}

/// A boolean column, which SQLite stores as 0/1.
fn gb(r: &db::Row, k: &str) -> bool {
    match r.get(k) {
        Some(J::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(J::String(s)) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => true,
    }
}

/// A JSON-encoded list column, returned as an array.
fn json_list(r: &db::Row, k: &str) -> J {
    let raw = gs(r, k);
    if raw.is_empty() {
        return J::Array(vec![]);
    }
    serde_json::from_str::<J>(&raw).unwrap_or(J::Array(vec![]))
}

pub fn user(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "name": gs(r, "name"),
        "email": gs(r, "email"),
        "phone": gs(r, "phone"),
        "company": gs(r, "company"),
        "role": gs(r, "role"),
        "status": gs(r, "status"),
        "createdAt": iso_of(r, "created_at"),
    })
}

pub fn request(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "trackingId": gs(r, "tracking_id"),
        "userId": gs(r, "user_id"),
        "name": gs(r, "name"),
        "email": gs(r, "email"),
        "phone": gs(r, "phone"),
        "company": gs(r, "company"),
        "service": gs(r, "service"),
        "title": gs(r, "title"),
        "description": gs(r, "description"),
        "tech": json_list(r, "tech"),
        "serviceDetails": json_list(r, "service_details"),
        "platform": gs(r, "platform"),
        "budget": gs(r, "budget"),
        "deadline": gs(r, "deadline"),
        "priority": gs(r, "priority"),
        "status": gs(r, "status"),
        "adminNote": gs(r, "admin_note"),
        "createdAt": iso_of(r, "created_at"),
    })
}

pub fn quote(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "requestId": gs(r, "request_id"),
        "trackingId": gs(r, "tracking_id"),
        "userId": gs(r, "user_id"),
        "items": json_list(r, "items"),
        "subtotal": gf(r, "subtotal"),
        "vat": gf(r, "vat"),
        "total": gf(r, "total"),
        "currency": gs(r, "currency"),
        "validUntil": iso_of(r, "valid_until"),
        "status": gs(r, "status"),
        "adminMessage": gs(r, "admin_message"),
        "createdAt": iso_of(r, "created_at"),
    })
}

/// A signature is `null` when it has not been given, which is how the frontend
/// decides whether to offer the signature pad.
fn sig(r: &db::Row, k: &str) -> J {
    let v = gs(r, k);
    if v.is_empty() {
        J::Null
    } else {
        J::String(v)
    }
}

pub fn contract(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "requestId": gs(r, "request_id"),
        "quoteId": gs(r, "quote_id"),
        "trackingId": gs(r, "tracking_id"),
        "clientName": gs(r, "client_name"),
        "projectTitle": gs(r, "project_title"),
        "scope": gs(r, "scope"),
        "duration": gs(r, "duration"),
        "total": gf(r, "total"),
        "status": gs(r, "status"),
        "clientSig": sig(r, "client_sig"),
        "clientSignedAt": iso_of(r, "client_signed_at"),
        "adminSig": sig(r, "admin_sig"),
        "adminSignedAt": iso_of(r, "admin_signed_at"),
        "createdAt": iso_of(r, "created_at"),
    })
}

pub fn invoice(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "trackingId": gs(r, "tracking_id"),
        "userId": gs(r, "user_id"),
        "title": gs(r, "title"),
        "amount": gf(r, "amount"),
        "status": gs(r, "status"),
        "method": gs(r, "method"),
        "dueDate": iso_of(r, "due_date"),
        "createdAt": iso_of(r, "created_at"),
    })
}

/// A ticket with its replies. Each reply carries the short keys the frontend
/// reads (`by`, `text`, `at`).
pub fn ticket(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "userId": gs(r, "user_id"),
        "name": gs(r, "name"),
        "email": gs(r, "email"),
        "subject": gs(r, "subject"),
        "message": gs(r, "message"),
        "status": gs(r, "status"),
        "createdAt": iso_of(r, "created_at"),
        "replies": replies(&gs(r, "id")),
    })
}

fn replies(ticket_id: &str) -> J {
    let rows = db::rows(
        "SELECT * FROM ticket_replies WHERE ticket_id = ? ORDER BY created_at ASC",
        &[db::from_json(json!(ticket_id))],
    );
    J::Array(
        rows.iter()
            .map(|r| {
                json!({
                    "by": gs(r, "by_name"),
                    "text": gs(r, "body"),
                    "at": iso_of(r, "created_at"),
                })
            })
            .collect(),
    )
}

pub fn news(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "title": gs(r, "title"),
        "category": gs(r, "category"),
        "excerpt": gs(r, "excerpt"),
        "body": gs(r, "body"),
        "imageUrl": gs(r, "image_url"),
        "img": gs(r, "image_url"),
        "isActive": gb(r, "is_active"),
        "createdAt": iso_of(r, "created_at"),
        "updatedAt": iso_of(r, "updated_at"),
    })
}

pub fn portfolio(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "title": gs(r, "title"),
        "category": gs(r, "category"),
        "description": gs(r, "description"),
        "imageUrl": gs(r, "image_url"),
        "img": gs(r, "image_url"),
        "statLabel": gs(r, "stat_label"),
        "s": gs(r, "stat_label"),
        "linkUrl": gs(r, "link_url"),
        "sortOrder": gi(r, "sort_order"),
        "isActive": gb(r, "is_active"),
        "createdAt": iso_of(r, "created_at"),
        "updatedAt": iso_of(r, "updated_at"),
    })
}

pub fn testimonial(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "name": gs(r, "name"),
        "role": gs(r, "role"),
        "r": gs(r, "role"),
        "text": gs(r, "text"),
        "t": gs(r, "text"),
        "stars": gi(r, "stars"),
        "s": gi(r, "stars"),
        "sortOrder": gi(r, "sort_order"),
        "isActive": gb(r, "is_active"),
        "createdAt": iso_of(r, "created_at"),
        "updatedAt": iso_of(r, "updated_at"),
    })
}

pub fn team(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "name": gs(r, "name"),
        "role": gs(r, "role"),
        "roleSw": gs(r, "role_sw"),
        "bio": gs(r, "bio"),
        "bioSw": gs(r, "bio_sw"),
        "imageUrl": gs(r, "image_url"),
        "img": gs(r, "image_url"),
        "email": gs(r, "email"),
        "phone": gs(r, "phone"),
        "linkedin": gs(r, "linkedin"),
        "sortOrder": gi(r, "sort_order"),
        "isActive": gb(r, "is_active"),
        "createdAt": iso_of(r, "created_at"),
        "updatedAt": iso_of(r, "updated_at"),
    })
}

pub fn feedback(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "name": gs(r, "name"),
        "email": gs(r, "email"),
        "category": gs(r, "category"),
        "subject": gs(r, "subject"),
        "message": gs(r, "message"),
        "rating": gi(r, "rating"),
        "status": gs(r, "status"),
        "createdAt": iso_of(r, "created_at"),
    })
}

pub fn payment(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "invoiceId": gs(r, "invoice_id"),
        "userId": gs(r, "user_id"),
        "gateway": gs(r, "gateway"),
        "amount": gf(r, "amount"),
        "currency": gs(r, "currency"),
        "status": gs(r, "status"),
        "reference": gs(r, "reference"),
        "verifyCode": gs(r, "verify_code"),
        "note": gs(r, "note"),
        "createdAt": iso_of(r, "created_at"),
    })
}

pub fn plan(r: &db::Row) -> J {
    json!({
        "id": gs(r, "id"),
        "code": gs(r, "code"),
        "group": gs(r, "group_name"),
        "name": gs(r, "name"),
        "price": gf(r, "price"),
        "currency": gs(r, "currency"),
        "period": gs(r, "period"),
        "features": json_list(r, "features"),
        "sort": gi(r, "sort_order"),
    })
}

/// Every mapper as a lookup, so a list endpoint can be driven by name.
pub fn by_name(kind: &str, r: &db::Row) -> J {
    match kind {
        "user" => user(r),
        "request" => request(r),
        "quote" => quote(r),
        "contract" => contract(r),
        "invoice" => invoice(r),
        "ticket" => ticket(r),
        "news" => news(r),
        "portfolio" => portfolio(r),
        "testimonial" => testimonial(r),
        "team" => team(r),
        "feedback" => feedback(r),
        "payment" => payment(r),
        "plan" => plan(r),
        _ => J::Object(r.clone()),
    }
}
