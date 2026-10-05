//! Ed25519 software signing for local OVT and verification flows.
//!
//! Keys are generated in memory and are not written to disk. This module does
//! not provide hardware-backed key storage. A valid signature identifies a
//! key, not the person or independent service that controls it.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;

/// Signing backend interface used by the harness.
pub trait SignerBackend {
    /// Stable display label; must never contain private key material.
    fn key_label(&self) -> &str;
    /// Returns the Ed25519 public key as lowercase hex.
    fn public_key_hex(&self) -> String;
    /// Signs a payload using this backend's private key and returns lowercase hex.
    fn sign(&self, payload: &[u8]) -> String;
    /// Verifies a signature using this backend's public key.
    fn verify(&self, payload: &[u8], signature_hex: &str) -> bool;
    /// True only when the signing key is protected by platform hardware.
    fn is_hardware_backed(&self) -> bool;
}

/// In-memory Ed25519 signer. The private key is generated using the operating
/// system random source and remains in memory for this process.
#[derive(Debug)]
pub struct SoftwareSigner {
    label: String,
    signing_key: SigningKey,
}

impl SoftwareSigner {
    /// Generates a new in-memory Ed25519 key pair.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    /// Creates a signer from a 32-byte Ed25519 seed. Callers must protect this
    /// seed themselves; it should never be committed or logged.
    pub fn from_seed(label: impl Into<String>, seed: [u8; 32]) -> Self {
        Self {
            label: label.into(),
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    /// Returns the corresponding public verification key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl SignerBackend for SoftwareSigner {
    fn key_label(&self) -> &str {
        &self.label
    }

    fn public_key_hex(&self) -> String {
        encode_hex(&self.signing_key.verifying_key().to_bytes())
    }

    fn sign(&self, payload: &[u8]) -> String {
        encode_hex(&self.signing_key.sign(payload).to_bytes())
    }

    fn verify(&self, payload: &[u8], signature_hex: &str) -> bool {
        let Ok(signature_bytes) = decode_hex(signature_hex) else {
            return false;
        };
        let Ok(signature_array) = <[u8; 64]>::try_from(signature_bytes.as_slice()) else {
            return false;
        };
        self.signing_key
            .verifying_key()
            .verify(payload, &Signature::from_bytes(&signature_array))
            .is_ok()
    }

    fn is_hardware_backed(&self) -> bool {
        false
    }
}

/// Reports whether an operating-system hardware signing backend is available.
/// No Secure Enclave or TPM integration is included in this release.
pub fn enclave_available() -> bool {
    false
}

/// Every nonzero amount requires a separate human confirmation.
pub fn needs_human_confirm(amount_cents: u64) -> bool {
    amount_cents > 0
}

/// Formats a human confirmation question for a payment action.
pub fn confirm_prompt(amount_cents: u64, payee: &str) -> String {
    format!(
        "Confirm ${}.{:02} to {}?",
        amount_cents / 100,
        amount_cents % 100,
        payee
    )
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn decode_hex(text: &str) -> Result<Vec<u8>, ()> {
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let [high, low] = pair else {
                return Err(());
            };
            let high = (*high as char).to_digit(16).ok_or(())?;
            let low = (*low as char).to_digit(16).ok_or(())?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ed25519_signatures_verify_with_the_matching_key_only() {
        let signer = SoftwareSigner::new("worker");
        let other = SoftwareSigner::new("reviewer");
        let signature = signer.sign(b"task=evidence");
        assert!(signer.verify(b"task=evidence", &signature));
        assert!(!signer.verify(b"task=changed", &signature));
        assert!(!other.verify(b"task=evidence", &signature));
        assert!(!signer.is_hardware_backed());
        assert_eq!(signer.public_key_hex().len(), 64);
    }

    #[test]
    fn invalid_signature_encoding_fails_closed() {
        let signer = SoftwareSigner::new("worker");
        assert!(!signer.verify(b"data", "bad"));
        assert!(!signer.verify(b"data", "00"));
    }

    #[test]
    fn confirmation_helpers_keep_nonzero_actions_human_gated() {
        assert!(!needs_human_confirm(0));
        assert!(needs_human_confirm(1));
        assert_eq!(confirm_prompt(1050, "acme"), "Confirm $10.50 to acme?");
        assert!(!enclave_available());
    }
}
