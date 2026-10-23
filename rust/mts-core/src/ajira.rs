//! Ajira — job openings and applications.
//!
//! The admin advertises a vacancy for a named department or service. An
//! applicant fills in their details — photo, profession, level and the rest —
//! and is then assessed twice: a theoretical paper and a practical task. The
//! two scores are weighted into one total, so a hiring decision rests on a
//! recorded number rather than an impression. Hiring an applicant creates the
//! employee record in one step.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS job_openings (
           id            VARCHAR(64)   NOT NULL,
           title         VARCHAR(200)  NOT NULL,
           department    VARCHAR(40)   NOT NULL DEFAULT 'employee',
           service       VARCHAR(160)  NOT NULL DEFAULT '',
           level         VARCHAR(30)   NOT NULL DEFAULT 'mid',
           location      VARCHAR(120)  NOT NULL DEFAULT 'Tanzania',
           kind          VARCHAR(40)   NOT NULL DEFAULT 'Full-time',
           salary        VARCHAR(120)  NOT NULL DEFAULT '',
           description   TEXT          NOT NULL DEFAULT '',
           requirements  TEXT          NOT NULL DEFAULT '[]',
           theoretical_weight INTEGER NOT NULL DEFAULT 50,
           practical_weight   INTEGER NOT NULL DEFAULT 50,
           positions     INTEGER       NOT NULL DEFAULT 1,
           is_open       INTEGER       NOT NULL DEFAULT 1,
           created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS job_applications (
           id                 VARCHAR(64)   NOT NULL,
           job_id             VARCHAR(64)   NOT NULL DEFAULT '',
           job_title          VARCHAR(200)  NOT NULL DEFAULT '',
           full_name          VARCHAR(160)  NOT NULL,
           email              VARCHAR(190)  NOT NULL DEFAULT '',
           phone              VARCHAR(60)   NOT NULL DEFAULT '',
           photo              TEXT          NOT NULL DEFAULT '',
           profession         VARCHAR(160)  NOT NULL DEFAULT '',
           level              VARCHAR(30)   NOT NULL DEFAULT 'mid',
           region             VARCHAR(80)   NOT NULL DEFAULT '',
           experience_years   INTEGER       NOT NULL DEFAULT 0,
           skills             TEXT          NOT NULL DEFAULT '[]',
           cv_url             VARCHAR(500)  NOT NULL DEFAULT '',
           cover_letter       TEXT          NOT NULL DEFAULT '',
           theoretical_score  INTEGER       NOT NULL DEFAULT 0,
           practical_score    INTEGER       NOT NULL DEFAULT 0,
           total_score        INTEGER       NOT NULL DEFAULT 0,
           assessment_notes   TEXT          NOT NULL DEFAULT '',
           status             VARCHAR(30)   NOT NULL DEFAULT 'Received',
           created_at         DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at         DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_app_job ON job_applications(job_id);",
    );
}

/// The statuses an application moves through, in order.
pub const STATUSES: &[&str] = &[
    "Received",
    "Screening",
    "Theory Done",
    "Practical Done",
    "Assessed",
    "Shortlisted",
    "Hired",
    "Rejected",
];

/// The weighted total, rounded to a whole percent.
pub fn weighted(theory: i64, practical: i64, tw: i64, pw: i64) -> i64 {
    let tw = tw.clamp(0, 100);
    let pw = pw.clamp(0, 100);
    let sum = tw + pw;
    if sum == 0 {
        return 0;
    }
    let t = theory.clamp(0, 100);
    let p = practical.clamp(0, 100);
    ((t * tw + p * pw) as f64 / sum as f64).round() as i64
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "status" => ok(json!({ "statuses": STATUSES })),
        "openings" => {
            let rows = db::rows(
                "SELECT * FROM job_openings WHERE is_open = 1 ORDER BY created_at DESC",
                &[],
            );
            ok(json!({ "openings": rows.iter().map(map::job).collect::<Vec<_>>() }))
        }
        "admin_openings" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM job_openings ORDER BY created_at DESC", &[]);
            ok(json!({ "openings": rows.iter().map(map::job).collect::<Vec<_>>() }))
        }
        "opening_save" => opening_save(c),
        "opening_delete" => opening_delete(c),
        "apply" => apply(c),
        "admin_applications" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let job_id = c.p("jobId");
            let rows = db::rows(
                "SELECT * FROM job_applications WHERE (?1 = '' OR job_id = ?1) ORDER BY total_score DESC, created_at DESC",
                &[arg(json!(job_id))],
            );
            ok(json!({ "applications": rows.iter().map(map::application).collect::<Vec<_>>() }))
        }
        "assess" => assess(c),
        "status_set" => status_set(c),
        "hire" => hire(c),
        "delete_application" => delete_application(c),
        _ => bad("Unknown ajira action."),
    }
}

fn opening_save(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let title = c.p("title");
    if title.trim().is_empty() {
        return bad("The vacancy needs a title.");
    }
    let reqs: Vec<String> = c
        .pv("requirements")
        .as_array()
        .map(|a| a.iter().map(|v| util::s(v)).collect())
        .unwrap_or_else(|| {
            c.p("requirements")
                .split('\n')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        });
    let reqs_json = serde_json::to_string(&reqs).unwrap_or_else(|_| "[]".into());
    let tw = util::i(&c.pv("theoreticalWeight"));
    let pw = util::i(&c.pv("practicalWeight"));
    let id = c.p("id");
    let now = util::now();
    if id.is_empty() || id == "new" {
        let new_id = util::new_id("JOB-");
        let _ = db::run(
            "INSERT INTO job_openings (id, title, department, service, level, location, kind, salary, description,
                requirements, theoretical_weight, practical_weight, positions, is_open, created_at, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(new_id)), arg(json!(title)), arg(json!(c.p("department"))), arg(json!(c.p("service"))),
                arg(json!(c.p("level"))), arg(json!(c.p("location"))), arg(json!(c.p("kind"))), arg(json!(c.p("salary"))),
                arg(json!(c.p("description"))), arg(json!(reqs_json)),
                arg(json!(if tw == 0 && pw == 0 { 50 } else { tw })),
                arg(json!(if tw == 0 && pw == 0 { 50 } else { pw })),
                arg(json!(util::i(&c.pv("positions")))), arg(json!(1)), arg(json!(now)), arg(json!(now)),
            ],
        );
        ok(json!({ "id": new_id, "saved": true }))
    } else {
        let _ = db::run(
            "UPDATE job_openings SET title=?, department=?, service=?, level=?, location=?, kind=?, salary=?,
                description=?, requirements=?, theoretical_weight=?, practical_weight=?, positions=?, is_open=?, updated_at=?
             WHERE id=?",
            &[
                arg(json!(title)), arg(json!(c.p("department"))), arg(json!(c.p("service"))), arg(json!(c.p("level"))),
                arg(json!(c.p("location"))), arg(json!(c.p("kind"))), arg(json!(c.p("salary"))), arg(json!(c.p("description"))),
                arg(json!(reqs_json)),
                arg(json!(if tw == 0 && pw == 0 { 50 } else { tw })),
                arg(json!(if tw == 0 && pw == 0 { 50 } else { pw })),
                arg(json!(util::i(&c.pv("positions")))),
                arg(json!(if c.p("isOpen").is_empty() { 1 } else { util::i(&c.pv("isOpen")) })),
                arg(json!(now)), arg(json!(id)),
            ],
        );
        ok(json!({ "id": id, "saved": true }))
    }
}

fn opening_delete(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let _ = db::run("DELETE FROM job_openings WHERE id = ?", &[arg(json!(c.p("id")))]);
    ok(json!({ "deleted": true }))
}

/// An applicant fills in their details. Open to anyone — this is the front door.
fn apply(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let name = c.p("fullName");
    if name.trim().is_empty() {
        return bad("Your name is required.");
    }
    let photo = c.p("photo");
    if !photo.is_empty() && !photo.starts_with("data:image/") {
        return bad("The photo must be an uploaded image (data:image/…).");
    }
    let job_id = c.p("jobId");
    let job = if job_id.is_empty() {
        None
    } else {
        db::row("SELECT * FROM job_openings WHERE id = ?", &[arg(json!(job_id))])
    };
    let job_title = job
        .as_ref()
        .map(|j| util::s(j.get("title").unwrap_or(&J::Null)))
        .unwrap_or_default();
    let skills: Vec<String> = c
        .pv("skills")
        .as_array()
        .map(|a| a.iter().map(|v| util::s(v)).collect())
        .unwrap_or_else(|| {
            c.p("skills")
                .split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        });
    let id = util::new_id("APP-");
    let now = util::now();
    let _ = db::run(
        "INSERT INTO job_applications (id, job_id, job_title, full_name, email, phone, photo, profession, level,
            region, experience_years, skills, cv_url, cover_letter, status, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(job_id)), arg(json!(job_title)), arg(json!(name)),
            arg(json!(c.p("email"))), arg(json!(c.p("phone"))), arg(json!(photo)), arg(json!(c.p("profession"))),
            arg(json!(c.p("level"))), arg(json!(c.p("region"))), arg(json!(util::i(&c.pv("experienceYears")))),
            arg(json!(serde_json::to_string(&skills).unwrap_or_else(|_| "[]".into()))),
            arg(json!(c.p("cvUrl"))), arg(json!(c.p("coverLetter"))), arg(json!("Received")), arg(json!(now)), arg(json!(now)),
        ],
    );
    ok(json!({
        "id": id,
        "applicationId": id,
        "received": true,
        "jobTitle": job_title,
        "message": "Maombi yako yamepokelewa. Utapigiwa tathmini ya nadharia na ya vitendo.",
    }))
}

/// The admin records both scores; the total is computed here, not by the caller,
/// so it can never disagree with its parts.
fn assess(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let Some(app) = db::row("SELECT * FROM job_applications WHERE id = ?", &[arg(json!(id))]) else {
        return err("Application not found.", 404);
    };
    let theory = util::i(&c.pv("theoreticalScore")).clamp(0, 100);
    let practical = util::i(&c.pv("practicalScore")).clamp(0, 100);
    let (tw, pw) = {
        let job_id = util::s(app.get("job_id").unwrap_or(&J::Null));
        match db::row("SELECT theoretical_weight, practical_weight FROM job_openings WHERE id = ?", &[arg(json!(job_id))]) {
            Some(j) => (
                util::i(j.get("theoretical_weight").unwrap_or(&J::Null)),
                util::i(j.get("practical_weight").unwrap_or(&J::Null)),
            ),
            None => (50, 50),
        }
    };
    let total = weighted(theory, practical, tw, pw);
    let _ = db::run(
        "UPDATE job_applications SET theoretical_score=?, practical_score=?, total_score=?, assessment_notes=?, status=?, updated_at=? WHERE id=?",
        &[
            arg(json!(theory)), arg(json!(practical)), arg(json!(total)),
            arg(json!(c.p("assessmentNotes"))), arg(json!("Assessed")), arg(json!(util::now())), arg(json!(id)),
        ],
    );
    ok(json!({
        "assessed": true, "theoreticalScore": theory, "practicalScore": practical,
        "totalScore": total, "weights": { "theoretical": tw, "practical": pw },
    }))
}

fn status_set(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let status = c.p("status");
    if !STATUSES.contains(&status.as_str()) {
        return bad("Unknown application status.");
    }
    let _ = db::run(
        "UPDATE job_applications SET status=?, updated_at=? WHERE id=?",
        &[arg(json!(status)), arg(json!(util::now())), arg(json!(id))],
    );
    ok(json!({ "status": status }))
}

/// Hiring an applicant creates the employee record, so the two never drift.
fn hire(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let Some(app) = db::row("SELECT * FROM job_applications WHERE id = ?", &[arg(json!(id))]) else {
        return err("Application not found.", 404);
    };
    // The role comes from the vacancy, so the new employee serves the right
    // service from their first day.
    let job_id = util::s(app.get("job_id").unwrap_or(&J::Null));
    let job = db::row("SELECT * FROM job_openings WHERE id = ?", &[arg(json!(job_id))]);
    let department = job.as_ref().map(|j| util::s(j.get("department").unwrap_or(&J::Null))).unwrap_or_else(|| "employee".into());
    let service = job.as_ref().map(|j| util::s(j.get("service").unwrap_or(&J::Null))).unwrap_or_default();
    let emp_id = util::new_id("EMP-");
    let now = util::now();
    let services_json = if service.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string(&vec![service.clone()]).unwrap_or_else(|_| "[]".into())
    };
    let _ = db::run(
        "INSERT INTO employees (id, full_name, photo, profession, level, department, email, phone, region, country,
            experience_years, skills, services, tools, bio, status, hired_at, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(emp_id)), arg(json!(util::s(app.get("full_name").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("photo").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("profession").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("level").unwrap_or(&J::Null)))),
            arg(json!(department)), arg(json!(util::s(app.get("email").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("phone").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("region").unwrap_or(&J::Null)))), arg(json!("Tanzania")),
            arg(json!(util::i(app.get("experience_years").unwrap_or(&J::Null)))),
            arg(json!(util::s(app.get("skills").unwrap_or(&J::Null)))),
            arg(json!(services_json)), arg(json!("[]")), arg(json!("Hired through the Ajira pipeline.")),
            arg(json!("active")), arg(json!(now)), arg(json!(now)), arg(json!(now)),
        ],
    );
    let _ = db::run(
        "UPDATE job_applications SET status='Hired', updated_at=? WHERE id=?",
        &[arg(json!(now)), arg(json!(id))],
    );
    ok(json!({
        "hired": true,
        "employeeId": emp_id,
        "employee": util::s(app.get("full_name").unwrap_or(&J::Null)),
        "department": department,
        "service": service,
    }))
}

fn delete_application(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let _ = db::run("DELETE FROM job_applications WHERE id = ?", &[arg(json!(c.p("id")))]);
    ok(json!({ "deleted": true }))
}
