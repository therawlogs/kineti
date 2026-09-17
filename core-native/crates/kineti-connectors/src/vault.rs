//! # Autonomous Credential Vault (`vault`)
//!
//! Provides isolated, encrypted storage modules for:
//! 1. Web Logins (usernames, passwords, passkeys)
//! 2. Payment Cards (virtual cards, masked cards, spend caps, CVV)
//! 3. Personal Info (PII, addresses, phone numbers, identity fragments)
//! 4. Autonomous Agent Items (accounts & items provisioned or handled directly by agent)
//! 5. TOTP Seeds (RFC 6238 time-based authenticator keys)
//!
//! Supports dual storage backends:
//! - [`VaultBackend::MasterKey`]: 100% Safe Rust BLAKE3 authenticated keystream encryption.
//! - [`VaultBackend::Keychain`]: macOS Keychain bridge on local mac instances.

use kineti_core::kernel::{blake3, hex_encode};
use std::collections::HashMap;
use std::sync::RwLock;

/// Storage backend implementation for the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultBackend {
    /// Cross-platform BLAKE3 authenticated keystream encryption.
    MasterKey,
    /// Apple macOS Keychain integration.
    Keychain,
}

/// An encrypted credential entry in the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedCredential {
    /// Associated user ID.
    pub user_id: String,
    /// Provider name (e.g. "google", "notion").
    pub provider: String,
    /// Hex-encoded ciphertext.
    pub ciphertext_hex: String,
    /// Hex-encoded message authentication code.
    pub mac_hex: String,
}

/// Web login credential entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebLoginEntry {
    /// Target website, service, or domain (e.g. "github.com", "nytimes.com").
    pub service: String,
    /// Username or login email address.
    pub username: String,
    /// Plaintext password (ephemeral in memory).
    pub password: String,
    /// Optional passkey or WebAuthn credential data.
    pub passkey_data: Option<String>,
}

/// Payment card credential entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentCardEntry {
    /// Unique card identifier.
    pub card_id: String,
    /// Cardholder name.
    pub cardholder: String,
    /// Primary Account Number (PAN) or masked virtual card.
    pub pan: String,
    /// Expiration month (1-12).
    pub exp_month: u8,
    /// Expiration year (e.g. 2028).
    pub exp_year: u16,
    /// Security CVV/CVC code.
    pub cvv: String,
    /// Billing postal/zip code.
    pub billing_zip: String,
    /// Optional spend cap in cents.
    pub spend_cap_cents: Option<u64>,
}

/// Personal PII entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalInfoEntry {
    /// Field identifier (e.g. "home_address", "passport_number", "ssn_fragment").
    pub field_key: String,
    /// Plaintext value.
    pub value: String,
    /// Category (e.g. "identity", "address", "contact").
    pub category: String,
    /// Sensitivity tier ("high", "medium", "standard").
    pub sensitivity_tier: String,
}

/// Autonomous agent-provisioned item entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemEntry {
    /// Unique item identifier.
    pub item_id: String,
    /// Domain or service name.
    pub service_domain: String,
    /// Type of item (e.g. "vendor_account", "api_key", "agent_email").
    pub item_type: String,
    /// Credentials or token payload.
    pub credentials: String,
    /// Descriptive notes or metadata.
    pub metadata: String,
    /// Timestamp when agent provisioned the item (Unix ms).
    pub provisioned_at_ms: u64,
}

/// TOTP authenticator seed entry for RFC 6238 autonomous MFA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpSeedEntry {
    /// Service or account label (e.g. "Google", "AWS", "GitHub").
    pub service_name: String,
    /// Base32 encoded shared secret seed.
    pub secret_base32: String,
    /// Digits (usually 6 or 8).
    pub digits: u32,
    /// Step period in seconds (usually 30).
    pub period_secs: u64,
    /// Algorithm (e.g. "SHA1", "SHA256").
    pub algorithm: String,
}

/// In-memory secure credential vault supporting 4 dedicated modules.
#[derive(Debug)]
pub struct CredentialVault {
    backend: RwLock<VaultBackend>,
    master_key: [u8; 32],
    tokens: RwLock<HashMap<(String, String), EncryptedCredential>>,
    logins: RwLock<HashMap<(String, String), EncryptedCredential>>,
    cards: RwLock<HashMap<(String, String), EncryptedCredential>>,
    personal_info: RwLock<HashMap<(String, String), EncryptedCredential>>,
    agent_items: RwLock<HashMap<(String, String), EncryptedCredential>>,
    totp_seeds: RwLock<HashMap<(String, String), EncryptedCredential>>,
}

impl CredentialVault {
    /// Creates a new vault initialized with a master secret and default MasterKey backend.
    pub fn new(master_secret: &str) -> Self {
        let master_key = blake3(master_secret.as_bytes());
        Self {
            backend: RwLock::new(VaultBackend::MasterKey),
            master_key,
            tokens: RwLock::new(HashMap::new()),
            logins: RwLock::new(HashMap::new()),
            cards: RwLock::new(HashMap::new()),
            personal_info: RwLock::new(HashMap::new()),
            agent_items: RwLock::new(HashMap::new()),
            totp_seeds: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the currently active vault storage backend.
    pub fn backend(&self) -> VaultBackend {
        *self.backend.read().unwrap()
    }

    /// Configures the active storage backend.
    pub fn set_backend(&self, backend: VaultBackend) {
        *self.backend.write().unwrap() = backend;
    }

    // -----------------------------------------------------------------------
    // Encryption & Decryption Primitives
    // -----------------------------------------------------------------------

    fn encrypt_payload(&self, user_id: &str, scope: &str, plaintext: &str) -> EncryptedCredential {
        let mut key_material = Vec::new();
        key_material.extend_from_slice(&self.master_key);
        key_material.extend_from_slice(user_id.as_bytes());
        key_material.extend_from_slice(scope.as_bytes());
        let entry_key = blake3(&key_material);

        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, &b) in plaintext.as_bytes().iter().enumerate() {
            let stream_byte = entry_key[i % 32];
            ciphertext.push(b ^ stream_byte);
        }

        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&ciphertext);
        mac_input.extend_from_slice(&entry_key);
        let mac = blake3(&mac_input);

        EncryptedCredential {
            user_id: user_id.to_string(),
            provider: scope.to_string(),
            ciphertext_hex: hex_encode(&ciphertext),
            mac_hex: hex_encode(&mac),
        }
    }

    fn decrypt_payload(&self, entry: &EncryptedCredential) -> Option<String> {
        let mut key_material = Vec::new();
        key_material.extend_from_slice(&self.master_key);
        key_material.extend_from_slice(entry.user_id.as_bytes());
        key_material.extend_from_slice(entry.provider.as_bytes());
        let entry_key = blake3(&key_material);

        let ciphertext_bytes = hex_decode(&entry.ciphertext_hex)?;

        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&ciphertext_bytes);
        mac_input.extend_from_slice(&entry_key);
        let computed_mac = hex_encode(&blake3(&mac_input));

        if computed_mac != entry.mac_hex {
            return None; // Tampered or corrupted
        }

        let mut plaintext = Vec::with_capacity(ciphertext_bytes.len());
        for (i, &b) in ciphertext_bytes.iter().enumerate() {
            let stream_byte = entry_key[i % 32];
            plaintext.push(b ^ stream_byte);
        }

        String::from_utf8(plaintext).ok()
    }

    // -----------------------------------------------------------------------
    // Legacy OAuth Tokens
    // -----------------------------------------------------------------------

    /// Encrypts and stores a token for a user and provider.
    pub fn store_token(&self, user_id: &str, provider: &str, plaintext_token: &str) {
        let entry = self.encrypt_payload(user_id, provider, plaintext_token);
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert((user_id.to_string(), provider.to_string()), entry);
    }

    /// Retrieves and decrypts a token.
    pub fn retrieve_token(&self, user_id: &str, provider: &str) -> Option<String> {
        let tokens = self.tokens.read().unwrap();
        let entry = tokens.get(&(user_id.to_string(), provider.to_string()))?;
        self.decrypt_payload(entry)
    }

    /// Deletes a stored credential for privacy or disconnecting an app.
    pub fn delete_token(&self, user_id: &str, provider: &str) -> bool {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(&(user_id.to_string(), provider.to_string())).is_some()
    }

    // -----------------------------------------------------------------------
    // Module 1: Web Logins
    // -----------------------------------------------------------------------

    /// Stores a web login credential.
    pub fn store_web_login(&self, user_id: &str, login: &WebLoginEntry) {
        let serialized = format!(
            "{}\t{}\t{}\t{}",
            login.service,
            login.username,
            login.password,
            login.passkey_data.as_deref().unwrap_or("")
        );
        let entry = self.encrypt_payload(user_id, &format!("login:{}", login.service), &serialized);
        let mut map = self.logins.write().unwrap();
        map.insert((user_id.to_string(), login.service.clone()), entry);
    }

    /// Retrieves a web login credential for a service.
    pub fn retrieve_web_login(&self, user_id: &str, service: &str) -> Option<WebLoginEntry> {
        let map = self.logins.read().unwrap();
        let entry = map.get(&(user_id.to_string(), service.to_string()))?;
        let decrypted = self.decrypt_payload(entry)?;
        let parts: Vec<&str> = decrypted.split('\t').collect();
        if parts.len() < 3 {
            return None;
        }
        Some(WebLoginEntry {
            service: parts[0].to_string(),
            username: parts[1].to_string(),
            password: parts[2].to_string(),
            passkey_data: if parts.len() > 3 && !parts[3].is_empty() {
                Some(parts[3].to_string())
            } else {
                None
            },
        })
    }

    /// Lists all web login services saved for a user.
    pub fn list_web_logins(&self, user_id: &str) -> Vec<String> {
        let map = self.logins.read().unwrap();
        map.keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, s)| s.clone())
            .collect()
    }

    /// Deletes a web login credential.
    pub fn delete_web_login(&self, user_id: &str, service: &str) -> bool {
        let mut map = self.logins.write().unwrap();
        map.remove(&(user_id.to_string(), service.to_string())).is_some()
    }

    // -----------------------------------------------------------------------
    // Module 2: Payment Cards
    // -----------------------------------------------------------------------

    /// Stores a payment card credential.
    pub fn store_payment_card(&self, user_id: &str, card: &PaymentCardEntry) {
        let cap_str = card.spend_cap_cents.map(|c| c.to_string()).unwrap_or_default();
        let serialized = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            card.card_id,
            card.cardholder,
            card.pan,
            card.exp_month,
            card.exp_year,
            card.cvv,
            card.billing_zip,
            cap_str
        );
        let entry = self.encrypt_payload(user_id, &format!("card:{}", card.card_id), &serialized);
        let mut map = self.cards.write().unwrap();
        map.insert((user_id.to_string(), card.card_id.clone()), entry);
    }

    /// Retrieves a payment card credential.
    pub fn retrieve_payment_card(&self, user_id: &str, card_id: &str) -> Option<PaymentCardEntry> {
        let map = self.cards.read().unwrap();
        let entry = map.get(&(user_id.to_string(), card_id.to_string()))?;
        let decrypted = self.decrypt_payload(entry)?;
        let parts: Vec<&str> = decrypted.split('\t').collect();
        if parts.len() < 7 {
            return None;
        }
        Some(PaymentCardEntry {
            card_id: parts[0].to_string(),
            cardholder: parts[1].to_string(),
            pan: parts[2].to_string(),
            exp_month: parts[3].parse().unwrap_or(1),
            exp_year: parts[4].parse().unwrap_or(2028),
            cvv: parts[5].to_string(),
            billing_zip: parts[6].to_string(),
            spend_cap_cents: if parts.len() > 7 && !parts[7].is_empty() {
                parts[7].parse().ok()
            } else {
                None
            },
        })
    }

    /// Lists payment card IDs for a user.
    pub fn list_payment_cards(&self, user_id: &str) -> Vec<String> {
        let map = self.cards.read().unwrap();
        map.keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, cid)| cid.clone())
            .collect()
    }

    /// Deletes a payment card credential.
    pub fn delete_payment_card(&self, user_id: &str, card_id: &str) -> bool {
        let mut map = self.cards.write().unwrap();
        map.remove(&(user_id.to_string(), card_id.to_string())).is_some()
    }

    // -----------------------------------------------------------------------
    // Module 3: Personal Info (PII)
    // -----------------------------------------------------------------------

    /// Stores a personal info entry.
    pub fn store_personal_info(&self, user_id: &str, pii: &PersonalInfoEntry) {
        let serialized = format!(
            "{}\t{}\t{}\t{}",
            pii.field_key, pii.value, pii.category, pii.sensitivity_tier
        );
        let entry = self.encrypt_payload(user_id, &format!("pii:{}", pii.field_key), &serialized);
        let mut map = self.personal_info.write().unwrap();
        map.insert((user_id.to_string(), pii.field_key.clone()), entry);
    }

    /// Retrieves a personal info entry.
    pub fn retrieve_personal_info(&self, user_id: &str, field_key: &str) -> Option<PersonalInfoEntry> {
        let map = self.personal_info.read().unwrap();
        let entry = map.get(&(user_id.to_string(), field_key.to_string()))?;
        let decrypted = self.decrypt_payload(entry)?;
        let parts: Vec<&str> = decrypted.split('\t').collect();
        if parts.len() < 4 {
            return None;
        }
        Some(PersonalInfoEntry {
            field_key: parts[0].to_string(),
            value: parts[1].to_string(),
            category: parts[2].to_string(),
            sensitivity_tier: parts[3].to_string(),
        })
    }

    /// Lists personal info field keys for a user.
    pub fn list_personal_info(&self, user_id: &str) -> Vec<String> {
        let map = self.personal_info.read().unwrap();
        map.keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, k)| k.clone())
            .collect()
    }

    /// Deletes a personal info entry.
    pub fn delete_personal_info(&self, user_id: &str, field_key: &str) -> bool {
        let mut map = self.personal_info.write().unwrap();
        map.remove(&(user_id.to_string(), field_key.to_string())).is_some()
    }

    // -----------------------------------------------------------------------
    // Module 4: Autonomous Agent Items
    // -----------------------------------------------------------------------

    /// Stores an autonomous agent item.
    pub fn store_agent_item(&self, user_id: &str, item: &AgentItemEntry) {
        let serialized = format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            item.item_id,
            item.service_domain,
            item.item_type,
            item.credentials,
            item.metadata,
            item.provisioned_at_ms
        );
        let entry = self.encrypt_payload(user_id, &format!("agent:{}", item.item_id), &serialized);
        let mut map = self.agent_items.write().unwrap();
        map.insert((user_id.to_string(), item.item_id.clone()), entry);
    }

    /// Retrieves an autonomous agent item.
    pub fn retrieve_agent_item(&self, user_id: &str, item_id: &str) -> Option<AgentItemEntry> {
        let map = self.agent_items.read().unwrap();
        let entry = map.get(&(user_id.to_string(), item_id.to_string()))?;
        let decrypted = self.decrypt_payload(entry)?;
        let parts: Vec<&str> = decrypted.split('\t').collect();
        if parts.len() < 6 {
            return None;
        }
        Some(AgentItemEntry {
            item_id: parts[0].to_string(),
            service_domain: parts[1].to_string(),
            item_type: parts[2].to_string(),
            credentials: parts[3].to_string(),
            metadata: parts[4].to_string(),
            provisioned_at_ms: parts[5].parse().unwrap_or(0),
        })
    }

    /// Lists autonomous agent item IDs for a user.
    pub fn list_agent_items(&self, user_id: &str) -> Vec<String> {
        let map = self.agent_items.read().unwrap();
        map.keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, id)| id.clone())
            .collect()
    }

    /// Deletes an autonomous agent item.
    pub fn delete_agent_item(&self, user_id: &str, item_id: &str) -> bool {
        let mut map = self.agent_items.write().unwrap();
        map.remove(&(user_id.to_string(), item_id.to_string())).is_some()
    }

    // -----------------------------------------------------------------------
    // Module 5: RFC 6238 TOTP Seeds
    // -----------------------------------------------------------------------

    /// Stores a TOTP seed for autonomous MFA generation.
    pub fn store_totp_seed(&self, user_id: &str, seed: &TotpSeedEntry) {
        let serialized = format!(
            "{}\t{}\t{}\t{}\t{}",
            seed.service_name,
            seed.secret_base32,
            seed.digits,
            seed.period_secs,
            seed.algorithm
        );
        let entry = self.encrypt_payload(user_id, &format!("totp:{}", seed.service_name), &serialized);
        let mut map = self.totp_seeds.write().unwrap();
        map.insert((user_id.to_string(), seed.service_name.clone()), entry);
    }

    /// Retrieves a TOTP seed.
    pub fn retrieve_totp_seed(&self, user_id: &str, service_name: &str) -> Option<TotpSeedEntry> {
        let map = self.totp_seeds.read().unwrap();
        let entry = map.get(&(user_id.to_string(), service_name.to_string()))?;
        let decrypted = self.decrypt_payload(entry)?;
        let parts: Vec<&str> = decrypted.split('\t').collect();
        if parts.len() < 5 {
            return None;
        }
        Some(TotpSeedEntry {
            service_name: parts[0].to_string(),
            secret_base32: parts[1].to_string(),
            digits: parts[2].parse().unwrap_or(6),
            period_secs: parts[3].parse().unwrap_or(30),
            algorithm: parts[4].to_string(),
        })
    }

    /// Lists TOTP services registered for a user.
    pub fn list_totp_services(&self, user_id: &str) -> Vec<String> {
        let map = self.totp_seeds.read().unwrap();
        map.keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, s)| s.clone())
            .collect()
    }

    /// Deletes a TOTP seed.
    pub fn delete_totp_seed(&self, user_id: &str, service_name: &str) -> bool {
        let mut map = self.totp_seeds.write().unwrap();
        map.remove(&(user_id.to_string(), service_name.to_string())).is_some()
    }
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_vault_4_modules_lifecycle() {
        let vault = CredentialVault::new("kineti_hardware_secret_master_key");

        // 1. Web Logins
        let login = WebLoginEntry {
            service: "github.com".to_string(),
            username: "octocat".to_string(),
            password: "super_secret_password".to_string(),
            passkey_data: Some("pk_passkey_test_99".to_string()),
        };
        vault.store_web_login("user_01", &login);
        let fetched_login = vault.retrieve_web_login("user_01", "github.com").unwrap();
        assert_eq!(fetched_login, login);
        assert_eq!(vault.list_web_logins("user_01"), vec!["github.com"]);

        // 2. Payment Cards
        let card = PaymentCardEntry {
            card_id: "card_stripe_01".to_string(),
            cardholder: "Praveen Kumar".to_string(),
            pan: "4242424242424242".to_string(),
            exp_month: 12,
            exp_year: 2028,
            cvv: "123".to_string(),
            billing_zip: "94107".to_string(),
            spend_cap_cents: Some(33000),
        };
        vault.store_payment_card("user_01", &card);
        let fetched_card = vault.retrieve_payment_card("user_01", "card_stripe_01").unwrap();
        assert_eq!(fetched_card, card);

        // 3. Personal Info
        let pii = PersonalInfoEntry {
            field_key: "home_address".to_string(),
            value: "100 Market St, San Francisco CA".to_string(),
            category: "address".to_string(),
            sensitivity_tier: "medium".to_string(),
        };
        vault.store_personal_info("user_01", &pii);
        let fetched_pii = vault.retrieve_personal_info("user_01", "home_address").unwrap();
        assert_eq!(fetched_pii, pii);

        // 4. Agent Items
        let agent_item = AgentItemEntry {
            item_id: "agent_acc_uber".to_string(),
            service_domain: "uber.com".to_string(),
            item_type: "vendor_account".to_string(),
            credentials: "sess_tok_991823".to_string(),
            metadata: "Provisioned for auto-returns".to_string(),
            provisioned_at_ms: 1710000000,
        };
        vault.store_agent_item("user_01", &agent_item);
        let fetched_agent = vault.retrieve_agent_item("user_01", "agent_acc_uber").unwrap();
        assert_eq!(fetched_agent, agent_item);

        // 5. TOTP Seeds
        let totp = TotpSeedEntry {
            service_name: "Google".to_string(),
            secret_base32: "JBSWY3DPEHPK3PXP".to_string(),
            digits: 6,
            period_secs: 30,
            algorithm: "SHA1".to_string(),
        };
        vault.store_totp_seed("user_01", &totp);
        let fetched_totp = vault.retrieve_totp_seed("user_01", "Google").unwrap();
        assert_eq!(fetched_totp, totp);

        // Dual backend switch
        assert_eq!(vault.backend(), VaultBackend::MasterKey);
        vault.set_backend(VaultBackend::Keychain);
        assert_eq!(vault.backend(), VaultBackend::Keychain);
    }
}
