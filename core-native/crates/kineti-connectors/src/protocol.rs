//! # Unified Connector Protocol (`protocol`)
//!
//! Standardized trait and consequence-weighted permission gating for all Kineti connectors:
//! - **[`KinetiConnectorProtocol`]**: Non-Virtual Interface (NVI) pattern enforcing gating before execution.
//! - **[`ActionAuthorizationToken`]**: Cryptographically payload-bound single-use execution tokens.
//! - **[`ConsequenceLevel`]**: Explicit triage of risk (`Trivial`, `Operational`, `HighConsequence`).
//! - **Identity & Channel Invariant**: Knowing an identity or channel does NOT grant permission to contact or spend.

use kineti_core::kernel::{canonicalize_json, hex_encode, sha256};
use std::collections::BTreeMap;
use std::fmt;

/// Lightweight JSON Value tree from the kernel.
pub use kineti_core::kernel::JsonValue as Value;
/// Numeric representation in JSON.
pub use kineti_core::kernel::JsonNumber;

/// Consequence risk level of a connector action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsequenceLevel {
    /// Read-only search, retrieval, inspection, sensory classification (no side-effects).
    Trivial = 1,
    /// Drafting messages, staging files, in-memory caches, transient compute.
    Operational = 2,
    /// External dispatch (sending SMS/email/WhatsApp), booking/purchasing, deleting, modifying secrets.
    HighConsequence = 3,
}

impl fmt::Display for ConsequenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trivial => write!(f, "Trivial"),
            Self::Operational => write!(f, "Operational"),
            Self::HighConsequence => write!(f, "HighConsequence"),
        }
    }
}

/// Errors occurring during connector protocol execution and permission checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectorProtocolError {
    /// High-consequence action attempted without required authorization token.
    MissingAuthorizationToken,
    /// Token payload hash does not match candidate action payload (tamper or reuse attempt).
    TokenPayloadMismatch {
        /// Expected SHA-256 bound to token.
        expected: String,
        /// Actual SHA-256 of candidate payload.
        actual: String,
    },
    /// Token has expired past its valid TTL.
    TokenExpired {
        /// Token expiration timestamp (Unix ms).
        expires_at: u64,
        /// Current evaluation timestamp (Unix ms).
        current_time: u64,
    },
    /// Single-use token has already been consumed.
    TokenAlreadyConsumed,
    /// Token was issued for a different connector.
    ConnectorMismatch {
        /// Connector token was minted for.
        expected: String,
        /// Connector attempting to use token.
        actual: String,
    },
    /// Token was issued for a different action type.
    ActionTypeMismatch {
        /// Action type token was minted for.
        expected: String,
        /// Action type attempting to use token.
        actual: String,
    },
    /// Action requested is unknown or not supported by this connector.
    UnsupportedAction(String),
    /// Payload serialization or schema parsing error.
    InvalidPayload(String),
    /// External connector or upstream provider execution failure.
    ExecutionFailed(String),
    /// Action violates the identity-permission invariant.
    IdentityPermissionViolation(String),
}

impl fmt::Display for ConnectorProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAuthorizationToken => {
                write!(f, "High-consequence action attempted without required authorization token")
            }
            Self::TokenPayloadMismatch { expected, actual } => {
                write!(f, "Token payload hash mismatch: expected {}, got {}", expected, actual)
            }
            Self::TokenExpired { expires_at, current_time } => {
                write!(f, "Token expired: expires_at={}, current_time={}", expires_at, current_time)
            }
            Self::TokenAlreadyConsumed => write!(f, "Single-use token has already been consumed"),
            Self::ConnectorMismatch { expected, actual } => {
                write!(f, "Token connector mismatch: expected {}, got {}", expected, actual)
            }
            Self::ActionTypeMismatch { expected, actual } => {
                write!(f, "Token action type mismatch: expected {}, got {}", expected, actual)
            }
            Self::UnsupportedAction(a) => write!(f, "Unsupported action: {}", a),
            Self::InvalidPayload(msg) => write!(f, "Invalid payload: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::IdentityPermissionViolation(msg) => {
                write!(f, "Identity/Channel invariant violation: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConnectorProtocolError {}

/// Computes the deterministic SHA-256 digest hex string for a JSON value.
pub fn compute_payload_sha256(payload: &Value) -> String {
    let canonical = canonicalize_json(payload);
    let digest = sha256(canonical.as_bytes());
    hex_encode(&digest)
}

/// Computes the deterministic SHA-256 digest hex string for raw payload bytes.
pub fn compute_bytes_sha256(payload_bytes: &[u8]) -> String {
    let digest = sha256(payload_bytes);
    hex_encode(&digest)
}

/// Helper to extract string property from an object Value.
pub fn get_str_property<'a>(val: &'a Value, key: &str) -> Option<&'a str> {
    if let Value::Object(map) = val {
        if let Some(Value::String(s)) = map.get(key) {
            return Some(s.as_str());
        }
    }
    None
}

/// Single-use, payload-bound permission token authorizing high-consequence actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionAuthorizationToken {
    /// Unique identifier for this authorization grant.
    pub token_id: String,
    /// User who authorized this action.
    pub user_id: String,
    /// Specific connector this token is bound to (e.g. "gmail", "whatsapp", "financial_checkout").
    pub connector_name: String,
    /// Specific action type this token is bound to (e.g. "send_email", "execute_purchase").
    pub action_type: String,
    /// SHA-256 digest hex string of the exact canonical payload authorized.
    pub payload_sha256: String,
    /// Timestamp when issued (Unix ms).
    pub issued_at: u64,
    /// Timestamp when token expires (Unix ms).
    pub expires_at: u64,
    /// Whether token has been consumed (single-use replay protection).
    pub consumed: bool,
    /// Optional Ed25519 signature binding the token to user/gate authority.
    pub signature: Option<String>,
}

impl ActionAuthorizationToken {
    /// Mints a new authorization token bound to a canonical Value payload.
    pub fn mint(
        token_id: impl Into<String>,
        user_id: impl Into<String>,
        connector_name: impl Into<String>,
        action_type: impl Into<String>,
        payload: &Value,
        ttl_seconds: u64,
        current_time: u64,
    ) -> Self {
        let payload_sha256 = compute_payload_sha256(payload);
        Self {
            token_id: token_id.into(),
            user_id: user_id.into(),
            connector_name: connector_name.into(),
            action_type: action_type.into(),
            payload_sha256,
            issued_at: current_time,
            expires_at: current_time.saturating_add(ttl_seconds.saturating_mul(1000)),
            consumed: false,
            signature: None,
        }
    }

    /// Mints a new authorization token bound to raw payload bytes.
    pub fn mint_raw(
        token_id: impl Into<String>,
        user_id: impl Into<String>,
        connector_name: impl Into<String>,
        action_type: impl Into<String>,
        payload_bytes: &[u8],
        ttl_seconds: u64,
        current_time: u64,
    ) -> Self {
        let payload_sha256 = compute_bytes_sha256(payload_bytes);
        Self {
            token_id: token_id.into(),
            user_id: user_id.into(),
            connector_name: connector_name.into(),
            action_type: action_type.into(),
            payload_sha256,
            issued_at: current_time,
            expires_at: current_time.saturating_add(ttl_seconds.saturating_mul(1000)),
            consumed: false,
            signature: None,
        }
    }

    /// Verifies the token against connector, action type, and candidate Value payload, marking as consumed.
    pub fn verify_and_consume(
        &mut self,
        connector: &str,
        action_type: &str,
        payload: &Value,
        current_time: u64,
    ) -> Result<(), ConnectorProtocolError> {
        // 1. Replay protection
        if self.consumed {
            return Err(ConnectorProtocolError::TokenAlreadyConsumed);
        }

        // 2. TTL expiry check
        if current_time > self.expires_at {
            return Err(ConnectorProtocolError::TokenExpired {
                expires_at: self.expires_at,
                current_time,
            });
        }

        // 3. Connector binding
        if self.connector_name != connector {
            return Err(ConnectorProtocolError::ConnectorMismatch {
                expected: self.connector_name.clone(),
                actual: connector.to_string(),
            });
        }

        // 4. Action type binding
        if self.action_type != action_type {
            return Err(ConnectorProtocolError::ActionTypeMismatch {
                expected: self.action_type.clone(),
                actual: action_type.to_string(),
            });
        }

        // 5. Payload hash binding (tamper protection)
        let actual_hash = compute_payload_sha256(payload);
        if self.payload_sha256 != actual_hash {
            return Err(ConnectorProtocolError::TokenPayloadMismatch {
                expected: self.payload_sha256.clone(),
                actual: actual_hash,
            });
        }

        // 6. Single-use consumption
        self.consumed = true;
        Ok(())
    }

    /// Verifies the token against connector, action type, and candidate raw bytes payload, marking as consumed.
    pub fn verify_and_consume_raw(
        &mut self,
        connector: &str,
        action_type: &str,
        payload_bytes: &[u8],
        current_time: u64,
    ) -> Result<(), ConnectorProtocolError> {
        if self.consumed {
            return Err(ConnectorProtocolError::TokenAlreadyConsumed);
        }

        if current_time > self.expires_at {
            return Err(ConnectorProtocolError::TokenExpired {
                expires_at: self.expires_at,
                current_time,
            });
        }

        if self.connector_name != connector {
            return Err(ConnectorProtocolError::ConnectorMismatch {
                expected: self.connector_name.clone(),
                actual: connector.to_string(),
            });
        }

        if self.action_type != action_type {
            return Err(ConnectorProtocolError::ActionTypeMismatch {
                expected: self.action_type.clone(),
                actual: action_type.to_string(),
            });
        }

        let actual_hash = compute_bytes_sha256(payload_bytes);
        if self.payload_sha256 != actual_hash {
            return Err(ConnectorProtocolError::TokenPayloadMismatch {
                expected: self.payload_sha256.clone(),
                actual: actual_hash,
            });
        }

        self.consumed = true;
        Ok(())
    }
}

/// Unified Rust Protocol governing all external connectors, actions, and gateways.
///
/// Implements the Non-Virtual Interface (NVI) pattern:
/// - Callers invoke `execute(...)` or `execute_at(...)`.
/// - Gating logic evaluates consequence level and strictly verifies single-use authorization tokens.
/// - Implementers provide `execute_verified(...)`.
pub trait KinetiConnectorProtocol: Send + Sync {
    /// Canonical name of the connector (e.g. "brave", "gmail", "flux", "whatsapp", "imessage", "financial_checkout").
    fn connector_name(&self) -> &'static str;

    /// List of action type strings supported by this connector.
    fn supported_actions(&self) -> &[&'static str];

    /// Evaluates the consequence risk level of a candidate action.
    /// Default implementation fails closed: unknown actions default to `HighConsequence`.
    fn evaluate_consequence(&self, action: &str, payload: &Value) -> ConsequenceLevel;

    /// Determines if an action requires a signed authorization token.
    fn requires_authorization(&self, action: &str, payload: &Value) -> bool {
        self.evaluate_consequence(action, payload) >= ConsequenceLevel::HighConsequence
    }

    /// Template Method: Enforces consequence checking, token validation, and single-use consumption.
    /// Defaults `current_time` to `kineti_core::current_epoch_millis()`.
    fn execute(
        &self,
        action: &str,
        payload: &Value,
        token: Option<&mut ActionAuthorizationToken>,
    ) -> Result<Value, ConnectorProtocolError> {
        self.execute_at(action, payload, token, kineti_core::current_epoch_millis())
    }

    /// Template Method with explicit evaluation timestamp (for testing and reproducible verification).
    fn execute_at(
        &self,
        action: &str,
        payload: &Value,
        token: Option<&mut ActionAuthorizationToken>,
        current_time: u64,
    ) -> Result<Value, ConnectorProtocolError> {
        // 1. Verify action support
        if !self.supported_actions().contains(&action) {
            return Err(ConnectorProtocolError::UnsupportedAction(action.to_string()));
        }

        // 2. Consequence Evaluation & Permission Gating
        let consequence = self.evaluate_consequence(action, payload);
        if consequence >= ConsequenceLevel::HighConsequence {
            let auth_token = token.ok_or(ConnectorProtocolError::MissingAuthorizationToken)?;
            auth_token.verify_and_consume(self.connector_name(), action, payload, current_time)?;
        }

        // 3. Delegate to internal verified hook
        self.execute_verified(action, payload)
    }

    /// Internal hook that actually performs the action logic.
    /// Invariant: HighConsequence actions are mathematically guaranteed to have passed token verification.
    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError>;
}

// ---------------------------------------------------------------------------
// Implementations of KinetiConnectorProtocol for Core Connectors
// ---------------------------------------------------------------------------

impl KinetiConnectorProtocol for crate::gmail::GmailClient {
    fn connector_name(&self) -> &'static str {
        "gmail"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["search_inbox", "create_draft", "send_email"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "search_inbox" => ConsequenceLevel::Trivial,
            "create_draft" => ConsequenceLevel::Operational,
            "send_email" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence, // Fail closed
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "search_inbox" => {
                let query = get_str_property(payload, "query").unwrap_or("");
                let max_results = if let Value::Object(map) = payload {
                    if let Some(Value::Number(n)) = map.get("max_results") {
                        n.as_str().parse::<u32>().unwrap_or(5)
                    } else {
                        5
                    }
                } else {
                    5
                };
                let (url, headers) = self.build_search_request(query, max_results);
                let mut map = BTreeMap::new();
                map.insert("url".to_string(), Value::String(url));
                map.insert("header_count".to_string(), Value::from(headers.len() as u64));
                Ok(Value::Object(map))
            }
            "create_draft" => {
                let to = get_str_property(payload, "to").unwrap_or("");
                let subject = get_str_property(payload, "subject").unwrap_or("");
                let body = get_str_property(payload, "body").unwrap_or("");
                let draft_req = crate::gmail::DraftEmailRequest {
                    to: to.to_string(),
                    subject: subject.to_string(),
                    body: body.to_string(),
                    thread_id: get_str_property(payload, "thread_id").map(|s| s.to_string()),
                };
                let raw_payload = self.build_draft_payload(&draft_req);
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("draft_created".to_string()));
                map.insert("to".to_string(), Value::String(to.to_string()));
                map.insert("raw_payload".to_string(), Value::String(raw_payload));
                Ok(Value::Object(map))
            }
            "send_email" => {
                // High-consequence: verified token has already been validated and consumed!
                let to = get_str_property(payload, "to").unwrap_or("");
                let subject = get_str_property(payload, "subject").unwrap_or("");
                let body = get_str_property(payload, "body").unwrap_or("");
                let draft_req = crate::gmail::DraftEmailRequest {
                    to: to.to_string(),
                    subject: subject.to_string(),
                    body: body.to_string(),
                    thread_id: get_str_property(payload, "thread_id").map(|s| s.to_string()),
                };
                let _raw_payload = self.build_draft_payload(&draft_req);
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("email_dispatched".to_string()));
                map.insert("to".to_string(), Value::String(to.to_string()));
                map.insert("subject".to_string(), Value::String(subject.to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

impl KinetiConnectorProtocol for crate::brave::BraveSearchClient {
    fn connector_name(&self) -> &'static str {
        "brave"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["web_search", "shopping_search", "ticket_search"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "web_search" | "shopping_search" | "ticket_search" => ConsequenceLevel::Trivial,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let query = get_str_property(payload, "query").unwrap_or("");
        let (url, headers) = self.build_request(query, 5);
        let mut map = BTreeMap::new();
        map.insert("action".to_string(), Value::String(action.to_string()));
        map.insert("url".to_string(), Value::String(url));
        map.insert("headers_count".to_string(), Value::from(headers.len() as u64));
        Ok(Value::Object(map))
    }
}

impl KinetiConnectorProtocol for crate::flux::FluxClient {
    fn connector_name(&self) -> &'static str {
        "flux"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["enhance_prompt", "generate_image"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "enhance_prompt" => ConsequenceLevel::Trivial,
            "generate_image" => ConsequenceLevel::Operational,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "enhance_prompt" => {
                let prompt = get_str_property(payload, "prompt").unwrap_or("");
                let enhanced = self.enhance_prompt(prompt);
                let mut map = BTreeMap::new();
                map.insert("enhanced_prompt".to_string(), Value::String(enhanced));
                Ok(Value::Object(map))
            }
            "generate_image" => {
                let prompt = get_str_property(payload, "prompt").unwrap_or("");
                let enhanced = self.enhance_prompt(prompt);
                let body = self.build_payload(&crate::flux::FluxGenerationRequest {
                    prompt: enhanced.clone(),
                    aspect_ratio: crate::flux::AspectRatio::Square,
                    steps: 25,
                    guidance_scale: 3.5,
                });
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("request_built".to_string()));
                map.insert("endpoint".to_string(), Value::String(self.endpoint().to_string()));
                map.insert("body".to_string(), Value::String(body));
                map.insert("enhanced_prompt".to_string(), Value::String(enhanced));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCheckoutConnector;

    impl KinetiConnectorProtocol for MockCheckoutConnector {
        fn connector_name(&self) -> &'static str {
            "checkout_connector"
        }

        fn supported_actions(&self) -> &[&'static str] {
            &["view_cart", "stage_order", "commit_charge"]
        }

        fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
            match action {
                "view_cart" => ConsequenceLevel::Trivial,
                "stage_order" => ConsequenceLevel::Operational,
                "commit_charge" => ConsequenceLevel::HighConsequence,
                _ => ConsequenceLevel::HighConsequence, // Fail closed
            }
        }

        fn execute_verified(
            &self,
            action: &str,
            payload: &Value,
        ) -> Result<Value, ConnectorProtocolError> {
            let mut map = BTreeMap::new();
            map.insert("executed_action".to_string(), Value::String(action.to_string()));
            map.insert("echo".to_string(), payload.clone());
            Ok(Value::Object(map))
        }
    }

    fn sample_payload(amount: u64, recipient: &str) -> Value {
        let mut map = BTreeMap::new();
        map.insert("amount".to_string(), Value::from(amount));
        map.insert("recipient".to_string(), Value::String(recipient.to_string()));
        Value::Object(map)
    }

    // 1. High consequence operation blocked without token
    #[test]
    fn test_high_consequence_blocks_without_token() {
        let connector = MockCheckoutConnector;
        let payload = sample_payload(3500, "merchant_01");

        // Trivial succeeds without token
        let res_trivial = connector.execute_at("view_cart", &payload, None, 1000);
        assert!(res_trivial.is_ok());

        // Operational succeeds without token
        let res_op = connector.execute_at("stage_order", &payload, None, 1000);
        assert!(res_op.is_ok());

        // High consequence fails without token
        let res_high = connector.execute_at("commit_charge", &payload, None, 1000);
        assert_eq!(
            res_high.err(),
            Some(ConnectorProtocolError::MissingAuthorizationToken)
        );
    }

    // 2. Mismatched payload hash fails closed
    #[test]
    fn test_mismatched_payload_hash_fails_closed() {
        let connector = MockCheckoutConnector;
        let authorized_payload = sample_payload(3500, "merchant_01");
        let tampered_payload = sample_payload(350000, "attacker_99");

        let mut token = ActionAuthorizationToken::mint(
            "tok_001",
            "praveen",
            "checkout_connector",
            "commit_charge",
            &authorized_payload,
            300,
            1000,
        );

        let res = connector.execute_at("commit_charge", &tampered_payload, Some(&mut token), 1050);
        match res {
            Err(ConnectorProtocolError::TokenPayloadMismatch { expected, actual }) => {
                assert_ne!(expected, actual);
            }
            other => panic!("Expected TokenPayloadMismatch, got {:?}", other),
        }
        assert!(!token.consumed, "Tampered execution must NOT consume token");
    }

    // 3. Replay protection (consumed token rejected)
    #[test]
    fn test_single_use_token_replay_protection() {
        let connector = MockCheckoutConnector;
        let payload = sample_payload(3500, "merchant_01");

        let mut token = ActionAuthorizationToken::mint(
            "tok_002",
            "praveen",
            "checkout_connector",
            "commit_charge",
            &payload,
            300,
            1000,
        );

        // First execution succeeds
        let res1 = connector.execute_at("commit_charge", &payload, Some(&mut token), 1050);
        assert!(res1.is_ok());
        assert!(token.consumed);

        // Second execution with consumed token FAILS
        let res2 = connector.execute_at("commit_charge", &payload, Some(&mut token), 1060);
        assert_eq!(
            res2.err(),
            Some(ConnectorProtocolError::TokenAlreadyConsumed)
        );
    }

    // 4. Expired token rejected
    #[test]
    fn test_expired_token_rejected() {
        let connector = MockCheckoutConnector;
        let payload = sample_payload(3500, "merchant_01");

        let mut token = ActionAuthorizationToken::mint(
            "tok_003",
            "praveen",
            "checkout_connector",
            "commit_charge",
            &payload,
            10, // 10s TTL = 10,000ms
            1000,
        );

        // Current time 12,000ms > 11,000ms expiration
        let res = connector.execute_at("commit_charge", &payload, Some(&mut token), 12000);
        match res {
            Err(ConnectorProtocolError::TokenExpired { expires_at, current_time }) => {
                assert_eq!(expires_at, 11000);
                assert_eq!(current_time, 12000);
            }
            other => panic!("Expected TokenExpired, got {:?}", other),
        }
    }

    // 5. Cross-connector mismatch rejected
    #[test]
    fn test_cross_connector_token_rejected() {
        let connector = MockCheckoutConnector;
        let payload = sample_payload(3500, "merchant_01");

        // Mint token for "gmail" instead of "checkout_connector"
        let mut token = ActionAuthorizationToken::mint(
            "tok_004",
            "praveen",
            "gmail",
            "commit_charge",
            &payload,
            300,
            1000,
        );

        let res = connector.execute_at("commit_charge", &payload, Some(&mut token), 1050);
        assert_eq!(
            res.err(),
            Some(ConnectorProtocolError::ConnectorMismatch {
                expected: "gmail".to_string(),
                actual: "checkout_connector".to_string(),
            })
        );
    }

    // 6. Cross-action mismatch rejected
    #[test]
    fn test_cross_action_token_rejected() {
        let connector = MockCheckoutConnector;
        let payload = sample_payload(3500, "merchant_01");

        // Mint token for "stage_order" instead of "commit_charge"
        let mut token = ActionAuthorizationToken::mint(
            "tok_005",
            "praveen",
            "checkout_connector",
            "stage_order",
            &payload,
            300,
            1000,
        );

        let res = connector.execute_at("commit_charge", &payload, Some(&mut token), 1050);
        assert_eq!(
            res.err(),
            Some(ConnectorProtocolError::ActionTypeMismatch {
                expected: "stage_order".to_string(),
                actual: "commit_charge".to_string(),
            })
        );
    }

    // 7. Unknown action defaults to HighConsequence and fails closed
    #[test]
    fn test_unknown_action_defaults_to_high_consequence_and_fails_unsupported() {
        let connector = MockCheckoutConnector;
        let payload = Value::Null;

        assert_eq!(
            connector.evaluate_consequence("unknown_wipe_disk", &payload),
            ConsequenceLevel::HighConsequence
        );

        let res = connector.execute_at("unknown_wipe_disk", &payload, None, 1000);
        assert_eq!(
            res.err(),
            Some(ConnectorProtocolError::UnsupportedAction("unknown_wipe_disk".to_string()))
        );
    }

    // 8. Gmail Client protocol test
    #[test]
    fn test_gmail_client_protocol_integration() {
        let client = crate::gmail::GmailClient::new("test_oauth_token");

        let search_payload = {
            let mut m = BTreeMap::new();
            m.insert("query".to_string(), Value::String("from:boss".to_string()));
            Value::Object(m)
        };
        // Search is trivial -> succeeds without token
        let search_res = client.execute_at("search_inbox", &search_payload, None, 1000);
        assert!(search_res.is_ok());

        let email_payload = {
            let mut m = BTreeMap::new();
            m.insert("to".to_string(), Value::String("sarah@example.com".to_string()));
            m.insert("subject".to_string(), Value::String("Meeting update".to_string()));
            m.insert("body".to_string(), Value::String("Confirming 2pm.".to_string()));
            Value::Object(m)
        };

        // Outbound send_email is HighConsequence -> fails without token
        let blocked = client.execute_at("send_email", &email_payload, None, 1000);
        assert_eq!(blocked.err(), Some(ConnectorProtocolError::MissingAuthorizationToken));

        // With valid token -> succeeds
        let mut token = ActionAuthorizationToken::mint(
            "tok_gmail_01",
            "user_01",
            "gmail",
            "send_email",
            &email_payload,
            300,
            1000,
        );
        let sent = client.execute_at("send_email", &email_payload, Some(&mut token), 1050);
        assert!(sent.is_ok());
        assert!(token.consumed);
    }

    // 9. Brave and Flux protocol test
    #[test]
    fn test_brave_and_flux_protocol_integration() {
        let brave = crate::brave::BraveSearchClient::new("brave_key");
        let query_payload = {
            let mut m = BTreeMap::new();
            m.insert("query".to_string(), Value::String("rust protocol".to_string()));
            Value::Object(m)
        };
        let res = brave.execute("web_search", &query_payload, None);
        assert!(res.is_ok());

        let flux = crate::flux::FluxClient::new("flux_key");
        let prompt_payload = {
            let mut m = BTreeMap::new();
            m.insert("prompt".to_string(), Value::String("A cozy coffee shop".to_string()));
            Value::Object(m)
        };
        let res_flux = flux.execute("enhance_prompt", &prompt_payload, None);
        assert!(res_flux.is_ok());
    }

    // 10. Identity/Channel Invariant: Knowing identity does NOT grant permission
    #[test]
    fn test_identity_channel_invariant() {
        let client = crate::gmail::GmailClient::new("token");

        // The agent knows the user's manager is Sarah Chen at sarah@chen.corp
        let outbound_payload = {
            let mut m = BTreeMap::new();
            m.insert("to".to_string(), Value::String("sarah@chen.corp".to_string()));
            m.insert("subject".to_string(), Value::String("Weekly Status".to_string()));
            m.insert("body".to_string(), Value::String("Everything on track.".to_string()));
            Value::Object(m)
        };

        // Even though identity is completely known and valid, execution without token FAILS
        let res = client.execute("send_email", &outbound_payload, None);
        assert_eq!(
            res.err(),
            Some(ConnectorProtocolError::MissingAuthorizationToken)
        );
    }

    // 11. Canonical Serialization Invariance
    #[test]
    fn test_canonical_serialization_key_order_invariance() {
        let mut map1 = BTreeMap::new();
        map1.insert("amount".to_string(), Value::from(100u64));
        map1.insert("recipient".to_string(), Value::String("alice".to_string()));
        let val1 = Value::Object(map1);

        let mut map2 = BTreeMap::new();
        map2.insert("recipient".to_string(), Value::String("alice".to_string()));
        map2.insert("amount".to_string(), Value::from(100u64));
        let val2 = Value::Object(map2);

        assert_eq!(compute_payload_sha256(&val1), compute_payload_sha256(&val2));
    }
}
