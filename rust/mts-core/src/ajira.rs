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
    // A database created before AI ranking existed still has to gain the two
    // columns, so the migration runs on every start and is a no-op once done.
    let _ = db::ensure_column("job_applications", "ai_score", "INTEGER NOT NULL DEFAULT 0");
    let _ = db::ensure_column("job_applications", "ai_note", "TEXT NOT NULL DEFAULT ''");
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
           ai_score           INTEGER       NOT NULL DEFAULT 0,
           ai_note            TEXT          NOT NULL DEFAULT '',
           assessment_notes   TEXT          NOT NULL DEFAULT '',
           status             VARCHAR(30)   NOT NULL DEFAULT 'Received',
           created_at         DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at         DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_app_job ON job_applications(job_id);
         CREATE TABLE IF NOT EXISTS employment_contracts (
           id            VARCHAR(64)   NOT NULL,
           employee_id   VARCHAR(64)   NOT NULL,
           employee_name VARCHAR(160)  NOT NULL DEFAULT '',
           role_title    VARCHAR(200)  NOT NULL DEFAULT '',
           department    VARCHAR(40)   NOT NULL DEFAULT '',
           service       VARCHAR(160)  NOT NULL DEFAULT '',
           salary        VARCHAR(120)  NOT NULL DEFAULT '',
           terms         TEXT          NOT NULL DEFAULT '',
           status        VARCHAR(30)   NOT NULL DEFAULT 'Draft',
           admin_sig     TEXT          NULL,
           employee_sig  TEXT          NULL,
           admin_signed_at DATETIME    NULL,
           employee_signed_at DATETIME NULL,
           created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_empcontract_emp ON employment_contracts(employee_id);",
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
        "ai_rank" => ai_rank(c).await,
        "question_bank" => question_bank(c),
        "status_set" => status_set(c),
        "hire" => hire(c),
        "employment_contracts" => employment_contracts(c),
        "employment_contract_sign" => employment_contract_sign(c),
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
    let _ = db::run(
        "DELETE FROM job_openings WHERE id = ?",
        &[arg(json!(c.p("id")))],
    );
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
        db::row(
            "SELECT * FROM job_openings WHERE id = ?",
            &[arg(json!(job_id))],
        )
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
    let Some(app) = db::row(
        "SELECT * FROM job_applications WHERE id = ?",
        &[arg(json!(id))],
    ) else {
        return err("Application not found.", 404);
    };
    let theory = util::i(&c.pv("theoreticalScore")).clamp(0, 100);
    let practical = util::i(&c.pv("practicalScore")).clamp(0, 100);
    let (tw, pw) = {
        let job_id = util::s(app.get("job_id").unwrap_or(&J::Null));
        match db::row(
            "SELECT theoretical_weight, practical_weight FROM job_openings WHERE id = ?",
            &[arg(json!(job_id))],
        ) {
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

/// Ask the model to rank the applicants for a vacancy, and write the score it
/// gives back onto each application. The model reads the same evidence the CEO
/// would — profession, level, experience, skills, the cover letter and the
/// assessment marks — and returns a score with a one-line reason. Nothing is
/// hired here: the ranking only orders the queue for the human decision.
async fn ai_rank(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    if !crate::llm::ready() {
        return bad("No AI provider is configured, so ranking cannot run. Add a key under AI Engine.");
    }
    let job_id = c.p("jobId");
    let job = if job_id.is_empty() {
        None
    } else {
        db::row("SELECT * FROM job_openings WHERE id = ?", &[arg(json!(job_id))])
    };
    let apps = db::rows(
        "SELECT * FROM job_applications WHERE (?1 = '' OR job_id = ?1) ORDER BY created_at DESC LIMIT 40",
        &[arg(json!(job_id))],
    );
    if apps.is_empty() {
        return bad("There are no applications to rank.");
    }
    let title = job
        .as_ref()
        .map(|j| util::s(j.get("title").unwrap_or(&J::Null)))
        .unwrap_or_else(|| "the role".into());
    let requirements = job
        .as_ref()
        .map(|j| util::s(j.get("requirements").unwrap_or(&J::Null)))
        .unwrap_or_default();
    let mut roster = String::new();
    for a in &apps {
        roster.push_str(&format!(
            "id={} | name={} | profession={} | level={} | years={} | skills={} | theory={} | practical={} | cover={}\n",
            util::s(a.get("id").unwrap_or(&J::Null)),
            util::s(a.get("full_name").unwrap_or(&J::Null)),
            util::s(a.get("profession").unwrap_or(&J::Null)),
            util::s(a.get("level").unwrap_or(&J::Null)),
            util::i(a.get("experience_years").unwrap_or(&J::Null)),
            util::s(a.get("skills").unwrap_or(&J::Null)),
            util::i(a.get("theoretical_score").unwrap_or(&J::Null)),
            util::i(a.get("practical_score").unwrap_or(&J::Null)),
            util::s(a.get("cover_letter").unwrap_or(&J::Null)).chars().take(240).collect::<String>(),
        ));
    }
    let prompt = format!(
        "You are the hiring panel for the role \"{title}\" at a Tanzanian technology company.\n\
         Requirements: {requirements}\n\n\
         Score each applicant from 0 to 100 for fit, weighing the assessment marks, the years of \
         experience, the level and how well the skills match the requirements. Reply with one line \
         per applicant and nothing else, in exactly this form:\n\
         <id>|<score>|<a short reason>\n\n\
         Applicants:\n{roster}"
    );
    let reply = match crate::llm::complete(&prompt).await {
        Ok(t) => t,
        Err(e) => return err(&format!("The model could not be reached: {e}"), 502),
    };
    // Parse the model's lines back onto the rows. Anything unparseable is left
    // alone rather than guessed at.
    let mut ranked: Vec<J> = Vec::new();
    for line in reply.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }
        let id = parts[0].trim();
        let Ok(score) = parts[1].trim().parse::<i64>() else {
            continue;
        };
        let score = score.clamp(0, 100);
        let reason = parts.get(2).map(|s| s.trim().to_string()).unwrap_or_default();
        let known = apps
            .iter()
            .any(|a| util::s(a.get("id").unwrap_or(&J::Null)) == id);
        if !known {
            continue;
        }
        let _ = db::run(
            "UPDATE job_applications SET ai_score=?, ai_note=?, updated_at=? WHERE id=?",
            &[arg(json!(score)), arg(json!(reason)), arg(json!(util::now())), arg(json!(id))],
        );
        ranked.push(json!({ "id": id, "aiScore": score, "aiNote": reason }));
    }
    ranked.sort_by(|a, b| {
        b["aiScore"].as_i64().unwrap_or(0).cmp(&a["aiScore"].as_i64().unwrap_or(0))
    });
    ok(json!({
        "ranked": ranked,
        "count": ranked.len(),
        "provider": crate::llm::provider_name(),
        "note": "The AI orders the queue; only the CEO hires.",
    }))
}

/// The theoretical questions for a role, and a practical brief. Returned for the
/// admin to put to the applicant; the marks are recorded through `assess`.
fn question_bank(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let job_id = c.p("jobId");
    let job = db::row("SELECT * FROM job_openings WHERE id = ?", &[arg(json!(job_id))]);
    let service = job
        .as_ref()
        .map(|j| util::s(j.get("service").unwrap_or(&J::Null)))
        .unwrap_or_default();
    let department = job
        .as_ref()
        .map(|j| util::s(j.get("department").unwrap_or(&J::Null)))
        .unwrap_or_default();
    let theory = questions_for(&service, &department);
    ok(json!({
        "theoretical": theory,
        "practical": practical_brief(&service, &department),
    }))
}

/// A short bank per family. This is a starting set the CEO can extend; the
/// point is that a vacancy is assessed the same way every time.
fn questions_for(service: &str, department: &str) -> Vec<J> {
    let key = format!("{service} {department}").to_lowercase();
    let common: Vec<&str> = vec![
        "Explain how you would break a large task into safe, testable steps.",
        "Describe a time you found and fixed a bug in code you did not write.",
        "How do you decide when work is finished enough to hand to a client?",
    ];
    let mut out: Vec<&str> = common;
    if key.contains("security") || key.contains("cyber") {
        out.extend_from_slice(&[
            "Walk through how you would test a web app for the OWASP top ten.",
            "What does a TLS handshake prove, and what does it not prove?",
        ]);
    } else if key.contains("ai") || key.contains("ml") {
        out.extend_from_slice(&[
            "How would you tell overfitting from underfitting on a fine-tune?",
            "Explain retrieval-augmented generation and when it beats a bigger model.",
        ]);
    } else if key.contains("software") || key.contains("developer") {
        out.extend_from_slice(&[
            "Compare a relational schema with a document store for an order system.",
            "What does idempotency mean for an HTTP API, and why does it matter?",
        ]);
    } else if key.contains("finance") {
        out.extend_from_slice(&[
            "Explain escrow to a client who has never used it.",
            "How is VAT computed on a mixed invoice?",
        ]);
    } else if key.contains("design") {
        out.extend_from_slice(&[
            "How do you make a dense dashboard readable at a glance?",
            "What makes a colour palette accessible, not just attractive?",
        ]);
    }
    out.into_iter()
        .map(|q| json!({ "question": q }))
        .collect()
}

/// The practical half: a brief the applicant works, so they are judged on doing,
/// not only on talking.
fn practical_brief(service: &str, department: &str) -> J {
    let key = format!("{service} {department}").to_lowercase();
    let brief = if key.contains("security") || key.contains("cyber") {
        "Given a small web service, produce a written audit: the three most likely \
         vulnerabilities, the evidence for each, and a fix for each."
    } else if key.contains("ai") || key.contains("ml") {
        "Take a small labelled dataset and produce a working classifier: the code, the \
         accuracy you measured, and one paragraph on what you would improve."
    } else if key.contains("software") || key.contains("developer") {
        "Build a small CRUD API with tests, and a one-page note on the design choices \
         you made and why."
    } else if key.contains("design") {
        "Produce two screens for a mobile app — a list and a detail view — and explain \
         the layout and colour decisions."
    } else {
        "Complete a short task drawn from the day-to-day work of the role, and write \
         half a page on how you approached it."
    };
    json!({ "brief": brief, "timeLimitHours": 72 })
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
    let Some(app) = db::row(
        "SELECT * FROM job_applications WHERE id = ?",
        &[arg(json!(id))],
    ) else {
        return err("Application not found.", 404);
    };
    // The role comes from the vacancy, so the new employee serves the right
    // service from their first day.
    let job_id = util::s(app.get("job_id").unwrap_or(&J::Null));
    let job = db::row(
        "SELECT * FROM job_openings WHERE id = ?",
        &[arg(json!(job_id))],
    );
    let department = job
        .as_ref()
        .map(|j| util::s(j.get("department").unwrap_or(&J::Null)))
        .unwrap_or_else(|| "employee".into());
    let service = job
        .as_ref()
        .map(|j| util::s(j.get("service").unwrap_or(&J::Null)))
        .unwrap_or_default();
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
    // Give the new worker a portal account, so the tools and the jobs their
    // role allows are reachable the moment they are hired.
    let linked = crate::wafanyakazi::link_account(
        &emp_id,
        &util::s(app.get("email").unwrap_or(&J::Null)),
        &util::s(app.get("full_name").unwrap_or(&J::Null)),
        &util::s(app.get("phone").unwrap_or(&J::Null)),
    )
    .is_some();
    // Draft the employment contract from the vacancy, so hiring and the paper
    // that records it are one action. The CEO signs it from the HR screen; it is
    // a draft until then, not a claim that anyone has agreed to anything.
    let contract_id = draft_employment_contract(
        &emp_id,
        &util::s(app.get("full_name").unwrap_or(&J::Null)),
        job.as_ref(),
        &department,
        &service,
    );
    ok(json!({
        "hired": true,
        "employeeId": emp_id,
        "employee": util::s(app.get("full_name").unwrap_or(&J::Null)),
        "department": department,
        "service": service,
        "portalAccount": linked,
        "contractId": contract_id,
    }))
}

/// Write the draft employment contract for a new hire. The terms are assembled
/// from the vacancy so the document matches what was advertised.
fn draft_employment_contract(
    employee_id: &str,
    employee_name: &str,
    job: Option<&db::Row>,
    department: &str,
    service: &str,
) -> String {
    let id = util::new_id("EC-");
    let g = |k: &str| {
        job.and_then(|j| j.get(k))
            .map(util::s)
            .unwrap_or_default()
    };
    let title = g("title");
    let salary = g("salary");
    let location = {
        let l = g("location");
        if l.is_empty() { "Tanzania".to_string() } else { l }
    };
    let terms = format!(
        "EMPLOYMENT CONTRACT\n\
         Employee: {employee_name}\n\
         Role: {title}\n\
         Department: {department}\n\
         Service: {service}\n\
         Location: {location}\n\
         Remuneration: {salary}\n\n\
         1. The Employee shall carry out the duties of the Role diligently and in \
         accordance with the instructions of the CEO.\n\
         2. Work is performed inside the company platform. The Employee shall use \
         only the tools the CEO assigns to them, and shall not use tools outside \
         the platform for company work.\n\
         3. The Employee shall keep confidential all client data, source code and \
         business information, both during and after employment.\n\
         4. Remuneration is paid after the client accepts the delivered work and \
         the funds are released from escrow, less the statutory deductions (PAYE, \
         NSSF and WCF).\n\
         5. Either party may end this contract with the notice period required by \
         the Employment and Labour Relations Act.\n\
         6. This contract is governed by the laws of the United Republic of Tanzania."
    );
    let _ = db::run(
        "INSERT INTO employment_contracts (id, employee_id, employee_name, role_title, department, service, salary, terms, status, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(employee_id)), arg(json!(employee_name)),
            arg(json!(title)), arg(json!(department)), arg(json!(service)),
            arg(json!(salary)), arg(json!(terms)), arg(json!("Draft")), arg(json!(util::now())),
        ],
    );
    id
}

fn delete_application(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let _ = db::run(
        "DELETE FROM job_applications WHERE id = ?",
        &[arg(json!(c.p("id")))],
    );
    ok(json!({ "deleted": true }))
}

/// Every employment contract, or the one belonging to a named employee. The CEO
/// reads them here to sign; a worker sees only their own.
fn employment_contracts(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    let employee_id = if role == "admin" {
        c.p("employeeId")
    } else {
        let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
        crate::wafanyakazi::employee_for_account(uid).unwrap_or_default()
    };
    let rows = db::rows(
        "SELECT * FROM employment_contracts WHERE (?1 = '' OR employee_id = ?1) ORDER BY created_at DESC",
        &[arg(json!(employee_id))],
    );
    let items: Vec<J> = rows
        .iter()
        .map(|r| {
            json!({
                "id": util::s(r.get("id").unwrap_or(&J::Null)),
                "employeeId": util::s(r.get("employee_id").unwrap_or(&J::Null)),
                "employeeName": util::s(r.get("employee_name").unwrap_or(&J::Null)),
                "roleTitle": util::s(r.get("role_title").unwrap_or(&J::Null)),
                "department": util::s(r.get("department").unwrap_or(&J::Null)),
                "service": util::s(r.get("service").unwrap_or(&J::Null)),
                "salary": util::s(r.get("salary").unwrap_or(&J::Null)),
                "terms": util::s(r.get("terms").unwrap_or(&J::Null)),
                "status": util::s(r.get("status").unwrap_or(&J::Null)),
                "adminSignedAt": util::iso(r.get("admin_signed_at").and_then(|v| v.as_str())),
                "employeeSignedAt": util::iso(r.get("employee_signed_at").and_then(|v| v.as_str())),
                "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
            })
        })
        .collect();
    ok(json!({ "contracts": items }))
}

/// Sign an employment contract. The CEO's signature is a real ECDSA over the
/// contract's terms, so a later change to the salary or the role breaks it. The
/// employee signs their own copy; once both have, the contract is `Active`.
fn employment_contract_sign(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let id = c.p("id");
    let Some(row) = db::row(
        "SELECT * FROM employment_contracts WHERE id = ?",
        &[arg(json!(id))],
    ) else {
        return err("Contract not found.", 404);
    };
    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("");
    // The bytes the signature covers: every term that matters, plus the role of
    // the signer, so the CEO's signature and the worker's are distinct and each
    // one says who agreed to what.
    let message = format!(
        "MTS-EMPLOYMENT|id={}|employee={}|role={}|department={}|service={}|salary={}|terms={}|created={}|signer={}",
        util::s(row.get("id").unwrap_or(&J::Null)),
        util::s(row.get("employee_id").unwrap_or(&J::Null)),
        util::s(row.get("role_title").unwrap_or(&J::Null)),
        util::s(row.get("department").unwrap_or(&J::Null)),
        util::s(row.get("service").unwrap_or(&J::Null)),
        util::s(row.get("salary").unwrap_or(&J::Null)),
        util::s(row.get("terms").unwrap_or(&J::Null)),
        util::s(row.get("created_at").unwrap_or(&J::Null)),
        if role == "admin" { "admin" } else { "employee" },
    );
    let sig = match crate::security::sign_bytes(message.as_bytes()) {
        Ok(s) => s,
        Err(e) => return err(&format!("The signature could not be made: {e}"), 500),
    };
    let now = util::now();
    if role == "admin" {
        let _ = db::run(
            "UPDATE employment_contracts SET admin_sig=?, admin_signed_at=? WHERE id=?",
            &[arg(json!(sig)), arg(json!(now)), arg(json!(id))],
        );
    } else {
        // Only the employee named on the contract may sign it.
        let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let mine = crate::wafanyakazi::employee_for_account(uid).unwrap_or_default();
        let on_contract = util::s(row.get("employee_id").unwrap_or(&J::Null));
        if mine.is_empty() || mine != on_contract {
            return err("This contract is not yours to sign.", 403);
        }
        let _ = db::run(
            "UPDATE employment_contracts SET employee_sig=?, employee_signed_at=? WHERE id=?",
            &[arg(json!(sig)), arg(json!(now)), arg(json!(id))],
        );
    }
    // Once both signatures are present the contract is live.
    let both = db::row(
        "SELECT admin_sig, employee_sig FROM employment_contracts WHERE id = ?",
        &[arg(json!(id))],
    )
    .map(|r| {
        both_signed(
            r.get("admin_sig").and_then(|v| v.as_str()).unwrap_or(""),
            r.get("employee_sig").and_then(|v| v.as_str()).unwrap_or(""),
        )
    })
    .unwrap_or(false);
    if both {
        let _ = db::run(
            "UPDATE employment_contracts SET status='Active' WHERE id=?",
            &[arg(json!(id))],
        );
    }
    ok(json!({
        "signed": true,
        "by": role,
        "status": if both { "Active" } else { "Draft" },
        "signature": sig,
        "algorithm": "ECDSA-secp256k1",
    }))
}

/// A contract is live only when both the CEO and the worker have signed it.
/// A single signature is a draft, not an agreement.
fn both_signed(admin_sig: &str, employee_sig: &str) -> bool {
    !admin_sig.is_empty() && !employee_sig.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contract_is_active_only_when_both_have_signed() {
        assert!(!both_signed("", ""));
        assert!(!both_signed("sig", ""));
        assert!(!both_signed("", "sig"));
        assert!(both_signed("sig", "sig"));
    }
}
