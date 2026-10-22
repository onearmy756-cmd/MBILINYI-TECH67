//! The settings table, which is where the admin keeps prices, payment accounts
//! and (now) gateway credentials. Reads are cached for the life of a request
//! batch; every write clears the cache.

use crate::db;
use crate::util;
use rusqlite::types::Value;
use serde_json::{json, Value as J};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static CACHE: OnceLock<Mutex<Option<HashMap<String, String>>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<HashMap<String, String>>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

pub fn all() -> HashMap<String, String> {
    let m = cache();
    let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = g.as_ref() {
        return c.clone();
    }
    let mut out = HashMap::new();
    for r in db::rows("SELECT skey, svalue FROM settings", &[]) {
        let k = r.get("skey").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let v = r.get("svalue").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !k.is_empty() {
            out.insert(k, v);
        }
    }
    *g = Some(out.clone());
    out
}

pub fn get(key: &str, default: &str) -> String {
    all().get(key).map(|s| s.clone()).unwrap_or_else(|| default.to_string())
}

pub fn num(key: &str, default: f64) -> f64 {
    let raw = get(key, "");
    if raw.trim().is_empty() {
        return default;
    }
    raw.trim().parse().unwrap_or(default)
}

pub fn flag(key: &str) -> bool {
    matches!(get(key, "").trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

pub fn set(key: &str, value: &str) {
    let _ = db::run(
        "INSERT INTO settings (skey, svalue) VALUES (?, ?)
         ON CONFLICT(skey) DO UPDATE SET svalue = excluded.svalue",
        &db::args(vec![json!(key), json!(value)]),
    );
    invalidate();
}

pub fn set_many(pairs: &[(String, String)]) {
    for (k, v) in pairs {
        let _ = db::run(
            "INSERT INTO settings (skey, svalue) VALUES (?, ?)
             ON CONFLICT(skey) DO UPDATE SET svalue = excluded.svalue",
            &db::args(vec![json!(k), json!(v)]),
        );
    }
    invalidate();
}

pub fn invalidate() {
    let m = cache();
    let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
    *g = None;
}

pub fn as_json() -> J {
    let mut m = serde_json::Map::new();
    for (k, v) in all() {
        m.insert(k, J::String(v));
    }
    J::Object(m)
}

/// Payment block shown to signed-in users (bank + mobile money), unchanged from
/// the PHP `payment_details()` so the portal keeps rendering it verbatim.
pub fn payment_details() -> J {
    let sw = {
        let s = get("pay_instructions_sw", "");
        if s.is_empty() { get("pay_instructions", "") } else { s }
    };
    let en = {
        let s = get("pay_instructions_en", "");
        if s.is_empty() { get("pay_instructions", "") } else { s }
    };
    json!({
        "bank": {
            "name": get("pay_bank_name", "NMB Bank"),
            "account": get("pay_bank_account", "51710099563"),
            "holder": get("pay_bank_holder", "JACKSON MBILINYI"),
            "branch": get("pay_bank_branch", ""),
            "swift": get("pay_bank_swift", ""),
        },
        "mobile": {
            "network": get("pay_mobile_network", "Vodacom"),
            "number": get("pay_mobile_number", "0796752645"),
            "holder": get("pay_mobile_holder", "JACKSON MBILINYI"),
        },
        "contact": get("pay_contact_person", "Jackson Mbilinyi (CEO)"),
        "currency": get("pay_currency", "TZS"),
        "instructions": get("pay_instructions", ""),
        "instructionsSw": sw,
        "instructionsEn": en,
    })
}

pub const CATALOGUE_KEYS: [&str; 16] = [
    "custom_software", "system_analysis", "it_consultancy", "maintenance",
    "cybersecurity", "forensics", "ai_chatbots", "ai_finetune", "cloud_devops",
    "brela", "tra_tax", "troubleshooting", "windows_os", "email_setup",
    "gov_eservices", "software_install",
];

pub fn service_hidden() -> Vec<String> {
    let raw = get("services_hidden", "");
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn service_visible(key: &str) -> bool {
    !service_hidden().iter().any(|k| k == key)
}

pub fn service_visibility() -> J {
    let hidden = service_hidden();
    let mut m = serde_json::Map::new();
    for k in CATALOGUE_KEYS {
        m.insert(k.to_string(), J::Bool(!hidden.iter().any(|h| h == k)));
    }
    J::Object(m)
}

/// Priced services. `price` is read live from settings so the CEO can change a
/// figure in Admin → Settings and the pricing page follows immediately.
pub fn service_price_list() -> Vec<J> {
    let catalogue: [(&str, &str, &str, &str, &str); 13] = [
        ("website_basic", "Basic / starter website (up to 5 pages)", "Tovuti ya msingi (kurasa hadi 5)", "/project", ""),
        ("website_business", "Business website (custom design, CMS)", "Tovuti ya kampuni (muundo maalum, CMS)", "/project", ""),
        ("ecommerce", "Online shop / e-commerce with payments", "Duka la mtandaoni lenye malipo", "/project", ""),
        ("mobile_app", "Mobile app (Android + iOS)", "Programu ya simu (Android + iOS)", "/project", ""),
        ("custom_system", "Custom business system (ERP / school / pharmacy)", "Mfumo maalum wa biashara (ERP / shule / duka)", "/project", ""),
        ("whatsapp_bot", "WhatsApp / SMS chatbot (Swahili + English)", "Chatbot ya WhatsApp / SMS (Kiswahili + Kiingereza)", "/project", ""),
        ("ai_finetune", "AI fine-tuning on your own data (RAG / LoRA)", "Kurekebisha AI kwa data yako (RAG / LoRA)", "/project", ""),
        ("security_audit", "Cyber-security audit (OWASP Top-10)", "Ukaguzi wa usalama (OWASP Top-10)", "/one-time", "cybersecurity"),
        ("pentest", "Penetration test (manual + automated)", "Mtihani wa uvamizi (manual + automated)", "/system", "cybersecurity"),
        ("maintenance", "Software maintenance & SLA (Starter)", "Matengenezo ya programu na SLA (Starter)", "/month", ""),
        ("brela", "BRELA business / company registration", "Usajili wa biashara / kampuni ya BRELA", "/service", ""),
        ("tra_tax", "TRA tax services (TIN, VAT, PAYE, returns)", "Huduma za kodi za TRA (TIN, VAT, PAYE)", "/service", ""),
        ("forensics", "Digital forensics & cyber-crime investigation", "Uchunguzi wa kidijitali na uhalifu wa mtandao", "/case", "forensics"),
    ];
    let mut out = Vec::new();
    for (code, en, sw, period, svc) in catalogue {
        if !svc.is_empty() && !service_visible(svc) {
            continue;
        }
        out.push(json!({
            "code": code,
            "en": en,
            "sw": sw,
            "price": num(&format!("price_{code}"), 0.0),
            "period": period,
        }));
    }
    out
}

/// Fill only blanks, so an admin edit is never overwritten on boot.
pub fn seed_defaults() {
    let defaults: [(&str, &str); 12] = [
        ("pay_bank_name", "NMB Bank"),
        ("pay_bank_account", "51710099563"),
        ("pay_bank_holder", "JACKSON MBILINYI"),
        ("pay_bank_branch", "Tanzania"),
        ("pay_bank_swift", "NMBTZTZ"),
        ("pay_mobile_network", "Vodacom"),
        ("pay_mobile_number", "0796752645"),
        ("pay_mobile_holder", "JACKSON MBILINYI"),
        ("pay_contact_person", "Jackson Mbilinyi (CEO)"),
        ("pay_currency", "TZS"),
        ("platform_name", "MbilinyiTech Platform"),
        ("webauthn_rp_name", "Mbilinyi Tech Solutions"),
    ];
    let have = all();
    for (k, v) in defaults {
        if !have.contains_key(k) {
            set(k, v);
        }
    }
    let _ = util::new_id("");
}

#[allow(dead_code)]
fn _unused(_v: Value) {}
