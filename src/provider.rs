use serde::{Deserialize, Serialize};
use crate::config::ProviderCfg;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Msg {
    pub role: String,
    pub content: String,
}

impl Msg {
    pub fn system(content: &str) -> Self { Msg { role: "system".into(), content: content.into() } }
    pub fn user(content: &str) -> Self { Msg { role: "user".into(), content: content.into() } }
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    #[serde(default)] pub prompt_tokens: u64,
    #[serde(default)] pub completion_tokens: u64,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    #[serde(default)]
    pub message: Option<AssistantMsg>,
}

#[derive(Deserialize, Debug)]
pub struct AssistantMsg {
    #[serde(default)]
    pub content: Option<String>,
}

pub struct ChatOk {
    pub text: String,
    pub usage: Usage,
    pub cost_usd: f64,
}

/// One blocking OpenAI-wire chat completion against any configured provider.
pub fn chat(p: &ProviderCfg, model: &str, messages: &[Msg]) -> Result<ChatOk, String> {
    let key = std::env::var(&p.api_key_env)
        .map_err(|_| format!("env var {} not set", p.api_key_env))?;
    let url = format!("{}/chat/completions", p.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
    });

    let attempt = |body: &serde_json::Value| -> Result<ChatResponse, String> {
        let resp = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", key))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(120))
            .send_json(body.clone())
            .map_err(|e| format!("http: {e}"))?;
        resp.into_json::<ChatResponse>().map_err(|e| format!("decode: {e}"))
    };

    // one retry on transport/5xx-style failures
    let parsed = match attempt(&body) {
        Ok(r) => r,
        Err(e) => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            attempt(&body).map_err(|e2| format!("{e}; retry failed: {e2}"))?
        }
    };

    let choice = parsed
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or_else(|| "no choices in response".to_string())?;
    let text = choice.content.clone().unwrap_or_default();
    let usage = parsed.usage.unwrap_or(Usage { prompt_tokens: 0, completion_tokens: 0 });
    let cost_usd = (usage.prompt_tokens as f64 / 1e6) * p.price_per_1m_input
        + (usage.completion_tokens as f64 / 1e6) * p.price_per_1m_output;

    Ok(ChatOk { text, usage, cost_usd })
}
