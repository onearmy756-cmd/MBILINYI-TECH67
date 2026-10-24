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
        let k = r
            .get("skey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let v = r
            .get("svalue")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !k.is_empty() {
            out.insert(k, v);
        }
    }
    *g = Some(out.clone());
    out
}

pub fn get(key: &str, default: &str) -> String {
    all()
        .get(key)
        .map(|s| s.clone())
        .unwrap_or_else(|| default.to_string())
}

pub fn num(key: &str, default: f64) -> f64 {
    let raw = get(key, "");
    if raw.trim().is_empty() {
        return default;
    }
    raw.trim().parse().unwrap_or(default)
}

pub fn flag(key: &str) -> bool {
    matches!(
        get(key, "").trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
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
        if s.is_empty() {
            get("pay_instructions", "")
        } else {
            s
        }
    };
    let en = {
        let s = get("pay_instructions_en", "");
        if s.is_empty() {
            get("pay_instructions", "")
        } else {
            s
        }
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

pub const CATALOGUE_KEYS: [&str; 21] = [
    "custom_software",
    "system_analysis",
    "it_consultancy",
    "maintenance",
    "cybersecurity",
    "forensics",
    "ai_chatbots",
    "ai_finetune",
    "cloud_devops",
    "brela",
    "tra_tax",
    "troubleshooting",
    "windows_os",
    "email_setup",
    "gov_eservices",
    "software_install",
    "graphics_design",
    "cv_writing",
    "mobile_desktop_app",
    "iot_gaming",
    "industry_solutions",
];

/// Services that only exist in Tanzania. Everyone else must not see or order
/// them. Held in settings so the CEO can move a service in or out without a
/// redeploy; the default covers BRELA, TRA and the government e-services.
pub fn tz_only_keys() -> Vec<String> {
    get("services_tz_only", "brela,tra_tax,gov_eservices")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Is this service restricted to Tanzania? Matched on letters and digits only,
/// so "BRELA & Business Registration" is recognised as the `brela` key.
pub fn is_tz_only(label: &str) -> bool {
    is_tz_only_in(label, &tz_only_keys())
}

/// Label fragments that identify a restricted service whose human label does
/// not contain its catalogue key. "Government e-Services" has no `eservices`
/// in it once the hyphen is dropped, so it is spelled out here. A key the CEO
/// adds themselves still matches on the key itself.
fn tz_aliases(key: &str) -> &'static [&'static str] {
    match key {
        "gov_eservices" => &["government", "nida", "nhif", "nssf", "ecitizen", "huduma"],
        _ => &[],
    }
}

fn squash(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// The pure half of `is_tz_only`, so it can be tested without a database.
fn is_tz_only_in(label: &str, keys: &[String]) -> bool {
    let key = squash(label);
    if key.is_empty() {
        return false;
    }
    keys.iter()
        .any(|k| key.contains(&squash(k)) || tz_aliases(k).iter().any(|a| key.contains(a)))
}

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
        (
            "website_basic",
            "Basic / starter website (up to 5 pages)",
            "Tovuti ya msingi (kurasa hadi 5)",
            "/project",
            "",
        ),
        (
            "website_business",
            "Business website (custom design, CMS)",
            "Tovuti ya kampuni (muundo maalum, CMS)",
            "/project",
            "",
        ),
        (
            "ecommerce",
            "Online shop / e-commerce with payments",
            "Duka la mtandaoni lenye malipo",
            "/project",
            "",
        ),
        (
            "mobile_app",
            "Mobile app (Android + iOS)",
            "Programu ya simu (Android + iOS)",
            "/project",
            "",
        ),
        (
            "custom_system",
            "Custom business system (ERP / school / pharmacy)",
            "Mfumo maalum wa biashara (ERP / shule / duka)",
            "/project",
            "",
        ),
        (
            "whatsapp_bot",
            "WhatsApp / SMS chatbot (Swahili + English)",
            "Chatbot ya WhatsApp / SMS (Kiswahili + Kiingereza)",
            "/project",
            "",
        ),
        (
            "ai_finetune",
            "AI fine-tuning on your own data (RAG / LoRA)",
            "Kurekebisha AI kwa data yako (RAG / LoRA)",
            "/project",
            "",
        ),
        (
            "security_audit",
            "Cyber-security audit (OWASP Top-10)",
            "Ukaguzi wa usalama (OWASP Top-10)",
            "/one-time",
            "cybersecurity",
        ),
        (
            "pentest",
            "Penetration test (manual + automated)",
            "Mtihani wa uvamizi (manual + automated)",
            "/system",
            "cybersecurity",
        ),
        (
            "maintenance",
            "Software maintenance & SLA (Starter)",
            "Matengenezo ya programu na SLA (Starter)",
            "/month",
            "",
        ),
        (
            "brela",
            "BRELA business / company registration",
            "Usajili wa biashara / kampuni ya BRELA",
            "/service",
            "",
        ),
        (
            "tra_tax",
            "TRA tax services (TIN, VAT, PAYE, returns)",
            "Huduma za kodi za TRA (TIN, VAT, PAYE)",
            "/service",
            "",
        ),
        (
            "forensics",
            "Digital forensics & cyber-crime investigation",
            "Uchunguzi wa kidijitali na uhalifu wa mtandao",
            "/case",
            "forensics",
        ),
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
    let defaults: [(&str, &str); 52] = [
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
        // Shillings per US dollar, used when a payer chooses the other currency.
        ("fx_usd_tzs", "2600"),
        // The other currencies the platform quotes, each as shillings per unit.
        // Wise can refresh these; the figures here are the fallback.
        ("fx_eur_tzs", "2800"),
        ("fx_gbp_tzs", "3300"),
        ("fx_kes_tzs", "20"),
        ("fx_source", "manual"),
        ("fx_updated_at", ""),
        // Payroll. The worker's share of a released contract, then the statutory
        // deductions. Bands are "ceiling:rate" pairs; the last is the top band.
        ("payroll_worker_share", "0.70"),
        ("payroll_nssf_rate", "0.10"),
        ("payroll_wcf_rate", "0.006"),
        ("payroll_paye_bands", "270000:0,520000:0.08,760000:0.20,1000000:0.25,9999999999:0.30"),
        ("platform_name", "MbilinyiTech Platform"),
        ("webauthn_rp_name", "Mbilinyi Tech Solutions"),
        // The AI provider the agent engine calls. Empty provider means "work it
        // out from whichever key is present"; the key itself is never seeded.
        ("llm_provider", ""),
        ("llm_model", ""),
        ("llm_base_url", ""),
        ("llm_max_tokens", "4096"),
        ("llm_temperature", "0.3"),
        ("llm_api_key", ""),
        // The ICE servers a live call uses. STUN alone cannot connect two
        // phones behind mobile carriers' symmetric NAT, so a TURN server is
        // needed for real calls. JSON, so the CEO can add their own without a
        // rebuild; the credentials stay in the settings table, not in the code.
        ("call_ice_servers", "[]"),
        // The SMS gateway the office sends through. Empty means "not
        // configured", and a send is then refused rather than faked.
        ("sms_gateway", ""),
        ("sms_base_url", ""),
        ("sms_api_key", ""),
        ("sms_secret", ""),
        ("sms_sender_id", ""),
        // The SMTP server the platform sends mail through. Empty means "not
        // configured"; mail is then refused rather than silently dropped.
        ("smtp_host", ""),
        ("smtp_port", "587"),
        ("smtp_user", ""),
        ("smtp_pass", ""),
        ("smtp_from", ""),
        ("smtp_sender_name", "Mbilinyi Tech Solutions"),
        // Stripe's webhook signing secret, so a callback can be authenticated
        // rather than believed.
        ("gw_stripe_webhook_secret", ""),
        ("gw_stripe_enabled", ""),
        // The company account the money settles into once escrow releases.
        ("company_account_name", "Mbilinyi Tech Solutions"),
        ("company_bank", "NMB Bank"),
        ("company_account_no", "51710099563"),
        ("company_swift", "NMBTZTZ"),
        ("company_mobile", "0796752645"),
        // The public base URL, used for Stripe redirects and canonical links.
        ("platform_url", ""),
        // Where the CEO's alerts go. A request, an inbound message or a contract
        // to sign is announced to both of these, so the CEO can decide from a
        // phone. Empty means that channel is simply not used.
        ("admin_alert_phone", "0796752645"),
        ("admin_alert_email", ""),
        // The domains the security sandbox may probe. Empty means nothing is
        // scanned: the client's written permission comes first, always.
        ("security_authorised_domains", ""),
        // The company account money settles into once the client accepts the
        // delivery and escrow releases.
        ("company_settlement_account", "51710099563"),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Vec<String> {
        vec!["brela".into(), "tra_tax".into(), "gov_eservices".into()]
    }

    #[test]
    fn the_tz_only_list_is_configurable_and_sane() {
        crate::test_bootstrap();
        let ks = tz_only_keys();
        assert!(ks.contains(&"brela".to_string()));
        assert!(ks.contains(&"tra_tax".to_string()));
    }

    #[test]
    fn tz_only_matches_by_label_not_exact_spelling() {
        // The grid sends a long human label, not the catalogue key.
        assert!(is_tz_only_in("BRELA & Business Registration", &keys()));
        assert!(is_tz_only_in("TRA & Tax Services", &keys()));
        assert!(is_tz_only_in("Government e-Services", &keys()));
    }

    #[test]
    fn international_services_are_never_tz_only() {
        for label in [
            "Custom Software Development",
            "Graphics Design",
            "CV & Profile Writing",
            "AI Fine-Tuning",
            "Cybersecurity Services",
        ] {
            assert!(
                !is_tz_only_in(label, &keys()),
                "{label} must be available worldwide"
            );
        }
    }

    #[test]
    fn empty_list_restricts_nothing() {
        assert!(!is_tz_only_in("BRELA & Business Registration", &[]));
        assert!(!is_tz_only_in("", &keys()));
    }

    #[test]
    fn catalogue_covers_every_service_the_ui_offers() {
        // A service missing from the catalogue cannot be switched off by the CEO.
        for k in [
            "graphics_design",
            "cv_writing",
            "mobile_desktop_app",
            "iot_gaming",
            "industry_solutions",
        ] {
            assert!(
                CATALOGUE_KEYS.contains(&k),
                "{k} is missing from CATALOGUE_KEYS"
            );
        }
    }
}
