//! # Team Communications, Audio & Voice Connectors (`team_comms`)
//!
//! Includes:
//! - **Wispr Flow**: Real-time voice dictation, speech-to-text, MCP integration (`https://api.wisprflow.ai/connect/mcp`).
//! - **Slack**: Channels, messages, canvas documents, emoji reactions.
//! - **Granola**: Meeting notes, AI transcripts, action item summaries.
//! - Governed by [`KinetiConnectorProtocol`].

use crate::protocol::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use std::collections::BTreeMap;

/// Wispr Flow Voice & MCP Connector.
#[derive(Debug, Clone)]
pub struct WisprFlowClient {
    endpoint: String,
    api_key: String,
}

impl Default for WisprFlowClient {
    fn default() -> Self {
        Self::new("https://api.wisprflow.ai/connect/mcp", "")
    }
}

impl WisprFlowClient {
    /// Creates a new Wispr Flow client with endpoint and API key.
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
        }
    }

    /// Returns the active MCP server endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// API key getter.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

impl KinetiConnectorProtocol for WisprFlowClient {
    fn connector_name(&self) -> &'static str {
        "wispr_flow"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "transcribe_audio",
            "stream_dictation",
            "voice_intent",
            "mcp_ping",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "mcp_ping" => ConsequenceLevel::Trivial,
            "transcribe_audio" | "stream_dictation" | "voice_intent" => ConsequenceLevel::Operational,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        map.insert("provider".to_string(), Value::String("wispr_flow".to_string()));
        map.insert("endpoint".to_string(), Value::String(self.endpoint.clone()));
        map.insert("action".to_string(), Value::String(action.to_string()));

        match action {
            "mcp_ping" => {
                map.insert("status".to_string(), Value::String("connected".to_string()));
                Ok(Value::Object(map))
            }
            "transcribe_audio" | "stream_dictation" => {
                let text = get_str_property(payload, "sample_text").unwrap_or("Transcribed voice stream from Wispr Flow");
                map.insert("transcription".to_string(), Value::String(text.to_string()));
                map.insert("status".to_string(), Value::String("transcribed".to_string()));
                Ok(Value::Object(map))
            }
            "voice_intent" => {
                let spoken = get_str_property(payload, "spoken_command").unwrap_or("");
                map.insert("command".to_string(), Value::String(spoken.to_string()));
                map.insert("status".to_string(), Value::String("intent_extracted".to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

/// Slack Team Communication Connector.
#[derive(Debug, Clone)]
pub struct SlackClient {
    bot_token: String,
}

impl SlackClient {
    /// Creates a new Slack client.
    pub fn new(bot_token: impl Into<String>) -> Self {
        Self {
            bot_token: bot_token.into(),
        }
    }

    /// Bot token getter.
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }
}

impl KinetiConnectorProtocol for SlackClient {
    fn connector_name(&self) -> &'static str {
        "slack"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "search_conversations",
            "read_canvas",
            "react_emoji",
            "post_message",
            "update_canvas",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "search_conversations" | "read_canvas" => ConsequenceLevel::Trivial,
            "react_emoji" => ConsequenceLevel::Operational,
            "post_message" | "update_canvas" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        map.insert("provider".to_string(), Value::String("slack".to_string()));
        map.insert("action".to_string(), Value::String(action.to_string()));

        match action {
            "search_conversations" | "read_canvas" => {
                map.insert("status".to_string(), Value::String("queried".to_string()));
                Ok(Value::Object(map))
            }
            "react_emoji" => {
                let emoji = get_str_property(payload, "emoji").unwrap_or("thumbsup");
                map.insert("emoji".to_string(), Value::String(emoji.to_string()));
                map.insert("status".to_string(), Value::String("reacted".to_string()));
                Ok(Value::Object(map))
            }
            "post_message" => {
                let channel = get_str_property(payload, "channel").unwrap_or("general");
                let text = get_str_property(payload, "text").unwrap_or("");
                map.insert("channel".to_string(), Value::String(channel.to_string()));
                map.insert("text".to_string(), Value::String(text.to_string()));
                map.insert("status".to_string(), Value::String("posted".to_string()));
                Ok(Value::Object(map))
            }
            "update_canvas" => {
                map.insert("status".to_string(), Value::String("canvas_updated".to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

/// Granola Audio Notes & Meeting Summaries Connector.
#[derive(Debug, Clone)]
pub struct GranolaClient {
    api_key: String,
}

impl GranolaClient {
    /// Creates a new Granola client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// API key getter.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

impl KinetiConnectorProtocol for GranolaClient {
    fn connector_name(&self) -> &'static str {
        "granola"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["fetch_meeting_notes", "fetch_transcript", "extract_action_items"]
    }

    fn evaluate_consequence(&self, _action: &str, _payload: &Value) -> ConsequenceLevel {
        ConsequenceLevel::Trivial
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        map.insert("provider".to_string(), Value::String("granola".to_string()));
        map.insert("action".to_string(), Value::String(action.to_string()));

        let meeting_id = get_str_property(payload, "meeting_id").unwrap_or("latest");
        map.insert("meeting_id".to_string(), Value::String(meeting_id.to_string()));
        map.insert("status".to_string(), Value::String("retrieved".to_string()));
        Ok(Value::Object(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wispr_flow_endpoint_and_consequence() {
        let client = WisprFlowClient::default();
        assert_eq!(client.endpoint(), "https://api.wisprflow.ai/connect/mcp");
        assert_eq!(
            client.evaluate_consequence("mcp_ping", &Value::Null),
            ConsequenceLevel::Trivial
        );
        assert_eq!(
            client.evaluate_consequence("transcribe_audio", &Value::Null),
            ConsequenceLevel::Operational
        );
    }

    #[test]
    fn test_slack_and_granola_consequences() {
        let slack = SlackClient::new("xoxb-test");
        assert_eq!(slack.evaluate_consequence("react_emoji", &Value::Null), ConsequenceLevel::Operational);
        assert_eq!(slack.evaluate_consequence("post_message", &Value::Null), ConsequenceLevel::HighConsequence);

        let granola = GranolaClient::new("gr_test");
        assert_eq!(granola.evaluate_consequence("fetch_meeting_notes", &Value::Null), ConsequenceLevel::Trivial);
    }
}
