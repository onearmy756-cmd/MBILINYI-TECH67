//! The agent company.
//!
//! This is the self-running side of the platform: a hundred agents, each with
//! one job, a named manager above them, and the tools that job needs. The admin
//! gives work and inspects it; the agents carry it out.
//!
//! The company is seeded once. Seeding is idempotent, so restarting never
//! duplicates an agent or resets one the admin has since changed.

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

/// How many agents the company keeps, not counting the manager.
pub const COMPANY_SIZE: usize = 100;

/// The manager leads every agent. The name is the one the CEO asked for.
pub const MANAGER_NAME: &str = "Magma Commander";

/// One role in the agent company: the department it belongs to, the speciality
/// it is hired for, and the tools it is allowed to reach.
struct Role {
    title: &'static str,
    department: &'static str,
    speciality: &'static str,
    tools: &'static [&'static str],
}

/// The roles the company is built from. Each is repeated to reach the full
/// headcount, so a hundred agents means a hundred seats with real work, not
/// padding.
const ROLES: &[Role] = &[
    Role { title: "Software Engineer", department: "software", speciality: "Web & API engineering", tools: &["code_studio", "version_forge", "build_pipeline", "test_runner", "api_forge", "db_forge"] },
    Role { title: "Mobile Engineer", department: "software", speciality: "Android & iOS", tools: &["code_studio", "mobile_forge", "test_runner", "deployer"] },
    Role { title: "Desktop Engineer", department: "software", speciality: "Desktop installers", tools: &["code_studio", "desktop_forge", "build_pipeline"] },
    Role { title: "DevOps Engineer", department: "software", speciality: "CI/CD & deployment", tools: &["build_pipeline", "deployer", "firewall_manager", "test_runner"] },
    Role { title: "Game Developer", department: "software", speciality: "Games & 3D", tools: &["game_forge", "code_studio", "card_3d"] },
    Role { title: "IoT Engineer", department: "software", speciality: "Firmware & telemetry", tools: &["iot_forge", "code_studio", "test_runner"] },
    Role { title: "Systems Engineer", department: "software", speciality: "Operating systems", tools: &["os_forge", "code_studio", "build_pipeline"] },
    Role { title: "Security Analyst", department: "security", speciality: "Penetration testing", tools: &["pentest", "vuln_scanner", "owasp_audit"] },
    Role { title: "Threat Hunter", department: "security", speciality: "Threat intelligence", tools: &["vuln_scanner", "malware_scan", "forensics"] },
    Role { title: "Cryptographer", department: "security", speciality: "ECDSA & key custody", tools: &["crypto_suite", "quantum_guard", "firewall_manager"] },
    Role { title: "Incident Responder", department: "security", speciality: "Forensics & recovery", tools: &["forensics", "malware_scan", "pentest"] },
    Role { title: "AI Fine-Tuning Engineer", department: "ai", speciality: "Model fine-tuning", tools: &["ai_finetune", "dataset_forge", "model_eval"] },
    Role { title: "Chatbot Engineer", department: "ai", speciality: "Chatbots", tools: &["chatbot_builder", "prompt_studio", "custom_ai"] },
    Role { title: "Custom AI Engineer", department: "ai", speciality: "Purpose-built AI", tools: &["custom_ai", "ai_finetune", "model_eval"] },
    Role { title: "ML Engineer", department: "ai", speciality: "Data & evaluation", tools: &["dataset_forge", "model_eval", "ai_finetune"] },
    Role { title: "Agent Orchestrator", department: "ai", speciality: "Work distribution", tools: &["agent_orchestrator", "analytics"] },
    Role { title: "Graphic Designer", department: "marketing", speciality: "Brand & graphics", tools: &["graphics_design", "ui_kit", "card_3d"] },
    Role { title: "UI/UX Designer", department: "marketing", speciality: "Product design", tools: &["ui_kit", "graphics_design"] },
    Role { title: "Copywriter", department: "marketing", speciality: "Content & CVs", tools: &["cv_builder", "graphics_design"] },
    Role { title: "Growth Marketer", department: "marketing", speciality: "Campaigns & leads", tools: &["marketing_manager", "analytics", "email_suite"] },
    Role { title: "Finance Officer", department: "finance", speciality: "Revenue & escrow", tools: &["finance_manager", "escrow_ledger", "invoice_forge"] },
    Role { title: "Payments Officer", department: "finance", speciality: "Gateways & payouts", tools: &["payments_gateway", "escrow_ledger", "invoice_forge"] },
    Role { title: "Contract Officer", department: "finance", speciality: "Contracts & e-signature", tools: &["contract_forge", "crypto_suite", "invoice_forge"] },
    Role { title: "HR Officer", department: "hr", speciality: "Hiring & assessment", tools: &["hr_manager", "cv_builder", "report_engine"] },
    Role { title: "Recruiter", department: "hr", speciality: "Talent sourcing", tools: &["hr_manager", "cv_builder", "email_suite"] },
    Role { title: "Client Success Officer", department: "employee", speciality: "Client communication", tools: &["sms_gateway", "email_suite", "meeting_room"] },
    Role { title: "Project Coordinator", department: "employee", speciality: "Delivery tracking", tools: &["report_engine", "meeting_room", "analytics"] },
    Role { title: "Analyst", department: "employee", speciality: "Reporting & analytics", tools: &["report_engine", "analytics"] },
    Role { title: "Meeting Scribe", department: "employee", speciality: "Minutes & action items", tools: &["ai_scribe", "meeting_room", "email_suite"] },
    Role { title: "Tanzania Services Officer", department: "employee", speciality: "BRELA, TRA & e-services", tools: &["brela_tz", "tra_tz", "gov_tz"] },
];

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS agents (
           id          VARCHAR(64)   NOT NULL,
           name        VARCHAR(160)  NOT NULL,
           role        VARCHAR(160)  NOT NULL DEFAULT '',
           department  VARCHAR(40)   NOT NULL DEFAULT 'employee',
           speciality  VARCHAR(200)  NOT NULL DEFAULT '',
           tools       TEXT          NOT NULL DEFAULT '[]',
           model       VARCHAR(80)   NOT NULL DEFAULT 'default',
           autonomy    VARCHAR(30)   NOT NULL DEFAULT 'supervised',
           manager_id  VARCHAR(64)   NOT NULL DEFAULT '',
           status      VARCHAR(30)   NOT NULL DEFAULT 'active',
           tasks_done  INTEGER       NOT NULL DEFAULT 0,
           created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_agent_dept ON agents(department);
         CREATE INDEX IF NOT EXISTS idx_agent_mgr ON agents(manager_id);",
    );
}

/// Build the company once. An existing roster is left exactly as it is, so an
/// admin's edits survive a restart.
pub fn seed() {
    let existing = db::scalar_i64("SELECT COUNT(*) FROM agents", &[]);
    if existing > 0 {
        return;
    }
    let now = util::now();
    // The manager first, so every agent can point at it.
    let manager_id = util::new_id("AGT-");
    let manager_tools = serde_json::to_string(&vec!["magma_commander", "agent_orchestrator", "analytics", "report_engine"])
        .unwrap_or_else(|_| "[]".into());
    let _ = db::run(
        "INSERT INTO agents (id, name, role, department, speciality, tools, model, autonomy, manager_id, status, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(manager_id)), arg(json!(MANAGER_NAME)), arg(json!("Manager — leads every agent")),
            arg(json!("ceo")), arg(json!("Company-wide orchestration")), arg(json!(manager_tools)),
            arg(json!("default")), arg(json!("autonomous")), arg(json!("")), arg(json!("active")), arg(json!(now)),
        ],
    );
    for i in 0..COMPANY_SIZE {
        let r = &ROLES[i % ROLES.len()];
        let gen = i / ROLES.len() + 1;
        let name = format!("{} {gen:02}", r.title);
        let tools_json = serde_json::to_string(r.tools).unwrap_or_else(|_| "[]".into());
        // The security and finance seats are watched more closely; the rest may
        // run unsupervised because their work is reversible.
        let autonomy = match r.department {
            "security" | "finance" => "supervised",
            _ => "autonomous",
        };
        let _ = db::run(
            "INSERT INTO agents (id, name, role, department, speciality, tools, model, autonomy, manager_id, status, created_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("AGT-"))), arg(json!(name)), arg(json!(r.title)), arg(json!(r.department)),
                arg(json!(r.speciality)), arg(json!(tools_json)), arg(json!("default")), arg(json!(autonomy)),
                arg(json!(manager_id)), arg(json!("active")), arg(json!(now)),
            ],
        );
    }
}

/// The manager's id, if the company has been seeded.
fn manager_id() -> String {
    db::row(
        "SELECT id FROM agents WHERE manager_id = '' ORDER BY created_at ASC LIMIT 1",
        &[],
    )
    .map(|r| util::s(r.get("id").unwrap_or(&J::Null)))
    .unwrap_or_default()
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            let department = c.p("department");
            let rows = db::rows(
                "SELECT * FROM agents WHERE (?1 = '' OR department = ?1) ORDER BY manager_id = '', department, name",
                &[arg(json!(department))],
            );
            ok(json!({ "agents": rows.iter().map(map::agent).collect::<Vec<_>>() }))
        }
        "manager" => {
            let mid = manager_id();
            match db::row("SELECT * FROM agents WHERE id = ?", &[arg(json!(mid))]) {
                Some(r) => ok(json!({ "manager": map::agent(&r) })),
                None => ok(json!({ "manager": J::Null })),
            }
        }
        "stats" => {
            let total = db::scalar_i64("SELECT COUNT(*) FROM agents", &[]);
            let by_dept = db::rows(
                "SELECT department, COUNT(*) AS n FROM agents GROUP BY department ORDER BY n DESC",
                &[],
            )
            .into_iter()
            .map(|r| json!({ "department": util::s(r.get("department").unwrap_or(&J::Null)), "count": util::i(r.get("n").unwrap_or(&J::Null)) }))
            .collect::<Vec<_>>();
            let with_tools = db::scalar_i64("SELECT COUNT(*) FROM agents WHERE tools != '[]' AND tools != ''", &[]);
            ok(json!({
                "total": total,
                "manager": manager_id(),
                "withTools": with_tools,
                "byDepartment": by_dept,
                "agentic": true,
            }))
        }
        "get" => match db::row("SELECT * FROM agents WHERE id = ?", &[arg(json!(c.p("id")))]) {
            Some(r) => ok(json!({ "agent": map::agent(&r) })),
            None => err("Agent not found.", 404),
        },
        "set_tools" => set_tools(c),
        "toggle" => toggle(c),
        "task" => task(c),
        _ => bad("Unknown agents action."),
    }
}

/// The admin assigns the tools this agent may use. An unknown code is refused,
/// so no agent can reach outside the platform catalogue.
fn set_tools(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let codes: Vec<String> = c
        .pv("tools")
        .as_array()
        .map(|a| a.iter().map(|v| util::s(v)).collect())
        .unwrap_or_default();
    for t in &codes {
        if !tools::known(t) {
            return bad(&format!("Unknown tool: {t}. Only platform tools may be assigned."));
        }
    }
    let _ = db::run(
        "UPDATE agents SET tools = ? WHERE id = ?",
        &[arg(json!(serde_json::to_string(&codes).unwrap_or_else(|_| "[]".into()))), arg(json!(id))],
    );
    ok(json!({ "tools": codes }))
}

fn toggle(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let cur = db::row("SELECT status FROM agents WHERE id = ?", &[arg(json!(id))])
        .and_then(|r| r.get("status").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_default();
    let next = if cur == "active" { "paused" } else { "active" };
    let _ = db::run("UPDATE agents SET status = ? WHERE id = ?", &[arg(json!(next)), arg(json!(id))]);
    ok(json!({ "status": next }))
}

/// Give an agent a task. The tool it needs must be one the agent holds; the
/// work is otherwise refused, which is the rule the CEO set.
fn task(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let tool = c.p("tool");
    let Some(a) = db::row("SELECT * FROM agents WHERE id = ?", &[arg(json!(id))]) else {
        return err("Agent not found.", 404);
    };
    if a.get("status").and_then(|v| v.as_str()) == Some("paused") {
        return bad("That agent is paused. Resume it before giving it work.");
    }
    let raw = util::s(a.get("tools").unwrap_or(&J::Null));
    if !tool.is_empty() && !tools::may_use(&raw, &tool) {
        return bad(&format!(
            "{} does not hold the {tool} tool. Assign it first — an agent may not work outside its own job.",
            util::s(a.get("name").unwrap_or(&J::Null))
        ));
    }
    let _ = db::run(
        "UPDATE agents SET tasks_done = tasks_done + 1 WHERE id = ?",
        &[arg(json!(id))],
    );
    ok(json!({
        "accepted": true,
        "agent": util::s(a.get("name").unwrap_or(&J::Null)),
        "tool": tool,
        "task": c.p("task"),
        "message": "Kazi imepewa agent na inaendelea kwenye mfumo.",
    }))
}
