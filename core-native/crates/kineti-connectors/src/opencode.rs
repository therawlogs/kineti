//! OpenCode Go LLM Inference Connector.
//!
//! Sends context-hydrated prompts to the OpenCode Go API
//! (`https://opencode.ai/zen/go/v1/chat/completions`) and parses streamed or
//! non-streamed responses. This connector is the primary LLM reasoning backend
//! for Kineti's deliberative cortex.
//!
//! ## Wire Protocol
//! Uses the OpenAI-compatible `/chat/completions` format so the same connector
//! works against OpenCode Go, OpenRouter, Groq, or any compatible gateway.

/// OpenCode Go inference client configuration.
#[derive(Debug, Clone)]
pub struct OpenCodeClient {
    /// API key for authentication.
    api_key: String,
    /// Base endpoint URL.
    endpoint: String,
    /// Model identifier (e.g. "opencode-go/claude-3-7-sonnet").
    model: String,
}

/// A chat message in the OpenAI-compatible format.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessage {
    /// Role: "system", "user", or "assistant".
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Request payload for OpenCode Go inference.
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Ordered list of chat messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0–2.0).
    pub temperature: f32,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Whether to stream the response.
    pub stream: bool,
}

/// Parsed inference response.
#[derive(Debug, Clone, PartialEq)]
pub struct InferenceResponse {
    /// Model that generated the response.
    pub model: String,
    /// Generated text content.
    pub content: String,
    /// Prompt tokens used.
    pub prompt_tokens: u32,
    /// Completion tokens generated.
    pub completion_tokens: u32,
    /// Finish reason: "stop", "length", or "content_filter".
    pub finish_reason: String,
}

impl OpenCodeClient {
    /// Creates a new OpenCode Go client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: "https://opencode.ai/zen/go/v1/chat/completions".to_string(),
            model: "opencode-go/claude-3-7-sonnet".to_string(),
        }
    }

    /// Creates a client with a custom endpoint and model.
    pub fn with_config(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }

    /// Returns the configured API endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Returns the configured model identifier.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Builds the HTTP request URL, headers, and JSON body for a chat completion.
    pub fn build_request(&self, request: &InferenceRequest) -> (String, Vec<(&'static str, String)>, String) {
        let headers = vec![
            ("Authorization", format!("Bearer {}", self.api_key)),
            ("Content-Type", "application/json".to_string()),
            ("User-Agent", "Kineti-OpenCode/1.0".to_string()),
        ];

        let mut messages_json = String::from("[");
        for (i, msg) in request.messages.iter().enumerate() {
            if i > 0 {
                messages_json.push(',');
            }
            messages_json.push_str(&format!(
                "{{\"role\":{},\"content\":{}}}",
                escape_json_string(&msg.role),
                escape_json_string(&msg.content),
            ));
        }
        messages_json.push(']');

        let body = format!(
            "{{\"model\":{},\"messages\":{},\"temperature\":{},\"max_tokens\":{},\"stream\":{}}}",
            escape_json_string(&self.model),
            messages_json,
            request.temperature,
            request.max_tokens,
            request.stream,
        );

        (self.endpoint.clone(), headers, body)
    }

    /// Parses a non-streamed JSON response into an [`InferenceResponse`].
    pub fn parse_response(&self, json_body: &str) -> Result<InferenceResponse, &'static str> {
        let model = extract_json_str(json_body, "\"model\":");
        let content = Self::extract_content(json_body);
        let finish_reason = extract_json_str(json_body, "\"finish_reason\":");
        let prompt_tokens = extract_json_num(json_body, "\"prompt_tokens\":");
        let completion_tokens = extract_json_num(json_body, "\"completion_tokens\":");

        if content.is_empty() {
            return Err("No content in response");
        }

        Ok(InferenceResponse {
            model: if model.is_empty() { self.model.clone() } else { model },
            content,
            prompt_tokens,
            completion_tokens,
            finish_reason: if finish_reason.is_empty() {
                "stop".to_string()
            } else {
                finish_reason
            },
        })
    }

    /// Extracts the assistant message content from a chat completion response.
    fn extract_content(json_body: &str) -> String {
        // Find the choices array, then the first message content
        if let Some(choices_pos) = json_body.find("\"choices\"") {
            let rest = &json_body[choices_pos..];
            if let Some(content_pos) = rest.find("\"content\":") {
                let after = &rest[content_pos + 10..];
                return extract_quoted(after);
            }
        }
        String::new()
    }

    /// Builds a simple single-turn inference request.
    pub fn build_simple_request(system_prompt: &str, user_message: &str) -> InferenceRequest {
        InferenceRequest {
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 1024,
            stream: false,
        }
    }
}

// ---------------------------------------------------------------------------
// JSON helpers (zero-dependency)
// ---------------------------------------------------------------------------

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn extract_json_str(s: &str, prefix: &str) -> String {
    if let Some(pos) = s.find(prefix) {
        let rest = &s[pos + prefix.len()..];
        return extract_quoted(rest);
    }
    String::new()
}

fn extract_quoted(s: &str) -> String {
    let trimmed = s.trim_start();
    if !trimmed.starts_with('"') {
        return String::new();
    }
    let after_q = &trimmed[1..];
    let mut out = String::new();
    let mut escaping = false;
    for c in after_q.chars() {
        if escaping {
            match c {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                _ => out.push(c),
            }
            escaping = false;
        } else if c == '\\' {
            escaping = true;
        } else if c == '"' {
            break;
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_json_num(s: &str, prefix: &str) -> u32 {
    if let Some(pos) = s.find(prefix) {
        let rest = &s[pos + prefix.len()..];
        let trimmed = rest.trim_start();
        let num_str: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num_str.parse().unwrap_or(0);
    }
    0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_client_defaults() {
        let client = OpenCodeClient::new("test_key_123");
        assert_eq!(client.endpoint(), "https://opencode.ai/zen/go/v1/chat/completions");
        assert_eq!(client.model(), "opencode-go/claude-3-7-sonnet");
    }

    #[test]
    fn test_opencode_custom_config() {
        let client = OpenCodeClient::with_config(
            "custom_key",
            "https://custom.api.com/v1/completions",
            "custom-model",
        );
        assert_eq!(client.endpoint(), "https://custom.api.com/v1/completions");
        assert_eq!(client.model(), "custom-model");
    }

    #[test]
    fn test_build_request_structure() {
        let client = OpenCodeClient::new("live_key_abc");
        let request = OpenCodeClient::build_simple_request(
            "You are Kineti, a helpful assistant.",
            "What is the weather?",
        );

        let (url, headers, body) = client.build_request(&request);

        assert_eq!(url, "https://opencode.ai/zen/go/v1/chat/completions");
        assert!(headers.iter().any(|(k, v)| *k == "Authorization" && v == "Bearer live_key_abc"));
        assert!(headers.iter().any(|(k, v)| *k == "Content-Type" && v == "application/json"));
        assert!(body.contains("opencode-go/claude-3-7-sonnet"));
        assert!(body.contains("What is the weather?"));
        assert!(body.contains("\"role\":\"system\""));
        assert!(body.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_parse_response_success() {
        let client = OpenCodeClient::new("key");
        let response_json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "model": "opencode-go/claude-3-7-sonnet",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "The weather in Tokyo is sunny, 24C."
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 42,
                "completion_tokens": 15,
                "total_tokens": 57
            }
        }"#;

        let resp = client.parse_response(response_json).expect("Parse should succeed");
        assert_eq!(resp.model, "opencode-go/claude-3-7-sonnet");
        assert_eq!(resp.content, "The weather in Tokyo is sunny, 24C.");
        assert_eq!(resp.finish_reason, "stop");
        assert_eq!(resp.prompt_tokens, 42);
        assert_eq!(resp.completion_tokens, 15);
    }

    #[test]
    fn test_parse_response_empty_content_fails() {
        let client = OpenCodeClient::new("key");
        let bad_json = r#"{"choices": []}"#;
        assert!(client.parse_response(bad_json).is_err());
    }

    #[test]
    fn test_simple_request_builder() {
        let request = OpenCodeClient::build_simple_request(
            "You are helpful.",
            "Hello!",
        );
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "Hello!");
        assert!((request.temperature - 0.7).abs() < 0.01);
        assert_eq!(request.max_tokens, 1024);
        assert!(!request.stream);
    }

    #[test]
    fn test_escape_json_string_special_chars() {
        let result = escape_json_string("Hello \"world\"\nnew line");
        assert_eq!(result, r#""Hello \"world\"\nnew line""#);
    }

    #[test]
    fn test_multi_turn_request() {
        let client = OpenCodeClient::new("key");
        let request = InferenceRequest {
            messages: vec![
                ChatMessage { role: "system".to_string(), content: "You are Kineti.".to_string() },
                ChatMessage { role: "user".to_string(), content: "Hi".to_string() },
                ChatMessage { role: "assistant".to_string(), content: "Hey! What's up?".to_string() },
                ChatMessage { role: "user".to_string(), content: "Book me a flight".to_string() },
            ],
            temperature: 0.5,
            max_tokens: 512,
            stream: false,
        };

        let (_, _, body) = client.build_request(&request);
        assert!(body.contains("You are Kineti."));
        assert!(body.contains("Book me a flight"));
        assert!(body.contains("\"temperature\":0.5"));
        assert!(body.contains("\"max_tokens\":512"));
    }
}
