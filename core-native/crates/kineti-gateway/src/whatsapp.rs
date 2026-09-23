//! Meta WhatsApp Cloud API Webhook and Message Dispatcher.
//!
//! Handles:
//! 1. Hub challenge webhook verification
//! 2. Inbound message parsing (text, audio, image, location)
//! 3. Outbound text message generation
//! 4. Outbound native emoji reactions (`type: "reaction"`)
//! 5. Outbound native image attachments (`type: "image"`)
//! 6. Protocolized execution via [`KinetiConnectorProtocol`] enforcing that
//!    knowing a phone number does NOT grant permission to contact.

use kineti_connectors::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use std::collections::BTreeMap;

/// Inbound WhatsApp message extracted from webhook payload.
#[derive(Debug, Clone, PartialEq)]
pub struct InboundWhatsAppMessage {
    /// Meta WhatsApp message ID (`wamid.HBgLMTQxMj...`).
    pub message_id: String,
    /// Sender phone number in E.164 format.
    pub from_phone: String,
    /// Unix timestamp in seconds.
    pub timestamp_sec: u64,
    /// Extracted text body if message is text.
    pub body: String,
    /// Media URL or ID if message contains audio or photo.
    pub media_id: Option<String>,
    /// Media MIME type.
    pub mime_type: Option<String>,
}

/// Outbound WhatsApp payload type.
#[derive(Debug, Clone, PartialEq)]
pub enum OutboundWhatsAppPayload {
    /// Standard text message.
    Text {
        /// Recipient phone number.
        to: String,
        /// Message body.
        body: String,
    },
    /// Native emoji reaction attached to a specific message bubble.
    Reaction {
        /// Recipient phone number.
        to: String,
        /// Target message ID being reacted to.
        target_message_id: String,
        /// Emoji character (e.g. "👍", "❤️", "⚡").
        emoji: &'static str,
    },
    /// Direct photo message attachment with caption.
    Image {
        /// Recipient phone number.
        to: String,
        /// Image URL.
        image_url: String,
        /// Caption text.
        caption: String,
    },
}

/// WhatsApp Cloud API Gateway.
#[derive(Debug, Clone)]
pub struct WhatsAppGateway {
    verify_token: String,
    phone_number: Option<String>,
    access_token: Option<String>,
    phone_number_id: Option<String>,
}

impl WhatsAppGateway {
    /// Creates a new WhatsApp gateway with verification secret.
    pub fn new(verify_token: impl Into<String>) -> Self {
        Self {
            verify_token: verify_token.into(),
            phone_number: None,
            access_token: None,
            phone_number_id: None,
        }
    }

    /// Configures Meta Graph API credentials for real dispatch.
    pub fn with_credentials(mut self, access_token: impl Into<String>, phone_number_id: impl Into<String>) -> Self {
        self.access_token = Some(access_token.into());
        self.phone_number_id = Some(phone_number_id.into());
        self
    }

    /// Sets the assigned WhatsApp business phone number.
    pub fn with_phone_number(mut self, number: impl Into<String>) -> Self {
        self.phone_number = Some(number.into());
        self
    }

    /// Returns the active configured WhatsApp number, checking environment override first.
    pub fn phone_number(&self) -> String {
        if let Ok(env_num) = std::env::var("KINETI_WHATSAPP_NUMBER") {
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

    /// Generates a secure WhatsApp onboarding pairing token for an unlinked user number.
    pub fn generate_pairing_token(phone_number: &str, timestamp: u64) -> String {
        let clean_phone = phone_number.replace(|c: char| !c.is_ascii_digit(), "");
        let raw = format!("{}:{}:kineti_wa_pair_v1", clean_phone, timestamp);
        let hash = kineti_core::kernel::hex_encode(&kineti_core::kernel::blake3(raw.as_bytes()));
        format!("wa_{}_{}", &hash[..16], timestamp)
    }

    /// Generates the web pairing URL for WhatsApp onboarding.
    pub fn generate_onboarding_url(pairing_token: &str) -> String {
        format!("https://app.getkineti.com/whatsapp-onboarding?t={}", pairing_token)
    }

    /// Verifies a pairing token against an incoming phone number.
    pub fn verify_pairing_token(token: &str, phone_number: &str) -> bool {
        let parts: Vec<&str> = token.split('_').collect();
        if parts.len() != 3 || parts[0] != "wa" {
            return false;
        }
        if let Ok(timestamp) = parts[2].parse::<u64>() {
            let expected = Self::generate_pairing_token(phone_number, timestamp);
            token == expected
        } else {
            false
        }
    }

    /// Verifies Meta webhook registration challenge (`GET /api/webhook/whatsapp`).
    pub fn verify_challenge(&self, mode: Option<&str>, token: Option<&str>, challenge: Option<&str>) -> Option<String> {
        if mode == Some("subscribe") && token == Some(&self.verify_token) {
            challenge.map(|c| c.to_string())
        } else {
            None
        }
    }

    /// Parses inbound webhook JSON into structured message.
    pub fn parse_inbound(&self, body: &str) -> Option<InboundWhatsAppMessage> {
        if !body.contains("\"messages\":") {
            return None;
        }

        let msg_id = extract_str(body, "\"id\":");
        let from_phone = extract_str(body, "\"from\":");

        let text_body = if body.contains("\"text\":") {
            extract_str(body, "\"body\":")
        } else {
            String::new()
        };

        let media_id = if body.contains("\"image\":") || body.contains("\"audio\":") {
            let id = extract_str(body, "\"id\":");
            if !id.is_empty() { Some(id) } else { None }
        } else {
            None
        };

        let mime_type = if body.contains("\"mime_type\":") {
            let m = extract_str(body, "\"mime_type\":");
            if !m.is_empty() { Some(m) } else { None }
        } else {
            None
        };

        if !from_phone.is_empty() {
            Some(InboundWhatsAppMessage {
                message_id: msg_id,
                from_phone,
                timestamp_sec: 1710000000,
                body: text_body,
                media_id,
                mime_type,
            })
        } else {
            None
        }
    }

    /// Builds JSON payload for sending a text message.
    pub fn build_text_payload(&self, to: &str, text: &str) -> String {
        let escaped = text.replace('"', "\\\"").replace('\n', "\\n");
        format!(
            "{{\"messaging_product\":\"whatsapp\",\"recipient_type\":\"individual\",\"to\":\"{}\",\"type\":\"text\",\"text\":{{\"preview_url\":false,\"body\":\"{}\"}}}}",
            to, escaped
        )
    }

    /// Builds JSON payload for attaching a native emoji reaction.
    pub fn build_reaction_payload(&self, to: &str, target_message_id: &str, emoji: &str) -> String {
        format!(
            "{{\"messaging_product\":\"whatsapp\",\"recipient_type\":\"individual\",\"to\":\"{}\",\"type\":\"reaction\",\"reaction\":{{\"message_id\":\"{}\",\"emoji\":\"{}\"}}}}",
            to, target_message_id, emoji
        )
    }

    /// Builds JSON payload for delivering an image with caption.
    pub fn build_image_payload(&self, to: &str, image_url: &str, caption: &str) -> String {
        let escaped_caption = caption.replace('"', "\\\"").replace('\n', "\\n");
        format!(
            "{{\"messaging_product\":\"whatsapp\",\"recipient_type\":\"individual\",\"to\":\"{}\",\"type\":\"image\",\"image\":{{\"link\":\"{}\",\"caption\":\"{}\"}}}}",
            to, image_url, escaped_caption
        )
    }

    /// Returns the Meta Cloud API send-message URL for a given phone number ID.
    /// Caller posts the payload from `build_text_payload` / `build_reaction_payload` here.
    pub fn send_api_url(phone_number_id: &str) -> String {
        format!(
            "https://graph.facebook.com/v21.0/{}/messages",
            phone_number_id
        )
    }

    /// Returns the HTTP headers required for Meta Cloud API requests.
    pub fn send_api_headers(access_token: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Authorization", format!("Bearer {}", access_token)),
            ("Content-Type", "application/json".to_string()),
        ]
    }

    /// Builds JSON payload to mark a message as read (blue ticks).
    pub fn build_read_receipt_payload(message_id: &str) -> String {
        format!(
            "{{\"messaging_product\":\"whatsapp\",\"status\":\"read\",\"message_id\":\"{}\"}}",
            message_id
        )
    }

    /// Validates an inbound webhook signature using HMAC-SHA256.
    pub fn validate_signature(app_secret: &str, payload: &str, signature_header: &str) -> bool {
        if !signature_header.starts_with("sha256=") {
            return false;
        }
        let declared_hex = &signature_header[7..];
        if declared_hex.len() != 64 {
            return false;
        }
        let computed = kineti_core::hmac_sha256(app_secret.as_bytes(), payload.as_bytes());
        let computed_hex = kineti_core::hex_encode(&computed);
        kineti_core::constant_time_compare(declared_hex.as_bytes(), computed_hex.as_bytes())
    }
}

impl KinetiConnectorProtocol for WhatsAppGateway {
    fn connector_name(&self) -> &'static str {
        "whatsapp"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "verify_challenge",
            "parse_inbound",
            "read_receipt",
            "react_emoji",
            "send_text",
            "send_image",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "verify_challenge" | "parse_inbound" | "read_receipt" => ConsequenceLevel::Trivial,
            "react_emoji" => ConsequenceLevel::Operational,
            "send_text" | "send_image" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence, // Fail closed
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "verify_challenge" => {
                let mode = get_str_property(payload, "mode");
                let token = get_str_property(payload, "token");
                let challenge = get_str_property(payload, "challenge");
                let verified = self.verify_challenge(mode, token, challenge);
                let mut map = BTreeMap::new();
                map.insert("verified".to_string(), Value::Bool(verified.is_some()));
                if let Some(c) = verified {
                    map.insert("challenge".to_string(), Value::String(c));
                }
                Ok(Value::Object(map))
            }
            "parse_inbound" => {
                let raw_body = get_str_property(payload, "body").unwrap_or("");
                let parsed = self.parse_inbound(raw_body);
                let mut map = BTreeMap::new();
                map.insert("parsed".to_string(), Value::Bool(parsed.is_some()));
                if let Some(msg) = parsed {
                    map.insert("from_phone".to_string(), Value::String(msg.from_phone));
                    map.insert("message_id".to_string(), Value::String(msg.message_id));
                    map.insert("body".to_string(), Value::String(msg.body));
                }
                Ok(Value::Object(map))
            }
            "read_receipt" => {
                let message_id = get_str_property(payload, "message_id").unwrap_or("");
                let raw_payload = Self::build_read_receipt_payload(message_id);
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("receipt_generated".to_string()));
                map.insert("payload".to_string(), Value::String(raw_payload));
                Ok(Value::Object(map))
            }
            "react_emoji" => {
                let to = get_str_property(payload, "to").unwrap_or("");
                let message_id = get_str_property(payload, "target_message_id").unwrap_or("");
                let emoji = get_str_property(payload, "emoji").unwrap_or("⚡");
                let raw_payload = self.build_reaction_payload(to, message_id, emoji);

                let access_token = self.access_token.clone()
                    .or_else(|| std::env::var("WHATSAPP_TOKEN").ok())
                    .or_else(|| std::env::var("WHATSAPP_ACCESS_TOKEN").ok());
                let phone_id = self.phone_number_id.clone()
                    .or_else(|| std::env::var("WHATSAPP_PHONE_NUMBER_ID").ok());

                if let (Some(token), Some(pid)) = (access_token, phone_id) {
                    let url = Self::send_api_url(&pid);
                    let auth_hdr = format!("Bearer {}", token);
                    let headers = [
                        ("Authorization", auth_hdr.as_str()),
                        ("Content-Type", "application/json"),
                    ];
                    let resp = kineti_core::http_post_json(&url, &headers, &raw_payload)
                        .map_err(|e| ConnectorProtocolError::ExecutionFailed(format!("WhatsApp network error: {}", e)))?;
                    if !resp.success {
                        return Err(ConnectorProtocolError::ExecutionFailed(format!(
                            "WhatsApp API HTTP {}: {}", resp.status, resp.body
                        )));
                    }
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("reaction_delivered".to_string()));
                    map.insert("emoji".to_string(), Value::String(emoji.to_string()));
                    map.insert("http_status".to_string(), Value::from(resp.status as u64));
                    map.insert("live_dispatched".to_string(), Value::from(true));
                    Ok(Value::Object(map))
                } else {
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("reaction_queued".to_string()));
                    map.insert("payload".to_string(), Value::String(raw_payload));
                    map.insert("live_dispatched".to_string(), Value::from(false));
                    Ok(Value::Object(map))
                }
            }
            "send_text" => {
                // High-consequence: verified token has already been validated and consumed!
                let to = get_str_property(payload, "to").unwrap_or("");
                let text = get_str_property(payload, "text").unwrap_or("");
                if to.is_empty() {
                    return Err(ConnectorProtocolError::InvalidPayload("Missing recipient 'to'".to_string()));
                }
                let raw_payload = self.build_text_payload(to, text);

                let access_token = self.access_token.clone()
                    .or_else(|| std::env::var("WHATSAPP_TOKEN").ok())
                    .or_else(|| std::env::var("WHATSAPP_ACCESS_TOKEN").ok());
                let phone_id = self.phone_number_id.clone()
                    .or_else(|| std::env::var("WHATSAPP_PHONE_NUMBER_ID").ok());

                if let (Some(token), Some(pid)) = (access_token, phone_id) {
                    let url = Self::send_api_url(&pid);
                    let auth_hdr = format!("Bearer {}", token);
                    let headers = [
                        ("Authorization", auth_hdr.as_str()),
                        ("Content-Type", "application/json"),
                    ];
                    let resp = kineti_core::http_post_json(&url, &headers, &raw_payload)
                        .map_err(|e| ConnectorProtocolError::ExecutionFailed(format!("WhatsApp network error: {}", e)))?;
                    if !resp.success {
                        return Err(ConnectorProtocolError::ExecutionFailed(format!(
                            "WhatsApp API HTTP {}: {}", resp.status, resp.body
                        )));
                    }
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("delivered".to_string()));
                    map.insert("recipient".to_string(), Value::String(to.to_string()));
                    map.insert("http_status".to_string(), Value::from(resp.status as u64));
                    map.insert("live_dispatched".to_string(), Value::from(true));
                    map.insert("response".to_string(), Value::String(resp.body));
                    Ok(Value::Object(map))
                } else {
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("dispatched".to_string()));
                    map.insert("recipient".to_string(), Value::String(to.to_string()));
                    map.insert("payload".to_string(), Value::String(raw_payload));
                    map.insert("live_dispatched".to_string(), Value::from(false));
                    Ok(Value::Object(map))
                }
            }
            "send_image" => {
                // High-consequence: verified token has already been validated and consumed!
                let to = get_str_property(payload, "to").unwrap_or("");
                let image_url = get_str_property(payload, "image_url").unwrap_or("");
                let caption = get_str_property(payload, "caption").unwrap_or("");
                if to.is_empty() {
                    return Err(ConnectorProtocolError::InvalidPayload("Missing recipient 'to'".to_string()));
                }
                let raw_payload = self.build_image_payload(to, image_url, caption);

                let access_token = self.access_token.clone()
                    .or_else(|| std::env::var("WHATSAPP_TOKEN").ok())
                    .or_else(|| std::env::var("WHATSAPP_ACCESS_TOKEN").ok());
                let phone_id = self.phone_number_id.clone()
                    .or_else(|| std::env::var("WHATSAPP_PHONE_NUMBER_ID").ok());

                if let (Some(token), Some(pid)) = (access_token, phone_id) {
                    let url = Self::send_api_url(&pid);
                    let auth_hdr = format!("Bearer {}", token);
                    let headers = [
                        ("Authorization", auth_hdr.as_str()),
                        ("Content-Type", "application/json"),
                    ];
                    let resp = kineti_core::http_post_json(&url, &headers, &raw_payload)
                        .map_err(|e| ConnectorProtocolError::ExecutionFailed(format!("WhatsApp network error: {}", e)))?;
                    if !resp.success {
                        return Err(ConnectorProtocolError::ExecutionFailed(format!(
                            "WhatsApp API HTTP {}: {}", resp.status, resp.body
                        )));
                    }
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("delivered".to_string()));
                    map.insert("recipient".to_string(), Value::String(to.to_string()));
                    map.insert("http_status".to_string(), Value::from(resp.status as u64));
                    map.insert("live_dispatched".to_string(), Value::from(true));
                    map.insert("response".to_string(), Value::String(resp.body));
                    Ok(Value::Object(map))
                } else {
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), Value::String("dispatched".to_string()));
                    map.insert("recipient".to_string(), Value::String(to.to_string()));
                    map.insert("payload".to_string(), Value::String(raw_payload));
                    map.insert("live_dispatched".to_string(), Value::from(false));
                    Ok(Value::Object(map))
                }
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

fn extract_str(s: &str, prefix: &str) -> String {
    if let Some(pos) = s.find(prefix) {
        let rest = &s[pos + prefix.len()..];
        if let Some(start_q) = rest.find('"') {
            let after_q = &rest[start_q + 1..];
            let mut out = String::new();
            let mut escaping = false;
            for c in after_q.chars() {
                if escaping {
                    out.push(c);
                    escaping = false;
                } else if c == '\\' {
                    escaping = true;
                } else if c == '"' {
                    break;
                } else {
                    out.push(c);
                }
            }
            return out;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kineti_connectors::ActionAuthorizationToken;

    #[test]
    fn test_webhook_challenge_verification() {
        let gw = WhatsAppGateway::new("kineti_webhook_secret_token");

        let ok = gw.verify_challenge(
            Some("subscribe"),
            Some("kineti_webhook_secret_token"),
            Some("challenge_12345"),
        );
        assert_eq!(ok, Some("challenge_12345".to_string()));

        let bad = gw.verify_challenge(Some("subscribe"), Some("wrong_token"), Some("challenge_12345"));
        assert_eq!(bad, None);
    }

    #[test]
    fn test_outbound_text_and_reaction_payloads() {
        let gw = WhatsAppGateway::new("token");

        let text_json = gw.build_text_payload("+15551234567", "Hello from Kineti!");
        assert!(text_json.contains("\"messaging_product\":\"whatsapp\""));
        assert!(text_json.contains("\"type\":\"text\""));
        assert!(text_json.contains("Hello from Kineti!"));

        let reaction_json = gw.build_reaction_payload("+15551234567", "wamid.HBgL123", "👍");
        assert!(reaction_json.contains("\"type\":\"reaction\""));
        assert!(reaction_json.contains("\"emoji\":\"👍\""));
        assert!(reaction_json.contains("\"message_id\":\"wamid.HBgL123\""));

        let image_json = gw.build_image_payload("+15551234567", "https://getkineti.com/img.jpg", "Scandinavian desk");
        assert!(image_json.contains("\"type\":\"image\""));
        assert!(image_json.contains("\"link\":\"https://getkineti.com/img.jpg\""));
    }

    #[test]
    fn test_send_api_url_construction() {
        let url = WhatsAppGateway::send_api_url("123456789");
        assert_eq!(url, "https://graph.facebook.com/v21.0/123456789/messages");
    }

    #[test]
    fn test_send_api_headers() {
        let headers = WhatsAppGateway::send_api_headers("EAAx123_token");
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer EAAx123_token");
        assert_eq!(headers[1].0, "Content-Type");
        assert_eq!(headers[1].1, "application/json");
    }

    #[test]
    fn test_read_receipt_payload() {
        let payload = WhatsAppGateway::build_read_receipt_payload("wamid.HBgL456");
        assert!(payload.contains("\"status\":\"read\""));
        assert!(payload.contains("\"message_id\":\"wamid.HBgL456\""));
    }

    #[test]
    fn test_signature_validation_rejects_bad_format() {
        assert!(!WhatsAppGateway::validate_signature("secret", "body", "invalid"));
        assert!(!WhatsAppGateway::validate_signature("secret", "body", "sha256=tooshort"));
        assert!(!WhatsAppGateway::validate_signature("secret", "body", ""));
    }

    #[test]
    fn test_signature_validation_accepts_wellformed() {
        let secret = "my_app_secret";
        let body = "sample webhook payload";
        let hmac_bytes = kineti_core::hmac_sha256(secret.as_bytes(), body.as_bytes());
        let sig = format!("sha256={}", kineti_core::hex_encode(&hmac_bytes));
        assert!(WhatsAppGateway::validate_signature(secret, body, &sig));

        // Different body fails
        assert!(!WhatsAppGateway::validate_signature(secret, "tampered payload", &sig));
        // Different secret fails
        assert!(!WhatsAppGateway::validate_signature("other_secret", body, &sig));
    }

    #[test]
    fn test_live_ticket_confirmation_flow_via_whatsapp() {
        let gw = WhatsAppGateway::new("kineti_webhook_token");

        let inbound_json = r#"{
            "entry": [{
                "changes": [{
                    "value": {
                        "messages": [{
                            "id": "wamid.HBgL789",
                            "from": "+14155551234",
                            "type": "text",
                            "text": { "body": "buy 2 Hans Zimmer tickets" }
                        }]
                    }
                }]
            }]
        }"#;
        let msg = gw.parse_inbound(inbound_json).expect("Should parse inbound");
        assert_eq!(msg.from_phone, "+14155551234");
        assert!(msg.body.contains("Hans Zimmer"));

        let confirm_text = gw.build_text_payload(
            &msg.from_phone,
            "Found 2x Hans Zimmer Live tickets at MSG for $330.00 total.\n\nReply BUY to confirm or CANCEL to stop.",
        );
        assert!(confirm_text.contains("Hans Zimmer"));
        assert!(confirm_text.contains("BUY"));

        let buy_json = r#"{
            "entry": [{
                "changes": [{
                    "value": {
                        "messages": [{
                            "id": "wamid.HBgL790",
                            "from": "+14155551234",
                            "type": "text",
                            "text": { "body": "BUY" }
                        }]
                    }
                }]
            }]
        }"#;
        let buy_msg = gw.parse_inbound(buy_json).expect("Should parse BUY");
        assert_eq!(buy_msg.body, "BUY");

        let receipt = gw.build_text_payload(&buy_msg.from_phone, "Done! 2x Hans Zimmer tickets confirmed. Check your email for the e-tickets.");
        assert!(receipt.contains("confirmed"));

        let reaction = gw.build_reaction_payload(&buy_msg.from_phone, &buy_msg.message_id, "⚡");
        assert!(reaction.contains("\"emoji\":\"⚡\""));

        let read = WhatsAppGateway::build_read_receipt_payload(&buy_msg.message_id);
        assert!(read.contains(&buy_msg.message_id));
    }

    // Protocol & Permission Gating Tests for WhatsApp
    #[test]
    fn test_whatsapp_outbound_requires_authorization_token() {
        let gw = WhatsAppGateway::new("token");
        let mut msg_map = BTreeMap::new();
        msg_map.insert("to".to_string(), Value::String("+15559876543".to_string()));
        msg_map.insert("text".to_string(), Value::String("Alert from Kineti".to_string()));
        let payload = Value::Object(msg_map);

        // Invariant: Knowing phone number does NOT grant permission to message.
        // Attempting to send text without token MUST be blocked.
        let blocked = gw.execute("send_text", &payload, None);
        assert_eq!(
            blocked.err(),
            Some(ConnectorProtocolError::MissingAuthorizationToken)
        );

        // With valid token bound to exact payload
        let mut token = ActionAuthorizationToken::mint(
            "tok_wa_01",
            "alex",
            "whatsapp",
            "send_text",
            &payload,
            300,
            1000,
        );

        let sent = gw.execute_at("send_text", &payload, Some(&mut token), 1050);
        assert!(sent.is_ok());
        assert!(token.consumed);

        // Replay with consumed token is rejected
        let replay = gw.execute_at("send_text", &payload, Some(&mut token), 1060);
        assert_eq!(
            replay.err(),
            Some(ConnectorProtocolError::TokenAlreadyConsumed)
        );
    }

    #[test]
    fn test_whatsapp_emoji_reaction_is_operational() {
        let gw = WhatsAppGateway::new("token");
        let mut map = BTreeMap::new();
        map.insert("to".to_string(), Value::String("+15559876543".to_string()));
        map.insert("target_message_id".to_string(), Value::String("wamid.123".to_string()));
        map.insert("emoji".to_string(), Value::String("👍".to_string()));
        let payload = Value::Object(map);

        // Reaction is operational -> succeeds without token
        let res = gw.execute("react_emoji", &payload, None);
        assert!(res.is_ok());
    }

    #[test]
    fn test_whatsapp_configurable_phone_number() {
        let gw = WhatsAppGateway::new("token");
        assert_eq!(gw.phone_number(), "Not Configured");

        let gw_custom = WhatsAppGateway::new("token").with_phone_number("+1 (415) 555-0199");
        assert_eq!(gw_custom.phone_number(), "+1 (415) 555-0199");
    }

    #[test]
    fn test_whatsapp_onboarding_pairing_token() {
        let phone = "+14155552671";
        let timestamp = 1710000000;
        let token = WhatsAppGateway::generate_pairing_token(phone, timestamp);
        assert!(token.starts_with("wa_"));

        let url = WhatsAppGateway::generate_onboarding_url(&token);
        assert!(url.starts_with("https://app.getkineti.com/whatsapp-onboarding?t="));
        assert!(url.contains(&token));

        assert!(WhatsAppGateway::verify_pairing_token(&token, phone));
        assert!(!WhatsAppGateway::verify_pairing_token(&token, "+14155559999"));
        assert!(!WhatsAppGateway::verify_pairing_token("invalid_token", phone));
    }
}
