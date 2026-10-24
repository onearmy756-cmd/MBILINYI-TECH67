//! The business endpoints: requests, quotes, contracts, invoices, tickets,
//! news, portfolio, testimonials, team, tracking, dashboard.
//!
//! Each keeps the exact `action` names and JSON shapes the frontend reads, so
//! `app.js` is unchanged.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

const STATUSES: [&str; 7] = [
    "Pending",
    "Under Review",
    "Quoted",
    "Approved",
    "In Progress",
    "Completed",
    "Rejected",
];

fn owned(c: &Ctx, table: &str, id: &str) -> Option<db::Row> {
    let Ok(u) = c.require_user() else { return None };
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let row = db::row(
        &format!("SELECT * FROM {table} WHERE id = ?"),
        &[arg(json!(id))],
    )?;
    if role == "admin" {
        return Some(row);
    }
    let owner = row.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    if owner == uid {
        Some(row)
    } else {
        None
    }
}

/* ---------------------------- requests ---------------------------- */

pub async fn requests(c: &Ctx) -> Response {
    match c.action().as_str() {
        "create" => req_create(c),
        "mine" => {
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let uid = u
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = u
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rows = db::rows(
                "SELECT * FROM requests WHERE user_id = ? OR lower(email) = ? ORDER BY created_at DESC",
                &[arg(json!(uid)), arg(json!(email))],
            );
            ok(json!({ "requests": rows.iter().map(map::request).collect::<Vec<_>>() }))
        }
        "get" => match owned(c, "requests", &c.p("id")) {
            Some(r) => ok(json!({ "request": map::request(&r) })),
            None => err("Request not found.", 404),
        },
        "admin_list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM requests ORDER BY created_at DESC", &[]);
            ok(json!({ "requests": rows.iter().map(map::request).collect::<Vec<_>>() }))
        }
        "set_status" => req_set_status(c),
        "note" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let _ = db::run(
                "UPDATE requests SET admin_note = ? WHERE id = ?",
                &[arg(json!(c.p("note"))), arg(json!(id))],
            );
            ok(json!({ "updated": true }))
        }
        "admin_update" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let _ = db::run(
                "UPDATE requests SET service=?, title=?, description=?, priority=?, budget=?, deadline=?, status=?, admin_note=? WHERE id=?",
                &[
                    arg(json!(c.p("service"))), arg(json!(c.p("title"))), arg(json!(c.p("description"))),
                    arg(json!(c.p("priority"))), arg(json!(c.p("budget"))), arg(json!(c.p("deadline"))),
                    arg(json!(c.p("status"))), arg(json!(c.p("adminNote"))), arg(json!(id)),
                ],
            );
            match db::row("SELECT * FROM requests WHERE id = ?", &[arg(json!(id))]) {
                Some(r) => ok(json!({ "request": map::request(&r) })),
                None => err("Request not found.", 404),
            }
        }
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM requests WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown requests action."),
    }
}

fn req_create(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let name = c.p("name");
    let phone = c.p("phone");
    let email = c.p("email").to_lowercase();
    let service = c.p("service");
    let title = c.p("title");
    let desc = c.p("description");

    if name.is_empty() || phone.is_empty() || email.is_empty() {
        return bad("Fill name, phone & email.");
    }
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return bad("Invalid email address.");
    }
    if service.is_empty() {
        return bad("Select a service. / Chagua huduma.");
    }
    // BRELA, TRA and government e-services exist only in Tanzania. The grid
    // hides them abroad, but a crafted request must be refused too.
    if settings::is_tz_only(&service) && crate::pages::country_from_headers(&c.headers) != "TZ" {
        return bad(
            "This service is available in Tanzania only. / Huduma hii inapatikana Tanzania tu.",
        );
    }
    if title.is_empty() || desc.chars().count() < 15 {
        return bad("Add project title + detailed requirements (15+ chars).");
    }

    // Where the request came from. The browser sends the country it resolved and
    // the client may name their region; both are recorded so growth is real.
    let country = {
        let given = c.p("country").trim().to_string();
        if given.is_empty() {
            crate::ofisi::country_name(&crate::pages::country_from_headers(&c.headers))
        } else {
            given
        }
    };
    let region = c.p("region").trim().to_string();

    // Submitting as a guest creates an account and signs the person in, so the
    // tracking page they land on is already theirs. An existing account with the
    // same email is reused rather than duplicated.
    let mut uid = c
        .user()
        .and_then(|u| u.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let mut auto_password = String::new();
    // Set when a guest submission signs a new (or existing) account in, so the
    // browser holds the session the request was filed under.
    let mut new_session: Option<String> = None;
    if uid.is_empty() {
        if let Some(existing) = db::row(
            "SELECT * FROM users WHERE lower(email) = ?",
            &[arg(json!(email))],
        ) {
            if existing.get("status").and_then(|v| v.as_str()) != Some("Active") {
                return bad("This account is suspended. Contact 0796 752 645.");
            }
            uid = existing
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        } else {
            let new_uid = util::new_id("U-");
            auto_password = format!("client{}", 1000 + (rand::random::<u32>() % 9000));
            let hash = bcrypt::hash(&auto_password, 10).unwrap_or_default();
            let _ = db::run(
                "INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
                 VALUES (?,?,?,?,?,?,?,?,?)",
                &[
                    arg(json!(new_uid)), arg(json!(name)), arg(json!(email)), arg(json!(phone)),
                    arg(json!(c.p("company"))), arg(json!(hash)), arg(json!("client")),
                    arg(json!("Active")), arg(json!(util::now())),
                ],
            );
            uid = new_uid;
        }
        // Sign the account in, so the client portal is reachable immediately.
        if let Some(u) = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(uid))]) {
            new_session = Some(crate::session::login_user(c, &u).id);
        }
    }

    let tech = match c.pv("tech") {
        J::Array(a) => J::Array(a),
        _ => json!([]),
    };
    // Each service asks its own questions, so empty answers are dropped rather
    // than stored as blank fields.
    let details = match c.pv("serviceDetails") {
        J::Array(a) => J::Array(
            a.into_iter()
                .filter_map(|v| match v {
                    J::String(s) if !s.trim().is_empty() => Some(J::String(s.trim().to_string())),
                    _ => None,
                })
                .collect(),
        ),
        _ => json!([]),
    };

    let id = util::new_id("R-");
    let tracking = unique_tracking();
    let _ = db::run(
        "INSERT INTO requests (id, tracking_id, user_id, name, email, phone, company, service, title,
                               description, tech, service_details, platform, budget, deadline, priority, status, admin_note, country, region, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(tracking)), arg(json!(uid)), arg(json!(name)),
            arg(json!(email)), arg(json!(phone)), arg(json!(c.p("company"))),
            arg(json!(service)), arg(json!(title)), arg(json!(desc)),
            arg(json!(tech.to_string())), arg(json!(details.to_string())),
            arg(json!(if c.p("platform").is_empty() { "Web Application".into() } else { c.p("platform") })),
            arg(json!(c.p("budget"))), arg(json!(c.p("deadline"))),
            arg(json!(if c.p("priority").is_empty() { "Normal".into() } else { c.p("priority") })),
            arg(json!("Pending")), arg(json!("")),
            arg(json!(country)), arg(json!(region)), arg(json!(util::now())),
        ],
    );

    // Record where the work came from, so the growth view counts real requests
    // rather than the seeded catalogue of places the platform can serve.
    crate::ofisi::note_request(&country, &region);

    // Route the request to the worker who serves this service, so the client is
    // served by a named person from the moment they submit. With no worker for
    // the service yet, the request simply waits in the admin inbox.
    let mut assigned_to = String::new();
    if let Some(emp) = crate::wafanyakazi::pick_for_service(&service) {
        let emp_id = util::s(emp.get("id").unwrap_or(&J::Null));
        let emp_name = util::s(emp.get("full_name").unwrap_or(&J::Null));
        let tools_raw = util::s(emp.get("tools").unwrap_or(&J::Null));
        let _ = db::run(
            "INSERT INTO request_assignment (id, request_id, employee_id, employee_name, service, tools, assigned_by, note, created_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("RA-"))), arg(json!(id)), arg(json!(emp_id)), arg(json!(emp_name)),
                arg(json!(service)), arg(json!(tools_raw)), arg(json!("auto")), arg(json!("")), arg(json!(util::now())),
            ],
        );
        assigned_to = emp_name;
    }

    // Put the work on the scheduler's queue *held*: the CEO can see exactly what
    // is waiting, and the loop will not touch it until the request is approved.
    // The gate is therefore visible in the queue rather than implied by a status.
    {
        let tools: Vec<String> = if assigned_to.is_empty() {
            Vec::new()
        } else {
            crate::wafanyakazi::pick_for_service(&service)
                .map(|e| crate::tools::granted(&util::s(e.get("tools").unwrap_or(&J::Null))))
                .unwrap_or_default()
        };
        let agent_id = crate::agents::pick_for_tools(&tools);
        let task = format!(
            "Serve client request {tracking}: {title}. Service: {service}. \
             Carry out the work end to end, run the tests the service needs, and write the report the client will read."
        );
        crate::scheduler::enqueue(
            &agent_id,
            &task,
            &id,
            &service,
            &tools,
            "client-request",
            true,
        );
    }

    let request = db::row("SELECT * FROM requests WHERE id = ?", &[arg(json!(id))])
        .map(|r| map::request(&r))
        .unwrap_or(J::Null);
    // Tell the client their request landed and the CEO that a decision is
    // waiting, by SMS and by mail. The admin's inbox is where the CEO approves
    // and signs, so this is what starts that.
    if let Some(r) = db::row("SELECT * FROM requests WHERE id = ?", &[arg(json!(id))]) {
        crate::notify::request_created(&r);
    }
    let user = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(uid))])
        .map(|r| map::user(&r))
        .unwrap_or(J::Null);
    let mut res = ok(json!({
        "trackingId": tracking,
        "request": request,
        "user": user,
        "autoPassword": auto_password,
        "assignedTo": assigned_to,
    }));
    if let Some(sid) = new_session {
        if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sid)) {
            res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
        }
    }
    res
}

fn unique_tracking() -> String {
    for _ in 0..40 {
        let n = (rand::random::<u32>() % 10000) as u32;
        let candidate = format!("MTS-2026-{n:04}");
        if db::row(
            "SELECT id FROM requests WHERE tracking_id = ?",
            &[arg(json!(candidate))],
        )
        .is_none()
        {
            return candidate;
        }
    }
    let n = rand::random::<u32>() % 1000000;
    format!("MTS-2026-{n:06}")
}

/// The 50/50 rule: work may not start until the deposit is paid. Enforced here
/// so every caller inherits it, exactly as the PHP backend did.
fn req_set_status(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let status = c.p("status");
    if !STATUSES.contains(&status.as_str()) {
        return bad("Unknown status.");
    }
    if matches!(status.as_str(), "In Progress" | "Completed") {
        let Some(req) = db::row(
            "SELECT tracking_id FROM requests WHERE id = ?",
            &[arg(json!(id))],
        ) else {
            return err("Request not found.", 404);
        };
        let trk = req
            .get("tracking_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if let Some(block) = deposit_block(trk) {
            return bad(&block);
        }
    }
    let _ = db::run(
        "UPDATE requests SET status = ? WHERE id = ?",
        &[arg(json!(status)), arg(json!(id))],
    );
    match db::row("SELECT * FROM requests WHERE id = ?", &[arg(json!(id))]) {
        Some(r) => {
            // Tell the client where their work has got to. The client asked to
            // watch the progress, so every move is announced.
            notify_status(&r, &status);
            // When the CEO approves a request, the work starts for real: the
            // assigned worker's agent picks the job up without another prompt.
            if status == "Approved" {
                start_approved_work(&r);
            }
            ok(json!({ "request": map::request(&r) }))
        }
        None => err("Request not found.", 404),
    }
}

/// Tell the client their request moved, in the language they use.
fn notify_status(req: &db::Row, status: &str) {
    let phone = util::s(req.get("phone").unwrap_or(&J::Null));
    if phone.trim().is_empty() {
        return;
    }
    let tracking = util::s(req.get("tracking_id").unwrap_or(&J::Null));
    let service = util::s(req.get("service").unwrap_or(&J::Null));
    let message = match status {
        "Under Review" => format!("Ombi lako {tracking} ({service}) linakaguliwa. Tutakujulisha hatua inayofuata."),
        "Quoted" => format!("Nukuu ya ombi lako {tracking} imetumwa. Ingia kwenye portal kuisoma na kukubali."),
        "Approved" => format!("Ombi lako {tracking} limeidhinishwa. Kazi inaanza; utaona maendeleo kwenye portal."),
        "In Progress" => format!("Kazi ya ombi lako {tracking} inaendelea. Fuatilia maendeleo kwenye portal."),
        "Completed" => format!("Kazi ya ombi lako {tracking} imekamilika. Ingia portal kukagua na kukubali uwasilishaji."),
        "Rejected" => format!("Ombi lako {tracking} halikuweza kuendelea. Wasiliana nasi kwa 0796 752 645."),
        _ => format!("Hali ya ombi lako {tracking} sasa ni {status}."),
    };
    crate::notify::sms(
        &phone,
        &message,
        &tracking,
        &format!("status_{}", status.to_lowercase().replace(' ', "_")),
    );
}

/// The CEO approved a request, so the assigned worker's agent starts the job.
/// Nothing here runs if no agent or worker is in place — the request simply
/// waits, and the admin sees why.
///
/// The work goes on the scheduler's queue rather than straight to an agent, so
/// the approval gate is enforced in one place: a job put on the queue *held*
/// cannot run until this function releases it, and the CEO's approval is the
/// only thing that releases it.
fn start_approved_work(req: &db::Row) {
    let rid = util::s(req.get("id").unwrap_or(&J::Null));
    let service = util::s(req.get("service").unwrap_or(&J::Null));
    let title = util::s(req.get("title").unwrap_or(&J::Null));
    let tracking = util::s(req.get("tracking_id").unwrap_or(&J::Null));
    // Any job still held for this request may now go: the CEO has approved it.
    crate::scheduler::release_for_request(&rid);
    // If a job was already queued for this request, releasing it was enough.
    let already = db::scalar_i64(
        "SELECT COUNT(*) FROM agent_jobs WHERE request_id=? AND status IN ('ready','running','done')",
        &[arg(json!(rid))],
    );
    if already > 0 {
        return;
    }
    // Which worker serves this service, and what tools their job may use.
    let Some(emp) = crate::wafanyakazi::pick_for_service(&service) else {
        return;
    };
    let tools = crate::tools::granted(&util::s(emp.get("tools").unwrap_or(&J::Null)));
    let task = format!(
        "Serve client request {tracking}: {title}. Service: {service}. \
         Carry out the work end to end, run the tests the service needs, and write the report the client will read."
    );
    // The agent that holds the tools for this job; the manager picks when the
    // worker's grant names none.
    let agent_id = crate::agents::pick_for_tools(&tools);
    let job = crate::scheduler::enqueue(
        &agent_id,
        &task,
        &rid,
        &service,
        &tools,
        "approval-gate",
        true,
    );
    crate::scheduler::release(&job);
}

/* ---------------------------- quotes ---------------------------- */

pub async fn quotes(c: &Ctx) -> Response {
    match c.action().as_str() {
        "mine" => {
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let uid = u
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = u
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rows = db::rows(
                "SELECT q.* FROM quotes q LEFT JOIN requests r ON r.tracking_id = q.tracking_id
                 WHERE q.user_id = ? OR lower(r.email) = ? ORDER BY q.created_at DESC",
                &[arg(json!(uid)), arg(json!(email))],
            );
            ok(json!({ "quotes": rows.iter().map(map::quote).collect::<Vec<_>>() }))
        }
        "get" => match owned(c, "quotes", &c.p("id")) {
            Some(r) => ok(json!({ "quote": map::quote(&r) })),
            None => err("Quote not found.", 404),
        },
        "respond" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let id = c.p("id");
            let status = c.p("status");
            if !matches!(status.as_str(), "Approved" | "Rejected") {
                return bad("Respond with Approved or Rejected.");
            }
            match owned(c, "quotes", &id) {
                Some(_) => {
                    let _ = db::run(
                        "UPDATE quotes SET status = ? WHERE id = ?",
                        &[arg(json!(status)), arg(json!(id))],
                    );
                    match db::row("SELECT * FROM quotes WHERE id = ?", &[arg(json!(id))]) {
                        Some(r) => ok(json!({ "quote": map::quote(&r) })),
                        None => err("Quote not found.", 404),
                    }
                }
                None => err("Quote not found.", 404),
            }
        }
        "admin_list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM quotes ORDER BY created_at DESC", &[]);
            ok(json!({ "quotes": rows.iter().map(map::quote).collect::<Vec<_>>() }))
        }
        "admin_save" => quote_save(c),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run("DELETE FROM quotes WHERE id = ?", &[arg(json!(c.p("id")))]);
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown quotes action."),
    }
}

fn quote_save(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let request_id = c.p("requestId");
    let items = match c.pv("items") {
        J::Array(a) => a,
        _ => vec![],
    };
    let subtotal: f64 = items
        .iter()
        .map(|i| {
            let qty = i.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let price = i.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            qty * price
        })
        .sum();
    let vat = (subtotal * 0.18).round();
    let total = subtotal + vat;
    let trk = db::row(
        "SELECT tracking_id, user_id FROM requests WHERE id = ?",
        &[arg(json!(request_id))],
    )
    .map(|r| {
        (
            r.get("tracking_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            r.get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        )
    })
    .unwrap_or_default();
    let existing = c.p("id");
    let id = if existing.is_empty() || existing == "new" {
        let newid = util::new_id("Q-");
        let _ = db::run(
            "INSERT INTO quotes (id, request_id, tracking_id, user_id, items, subtotal, vat, total, currency, valid_until, status, admin_message, created_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(newid)), arg(json!(request_id)), arg(json!(trk.0)), arg(json!(trk.1)),
                arg(json!(serde_json::Value::Array(items.clone()).to_string())),
                arg(json!(subtotal)), arg(json!(vat)), arg(json!(total)),
                arg(json!(settings::get("pay_currency", "TZS"))), arg(json!(c.p("validUntil"))),
                arg(json!(if c.p("status").is_empty() { "Sent".into() } else { c.p("status") })),
                arg(json!(c.p("adminMessage"))), arg(json!(util::now())),
            ],
        );
        newid
    } else {
        let _ = db::run(
            "UPDATE quotes SET items=?, subtotal=?, vat=?, total=?, valid_until=?, status=?, admin_message=? WHERE id=?",
            &[
                arg(json!(serde_json::Value::Array(items).to_string())), arg(json!(subtotal)),
                arg(json!(vat)), arg(json!(total)), arg(json!(c.p("validUntil"))),
                arg(json!(c.p("status"))), arg(json!(c.p("adminMessage"))), arg(json!(existing)),
            ],
        );
        existing
    };
    match db::row("SELECT * FROM quotes WHERE id = ?", &[arg(json!(id))]) {
        Some(r) => ok(json!({ "quote": map::quote(&r) })),
        None => err("Quote not found.", 404),
    }
}

/* ---------------------------- contracts ---------------------------- */

pub async fn contracts(c: &Ctx) -> Response {
    match c.action().as_str() {
        "mine" => {
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let uid = u
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rows = db::rows(
                "SELECT c.* FROM contracts c JOIN requests r ON r.tracking_id = c.tracking_id
                 WHERE r.user_id = ? ORDER BY c.created_at DESC",
                &[arg(json!(uid))],
            );
            ok(json!({ "contracts": rows.iter().map(map::contract).collect::<Vec<_>>() }))
        }
        "admin_list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM contracts ORDER BY created_at DESC", &[]);
            ok(json!({ "contracts": rows.iter().map(map::contract).collect::<Vec<_>>() }))
        }
        "get" => match owned(c, "contracts", &c.p("id")) {
            Some(r) => ok(json!({ "contract": map::contract(&r) })),
            None => err("Contract not found.", 404),
        },
        "create" => contract_create(c),
        "update" => contract_update(c),
        "sign_client" => sign(c, "client"),
        "sign_admin" => sign(c, "admin"),
        "deposit_info" => deposit_info(c),
        "create_deposit_invoice" => create_deposit_invoice(c),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM contracts WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown contracts action."),
    }
}

fn contract_create(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let request_id = c.p("requestId");
    let Some(req) = db::row(
        "SELECT * FROM requests WHERE id = ?",
        &[arg(json!(request_id))],
    ) else {
        return err("Request not found.", 404);
    };
    let trk = req
        .get("tracking_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let quote = db::row(
        "SELECT * FROM quotes WHERE request_id = ? ORDER BY created_at DESC",
        &[arg(json!(request_id))],
    );
    let total = quote
        .as_ref()
        .and_then(|q| q.get("total").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let quote_id = quote
        .as_ref()
        .and_then(|q| q.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let id = util::new_id("C-");
    let title = req
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let _ = db::run(
        "INSERT INTO contracts (id, request_id, quote_id, tracking_id, client_name, project_title, scope, duration, total, status, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(request_id)), arg(json!(quote_id)), arg(json!(trk)),
            arg(json!(req.get("name").and_then(|v| v.as_str()).unwrap_or(""))),
            arg(json!(title)),
            arg(json!(req.get("description").and_then(|v| v.as_str()).unwrap_or(""))),
            arg(json!("45 days")), arg(json!(total)), arg(json!("Sent")), arg(json!(util::now())),
        ],
    );
    // The 50% deposit invoice is created with the contract, so the client can
    // pay without waiting for the admin to raise it separately.
    let created = create_deposit_invoice_row(&trk, total);
    ok(json!({
        "contract": db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]).as_ref().map(map::contract),
        "created": created,
    }))
}

fn create_deposit_invoice_row(tracking: &str, total: f64) -> bool {
    if total <= 0.0 || tracking.is_empty() {
        return false;
    }
    let already = db::scalar_i64(
        "SELECT count(*) FROM invoices WHERE tracking_id = ? AND lower(title) LIKE '%deposit%'",
        &[arg(json!(tracking))],
    );
    if already > 0 {
        return false;
    }
    let uid = db::row(
        "SELECT user_id FROM requests WHERE tracking_id = ?",
        &[arg(json!(tracking))],
    )
    .and_then(|r| {
        r.get("user_id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    })
    .unwrap_or_default();
    let _ = db::run(
        "INSERT INTO invoices (id, tracking_id, user_id, title, amount, status, method, created_at)
         VALUES (?,?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("INV-"))),
            arg(json!(tracking)),
            arg(json!(uid)),
            arg(json!("Project — 50% Deposit")),
            arg(json!((total / 2.0).round())),
            arg(json!("Unpaid")),
            arg(json!("M-Pesa / Bank")),
            arg(json!(util::now())),
        ],
    );
    true
}

fn create_deposit_invoice(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let Some(ct) = db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]) else {
        return err("Contract not found.", 404);
    };
    let trk = ct.get("tracking_id").and_then(|v| v.as_str()).unwrap_or("");
    let total = ct.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let created = create_deposit_invoice_row(trk, total);
    ok(json!({ "created": created }))
}

/// The deposit state for a contract, which the admin's countersign gate reads.
fn deposit_info(c: &Ctx) -> Response {
    let Ok(_u) = c.require_user() else {
        return util::unauth();
    };
    let id = c.p("id");
    let Some(ct) = db::row(
        "SELECT tracking_id FROM contracts WHERE id = ?",
        &[arg(json!(id))],
    ) else {
        return err("Contract not found.", 404);
    };
    let trk = ct.get("tracking_id").and_then(|v| v.as_str()).unwrap_or("");
    let inv = db::row(
        "SELECT * FROM invoices WHERE tracking_id = ? AND lower(title) LIKE '%deposit%' ORDER BY created_at DESC",
        &[arg(json!(trk))],
    );
    match inv {
        Some(i) => ok(json!({
            "deposit": {
                "paid": i.get("status").and_then(|v| v.as_str()) == Some("Paid"),
                "invoiceId": i.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                "status": i.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                "amount": i.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
            }
        })),
        None => ok(
            json!({ "deposit": { "paid": false, "invoiceId": "", "status": "none", "amount": 0 } }),
        ),
    }
}

/// Price is locked once the client has signed: the agreement is already sealed.
fn contract_update(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let Some(ct) = db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]) else {
        return err("Contract not found.", 404);
    };
    if ct.get("client_sig").map(|v| !v.is_null()).unwrap_or(false) {
        return bad(
            "Contract is locked: the client already signed. The agreed price can no longer change.",
        );
    }
    let _ = db::run(
        "UPDATE contracts SET project_title=?, scope=?, duration=?, total=? WHERE id=?",
        &[
            arg(json!(c.p("projectTitle"))),
            arg(json!(c.p("scope"))),
            arg(json!(c.p("duration"))),
            arg(json!(c.p("total").parse::<f64>().unwrap_or(0.0))),
            arg(json!(id)),
        ],
    );
    match db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]) {
        Some(r) => ok(json!({ "contract": map::contract(&r) })),
        None => err("Contract not found.", 404),
    }
}

/// Signature order is fixed: the client signs first, then the admin may
/// countersign, and only once the deposit is paid.
fn sign(c: &Ctx, who: &str) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let id = c.p("id");
    let sig = c.p("sig");
    if sig.len() < 20 {
        return bad("Draw your signature first.");
    }
    let Some(ct) = db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]) else {
        return err("Contract not found.", 404);
    };
    if who == "client" {
        let Ok(u) = c.require_user() else {
            return util::unauth();
        };
        let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let owns = db::scalar_i64(
            "SELECT count(*) FROM contracts c JOIN requests r ON r.tracking_id = c.tracking_id
             WHERE c.id = ? AND r.user_id = ?",
            &[arg(json!(id)), arg(json!(uid))],
        );
        if owns == 0 {
            return err("Not your contract.", 403);
        }
        if ct.get("client_sig").map(|v| !v.is_null()).unwrap_or(false) {
            return bad("This contract has already been signed by you.");
        }
        let _ = db::run(
            "UPDATE contracts SET client_sig=?, client_signed_at=?, status=? WHERE id=?",
            &[
                arg(json!(sig)),
                arg(json!(util::now())),
                arg(json!("Client Signed")),
                arg(json!(id)),
            ],
        );
    } else {
        if let Err(e) = c.require_admin() {
            return e;
        }
        if ct.get("client_sig").map(|v| v.is_null()).unwrap_or(true) {
            return bad(
                "The client has not signed yet — the client signs first, then you countersign.",
            );
        }
        if ct.get("admin_sig").map(|v| !v.is_null()).unwrap_or(false) {
            return bad("This contract has already been countersigned.");
        }
        // The deposit gate.
        let trk = ct.get("tracking_id").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(block) = deposit_block(trk) {
            return bad(&block);
        }
        let _ = db::run(
            "UPDATE contracts SET admin_sig=?, admin_signed_at=?, status=? WHERE id=?",
            &[
                arg(json!(sig)),
                arg(json!(util::now())),
                arg(json!("Signed")),
                arg(json!(id)),
            ],
        );
    }
    match db::row("SELECT * FROM contracts WHERE id = ?", &[arg(json!(id))]) {
        Some(r) => ok(json!({ "contract": map::contract(&r) })),
        None => err("Contract not found.", 404),
    }
}

/* ---------------------------- invoices ---------------------------- */

pub async fn invoices(c: &Ctx) -> Response {
    match c.action().as_str() {
        "mine" => {
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let uid = u
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = u
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rows = db::rows(
                "SELECT i.* FROM invoices i LEFT JOIN requests r ON r.tracking_id = i.tracking_id
                 WHERE i.user_id = ? OR lower(r.email) = ? ORDER BY i.created_at DESC",
                &[arg(json!(uid)), arg(json!(email))],
            );
            ok(json!({ "invoices": rows.iter().map(map::invoice).collect::<Vec<_>>() }))
        }
        "pay" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let id = c.p("id");
            let reference = c.p("ref");
            if reference.len() < 4 {
                return bad("Enter transaction reference code.");
            }
            let Some(row) = db::row("SELECT * FROM invoices WHERE id = ?", &[arg(json!(id))])
            else {
                return err("Invoice not found.", 404);
            };
            let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role != "admin" && !invoice_owned_by(&row, &u) {
                return err("Not your invoice.", 403);
            }
            if row.get("status").and_then(|v| v.as_str()) == Some("Paid") {
                return bad("Invoice already paid.");
            }
            let method = row.get("method").and_then(|v| v.as_str()).unwrap_or("");
            let _ = db::run(
                "UPDATE invoices SET status = ?, method = ? WHERE id = ?",
                &[
                    arg(json!("Paid")),
                    arg(json!(format!("{method} • Ref {reference}"))),
                    arg(json!(id)),
                ],
            );
            match db::row("SELECT * FROM invoices WHERE id = ?", &[arg(json!(id))]) {
                Some(r) => ok(json!({ "invoice": map::invoice(&r) })),
                None => err("Invoice not found.", 404),
            }
        }
        "admin_list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM invoices ORDER BY created_at DESC", &[]);
            ok(json!({ "invoices": rows.iter().map(map::invoice).collect::<Vec<_>>() }))
        }
        "admin_save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let new_id = if id.is_empty() || id == "new" {
                util::new_id("INV-")
            } else {
                id.clone()
            };
            // An invoice records the currency it was written in, so a later
            // change to the platform default never reinterprets an old figure.
            let cur = {
                let want = c.p("currency");
                if want.is_empty() {
                    settings::get("pay_currency", "TZS")
                } else {
                    want
                }
            };
            if id.is_empty() || id == "new" {
                let _ = db::run(
                    "INSERT INTO invoices (id, tracking_id, user_id, title, amount, currency, status, method, due_date, created_at)
                     VALUES (?,?,?,?,?,?,?,?,?,?)",
                    &[
                        arg(json!(new_id)), arg(json!(c.p("trackingId"))), arg(json!(c.p("userId"))),
                        arg(json!(c.p("title"))), arg(json!(c.p("amount").parse::<f64>().unwrap_or(0.0))),
                        arg(json!(cur)),
                        arg(json!(if c.p("status").is_empty() { "Unpaid".into() } else { c.p("status") })),
                        arg(json!(c.p("method"))), arg(json!(c.p("dueDate"))), arg(json!(util::now())),
                    ],
                );
            } else {
                let _ = db::run(
                    "UPDATE invoices SET title=?, amount=?, currency=?, status=?, method=?, due_date=?, tracking_id=? WHERE id=?",
                    &[
                        arg(json!(c.p("title"))), arg(json!(c.p("amount").parse::<f64>().unwrap_or(0.0))),
                        arg(json!(cur)), arg(json!(c.p("status"))), arg(json!(c.p("method"))),
                        arg(json!(c.p("dueDate"))), arg(json!(c.p("trackingId"))), arg(json!(new_id)),
                    ],
                );
            }
            let rows = db::rows("SELECT * FROM invoices ORDER BY created_at DESC", &[]);
            ok(json!({ "invoices": rows.iter().map(map::invoice).collect::<Vec<_>>() }))
        }
        "admin_delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM invoices WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        "mark_paid" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let _ = db::run(
                "UPDATE invoices SET status = ? WHERE id = ?",
                &[arg(json!("Paid")), arg(json!(id))],
            );
            match db::row("SELECT * FROM invoices WHERE id = ?", &[arg(json!(id))]) {
                Some(r) => ok(json!({ "invoice": map::invoice(&r) })),
                None => err("Invoice not found.", 404),
            }
        }
        _ => bad("Unknown invoices action."),
    }
}

/* ---------------------------- tickets ---------------------------- */

pub async fn tickets(c: &Ctx) -> Response {
    match c.action().as_str() {
        "create" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let name = c.p("name");
            let email = c.p("email").to_lowercase();
            let message = c.p("message");
            let mut subject = c.p("subject");
            if name.is_empty() || email.is_empty() || message.is_empty() {
                return bad("Please fill name, email and message. / Jaza sehemu zote muhimu.");
            }
            if !email.contains('@') {
                return bad("Invalid email address.");
            }
            if subject.is_empty() {
                subject = "Website inquiry".to_string();
            }
            // A signed-in caller owns the ticket; a guest is recorded as GUEST.
            let uid = c
                .user()
                .and_then(|u| u.get("id").and_then(|v| v.as_str()).map(str::to_string))
                .unwrap_or_else(|| "GUEST".to_string());
            let id = util::new_id("T-");
            let _ = db::run(
                "INSERT INTO tickets (id, user_id, name, email, subject, message, status, created_at)
                 VALUES (?,?,?,?,?,?,?,?)",
                &[
                    arg(json!(id)), arg(json!(uid)), arg(json!(name)), arg(json!(email)),
                    arg(json!(subject)), arg(json!(message)), arg(json!("Open")), arg(json!(util::now())),
                ],
            );
            match db::row("SELECT * FROM tickets WHERE id = ?", &[arg(json!(id))]) {
                Some(r) => ok(json!({ "ticket": map::ticket(&r) })),
                None => err("Ticket not found.", 404),
            }
        }
        "mine" => {
            let Ok(u) = c.require_user() else {
                return util::unauth();
            };
            let uid = u
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = u
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rows = db::rows(
                "SELECT * FROM tickets WHERE user_id = ? OR lower(email) = ? ORDER BY created_at DESC",
                &[arg(json!(uid)), arg(json!(email))],
            );
            ok(json!({ "tickets": rows.iter().map(map::ticket).collect::<Vec<_>>() }))
        }
        "all" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM tickets ORDER BY created_at DESC", &[]);
            ok(json!({ "tickets": rows.iter().map(map::ticket).collect::<Vec<_>>() }))
        }
        "reply" => ticket_reply(c, false),
        "admin_reply" => ticket_reply(c, true),
        "admin_delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let _ = db::run(
                "DELETE FROM ticket_replies WHERE ticket_id = ?",
                &[arg(json!(id))],
            );
            let _ = db::run("DELETE FROM tickets WHERE id = ?", &[arg(json!(id))]);
            ok(json!({ "deleted": true }))
        }
        "toggle" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let Some(row) = db::row("SELECT * FROM tickets WHERE id = ?", &[arg(json!(id))]) else {
                return err("Ticket not found.", 404);
            };
            let next = if row.get("status").and_then(|v| v.as_str()) == Some("Open") {
                "Closed"
            } else {
                "Open"
            };
            let _ = db::run(
                "UPDATE tickets SET status = ? WHERE id = ?",
                &[arg(json!(next)), arg(json!(id))],
            );
            ok(json!({ "status": next }))
        }
        _ => bad("Unknown tickets action."),
    }
}

/// The ticket's `text` field holds the reply; `by_name` records who wrote it.
fn ticket_reply(c: &Ctx, admin: bool) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let id = c.p("id");
    let text = c.p("text");
    if text.is_empty() {
        return bad("Write a reply first.");
    }
    let Some(t) = db::row("SELECT * FROM tickets WHERE id = ?", &[arg(json!(id))]) else {
        return err("Ticket not found.", 404);
    };
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("client");
    if admin {
        if role != "admin" {
            return util::forbidden("Admin access required.");
        }
    } else if role != "admin" && !ticket_owned_by(&t, &u) {
        return err("Not your ticket.", 403);
    }
    let by = if admin {
        "Jackson Mbilinyi (CEO)".to_string()
    } else {
        format!(
            "{} (Client)",
            u.get("name").and_then(|v| v.as_str()).unwrap_or("")
        )
    };
    let _ = db::run(
        "INSERT INTO ticket_replies (id, ticket_id, by_name, body, created_at) VALUES (?,?,?,?,?)",
        &[
            arg(json!(util::new_id("TR-"))),
            arg(json!(id)),
            arg(json!(by)),
            arg(json!(text)),
            arg(json!(util::now())),
        ],
    );
    match db::row("SELECT * FROM tickets WHERE id = ?", &[arg(json!(id))]) {
        Some(r) => ok(json!({ "ticket": map::ticket(&r) })),
        None => err("Ticket not found.", 404),
    }
}

/// A ticket belongs to its `user_id` or to the account with the same email.
fn ticket_owned_by(t: &db::Row, user: &db::Row) -> bool {
    let uid = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    t.get("user_id").and_then(|v| v.as_str()) == Some(uid)
        || t.get("email")
            .and_then(|v| v.as_str())
            .map(|e| e.to_lowercase())
            == Some(email)
}

/* ---------------------------- content: news, portfolio, testimonials, team ---------------------------- */

pub async fn news(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            // The public list hides unpublished posts; the admin console asks for
            // everything with `all=1` so it can manage drafts.
            let all = c.p("all") == "1" && c.require_admin().is_ok();
            let sql = if all {
                "SELECT * FROM news_posts ORDER BY created_at DESC"
            } else {
                "SELECT * FROM news_posts WHERE is_active = 1 ORDER BY created_at DESC"
            };
            let rows = db::rows(sql, &[]);
            ok(json!({ "news": rows.iter().map(map::news).collect::<Vec<_>>() }))
        }
        "get" => match db::row(
            "SELECT * FROM news_posts WHERE id = ?",
            &[arg(json!(c.p("id")))],
        ) {
            Some(r) => ok(json!({ "post": map::news(&r) })),
            None => err("Post not found.", 404),
        },
        "add" | "update" => content_save(
            c,
            "news_posts",
            &["title", "category", "excerpt", "body", "image_url"],
            "N-",
        ),
        "toggle" => toggle_field(c, "news_posts", "published"),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM news_posts WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown news action."),
    }
}

/// The editor sends `quote`; the column is `text`. Mapping it here keeps the
/// frontend's field name without renaming a column the database already had.
fn testimonials_save(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let name = c.p("name");
    let text = c.p("quote");
    if name.is_empty() || text.is_empty() {
        return bad("Name and quote are required.");
    }
    let stars = c.p("stars").parse::<i64>().unwrap_or(5).clamp(1, 5);
    let sort = c.p("sort_order").parse::<i64>().unwrap_or(0);
    if id.is_empty() || id == "new" {
        let new_id = util::new_id("TS-");
        let _ = db::run(
            "INSERT INTO testimonials (id, name, role, text, stars, sort_order, is_active, created_at, updated_at)
             VALUES (?,?,?,?,?,?,1,?,?)",
            &[
                arg(json!(new_id)), arg(json!(name)), arg(json!(c.p("role"))), arg(json!(text)),
                arg(json!(stars)), arg(json!(sort)), arg(json!(util::now())), arg(json!(util::now())),
            ],
        );
        ok(json!({ "id": new_id, "saved": true }))
    } else {
        let _ = db::run(
            "UPDATE testimonials SET name=?, role=?, text=?, stars=?, sort_order=?, updated_at=? WHERE id=?",
            &[
                arg(json!(name)), arg(json!(c.p("role"))), arg(json!(text)),
                arg(json!(stars)), arg(json!(sort)), arg(json!(util::now())), arg(json!(id)),
            ],
        );
        ok(json!({ "id": id, "saved": true }))
    }
}

/// The 50/50 rule as one message: `None` when work may start, otherwise what to
/// tell the admin. Mirrors `deposit_block_message()` in the PHP, including its
/// choice of the *earliest* deposit invoice — the one the contract created.
fn deposit_block(tracking: &str) -> Option<String> {
    let inv = db::row(
        "SELECT * FROM invoices WHERE tracking_id = ? AND title LIKE '%50%Deposit%' ORDER BY created_at ASC LIMIT 1",
        &[arg(json!(tracking))],
    );
    let Some(inv) = inv else {
        return Some(
            "Deposit invoice bado haipo. Tengeneza 50% deposit invoice, mteja alipe, ndipo mradi unaruhusiwa kuanza."
                .to_string(),
        );
    };
    if inv.get("status").and_then(|v| v.as_str()) == Some("Paid") {
        return None;
    }
    let amount = inv.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let id = inv.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Some(format!(
        "Mteja hajalipa 50% deposit ({} TZS — {}). Lipa 50% kwanza kabla project kuanza.",
        util::thousands(amount),
        id
    ))
}

/// An invoice belongs to its `user_id`, and also to whoever owns the request
/// with the same tracking id. A client who submitted work as a guest and then
/// registered owns it by email, so the tracking route has to be honoured too.
fn invoice_owned_by(invoice: &db::Row, user: &db::Row) -> bool {
    let uid = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    if invoice.get("user_id").and_then(|v| v.as_str()) == Some(uid) {
        return true;
    }
    let trk = invoice
        .get("tracking_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if trk.is_empty() {
        return false;
    }
    db::scalar_i64(
        "SELECT count(*) FROM requests WHERE tracking_id = ? AND (user_id = ? OR lower(email) = ?)",
        &[arg(json!(trk)), arg(json!(uid)), arg(json!(email))],
    ) > 0
}

fn content_save(c: &Ctx, table: &str, fields: &[&str], prefix: &str) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    if id.is_empty() || id == "new" {
        let new_id = util::new_id(prefix);
        let cols: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
        let placeholders = vec!["?"; cols.len() + 2].join(",");
        let sql = format!(
            "INSERT INTO {table} (id, {}, created_at) VALUES ({placeholders})",
            cols.join(", ")
        );
        let mut args: Vec<Value> = vec![arg(json!(new_id))];
        for f in fields {
            args.push(arg(json!(c.p(f))));
        }
        args.push(arg(json!(util::now())));
        let _ = db::run(&sql, &args);
        ok(json!({ "id": new_id, "saved": true }))
    } else {
        let sets: Vec<String> = fields.iter().map(|f| format!("{f} = ?")).collect();
        let sql = format!(
            "UPDATE {table} SET {}, updated_at = ? WHERE id = ?",
            sets.join(", ")
        );
        let mut args: Vec<Value> = fields.iter().map(|f| arg(json!(c.p(f)))).collect();
        args.push(arg(json!(util::now())));
        args.push(arg(json!(id)));
        let _ = db::run(&sql, &args);
        ok(json!({ "id": id, "saved": true }))
    }
}

fn toggle_field(c: &Ctx, table: &str, field: &str) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let cur = db::row(
        &format!("SELECT {field} FROM {table} WHERE id = ?"),
        &[arg(json!(id))],
    )
    .and_then(|r| r.get(field).and_then(|v| v.as_str()).map(str::to_string))
    .unwrap_or_default();
    let next = if cur == "1" || cur.eq_ignore_ascii_case("true") {
        "0"
    } else {
        "1"
    };
    let _ = db::run(
        &format!("UPDATE {table} SET {field} = ? WHERE id = ?"),
        &[arg(json!(next)), arg(json!(id))],
    );
    ok(json!({ "value": next }))
}

pub async fn portfolio(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            let rows = db::rows(
                "SELECT * FROM portfolio_projects ORDER BY created_at DESC",
                &[],
            );
            ok(json!({ "projects": rows.iter().map(map::portfolio).collect::<Vec<_>>() }))
        }
        "add" | "update" => content_save(
            c,
            "portfolio_projects",
            &[
                "title",
                "category",
                "image_url",
                "description",
                "stat_label",
                "link_url",
                "sort_order",
            ],
            "P-",
        ),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM portfolio_projects WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown portfolio action."),
    }
}

pub async fn testimonials(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            let rows = db::rows("SELECT * FROM testimonials ORDER BY created_at DESC", &[]);
            ok(json!({ "testimonials": rows.iter().map(map::testimonial).collect::<Vec<_>>() }))
        }
        "add" | "update" => testimonials_save(c),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM testimonials WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown testimonials action."),
    }
}

pub async fn team(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            let rows = db::rows("SELECT * FROM team_members ORDER BY created_at ASC", &[]);
            ok(json!({ "team": rows.iter().map(map::team).collect::<Vec<_>>() }))
        }
        "add" | "update" => content_save(
            c,
            "team_members",
            &[
                "name",
                "role",
                "role_sw",
                "bio",
                "bio_sw",
                "image_url",
                "email",
                "phone",
                "linkedin",
                "sort_order",
            ],
            "TM-",
        ),
        "toggle" => toggle_field(c, "team_members", "is_active"),
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "DELETE FROM team_members WHERE id = ?",
                &[arg(json!(c.p("id")))],
            );
            ok(json!({ "deleted": true }))
        }
        _ => bad("Unknown team action."),
    }
}

/* ---------------------------- tracking + dashboard ---------------------------- */

pub async fn track(c: &Ctx) -> Response {
    let id = c.p("id");
    if id.is_empty() {
        return bad("Enter a tracking ID.");
    }
    let Some(req) = db::row(
        "SELECT * FROM requests WHERE tracking_id = ? OR id = ?",
        &[arg(json!(id)), arg(json!(id))],
    ) else {
        return err("No request found with that tracking ID.", 404);
    };
    let trk = req
        .get("tracking_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let quote = db::row(
        "SELECT * FROM quotes WHERE tracking_id = ? ORDER BY created_at DESC",
        &[arg(json!(trk))],
    );
    let contract = db::row(
        "SELECT * FROM contracts WHERE tracking_id = ? ORDER BY created_at DESC",
        &[arg(json!(trk))],
    );
    let invoices: Vec<J> = db::rows(
        "SELECT * FROM invoices WHERE tracking_id = ? ORDER BY created_at DESC",
        &[arg(json!(trk))],
    )
    .iter()
    .map(map::invoice)
    .collect();
    ok(json!({
        "request": map::request(&req),
        "quote": quote.as_ref().map(map::quote),
        "contract": contract.as_ref().map(map::contract),
        "invoices": invoices,
    }))
}

pub async fn dashboard(c: &Ctx) -> Response {
    match c.action().as_str() {
        "stats" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let sum = |sql: &str| -> f64 {
                db::rows(sql, &[])
                    .first()
                    .and_then(|r| r.get("s"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };
            let paid =
                sum("SELECT COALESCE(SUM(amount),0) AS s FROM invoices WHERE status = 'Paid'");
            let unpaid =
                sum("SELECT COALESCE(SUM(amount),0) AS s FROM invoices WHERE status <> 'Paid'");
            let pipeline =
                sum("SELECT COALESCE(SUM(total),0) AS s FROM quotes WHERE status <> 'Rejected'");
            let pending = db::scalar_i64(
                "SELECT count(*) FROM requests WHERE status IN ('Pending','Under Review')",
                &[],
            );
            let by_service = {
                let mut m = serde_json::Map::new();
                for r in db::rows(
                    "SELECT service, count(*) AS c FROM requests GROUP BY service",
                    &[],
                ) {
                    m.insert(
                        r.get("service")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        json!(r.get("c").and_then(|v| v.as_i64()).unwrap_or(0)),
                    );
                }
                J::Object(m)
            };
            let recent: Vec<J> = db::rows(
                "SELECT * FROM requests ORDER BY created_at DESC LIMIT 3",
                &[],
            )
            .iter()
            .map(map::request)
            .collect();
            ok(json!({
                "revenue": paid,
                "pipeline": pipeline,
                "paid": paid,
                "unpaid": unpaid,
                "pending": pending,
                "requestCount": db::scalar_i64("SELECT count(*) FROM requests", &[]),
                "userCount": db::scalar_i64("SELECT count(*) FROM users", &[]),
                "byService": by_service,
                "recent": recent,
                "escrowHeld": crate::payments::escrow::total_held(),
            }))
        }
        "visitor_stats" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let total = db::scalar_i64("SELECT count(*) FROM visitor_logs", &[]);
            // A visit is a distinct session per day, so one person reloading a
            // page ten times still counts once.
            let today = db::scalar_i64(
                "SELECT count(DISTINCT session_id) FROM visitor_logs WHERE substr(visit_date,1,10) = ?",
                &[arg(json!(chrono::Local::now()
                    .format("%Y-%m-%d")
                    .to_string()))],
            );
            let yesterday = db::scalar_i64(
                "SELECT count(DISTINCT session_id) FROM visitor_logs WHERE substr(visit_date,1,10) = ?",
                &[arg(json!((chrono::Local::now() - chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string()))],
            );
            // The last seven days include today, so the window starts six days back.
            let this_week = db::scalar_i64(
                "SELECT count(DISTINCT session_id) FROM visitor_logs WHERE substr(visit_date,1,10) >= ?",
                &[arg(json!((chrono::Local::now() - chrono::Duration::days(6))
                    .format("%Y-%m-%d")
                    .to_string()))],
            );
            let this_month = db::scalar_i64(
                "SELECT count(DISTINCT session_id) FROM visitor_logs WHERE substr(visit_date,1,7) = ?",
                &[arg(json!(chrono::Local::now().format("%Y-%m").to_string()))],
            );
            ok(json!({
                "total": total,
                "today": today,
                "yesterday": yesterday,
                "this_week": this_week,
                "this_month": this_month,
            }))
        }
        _ => bad("Unknown dashboard action."),
    }
}

/* ---------------------------- ollama (AI chat) ---------------------------- */

pub async fn ollama(c: &Ctx) -> Response {
    match c.action().as_str() {
        "config" => ok(json!({
            "configured": !std::env::var("OLLAMA_CLOUD_API_KEY").unwrap_or_default().is_empty(),
            "model": std::env::var("OLLAMA_MODEL").unwrap_or_default(),
        })),
        "chat" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let message = c.p("message");
            if message.is_empty() {
                return bad("Ask a question first.");
            }
            // The assistant answers from the live price list so it can never
            // quote a figure the admin has since changed.
            let prices: Vec<String> = settings::service_price_list()
                .iter()
                .map(|p| {
                    format!(
                        "{}: {}",
                        p.get("en").and_then(|v| v.as_str()).unwrap_or(""),
                        util::tzs(p.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0))
                    )
                })
                .collect();
            ok(json!({
                "reply": format!(
                    "Nashukuru kwa swali lako. Hapa kuna bei zetu za sasa:\n{}",
                    prices.join("\n")
                ),
                "source": "price_list",
            }))
        }
        _ => bad("Unknown ollama action."),
    }
}
