//! Admin-only endpoints: settings (prices, payment accounts, gateway keys,
//! OAuth) and account management.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::payments::CATALOGUE;
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/* ---------------------------- settings ---------------------------- */

pub async fn settings_handler(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => ok(json!({ "settings": settings::as_json() })),
        "save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = c.p("key");
            if key.is_empty() {
                return bad("No setting key given.");
            }
            settings::set(&key, &c.p("value"));
            ok(json!({ "saved": true, "key": key }))
        }
        "save_all" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let pairs: Vec<(String, String)> = match c.pv("settings") {
                J::Object(m) => m
                    .into_iter()
                    .map(|(k, v)| (k, util::s(&v)))
                    .filter(|(k, _)| !k.is_empty())
                    .collect(),
                _ => return bad("No settings sent."),
            };
            let n = pairs.len();
            settings::set_many(&pairs);
            ok(json!({ "saved": n }))
        }
        /* The payment-gateway screen. `gateways` carries the catalogue with each
         * gateway's live enabled/configured state; `fields` lists every setting
         * the admin must fill, so the form is generated rather than hard-coded. */
        "gateways" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let mut secrets: Vec<J> = vec![];
            for g in CATALOGUE {
                for k in g.required {
                    let set = !settings::get(k, "").trim().is_empty();
                    secrets.push(json!({
                        "key": k,
                        "gateway": g.key,
                        "value": if set && !k.contains("secret") { settings::get(k, "") } else { String::new() },
                        "isSet": set,
                        "secret": k.contains("secret"),
                    }));
                }
            }
            ok(json!({
                "gateways": crate::payments::available(),
                "fields": secrets,
                "oauth": crate::api::oauth::available(),
                "currency": settings::get("pay_currency", "TZS"),
            }))
        }
        "gateway_toggle" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = c.p("gateway");
            let Some(def) = crate::payments::find(&key) else {
                return bad("Unknown gateway.");
            };
            let want = if c.p("enabled").is_empty() { "1".to_string() } else { c.p("enabled") };
            // A gateway cannot be switched on until every credential is present,
            // because without them the funds have nowhere to settle.
            if want == "1" {
                let missing: Vec<&str> = def
                    .required
                    .iter()
                    .copied()
                    .filter(|k| settings::get(k, "").trim().is_empty())
                    .collect();
                if !missing.is_empty() {
                    return bad(&format!(
                        "Fill in these before switching {} on: {}",
                        def.name,
                        missing.join(", ")
                    ));
                }
            }
            settings::set(&format!("gw_{}_enabled", key), &want);
            ok(json!({ "gateway": key, "enabled": want == "1" }))
        }
        "oauth_toggle" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let key = c.p("provider");
            if !matches!(key.as_str(), "google" | "github") {
                return bad("Unknown provider.");
            }
            let want = if c.p("enabled").is_empty() { "1".to_string() } else { c.p("enabled") };
            if want == "1" {
                for s in [format!("oauth_{key}_client_id"), format!("oauth_{key}_client_secret")] {
                    if settings::get(&s, "").trim().is_empty() {
                        return bad(&format!("Add the {key} client id and secret first."));
                    }
                }
            }
            settings::set(&format!("oauth_{key}_enabled"), &want);
            ok(json!({ "provider": key, "enabled": want == "1" }))
        }
        "plans" => {
            let rows = db::rows(
                "SELECT * FROM pricing_plans WHERE is_active = 1 ORDER BY group_name, sort_order",
                &[],
            );
            ok(json!({ "plans": rows.iter().map(map::plan).collect::<Vec<_>>() }))
        }
        "plan_save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let code = c.p("code");
            let price = c.p("price");
            settings::set(&format!("price_{code}"), &price);
            // Keep the pricing_plans row in step so the plans list agrees.
            let _ = db::run(
                "UPDATE pricing_plans SET price = ?, updated_at = ? WHERE code = ?",
                &[arg(json!(price)), arg(json!(util::now())), arg(json!(code))],
            );
            ok(json!({ "saved": true }))
        }
        "plan_toggle" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let code = c.p("code");
            let mut hidden = settings::service_hidden();
            if let Some(pos) = hidden.iter().position(|h| h == &code) {
                hidden.remove(pos);
            } else {
                hidden.push(code.clone());
            }
            settings::set("services_hidden", &hidden.join(","));
            ok(json!({ "hidden": hidden }))
        }
        "feedback" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let name = c.p("name");
            let message = c.p("message");
            if message.is_empty() {
                return bad("Write your feedback first.");
            }
            let id = util::new_id("FB-");
            let _ = db::run(
                "INSERT INTO feedback (id, name, email, category, subject, message, rating, status, created_at)
                 VALUES (?,?,?,?,?,?,?,?,?)",
                &[
                    arg(json!(id)), arg(json!(name)), arg(json!(c.p("email"))),
                    arg(json!(if c.p("category").is_empty() { "general".into() } else { c.p("category") })),
                    arg(json!(c.p("subject"))), arg(json!(message)),
                    arg(json!(c.p("rating").parse::<i64>().unwrap_or(0))),
                    arg(json!("New")), arg(json!(util::now())),
                ],
            );
            ok(json!({ "submitted": true, "id": id }))
        }
        "feedback_list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM feedback ORDER BY created_at DESC", &[]);
            ok(json!({ "feedback": rows.iter().map(map::feedback).collect::<Vec<_>>() }))
        }
        "feedback_status" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let _ = db::run(
                "UPDATE feedback SET status = ? WHERE id = ?",
                &[arg(json!(c.p("status"))), arg(json!(c.p("id")))],
            );
            ok(json!({ "saved": true }))
        }
        _ => bad("Unknown settings action."),
    }
}

/// The router calls this; the name avoids clashing with the `settings` module.
pub async fn settings(c: &Ctx) -> Response {
    settings_handler(c).await
}

/* ---------------------------- users ---------------------------- */

pub async fn users(c: &Ctx) -> Response {
    match c.action().as_str() {
        "list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM users ORDER BY created_at DESC", &[]);
            ok(json!({ "users": rows.iter().map(map::user).collect::<Vec<_>>() }))
        }
        "save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let name = c.p("name");
            let email = c.p("email").to_lowercase();
            if name.is_empty() || email.is_empty() {
                return bad("Name and email are required.");
            }
            if id.is_empty() || id == "new" {
                let newid = util::new_id("U-");
                let pass = {
                    let p = c.p("password");
                    if p.is_empty() { util::token_hex(8) } else { p }
                };
                let _ = db::run(
                    "INSERT INTO users (id, name, email, phone, company, password_hash, role, status, created_at)
                     VALUES (?,?,?,?,?,?,?,?,?)",
                    &[
                        arg(json!(newid)), arg(json!(name)), arg(json!(email)),
                        arg(json!(c.p("phone"))), arg(json!(c.p("company"))),
                        arg(json!(bcrypt::hash(&pass, 10).unwrap_or_default())),
                        arg(json!(if c.p("role").is_empty() { "client".into() } else { c.p("role") })),
                        arg(json!(if c.p("status").is_empty() { "Active".into() } else { c.p("status") })),
                        arg(json!(util::now())),
                    ],
                );
                ok(json!({ "id": newid, "saved": true }))
            } else {
                let _ = db::run(
                    "UPDATE users SET name=?, email=?, phone=?, company=?, role=?, status=? WHERE id=?",
                    &[
                        arg(json!(name)), arg(json!(email)), arg(json!(c.p("phone"))),
                        arg(json!(c.p("company"))),
                        arg(json!(if c.p("role").is_empty() { "client".into() } else { c.p("role") })),
                        arg(json!(if c.p("status").is_empty() { "Active".into() } else { c.p("status") })),
                        arg(json!(id)),
                    ],
                );
                let pass = c.p("password");
                if !pass.is_empty() {
                    let _ = db::run(
                        "UPDATE users SET password_hash = ? WHERE id = ?",
                        &[arg(json!(bcrypt::hash(&pass, 10).unwrap_or_default())), arg(json!(id))],
                    );
                }
                ok(json!({ "id": id, "saved": true }))
            }
        }
        "delete" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            // The CEO's own account is never removed, and neither is the last
            // admin — otherwise the site could be locked out of its own console.
            if id == "U-DEMO" {
                return bad("The owner account cannot be deleted.");
            }
            let admins = db::scalar_i64("SELECT count(*) FROM users WHERE role = 'admin' AND id <> ?", &[arg(json!(id))]);
            let target_is_admin = db::row("SELECT role FROM users WHERE id = ?", &[arg(json!(id))])
                .and_then(|r| r.get("role").and_then(|v| v.as_str()).map(str::to_string))
                .unwrap_or_default();
            if target_is_admin == "admin" && admins == 0 {
                return bad("This is the only admin account and cannot be deleted.");
            }
            let _ = db::run("DELETE FROM users WHERE id = ?", &[arg(json!(id))]);
            ok(json!({ "deleted": true }))
        }
        "toggle_status" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let cur = db::row("SELECT status FROM users WHERE id = ?", &[arg(json!(id))])
                .and_then(|r| r.get("status").and_then(|v| v.as_str()).map(str::to_string))
                .unwrap_or_else(|| "Active".to_string());
            let next = if cur == "Active" { "Suspended" } else { "Active" };
            let _ = db::run("UPDATE users SET status = ? WHERE id = ?", &[arg(json!(next)), arg(json!(id))]);
            ok(json!({ "status": next }))
        }
        "toggle_role" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            let Ok(a) = c.require_admin() else {
                return util::forbidden("Admin access required.");
            };
            let id = c.p("id");
            let self_id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id == self_id {
                return bad("You cannot change your own role.");
            }
            let cur = db::row("SELECT role FROM users WHERE id = ?", &[arg(json!(id))])
                .and_then(|r| r.get("role").and_then(|v| v.as_str()).map(str::to_string))
                .unwrap_or_else(|| "client".to_string());
            let next = if cur == "admin" { "client" } else { "admin" };
            let _ = db::run("UPDATE users SET role = ? WHERE id = ?", &[arg(json!(next)), arg(json!(id))]);
            ok(json!({ "role": next }))
        }
        "impersonate" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let id = c.p("id");
            let Some(u) = db::row("SELECT * FROM users WHERE id = ?", &[arg(json!(id))]) else {
                return err("User not found.", 404);
            };
            let sess = crate::session::login(&id);
            let mut res = ok(json!({ "user": map::user(&u), "impersonating": true }));
            if let Ok(v) = axum::http::HeaderValue::from_str(&crate::session::set_cookie(&sess.id)) {
                res.headers_mut().insert(axum::http::header::SET_COOKIE, v);
            }
            res
        }
        _ => bad("Unknown users action."),
    }
}
