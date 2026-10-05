//! Ed25519-signed Outcome Verification Tickets (OVTs).
//!
//! An OVT signature protects the ticket fields from alteration and proves that
//! the corresponding private key signed them. It does not prove that the
//! evidence is true, authenticate a public key by itself, or make a local
//! worker and reviewer independent. Callers must obtain expected public keys
//! from a trusted source before accepting a ticket.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::fmt;

const TICKET_VERSION: u8 = 1;
const WORKER_DOMAIN: &[u8] = b"kineti-ovt-worker-v1";
const REVIEWER_DOMAIN: &[u8] = b"kineti-ovt-reviewer-v1";

/// An Ed25519 dual-signed ticket describing a reviewed task result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeVerificationTicket {
    /// OVT wire format version.
    pub version: u8,
    /// Unique identifier of this ticket.
    pub ticket_id: String,
    /// Identifier of the task being verified.
    pub task_id: String,
    /// Hash of the root goal.
    pub root_goal_hash: String,
    /// Identifier of the worker agent.
    pub worker_id: String,
    /// Worker Ed25519 public key, encoded as lowercase hex.
    pub worker_pubkey: String,
    /// Identifier of the reviewer agent.
    pub reviewer_id: String,
    /// Reviewer Ed25519 public key, encoded as lowercase hex.
    pub reviewer_pubkey: String,
    /// Hash of the evidence being reviewed.
    pub evidence_hash: String,
    /// Whether the caller reports that required checks passed.
    pub tests_passed: bool,
    /// Worker Ed25519 signature, encoded as lowercase hex.
    pub worker_signature_hex: String,
    /// Reviewer Ed25519 signature, encoded as lowercase hex.
    pub reviewer_signature_hex: String,
    /// Timestamp when the reviewer signed the ticket, in milliseconds.
    pub verified_at: u64,
}

/// Errors from OVT creation or verification.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OvtError {
    /// The worker and reviewer identifiers are identical.
    WorkerCannotReviewSelf,
    /// The worker and reviewer public keys are identical.
    IdenticalPublicKeys,
    /// The ticket embeds a different worker key than the verifier expects.
    WorkerKeyMismatch,
    /// The ticket embeds a different reviewer key than the verifier expects.
    ReviewerKeyMismatch,
    /// The ticket version is unsupported.
    UnsupportedVersion,
    /// Ticket creation was requested when checks did not pass.
    TestsFailed,
    /// A key or signature has an invalid encoding or length.
    InvalidEncoding,
    /// The worker signature did not verify against the signed fields.
    InvalidWorkerSignature,
    /// The reviewer signature did not verify against the signed fields.
    InvalidReviewerSignature,
}

impl fmt::Display for OvtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::WorkerCannotReviewSelf => "worker cannot review its own result",
            Self::IdenticalPublicKeys => "worker and reviewer keys must differ",
            Self::WorkerKeyMismatch => "ticket worker key does not match the trusted key",
            Self::ReviewerKeyMismatch => "ticket reviewer key does not match the trusted key",
            Self::UnsupportedVersion => "unsupported OVT version",
            Self::TestsFailed => "cannot create an OVT when checks failed",
            Self::InvalidEncoding => "invalid Ed25519 key or signature encoding",
            Self::InvalidWorkerSignature => "worker signature verification failed",
            Self::InvalidReviewerSignature => "reviewer signature verification failed",
        };
        f.write_str(message)
    }
}

impl std::error::Error for OvtError {}

/// Creates and verifies Ed25519-signed OVTs.
pub struct OvtCoordinator;

impl OvtCoordinator {
    /// Creates a ticket with different worker and reviewer signing keys.
    ///
    /// The boolean is a caller-provided result, not independent proof that
    /// tests ran. The signed `evidence_hash` should identify evidence that a
    /// separate trusted verifier can inspect.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_ticket(
        task_id: &str,
        root_goal_hash: &str,
        worker_id: &str,
        worker_key: &SigningKey,
        reviewer_id: &str,
        reviewer_key: &SigningKey,
        evidence_hash: &str,
        tests_passed: bool,
        timestamp: u64,
    ) -> Result<OutcomeVerificationTicket, OvtError> {
        if worker_id == reviewer_id {
            return Err(OvtError::WorkerCannotReviewSelf);
        }
        if worker_key.verifying_key() == reviewer_key.verifying_key() {
            return Err(OvtError::IdenticalPublicKeys);
        }
        if !tests_passed {
            return Err(OvtError::TestsFailed);
        }

        let worker_public = worker_key.verifying_key().to_bytes();
        let reviewer_public = reviewer_key.verifying_key().to_bytes();
        let ticket_id = format!("ovt_{task_id}_{timestamp}");
        let worker_payload = worker_payload(
            &ticket_id,
            task_id,
            root_goal_hash,
            worker_id,
            &worker_public,
            reviewer_id,
            &reviewer_public,
            evidence_hash,
            tests_passed,
        );
        let worker_signature = worker_key.sign(&worker_payload).to_bytes();
        let reviewer_payload = reviewer_payload(
            &worker_payload,
            &worker_signature,
            reviewer_id,
            &reviewer_public,
            timestamp,
        );
        let reviewer_signature = reviewer_key.sign(&reviewer_payload).to_bytes();

        Ok(OutcomeVerificationTicket {
            version: TICKET_VERSION,
            ticket_id,
            task_id: task_id.to_string(),
            root_goal_hash: root_goal_hash.to_string(),
            worker_id: worker_id.to_string(),
            worker_pubkey: encode_hex(&worker_public),
            reviewer_id: reviewer_id.to_string(),
            reviewer_pubkey: encode_hex(&reviewer_public),
            evidence_hash: evidence_hash.to_string(),
            tests_passed,
            worker_signature_hex: encode_hex(&worker_signature),
            reviewer_signature_hex: encode_hex(&reviewer_signature),
            verified_at: timestamp,
        })
    }

    /// Verifies a ticket against caller-supplied trusted public keys.
    pub fn verify_ticket(
        ticket: &OutcomeVerificationTicket,
        expected_worker_key: &VerifyingKey,
        expected_reviewer_key: &VerifyingKey,
    ) -> Result<bool, OvtError> {
        if ticket.version != TICKET_VERSION {
            return Err(OvtError::UnsupportedVersion);
        }
        if !ticket.tests_passed {
            return Err(OvtError::TestsFailed);
        }
        if ticket.worker_id == ticket.reviewer_id {
            return Err(OvtError::WorkerCannotReviewSelf);
        }
        if expected_worker_key == expected_reviewer_key {
            return Err(OvtError::IdenticalPublicKeys);
        }

        let embedded_worker = decode_public_key(&ticket.worker_pubkey)?;
        let embedded_reviewer = decode_public_key(&ticket.reviewer_pubkey)?;
        if &embedded_worker != expected_worker_key {
            return Err(OvtError::WorkerKeyMismatch);
        }
        if &embedded_reviewer != expected_reviewer_key {
            return Err(OvtError::ReviewerKeyMismatch);
        }

        let worker_signature = decode_signature(&ticket.worker_signature_hex)?;
        let reviewer_signature = decode_signature(&ticket.reviewer_signature_hex)?;
        let worker_payload = worker_payload(
            &ticket.ticket_id,
            &ticket.task_id,
            &ticket.root_goal_hash,
            &ticket.worker_id,
            &embedded_worker.to_bytes(),
            &ticket.reviewer_id,
            &embedded_reviewer.to_bytes(),
            &ticket.evidence_hash,
            ticket.tests_passed,
        );
        expected_worker_key
            .verify(&worker_payload, &worker_signature)
            .map_err(|_| OvtError::InvalidWorkerSignature)?;

        let reviewer_payload = reviewer_payload(
            &worker_payload,
            &worker_signature.to_bytes(),
            &ticket.reviewer_id,
            &embedded_reviewer.to_bytes(),
            ticket.verified_at,
        );
        expected_reviewer_key
            .verify(&reviewer_payload, &reviewer_signature)
            .map_err(|_| OvtError::InvalidReviewerSignature)?;
        Ok(true)
    }
}

fn append_field(output: &mut Vec<u8>, field: &[u8]) {
    output.extend_from_slice(&(field.len() as u64).to_be_bytes());
    output.extend_from_slice(field);
}

#[allow(clippy::too_many_arguments)]
fn worker_payload(
    ticket_id: &str,
    task_id: &str,
    root_goal_hash: &str,
    worker_id: &str,
    worker_public: &[u8; 32],
    reviewer_id: &str,
    reviewer_public: &[u8; 32],
    evidence_hash: &str,
    tests_passed: bool,
) -> Vec<u8> {
    let mut payload = Vec::new();
    append_field(&mut payload, WORKER_DOMAIN);
    append_field(&mut payload, &[TICKET_VERSION]);
    append_field(&mut payload, ticket_id.as_bytes());
    append_field(&mut payload, task_id.as_bytes());
    append_field(&mut payload, root_goal_hash.as_bytes());
    append_field(&mut payload, worker_id.as_bytes());
    append_field(&mut payload, worker_public);
    append_field(&mut payload, reviewer_id.as_bytes());
    append_field(&mut payload, reviewer_public);
    append_field(&mut payload, evidence_hash.as_bytes());
    append_field(&mut payload, &[u8::from(tests_passed)]);
    payload
}

fn reviewer_payload(
    worker_payload: &[u8],
    worker_signature: &[u8; 64],
    reviewer_id: &str,
    reviewer_public: &[u8; 32],
    timestamp: u64,
) -> Vec<u8> {
    let mut payload = Vec::new();
    append_field(&mut payload, REVIEWER_DOMAIN);
    append_field(&mut payload, worker_payload);
    append_field(&mut payload, worker_signature);
    append_field(&mut payload, reviewer_id.as_bytes());
    append_field(&mut payload, reviewer_public);
    append_field(&mut payload, &timestamp.to_be_bytes());
    payload
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

fn decode_hex(text: &str) -> Result<Vec<u8>, OvtError> {
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let [high, low] = pair else {
                return Err(OvtError::InvalidEncoding);
            };
            let high = (*high as char)
                .to_digit(16)
                .ok_or(OvtError::InvalidEncoding)?;
            let low = (*low as char)
                .to_digit(16)
                .ok_or(OvtError::InvalidEncoding)?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}

fn decode_public_key(text: &str) -> Result<VerifyingKey, OvtError> {
    let bytes: [u8; 32] = decode_hex(text)?
        .try_into()
        .map_err(|_| OvtError::InvalidEncoding)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| OvtError::InvalidEncoding)
}

fn decode_signature(text: &str) -> Result<Signature, OvtError> {
    let bytes: [u8; 64] = decode_hex(text)?
        .try_into()
        .map_err(|_| OvtError::InvalidEncoding)?;
    Ok(Signature::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn keys() -> (SigningKey, SigningKey) {
        (
            SigningKey::generate(&mut OsRng),
            SigningKey::generate(&mut OsRng),
        )
    }

    #[test]
    fn known_ed25519_test_vector_matches_rfc8032() {
        let seed: [u8; 32] =
            decode_hex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap()
                .try_into()
                .unwrap();
        let key = SigningKey::from_bytes(&seed);
        assert_eq!(
            encode_hex(&key.verifying_key().to_bytes()),
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        );
        assert_eq!(
            encode_hex(&key.sign(b"").to_bytes()),
            concat!(
                "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155",
                "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
            ),
        );
    }

    #[test]
    fn valid_ticket_verifies_only_with_the_expected_public_keys() {
        let (worker, reviewer) = keys();
        let ticket = OvtCoordinator::generate_ticket(
            "task_100",
            "goal_hash",
            "worker_1",
            &worker,
            "reviewer_2",
            &reviewer,
            "evidence_hash",
            true,
            1_789_000_000,
        )
        .unwrap();
        assert!(OvtCoordinator::verify_ticket(
            &ticket,
            &worker.verifying_key(),
            &reviewer.verifying_key()
        )
        .unwrap());

        let (wrong_worker, _) = keys();
        assert_eq!(
            OvtCoordinator::verify_ticket(
                &ticket,
                &wrong_worker.verifying_key(),
                &reviewer.verifying_key()
            ),
            Err(OvtError::WorkerKeyMismatch),
        );
    }

    #[test]
    fn altered_ticket_fields_and_signatures_fail() {
        let (worker, reviewer) = keys();
        let ticket = OvtCoordinator::generate_ticket(
            "task_200",
            "goal_hash",
            "worker_1",
            &worker,
            "reviewer_2",
            &reviewer,
            "evidence_hash",
            true,
            1_789_000_001,
        )
        .unwrap();
        let mut changed_evidence = ticket.clone();
        changed_evidence.evidence_hash.push('x');
        assert_eq!(
            OvtCoordinator::verify_ticket(
                &changed_evidence,
                &worker.verifying_key(),
                &reviewer.verifying_key()
            ),
            Err(OvtError::InvalidWorkerSignature),
        );

        let mut changed_reviewer_signature = ticket;
        let replacement = if changed_reviewer_signature
            .reviewer_signature_hex
            .starts_with("00")
        {
            "01"
        } else {
            "00"
        };
        changed_reviewer_signature
            .reviewer_signature_hex
            .replace_range(0..2, replacement);
        assert_eq!(
            OvtCoordinator::verify_ticket(
                &changed_reviewer_signature,
                &worker.verifying_key(),
                &reviewer.verifying_key()
            ),
            Err(OvtError::InvalidReviewerSignature),
        );
    }

    #[test]
    fn refuses_self_review_same_key_and_failed_checks() {
        let (worker, reviewer) = keys();
        assert_eq!(
            OvtCoordinator::generate_ticket(
                "t", "g", "same", &worker, "same", &reviewer, "e", true, 1
            ),
            Err(OvtError::WorkerCannotReviewSelf),
        );
        assert_eq!(
            OvtCoordinator::generate_ticket("t", "g", "w", &worker, "r", &worker, "e", true, 1),
            Err(OvtError::IdenticalPublicKeys),
        );
        assert_eq!(
            OvtCoordinator::generate_ticket("t", "g", "w", &worker, "r", &reviewer, "e", false, 1),
            Err(OvtError::TestsFailed),
        );
    }
}
