//! Wafanyakazi (employees), departments and client-request routing.
//!
//! The admin hires a worker for a named service, records their details — photo,
//! profession, level and the rest — and grants the tools that job needs. When a
//! client submits a request, it is routed to the worker who serves that service,
//! so the client is served by a named person rather than by a queue.
//!
//! Every department (CEO, employee, finance, marketing, HR) is a row here, so
//! the same admin screen governs all of them.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::tools;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// The management departments the CEO oversees. Order is the display order.
pub const DEPARTMENTS: &[(&str, &str, &str)] = &[
    ("ceo", "CEO Management", "fa-solid fa-crown"),
    ("employee", "Employee Management", "fa-solid fa-people-roof"),
    ("finance", "Finance Management", "fa-solid fa-coins"),
    ("marketing", "Marketing", "fa-solid fa-bullhorn"),
    ("hr", "HR Management", "fa-solid fa-user-tie"),
];

pub const LEVELS: &[&str] = &["junior", "mid", "senior", "lead", "principal"];

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS employees (
           id            VARCHAR(64)   NOT NULL,
           user_id       VARCHAR(64)   NOT NULL DEFAULT '',
           full_name     VARCHAR(160)  NOT NULL,
           photo         TEXT          NOT NULL DEFAULT '',
           profession    VARCHAR(160)  NOT NULL DEFAULT '',
           level         VARCHAR(30)   NOT NULL DEFAULT 'mid',
           department    VARCHAR(40)   NOT NULL DEFAULT 'employee',
           email         VARCHAR(190)  NOT NULL DEFAULT '',
           phone         VARCHAR(60)   NOT NULL DEFAULT '',
           region        VARCHAR(80)   NOT NULL DEFAULT '',
           country       VARCHAR(80)   NOT NULL DEFAULT 'Tanzania',
           languages     VARCHAR(160)  NOT NULL DEFAULT '',
           experience_years INTEGER    NOT NULL DEFAULT 0,
           skills        TEXT          NOT NULL DEFAULT '[]',
           services      TEXT          NOT NULL DEFAULT '[]',
           tools         TEXT          NOT NULL DEFAULT '[]',
           salary        DECIMAL(14,2) NOT NULL DEFAULT 0,
           currency      VARCHAR(10)   NOT NULL DEFAULT 'TZS',
           bio           TEXT          NOT NULL DEFAULT '',
           status        VARCHAR(30)   NOT NULL DEFAULT 'active',
           hired_at      DATETIME      NULL,
           created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS request_assignment (
           id           VARCHAR(64)  NOT NULL,
           request_id   VARCHAR(64)  NOT NULL,
           employee_id  VARCHAR(64)  NOT NULL,
           employee_name VARCHAR(160) NOT NULL DEFAULT '',
           service      VARCHAR(160) NOT NULL DEFAULT '',
           tools        TEXT         NOT NULL DEFAULT '[]',
           assigned_by  VARCHAR(120) NOT NULL DEFAULT 'admin',
           note         VARCHAR(255) NOT NULL DEFAULT '',
           created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_emp_service ON employees(status);
         CREATE INDEX IF NOT EXISTS idx_req_assign ON request_assignment(request_id);",
    );
}

/// Fold a service label to a stable key so the two label sets in the platform
/// (the customer-facing services grid and the priced catalogue) route to the
/// same worker. Lower case, letters and digits only: "AI & Fine-Tuning" and
/// "AI fine-tuning on your own data (RAG / LoRA)" both reduce to a shared stem.
fn service_key(s: &str) -> String {
    s.chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

/// Do two service labels mean the same work? An exact match, or one key
/// containing the other (so a longer priced-catalogue label matches the shorter
/// grid label), or a shared leading stem.
fn same_service(a: &str, b: &str) -> bool {
    let (ka, kb) = (service_key(a), service_key(b));
    if ka.is_empty() || kb.is_empty() {
        return false;
    }
    if ka == kb || ka.contains(&kb) || kb.contains(&ka) {
        return true;
    }
    let stem = |k: &str| k.chars().take(6).collect::<String>();
    stem(&ka) == stem(&kb)
}

/// The worker who serves a service: an active employee whose `services` list
/// matches it, preferring the highest level and the least busy.
pub fn pick_for_service(service: &str) -> Option<db::Row> {
    let all = db::rows(
        "SELECT * FROM employees WHERE status = 'active' ORDER BY created_at ASC",
        &[],
    );
    let rank = |lvl: &str| match lvl {
        "principal" => 5,
        "lead" => 4,
        "senior" => 3,
        "mid" => 2,
        "junior" => 1,
        _ => 0,
    };
    let mut best: Option<(i64, i64, db::Row)> = None;
    for r in all {
        let raw = r.get("services").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
        let serves: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
        let serves_this = serves.iter().any(|s| same_service(s, service))
            || serves.iter().any(|s| s.eq_ignore_ascii_case("all"));
        if !serves_this {
            continue;
        }
        let lvl = r.get("level").and_then(|v| v.as_str()).unwrap_or("mid").to_string();
        let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let busy = db::scalar_i64(
            "SELECT COUNT(*) FROM request_assignment WHERE employee_id = ?",
            &[arg(json!(id))],
        );
        let score = (rank(&lvl), -busy);
        match &best {
            Some((bl, bb, _)) if (*bl, *bb) >= score => {}
            _ => best = Some((score.0, score.1, r)),
        }
    }
    best.map(|(_, _, r)| r)
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "departments" => ok(json!({
            "departments": DEPARTMENTS.iter().map(|(k, n, i)| json!({"key": k, "name": n, "icon": i})).collect::<Vec<_>>(),
            "levels": LEVELS,
        })),
        "list" => {
            let department = c.p("department");
            let rows = db::rows(
                "SELECT * FROM employees WHERE (?1 = '' OR department = ?1) ORDER BY department, level DESC, created_at ASC",
                &[arg(json!(department))],
            );
            ok(json!({ "employees": rows.iter().map(map::employee).collect::<Vec<_>>() }))
        }
        "public" => {
            // The staff a visitor may see: name, photo, profession, level only.
            let rows = db::rows(
                "SELECT id, full_name, photo, profession, level, department, region, languages, experience_years, skills, bio
                 FROM employees WHERE status = 'active' ORDER BY department, level DESC",
                &[],
            );
            ok(json!({ "employees": rows.iter().map(map::employee_public).collect::<Vec<_>>() }))
        }
        "get" => match db::row("SELECT * FROM employees WHERE id = ?", &[arg(json!(c.p("id")))]) {
            Some(r) => ok(json!({ "employee": map::employee(&r) })),
            None => err("Employee not found.", 404),
        },
        "add" | "update" => save(c),
        "toggle" => toggle(c),
        "delete" => delete(c),
        "assign" => assign(c),
        "auto_assign" => auto_assign(c),
        "assignments" => {
            let Ok(_a) = c.require_admin() else {
                return util::forbidden("Admin access required.");
            };
            let rid = c.p("requestId");
            let rows = db::rows(
                "SELECT * FROM request_assignment WHERE (?1 = '' OR request_id = ?1) ORDER BY created_at DESC",
                &[arg(json!(rid))],
            );
            ok(json!({ "assignments": rows.iter().map(map::assignment).collect::<Vec<_>>() }))
        }
        // What the client sees: who is serving them and how far along the work is.
        "my_worker" => my_worker(c),
        _ => bad("Unknown wafanyakazi action."),
    }
}

fn save(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let name = c.p("fullName");
    if name.trim().is_empty() {
        return bad("The employee's name is required.");
    }
    // A photo, when given, must be an inline image — never a remote URL that
    // could leak the viewer's address to a third party.
    let photo = c.p("photo");
    if !photo.is_empty() && !photo.starts_with("data:image/") {
        return bad("The photo must be an uploaded image (data:image/…).");
    }
    // SVG is an image but can carry a script, so it is not accepted here.
    if photo.starts_with("data:image/svg") {
        return bad("Upload a PNG, JPG or WebP photo — SVG is not accepted.");
    }
    let tools_in: Vec<String> = c
        .pv("tools")
        .as_array()
        .map(|a| a.iter().map(|v| util::s(v)).collect())
        .unwrap_or_default();
    for t in &tools_in {
        if !tools::known(t) {
            return bad(&format!("Unknown tool: {t}. Only platform tools may be granted."));
        }
    }
    let list_json = |key: &str| -> String {
        let v = c.pv(key);
        let items: Vec<String> = v
            .as_array()
            .map(|a| a.iter().map(|x| util::s(x)).collect())
            .unwrap_or_else(|| {
                let s = util::s(&v);
                if s.trim().is_empty() {
                    Vec::new()
                } else {
                    s.split(',').map(|x| x.trim().to_string()).collect()
                }
            });
        serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
    };
    let id = c.p("id");
    let now = util::now();
    // An omitted field takes its sensible default, so a half-filled form still
    // creates a worker who is active and in a real department.
    let department = {
        let d = c.p("department");
        if d.trim().is_empty() { "employee".to_string() } else { d }
    };
    let level = {
        let l = c.p("level");
        if l.trim().is_empty() { "mid".to_string() } else { l }
    };
    let status = {
        let s = c.p("status");
        if s.trim().is_empty() { "active".to_string() } else { s }
    };
    let country = {
        let x = c.p("country");
        if x.trim().is_empty() { "Tanzania".to_string() } else { x }
    };
    let currency = {
        let x = c.p("currency");
        if x.trim().is_empty() { "TZS".to_string() } else { x }
    };
    if id.is_empty() || id == "new" {
        let new_id = util::new_id("EMP-");
        let _ = db::run(
            "INSERT INTO employees (id, full_name, photo, profession, level, department, email, phone, region, country,
                languages, experience_years, skills, services, tools, salary, currency, bio, status, hired_at, created_at, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(new_id)), arg(json!(name)), arg(json!(photo)), arg(json!(c.p("profession"))),
                arg(json!(level)), arg(json!(department)), arg(json!(c.p("email"))), arg(json!(c.p("phone"))),
                arg(json!(c.p("region"))), arg(json!(country)), arg(json!(c.p("languages"))),
                arg(json!(util::i(&c.pv("experienceYears")))), arg(json!(list_json("skills"))), arg(json!(list_json("services"))),
                arg(json!(serde_json::to_string(&tools_in).unwrap_or_else(|_| "[]".into()))),
                arg(json!(util::f(&c.pv("salary")))), arg(json!(currency)), arg(json!(c.p("bio"))),
                arg(json!(status)), arg(json!(now)), arg(json!(now)), arg(json!(now)),
            ],
        );
        ok(json!({ "id": new_id, "saved": true }))
    } else {
        let _ = db::run(
            "UPDATE employees SET full_name=?, photo=?, profession=?, level=?, department=?, email=?, phone=?, region=?,
                country=?, languages=?, experience_years=?, skills=?, services=?, tools=?, salary=?, currency=?, bio=?, status=?, updated_at=?
             WHERE id=?",
            &[
                arg(json!(name)), arg(json!(photo)), arg(json!(c.p("profession"))), arg(json!(level)),
                arg(json!(department)), arg(json!(c.p("email"))), arg(json!(c.p("phone"))), arg(json!(c.p("region"))),
                arg(json!(country)), arg(json!(c.p("languages"))), arg(json!(util::i(&c.pv("experienceYears")))),
                arg(json!(list_json("skills"))), arg(json!(list_json("services"))),
                arg(json!(serde_json::to_string(&tools_in).unwrap_or_else(|_| "[]".into()))),
                arg(json!(util::f(&c.pv("salary")))), arg(json!(currency)), arg(json!(c.p("bio"))),
                arg(json!(status)), arg(json!(now)), arg(json!(id)),
            ],
        );
        ok(json!({ "id": id, "saved": true }))
    }
}

fn toggle(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let cur = db::row("SELECT status FROM employees WHERE id = ?", &[arg(json!(id))])
        .and_then(|r| r.get("status").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let next = if cur == "active" { "inactive" } else { "active" };
    let _ = db::run(
        "UPDATE employees SET status = ?, updated_at = ? WHERE id = ?",
        &[arg(json!(next)), arg(json!(util::now())), arg(json!(id))],
    );
    ok(json!({ "status": next }))
}

fn delete(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let _ = db::run("DELETE FROM employees WHERE id = ?", &[arg(json!(c.p("id")))]);
    ok(json!({ "deleted": true }))
}

/// Route a request to a named worker. The admin decides; the client is told.
fn assign(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let request_id = c.p("requestId");
    let employee_id = c.p("employeeId");
    let Some(emp) = db::row("SELECT * FROM employees WHERE id = ? AND status = 'active'", &[arg(json!(employee_id))]) else {
        return bad("That employee is not available.");
    };
    let Some(req) = db::row("SELECT * FROM requests WHERE id = ? OR tracking_id = ?", &[arg(json!(request_id)), arg(json!(request_id))]) else {
        return bad("Request not found.");
    };
    let rid = util::s(req.get("id").unwrap_or(&J::Null));
    let service = util::s(req.get("service").unwrap_or(&J::Null));
    let emp_name = util::s(emp.get("full_name").unwrap_or(&J::Null));
    let tools_raw = util::s(emp.get("tools").unwrap_or(&J::Null));
    let _ = db::run(
        "INSERT INTO request_assignment (id, request_id, employee_id, employee_name, service, tools, assigned_by, note, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("RA-"))), arg(json!(rid)), arg(json!(employee_id)), arg(json!(emp_name)),
            arg(json!(service)), arg(json!(tools_raw)), arg(json!(c.p("by"))), arg(json!(c.p("note"))), arg(json!(util::now())),
        ],
    );
    // The request now shows a named owner, which is what the client watches.
    let _ = db::run(
        "UPDATE requests SET status = CASE WHEN status IN ('Pending','Under Review') THEN 'Approved' ELSE status END WHERE id = ?",
        &[arg(json!(rid))],
    );
    notify_sms(
        &c.p("phone"),
        &format!("Ombi lako {service} limepewa {emp_name}, atakuhudumia. Fuatilia maendeleo kwenye mfumo."),
        &rid,
        "assign",
    );
    ok(json!({ "assigned": true, "employee": emp_name, "employeeId": employee_id, "requestId": rid }))
}

/// Route a request to whichever worker serves its service.
fn auto_assign(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let rid_in = c.p("requestId");
    let Some(req) = db::row("SELECT * FROM requests WHERE id = ? OR tracking_id = ?", &[arg(json!(rid_in)), arg(json!(rid_in))]) else {
        return bad("Request not found.");
    };
    let service = util::s(req.get("service").unwrap_or(&J::Null));
    let Some(emp) = pick_for_service(&service) else {
        return ok(json!({
            "assigned": false,
            "reason": format!("No active employee serves {service}. Hire one, or assign by hand."),
        }));
    };
    let rid = util::s(req.get("id").unwrap_or(&J::Null));
    let emp_id = util::s(emp.get("id").unwrap_or(&J::Null));
    let emp_name = util::s(emp.get("full_name").unwrap_or(&J::Null));
    let tools_raw = util::s(emp.get("tools").unwrap_or(&J::Null));
    let _ = db::run(
        "INSERT INTO request_assignment (id, request_id, employee_id, employee_name, service, tools, assigned_by, note, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("RA-"))), arg(json!(rid)), arg(json!(emp_id)), arg(json!(emp_name)),
            arg(json!(service)), arg(json!(tools_raw)), arg(json!("auto")), arg(json!("")), arg(json!(util::now())),
        ],
    );
    let _ = db::run(
        "UPDATE requests SET status = CASE WHEN status IN ('Pending','Under Review') THEN 'Approved' ELSE status END WHERE id = ?",
        &[arg(json!(rid))],
    );
    notify_sms(
        &util::s(req.get("phone").unwrap_or(&J::Null)),
        &format!("Ombi lako {service} limepewa {emp_name}, atakuhudumia. Fuatilia maendeleo kwenye mfumo."),
        &rid,
        "auto_assign",
    );
    ok(json!({ "assigned": true, "employee": emp_name, "employeeId": emp_id, "requestId": rid }))
}

/// The client's view: the worker serving them, the tools that job uses, and the
/// progress the client can watch.
fn my_worker(c: &Ctx) -> Response {
    let rid = c.p("requestId");
    let rid = if rid.is_empty() { c.p("trackingId") } else { rid };
    let Some(req) = db::row("SELECT * FROM requests WHERE id = ? OR tracking_id = ?", &[arg(json!(rid)), arg(json!(rid))]) else {
        return err("Request not found.", 404);
    };
    let id = util::s(req.get("id").unwrap_or(&J::Null));
    let a = db::row(
        "SELECT * FROM request_assignment WHERE request_id = ? ORDER BY created_at DESC LIMIT 1",
        &[arg(json!(id))],
    );
    let (worker, tools_out) = match &a {
        Some(row) => {
            let emp_id = util::s(row.get("employee_id").unwrap_or(&J::Null));
            let emp = db::row(
                "SELECT id, full_name, photo, profession, level, department, region, languages FROM employees WHERE id = ?",
                &[arg(json!(emp_id))],
            );
            let codes = tools::granted(&util::s(row.get("tools").unwrap_or(&J::Null)));
            let t: Vec<J> = codes
                .iter()
                .filter_map(|x| tools::find(x))
                .map(|t| json!({"code": t.code, "name": t.name, "icon": t.icon, "family": t.family}))
                .collect();
            (emp.map(|e| map::employee_public(&e)), t)
        }
        None => (None, Vec::new()),
    };
    ok(json!({
        "request": map::request(&req),
        "worker": worker,
        "tools": tools_out,
        "status": util::s(req.get("status").unwrap_or(&J::Null)),
        "progress": progress_of(&util::s(req.get("status").unwrap_or(&J::Null))),
        "assignedAt": a.as_ref().map(|r| util::iso(r.get("created_at").and_then(|v| v.as_str()))),
    }))
}

/// How far along a request is, as a percentage the client can see.
pub fn progress_of(status: &str) -> i64 {
    match status {
        "Pending" => 10,
        "Under Review" => 25,
        "Quoted" => 40,
        "Approved" => 55,
        "In Progress" => 75,
        "Completed" => 100,
        "Rejected" => 0,
        _ => 5,
    }
}

/// Record an SMS so the admin can see every notification the platform sent.
/// Delivery itself is a gateway concern; the log is what the admin audits.
fn notify_sms(phone: &str, message: &str, request_id: &str, kind: &str) {
    if phone.trim().is_empty() {
        return;
    }
    let _ = db::run(
        "INSERT INTO sms_log (id, phone, message, request_id, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("SMS-"))), arg(json!(phone)), arg(json!(message)), arg(json!(request_id)),
            arg(json!(kind)), arg(json!("queued")), arg(json!(util::now())),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_label_sets_route_to_the_same_worker() {
        // The services grid and the priced catalogue name the same work
        // differently; both must match one grant.
        assert!(same_service("AI Fine-Tuning", "AI fine-tuning on your own data (RAG / LoRA)"));
        assert!(same_service("Cybersecurity Services", "Cyber-security audit (OWASP Top-10)"));
        assert!(same_service("Custom Software Development", "Custom software development"));
        assert!(same_service("AI Chatbots", "AI chatbot"));
        assert!(same_service("BRELA & Business Registration", "BRELA business / company registration"));
    }

    #[test]
    fn unrelated_services_do_not_match() {
        assert!(!same_service("Digital Forensics", "Mobile app (Android + iOS)"));
        assert!(!same_service("Email & Communication Setup", "Windows & OS Installation"));
        assert!(!same_service("", "AI Fine-Tuning"));
        assert!(!same_service("AI Fine-Tuning", ""));
    }
}

