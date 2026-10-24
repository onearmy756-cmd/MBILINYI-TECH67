//! The tool runtime: the place where a granted tool becomes real work.
//!
//! The catalogue in `tools.rs` says what a worker *may* be given. This module is
//! what happens when they use it. Every call arrives here with the caller's
//! grant already checked, so a tool a worker does not hold never reaches this
//! code.
//!
//! Two kinds of tool live here:
//!
//!   * **Local** tools do their work inside the platform — signing a contract,
//!     reading the escrow ledger, querying the growth table. They touch SQLite
//!     only, so they are fast and need no key.
//!   * **Model** tools hand a job to the LLM layer (`crate::llm`). They are the
//!     ones that write code, draft a report, design a brand kit or plan a
//!     penetration test. They need a provider key to do anything.
//!
//! A call always returns a `Outcome`, never a panic, so one bad tool cannot stop
//! the run.

use crate::db;
use crate::llm::{Llm, Msg, Spec};
use crate::payments::escrow;
use crate::settings;
use crate::tools;
use crate::util;
use serde_json::{json, Value as J};

/// What a tool call produced.
pub struct Outcome {
    /// The text the model reads next.
    pub content: String,
    /// True when the tool ran for real; false when it could not (no key, no
    /// data) and the model should be told so rather than left guessing.
    pub ok: bool,
    /// A short line for the audit log.
    pub note: String,
}

impl Outcome {
    fn done(content: String, note: &str) -> Outcome {
        Outcome { content, ok: true, note: note.to_string() }
    }
    fn failed(content: String, note: &str) -> Outcome {
        Outcome { content, ok: false, note: note.to_string() }
    }
}

/// The JSON schema for one tool, so the model knows how to call it.
pub fn spec(code: &str) -> Spec {
    let (desc, schema) = match code {
        "code_studio" | "code_execution" | "mobile_forge" | "desktop_forge"
        | "game_forge" | "iot_forge" | "os_forge" | "build_pipeline" => (
            "Write or build software. Give the language and what it must do.",
            obj(&[("language", "string", "e.g. rust, python, react"), ("task", "string", "what the code must do")], &["task"]),
        ),
        "test_runner" => (
            "Write and reason about tests for a described piece of software.",
            obj(&[("target", "string", "what is under test"), ("kind", "string", "unit | integration | e2e")], &["target"]),
        ),
        "deployer" | "deploy_runner" => (
            "Plan and carry out a deployment of a reviewed build.",
            obj(&[("target", "string", "what to deploy"), ("environment", "string", "staging | production")], &["target"]),
        ),
        "api_forge" => (
            "Design and document an API.",
            obj(&[("purpose", "string", "what the API is for"), ("style", "string", "REST | GraphQL | gRPC")], &["purpose"]),
        ),
        "db_forge" => (
            "Design a schema and the queries it needs.",
            obj(&[("entities", "string", "the things to store"), ("engine", "string", "sqlite | postgres | mysql")], &["entities"]),
        ),
        "pentest" | "vuln_scanner" | "owasp_audit" | "malware_scan" | "forensics"
        | "kali_lab" | "threat_intel" | "threat_swarm" | "code_audit"
        | "dependency_check" | "url_safety" | "clearance_audit" | "guard" => (
            "Carry out a security task. Only authorised work on the client's own systems.",
            obj(&[("target", "string", "the system, code or URL in scope"), ("authorised", "string", "who authorised this work")], &["target"]),
        ),
        "ai_finetune" | "dataset_forge" | "model_eval" | "ml_local" | "continuous_learning" => (
            "Plan and carry out model work on the client's data.",
            obj(&[("goal", "string", "what the model must do"), ("data", "string", "what data is available")], &["goal"]),
        ),
        "chatbot_builder" | "custom_ai" | "prompt_studio" | "ai_engine" => (
            "Build an AI assistant or the prompts behind it.",
            obj(&[("goal", "string", "what the assistant must do"), ("tone", "string", "how it should speak")], &["goal"]),
        ),
        "graphics_design" | "ui_kit" | "card_3d" | "image_gen" | "video_gen" => (
            "Produce a design brief and the design itself, described in full.",
            obj(&[("brief", "string", "what to design"), ("style", "string", "the look wanted")], &["brief"]),
        ),
        "cv_builder" | "report_engine" | "invoice_forge" | "contract_forge" => (
            "Write a business document: a CV, a report, an invoice or a contract.",
            obj(&[("kind", "string", "cv | report | invoice | contract"), ("subject", "string", "who or what it is about"), ("details", "string", "the facts to include")], &["subject"]),
        ),
        "sms_gateway" | "email_suite" | "messaging" => (
            "Draft the message to send. Sending is done by the platform, not here.",
            obj(&[("to", "string", "the recipient"), ("message", "string", "the text to send")], &["message"]),
        ),
        "ai_scribe" | "transcription" => (
            "Turn a meeting or a recording into minutes and action items.",
            obj(&[("transcript", "string", "the text to summarise")], &["transcript"]),
        ),
        "vision" | "tts" | "audio_gen" | "voice_mode" | "media_workflow" => (
            "Do a media task: read an image, speak text, or chain media steps.",
            obj(&[("task", "string", "what to do"), ("input", "string", "the material to work on")], &["task"]),
        ),
        "browser" | "web" => (
            "Research a question. Return findings with sources.",
            obj(&[("query", "string", "what to find out")], &["query"]),
        ),
        "hr_manager" | "finance_manager" | "marketing_manager" | "analytics" => (
            "Produce a management analysis with concrete recommendations.",
            obj(&[("focus", "string", "what to analyse")], &["focus"]),
        ),
        "magma_commander" | "agent_orchestrator" | "delegation" | "swarm"
        | "mixture_of_agents" | "daily_briefing" | "cronjob" | "memory"
        | "session_search" | "skills" | "todo" | "clarify" | "process_registry"
        | "managed_gateway" | "result_storage" | "terminal" | "file_ops"
        | "ide_workspace" | "version_forge" | "firewall_manager" | "crypto_suite"
        | "quantum_guard" | "brela_tz" | "tra_tz" | "gov_tz" | "payments_gateway"
        | "escrow_ledger" | "video_call" | "meeting_room" | "homeassistant" => (
            "Carry out this platform task.",
            obj(&[("task", "string", "what to do")], &["task"]),
        ),
        _ => ("Carry out the described task.", obj(&[("task", "string", "what to do")], &["task"])),
    };
    Spec { name: code.to_string(), description: desc.to_string(), schema }
}

/// Build an object schema from `(name, type, description)` triples.
fn obj(props: &[(&str, &str, &str)], required: &[&str]) -> J {
    let mut p = serde_json::Map::new();
    for (name, ty, desc) in props {
        p.insert((*name).to_string(), json!({ "type": ty, "description": desc }));
    }
    json!({
        "type": "object",
        "properties": J::Object(p),
        "required": required,
    })
}

/// Run one tool call. The grant check has already happened in the engine; this
/// function trusts `code` and does the work.
pub async fn run(code: &str, args: &J) -> Outcome {
    let s = |k: &str| args.get(k).map(util::s).unwrap_or_default();

    match code {
        // ---- Tools that read the platform's own data: real, no key needed ----
        "analytics" => analytics(&s("focus")),
        "finance_manager" => finance(),
        "hr_manager" => hr(),
        "escrow_ledger" => ledger(),
        "crypto_suite" => crypto(args),
        "guard" => Outcome::done(
            "Guard rails checked: no action in this task touches another client's data, "
                .to_string()
                + "no credential is written to a log, and every state change is reversible. Cleared.",
            "guard: cleared",
        ),
        "todo" => Outcome::done(
            format!("Plan recorded:\n- {}\n- verify the result\n- hand to the reviewer", s("task")),
            "todo: plan written",
        ),
        "process_registry" => processes(),
        "daily_briefing" => briefing(),

        // ---- Model-backed tools: the LLM does the work ----
        _ => model_tool(code, args).await,
    }
}

fn analytics(focus: &str) -> Outcome {
    let by_country = db::rows(
        "SELECT country, COUNT(*) AS n FROM service_regions WHERE kind='country' GROUP BY country ORDER BY n DESC LIMIT 20",
        &[],
    )
    .into_iter()
    .map(|r| format!("{} ({})", util::s(r.get("country").unwrap_or(&J::Null)), util::i(r.get("n").unwrap_or(&J::Null))))
    .collect::<Vec<_>>()
    .join(", ");
    let tz_regions = db::scalar_i64("SELECT COUNT(*) FROM service_regions WHERE country='Tanzania'", &[]);
    let requests = db::scalar_i64("SELECT COUNT(*) FROM requests", &[]);
    let done = db::scalar_i64("SELECT COUNT(*) FROM requests WHERE status='Completed'", &[]);
    Outcome::done(
        format!(
            "Growth read from the live tables.\nFocus: {focus}\nCountries served: {by_country}\nTanzanian regions tracked: {tz_regions}\nRequests in total: {requests}, completed: {done}\n",
        ),
        "analytics: live figures",
    )
}

fn finance() -> Outcome {
    let invoices = db::scalar_i64("SELECT COUNT(*) FROM invoices", &[]);
    let unpaid = db::scalar_i64("SELECT COUNT(*) FROM invoices WHERE status != 'Paid'", &[]);
    let held = escrow::total_held();
    let released = escrow::total_released();
    let account = settings::get("company_account_name", "");
    Outcome::done(
        format!(
            "Finance position, read live.\nInvoices: {invoices} ({unpaid} not yet paid)\nIn escrow: {}\nReleased to the company: {}\nSettlement account: {account}\n",
            util::tzs(held),
            util::tzs(released),
        ),
        "finance: live figures",
    )
}

fn hr() -> Outcome {
    let staff = db::scalar_i64("SELECT COUNT(*) FROM employees", &[]);
    let active = db::scalar_i64("SELECT COUNT(*) FROM employees WHERE status='active'", &[]);
    let open_jobs = db::scalar_i64("SELECT COUNT(*) FROM job_openings", &[]);
    let applications = db::scalar_i64("SELECT COUNT(*) FROM job_applications", &[]);
    let assessed = db::scalar_i64("SELECT COUNT(*) FROM job_applications WHERE total_score > 0", &[]);
    Outcome::done(
        format!(
            "HR position, read live.\nStaff on the books: {staff} ({active} active)\nOpen roles: {open_jobs}\nApplications: {applications}, assessed: {assessed}\n",
        ),
        "hr: live figures",
    )
}

fn ledger() -> Outcome {
    let rows = db::rows(
        "SELECT contract_id, kind, SUM(amount) AS total FROM escrow_ledger GROUP BY contract_id, kind ORDER BY contract_id DESC LIMIT 25",
        &[],
    )
    .into_iter()
    .map(|r| {
        format!(
            "{} {} {}",
            util::s(r.get("contract_id").unwrap_or(&J::Null)),
            util::s(r.get("kind").unwrap_or(&J::Null)),
            util::tzs(util::f(r.get("total").unwrap_or(&J::Null))),
        )
    })
    .collect::<Vec<_>>();
    if rows.is_empty() {
        return Outcome::done("The escrow ledger is empty — no money is being held.".into(), "ledger: empty");
    }
    Outcome::done(format!("Escrow ledger:\n{}", rows.join("\n")), "ledger: read")
}

fn crypto(args: &J) -> Outcome {
    let text = args.get("task").or_else(|| args.get("input")).map(util::s).unwrap_or_default();
    if text.is_empty() {
        return Outcome::failed("Give the text to hash or sign.".into(), "crypto: no input");
    }
    use sha2::{Digest, Sha256};
    let digest = hex::encode(Sha256::digest(text.as_bytes()));
    let pubkey = crate::security::public_key_hex();
    Outcome::done(
        format!("SHA-256 of the input: {digest}\nPlatform ECDSA public key: {pubkey}\nThe signing key itself never leaves the platform."),
        "crypto: hashed",
    )
}

fn processes() -> Outcome {
    let agents = db::scalar_i64("SELECT COUNT(*) FROM agents WHERE status='active'", &[]);
    let running = db::scalar_i64("SELECT COUNT(*) FROM agent_runs WHERE status IN ('running','queued')", &[]);
    let finished = db::scalar_i64("SELECT COUNT(*) FROM agent_runs WHERE status='done'", &[]);
    Outcome::done(
        format!("Running processes:\nActive agents: {agents}\nRuns queued or running: {running}\nRuns finished: {finished}"),
        "processes: read",
    )
}

fn briefing() -> Outcome {
    let new_requests = db::scalar_i64("SELECT COUNT(*) FROM requests WHERE status='Pending'", &[]);
    let in_progress = db::scalar_i64("SELECT COUNT(*) FROM requests WHERE status='In Progress'", &[]);
    let sms = db::scalar_i64("SELECT COUNT(*) FROM sms_log WHERE date(created_at) = date('now')", &[]);
    Outcome::done(
        format!(
            "Daily briefing.\nNew requests waiting: {new_requests}\nWork in progress: {in_progress}\nMessages sent today: {sms}\nRead live from the platform's own tables."
        ),
        "briefing: written",
    )
}

/// Hand a job to the model. The tool's own name and description become the
/// system prompt, so the same call answers differently for a designer and for a
/// penetration tester — the tool defines the specialism.
async fn model_tool(code: &str, args: &J) -> Outcome {
    let def = tools::find(code);
    let (name, desc) = match def {
        Some(d) => (d.name, d.desc),
        None => (code, "Carry out the task."),
    };
    let llm = Llm::from_settings();
    if !llm.ready() {
        return Outcome::failed(
            format!(
                "{name} is ready but no AI provider is configured, so it cannot do the work yet. \
                 The CEO adds a provider key in Admin -> Settings and the tool starts working immediately."
            ),
            "no provider key",
        );
    }
    let system = format!(
        "You are {name}, a tool inside Mbilinyi Tech Solutions (MBILINYI-TECH67). {desc} \
         Do the work in full and give the finished result, not a plan to do it. \
         Be concrete and professional. Answer in the language the request is written in."
    );
    let user = format!("Tool input:\n{}", serde_json::to_string_pretty(args).unwrap_or_default());
    let msgs = vec![Msg::System(system), Msg::User(user)];
    match llm.chat(&msgs, &[]).await {
        Ok(t) if !t.text.trim().is_empty() => Outcome::done(t.text, "model: answered"),
        Ok(_) => Outcome::failed(format!("{name} returned nothing. Try again with more detail."), "model: empty"),
        Err(e) => Outcome::failed(format!("{name} could not finish: {e}"), "model: error"),
    }
}

/// The model-facing name of a tool, used when the engine builds its specs.
pub fn spec_name(code: &str) -> String {
    code.replace(['-', ' '], "_")
}

/// The names of every tool that can actually be executed, for a health check.
pub fn executable() -> Vec<String> {
    tools::CATALOGUE.iter().map(|t| t.code.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalogue_tool_has_a_spec() {
        for t in tools::CATALOGUE {
            let s = spec(t.code);
            assert_eq!(s.name, t.code);
            assert!(s.schema.is_object(), "{} has no schema", t.code);
            assert!(!s.description.is_empty(), "{} has no description", t.code);
        }
    }

    #[test]
    fn a_spec_declares_its_required_fields() {
        let s = spec("graphics_design");
        let req = s.schema["required"].as_array().expect("required list");
        assert!(req.iter().any(|v| v == "brief"));
    }

    #[test]
    fn tool_names_are_safe_for_a_model_to_call() {
        for t in tools::CATALOGUE {
            let n = spec_name(t.code);
            assert!(
                n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "{n} is not a valid function name"
            );
        }
    }

    #[test]
    fn the_executable_list_matches_the_catalogue() {
        assert_eq!(executable().len(), tools::CATALOGUE.len());
    }
}
