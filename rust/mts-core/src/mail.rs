//! Real email delivery over SMTP.
//!
//! The platform sends four kinds of mail: the sign-in code, a contract ready to
//! sign, a payment confirmation and a password reset. All four go through
//! `send`, so there is one place where the credentials are read and one place
//! where the true outcome is recorded.
//!
//! With nothing configured a send is refused and says so, rather than reporting
//! success for a message that never left the machine. Every attempt is written
//! to `email_log`, which is what the admin audits — the same rule SMS follows.

use crate::db;
use crate::settings;
use lettre::message::{header::ContentType, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Tokio1Executor;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS email_log (
           id         VARCHAR(64)  NOT NULL,
           recipient  VARCHAR(190) NOT NULL DEFAULT '',
           subject    VARCHAR(255) NOT NULL DEFAULT '',
           body       TEXT         NOT NULL DEFAULT '',
           kind       VARCHAR(40)  NOT NULL DEFAULT '',
           status     VARCHAR(60)  NOT NULL DEFAULT 'queued',
           created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}

struct Smtp {
    host: String,
    port: u16,
    user: String,
    pass: String,
    from: String,
    from_name: String,
    starttls: bool,
}

fn pick(setting: &str, env: &str) -> String {
    let s = settings::get(setting, "");
    if !s.trim().is_empty() {
        return s.trim().to_string();
    }
    std::env::var(env).unwrap_or_default()
}

fn from_settings() -> Smtp {
    let host = pick("smtp_host", "SMTP_HOST");
    let port = {
        let p = pick("smtp_port", "SMTP_PORT");
        p.parse::<u16>().unwrap_or(587)
    };
    let from = {
        let f = pick("smtp_from", "SMTP_FROM");
        if f.is_empty() {
            pick("smtp_user", "SMTP_USER")
        } else {
            f
        }
    };
    Smtp {
        host,
        port,
        user: pick("smtp_user", "SMTP_USER"),
        pass: pick("smtp_pass", "SMTP_PASS"),
        from,
        from_name: {
            let n = pick("smtp_sender_name", "SMTP_SENDER_NAME");
            if n.is_empty() {
                "Mbilinyi Tech Solutions".to_string()
            } else {
                n
            }
        },
        // Port 465 is implicit TLS; 587/25 use STARTTLS.
        starttls: port != 465,
    }
}

impl Smtp {
    fn ready(&self) -> bool {
        !self.host.is_empty() && !self.from.is_empty()
    }
}

/// What the admin panel shows, and what a test send reports.
pub fn status() -> J {
    let s = from_settings();
    json!({
        "configured": s.ready(),
        "host": s.host,
        "port": s.port,
        "user": s.user,
        "from": s.from,
        "fromName": s.from_name,
        "secure": !s.starttls,
        "passSet": !s.pass.is_empty(),
    })
}

/// A usable recipient: something before an `@`, and a dot after it.
fn is_address(s: &str) -> bool {
    let s = s.trim();
    match s.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty() && domain.contains('.') && !domain.starts_with('.')
        }
        None => false,
    }
}

pub struct Sent {
    pub ok: bool,
    pub detail: String,
}

/// Send one message. The outcome is the provider's answer, never an assumption.
pub async fn send(to: &str, subject: &str, body: &str) -> Sent {
    let s = from_settings();
    if !s.ready() {
        return Sent {
            ok: false,
            detail: "No SMTP server is configured. The CEO adds one in Admin -> Email Setup."
                .into(),
        };
    }
    if !is_address(to) {
        return Sent {
            ok: false,
            detail: "That recipient address is not valid.".into(),
        };
    }
    let from_mailbox = match s.from.parse() {
        Ok(m) => m,
        Err(_) => {
            return Sent {
                ok: false,
                detail: format!("The From address {} is not valid.", s.from),
            }
        }
    };
    let email = match Message::builder()
        .from(from_mailbox)
        .to(match to.parse() {
            Ok(m) => m,
            Err(_) => return Sent { ok: false, detail: "The recipient address is not valid.".into() },
        })
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "<div style=\"font-family:Inter,Arial,sans-serif;background:#050914;color:#E6ECF5;padding:28px;border-radius:16px\">\
             <div style=\"font-weight:800;font-size:18px;color:#22d3ee\">{}</div>\
             <div style=\"margin-top:14px;font-size:15px;line-height:1.6\">{}</div>\
             <div style=\"margin-top:22px;font-size:12px;color:#8EA0BF\">Mbilinyi Tech Solutions — BRELA Reg. • TIN 192-147-522 • Dodoma, Tanzania</div>\
             </div>",
            crate::util::h(&s.from_name),
            body.replace('\n', "<br>"),
        ))
    {
        Ok(m) => m,
        Err(e) => return Sent { ok: false, detail: format!("Could not build the message: {e}") },
    };

    let mut builder = if s.starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&s.host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&s.host)
    }
    .unwrap_or_else(|_| AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&s.host))
    .port(s.port);
    if !s.user.is_empty() {
        builder = builder.credentials(Credentials::new(s.user.clone(), s.pass.clone()));
    }
    match builder.build().send(email).await {
        Ok(_) => Sent {
            ok: true,
            detail: "The mail server accepted the message.".into(),
        },
        Err(e) => Sent {
            ok: false,
            detail: scrub(&format!("{e}")),
        },
    }
}

/// Never let a password reach a log or a screen. Anything that looks like a
/// credential is replaced before the text is stored or shown.
fn scrub(msg: &str) -> String {
    let mut out = msg.to_string();
    for key in ["smtp_pass", "SMTP_PASS", "password", "pass="] {
        if let Some(pos) = out.find(key) {
            let end = (pos + key.len() + 40).min(out.len());
            out.replace_range(pos..end, &format!("{key}…(hidden)"));
        }
    }
    out.chars().take(300).collect()
}

/// Send and record. Every attempt lands in `email_log`, success or failure, so
/// the admin can see what the platform actually sent.
pub async fn send_logged(to: &str, subject: &str, body: &str, kind: &str) -> Sent {
    let id = crate::util::new_id("MAIL-");
    let _ = db::run(
        "INSERT INTO email_log (id, recipient, subject, body, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(to)), arg(json!(subject)), arg(json!(body)),
            arg(json!(kind)), arg(json!("queued")), arg(json!(crate::util::now())),
        ],
    );
    let r = send(to, subject, body).await;
    let status = if r.ok {
        format!("sent: {}", r.detail)
    } else {
        format!("failed: {}", r.detail)
    };
    let _ = db::run(
        "UPDATE email_log SET status = ? WHERE id = ?",
        &[arg(json!(status)), arg(json!(id))],
    );
    r
}

/// Send without making the caller wait on the mail server. The log row is
/// written first, so a slow provider never delays an HTTP response.
pub fn spawn(to: String, subject: String, body: String, kind: &str) {
    let kind = kind.to_string();
    tokio::spawn(async move {
        let _ = send_logged(&to, &subject, &body, &kind).await;
    });
}

/// The sign-in code mail. Sent in the background from the sign-in flow.
pub fn send_login_code(to: &str, code: &str) {
    spawn(
        to.to_string(),
        "Your sign-in code — Mbilinyi Tech Solutions".to_string(),
        format!(
            "Your one-time sign-in code is <b style=\"font-size:22px;letter-spacing:4px;color:#22d3ee\">{}</b>.<br>It expires in 10 minutes. If you did not ask for it, ignore this message.",
            crate::util::h(code)
        ),
        "login_code",
    );
}

pub fn send_contract_ready(to: &str, contract_id: &str, title: &str) {
    spawn(
        to.to_string(),
        format!("Contract {contract_id} is ready to sign"),
        format!(
            "Contract <b>{}</b> for <b>{}</b> is ready. Sign in to your portal to review and sign it.",
            crate::util::h(contract_id),
            crate::util::h(title)
        ),
        "contract",
    );
}

pub fn send_payment_received(to: &str, reference: &str, amount: &str) {
    spawn(
        to.to_string(),
        "Payment received — Mbilinyi Tech Solutions".to_string(),
        format!(
            "We received your payment of <b>{}</b> (reference {}). It is held in escrow until you accept the delivery.",
            crate::util::h(amount),
            crate::util::h(reference)
        ),
        "payment",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_never_survives_the_scrubber() {
        let leaked = "error: authentication failed for SMTP_PASS=hunter2secretvalue";
        let clean = scrub(leaked);
        assert!(!clean.contains("hunter2secretvalue"));
    }

    #[test]
    fn the_scrubber_caps_what_it_stores() {
        let long = "x".repeat(2000);
        assert!(scrub(&long).chars().count() <= 300);
    }

    #[test]
    fn an_address_without_an_at_sign_is_not_a_recipient() {
        assert!(!is_address("not-an-address"));
        assert!(is_address("client@example.co.tz"));
    }
}
