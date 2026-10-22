//! Google and GitHub sign-in.
//!
//! Both are switched off until the admin stores a client id and secret, because
//! the redirect URI registered with the provider must match this deployment.
//! The flow is the standard authorization-code exchange; the state value is kept
//! on the session so a callback cannot be forged from another browser.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::settings;
use crate::util::{self, bad, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::json;

fn arg(v: serde_json::Value) -> Value {
    db::from_json(v)
}

struct Provider {
    key: &'static str,
    auth_url: &'static str,
    token_url: &'static str,
    user_url: &'static str,
    scope: &'static str,
    id_setting: &'static str,
    secret_setting: &'static str,
}

const PROVIDERS: &[Provider] = &[
    Provider {
        key: "google",
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
        token_url: "https://oauth2.googleapis.com/token",
        user_url: "https://www.googleapis.com/oauth2/v3/userinfo",
        scope: "openid email profile",
        id_setting: "oauth_google_client_id",
        secret_setting: "oauth_google_client_secret",
    },
    Provider {
        key: "github",
        auth_url: "https://github.com/login/oauth/authorize",
        token_url: "https://github.com/login/oauth/access_token",
        user_url: "https://api.github.com/user",
        scope: "read:user user:email",
        id_setting: "oauth_github_client_id",
        secret_setting: "oauth_github_client_secret",
    },
];

fn find(key: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.key == key)
}

/// Enabled when the admin switched it on AND both credentials are present.
pub fn enabled(p: &Provider) -> bool {
    settings::flag(&format!("oauth_{}_enabled", p.key))
        && !settings::get(p.id_setting, "").trim().is_empty()
        && !settings::get(p.secret_setting, "").trim().is_empty()
}

/// What the sign-in page shows: which buttons to render.
pub fn available() -> Vec<serde_json::Value> {
    PROVIDERS
        .iter()
        .map(|p| {
            json!({
                "key": p.key,
                "name": if p.key == "google" { "Google" } else { "GitHub" },
                "icon": if p.key == "google" { "fa-brands fa-google" } else { "fa-brands fa-github" },
                "enabled": enabled(p),
            })
        })
        .collect()
}

fn redirect_uri(p: &Provider) -> String {
    let base = settings::get("platform_url", "");
    let base = if base.is_empty() {
        std::env::var("MTS_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:12000".to_string())
    } else {
        base
    };
    format!("{}/api/oauth?action=callback&provider={}", base.trim_end_matches('/'), p.key)
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "providers" => ok(json!({ "providers": available() })),
        "start" => start(c),
        "callback" => callback(c).await,
        _ => bad("Unknown oauth action."),
    }
}

/// Hand the browser to the provider.
fn start(c: &Ctx) -> Response {
    let key = c.p("provider");
    let Some(p) = find(&key) else {
        return bad("Unknown provider.");
    };
    if !enabled(p) {
        return bad("That sign-in method is not switched on yet.");
    }
    let state = util::token_hex(16);
    let _ = db::run(
        "INSERT INTO oauth_states (id, provider, created_at) VALUES (?,?,?)",
        &[arg(json!(state)), arg(json!(p.key)), arg(json!(util::now()))],
    );
    let url = format!(
        "{}?client_id={}&redirect_uri={}&scope={}&state={}&response_type=code",
        p.auth_url,
        urlencode(&settings::get(p.id_setting, "")),
        urlencode(&redirect_uri(p)),
        urlencode(p.scope),
        state,
    );
    ok(json!({ "redirect": url }))
}

fn urlencode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Exchange the code, find or create the account, and sign in.
async fn callback(c: &Ctx) -> Response {
    let key = c.p("provider");
    let Some(p) = find(&key) else {
        return bad("Unknown provider.");
    };
    let code = c.p("code");
    let state = c.p("state");
    if code.is_empty() {
        return bad("The provider returned no code.");
    }
    // The state must be one we issued, so a callback from elsewhere is refused.
    let known = db::row("SELECT id FROM oauth_states WHERE id = ? AND provider = ?", &[arg(json!(state)), arg(json!(key))]);
    if known.is_none() {
        return bad("This sign-in attempt has expired. Start again.");
    }
    let _ = db::run("DELETE FROM oauth_states WHERE id = ?", &[arg(json!(state))]);

    let client_id = settings::get(p.id_setting, "");
    let client_secret = settings::get(p.secret_setting, "");
    let client = reqwest::Client::new();
    let token_res = client
        .post(p.token_url)
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri(p).as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await;
    let token: serde_json::Value = match token_res {
        Ok(r) => r.json().await.unwrap_or(json!({})),
        Err(e) => return bad(&format!("Could not reach the provider: {e}")),
    };
    let access = token.get("access_token").and_then(|v| v.as_str()).unwrap_or("");
    if access.is_empty() {
        return bad("The provider did not return an access token.");
    }

    let user_res = client
        .get(p.user_url)
        .header("Authorization", format!("Bearer {access}"))
        .header("User-Agent", "MbilinyiTech")
        .send()
        .await;
    let prof: serde_json::Value = match user_res {
        Ok(r) => r.json().await.unwrap_or(json!({})),
        Err(e) => return bad(&format!("Could not read the profile: {e}")),
    };
    let subject = prof
        .get("sub")
        .or_else(|| prof.get("id"))
        .map(util::s)
        .unwrap_or_default();
    let email = prof.get("email").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let name = prof
        .get("name")
        .or_else(|| prof.get("login"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if subject.is_empty() {
        return bad("The provider returned no account id.");
    }
    if email.is_empty() {
        return bad("The provider returned no email address. Make your email public and try again.");
    }

    // An existing link wins, then an existing account with that email, and only
    // then a new account — so a second provider cannot fork one person in two.
    let user_id = match db::row(
        "SELECT user_id FROM oauth_links WHERE provider = ? AND subject = ?",
        &[arg(json!(key)), arg(json!(subject))],
    ) {
        Some(l) => l.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        None => match db::row("SELECT id FROM users WHERE lower(email) = ?", &[arg(json!(email))]) {
            Some(u) => {
                let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let _ = db::run(
                    "INSERT INTO oauth_links (id, user_id, provider, subject, created_at) VALUES (?,?,?,?,?)",
                    &[
                        arg(json!(util::new_id("OL-"))), arg(json!(uid)), arg(json!(key)),
                        arg(json!(subject)), arg(json!(util::now())),
                    ],
                );
                uid
            }
            None => {
                let uid = util::new_id("U-");
                let _ = db::run(
                    "INSERT INTO users (id, name, email, phone, company, password_hash, role, status, created_at)
                     VALUES (?,?,?,'','',?,'client','Active',?)",
                    &[
                        arg(json!(uid)), arg(json!(if name.is_empty() { email.clone() } else { name })),
                        arg(json!(email)),
                        // No password: this account signs in only through its provider
                        // until the owner sets one.
                        arg(json!("")), arg(json!(util::now())),
                    ],
                );
                let _ = db::run(
                    "INSERT INTO oauth_links (id, user_id, provider, subject, created_at) VALUES (?,?,?,?,?)",
                    &[
                        arg(json!(util::new_id("OL-"))), arg(json!(uid)), arg(json!(key)),
                        arg(json!(subject)), arg(json!(util::now())),
                    ],
                );
                uid
            }
        },
    };

    let Some(u) = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(user_id))]) else {
        return bad("Could not create the account.");
    };
    if u.get("status").and_then(|v| v.as_str()) != Some("Active") {
        return util::forbidden("This account is suspended. Contact 0796 752 645.");
    }
    let sess = crate::session::login(&user_id);
    let mut res = ok(json!({ "user": map::user(&u), "signedIn": true }));
    if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sess.id)) {
        res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
    }
    res
}

/// The tables this module needs.
pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS oauth_states (
           id         VARCHAR(64) NOT NULL,
           provider   VARCHAR(30) NOT NULL,
           created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}
