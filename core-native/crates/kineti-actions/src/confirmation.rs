//! Two-Step Confirmation Safety Gate for Financial Actions.
//!
//! Enforces that Kineti NEVER charges a credit card or executes a real-world
//! financial transaction without explicit, interactive confirmation in chat.
//! Upon user confirmation (`BUY`, `CONFIRM`, etc.), mints and cryptographically
//! binds an [`ActionAuthorizationToken`] to the exact canonical transaction payload.

use kineti_connectors::{
    ActionAuthorizationToken, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol,
    Value,
};
use kineti_core::current_epoch_millis;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

/// Status of a pending financial action.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFinancialAction {
    /// Unique action ID.
    pub id: String,
    /// User requesting the action.
    pub user_id: String,
    /// Action description (e.g. "Buy 2 Tickets for Hans Zimmer Live").
    pub description: String,
    /// Total amount in USD cents.
    pub amount_cents: u32,
    /// Creation timestamp (Unix ms).
    pub created_at_ms: u64,
    /// Target merchant.
    pub merchant: String,
}

impl PendingFinancialAction {
    /// Constructs the canonical JSON payload representing this financial action.
    pub fn to_canonical_payload(&self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("action".to_string(), Value::String("execute_purchase".to_string()));
        map.insert("amount_cents".to_string(), Value::from(self.amount_cents as u64));
        map.insert("description".to_string(), Value::String(self.description.clone()));
        map.insert("id".to_string(), Value::String(self.id.clone()));
        map.insert("merchant".to_string(), Value::String(self.merchant.clone()));
        map.insert("user_id".to_string(), Value::String(self.user_id.clone()));
        Value::Object(map)
    }
}

/// Confirmation gate decision.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationDecision {
    /// Confirmed by user -> yields the confirmed action and the cryptographically bound single-use authorization token.
    Confirmed {
        /// Confirmed pending financial action.
        action: PendingFinancialAction,
        /// Cryptographically payload-bound single-use authorization token.
        token: ActionAuthorizationToken,
    },
    /// Cancelled or rejected by user.
    Cancelled(PendingFinancialAction),
    /// Confirmation timed out (after 10 minutes).
    Expired,
    /// No pending action found for user.
    NoPendingAction,
}

/// Action Confirmation Safety Gate.
#[derive(Debug, Default)]
pub struct ActionConfirmationGate {
    pending_actions: RwLock<HashMap<String, PendingFinancialAction>>,
}

impl ActionConfirmationGate {
    /// Creates a new confirmation gate.
    pub fn new() -> Self {
        Self {
            pending_actions: RwLock::new(HashMap::new()),
        }
    }

    /// Proposes a financial action and stores it awaiting confirmation (valid for 10 minutes).
    pub fn propose_action(
        &self,
        user_id: &str,
        action_id: &str,
        description: &str,
        amount_cents: u32,
        merchant: &str,
    ) {
        self.propose_action_at(user_id, action_id, description, amount_cents, merchant, current_epoch_millis());
    }

    /// Proposes a financial action with an explicit timestamp.
    pub fn propose_action_at(
        &self,
        user_id: &str,
        action_id: &str,
        description: &str,
        amount_cents: u32,
        merchant: &str,
        now: u64,
    ) {
        let action = PendingFinancialAction {
            id: action_id.to_string(),
            user_id: user_id.to_string(),
            description: description.to_string(),
            amount_cents,
            created_at_ms: now,
            merchant: merchant.to_string(),
        };

        let mut map = self.pending_actions.write().unwrap();
        map.insert(user_id.to_string(), action);
    }

    /// Evaluates an inbound user message against any pending financial action.
    pub fn evaluate_reply(&self, user_id: &str, reply_text: &str) -> ConfirmationDecision {
        self.evaluate_reply_at(user_id, reply_text, current_epoch_millis())
    }

    /// Evaluates an inbound user message against any pending financial action with an explicit timestamp.
    pub fn evaluate_reply_at(
        &self,
        user_id: &str,
        reply_text: &str,
        now: u64,
    ) -> ConfirmationDecision {
        let mut map = self.pending_actions.write().unwrap();
        let action = match map.remove(user_id) {
            Some(a) => a,
            None => return ConfirmationDecision::NoPendingAction,
        };

        let ten_minutes_ms = 10 * 60 * 1000;
        if now.saturating_sub(action.created_at_ms) > ten_minutes_ms {
            return ConfirmationDecision::Expired;
        }

        let clean = reply_text.trim().to_uppercase();
        if clean == "BUY" || clean == "CONFIRM" || clean == "YES" || clean == "PROCEED" {
            let payload = action.to_canonical_payload();
            let token = ActionAuthorizationToken::mint(
                format!("auth_tok_{}", action.id),
                &action.user_id,
                "financial_checkout",
                "execute_purchase",
                &payload,
                300, // 5-minute TTL
                now,
            );
            ConfirmationDecision::Confirmed { action, token }
        } else if clean == "CANCEL" || clean == "NO" || clean == "STOP" || clean == "ABORT" {
            ConfirmationDecision::Cancelled(action)
        } else {
            // Unrecognized reply -> restore pending action so user can still confirm or cancel
            map.insert(user_id.to_string(), action);
            ConfirmationDecision::NoPendingAction
        }
    }
}

/// Protocolized Financial Checkout Connector.
///
/// Implements [`KinetiConnectorProtocol`] to ensure purchases cannot execute
/// without an [`ActionAuthorizationToken`] cryptographically bound to the canonical payload.
#[derive(Debug, Default, Clone)]
pub struct FinancialCheckoutConnector;

impl FinancialCheckoutConnector {
    /// Creates a new checkout connector.
    pub fn new() -> Self {
        Self
    }
}

impl KinetiConnectorProtocol for FinancialCheckoutConnector {
    fn connector_name(&self) -> &'static str {
        "financial_checkout"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["get_quote", "stage_order", "execute_purchase"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "get_quote" => ConsequenceLevel::Trivial,
            "stage_order" => ConsequenceLevel::Operational,
            "execute_purchase" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence, // Fail closed
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "get_quote" => {
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("quoted".to_string()));
                map.insert("valid_ms".to_string(), Value::from(60_000u64));
                Ok(Value::Object(map))
            }
            "stage_order" => {
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("staged".to_string()));
                map.insert("order_ref".to_string(), Value::String(format!("stg_{}", current_epoch_millis())));
                Ok(Value::Object(map))
            }
            "execute_purchase" => {
                // High-consequence: verified token has already been validated and consumed!
                let mut map = BTreeMap::new();
                map.insert("status".to_string(), Value::String("purchase_executed".to_string()));
                map.insert(
                    "transaction_id".to_string(),
                    Value::String(format!("tx_{}", current_epoch_millis())),
                );
                map.insert("payload".to_string(), payload.clone());
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
    fn test_confirmation_gate_lifecycle_confirm() {
        let gate = ActionConfirmationGate::new();
        gate.propose_action(
            "user_01",
            "act_123",
            "Hans Zimmer 2x Tickets",
            33000,
            "Madison Square Garden",
        );

        // User confirms with "BUY"
        let decision = gate.evaluate_reply("user_01", "BUY");
        if let ConfirmationDecision::Confirmed { action, mut token } = decision {
            assert_eq!(action.id, "act_123");
            assert_eq!(action.amount_cents, 33000);
            assert_eq!(token.connector_name, "financial_checkout");
            assert_eq!(token.action_type, "execute_purchase");
            assert!(!token.consumed);

            // Execute purchase with the minted token
            let connector = FinancialCheckoutConnector::new();
            let payload = action.to_canonical_payload();
            let res = connector.execute("execute_purchase", &payload, Some(&mut token));
            assert!(res.is_ok());
            assert!(token.consumed);

            // Replay with the consumed token FAILS
            let replay = connector.execute("execute_purchase", &payload, Some(&mut token));
            assert_eq!(
                replay.err(),
                Some(ConnectorProtocolError::TokenAlreadyConsumed)
            );
        } else {
            panic!("Expected Confirmed decision");
        }

        // Subsequent reply has no pending action
        assert_eq!(
            gate.evaluate_reply("user_01", "BUY"),
            ConfirmationDecision::NoPendingAction
        );
    }

    #[test]
    fn test_confirmation_gate_cancellation() {
        let gate = ActionConfirmationGate::new();
        gate.propose_action("user_01", "act_456", "Sony Headphones", 32800, "Amazon");

        let decision = gate.evaluate_reply("user_01", "cancel");
        assert!(matches!(decision, ConfirmationDecision::Cancelled(_)));
    }

    #[test]
    fn test_financial_checkout_tampered_payload_rejected() {
        let gate = ActionConfirmationGate::new();
        gate.propose_action("user_01", "act_789", "Sony Headphones", 32800, "Amazon");

        let decision = gate.evaluate_reply("user_01", "BUY");
        if let ConfirmationDecision::Confirmed { action, mut token } = decision {
            let mut tampered_payload = action.to_canonical_payload();
            if let Value::Object(ref mut map) = tampered_payload {
                // Malicious tampering: increase amount to $3,280.00
                map.insert("amount_cents".to_string(), Value::from(328000u64));
            }

            let connector = FinancialCheckoutConnector::new();
            let res = connector.execute("execute_purchase", &tampered_payload, Some(&mut token));
            match res {
                Err(ConnectorProtocolError::TokenPayloadMismatch { expected, actual }) => {
                    assert_ne!(expected, actual);
                }
                other => panic!("Expected TokenPayloadMismatch, got {:?}", other),
            }
            assert!(!token.consumed);
        } else {
            panic!("Expected Confirmed decision");
        }
    }
}
