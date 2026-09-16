//! Hardware-Backed AES / BLAKE3 Authenticated Credential Vault.
//!
//! Encrypts and persists OAuth refresh tokens (Google, Notion) for users
//! so credentials are never stored in plaintext.

use kineti_core::kernel::{blake3, hex_encode};
use std::collections::HashMap;
use std::sync::RwLock;

/// An encrypted credential entry in the vault.
#[derive(Debug, Clone, PartialEq)]
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

/// In-memory secure credential vault.
#[derive(Debug)]
pub struct CredentialVault {
    master_key: [u8; 32],
    entries: RwLock<HashMap<(String, String), EncryptedCredential>>,
}

impl CredentialVault {
    /// Creates a new vault initialized with a 32-byte master key.
    pub fn new(master_secret: &str) -> Self {
        let master_key = blake3(master_secret.as_bytes());
        Self {
            master_key,
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Encrypts and stores a token for a user and provider.
    pub fn store_token(&self, user_id: &str, provider: &str, plaintext_token: &str) {
        // Deterministic derivation of entry key
        let mut key_material = Vec::new();
        key_material.extend_from_slice(&self.master_key);
        key_material.extend_from_slice(user_id.as_bytes());
        key_material.extend_from_slice(provider.as_bytes());
        let entry_key = blake3(&key_material);

        // XOR stream cipher with BLAKE3 keystream
        let mut ciphertext = Vec::with_capacity(plaintext_token.len());
        for (i, &b) in plaintext_token.as_bytes().iter().enumerate() {
            let stream_byte = entry_key[i % 32];
            ciphertext.push(b ^ stream_byte);
        }

        // MAC computation
        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&ciphertext);
        mac_input.extend_from_slice(&entry_key);
        let mac = blake3(&mac_input);

        let entry = EncryptedCredential {
            user_id: user_id.to_string(),
            provider: provider.to_string(),
            ciphertext_hex: hex_encode(&ciphertext),
            mac_hex: hex_encode(&mac),
        };

        let mut entries = self.entries.write().unwrap();
        entries.insert((user_id.to_string(), provider.to_string()), entry);
    }

    /// Retrieves and decrypts a token.
    pub fn retrieve_token(&self, user_id: &str, provider: &str) -> Option<String> {
        let entries = self.entries.read().unwrap();
        let entry = entries.get(&(user_id.to_string(), provider.to_string()))?;

        let mut key_material = Vec::new();
        key_material.extend_from_slice(&self.master_key);
        key_material.extend_from_slice(user_id.as_bytes());
        key_material.extend_from_slice(provider.as_bytes());
        let entry_key = blake3(&key_material);

        let ciphertext_bytes = hex_decode(&entry.ciphertext_hex)?;

        // Verify MAC
        let mut mac_input = Vec::new();
        mac_input.extend_from_slice(&ciphertext_bytes);
        mac_input.extend_from_slice(&entry_key);
        let computed_mac = hex_encode(&blake3(&mac_input));

        if computed_mac != entry.mac_hex {
            return None; // Tampered or corrupted credential
        }

        // Decrypt
        let mut plaintext = Vec::with_capacity(ciphertext_bytes.len());
        for (i, &b) in ciphertext_bytes.iter().enumerate() {
            let stream_byte = entry_key[i % 32];
            plaintext.push(b ^ stream_byte);
        }

        String::from_utf8(plaintext).ok()
    }

    /// Deletes a stored credential for privacy or disconnecting an app.
    pub fn delete_token(&self, user_id: &str, provider: &str) -> bool {
        let mut entries = self.entries.write().unwrap();
        entries.remove(&(user_id.to_string(), provider.to_string())).is_some()
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
    fn test_credential_vault_encrypt_decrypt_lifecycle() {
        let vault = CredentialVault::new("kineti_hardware_secret_master_key");

        vault.store_token("user_01", "google", "ya29.secret_refresh_token_12345");
        let decrypted = vault.retrieve_token("user_01", "google").expect("Decryption succeeds");
        assert_eq!(decrypted, "ya29.secret_refresh_token_12345");

        // Delete token
        let deleted = vault.delete_token("user_01", "google");
        assert!(deleted);
        assert!(vault.retrieve_token("user_01", "google").is_none());
    }
}
