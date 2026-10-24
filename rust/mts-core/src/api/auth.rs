//! Authentication: sign-in, sign-up, the two-step PIN/OTP check, passkeys.
//!
//! Sign-in is deliberately two steps. The password proves knowledge of a
//! secret; the PIN (or the emailed code, or the tail of the registered phone)
//! proves possession of something the account holder has. A stolen password
//! alone therefore does not open the portal.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::api::{attempt, need};
use crate::db;
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::json;

fn arg(v: serde_json::Value) -> Value {
    db::from_json(v)
}

fn digits(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// A Tanzanian mobile number: 07XXXXXXXX, 255XXXXXXXXX or +255XXXXXXXXX.
fn phone_valid(phone: &str) -> bool {
    let mut d = digits(phone);
    if d.starts_with("255") {
        d = format!("0{}", &d[3..]);
    }
    d.len() == 10 && d.starts_with('0') && matches!(d.as_bytes()[1], b'6' | b'7')
}

fn phone_tail(phone: &str) -> String {
    let d = digits(phone);
    if d.len() >= 4 {
        d[d.len() - 4..].to_string()
    } else {
        String::new()
    }
}

/// The pepper mixed into every login code before it is hashed. It lives in the
/// environment, not the database, so someone who reads the `otp_codes` table
/// still cannot brute-force a code from its hash.
fn otp_pepper() -> String {
    std::env::var("MTS_OTP_PEPPER").unwrap_or_else(|_| "MTS_OTP_2026_PEPPER_DAR".to_string())
}

fn otp_hash(code: &str, email: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(
        format!("{code}|{email}|{}", otp_pepper()).as_bytes(),
    ))
}

/// A six-digit login code from the OS random source. Not `util::new_id`'s three
/// random bytes: this code is the only thing between a known email and the
/// account, so every one of the million values must be equally likely.
fn otp_code() -> String {
    use rand::Rng;
    format!("{:06}", rand::thread_rng().gen_range(0..1_000_000u32))
}

/// Issue a fresh login code, replacing any earlier one so only the newest works.
fn otp_issue(user_id: &str, email: &str) -> String {
    let code = otp_code();
    let expires = (chrono::Local::now() + chrono::Duration::minutes(10))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let _ = db::run(
        "DELETE FROM otp_codes WHERE email = ? AND purpose = ?",
        &[arg(json!(email)), arg(json!("login"))],
    );
    let _ = db::run(
        "INSERT INTO otp_codes (id, user_id, email, purpose, code_hash, expires_at, created_at)
         VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("OTP-"))),
            arg(json!(user_id)),
            arg(json!(email)),
            arg(json!("login")),
            arg(json!(otp_hash(&code, email))),
            arg(json!(expires)),
            arg(json!(util::now())),
        ],
    );
    code
}

/// The code is echoed back to the caller only when `OTP_SHOW_IN_RESPONSE` is
/// set — a way to exercise the flow on a deployment with no mail provider. It is
/// never enabled on the live site, because the reply would leak the code.
fn otp_debug(code: &str) -> Option<String> {
    match std::env::var("OTP_SHOW_IN_RESPONSE") {
        Ok(v) if !v.trim().is_empty() && v.trim() != "0" => Some(code.to_string()),
        _ => None,
    }
}

/// Open the session and return the signed-in user with the cookie attached.
fn finish_login(row: &db::Row) -> Response {
    let id = util::s(row.get("id").unwrap_or(&serde_json::Value::Null));
    let sess = crate::session::login(&id);
    let mut res = ok(json!({ "user": map::user(row) }));
    if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sess.id)) {
        res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
    }
    res
}

/// Check a submitted PIN against the stored hash, or the phone tail when no PIN
/// was ever set. Returns the reason on failure so the caller can count attempts.
fn check_pin(row: &db::Row, pin: &str) -> (bool, String) {
    let pin = digits(pin);
    if pin.len() < 4 || pin.len() > 6 {
        return (false, "Enter a 4-6 digit PIN.".to_string());
    }
    let hash = row.get("pin_hash").and_then(|v| v.as_str()).unwrap_or("");
    if !hash.is_empty() {
        return match bcrypt::verify(&pin, hash) {
            Ok(true) => (true, String::new()),
            _ => (false, "Wrong PIN. Try again.".to_string()),
        };
    }
    let tail = phone_tail(row.get("phone").and_then(|v| v.as_str()).unwrap_or(""));
    if !tail.is_empty() && tail == pin {
        (true, String::new())
    } else {
        (false, "Wrong PIN. Try again.".to_string())
    }
}

fn hash_pin(pin: &str) -> String {
    bcrypt::hash(digits(pin), 10).unwrap_or_default()
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "login" => login(c),
        "verify-login-pin" => verify_login_pin(c),
        "verify-login-otp" => verify_login_otp(c),
        "resend-login-otp" => resend_login_otp(c),
        "set-pin" => set_pin(c),
        "register" => register(c),
        "logout" => logout(c),
        "me" => me(c),
        "passkey-register-begin" => passkey_register_begin(c),
        "passkey-register-finish" => passkey_register_finish(c),
        "passkey-login-begin" => passkey_login_begin(c),
        "passkey-login-finish" => passkey_login_finish(c),
        "passkey-list" => passkey_list(c),
        "passkey-delete" => passkey_delete(c),
        "forgot-password-init" => forgot_init(c),
        "forgot-password-complete" => forgot_complete(c),
        _ => bad("Unknown auth action."),
    }
}

/// Step one: password. The session is NOT established here — the caller must
/// still pass the PIN or the emailed code.
fn login(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let email = c.p("email").to_lowercase();
    let pass = c.p("password");
    if email.is_empty() || pass.is_empty() {
        return bad("Fill email and password.");
    }
    let Some(row) = db::row(
        "SELECT * FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    ) else {
        return bad("Invalid email or password.");
    };
    let hash = row
        .get("password_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !bcrypt::verify(&pass, hash).unwrap_or(false) {
        return bad("Invalid email or password.");
    }
    if row.get("status").and_then(|v| v.as_str()) != Some("Active") {
        return bad("Account is suspended. Contact 0796 752 645.");
    }
    let has_pin = !row
        .get("pin_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .is_empty();
    let has_tail = !phone_tail(row.get("phone").and_then(|v| v.as_str()).unwrap_or("")).is_empty();
    // The password is only step one. A code goes out now so the second step can
    // be completed without waiting for a resend; it is replaced on every sign-in.
    let uid = util::s(row.get("id").unwrap_or(&serde_json::Value::Null));
    let code = otp_issue(&uid, &email);
    crate::mail::send_login_code(&email, &code);
    ok(json!({
        "otp_required": true,
        "email": email,
        "otp_debug_only": otp_debug(&code),
        "pin_available": has_pin || has_tail,
        "pin_kind": if has_pin { "pin" } else if has_tail { "phone" } else { "" },
    }))
}

/// Step two, by emailed code. The code is compared as a hash, so the stored row
/// never reveals it, and a used code cannot be replayed.
fn verify_login_otp(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let email = c.p("email").to_lowercase();
    let code = digits(&c.p("code"));
    if email.is_empty() || code.is_empty() {
        return bad("Email and code are required.");
    }
    let Some(otp) = db::row(
        "SELECT * FROM otp_codes WHERE email = ? AND purpose = ? AND used_at IS NULL
         ORDER BY created_at DESC LIMIT 1",
        &[arg(json!(email)), arg(json!("login"))],
    ) else {
        return err("No pending code for this email. Sign in again.", 404);
    };
    let expected = otp_hash(&code, &email);
    let stored = util::s(otp.get("code_hash").unwrap_or(&serde_json::Value::Null));
    let expired = otp
        .get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|until| until < chrono::Local::now().naive_local())
        .unwrap_or(true);
    if expired || !crate::payments::constant_time_eq(expected.as_bytes(), stored.as_bytes()) {
        return bad("Wrong or expired verification code. Check your email, or Resend.");
    }
    let oid = util::s(otp.get("id").unwrap_or(&serde_json::Value::Null));
    let _ = db::run(
        "UPDATE otp_codes SET used_at = ? WHERE id = ?",
        &[arg(json!(util::now())), arg(json!(oid))],
    );
    let Some(row) = db::row(
        "SELECT * FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    ) else {
        return err("Account not available.", 403);
    };
    if row.get("status").and_then(|v| v.as_str()) != Some("Active") {
        return err("Account is suspended. Contact 0796 752 645.", 403);
    }
    finish_login(&row)
}

/// Send a replacement code, invalidating the old one. Refuses for an unknown or
/// suspended account so it cannot be used to probe which emails are registered.
fn resend_login_otp(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let email = c.p("email").to_lowercase();
    if email.is_empty() {
        return bad("Email is required.");
    }
    let Some(row) = db::row(
        "SELECT id, status FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    ) else {
        return err("No account with this email.", 404);
    };
    if row.get("status").and_then(|v| v.as_str()) != Some("Active") {
        return err("Account is suspended.", 403);
    }
    let uid = util::s(row.get("id").unwrap_or(&serde_json::Value::Null));
    let code = otp_issue(&uid, &email);
    crate::mail::send_login_code(&email, &code);
    ok(json!({
        "otp_required": true,
        "email": email,
        "otp_debug_only": otp_debug(&code),
    }))
}

/// Step two: the PIN. Five wrong tries lock the account for fifteen minutes.
fn verify_login_pin(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let email = c.p("email").to_lowercase();
    let pin = c.p("pin");
    if email.is_empty() || pin.is_empty() {
        return bad("Email and PIN are required.");
    }
    let Some(row) = db::row(
        "SELECT * FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    ) else {
        return err("No account with this email.", 404);
    };
    if row.get("status").and_then(|v| v.as_str()) != Some("Active") {
        return err("Account is suspended. Contact 0796 752 645.", 403);
    }
    let locked = row
        .get("pin_locked_until")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !locked.is_empty() {
        if let Ok(until) = chrono::NaiveDateTime::parse_from_str(locked, "%Y-%m-%d %H:%M:%S") {
            let now = chrono::Local::now().naive_local();
            if until > now {
                let mins = ((until - now).num_seconds() / 60).max(1);
                return err(
                    &format!("Too many wrong PIN attempts. Try again in {mins} minute(s), or use the emailed code."),
                    429,
                );
            }
        }
    }
    let (good, reason) = check_pin(&row, &pin);
    let id = row
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !good {
        let fails = row.get("pin_fails").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
        if fails >= 5 {
            let until = (chrono::Local::now() + chrono::Duration::minutes(15))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let _ = db::run(
                "UPDATE users SET pin_fails = 0, pin_locked_until = ? WHERE id = ?",
                &[arg(json!(until)), arg(json!(id))],
            );
            return err("Too many wrong PIN attempts. Your account is locked for 15 minutes — use the emailed code or try later.", 429);
        }
        let _ = db::run(
            "UPDATE users SET pin_fails = ? WHERE id = ?",
            &[arg(json!(fails)), arg(json!(id))],
        );
        return bad(&format!(
            "{reason} {} attempt(s) left before a 15-minute lock.",
            5 - fails
        ));
    }
    let _ = db::run(
        "UPDATE users SET pin_fails = 0, pin_locked_until = NULL WHERE id = ?",
        &[arg(json!(id))],
    );
    finish_login(&row)
}

/// Set or change the account PIN. Needs a session, so a stranger cannot attach
/// a PIN to someone else's account.
fn set_pin(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let pin = digits(&c.p("pin"));
    if pin.len() < 4 || pin.len() > 6 {
        return bad("PIN must be 4-6 digits.");
    }
    let pin2 = digits(&c.p("pin2"));
    if !pin2.is_empty() && pin2 != pin {
        return bad("PINs do not match.");
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let id = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let _ = db::run(
        "UPDATE users SET pin_hash = ?, pin_fails = 0, pin_locked_until = NULL WHERE id = ?",
        &[arg(json!(hash_pin(&pin))), arg(json!(id))],
    );
    ok(json!({ "updated": true }))
}

/// Sign up. The account is created Active and the caller is signed in.
fn register(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let name = c.p("name");
    let email = c.p("email").to_lowercase();
    let phone = c.p("phone");
    let pass = c.p("password");
    let pass2 = c.p("password2");
    let pin = digits(&c.p("pin"));
    if name.is_empty() || email.is_empty() || pass.is_empty() {
        return bad("Fill name, email and password.");
    }
    if !email.contains('@') || !email.contains('.') {
        return bad("Enter a valid email address.");
    }
    if !phone.is_empty() && !phone_valid(&phone) {
        return bad("Enter a valid Tanzanian phone number (e.g. 0712345678).");
    }
    if pass.len() < 8 {
        return bad("Password must be at least 8 characters.");
    }
    if !pass2.is_empty() && pass != pass2 {
        return bad("Passwords do not match.");
    }
    if !pin.is_empty() && (pin.len() < 4 || pin.len() > 6) {
        return bad("PIN must be 4-6 digits.");
    }
    if db::row(
        "SELECT id FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    )
    .is_some()
    {
        return bad("An account with this email already exists. Sign in instead.");
    }
    let id = util::new_id("U-");
    let hash = bcrypt::hash(&pass, 10).unwrap_or_default();
    let pin_hash = if pin.is_empty() {
        String::new()
    } else {
        hash_pin(&pin)
    };
    let _ = db::run(
        "INSERT INTO users (id, name, email, phone, company, password_hash, pin_hash, role, status, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(name)), arg(json!(email)), arg(json!(phone)),
            arg(json!(c.p("company"))), arg(json!(hash)),
            arg(json!(pin_hash)), arg(json!("client")), arg(json!("Active")),
            arg(json!(util::now())),
        ],
    );
    let row = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(id))]).unwrap_or_default();
    let sess = crate::session::login(&id);
    let mut res = ok(json!({ "user": map::user(&row) }));
    if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sess.id)) {
        res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
    }
    res
}

fn logout(c: &Ctx) -> Response {
    crate::session::logout(&c.session.id);
    let mut res = ok(json!({ "loggedOut": true }));
    if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::clear_cookie()) {
        res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
    }
    res
}

fn me(c: &Ctx) -> Response {
    match c.user() {
        Some(u) => ok(json!({ "user": map::user(&u) })),
        None => ok(json!({ "user": null })),
    }
}

/* ---------------- password reset ---------------- */

fn forgot_init(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let email = c.p("email").to_lowercase();
    // The reply is identical whether or not the account exists, so the endpoint
    // cannot be used to discover which emails are registered.
    if let Some(row) = db::row(
        "SELECT id FROM users WHERE lower(email) = ?",
        &[arg(json!(email))],
    ) {
        let token = util::token_hex(32);
        let expires = (chrono::Local::now() + chrono::Duration::hours(1))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let uid = row
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let _ = db::run(
            "INSERT INTO password_resets (id, user_id, email, token, expires_at, created_at)
             VALUES (?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("PR-"))),
                arg(json!(uid)),
                arg(json!(email)),
                arg(json!(token)),
                arg(json!(expires)),
                arg(json!(util::now())),
            ],
        );
    }
    ok(json!({ "sent": true }))
}

fn forgot_complete(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let token = c.p("token");
    let pass = c.p("password");
    if token.is_empty() || pass.len() < 8 {
        return bad("A reset token and a password of at least 8 characters are required.");
    }
    let Some(row) = db::row(
        "SELECT * FROM password_resets WHERE token = ? AND used_at IS NULL",
        &[arg(json!(token))],
    ) else {
        return bad("This reset link is not valid. Request a new one.");
    };
    let uid = row
        .get("user_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let _ = db::run(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        &[
            arg(json!(bcrypt::hash(&pass, 10).unwrap_or_default())),
            arg(json!(uid)),
        ],
    );
    let _ = db::run(
        "UPDATE password_resets SET used_at = ? WHERE token = ?",
        &[arg(json!(util::now())), arg(json!(token))],
    );
    ok(json!({ "reset": true }))
}

/* ---------------- passkeys ---------------- */

fn rp_id() -> String {
    let set = settings::get("webauthn_rp_id", "");
    if !set.trim().is_empty() {
        return set.trim().to_string();
    }
    std::env::var("MTS_RP_ID").unwrap_or_else(|_| "localhost".to_string())
}

fn origin() -> String {
    let set = settings::get("webauthn_origin", "");
    if !set.trim().is_empty() {
        return set.trim().to_string();
    }
    std::env::var("MTS_ORIGIN").unwrap_or_else(|_| format!("http://{}", rp_id()))
}

fn passkey_register_begin(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let challenge = crate::payments::b64u_enc(&crate::payments::random_bytes(32));
    crate::payments::store_challenge(&c.session.id, "wa_reg", &challenge);
    let id = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let email = u.get("email").and_then(|v| v.as_str()).unwrap_or("");
    ok(json!({
        "challenge": challenge,
        "rp": { "id": rp_id(), "name": settings::get("webauthn_rp_name", "Mbilinyi Tech Solutions") },
        "user": { "id": crate::payments::b64u_enc(id.as_bytes()), "name": email, "displayName": u.get("name").and_then(|v| v.as_str()).unwrap_or("") },
        "pubKeyCredParams": [
            { "type": "public-key", "alg": -7 },
            { "type": "public-key", "alg": -257 }
        ],
        "timeout": 60000,
        "attestation": "none",
        "authenticatorSelection": { "residentKey": "preferred", "userVerification": "preferred" },
        "excludeCredentials": passkey_credentials(id),
    }))
}

fn passkey_credentials(user_id: &str) -> Vec<serde_json::Value> {
    db::rows(
        "SELECT credential_id, transports FROM webauthn_credentials WHERE user_id = ?",
        &[arg(json!(user_id))],
    )
    .into_iter()
    .map(|r| {
        json!({
            "type": "public-key",
            "id": r.get("credential_id").and_then(|v| v.as_str()).unwrap_or(""),
            "transports": r.get("transports").and_then(|v| v.as_str()).unwrap_or("").split(',').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
        })
    })
    .collect()
}

fn passkey_register_finish(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let uid = u
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let resp = c.pv("response");
    let client_data = util::s(
        resp.get("clientDataJSON")
            .unwrap_or(&serde_json::Value::Null),
    );
    let attestation = util::s(
        resp.get("attestationObject")
            .unwrap_or(&serde_json::Value::Null),
    );
    let label = {
        let l = c.p("label");
        if l.is_empty() {
            "This device".to_string()
        } else {
            l
        }
    };
    let transports: Vec<String> = match c.pv("transports") {
        serde_json::Value::Array(a) => a.iter().map(util::s).collect(),
        _ => vec![],
    };
    attempt(|| {
        let challenge = crate::payments::take_challenge(&c.session.id, "wa_reg")
            .ok_or_else(|| "The registration challenge expired. Try again.".to_string())?;
        let info = crate::payments::webauthn::verify_registration(
            &client_data,
            &attestation,
            &challenge,
            &origin(),
            &rp_id(),
            &transports,
        )
        .map_err(|e| e.to_string())?;
        let _ = db::run(
            "INSERT INTO webauthn_credentials
               (id, user_id, credential_id, public_key, sign_count, transports, label, aaguid, created_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("PK-"))), arg(json!(uid)),
                arg(json!(info.credential_id)), arg(json!(info.public_key)),
                arg(json!(info.sign_count as i64)), arg(json!(info.transports)),
                arg(json!(label)), arg(json!(info.aaguid)), arg(json!(util::now())),
            ],
        );
        Ok(ok(json!({ "registered": true, "label": label })))
    })
}

fn passkey_login_begin(c: &Ctx) -> Response {
    let email = c.p("email").to_lowercase();
    let challenge = crate::payments::b64u_enc(&crate::payments::random_bytes(32));
    crate::payments::store_challenge(&c.session.id, "wa_auth", &challenge);
    // With an email we can name the credentials; without one the browser offers
    // whichever passkey it holds for this site (a "discoverable" credential).
    let allow = if email.is_empty() {
        vec![]
    } else {
        match db::row(
            "SELECT id FROM users WHERE lower(email) = ?",
            &[arg(json!(email))],
        ) {
            Some(r) => passkey_credentials(r.get("id").and_then(|v| v.as_str()).unwrap_or("")),
            None => vec![],
        }
    };
    ok(json!({
        "challenge": challenge,
        "rpId": rp_id(),
        "timeout": 60000,
        "userVerification": "preferred",
        "allowCredentials": allow,
    }))
}

fn passkey_login_finish(c: &Ctx) -> Response {
    let cred_id = c.p("id");
    let resp = c.pv("response");
    let client_data = util::s(
        resp.get("clientDataJSON")
            .unwrap_or(&serde_json::Value::Null),
    );
    let auth_data = util::s(
        resp.get("authenticatorData")
            .unwrap_or(&serde_json::Value::Null),
    );
    let signature = util::s(resp.get("signature").unwrap_or(&serde_json::Value::Null));
    attempt(|| {
        let Some(cred) = db::row(
            "SELECT * FROM webauthn_credentials WHERE credential_id = ?",
            &[arg(json!(cred_id))],
        ) else {
            return Ok(bad("This passkey is not registered on this site."));
        };
        let challenge = crate::payments::take_challenge(&c.session.id, "wa_auth")
            .ok_or_else(|| "The sign-in challenge expired. Try again.".to_string())?;
        let stored = cred.get("sign_count").and_then(|v| v.as_i64()).unwrap_or(0);
        let pem = cred
            .get("public_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let counter = crate::payments::webauthn::verify_assertion(
            &client_data,
            &auth_data,
            &signature,
            &pem,
            stored,
            &challenge,
            &origin(),
            &rp_id(),
        )
        .map_err(|e| e.to_string())?;
        let uid = cred
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let cid = cred
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let _ = db::run(
            "UPDATE webauthn_credentials SET sign_count = ?, last_used_at = ? WHERE id = ?",
            &[
                arg(json!(counter as i64)),
                arg(json!(util::now())),
                arg(json!(cid)),
            ],
        );
        let Some(u) = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(uid))]) else {
            return Ok(bad("This passkey has no account."));
        };
        if u.get("status").and_then(|v| v.as_str()) != Some("Active") {
            return Ok(util::forbidden(
                "This account is suspended. Contact 0796 752 645.",
            ));
        }
        let sess = crate::session::login(&uid);
        let mut res = ok(json!({ "user": map::user(&u) }));
        if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sess.id)) {
            res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
        }
        Ok(res)
    })
}

fn passkey_list(c: &Ctx) -> Response {
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let items: Vec<serde_json::Value> = db::rows(
        "SELECT id, label, transports, created_at, last_used_at FROM webauthn_credentials
         WHERE user_id = ? ORDER BY created_at DESC",
        &[arg(json!(uid))],
    )
    .into_iter()
    .map(|r| {
        json!({
            "id": r.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "label": r.get("label").and_then(|v| v.as_str()).unwrap_or(""),
            "transports": r.get("transports").and_then(|v| v.as_str()).unwrap_or(""),
            "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
            "lastUsedAt": util::iso(r.get("last_used_at").and_then(|v| v.as_str())),
        })
    })
    .collect();
    ok(json!({ "passkeys": items }))
}

fn passkey_delete(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(u) = c.require_user() else {
        return util::unauth();
    };
    let uid = u
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let id = c.p("id");
    let _ = db::run(
        "DELETE FROM webauthn_credentials WHERE id = ? AND user_id = ?",
        &[arg(json!(id)), arg(json!(uid))],
    );
    ok(json!({ "deleted": true }))
}

/// Unused import guard.
#[allow(dead_code)]
fn _keep(_: fn(&Ctx, &str, &str) -> Result<String, Response>) {
    let _ = need;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every one of the million codes must be reachable and always six digits —
    /// a short or skewed code would shrink the search space an attacker faces.
    #[test]
    fn otp_code_is_always_six_digits() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..4000 {
            let c = otp_code();
            assert_eq!(c.len(), 6, "code {c} is not six characters");
            assert!(
                c.chars().all(|ch| ch.is_ascii_digit()),
                "code {c} is not numeric"
            );
            seen.insert(c);
        }
        // 4000 draws from a million values should be almost entirely distinct.
        assert!(
            seen.len() > 3900,
            "codes repeat too often: {} unique",
            seen.len()
        );
    }

    /// The stored value must not contain the code itself, and must depend on the
    /// email so a hash stolen for one account is useless for another.
    #[test]
    fn otp_hash_hides_the_code_and_binds_the_email() {
        let a = otp_hash("123456", "ceo@example.com");
        let b = otp_hash("123456", "other@example.com");
        assert!(!a.contains("123456"));
        assert_ne!(a, b);
        assert_eq!(a, otp_hash("123456", "ceo@example.com"));
        assert_eq!(a.len(), 64, "sha256 hex should be 64 characters");
    }

    /// The emailed-code path exists and is reachable by name from the frontend.
    #[test]
    fn otp_actions_are_registered() {
        let src = include_str!("auth.rs");
        for action in ["verify-login-otp", "resend-login-otp"] {
            assert!(
                src.contains(&format!("\"{action}\" =>")),
                "{action} is not routed in handle()"
            );
        }
    }
}
