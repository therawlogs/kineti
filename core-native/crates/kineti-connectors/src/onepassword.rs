//! # 1Password Credential Brokering Connector (`onepassword`)
//!
//! Provides zero-disk in-memory credential brokering through 1Password:
//! - Category classification (`LoyaltyAccounts`, `SubscriptionPortals`, `HealthcareDashboards`, `TaxPayrollPlatforms`).
//! - High-consequence safety gating for sensitive platforms (Healthcare, Tax/Payroll).
//! - Ephemeral checkout: credentials expire after short TTL and are never written to disk.
//! - Outcome Verification Ticket (OVT) audit receipt generation.
//! - Implements [`KinetiConnectorProtocol`].

use crate::protocol::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use kineti_core::current_epoch_millis;
use kineti_core::kernel::{blake3, hex_encode};
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

/// Sensitivity category of 1Password credential item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnePasswordCategory {
    /// Airline, hotel, car rental loyalty portals.
    LoyaltyAccounts,
    /// SaaS subscriptions, streaming services, e-commerce logins.
    SubscriptionPortals,
    /// Patient portals, pharmacy logins, health insurance dashboards.
    HealthcareDashboards,
    /// IRS, Gusto, ADP, TurboTax, payroll platforms.
    TaxPayrollPlatforms,
    /// General web logins.
    General,
}

impl OnePasswordCategory {
    /// Determines the consequence risk level for this category.
    pub fn consequence_level(&self) -> ConsequenceLevel {
        match self {
            Self::LoyaltyAccounts | Self::SubscriptionPortals => ConsequenceLevel::Operational,
            Self::HealthcareDashboards | Self::TaxPayrollPlatforms => ConsequenceLevel::HighConsequence,
            Self::General => ConsequenceLevel::Operational,
        }
    }

    /// Parses category string.
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("loyalty") || lower.contains("airline") || lower.contains("hotel") {
            Self::LoyaltyAccounts
        } else if lower.contains("sub") || lower.contains("streaming") || lower.contains("saas") {
            Self::SubscriptionPortals
        } else if lower.contains("health") || lower.contains("medical") || lower.contains("pharmacy") {
            Self::HealthcareDashboards
        } else if lower.contains("tax") || lower.contains("payroll") || lower.contains("irs") || lower.contains("gusto") {
            Self::TaxPayrollPlatforms
        } else {
            Self::General
        }
    }
}

/// Ephemeral in-memory credential item checked out from 1Password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnePasswordItem {
    /// Item UUID in 1Password.
    pub item_id: String,
    /// Title / label (e.g. "Delta SkyMiles", "Kaiser Permanente").
    pub title: String,
    /// Category classification.
    pub category: OnePasswordCategory,
    /// Username or account ID.
    pub username: String,
    /// Secret password or token (ephemeral).
    pub secret: String,
    /// Checkout timestamp (Unix ms).
    pub checked_out_at: u64,
    /// Expiration timestamp (Unix ms).
    pub expires_at: u64,
}

/// Audit receipt generated upon checking out credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnePasswordAuditReceipt {
    /// SHA-256 / BLAKE3 hash of item ID.
    pub item_id_hash: String,
    /// Requesting user.
    pub user_id: String,
    /// Category accessed.
    pub category: String,
    /// Consequence level evaluated.
    pub consequence: ConsequenceLevel,
    /// Timestamp (Unix ms).
    pub timestamp_ms: u64,
}

/// 1Password Credential Brokering Client.
#[derive(Debug, Default)]
pub struct OnePasswordBrokerClient {
    items: RwLock<HashMap<String, OnePasswordItem>>,
}

impl OnePasswordBrokerClient {
    /// Creates a new 1Password broker client.
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a mocked or fetched item into the broker cache with TTL.
    pub fn register_item(&self, item: OnePasswordItem) {
        let mut map = self.items.write().unwrap();
        map.insert(item.item_id.clone(), item);
    }

    /// Fetches an item while checking TTL expiration and returning an audit receipt.
    pub fn checkout_item(
        &self,
        user_id: &str,
        item_id: &str,
        now_ms: u64,
    ) -> Result<(OnePasswordItem, OnePasswordAuditReceipt), &'static str> {
        let map = self.items.read().unwrap();
        let item = map.get(item_id).ok_or("1Password item not found")?;

        if now_ms > item.expires_at {
            return Err("1Password ephemeral checkout expired");
        }

        let item_id_hash = hex_encode(&blake3(item_id.as_bytes()));
        let receipt = OnePasswordAuditReceipt {
            item_id_hash,
            user_id: user_id.to_string(),
            category: format!("{:?}", item.category),
            consequence: item.category.consequence_level(),
            timestamp_ms: now_ms,
        };

        Ok((item.clone(), receipt))
    }
}

impl KinetiConnectorProtocol for OnePasswordBrokerClient {
    fn connector_name(&self) -> &'static str {
        "onepassword"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["fetch_item", "fill_credentials", "list_vault_items"]
    }

    fn evaluate_consequence(&self, action: &str, payload: &Value) -> ConsequenceLevel {
        match action {
            "list_vault_items" => ConsequenceLevel::Trivial,
            "fetch_item" | "fill_credentials" => {
                let category_str = get_str_property(payload, "category").unwrap_or("general");
                let cat = OnePasswordCategory::from_str(category_str);
                cat.consequence_level()
            }
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        match action {
            "list_vault_items" => {
                let map = self.items.read().unwrap();
                let keys: Vec<Value> = map
                    .values()
                    .map(|it| {
                        let mut m = BTreeMap::new();
                        m.insert("item_id".to_string(), Value::String(it.item_id.clone()));
                        m.insert("title".to_string(), Value::String(it.title.clone()));
                        m.insert("category".to_string(), Value::String(format!("{:?}", it.category)));
                        Value::Object(m)
                    })
                    .collect();
                let mut res = BTreeMap::new();
                res.insert("items".to_string(), Value::Array(keys));
                Ok(Value::Object(res))
            }
            "fetch_item" | "fill_credentials" => {
                let user_id = get_str_property(payload, "user_id").unwrap_or("default");
                let item_id = get_str_property(payload, "item_id").unwrap_or("");
                let now_ms = current_epoch_millis();

                match self.checkout_item(user_id, item_id, now_ms) {
                    Ok((item, receipt)) => {
                        let mut map = BTreeMap::new();
                        map.insert("item_id".to_string(), Value::String(item.item_id));
                        map.insert("title".to_string(), Value::String(item.title));
                        map.insert("username".to_string(), Value::String(item.username));
                        map.insert("secret".to_string(), Value::String(item.secret));
                        map.insert("category".to_string(), Value::String(format!("{:?}", item.category)));
                        map.insert("audit_hash".to_string(), Value::String(receipt.item_id_hash));
                        Ok(Value::Object(map))
                    }
                    Err(err) => Err(ConnectorProtocolError::ExecutionFailed(err.to_string())),
                }
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onepassword_category_consequence_levels() {
        assert_eq!(
            OnePasswordCategory::LoyaltyAccounts.consequence_level(),
            ConsequenceLevel::Operational
        );
        assert_eq!(
            OnePasswordCategory::SubscriptionPortals.consequence_level(),
            ConsequenceLevel::Operational
        );
        assert_eq!(
            OnePasswordCategory::HealthcareDashboards.consequence_level(),
            ConsequenceLevel::HighConsequence
        );
        assert_eq!(
            OnePasswordCategory::TaxPayrollPlatforms.consequence_level(),
            ConsequenceLevel::HighConsequence
        );
    }

    #[test]
    fn test_onepassword_checkout_and_audit_receipt() {
        let client = OnePasswordBrokerClient::new();
        let now = 1710000000u64;

        let item = OnePasswordItem {
            item_id: "op_item_delta".to_string(),
            title: "Delta Airlines".to_string(),
            category: OnePasswordCategory::LoyaltyAccounts,
            username: "praveen_flyer".to_string(),
            secret: "delta_pass_123".to_string(),
            checked_out_at: now,
            expires_at: now + 120_000, // 2-min TTL
        };
        client.register_item(item);

        let (retrieved, receipt) = client.checkout_item("user_01", "op_item_delta", now + 1000).unwrap();
        assert_eq!(retrieved.username, "praveen_flyer");
        assert_eq!(receipt.user_id, "user_01");
        assert_eq!(receipt.category, "LoyaltyAccounts");
        assert!(!receipt.item_id_hash.is_empty());
    }
}
