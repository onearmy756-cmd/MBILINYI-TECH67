//! The in-system tool catalogue.
//!
//! Every tool a member of staff or an agent is allowed to use lives here, not
//! somewhere outside the platform. The admin grants tools per employee and per
//! agent; nothing else can reach them. A tool a worker was not granted is
//! refused at the endpoint, so "no-one may work outside their own job" is
//! enforced rather than merely promised.
//!
//! The catalogue is data, so the UI can render it and the grant check is a
//! single lookup.

use crate::api::ctx::Ctx;
use crate::db;
use crate::util::{self, bad, ok};
use axum::response::Response;
use serde_json::{json, Value as J};

/// One tool: a code, a human name, the family it belongs to and the blurb the
/// admin reads before granting it.
pub struct ToolDef {
    pub code: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub icon: &'static str,
    pub desc: &'static str,
    /// Agentic tools run themselves and report back; the rest wait for input.
    pub agentic: bool,
}

pub const CATALOGUE: &[ToolDef] = &[
    // --- Software development -------------------------------------------
    ToolDef { code: "code_studio", name: "Code Studio", family: "software", icon: "fa-solid fa-code", desc: "Write, run and review code in the platform.", agentic: true },
    ToolDef { code: "version_forge", name: "Version Forge", family: "software", icon: "fa-solid fa-code-branch", desc: "Branches, diffs and merge requests.", agentic: false },
    ToolDef { code: "build_pipeline", name: "Build Pipeline", family: "software", icon: "fa-solid fa-gears", desc: "Compile, package and release builds.", agentic: true },
    ToolDef { code: "test_runner", name: "Test Runner", family: "software", icon: "fa-solid fa-vial", desc: "Run unit, integration and end-to-end suites.", agentic: true },
    ToolDef { code: "deployer", name: "Deployer", family: "software", icon: "fa-solid fa-rocket", desc: "Ship to staging and production.", agentic: true },
    ToolDef { code: "api_forge", name: "API Forge", family: "software", icon: "fa-solid fa-plug", desc: "Design and publish APIs.", agentic: true },
    ToolDef { code: "db_forge", name: "Database Forge", family: "software", icon: "fa-solid fa-database", desc: "Schema, migrations and queries.", agentic: false },
    ToolDef { code: "mobile_forge", name: "Mobile Forge", family: "software", icon: "fa-solid fa-mobile-screen", desc: "Build Android and iOS apps.", agentic: true },
    ToolDef { code: "desktop_forge", name: "Desktop Forge", family: "software", icon: "fa-solid fa-desktop", desc: "Build desktop installers.", agentic: true },
    ToolDef { code: "game_forge", name: "Game Studio", family: "software", icon: "fa-solid fa-gamepad", desc: "Game and 3D interactive builds.", agentic: true },
    ToolDef { code: "iot_forge", name: "IoT Forge", family: "software", icon: "fa-solid fa-microchip", desc: "Device firmware and telemetry.", agentic: true },
    ToolDef { code: "os_forge", name: "OS Forge", family: "software", icon: "fa-solid fa-compact-disc", desc: "Operating-system images and drivers.", agentic: true },
    // --- Cyber security --------------------------------------------------
    ToolDef { code: "pentest", name: "Penetration Test Kit", family: "security", icon: "fa-solid fa-user-secret", desc: "Authorised attack simulation.", agentic: true },
    ToolDef { code: "vuln_scanner", name: "Vulnerability Scanner", family: "security", icon: "fa-solid fa-radar", desc: "Continuous weakness detection.", agentic: true },
    ToolDef { code: "owasp_audit", name: "OWASP Audit", family: "security", icon: "fa-solid fa-list-check", desc: "OWASP Top-10 review with fixes.", agentic: true },
    ToolDef { code: "malware_scan", name: "Malware Scan", family: "security", icon: "fa-solid fa-bug-slash", desc: "Find and remove hostile code.", agentic: true },
    ToolDef { code: "firewall_manager", name: "Firewall Manager", family: "security", icon: "fa-solid fa-shield-halved", desc: "Rules, WAF and rate limits.", agentic: false },
    ToolDef { code: "crypto_suite", name: "Cryptography Suite", family: "security", icon: "fa-solid fa-key", desc: "ECDSA signing, hashing, key custody.", agentic: false },
    ToolDef { code: "quantum_guard", name: "Quantum Guard", family: "security", icon: "fa-solid fa-atom", desc: "Post-quantum migration planning.", agentic: true },
    ToolDef { code: "forensics", name: "Digital Forensics", family: "security", icon: "fa-solid fa-fingerprint", desc: "Incident and evidence analysis.", agentic: true },
    // --- AI --------------------------------------------------------------
    ToolDef { code: "ai_finetune", name: "AI Fine-Tuning", family: "ai", icon: "fa-solid fa-brain", desc: "Train and fine-tune models on your data.", agentic: true },
    ToolDef { code: "chatbot_builder", name: "Chatbot Builder", family: "ai", icon: "fa-solid fa-comments", desc: "Build and host chatbots.", agentic: true },
    ToolDef { code: "custom_ai", name: "Custom AI Studio", family: "ai", icon: "fa-solid fa-robot", desc: "Purpose-built AI for a client's work.", agentic: true },
    ToolDef { code: "prompt_studio", name: "Prompt Studio", family: "ai", icon: "fa-solid fa-wand-magic-sparkles", desc: "Author and version prompts.", agentic: false },
    ToolDef { code: "dataset_forge", name: "Dataset Forge", family: "ai", icon: "fa-solid fa-table-columns", desc: "Clean and label training data.", agentic: true },
    ToolDef { code: "model_eval", name: "Model Evaluation", family: "ai", icon: "fa-solid fa-chart-line", desc: "Score accuracy and safety.", agentic: true },
    ToolDef { code: "agent_orchestrator", name: "Agent Orchestrator", family: "ai", icon: "fa-solid fa-sitemap", desc: "Assign work across the agent company.", agentic: true },
    ToolDef { code: "magma_commander", name: "Magma Commander", family: "ai", icon: "fa-solid fa-diagram-project", desc: "Lead and supervise every agent.", agentic: true },
    // --- Design & documents ---------------------------------------------
    ToolDef { code: "graphics_design", name: "Graphics Design", family: "design", icon: "fa-solid fa-palette", desc: "Logos, posters and brand kits.", agentic: true },
    ToolDef { code: "cv_builder", name: "CV & Document Builder", family: "design", icon: "fa-solid fa-file-lines", desc: "CVs, reports and proposals.", agentic: true },
    ToolDef { code: "ui_kit", name: "UI Kit", family: "design", icon: "fa-solid fa-layer-group", desc: "Screens and design systems.", agentic: false },
    ToolDef { code: "card_3d", name: "3D Card Studio", family: "design", icon: "fa-solid fa-cube", desc: "3D cards and product visuals.", agentic: true },
    // --- Business & Tanzanian services ----------------------------------
    ToolDef { code: "brela_tz", name: "BRELA (Tanzania)", family: "business", icon: "fa-solid fa-building-columns", desc: "Company and business registration. Tanzania only.", agentic: false },
    ToolDef { code: "tra_tz", name: "TRA & Tax (Tanzania)", family: "business", icon: "fa-solid fa-file-invoice-dollar", desc: "TIN, returns and TRA systems. Tanzania only.", agentic: false },
    ToolDef { code: "gov_tz", name: "Government e-Services (Tanzania)", family: "business", icon: "fa-solid fa-landmark", desc: "NIDA, NHIF, NSSF support. Tanzania only.", agentic: false },
    ToolDef { code: "invoice_forge", name: "Invoice Forge", family: "business", icon: "fa-solid fa-receipt", desc: "Quotes, invoices and receipts.", agentic: false },
    ToolDef { code: "contract_forge", name: "Contract Forge", family: "business", icon: "fa-solid fa-file-signature", desc: "Contracts with ECDSA e-signature.", agentic: false },
    ToolDef { code: "report_engine", name: "Report Engine", family: "business", icon: "fa-solid fa-chart-pie", desc: "Analytics and R reports.", agentic: true },
    ToolDef { code: "escrow_ledger", name: "Escrow Ledger", family: "business", icon: "fa-solid fa-vault", desc: "Hold, release and settle funds.", agentic: false },
    // --- Communication ---------------------------------------------------
    ToolDef { code: "sms_gateway", name: "SMS Gateway", family: "comms", icon: "fa-solid fa-comment-sms", desc: "Notify clients and staff by SMS.", agentic: true },
    ToolDef { code: "email_suite", name: "Email Suite", family: "comms", icon: "fa-solid fa-envelope", desc: "Transactional email and SMTP.", agentic: true },
    ToolDef { code: "video_call", name: "Video Call Room", family: "comms", icon: "fa-solid fa-video", desc: "Live meetings inside the platform.", agentic: false },
    ToolDef { code: "ai_scribe", name: "AI Scribe", family: "comms", icon: "fa-solid fa-pen-nib", desc: "Meeting minutes and action items.", agentic: true },
    ToolDef { code: "meeting_room", name: "Online Meeting Room", family: "comms", icon: "fa-solid fa-users-rectangle", desc: "Scheduled online meetings.", agentic: false },
    // --- Platform --------------------------------------------------------
    ToolDef { code: "payments_gateway", name: "Payments Gateway", family: "platform", icon: "fa-solid fa-credit-card", desc: "Stripe (USD) and mobile money (TZS).", agentic: false },
    ToolDef { code: "analytics", name: "Growth Analytics", family: "platform", icon: "fa-solid fa-globe", desc: "Service growth by country and region.", agentic: true },
    ToolDef { code: "hr_manager", name: "HR Manager", family: "platform", icon: "fa-solid fa-people-roof", desc: "Hiring, assessment and staffing.", agentic: true },
    ToolDef { code: "finance_manager", name: "Finance Manager", family: "platform", icon: "fa-solid fa-coins", desc: "Revenue, escrow and payouts.", agentic: true },
    ToolDef { code: "marketing_manager", name: "Marketing Manager", family: "platform", icon: "fa-solid fa-bullhorn", desc: "Campaigns and lead generation.", agentic: true },
    // --- Agent runtime (the OpenHands-class tools, brought in from MTECHOS) --
    ToolDef { code: "browser", name: "Browser", family: "agent", icon: "fa-solid fa-window-maximize", desc: "Drive a real browser to read and act on pages.", agentic: true },
    ToolDef { code: "web", name: "Web Research", family: "agent", icon: "fa-solid fa-magnifying-glass", desc: "Search and read the open web.", agentic: true },
    ToolDef { code: "code_execution", name: "Code Execution", family: "agent", icon: "fa-solid fa-terminal", desc: "Run code in a sandbox and read the result.", agentic: true },
    ToolDef { code: "terminal", name: "Terminal", family: "agent", icon: "fa-solid fa-square-terminal", desc: "Shell access inside the workspace.", agentic: true },
    ToolDef { code: "file_ops", name: "File Operations", family: "agent", icon: "fa-solid fa-folder-open", desc: "Read, write and edit files.", agentic: false },
    ToolDef { code: "ide_workspace", name: "IDE Workspace", family: "agent", icon: "fa-solid fa-laptop-code", desc: "The online editor every worker codes in.", agentic: false },
    ToolDef { code: "memory", name: "Memory", family: "agent", icon: "fa-solid fa-box-archive", desc: "Long-term memory for agents and staff.", agentic: true },
    ToolDef { code: "session_search", name: "Session Search", family: "agent", icon: "fa-solid fa-clock-rotate-left", desc: "Search everything the system has done before.", agentic: true },
    ToolDef { code: "delegation", name: "Delegation", family: "agent", icon: "fa-solid fa-people-arrows", desc: "Hand a task to another agent or worker.", agentic: true },
    ToolDef { code: "swarm", name: "Agent Swarm", family: "agent", icon: "fa-solid fa-hexagon-nodes", desc: "Run many agents on one problem at once.", agentic: true },
    ToolDef { code: "mixture_of_agents", name: "Mixture of Agents", family: "agent", icon: "fa-solid fa-brain", desc: "Cross-check several agents' answers.", agentic: true },
    ToolDef { code: "cronjob", name: "Scheduled Jobs", family: "agent", icon: "fa-solid fa-clock", desc: "Run work on a schedule, unattended.", agentic: true },
    ToolDef { code: "skills", name: "Skills Library", family: "agent", icon: "fa-solid fa-book-open", desc: "Reusable procedures a worker can call.", agentic: false },
    ToolDef { code: "todo", name: "Task Board", family: "agent", icon: "fa-solid fa-list-check", desc: "Plan and track the steps of a job.", agentic: false },
    ToolDef { code: "clarify", name: "Ask the Client", family: "agent", icon: "fa-solid fa-circle-question", desc: "Ask a question when a job is ambiguous.", agentic: false },
    ToolDef { code: "process_registry", name: "Process Registry", family: "agent", icon: "fa-solid fa-list-ul", desc: "See and manage running processes.", agentic: false },
    ToolDef { code: "managed_gateway", name: "Managed Tool Gateway", family: "agent", icon: "fa-solid fa-door-open", desc: "The single doorway every tool call passes.", agentic: false },
    ToolDef { code: "result_storage", name: "Result Storage", family: "agent", icon: "fa-solid fa-hard-drive", desc: "Store large outputs instead of losing them.", agentic: false },
    ToolDef { code: "deploy_runner", name: "Deploy Runner", family: "agent", icon: "fa-solid fa-truck-fast", desc: "Take a reviewed build live.", agentic: true },
    ToolDef { code: "daily_briefing", name: "Daily Briefing", family: "agent", icon: "fa-solid fa-newspaper", desc: "A written summary of the day's work.", agentic: true },
    // --- Media & vision (from MTECHOS) ------------------------------------
    ToolDef { code: "vision", name: "Vision", family: "media", icon: "fa-solid fa-eye", desc: "Read images, screenshots and documents.", agentic: true },
    ToolDef { code: "image_gen", name: "Image Generation", family: "media", icon: "fa-solid fa-image", desc: "Create images for client work.", agentic: true },
    ToolDef { code: "video_gen", name: "Video Generation", family: "media", icon: "fa-solid fa-film", desc: "Create short video for client work.", agentic: true },
    ToolDef { code: "audio_gen", name: "Audio Generation", family: "media", icon: "fa-solid fa-music", desc: "Create audio and voice-over.", agentic: true },
    ToolDef { code: "tts", name: "Text to Speech", family: "media", icon: "fa-solid fa-volume-high", desc: "Read text aloud, in Kiswahili or English.", agentic: true },
    ToolDef { code: "transcription", name: "Transcription", family: "media", icon: "fa-solid fa-microphone-lines", desc: "Turn a recording into text.", agentic: true },
    ToolDef { code: "voice_mode", name: "Voice Mode", family: "media", icon: "fa-solid fa-phone-volume", desc: "Spoken back-and-forth with a client.", agentic: true },
    ToolDef { code: "media_workflow", name: "Media Workflow", family: "media", icon: "fa-solid fa-diagram-successor", desc: "Chain media steps into one job.", agentic: true },
    // --- Security additions (from MTECHOS) --------------------------------
    ToolDef { code: "threat_intel", name: "Threat Intelligence", family: "security", icon: "fa-solid fa-satellite-dish", desc: "Track threats against clients.", agentic: true },
    ToolDef { code: "kali_lab", name: "Security Lab", family: "security", icon: "fa-solid fa-flask-vial", desc: "An isolated lab for authorised testing.", agentic: true },
    ToolDef { code: "clearance_audit", name: "Clearance Audit", family: "security", icon: "fa-solid fa-id-badge", desc: "Check who may enter which section.", agentic: true },
    ToolDef { code: "code_audit", name: "Code Audit", family: "security", icon: "fa-solid fa-magnifying-glass-chart", desc: "Read code for weaknesses and fixes.", agentic: true },
    ToolDef { code: "dependency_check", name: "Dependency Check", family: "security", icon: "fa-solid fa-cubes", desc: "Flag vulnerable libraries.", agentic: true },
    ToolDef { code: "url_safety", name: "URL Safety", family: "security", icon: "fa-solid fa-link-slash", desc: "Check a link before anyone opens it.", agentic: true },
    ToolDef { code: "guard", name: "Guard Rails", family: "security", icon: "fa-solid fa-shield", desc: "Refuse unsafe actions automatically.", agentic: true },
    ToolDef { code: "threat_swarm", name: "Threat Swarm", family: "security", icon: "fa-solid fa-spider", desc: "Many agents hunting one threat.", agentic: true },
    // --- AI additions (from MTECHOS) --------------------------------------
    ToolDef { code: "continuous_learning", name: "Continuous Learning", family: "ai", icon: "fa-solid fa-arrows-rotate", desc: "Learn from each finished job.", agentic: true },
    ToolDef { code: "ml_local", name: "Local Models", family: "ai", icon: "fa-solid fa-server", desc: "Run models on our own machines.", agentic: true },
    ToolDef { code: "homeassistant", name: "Home & Device Automation", family: "ai", icon: "fa-solid fa-house-signal", desc: "Automate devices and buildings.", agentic: true },
    ToolDef { code: "messaging", name: "Messaging Bridge", family: "comms", icon: "fa-solid fa-comment-dots", desc: "Send and receive client messages.", agentic: true },
    ToolDef { code: "ai_engine", name: "AI Engine", family: "ai", icon: "fa-solid fa-microchip", desc: "The model layer every tool calls.", agentic: false },
];

pub fn find(code: &str) -> Option<&'static ToolDef> {
    CATALOGUE.iter().find(|t| t.code == code)
}

pub fn known(code: &str) -> bool {
    find(code).is_some()
}

/// Only these tools may be handed to an outside client's work. The Tanzanian
/// registration tools are Tanzania-only and are never offered abroad.
pub fn is_tanzania_only(code: &str) -> bool {
    matches!(code, "brela_tz" | "tra_tz" | "gov_tz")
}

/// The catalogue as JSON, grouped for the UI.
pub fn as_json() -> Vec<J> {
    CATALOGUE
        .iter()
        .map(|t| {
            json!({
                "code": t.code,
                "name": t.name,
                "family": t.family,
                "icon": t.icon,
                "desc": t.desc,
                "agentic": t.agentic,
                "tanzaniaOnly": is_tanzania_only(t.code),
            })
        })
        .collect()
}

/// The families, with their tools, for a grouped picker.
pub fn families() -> Vec<J> {
    let mut seen: Vec<&str> = Vec::new();
    for t in CATALOGUE {
        if !seen.contains(&t.family) {
            seen.push(t.family);
        }
    }
    seen.iter()
        .map(|f| {
            json!({
                "family": f,
                "tools": CATALOGUE.iter().filter(|t| t.family == *f).map(|t| json!({
                    "code": t.code, "name": t.name, "icon": t.icon,
                    "agentic": t.agentic, "tanzaniaOnly": is_tanzania_only(t.code),
                })).collect::<Vec<_>>(),
            })
        })
        .collect()
}

/// Every tool granted to an employee or agent, from a stored JSON array.
pub fn granted(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

/// May this holder use this tool? An empty grant means nothing is allowed, so a
/// worker with no tools cannot reach a tool they were never given.
pub fn may_use(grant: &str, code: &str) -> bool {
    granted(grant).iter().any(|c| c == code)
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "catalogue" | "list" => ok(json!({
            "tools": as_json(),
            "families": families(),
            "count": CATALOGUE.len(),
        })),
        // Which tools a named employee or agent holds.
        "for_employee" => {
            let Ok(_a) = c.require_admin() else {
                return util::forbidden("Admin access required.");
            };
            let id = c.p("id");
            let row = db::row("SELECT tools, full_name FROM employees WHERE id = ?", &[db::from_json(json!(id))]);
            match row {
                Some(r) => {
                    let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
                    let codes = granted(&raw);
                    ok(json!({
                        "holder": r.get("full_name"),
                        "tools": codes.iter().filter_map(|x| find(x)).map(|t| json!({
                            "code": t.code, "name": t.name, "family": t.family, "icon": t.icon, "agentic": t.agentic,
                        })).collect::<Vec<_>>(),
                        "codes": codes,
                    }))
                }
                None => bad("No such employee."),
            }
        }
        "for_agent" => {
            let Ok(_a) = c.require_admin() else {
                return util::forbidden("Admin access required.");
            };
            let id = c.p("id");
            let row = db::row("SELECT tools, name FROM agents WHERE id = ?", &[db::from_json(json!(id))]);
            match row {
                Some(r) => {
                    let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
                    let codes = granted(&raw);
                    ok(json!({
                        "holder": r.get("name"),
                        "tools": codes.iter().filter_map(|x| find(x)).map(|t| json!({
                            "code": t.code, "name": t.name, "family": t.family, "icon": t.icon, "agentic": t.agentic,
                        })).collect::<Vec<_>>(),
                        "codes": codes,
                    }))
                }
                None => bad("No such agent."),
            }
        }
        // The admin grants a tool to an employee.
        "grant_employee" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let code = c.p("tool");
            if !known(&code) {
                return bad("That tool is not in the platform catalogue.");
            }
            let Some(r) = db::row("SELECT tools FROM employees WHERE id = ?", &[db::from_json(json!(id))]) else {
                return bad("No such employee.");
            };
            let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
            let mut codes = granted(&raw);
            if !codes.iter().any(|x| x == &code) {
                codes.push(code.clone());
            }
            let _ = db::run(
                "UPDATE employees SET tools = ?, updated_at = ? WHERE id = ?",
                &[db::from_json(json!(serde_json::to_string(&codes).unwrap_or_default())), db::from_json(json!(util::now())), db::from_json(json!(id))],
            );
            ok(json!({ "granted": code, "tools": codes }))
        }
        "revoke_employee" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let code = c.p("tool");
            let Some(r) = db::row("SELECT tools FROM employees WHERE id = ?", &[db::from_json(json!(id))]) else {
                return bad("No such employee.");
            };
            let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
            let codes: Vec<String> = granted(&raw).into_iter().filter(|x| x != &code).collect();
            let _ = db::run(
                "UPDATE employees SET tools = ?, updated_at = ? WHERE id = ?",
                &[db::from_json(json!(serde_json::to_string(&codes).unwrap_or_default())), db::from_json(json!(util::now())), db::from_json(json!(id))],
            );
            ok(json!({ "revoked": code, "tools": codes }))
        }
        "grant_agent" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let code = c.p("tool");
            if !known(&code) {
                return bad("That tool is not in the platform catalogue.");
            }
            let Some(r) = db::row("SELECT tools FROM agents WHERE id = ?", &[db::from_json(json!(id))]) else {
                return bad("No such agent.");
            };
            let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
            let mut codes = granted(&raw);
            if !codes.iter().any(|x| x == &code) {
                codes.push(code.clone());
            }
            let _ = db::run(
                "UPDATE agents SET tools = ? WHERE id = ?",
                &[db::from_json(json!(serde_json::to_string(&codes).unwrap_or_default())), db::from_json(json!(id))],
            );
            ok(json!({ "granted": code, "tools": codes }))
        }
        "revoke_agent" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let code = c.p("tool");
            let Some(r) = db::row("SELECT tools FROM agents WHERE id = ?", &[db::from_json(json!(id))]) else {
                return bad("No such agent.");
            };
            let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
            let codes: Vec<String> = granted(&raw).into_iter().filter(|x| x != &code).collect();
            let _ = db::run(
                "UPDATE agents SET tools = ? WHERE id = ?",
                &[db::from_json(json!(serde_json::to_string(&codes).unwrap_or_default())), db::from_json(json!(id))],
            );
            ok(json!({ "revoked": code, "tools": codes }))
        }
        // The enforcement point: a worker asking to run a tool they were not
        // granted is refused here.
        "can_use" => {
            let Ok(_u) = c.require_user() else {
                return util::unauth();
            };
            let holder = c.p("holder");
            let code = c.p("tool");
            let table = if c.p("kind") == "agent" { "agents" } else { "employees" };
            let row = db::row(
                &format!("SELECT tools FROM {table} WHERE id = ?"),
                &[db::from_json(json!(holder))],
            );
            match row {
                Some(r) => {
                    let raw = r.get("tools").and_then(|v| v.as_str()).unwrap_or("[]").to_string();
                    ok(json!({ "allowed": may_use(&raw, &code), "tool": code }))
                }
                None => ok(json!({ "allowed": false, "tool": code })),
            }
        }
        _ => bad("Unknown tools action."),
    }
}
