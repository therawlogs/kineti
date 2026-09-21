//! Hardware-backed signing abstraction (`signer`).
//!
//! Financial and contract execution tokens should be signed inside platform
//! hardware (Apple Secure Enclave, TPM) so the key never sits in process
//! memory. That platform bridge is not built yet: it needs a Swift/ObjC
//! bridge on macOS and a TPM2 stack on Linux, both outside the current
//! zero-outside-crate rule.
//!
//! This module therefore provides:
//! - [`HardwareSigner`]: the trait every backend implements. The user only
//!   ever sees "confirm $X?" regardless of backend.
//! - [`SoftwareSigner`]: std-only fallback using the same digest
//!   construction as OVT attestation. `is_hardware_backed()` is false.
//! - [`enclave_available`]: reports false on all targets today. When a
//!   platform bridge lands, it flips per-target behind `cfg` with no
//!   caller changes.
//! - [`needs_human_confirm`]: every nonzero amount needs an explicit human
//!   yes. There is no auto-approve path.

use kineti_core::kernel::{hex_encode, sha256};

/// Signing backend for high-consequence tokens.
pub trait HardwareSigner {
    /// Stable label shown in audit lines (never key material).
    fn key_label(&self) -> String;
    /// Signs a payload, returning a hex digest.
    fn sign(&self, payload: &[u8]) -> String;
    /// Verifies a signature produced by [`HardwareSigner::sign`].
    fn verify(&self, payload: &[u8], signature_hex: &str) -> bool;
    /// True only when the key lives in platform hardware.
    fn is_hardware_backed(&self) -> bool;
}

/// Std-only software fallback signer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareSigner {
    label: String,
    secret: String,
}

impl SoftwareSigner {
    /// Creates a software signer. The secret stays in process memory,
    /// unlike a hardware backend.
    pub fn new(label: impl Into<String>, secret: impl Into<String>) -> Self {
        Self { label: label.into(), secret: secret.into() }
    }
}

impl HardwareSigner for SoftwareSigner {
    fn key_label(&self) -> String {
        format!("software:{}", self.label)
    }

    fn sign(&self, payload: &[u8]) -> String {
        let combined = format!("{}:{}:{}", self.secret, hex_encode(payload), self.secret);
        hex_encode(&sha256(combined.as_bytes()))
    }

    fn verify(&self, payload: &[u8], signature_hex: &str) -> bool {
        self.sign(payload) == signature_hex
    }

    fn is_hardware_backed(&self) -> bool {
        false
    }
}

/// Reports whether a platform hardware backend exists on this target.
/// Always false today; the Secure Enclave / TPM bridge is a documented
/// follow-up requiring platform-native code outside the zero-dep rule.
pub fn enclave_available() -> bool {
    false
}

/// Every nonzero cent amount needs an explicit human yes. Zero-amount
/// intents (balance checks, quotes) do not move money and pass through.
pub fn needs_human_confirm(amount_cents: u64) -> bool {
    amount_cents > 0
}

/// Formats the only question the user ever sees for money movement.
pub fn confirm_prompt(amount_cents: u64, payee: &str) -> String {
    let dollars = amount_cents / 100;
    let cents = amount_cents % 100;
    format!("Confirm ${dollars}.{cents:02} to {payee}?")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_roundtrip() {
        let s = SoftwareSigner::new("test-key", "secret-abc");
        let sig = s.sign(b"pay $10 to acme");
        assert!(s.verify(b"pay $10 to acme", &sig));
        assert_eq!(s.key_label(), "software:test-key");
        assert!(!s.is_hardware_backed());
    }

    #[test]
    fn test_tampered_payload_fails() {
        let s = SoftwareSigner::new("k", "secret-abc");
        let sig = s.sign(b"pay $10 to acme");
        assert!(!s.verify(b"pay $11 to acme", &sig));
    }

    #[test]
    fn test_different_secrets_differ() {
        let a = SoftwareSigner::new("a", "secret-1");
        let b = SoftwareSigner::new("b", "secret-2");
        assert_ne!(a.sign(b"same payload"), b.sign(b"same payload"));
        assert!(!a.verify(b"same payload", &b.sign(b"same payload")));
    }

    #[test]
    fn test_confirm_rule_and_prompt() {
        assert!(!needs_human_confirm(0));
        assert!(needs_human_confirm(1));
        assert!(needs_human_confirm(10_000));
        assert_eq!(confirm_prompt(1050, "acme"), "Confirm $10.50 to acme?");
        assert!(!enclave_available());
    }
}
