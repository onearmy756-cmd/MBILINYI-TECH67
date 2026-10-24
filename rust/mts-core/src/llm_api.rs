//! The AI provider endpoints: what the CEO configures to switch the engine on.
//!
//! The key is written to settings and never read back — `status` reports only
//! whether one is present, and its last four characters, so a screenshot of the
//! admin page cannot leak it. `test` makes one real call so the CEO knows the
//! key works before trusting the company to it.

use crate::api::ctx::Ctx;
use crate::llm::{Llm, Msg};
use crate::settings;
use crate::util::{self, bad, ok};
use axum::response::Response;
use serde_json::json;

/// The providers the platform can reach.
const PROVIDERS: &[(&str, &str, &str)] = &[
    ("openai", "OpenAI (GPT)", "gpt-4o-mini"),
    (
        "anthropic",
        "Anthropic (Claude)",
        "claude-3-5-sonnet-latest",
    ),
    ("gemini", "Google Gemini", "gemini-1.5-pro"),
    ("ollama", "Ollama (local or cloud)", "llama3.1"),
];

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "status" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let l = Llm::from_settings();
            let stored = settings::get("llm_api_key", "");
            ok(json!({
                "provider": l.provider,
                "model": l.model,
                "baseUrl": l.base,
                "ready": l.ready(),
                "maxTokens": l.max_tokens,
                "temperature": l.temperature,
                "keySet": !stored.trim().is_empty(),
                "keyHint": hint(&stored),
                "providers": PROVIDERS.iter().map(|(k, n, m)| json!({
                    "key": k, "name": n, "defaultModel": m,
                })).collect::<Vec<_>>(),
            }))
        }
        "save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let provider = c.p("provider").trim().to_lowercase();
            if !PROVIDERS.iter().any(|(k, _, _)| *k == provider) {
                return bad("Choose one of the supported providers.");
            }
            settings::set("llm_provider", &provider);
            if !c.p("model").trim().is_empty() {
                settings::set("llm_model", c.p("model").trim());
            }
            if !c.p("baseUrl").trim().is_empty() {
                settings::set("llm_base_url", c.p("baseUrl").trim());
            }
            // An empty key field means "leave the stored key alone", so saving
            // the model does not wipe the credential.
            let key = c.p("apiKey");
            if !key.trim().is_empty() {
                settings::set("llm_api_key", key.trim());
            }
            if !c.p("maxTokens").trim().is_empty() {
                settings::set("llm_max_tokens", &util::i(&c.pv("maxTokens")).to_string());
            }
            if !c.p("temperature").trim().is_empty() {
                settings::set(
                    "llm_temperature",
                    &format!("{}", util::f(&c.pv("temperature"))),
                );
            }
            let l = Llm::from_settings();
            ok(
                json!({ "saved": true, "provider": l.provider, "model": l.model, "ready": l.ready() }),
            )
        }
        "clear_key" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            settings::set("llm_api_key", "");
            ok(json!({ "cleared": true }))
        }
        // One real call, so the CEO can see the engine answer before relying on it.
        "test" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let l = Llm::from_settings();
            if !l.ready() {
                return bad("Add an API key first, then test the connection.");
            }
            let msgs = vec![
                Msg::System("You are the AI engine of Mbilinyi Tech Solutions. Reply in one short sentence.".into()),
                Msg::User("Say that you are connected and ready, and name the model you are.".into()),
            ];
            let started = std::time::Instant::now();
            match l.chat(&msgs, &[]).await {
                Ok(t) => ok(json!({
                    "connected": true,
                    "reply": t.text.trim(),
                    "provider": l.provider,
                    "model": l.model,
                    "ms": started.elapsed().as_millis() as u64,
                })),
                Err(e) => ok(
                    json!({ "connected": false, "error": e, "provider": l.provider, "model": l.model }),
                ),
            }
        }
        _ => bad("Unknown llm action."),
    }
}

/// The last four characters of a key, so the CEO can tell two keys apart without
/// the page ever holding the whole secret.
fn hint(key: &str) -> String {
    let t = key.trim();
    if t.is_empty() {
        return String::new();
    }
    let tail: String = t
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_never_shows_the_whole_key() {
        let k = "sk-proj-abcdefghijklmnop";
        let h = hint(k);
        assert_eq!(h, "…mnop");
        assert!(!h.contains("sk-proj"));
    }

    #[test]
    fn an_empty_key_has_no_hint() {
        assert_eq!(hint(""), "");
        assert_eq!(hint("   "), "");
    }

    #[test]
    fn a_short_key_is_still_only_hinted() {
        assert_eq!(hint("ab"), "…ab");
    }
}
