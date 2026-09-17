//! # Stripe Issuing & Link Autonomous Payments Engine (`stripe_issuing`)
//!
//! Autonomous generation of dynamically authorized, one-time-use virtual cards:
//! - Single-use virtual cards masked from end merchants.
//! - Enforces exact user-approved spend caps (`amount_cents <= approved_cap`).
//! - Short TTL (default 15 minutes) with auto-cancellation.
//! - SAGA LIFO reversibility for automatic cancellation on transaction abort.
//! - Governed by [`KinetiConnectorProtocol`].

use kineti_connectors::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use kineti_core::current_epoch_millis;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

/// Single-use virtual card generated for an autonomous transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualCard {
    /// Unique card identifier in Stripe Issuing (e.g. "ic_1A2B3C...").
    pub card_id: String,
    /// Associated user ID.
    pub user_id: String,
    /// Primary Account Number (masked for display, full for execution).
    pub pan: String,
    /// 3-digit security code.
    pub cvv: String,
    /// Expiration month (1-12).
    pub exp_month: u8,
    /// Expiration year (e.g. 2028).
    pub exp_year: u16,
    /// Exact hard spend limit in USD cents.
    pub spend_cap_cents: u32,
    /// Target merchant name.
    pub merchant: String,
    /// Timestamp when card was created (Unix ms).
    pub created_at_ms: u64,
    /// Timestamp when card expires (Unix ms).
    pub expires_at_ms: u64,
    /// Card status ("active", "consumed", "canceled").
    pub status: String,
}

/// Stripe Issuing & Link Virtual Card Engine.
#[derive(Debug, Default)]
pub struct StripeIssuingEngine {
    cards: RwLock<HashMap<String, VirtualCard>>,
}

impl StripeIssuingEngine {
    /// Creates a new Stripe Issuing engine.
    pub fn new() -> Self {
        Self {
            cards: RwLock::new(HashMap::new()),
        }
    }

    /// Mints a new one-time virtual card with an exact hard spend cap and 15-minute TTL.
    pub fn create_virtual_card(
        &self,
        user_id: &str,
        spend_cap_cents: u32,
        merchant: &str,
    ) -> VirtualCard {
        let now = current_epoch_millis();
        let card_id = format!("ic_vc_{}_{}", user_id, now);
        // Deterministic pseudo-card generation for safe sandbox testing
        let seed = now ^ (spend_cap_cents as u64);
        let pan = format!("40001234{:08}", seed % 100_000_000);
        let cvv = format!("{:03}", (seed % 900) + 100);

        let card = VirtualCard {
            card_id: card_id.clone(),
            user_id: user_id.to_string(),
            pan,
            cvv,
            exp_month: 12,
            exp_year: 2028,
            spend_cap_cents,
            merchant: merchant.to_string(),
            created_at_ms: now,
            expires_at_ms: now + (15 * 60 * 1000), // 15-minute TTL
            status: "active".to_string(),
        };

        let mut map = self.cards.write().unwrap();
        map.insert(card_id, card.clone());
        card
    }

    /// Authorizes a transaction against a virtual card, strictly enforcing spend cap.
    pub fn authorize_charge(
        &self,
        card_id: &str,
        attempted_cents: u32,
        now_ms: u64,
    ) -> Result<String, &'static str> {
        let mut map = self.cards.write().unwrap();
        let card = match map.get_mut(card_id) {
            Some(c) => c,
            None => return Err("Virtual card not found"),
        };

        if card.status != "active" {
            return Err("Virtual card is no longer active");
        }

        if now_ms > card.expires_at_ms {
            card.status = "expired".to_string();
            return Err("Virtual card has expired");
        }

        if attempted_cents > card.spend_cap_cents {
            return Err("Charge declined: Attempted amount exceeds approved spend cap");
        }

        card.status = "consumed".to_string();
        let tx_id = format!("ch_stripe_{}", now_ms);
        Ok(tx_id)
    }

    /// Cancels a virtual card (used by SAGA LIFO undo rollback).
    pub fn cancel_card(&self, card_id: &str) -> bool {
        let mut map = self.cards.write().unwrap();
        if let Some(card) = map.get_mut(card_id) {
            card.status = "canceled".to_string();
            true
        } else {
            false
        }
    }

    /// Retrieves an active virtual card.
    pub fn get_card(&self, card_id: &str) -> Option<VirtualCard> {
        let map = self.cards.read().unwrap();
        map.get(card_id).cloned()
    }
}

impl KinetiConnectorProtocol for StripeIssuingEngine {
    fn connector_name(&self) -> &'static str {
        "stripe_issuing"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["create_virtual_card", "authorize_charge", "cancel_card"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "cancel_card" => ConsequenceLevel::Operational,
            "create_virtual_card" | "authorize_charge" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "create_virtual_card" => {
                let user_id = get_str_property(payload, "user_id").unwrap_or("default");
                let merchant = get_str_property(payload, "merchant").unwrap_or("merchant");
                let cap_cents = if let Value::Object(m) = payload {
                    m.get("spend_cap_cents").and_then(|v| match v {
                        Value::Number(n) => n.as_str().parse::<u32>().ok(),
                        _ => None,
                    }).unwrap_or(5000)
                } else {
                    5000
                };

                let card = self.create_virtual_card(user_id, cap_cents, merchant);
                let mut map = BTreeMap::new();
                map.insert("card_id".to_string(), Value::String(card.card_id));
                map.insert("masked_pan".to_string(), Value::String(format!("•••• {}", &card.pan[card.pan.len().saturating_sub(4)..])));
                map.insert("spend_cap_cents".to_string(), Value::from(card.spend_cap_cents as u64));
                map.insert("status".to_string(), Value::String(card.status));
                map.insert("expires_at_ms".to_string(), Value::from(card.expires_at_ms));
                Ok(Value::Object(map))
            }
            "authorize_charge" => {
                let card_id = get_str_property(payload, "card_id").unwrap_or("");
                let amount_cents = if let Value::Object(m) = payload {
                    m.get("amount_cents").and_then(|v| match v {
                        Value::Number(n) => n.as_str().parse::<u32>().ok(),
                        _ => None,
                    }).unwrap_or(0)
                } else {
                    0
                };

                let now = current_epoch_millis();
                match self.authorize_charge(card_id, amount_cents, now) {
                    Ok(tx_id) => {
                        let mut map = BTreeMap::new();
                        map.insert("status".to_string(), Value::String("charge_authorized".to_string()));
                        map.insert("transaction_id".to_string(), Value::String(tx_id));
                        Ok(Value::Object(map))
                    }
                    Err(err) => Err(ConnectorProtocolError::ExecutionFailed(err.to_string())),
                }
            }
            "cancel_card" => {
                let card_id = get_str_property(payload, "card_id").unwrap_or("");
                let canceled = self.cancel_card(card_id);
                let mut map = BTreeMap::new();
                map.insert("canceled".to_string(), Value::Bool(canceled));
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
    fn test_virtual_card_generation_and_spend_cap_enforcement() {
        let engine = StripeIssuingEngine::new();
        let card = engine.create_virtual_card("user_01", 33000, "Ticketmaster"); // $330.00 cap

        assert_eq!(card.spend_cap_cents, 33000);
        assert_eq!(card.status, "active");
        assert!(card.pan.starts_with("4000"));

        let now = current_epoch_millis();

        // 1. Charge above cap ($330.01 = 33001 cents) is DECLINED
        let over_err = engine.authorize_charge(&card.card_id, 33001, now).unwrap_err();
        assert!(over_err.contains("exceeds approved spend cap"));

        // 2. Charge within cap ($330.00) SUCCEEDS
        let ok_tx = engine.authorize_charge(&card.card_id, 33000, now).expect("Charge succeeds");
        assert!(ok_tx.starts_with("ch_stripe_"));

        // 3. Second charge on consumed single-use card is DECLINED
        let replay_err = engine.authorize_charge(&card.card_id, 1000, now).unwrap_err();
        assert_eq!(replay_err, "Virtual card is no longer active");
    }

    #[test]
    fn test_virtual_card_cancellation_by_saga() {
        let engine = StripeIssuingEngine::new();
        let card = engine.create_virtual_card("user_01", 5000, "Amazon");

        let canceled = engine.cancel_card(&card.card_id);
        assert!(canceled);

        let err = engine.authorize_charge(&card.card_id, 1000, current_epoch_millis()).unwrap_err();
        assert_eq!(err, "Virtual card is no longer active");
    }
}
