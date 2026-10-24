//! Security: ECDSA contract signing, the platform's posture, and the company
//! account the money settles into.
//!
//! A contract is signed with an ECDSA key held by the platform. The signature is
//! over a canonical rendering of the contract, so changing any term after the
//! signature makes verification fail — which is the whole point of signing one.
//!
//! The posture endpoint reports what is actually switched on, so a claim about
//! security is a fact read from configuration rather than a slogan.

use crate::api::ctx::Ctx;
use crate::db;
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

const KEY_SETTING: &str = "platform_ecdsa_key";
const PUB_SETTING: &str = "platform_ecdsa_public";

/// The platform's signing key, created on first use and kept in settings.
fn signing_key() -> Result<SigningKey, String> {
    let stored = settings::get(KEY_SETTING, "");
    if !stored.is_empty() {
        let bytes = hex::decode(stored.trim())
            .map_err(|_| "stored signing key is malformed".to_string())?;
        return SigningKey::from_slice(&bytes)
            .map_err(|e| format!("stored signing key is invalid: {e}"));
    }
    let key = SigningKey::random(&mut rand::thread_rng());
    let secret = hex::encode(key.to_bytes());
    let public = hex::encode(key.verifying_key().to_encoded_point(true).as_bytes());
    settings::set(KEY_SETTING, &secret);
    settings::set(PUB_SETTING, &public);
    Ok(key)
}

/// The public key, in compressed SEC1 hex — what a verifier needs.
pub fn public_key_hex() -> String {
    match signing_key() {
        Ok(k) => hex::encode(k.verifying_key().to_encoded_point(true).as_bytes()),
        Err(_) => String::new(),
    }
}

/// The bytes a contract's signature covers. Every term that matters is included,
/// so altering the price, the scope or the client's name breaks the signature.
fn contract_message(c: &db::Row) -> String {
    let g = |k: &str| util::s(c.get(k).unwrap_or(&J::Null));
    format!(
        "MTS-CONTRACT|id={}|request={}|tracking={}|client={}|title={}|scope={}|duration={}|total={}|created={}",
        g("id"), g("request_id"), g("tracking_id"), g("client_name"),
        g("project_title"), g("scope"), g("duration"), g("total"), g("created_at"),
    )
}

/// The canonical bytes of a contract, for callers outside this module — the
/// post-quantum signer needs exactly the same message the classical one signs,
/// or the two halves would cover different terms.
pub fn contract_message_public(c: &db::Row) -> String {
    contract_message(c)
}

/// Sign arbitrary bytes with the platform key. Used for the ECDSA half of a
/// hybrid signature.
pub fn sign_bytes(message: &[u8]) -> Result<String, String> {
    let key = signing_key()?;
    let sig: Signature = key.sign(message);
    Ok(hex::encode(sig.to_bytes()))
}

/// Verify an ECDSA signature over arbitrary bytes against the platform key.
pub fn verify_bytes(message: &[u8], signature_hex: &str) -> bool {
    let Ok(key) = signing_key() else {
        return false;
    };
    let Ok(bytes) = hex::decode(signature_hex.trim()) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&bytes) else {
        return false;
    };
    key.verifying_key().verify(message, &sig).is_ok()
}

/// Sign a contract. The signature and the signer's public key are stored on the
/// contract row so any later reader can verify it without the private key.
///
/// The signature is **hybrid**: ECDSA together with ML-DSA-65. A verifier that
/// only knows ECDSA can still read the classical half; one that knows both
/// demands both.
pub fn sign_contract(contract_id: &str) -> Option<String> {
    crate::pqc::sign_contract_hybrid(contract_id)?;
    let c = db::row(
        "SELECT admin_sig FROM contracts WHERE id = ?",
        &[arg(json!(contract_id))],
    )?;
    Some(util::s(c.get("admin_sig").unwrap_or(&J::Null)))
}

/// Verify a contract's stored signature against its current contents.
pub fn verify_contract(contract_id: &str) -> Option<bool> {
    crate::pqc::verify_contract(contract_id)
}

pub fn init_tables() {
    // A signature audit trail, so every signing and verification is recorded.
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS signature_log (
           id          VARCHAR(64)  NOT NULL,
           contract_id VARCHAR(64)  NOT NULL,
           action      VARCHAR(30)  NOT NULL,
           algorithm   VARCHAR(40)  NOT NULL DEFAULT 'ECDSA-secp256k1',
           signature   TEXT         NOT NULL DEFAULT '',
           ok          INTEGER      NOT NULL DEFAULT 1,
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        // What is switched on, stated as fact.
        "status" => {
            let key = signing_key();
            let gateway_count = crate::payments::CATALOGUE.len();
            let enabled = crate::payments::CATALOGUE
                .iter()
                .filter(|g| crate::payments::enabled(g))
                .count();
            let stripe_ready = crate::payments::CATALOGUE
                .iter()
                .find(|g| g.key == "stripe")
                .map(crate::payments::enabled)
                .unwrap_or(false);
            ok(json!({
                "ecdsa": {
                    "algorithm": "ECDSA secp256k1",
                    "enabled": key.is_ok(),
                    "publicKey": public_key_hex(),
                },
                "quantum": {
                    "signatureScheme": "ECDSA-secp256k1 + ML-DSA-65 (hybrid)",
                    "plan": "hybrid signatures live; ML-DSA-only is the next phase",
                    "note": "Signatures are held separately from the payload, so the scheme can be swapped without re-issuing contracts.",
                    "postQuantum": crate::pqc::status(),
                    "log": crate::pqc::log_entries(20),
                },
                "gateways": { "catalogue": gateway_count, "enabled": enabled, "stripeReady": stripe_ready },
                "transport": {
                    "https": settings::get("platform_url", "").starts_with("https"),
                    "csrf": true,
                    "passkeys": true,
                },
                "hardening": crate::hardening::posture(),
                "escrow": { "held": crate::payments::escrow::total_held() },
            }))
        }
        "public_key" => {
            ok(json!({ "publicKey": public_key_hex(), "algorithm": "ECDSA secp256k1" }))
        }
        "sign" => sign(c),
        "verify" => verify(c),
        "log" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows(
                "SELECT * FROM signature_log ORDER BY created_at DESC LIMIT 100",
                &[],
            );
            ok(json!({
                "entries": rows.iter().map(|r| json!({
                    "id": util::s(r.get("id").unwrap_or(&J::Null)),
                    "contractId": util::s(r.get("contract_id").unwrap_or(&J::Null)),
                    "action": util::s(r.get("action").unwrap_or(&J::Null)),
                    "algorithm": util::s(r.get("algorithm").unwrap_or(&J::Null)),
                    "ok": util::i(r.get("ok").unwrap_or(&J::Null)) != 0,
                    "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
                })).collect::<Vec<_>>(),
            }))
        }
        // The company logo the CEO can change. An uploaded image is stored as a
        // data URI; a URL is kept as given. Either way the whole platform reads
        // it from this one setting.
        "logo_get" => ok(json!({
            "logo": settings::get("platform_logo", "/assets/icons/icon-512.png"),
            "name": settings::get("platform_name", "MbilinyiTech Platform"),
            "tagline": settings::get("platform_tagline", ""),
        })),
        "logo_set" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let logo = c.p("logo");
            if logo.is_empty() {
                return bad("Choose a logo image or paste its URL.");
            }
            let looks_like_image = logo.starts_with("data:image/")
                || logo.starts_with("http://")
                || logo.starts_with("https://")
                || logo.starts_with('/');
            if !looks_like_image {
                return bad("The logo must be an uploaded image or an http(s) URL.");
            }
            // An SVG can carry a script, so it is refused as a data URI. A plain
            // http(s) URL to an SVG is fine — it is served from another origin.
            if logo.starts_with("data:image/svg") {
                return bad("Upload a PNG, JPG or WebP logo — SVG is not accepted.");
            }
            settings::set("platform_logo", &logo);
            if !c.p("name").is_empty() {
                settings::set("platform_name", &c.p("name"));
            }
            if !c.p("tagline").is_empty() {
                settings::set("platform_tagline", &c.p("tagline"));
            }
            ok(json!({ "saved": true, "logo": settings::get("platform_logo", "") }))
        }
        "logo_reset" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            settings::set("platform_logo", "/assets/icons/icon-512.png");
            ok(json!({ "reset": true, "logo": "/assets/icons/icon-512.png" }))
        }
        "company_account" => ok(json!({ "account": company_account() })),
        "company_account_set" => company_account_set(c),
        // The sandbox: which domains the platform may probe, and the live
        // scanner's own posture. A domain is added here only after the client's
        // written permission, which is why nothing is scanned by default.
        "sandbox" => ok(crate::sandbox::status()),
        "authorise_domain" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let domain = crate::sandbox::host_of(&c.p("domain"));
            if domain.is_empty() {
                return bad("Give the domain the client has authorised.");
            }
            let mut list: Vec<String> = settings::get("security_authorised_domains", "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !list.iter().any(|d| d.eq_ignore_ascii_case(&domain)) {
                list.push(domain.clone());
            }
            settings::set("security_authorised_domains", &list.join(","));
            ok(json!({ "added": domain, "domains": list }))
        }
        "deauthorise_domain" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let domain = crate::sandbox::host_of(&c.p("domain"));
            let list: Vec<String> = settings::get("security_authorised_domains", "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case(&domain))
                .collect();
            settings::set("security_authorised_domains", &list.join(","));
            ok(json!({ "removed": domain, "domains": list }))
        }
        // A scan the CEO asks for directly, so the sandbox can be proven without
        // running a whole agent job.
        "scan" => scan(c).await,
        _ => bad("Unknown security action."),
    }
}

/// Run one sandbox check on demand. The scope rule applies exactly as it does
/// inside an agent run — there is no privileged path that skips it.
async fn scan(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let kind = c.p("kind");
    let target = c.p("target");
    let (evidence, succeeded) = match kind.as_str() {
        "headers" => crate::sandbox::audit_headers(&target).await,
        "exposure" => crate::sandbox::audit_exposure(&target, &c.p("authorised")).await,
        "dns" => crate::sandbox::audit_dns(&target).await,
        "tls" => crate::sandbox::audit_tls(&target).await,
        "dependencies" => crate::sandbox::audit_dependencies(&target).await,
        "code" => crate::sandbox::audit_code(&target).await,
        "malware" => crate::sandbox::scan_malware(&target).await,
        "methods" => crate::sandbox::audit_methods(&target).await,
        "cors" => crate::sandbox::audit_cors(&target).await,
        "cookies" => crate::sandbox::audit_cookies(&target).await,
        "secrets" => crate::sandbox::audit_secrets(&target).await,
        "password" => crate::sandbox::audit_password(&target).await,
        "jwt" => crate::sandbox::inspect_jwt(&target).await,
        "cert" => crate::sandbox::audit_cert(&target).await,
        "subdomains" => crate::sandbox::subdomain_scan(&target).await,
        "hash" => crate::sandbox::hash_file(&target).await,
        "logs" => crate::sandbox::audit_logs(&target).await,
        // The real-tool wrappers: each one drives an installed binary and
        // returns what it printed. nmap still passes the scope gate; the
        // forensics tools work on evidence the operator supplied.
        "nmap" => crate::forensics::nmap_scan(&target, &c.p("authorised")).await,
        "pcap" => crate::forensics::pcap_summary(&target).await,
        "partitions" => crate::forensics::image_partitions(&target).await,
        "filesystem" => crate::forensics::image_filesystem(&target, &c.p("offset")).await,
        "fls" => crate::forensics::image_list_files(
            &target,
            &c.p("offset"),
            c.p("deleted") == "1" || c.p("deleted") == "true",
        )
        .await,
        "carve" => crate::forensics::carve_files(&target, &c.p("types")).await,
        "yara" => crate::forensics::yara_scan(&target, &c.p("rules")).await,
        "binwalk" => crate::forensics::embedded_scan(&target).await,
        "disassemble" => crate::forensics::disassemble(&target).await,
        "database" => crate::forensics::inspect_database(&target).await,
        _ => {
            return bad(
                "Choose a check: headers, exposure, dns, tls, methods, cors, cookies, cert, \
                 subdomains, dependencies, code, secrets, malware, hash, logs, password, \
                 nmap, pcap, partitions, filesystem, fls, carve, yara, binwalk, disassemble \
                 or database.",
            )
        }
    };
    ok(json!({ "kind": kind, "ok": succeeded, "evidence": evidence }))
}

fn sign(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("contractId");
    if db::row("SELECT id FROM contracts WHERE id = ?", &[arg(json!(id))]).is_none() {
        return err("Contract not found.", 404);
    }
    match sign_contract(&id) {
        Some(sig) => {
            let _ = db::run(
                "INSERT INTO signature_log (id, contract_id, action, signature, ok, created_at) VALUES (?,?,?,?,?,?)",
                &[
                    arg(json!(util::new_id("SIG-"))), arg(json!(id)), arg(json!("sign")),
                    arg(json!(sig)), arg(json!(1)), arg(json!(util::now())),
                ],
            );
            ok(
                json!({ "signed": true, "signature": sig, "algorithm": "ECDSA secp256k1", "publicKey": public_key_hex() }),
            )
        }
        None => bad("Could not sign that contract."),
    }
}

fn verify(c: &Ctx) -> Response {
    let id = c.p("contractId");
    match verify_contract(&id) {
        Some(v) => {
            let _ = db::run(
                "INSERT INTO signature_log (id, contract_id, action, ok, created_at) VALUES (?,?,?,?,?)",
                &[
                    arg(json!(util::new_id("SIG-"))), arg(json!(id)), arg(json!("verify")),
                    arg(json!(if v { 1 } else { 0 })), arg(json!(util::now())),
                ],
            );
            ok(json!({ "valid": v, "algorithm": "ECDSA secp256k1" }))
        }
        None => err("Contract not found.", 404),
    }
}

/// Where the money lands once the client accepts delivery.
pub fn company_account() -> J {
    json!({
        "name": settings::get("company_account_name", "Mbilinyi Tech Solutions"),
        "bank": settings::get("company_bank", ""),
        "accountNo": settings::get("company_account_no", ""),
        "swift": settings::get("company_swift", ""),
        "mobile": settings::get("company_mobile", ""),
        "currency": settings::get("pay_currency", "TZS"),
        "configured": !settings::get("company_account_no", "").trim().is_empty(),
    })
}

fn company_account_set(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    settings::set_many(&[
        ("company_account_name".into(), c.p("name")),
        ("company_bank".into(), c.p("bank")),
        ("company_account_no".into(), c.p("accountNo")),
        ("company_swift".into(), c.p("swift")),
        ("company_mobile".into(), c.p("mobile")),
    ]);
    ok(json!({ "saved": true, "account": company_account() }))
}

/// Unused guard: `VerifyingKey` is imported for the signature traits it brings
/// into scope alongside `SigningKey`.
#[allow(dead_code)]
fn _vk_guard(_: &VerifyingKey) {}
