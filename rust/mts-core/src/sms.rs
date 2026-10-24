//! Real SMS delivery.
//!
//! `ofisi::sms_send` used to write a row and stop there — the message was
//! "queued" and never left the building. This module is the sender: it reads the
//! gateway the CEO configured and actually calls it, then records the true
//! outcome back on the log row.
//!
//! Four gateways are supported, covering Tanzania and the rest of the world:
//!
//!   * **Beem** (Tanzania) — the default for local numbers,
//!   * **Africa's Talking** (pan-African),
//!   * **Twilio** (worldwide),
//!   * **Generic** — any HTTP endpoint the CEO points at, with a token header.
//!
//! With nothing configured the send is refused rather than silently swallowed,
//! so the admin sees "not configured" instead of believing a message went out.

use crate::db;
use crate::settings;
use rusqlite::types::Value;
use serde_json::{json, Value as J};
use std::time::Duration;

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// The gateways, for the admin picker.
pub const GATEWAYS: &[(&str, &str, &str)] = &[
    ("beem", "Beem (Tanzania)", "https://apisms.beem.africa/v1/send"),
    ("africastalking", "Africa's Talking", "https://api.africastalking.com/version1/messaging"),
    ("twilio", "Twilio", ""),
    ("generic", "Generic HTTP endpoint", ""),
];

/// A resolved gateway, ready to send.
struct Gateway {
    name: String,
    api_key: String,
    secret: String,
    sender: String,
    base: String,
}

fn from_settings() -> Gateway {
    let name = {
        let n = settings::get("sms_gateway", "");
        if n.trim().is_empty() {
            // Fall back to the environment, which is how a container is wired.
            std::env::var("SMS_GATEWAY").unwrap_or_default()
        } else {
            n
        }
    }
    .trim()
    .to_lowercase();
    let base = {
        let b = settings::get("sms_base_url", "");
        if !b.trim().is_empty() {
            b.trim().to_string()
        } else {
            std::env::var("SMS_BASE_URL").unwrap_or_else(|_| {
                GATEWAYS.iter().find(|(k, _, _)| *k == name).map(|(_, _, u)| u.to_string()).unwrap_or_default()
            })
        }
    };
    Gateway {
        name,
        api_key: pick("sms_api_key", "SMS_API_KEY"),
        secret: pick("sms_secret", "SMS_SECRET"),
        sender: pick("sms_sender_id", "SMS_SENDER_ID"),
        base: base.trim_end_matches('/').to_string(),
    }
}

fn pick(setting: &str, env: &str) -> String {
    let s = settings::get(setting, "");
    if !s.trim().is_empty() {
        return s.trim().to_string();
    }
    std::env::var(env).unwrap_or_default()
}

impl Gateway {
    fn ready(&self) -> bool {
        !self.name.is_empty() && !self.base.is_empty() && !self.api_key.is_empty()
    }
}

/// Normalise a Tanzanian number to the international form the gateways want.
/// `0712 345 678` becomes `255712345678`; an already-international number is
/// left alone.
pub fn normalise_phone(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return String::new();
    }
    if digits.starts_with("255") {
        return digits;
    }
    if let Some(rest) = digits.strip_prefix('0') {
        return format!("255{rest}");
    }
    if digits.len() == 9 {
        return format!("255{digits}");
    }
    digits
}

/// The result of one send attempt.
pub struct SendResult {
    pub ok: bool,
    pub detail: String,
}

/// Send one message and record the outcome. Returns the gateway's answer.
pub async fn send(phone: &str, message: &str) -> SendResult {
    let g = from_settings();
    if !g.ready() {
        return SendResult {
            ok: false,
            detail: "No SMS gateway is configured. The CEO adds one in Admin -> Settings.".into(),
        };
    }
    let to = normalise_phone(phone);
    if to.is_empty() {
        return SendResult { ok: false, detail: "That phone number has no digits.".into() };
    }
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(30)).build() {
        Ok(c) => c,
        Err(e) => return SendResult { ok: false, detail: format!("cannot build the HTTP client: {e}") },
    };

    let outcome = match g.name.as_str() {
        "beem" => send_beem(&client, &g, &to, message).await,
        "africastalking" => send_at(&client, &g, &to, message).await,
        "twilio" => send_twilio(&client, &g, &to, message).await,
        _ => send_generic(&client, &g, &to, message).await,
    };
    SendResult {
        ok: outcome.is_ok(),
        detail: outcome.unwrap_or_else(|e| e),
    }
}

type Sent = Result<String, String>;

/// Read a gateway reply, treating a 2xx as success and surfacing the body on a
/// failure so the admin can see what the provider said.
async fn finish(res: reqwest::Response, name: &str) -> Sent {
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    let short: String = body.chars().take(300).collect();
    if status.is_success() {
        Ok(format!("{name} accepted the message (HTTP {}). {short}", status.as_u16()))
    } else {
        Err(format!("{name} refused it (HTTP {}): {short}", status.as_u16()))
    }
}

/// Beem: basic-auth with the key and secret, JSON body.
async fn send_beem(client: &reqwest::Client, g: &Gateway, to: &str, msg: &str) -> Sent {
    let body = json!({
        "source_addr": if g.sender.is_empty() { "MBILINYI".to_string() } else { g.sender.clone() },
        "encoding": 0,
        "message": msg,
        "recipients": [{ "recipient_id": "1", "dest_addr": to }],
    });
    let res = client
        .post(format!("{}/v1/send", g.base))
        .basic_auth(&g.api_key, Some(&g.secret))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Beem could not be reached: {e}"))?;
    finish(res, "Beem").await
}

/// Africa's Talking: form body, `apiKey` header.
async fn send_at(client: &reqwest::Client, g: &Gateway, to: &str, msg: &str) -> Sent {
    let res = client
        .post(format!("{}/version1/messaging", g.base))
        .header("apiKey", &g.api_key)
        .header("Accept", "application/json")
        .form(&[
            ("username", g.sender.clone()),
            ("to", format!("+{to}")),
            ("message", msg.to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("Africa's Talking could not be reached: {e}"))?;
    finish(res, "Africa's Talking").await
}

/// Twilio: account SID as the user, the auth token as the password.
async fn send_twilio(client: &reqwest::Client, g: &Gateway, to: &str, msg: &str) -> Sent {
    let sid = &g.api_key;
    let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json");
    let from = if g.sender.is_empty() { "MBILINYI" } else { &g.sender };
    let res = client
        .post(url)
        .basic_auth(sid, Some(&g.secret))
        .form(&[
            ("To", format!("+{to}")),
            ("From", from.to_string()),
            ("Body", msg.to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("Twilio could not be reached: {e}"))?;
    finish(res, "Twilio").await
}

/// Anything else: a bearer-authenticated JSON POST the CEO defined.
async fn send_generic(client: &reqwest::Client, g: &Gateway, to: &str, msg: &str) -> Sent {
    let body = json!({
        "to": to, "message": msg,
        "sender": if g.sender.is_empty() { "MBILINYI".to_string() } else { g.sender.clone() },
    });
    let res = client
        .post(&g.base)
        .bearer_auth(&g.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("the SMS endpoint could not be reached: {e}"))?;
    finish(res, "The gateway").await
}

/// Send and write the true status back to the log row. Called from the
/// background so the HTTP handler does not wait on the provider.
pub async fn send_and_record(log_id: &str, phone: &str, message: &str) {
    let result = send(phone, message).await;
    let status = if result.ok { "sent" } else { "failed" };
    let _ = db::run(
        "UPDATE sms_log SET status = ? WHERE id = ?",
        &[arg(json!(format!("{status}: {}", result.detail))), arg(json!(log_id))],
    );
}

/// Is a gateway configured? The office UI shows this so the CEO knows whether
/// messages will actually leave the platform.
pub fn configured() -> bool {
    from_settings().ready()
}

/// Send in the background, so the HTTP request returns at once and the provider
/// is never waited on inside a handler.
pub fn spawn_send(log_id: String, phone: String, message: String) {
    tokio::spawn(async move {
        send_and_record(&log_id, &phone, &message).await;
    });
}

/// A short description of the active gateway, for the admin panel.
pub fn status() -> J {
    let g = from_settings();
    json!({
        "configured": g.ready(),
        "gateway": g.name,
        "baseUrl": g.base,
        "senderId": g.sender,
        "keySet": !g.api_key.is_empty(),
        "gateways": GATEWAYS.iter().map(|(k, n, u)| json!({ "key": k, "name": n, "url": u })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_local_number_becomes_international() {
        assert_eq!(normalise_phone("0712345678"), "255712345678");
        assert_eq!(normalise_phone("0712 345 678"), "255712345678");
        assert_eq!(normalise_phone("+255 712 345 678"), "255712345678");
        assert_eq!(normalise_phone("712345678"), "255712345678");
        assert_eq!(normalise_phone("255712345678"), "255712345678");
    }

    #[test]
    fn a_foreign_number_is_left_as_it_is() {
        // A UK number already carries its country code.
        assert_eq!(normalise_phone("+44 7911 123456"), "447911123456");
    }

    #[test]
    fn a_number_with_no_digits_is_empty() {
        assert_eq!(normalise_phone("no digits here"), "");
    }

    #[test]
    fn the_gateway_list_covers_tanzania_and_the_world() {
        let keys: Vec<&str> = GATEWAYS.iter().map(|(k, _, _)| *k).collect();
        assert!(keys.contains(&"beem"));
        assert!(keys.contains(&"twilio"));
    }
}
