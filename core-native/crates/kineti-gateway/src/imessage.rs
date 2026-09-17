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

    /// Executes an AppleScript string using the macOS system `osascript` binary.
    pub fn execute_applescript(script: &str) -> Result<String, String> {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to invoke osascript: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("osascript error: {}", stderr.trim()))
        }
    }

    /// Reads recent messages from the macOS Messages SQLite database.
    pub fn read_recent_messages(limit: usize) -> Result<String, String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = format!("{}/Library/Messages/chat.db", home);

        let query = format!(
            "SELECT m.rowid, m.text, datetime(m.date/1000000000 + strftime('%s', '2001-01-01'), 'unixepoch') as sent_at, h.id as sender FROM message m LEFT JOIN handle h ON m.handle_id = h.rowid WHERE m.text IS NOT NULL ORDER BY m.date DESC LIMIT {};",
            limit.clamp(1, 50)
        );

        let output = std::process::Command::new("sqlite3")
            .arg("-json")
            .arg(&db_path)
            .arg(&query)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr);
                    Err(format!("Messages DB query error: {}", err.trim()))
                }
            }
            Err(e) => Err(format!("sqlite3 command execution failed: {}", e)),
        }
    }
}

impl KinetiConnectorProtocol for IMessageBridge {
    fn connector_name(&self) -> &'static str {
        "imessage"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["send_text", "send_file", "read_recent"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "read_recent" => ConsequenceLevel::Trivial,
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
            "read_recent" => {
                let limit = if let Value::Object(m) = payload {
                    if let Some(Value::Number(n)) = m.get("limit") {
                        n.as_str().parse::<usize>().unwrap_or(10)
                    } else {
                        10
                    }
                } else {
                    10
                };
                let mut map = BTreeMap::new();
                match Self::read_recent_messages(limit) {
                    Ok(json_str) => {
                        map.insert("status".to_string(), Value::String("read_success".to_string()));
                        map.insert("messages".to_string(), Value::String(json_str));
                        map.insert("live_dispatched".to_string(), Value::from(true));
                    }
                    Err(err) => {
                        map.insert("status".to_string(), Value::String("read_restricted".to_string()));
                        map.insert("reason".to_string(), Value::String(err));
                        map.insert("live_dispatched".to_string(), Value::from(false));
                    }
                }
                Ok(Value::Object(map))
            }
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
                let live_send = std::env::var("KINETI_LIVE_IMESSAGE").map(|v| v == "1").unwrap_or(false);
                if live_send {
                    match Self::execute_applescript(&script) {
                        Ok(res) => {
                            map.insert("status".to_string(), Value::String("sent".to_string()));
                            map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                            map.insert("result".to_string(), Value::String(res));
                            map.insert("live_dispatched".to_string(), Value::from(true));
                        }
                        Err(err) => {
                            return Err(ConnectorProtocolError::ExecutionFailed(format!(
                                "iMessage AppleScript execution failed: {}", err
                            )));
                        }
                    }
                } else {
                    map.insert("status".to_string(), Value::String("script_generated".to_string()));
                    map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                    map.insert("script".to_string(), Value::String(script));
                    map.insert("live_dispatched".to_string(), Value::from(false));
                }
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
                let live_send = std::env::var("KINETI_LIVE_IMESSAGE").map(|v| v == "1").unwrap_or(false);
                if live_send {
                    match Self::execute_applescript(&script) {
                        Ok(res) => {
                            map.insert("status".to_string(), Value::String("sent".to_string()));
                            map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                            map.insert("result".to_string(), Value::String(res));
                            map.insert("live_dispatched".to_string(), Value::from(true));
                        }
                        Err(err) => {
                            return Err(ConnectorProtocolError::ExecutionFailed(format!(
                                "iMessage AppleScript execution failed: {}", err
                            )));
                        }
                    }
                } else {
                    map.insert("status".to_string(), Value::String("script_generated".to_string()));
                    map.insert("recipient".to_string(), Value::String(recipient.to_string()));
                    map.insert("script".to_string(), Value::String(script));
                    map.insert("live_dispatched".to_string(), Value::from(false));
                }
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
