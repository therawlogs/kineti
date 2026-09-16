//! Two-Step Confirmation Safety Gate for Financial Actions.
//!
//! Enforces that Kineti NEVER charges a credit card or executes a real-world
//! financial transaction without explicit, interactive confirmation in chat.

use kineti_core::current_epoch_millis;
use std::collections::HashMap;
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

/// Confirmation gate decision.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationDecision {
    /// Confirmed by user -> execute purchase.
    Confirmed(PendingFinancialAction),
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
        let action = PendingFinancialAction {
            id: action_id.to_string(),
            user_id: user_id.to_string(),
            description: description.to_string(),
            amount_cents,
            created_at_ms: current_epoch_millis(),
            merchant: merchant.to_string(),
        };

        let mut map = self.pending_actions.write().unwrap();
        map.insert(user_id.to_string(), action);
    }

    /// Evaluates an inbound user message against any pending financial action.
    pub fn evaluate_reply(&self, user_id: &str, reply_text: &str) -> ConfirmationDecision {
        let mut map = self.pending_actions.write().unwrap();
        let action = match map.remove(user_id) {
            Some(a) => a,
            None => return ConfirmationDecision::NoPendingAction,
        };

        let now = current_epoch_millis();
        let ten_minutes_ms = 10 * 60 * 1000;
        if now.saturating_sub(action.created_at_ms) > ten_minutes_ms {
            return ConfirmationDecision::Expired;
        }

        let clean = reply_text.trim().to_uppercase();
        if clean == "BUY" || clean == "CONFIRM" || clean == "YES" || clean == "PROCEED" {
            ConfirmationDecision::Confirmed(action)
        } else if clean == "CANCEL" || clean == "NO" || clean == "STOP" || clean == "ABORT" {
            ConfirmationDecision::Cancelled(action)
        } else {
            // Unrecognized reply -> restore pending action so user can still confirm or cancel
            map.insert(user_id.to_string(), action);
            ConfirmationDecision::NoPendingAction
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
        if let ConfirmationDecision::Confirmed(action) = decision {
            assert_eq!(action.id, "act_123");
            assert_eq!(action.amount_cents, 33000);
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
}
