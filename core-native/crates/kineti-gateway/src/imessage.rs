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
pub struct IMessageBridge {
    phone_number: Option<String>,
}

impl IMessageBridge {
    /// Creates a new iMessage bridge client.
    pub fn new() -> Self {
        Self { phone_number: None }
    }

    /// Sets the assigned iMessage phone number or email handle.
    pub fn with_phone_number(mut self, number: impl Into<String>) -> Self {
        self.phone_number = Some(number.into());
        self
    }

    /// Returns the active configured iMessage number, checking environment override first.
    pub fn phone_number(&self) -> String {
        if let Ok(env_num) = std::env::var("KINETI_IMESSAGE_NUMBER") {
            if !env_num.is_empty() {
                return env_num;
            }
        }
        if let Some(ref num) = self.phone_number {
            num.clone()
        } else {
            "Not Configured".to_string()
        }
    }

    /// Parses latitude and longitude coordinates from location URLs or raw coordinate strings.
    pub fn extract_location_coordinates(text: &str) -> Option<(f64, f64)> {
        // 1. Check for Apple Maps format: ?ll=lat,lon or ?q=lat,lon
        if let Some(pos) = text.find("maps.apple.com/?ll=").or_else(|| text.find("maps.apple.com/?q=")) {
            let rest = &text[pos..];
            if let Some(eq_pos) = rest.find('=') {
                let query_part = &rest[eq_pos + 1..];
                let coords_str = query_part.split('&').next().unwrap_or("").trim();
                return Self::parse_lat_lon_pair(coords_str);
            }
        }

        // 2. Check for Google Maps format: maps.google.com/?q=lat,lon
        if let Some(pos) = text.find("google.com/maps?q=").or_else(|| text.find("maps.google.com/?q=")) {
            let rest = &text[pos..];
            if let Some(eq_pos) = rest.find('=') {
                let query_part = &rest[eq_pos + 1..];
                let coords_str = query_part.split('&').next().unwrap_or("").trim();
                return Self::parse_lat_lon_pair(coords_str);
            }
        }

        // 3. Fallback: Parse explicit latitude/longitude pair (e.g. "37.7749, -122.4194")
        for line in text.lines() {
            let cleaned = line.trim().trim_start_matches("Location:").trim();
            if let Some(coords) = Self::parse_lat_lon_pair(cleaned) {
                return Some(coords);
            }
        }

        None
    }

    fn parse_lat_lon_pair(s: &str) -> Option<(f64, f64)> {
        let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
        if parts.len() == 2 {
            if let (Ok(lat), Ok(lon)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
                    return Some((lat, lon));
                }
            }
        }
        None
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

    #[test]
    fn test_imessage_configurable_phone_number() {
        let bridge = IMessageBridge::new();
        assert_eq!(bridge.phone_number(), "Not Configured");

        let bridge_custom = IMessageBridge::new().with_phone_number("+1 (415) 555-0100");
        assert_eq!(bridge_custom.phone_number(), "+1 (415) 555-0100");
    }

    #[test]
    fn test_imessage_extract_location_coordinates() {
        // Apple Maps link
        let apple_link = "I parked here: https://maps.apple.com/?ll=37.774929,-122.419416&q=My%20Location";
        let (lat, lon) = IMessageBridge::extract_location_coordinates(apple_link).unwrap();
        assert!((lat - 37.774929).abs() < 1e-5);
        assert!((lon - (-122.419416)).abs() < 1e-5);

        // Google Maps link
        let google_link = "Meeting spot: https://maps.google.com/?q=37.783333,-122.416667";
        let (glat, glon) = IMessageBridge::extract_location_coordinates(google_link).unwrap();
        assert!((glat - 37.783333).abs() < 1e-5);
        assert!((glon - (-122.416667)).abs() < 1e-5);

        // Plain text location
        let plain = "Location: 37.7900, -122.4000";
        let (plat, plon) = IMessageBridge::extract_location_coordinates(plain).unwrap();
        assert!((plat - 37.7900).abs() < 1e-5);
        assert!((plon - (-122.4000)).abs() < 1e-5);

        // Non-location text returns None
        assert!(IMessageBridge::extract_location_coordinates("Hey, what's up?").is_none());
    }
}
