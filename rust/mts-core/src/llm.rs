//! The model layer every agent and tool calls.
//!
//! Four providers are supported, chosen from the admin settings: OpenAI,
//! Anthropic, Google Gemini and a local Ollama. Each is reached over HTTPS with
//! `reqwest`; the reply is normalised into one shape so the engine above does
//! not care which provider answered.
//!
//! Nothing here holds the database guard across an `await`. The caller reads
//! what it needs, drops the guard, awaits the model, then writes the result —
//! the rule the whole crate follows.

use crate::settings;
use serde_json::{json, Value as J};
use std::time::Duration;

/// One tool the model may call, in the shape the engine uses.
#[derive(Clone)]
pub struct Spec {
    pub name: String,
    pub description: String,
    pub schema: J,
}

/// A tool call the model asked for.
#[derive(Clone)]
pub struct Call {
    pub id: String,
    pub name: String,
    pub args: J,
}

/// A turn of the conversation, as the engine keeps it.
#[derive(Clone)]
pub enum Msg {
    System(String),
    User(String),
    Assistant { text: String, calls: Vec<Call> },
    ToolResult { id: String, name: String, content: String },
}

/// What the model replied with: prose, tool calls, or both.
pub struct Turn {
    pub text: String,
    pub calls: Vec<Call>,
}

/// The resolved provider configuration.
pub struct Llm {
    pub provider: String,
    pub model: String,
    pub base: String,
    pub key: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

fn env_first(keys: &[&str]) -> String {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            if !v.trim().is_empty() {
                return v.trim().to_string();
            }
        }
    }
    String::new()
}

/// The API key: the admin setting wins, then the matching environment variable,
/// so a deployment can inject a key without writing it into the database.
fn key_for(provider: &str) -> String {
    let from_settings = settings::get("llm_api_key", "");
    if !from_settings.trim().is_empty() {
        return from_settings.trim().to_string();
    }
    match provider {
        "openai" => env_first(&["OPENAI_API_KEY", "OPENAI_KEY"]),
        "anthropic" => env_first(&["ANTHROPIC_API_KEY"]),
        "gemini" => env_first(&["GEMINI_API_KEY", "GOOGLE_API_KEY"]),
        "ollama" => env_first(&["OLLAMA_CLOUD_API_KEY", "OLLAMA_API_KEY"]),
        _ => String::new(),
    }
}

fn default_model(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "claude-3-5-sonnet-latest",
        "gemini" => "gemini-1.5-pro",
        "ollama" => "llama3.1",
        _ => "gpt-4o-mini",
    }
}

fn default_base(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "https://api.anthropic.com",
        "gemini" => "https://generativelanguage.googleapis.com",
        "ollama" => "http://127.0.0.1:11434",
        _ => "https://api.openai.com/v1",
    }
}

/// Read the provider from settings, falling back to the environment. A key set
/// in the environment alone is enough to switch a provider on, which is what a
/// container deployment wants.
impl Llm {
    pub fn from_settings() -> Llm {
    let mut provider = settings::get("llm_provider", "").trim().to_lowercase();
    if provider.is_empty() {
        provider = if !env_first(&["ANTHROPIC_API_KEY"]).is_empty() {
            "anthropic".into()
        } else if !env_first(&["GEMINI_API_KEY", "GOOGLE_API_KEY"]).is_empty() {
            "gemini".into()
        } else if !env_first(&["OLLAMA_CLOUD_API_KEY", "OLLAMA_API_KEY"]).is_empty() {
            "ollama".into()
        } else {
            "openai".into()
        };
    }
    let model = {
        let m = settings::get("llm_model", "");
        if m.trim().is_empty() {
            env_first(&["LLM_MODEL"]).if_empty(default_model(&provider))
        } else {
            m.trim().to_string()
        }
    };
    let base = {
        let b = settings::get("llm_base_url", "");
        if b.trim().is_empty() {
            env_first(&["LLM_BASE_URL", "OLLAMA_HOST"]).if_empty(default_base(&provider))
        } else {
            b.trim().to_string()
        }
    };
    Llm {
        provider: provider.clone(),
        model,
        base: base.trim_end_matches('/').to_string(),
        key: key_for(&provider),
        max_tokens: settings::num("llm_max_tokens", 4096.0) as u32,
        temperature: settings::num("llm_temperature", 0.3),
    }
    }
}

/// Small helper so the nested `if_empty` above reads as one expression.
trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}
impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

impl Llm {
    /// A provider is usable when it has a key — except Ollama, which may be a
    /// local server with no authentication at all.
    pub fn ready(&self) -> bool {
        !self.key.is_empty() || self.provider == "ollama"
    }

    fn client() -> Result<reqwest::Client, String> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| format!("cannot build the HTTP client: {e}"))
    }

    /// One round trip. `msgs` is the whole conversation so far and `specs` the
    /// tools on offer; the reply is normalised into `Turn`.
    pub async fn chat(&self, msgs: &[Msg], specs: &[Spec]) -> Result<Turn, String> {
        if !self.ready() {
            return Err(format!(
                "No model is configured. Add a {} API key in Admin -> Settings.",
                self.provider
            ));
        }
        match self.provider.as_str() {
            "anthropic" => self.chat_anthropic(msgs, specs).await,
            "gemini" => self.chat_gemini(msgs, specs).await,
            "ollama" => self.chat_ollama(msgs, specs).await,
            _ => self.chat_openai(msgs, specs).await,
        }
    }

    /* ---- OpenAI (and anything that speaks the same shape) ---- */

    fn openai_messages(&self, msgs: &[Msg]) -> Vec<J> {
        msgs.iter()
            .map(|m| match m {
                Msg::System(t) => json!({ "role": "system", "content": t }),
                Msg::User(t) => json!({ "role": "user", "content": t }),
                Msg::Assistant { text, calls } => {
                    if calls.is_empty() {
                        json!({ "role": "assistant", "content": text })
                    } else {
                        json!({
                            "role": "assistant",
                            "content": if text.is_empty() { J::Null } else { json!(text) },
                            "tool_calls": calls.iter().map(|c| json!({
                                "id": c.id, "type": "function",
                                "function": { "name": c.name, "arguments": c.args.to_string() },
                            })).collect::<Vec<_>>(),
                        })
                    }
                }
                Msg::ToolResult { id, name, content } => json!({
                    "role": "tool", "tool_call_id": id, "name": name, "content": content,
                }),
            })
            .collect()
    }

    fn openai_tools(specs: &[Spec]) -> Vec<J> {
        specs
            .iter()
            .map(|s| {
                json!({
                    "type": "function",
                    "function": { "name": s.name, "description": s.description, "parameters": s.schema },
                })
            })
            .collect()
    }

    async fn chat_openai(&self, msgs: &[Msg], specs: &[Spec]) -> Result<Turn, String> {
        let mut body = json!({
            "model": self.model,
            "messages": self.openai_messages(msgs),
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
        });
        if !specs.is_empty() {
            body["tools"] = json!(Self::openai_tools(specs));
            body["tool_choice"] = json!("auto");
        }
        let res = Self::client()?
            .post(format!("{}/chat/completions", self.base))
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("the model could not be reached: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(provider_error("OpenAI", status.as_u16(), &text));
        }
        let v: J = serde_json::from_str(&text).map_err(|e| format!("the model sent an unreadable reply: {e}"))?;
        let msg = &v["choices"][0]["message"];
        let prose = msg["content"].as_str().unwrap_or("").to_string();
        let calls = msg["tool_calls"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|c| Call {
                        id: c["id"].as_str().unwrap_or("call").to_string(),
                        name: c["function"]["name"].as_str().unwrap_or("").to_string(),
                        args: parse_args(c["function"]["arguments"].as_str().unwrap_or("{}")),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Turn { text: prose, calls })
    }

    /* ---- Anthropic ---- */

    async fn chat_anthropic(&self, msgs: &[Msg], specs: &[Spec]) -> Result<Turn, String> {
        // Anthropic takes the system prompt as a top-level field and the rest as
        // user/assistant turns; tool results are user turns with a content block.
        let system: String = msgs
            .iter()
            .filter_map(|m| match m {
                Msg::System(t) => Some(t.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut out: Vec<J> = Vec::new();
        for m in msgs {
            match m {
                Msg::System(_) => {}
                Msg::User(t) => out.push(json!({ "role": "user", "content": t })),
                Msg::Assistant { text, calls } => {
                    let mut blocks: Vec<J> = Vec::new();
                    if !text.is_empty() {
                        blocks.push(json!({ "type": "text", "text": text }));
                    }
                    for c in calls {
                        blocks.push(json!({
                            "type": "tool_use", "id": c.id, "name": c.name, "input": c.args,
                        }));
                    }
                    if !blocks.is_empty() {
                        out.push(json!({ "role": "assistant", "content": blocks }));
                    }
                }
                Msg::ToolResult { id, content, .. } => out.push(json!({
                    "role": "user",
                    "content": [{ "type": "tool_result", "tool_use_id": id, "content": content }],
                })),
            }
        }
        let mut body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "messages": out,
        });
        if !system.is_empty() {
            body["system"] = json!(system);
        }
        if !specs.is_empty() {
            body["tools"] = json!(specs
                .iter()
                .map(|s| json!({ "name": s.name, "description": s.description, "input_schema": s.schema }))
                .collect::<Vec<_>>());
        }
        let res = Self::client()?
            .post(format!("{}/v1/messages", self.base))
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("the model could not be reached: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(provider_error("Anthropic", status.as_u16(), &text));
        }
        let v: J = serde_json::from_str(&text).map_err(|e| format!("the model sent an unreadable reply: {e}"))?;
        let mut prose = String::new();
        let mut calls = Vec::new();
        if let Some(blocks) = v["content"].as_array() {
            for b in blocks {
                match b["type"].as_str().unwrap_or("") {
                    "text" => prose.push_str(b["text"].as_str().unwrap_or("")),
                    "tool_use" => calls.push(Call {
                        id: b["id"].as_str().unwrap_or("call").to_string(),
                        name: b["name"].as_str().unwrap_or("").to_string(),
                        args: b["input"].clone(),
                    }),
                    _ => {}
                }
            }
        }
        Ok(Turn { text: prose, calls })
    }

    /* ---- Google Gemini ---- */

    async fn chat_gemini(&self, msgs: &[Msg], specs: &[Spec]) -> Result<Turn, String> {
        let system: String = msgs
            .iter()
            .filter_map(|m| match m {
                Msg::System(t) => Some(t.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut contents: Vec<J> = Vec::new();
        for m in msgs {
            match m {
                Msg::System(_) => {}
                Msg::User(t) => contents.push(json!({ "role": "user", "parts": [{ "text": t }] })),
                Msg::Assistant { text, calls } => {
                    let mut parts: Vec<J> = Vec::new();
                    if !text.is_empty() {
                        parts.push(json!({ "text": text }));
                    }
                    for c in calls {
                        parts.push(json!({ "functionCall": { "name": c.name, "args": c.args } }));
                    }
                    if !parts.is_empty() {
                        contents.push(json!({ "role": "model", "parts": parts }));
                    }
                }
                Msg::ToolResult { name, content, .. } => contents.push(json!({
                    "role": "user",
                    "parts": [{ "functionResponse": { "name": name, "response": { "content": content } } }],
                })),
            }
        }
        let mut body = json!({
            "contents": contents,
            "generationConfig": { "temperature": self.temperature, "maxOutputTokens": self.max_tokens },
        });
        if !system.is_empty() {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
        if !specs.is_empty() {
            body["tools"] = json!([{
                "functionDeclarations": specs.iter().map(|s| json!({
                    "name": s.name, "description": s.description, "parameters": s.schema,
                })).collect::<Vec<_>>(),
            }]);
        }
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base, self.model, self.key
        );
        let res = Self::client()?
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("the model could not be reached: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(provider_error("Gemini", status.as_u16(), &text));
        }
        let v: J = serde_json::from_str(&text).map_err(|e| format!("the model sent an unreadable reply: {e}"))?;
        let mut prose = String::new();
        let mut calls = Vec::new();
        if let Some(parts) = v["candidates"][0]["content"]["parts"].as_array() {
            for (i, p) in parts.iter().enumerate() {
                if let Some(t) = p["text"].as_str() {
                    prose.push_str(t);
                }
                if let Some(fc) = p.get("functionCall") {
                    calls.push(Call {
                        id: format!("gemini-{i}"),
                        name: fc["name"].as_str().unwrap_or("").to_string(),
                        args: fc["args"].clone(),
                    });
                }
            }
        }
        Ok(Turn { text: prose, calls })
    }

    /* ---- Ollama (local or cloud) ---- */

    async fn chat_ollama(&self, msgs: &[Msg], specs: &[Spec]) -> Result<Turn, String> {
        let mut body = json!({
            "model": self.model,
            "messages": self.openai_messages(msgs),
            "stream": false,
            "options": { "temperature": self.temperature },
        });
        if !specs.is_empty() {
            body["tools"] = json!(Self::openai_tools(specs));
        }
        let mut req = Self::client()?.post(format!("{}/api/chat", self.base)).json(&body);
        if !self.key.is_empty() {
            req = req.bearer_auth(&self.key);
        }
        let res = req
            .send()
            .await
            .map_err(|e| format!("the model could not be reached: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(provider_error("Ollama", status.as_u16(), &text));
        }
        let v: J = serde_json::from_str(&text).map_err(|e| format!("the model sent an unreadable reply: {e}"))?;
        let msg = &v["message"];
        let prose = msg["content"].as_str().unwrap_or("").to_string();
        let calls = msg["tool_calls"]
            .as_array()
            .map(|a| {
                a.iter()
                    .enumerate()
                    .map(|(i, c)| Call {
                        id: format!("ollama-{i}"),
                        name: c["function"]["name"].as_str().unwrap_or("").to_string(),
                        args: c["function"]["arguments"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Turn { text: prose, calls })
    }
}

/// Tool arguments arrive as a JSON string on OpenAI and as an object elsewhere.
/// Either way the engine wants an object.
fn parse_args(raw: &str) -> J {
    match serde_json::from_str::<J>(raw) {
        Ok(v) if v.is_object() => v,
        Ok(v) => json!({ "input": v }),
        Err(_) => {
            if raw.trim().is_empty() {
                json!({})
            } else {
                json!({ "input": raw })
            }
        }
    }
}

/// A provider error, with the key stripped so it can never reach a log or a page.
fn provider_error(name: &str, code: u16, body: &str) -> String {
    let short: String = body.chars().take(400).collect();
    let short = redact(&short);
    match code {
        401 | 403 => format!("{name} refused the API key. Check it in Admin -> Settings. ({short})"),
        429 => format!("{name} is rate-limiting us. Try again shortly. ({short})"),
        _ => format!("{name} returned HTTP {code}: {short}"),
    }
}

/// Remove anything that looks like a credential before it is stored or shown.
pub fn redact(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for token in s.split_inclusive(|c: char| c.is_whitespace()) {
        let t = token.trim();
        let looks_secret = t.starts_with("sk-")
            || t.starts_with("sk_")
            || t.starts_with("sk-ant")
            || t.starts_with("AIza")
            || (t.len() > 32 && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        if looks_secret && !t.is_empty() {
            out.push_str("[redacted] ");
        } else {
            out.push_str(token);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_normalised_to_an_object() {
        assert!(parse_args("{\"a\":1}").is_object());
        assert_eq!(parse_args("[1,2]")["input"][0], json!(1));
        assert!(parse_args("").is_object());
        assert_eq!(parse_args("hello")["input"], json!("hello"));
    }

    #[test]
    fn a_key_never_survives_into_an_error() {
        let leaked = "invalid api key sk-proj-abcdefghijklmnopqrstuvwxyz0123456789 provided";
        let clean = redact(leaked);
        assert!(!clean.contains("sk-proj"), "the key survived: {clean}");
        assert!(clean.contains("[redacted]"));
    }

    #[test]
    fn a_local_ollama_needs_no_key() {
        let l = Llm {
            provider: "ollama".into(),
            model: "llama3.1".into(),
            base: "http://127.0.0.1:11434".into(),
            key: String::new(),
            max_tokens: 100,
            temperature: 0.0,
        };
        assert!(l.ready());
    }

    #[test]
    fn a_remote_provider_without_a_key_is_not_ready() {
        let l = Llm {
            provider: "openai".into(),
            model: "gpt-4o-mini".into(),
            base: "https://api.openai.com/v1".into(),
            key: String::new(),
            max_tokens: 100,
            temperature: 0.0,
        };
        assert!(!l.ready());
    }

    #[test]
    fn tool_specs_map_to_the_openai_shape() {
        let specs = vec![Spec {
            name: "web".into(),
            description: "search".into(),
            schema: json!({"type": "object"}),
        }];
        let t = Llm::openai_tools(&specs);
        assert_eq!(t[0]["type"], json!("function"));
        assert_eq!(t[0]["function"]["name"], json!("web"));
    }

    #[test]
    fn a_tool_result_becomes_a_tool_message() {
        let l = Llm {
            provider: "openai".into(),
            model: "m".into(),
            base: "b".into(),
            key: "k".into(),
            max_tokens: 1,
            temperature: 0.0,
        };
        let msgs = vec![Msg::ToolResult { id: "c1".into(), name: "web".into(), content: "ok".into() }];
        let m = l.openai_messages(&msgs);
        assert_eq!(m[0]["role"], json!("tool"));
        assert_eq!(m[0]["tool_call_id"], json!("c1"));
    }
}
