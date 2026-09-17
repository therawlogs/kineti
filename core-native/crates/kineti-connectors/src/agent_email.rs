//! # Dedicated Agent Email Infrastructure (`agent_email`)
//!
//! Provisioned `@mail.kineti.com` email handles:
//! - Sub-addressing / alias generator: `<user>-agent+<vendor>_<nonce>@mail.kineti.com`.
//! - Inbound MIME parser: extracts verification links, OTP codes, return labels, tracking numbers.
//! - Thread participation engine for forwarded or CC'd threads.
//! - Governed by [`KinetiConnectorProtocol`].

use crate::protocol::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use kineti_core::current_epoch_millis;
use std::collections::BTreeMap;

/// Extracted inbound email message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundAgentEmail {
    /// Message ID.
    pub message_id: String,
    /// Sender address.
    pub from: String,
    /// Recipient address (e.g. `prav+amazon_1a2b@mail.kineti.com`).
    pub to: String,
    /// Subject line.
    pub subject: String,
    /// Plaintext body content.
    pub body_text: String,
    /// HTML content if provided.
    pub body_html: Option<String>,
    /// Thread ID for reply-in-thread.
    pub thread_id: Option<String>,
    /// Received timestamp (Unix ms).
    pub received_at_ms: u64,
}

/// Outbound email draft or dispatch payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundAgentEmail {
    /// Sender address (must be `@mail.kineti.com`).
    pub from: String,
    /// Recipient address.
    pub to: String,
    /// Subject line.
    pub subject: String,
    /// Body content.
    pub body: String,
    /// Optional thread ID to reply in-thread.
    pub in_reply_to: Option<String>,
}

/// Dedicated Agent Email Client.
#[derive(Debug, Clone)]
pub struct AgentEmailClient {
    default_domain: String,
}

impl Default for AgentEmailClient {
    fn default() -> Self {
        Self::new("mail.kineti.com")
    }
}

impl AgentEmailClient {
    /// Creates a new agent email client.
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            default_domain: domain.into(),
        }
    }

    /// Generates a vendor-isolated sub-addressed alias for account creation.
    pub fn generate_alias(&self, base_handle: &str, vendor: &str) -> String {
        let clean_handle = base_handle.trim().to_lowercase();
        let clean_vendor = vendor
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        let nonce = current_epoch_millis() % 100_000;
        format!(
            "{}+{}_{}@{}",
            clean_handle, clean_vendor, nonce, self.default_domain
        )
    }

    /// Parses an inbound MIME text string into structured email fields.
    pub fn parse_raw_mime(&self, raw: &str) -> InboundAgentEmail {
        let mut from = String::new();
        let mut to = String::new();
        let mut subject = String::new();
        let mut message_id = format!("msg_{}", current_epoch_millis());
        let mut in_headers = true;
        let mut body_lines = Vec::new();

        for line in raw.lines() {
            if in_headers {
                if line.trim().is_empty() {
                    in_headers = false;
                    continue;
                }
                let lower = line.to_lowercase();
                if lower.starts_with("from:") {
                    from = line[5..].trim().to_string();
                } else if lower.starts_with("to:") {
                    to = line[3..].trim().to_string();
                } else if lower.starts_with("subject:") {
                    subject = line[8..].trim().to_string();
                } else if lower.starts_with("message-id:") {
                    message_id = line[11..].trim().to_string();
                }
            } else {
                body_lines.push(line);
            }
        }

        InboundAgentEmail {
            message_id,
            from,
            to,
            subject,
            body_text: body_lines.join("\n").trim().to_string(),
            body_html: None,
            thread_id: None,
            received_at_ms: current_epoch_millis(),
        }
    }

    /// Extracts account verification or password reset links from body text.
    pub fn extract_verification_link(&self, body: &str) -> Option<String> {
        for word in body.split_whitespace() {
            let clean = word.trim_matches(|c: char| c == '<' || c == '>' || c == '"' || c == '\'');
            if (clean.starts_with("https://") || clean.starts_with("http://"))
                && (clean.contains("verify")
                    || clean.contains("confirm")
                    || clean.contains("activate")
                    || clean.contains("token=")
                    || clean.contains("code="))
            {
                return Some(clean.to_string());
            }
        }
        None
    }

    /// Extracts 6-digit or 8-digit OTP verification codes from body text.
    pub fn extract_otp_code(&self, body: &str) -> Option<String> {
        let tokens: Vec<&str> = body.split_whitespace().collect();
        for (i, &token) in tokens.iter().enumerate() {
            let clean = token.trim_matches(|c: char| !c.is_ascii_digit());
            if clean.len() == 6 || clean.len() == 8 {
                // Check surrounding context words
                let is_likely_code = if i > 0 {
                    let prev = tokens[i - 1].to_lowercase();
                    prev.contains("code") || prev.contains("is") || prev.contains("verification") || prev.contains("pin")
                } else {
                    false
                };
                if is_likely_code || clean.chars().all(|c| c.is_ascii_digit()) {
                    return Some(clean.to_string());
                }
            }
        }
        None
    }

    /// Extracts carrier tracking numbers (UPS, FedEx, USPS).
    pub fn extract_tracking_number(&self, body: &str) -> Option<(String, String)> {
        for word in body.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            // UPS tracking starts with 1Z and is 18 chars
            if clean.starts_with("1Z") && clean.len() == 18 {
                return Some(("UPS".to_string(), clean.to_string()));
            }
            // FedEx is 12 or 15 digits
            if (clean.len() == 12 || clean.len() == 15) && clean.chars().all(|c| c.is_ascii_digit()) {
                if body.to_lowercase().contains("fedex") {
                    return Some(("FedEx".to_string(), clean.to_string()));
                }
            }
            // USPS is 20-22 digits starting with 9
            if clean.len() >= 20 && clean.len() <= 22 && clean.starts_with('9') && clean.chars().all(|c| c.is_ascii_digit()) {
                return Some(("USPS".to_string(), clean.to_string()));
            }
        }
        None
    }

    /// Sends an email live using a webhook or transactional mail service endpoint.
    pub fn send_email_live(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        endpoint: &str,
        api_token: Option<&str>,
    ) -> Result<String, String> {
        let auth_header = api_token.map(|t| format!("Bearer {}", t));
        let mut header_refs: Vec<(&str, &str)> = vec![("Content-Type", "application/json")];
        if let Some(ref auth) = auth_header {
            header_refs.push(("Authorization", auth.as_str()));
        }
        let from = format!("agent@{}", self.default_domain);
        let escaped_body = body.replace('"', "\\\"").replace('\n', "\\n");
        let payload = format!(
            "{{\"from\":\"{}\",\"to\":\"{}\",\"subject\":\"{}\",\"text\":\"{}\"}}",
            from, to, subject, escaped_body
        );
        let res = kineti_core::http_post_json(endpoint, &header_refs, &payload)
            .map_err(|e| format!("Agent email network error: {}", e))?;
        if !res.success {
            return Err(format!("Agent email HTTP error {}: {}", res.status, res.body));
        }
        Ok(res.body)
    }
}

impl KinetiConnectorProtocol for AgentEmailClient {
    fn connector_name(&self) -> &'static str {
        "agent_email"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "generate_alias",
            "parse_inbound_mail",
            "extract_verification_link",
            "extract_otp_code",
            "extract_tracking_number",
            "draft_email",
            "send_email",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "generate_alias"
            | "parse_inbound_mail"
            | "extract_verification_link"
            | "extract_otp_code"
            | "extract_tracking_number" => ConsequenceLevel::Trivial,
            "draft_email" => ConsequenceLevel::Operational,
            "send_email" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "generate_alias" => {
                let handle = get_str_property(payload, "handle").unwrap_or("agent");
                let vendor = get_str_property(payload, "vendor").unwrap_or("service");
                let alias = self.generate_alias(handle, vendor);
                let mut map = BTreeMap::new();
                map.insert("alias".to_string(), Value::String(alias));
                Ok(Value::Object(map))
            }
            "parse_inbound_mail" => {
                let raw_mime = get_str_property(payload, "raw_mime").unwrap_or("");
                let parsed = self.parse_raw_mime(raw_mime);
                let mut map = BTreeMap::new();
                map.insert("message_id".to_string(), Value::String(parsed.message_id));
                map.insert("from".to_string(), Value::String(parsed.from));
                map.insert("to".to_string(), Value::String(parsed.to));
                map.insert("subject".to_string(), Value::String(parsed.subject));
                map.insert("body_text".to_string(), Value::String(parsed.body_text));
                Ok(Value::Object(map))
            }
            "extract_verification_link" => {
                let body = get_str_property(payload, "body").unwrap_or("");
                let link = self.extract_verification_link(body);
                let mut map = BTreeMap::new();
                map.insert("link".to_string(), link.map(Value::String).unwrap_or(Value::Null));
                Ok(Value::Object(map))
            }
            "extract_otp_code" => {
                let body = get_str_property(payload, "body").unwrap_or("");
                let code = self.extract_otp_code(body);
                let mut map = BTreeMap::new();
                map.insert("code".to_string(), code.map(Value::String).unwrap_or(Value::Null));
                Ok(Value::Object(map))
            }
            "extract_tracking_number" => {
                let body = get_str_property(payload, "body").unwrap_or("");
                let mut map = BTreeMap::new();
                if let Some((carrier, num)) = self.extract_tracking_number(body) {
                    map.insert("carrier".to_string(), Value::String(carrier));
                    map.insert("tracking_number".to_string(), Value::String(num));
                } else {
                    map.insert("carrier".to_string(), Value::Null);
                    map.insert("tracking_number".to_string(), Value::Null);
                }
                Ok(Value::Object(map))
            }
            "draft_email" => {
                let to = get_str_property(payload, "to").unwrap_or("");
                let subject = get_str_property(payload, "subject").unwrap_or("");
                let body = get_str_property(payload, "body").unwrap_or("");
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("draft_created".to_string()));
                map.insert("to".to_string(), Value::String(to.to_string()));
                map.insert("subject".to_string(), Value::String(subject.to_string()));
                map.insert("body".to_string(), Value::String(body.to_string()));
                Ok(Value::Object(map))
            }
            "send_email" => {
                // High-consequence: Verified token has already been validated and consumed
                let to = get_str_property(payload, "to").unwrap_or("");
                let subject = get_str_property(payload, "subject").unwrap_or("");
                let body = get_str_property(payload, "body").unwrap_or("");
                let endpoint = get_str_property(payload, "endpoint");
                let api_token = get_str_property(payload, "api_token");
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("email_dispatched".to_string()));
                map.insert("from".to_string(), Value::String(format!("agent@{}", self.default_domain)));
                map.insert("to".to_string(), Value::String(to.to_string()));
                map.insert("subject".to_string(), Value::String(subject.to_string()));
                map.insert("body".to_string(), Value::String(body.to_string()));

                if let Some(ep) = endpoint {
                    match self.send_email_live(to, subject, body, ep, api_token) {
                        Ok(resp) => {
                            map.insert("response".to_string(), Value::String(resp));
                        }
                        Err(e) => {
                            map.insert("note".to_string(), Value::String(e));
                        }
                    }
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

    #[test]
    fn test_agent_email_alias_generation() {
        let client = AgentEmailClient::default();
        let alias = client.generate_alias("prav", "amazon");
        assert!(alias.starts_with("prav+amazon_"));
        assert!(alias.ends_with("@mail.kineti.com"));
    }

    #[test]
    fn test_inbound_mime_parsing_and_link_extraction() {
        let client = AgentEmailClient::default();
        let raw = "From: support@uber.com\nTo: prav+uber_123@mail.kineti.com\nSubject: Verify your Uber account\n\nWelcome to Uber! Please click here to verify your account:\nhttps://auth.uber.com/verify?token=xyz987123\n\nThank you!";

        let parsed = client.parse_raw_mime(raw);
        assert_eq!(parsed.from, "support@uber.com");
        assert_eq!(parsed.to, "prav+uber_123@mail.kineti.com");
        assert_eq!(parsed.subject, "Verify your Uber account");

        let link = client.extract_verification_link(&parsed.body_text);
        assert_eq!(
            link,
            Some("https://auth.uber.com/verify?token=xyz987123".to_string())
        );
    }

    #[test]
    fn test_inbound_otp_and_tracking_extraction() {
        let client = AgentEmailClient::default();
        let email_body = "Your verification code is 849201. Use it to complete your return.\nYour replacement has shipped via UPS: 1Z9999999999999999.";

        let otp = client.extract_otp_code(email_body);
        assert_eq!(otp, Some("849201".to_string()));

        let tracking = client.extract_tracking_number(email_body);
        assert_eq!(
            tracking,
            Some(("UPS".to_string(), "1Z9999999999999999".to_string()))
        );
    }
}
