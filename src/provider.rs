use serde::{Deserialize, Serialize};

use crate::config::ProviderCfg;

#[derive(Clone, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool { tool_call_id: String },
}

#[derive(Clone, Debug)]
pub struct Msg {
    pub role: Role,
    pub content: String,
    /// assistant-only: tool calls requested by the model
    pub tool_calls: Vec<ToolCallReq>,
    /// assistant-only: provider-private extras (Gemini thought_signature)
    pub extra: Option<serde_json::Value>,
}

impl Msg {
    pub fn system(content: &str) -> Self {
        Msg { role: Role::System, content: content.into(), tool_calls: vec![], extra: None }
    }
    pub fn user(content: &str) -> Self {
        Msg { role: Role::User, content: content.into(), tool_calls: vec![], extra: None }
    }
    #[allow(dead_code)]
    pub fn assistant(content: &str, tool_calls: Vec<ToolCallReq>) -> Self {
        Msg { role: Role::Assistant, content: content.into(), tool_calls, extra: None }
    }
    pub fn assistant_extra(
        content: &str,
        tool_calls: Vec<ToolCallReq>,
        extra: Option<serde_json::Value>,
    ) -> Self {
        Msg { role: Role::Assistant, content: content.into(), tool_calls, extra }
    }
    pub fn tool_result(tool_call_id: &str, content: &str) -> Self {
        Msg { role: Role::Tool { tool_call_id: tool_call_id.into() }, content: content.into(), tool_calls: vec![], extra: None }
    }

    fn to_wire(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        match &self.role {
            Role::System => { m.insert("role".into(), "system".into()); }
            Role::User => { m.insert("role".into(), "user".into()); }
            Role::Assistant => { m.insert("role".into(), "assistant".into()); }
            Role::Tool { tool_call_id } => {
                m.insert("role".into(), "tool".into());
                m.insert("tool_call_id".into(), tool_call_id.clone().into());
            }
        }
        m.insert("content".into(), self.content.clone().into());
        if !self.tool_calls.is_empty() {
            let tcs: Vec<serde_json::Value> = self
                .tool_calls
                .iter()
                .map(|t| {
                    let mut tc = serde_json::json!({
                        "id": t.id, "type": "function",
                        "function": {"name": t.name, "arguments": t.arguments}
                    });
                    if let Some(extra) = &t.extra {
                        tc["extra_content"] = extra.clone();
                    }
                    tc
                })
                .collect();
            m.insert("tool_calls".into(), serde_json::Value::Array(tcs));
        }
        if let Some(extra) = &self.extra {
            m.insert("extra_content".into(), extra.clone());
        }
        serde_json::Value::Object(m)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallReq {
    pub id: String,
    pub name: String,
    /// raw JSON string of the arguments object
    pub arguments: String,
    // provider-private extras (Gemini thought_signature) — echoed back verbatim
    #[serde(default, skip_serializing)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
}

impl ToolDef {
    pub fn to_wire(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            }
        })
    }
}

#[derive(Deserialize, Debug)]
struct WireMsg {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    extra_content: Option<serde_json::Value>,
    #[serde(default)]
    tool_calls: Option<Vec<WireToolCall>>,
}

#[derive(Clone, Deserialize, Debug)]
struct WireToolCall {
    id: String,
    #[serde(default)]
    extra_content: Option<serde_json::Value>,
    #[serde(default = "default_type")]
    _type: String,
    function: WireFn,
}
fn default_type() -> String { "function".into() }

#[derive(Clone, Deserialize, Debug)]
struct WireFn {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    #[serde(default)] pub prompt_tokens: u64,
    #[serde(default)] pub completion_tokens: u64,
}

#[derive(Deserialize, Debug)]
struct WireResp {
    #[serde(default)]
    choices: Vec<WireChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
struct WireChoice {
    #[serde(default)]
    message: Option<WireMsg>,
}

pub struct ChatOk {
    pub content: String,
    pub extra: Option<serde_json::Value>,
    pub tool_calls: Vec<ToolCallReq>,
    pub usage: Usage,
    pub cost_usd: f64,
}

fn truncate_body(s: &str) -> String {
    if s.len() > 400 { format!("{}…", &s[..400]) } else { s.to_string() }
}

/// One blocking OpenAI-wire chat completion against any configured provider.
/// `tools` empty = plain completion.
pub fn chat(
    p: &ProviderCfg,
    model: &str,
    messages: &[Msg],
    tools: &[ToolDef],
) -> Result<ChatOk, String> {
    let key = std::env::var(&p.api_key_env)
        .map_err(|_| format!("env var {} not set", p.api_key_env))?;
    let url = format!("{}/chat/completions", p.base_url.trim_end_matches('/'));

    let mut body = serde_json::json!({ "model": model });
    body["messages"] = serde_json::Value::Array(messages.iter().map(|m| m.to_wire()).collect());
    if !tools.is_empty() {
        body["tools"] = serde_json::Value::Array(tools.iter().map(|t| t.to_wire()).collect());
    }

    // ── ETHOS §7.1: the send is recorded BEFORE it leaves the machine ──
    crate::enforce::egress::record(
        &url,
        &format!("model call: {model}, {} messages", messages.len()),
        &crate::memory::journal::sha256_hex(&body.to_string()),
    );

    let attempt = |body: &serde_json::Value| -> Result<WireResp, String> {
        match ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", key))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(180))
            .send_json(body.clone())
        {
            Ok(resp) => resp.into_json::<WireResp>().map_err(|e| format!("decode: {e}")),
            Err(ureq::Error::Status(code, r)) => {
                let detail = r.into_string().unwrap_or_default();
                Err(format!("http {code}: {}", truncate_body(&detail)))
            }
            Err(e) => Err(format!("http: {e}")),
        }
    };

    let parsed = match attempt(&body) {
        Ok(r) => r,
        Err(e) => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            attempt(&body).map_err(|e2| format!("{e}; retry failed: {e2}"))?
        }
    };

    let wire_msg = parsed
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or_else(|| "no choices in response".to_string())?;

    let tool_calls = wire_msg
        .tool_calls
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|t| ToolCallReq { id: t.id, name: t.function.name, arguments: t.function.arguments, extra: t.extra_content })
        .collect();

    let usage = parsed.usage.unwrap_or(Usage { prompt_tokens: 0, completion_tokens: 0 });
    let cost_usd = (usage.prompt_tokens as f64 / 1e6) * p.price_per_1m_input
        + (usage.completion_tokens as f64 / 1e6) * p.price_per_1m_output;

    Ok(ChatOk {
        content: wire_msg.content.clone().unwrap_or_default(),
        extra: wire_msg.extra_content.clone(),
        tool_calls,
        usage,
        cost_usd,
    })
}
