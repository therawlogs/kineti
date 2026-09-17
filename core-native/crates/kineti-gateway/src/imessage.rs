//! macOS iMessage Bridge Connector.
//!
//! Provides local AppleScript commands and SQLite integration for
//! reading and sending iMessages natively on macOS.
//! Enforces [`KinetiConnectorProtocol`] so that outbound messaging
//! strictly requires an [`ActionAuthorizationToken`].

use kineti_connectors::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use std::collections::BTreeMap;

/// macOS iMessage Bridge Client.
#[derive(Debug, Default, Clone)]
pub struct IMessageBridge;

impl IMessageBridge {
    /// Creates a new iMessage bridge client.
    pub fn new() -> Self {
        Self
    }

    /// Builds AppleScript command string to send a plain text message to a phone or email.
    pub fn build_send_text_script(&self, recipient: &str, body: &str) -> String {
        let escaped_body = body.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_recipient = recipient.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "tell application \"Messages\"\n  set targetService to 1st service whose service type = iMessage\n  set targetBuddy to buddy \"{}\" of targetService\n  send \"{}\" to targetBuddy\nend tell",
            escaped_recipient, escaped_body
        )
    }

    /// Builds AppleScript command string to send an image or file attachment.
    pub fn build_send_file_script(&self, recipient: &str, posix_file_path: &str) -> String {
        let escaped_path = posix_file_path.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_recipient = recipient.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "tell application \"Messages\"\n  set targetService to 1st service whose service type = iMessage\n  set targetBuddy to buddy \"{}\" of targetService\n  send POSIX file \"{}\" to targetBuddy\nend tell",
            escaped_recipient, escaped_path
        )
    }
}

impl KinetiConnectorProtocol for IMessageBridge {
    fn connector_name(&self) -> &'static str {
        "imessage"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["send_text", "send_file"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "send_text" | "send_file" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence, // Fail closed
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "send_text" => {
                let recipient = get_str_property(payload, "recipient").unwrap_or("");
                let body = get_str_property(payload, "body").unwrap_or("");
                if recipient.is_empty() {
                    return Err(ConnectorProtocolError::InvalidPayload(
                        "Missing recipient in payload".to_string(),
                    ));
                }
                let script = self.build_send_text_script(recipient, body);
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("script_generated".to_string()));
                map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                map.insert("script".to_string(), Value::String(script));
                Ok(Value::Object(map))
            }
            "send_file" => {
                let recipient = get_str_property(payload, "recipient").unwrap_or("");
                let file_path = get_str_property(payload, "file_path").unwrap_or("");
                if recipient.is_empty() {
                    return Err(ConnectorProtocolError::InvalidPayload(
                        "Missing recipient in payload".to_string(),
                    ));
                }
                let script = self.build_send_file_script(recipient, file_path);
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("script_generated".to_string()));
                map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                map.insert("script".to_string(), Value::String(script));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kineti_connectors::ActionAuthorizationToken;

    #[test]
    fn test_imessage_applescript_builder() {
        let bridge = IMessageBridge::new();
        let script = bridge.build_send_text_script("+15551234567", "Hello from Kineti iMessage!");

        assert!(script.contains("tell application \"Messages\""));
        assert!(script.contains("buddy \"+15551234567\""));
        assert!(script.contains("send \"Hello from Kineti iMessage!\""));
    }

    #[test]
    fn test_imessage_send_file_script() {
        let bridge = IMessageBridge::new();
        let script = bridge.build_send_file_script("+15551234567", "/tmp/generated_workspace.png");

        assert!(script.contains("send POSIX file \"/tmp/generated_workspace.png\""));
    }

    #[test]
    fn test_imessage_outbound_requires_authorization_token() {
        let bridge = IMessageBridge::new();
        let mut msg_map = BTreeMap::new();
        msg_map.insert("recipient".to_string(), Value::String("+15551234567".to_string()));
        msg_map.insert("body".to_string(), Value::String("Outbound update".to_string()));
        let payload = Value::Object(msg_map);

        // Invariant: Knowing recipient does NOT grant permission to contact.
        // Attempting to send text without token MUST fail.
        let blocked = bridge.execute("send_text", &payload, None);
        assert_eq!(
            blocked.err(),
            Some(ConnectorProtocolError::MissingAuthorizationToken)
        );

        // With valid token bound to exact payload
        let mut token = ActionAuthorizationToken::mint(
            "tok_imsg_01",
            "praveen",
            "imessage",
            "send_text",
            &payload,
            300,
            1000,
        );

        let sent = bridge.execute_at("send_text", &payload, Some(&mut token), 1050);
        assert!(sent.is_ok());
        assert!(token.consumed);

        // Replay with consumed token is rejected
        let replay = bridge.execute_at("send_text", &payload, Some(&mut token), 1060);
        assert_eq!(
            replay.err(),
            Some(ConnectorProtocolError::TokenAlreadyConsumed)
        );
    }
}
