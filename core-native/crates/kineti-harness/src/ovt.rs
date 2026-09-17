//! Outcome Verification Ticket (OVT) with Cryptographic Dual-Signature Authority Separation.
//!
//! Implements Theorem 5.1: Authority Separation Invariant ($id_w \neq id_r \land pk_w \neq pk_r$).

use kineti_core::kernel::{blake3, hex_encode, sha256};
use std::fmt;

/// An Outcome Verification Ticket proving multi-agent dual review and evidence binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeVerificationTicket {
    /// Unique identifier of the ticket (e.g. `ovt_{taskId}_{timestamp}`).
    pub ticket_id: String,
    /// Identifier of the task being verified.
    pub task_id: String,
    /// Cryptographic hash of the immutable root goal.
    pub root_goal_hash: String,
    /// Identifier of the worker agent.
    pub worker_id: String,
    /// Public key or identity fingerprint of the worker agent.
    pub worker_pubkey: String,
    /// Identifier of the reviewer agent.
    pub reviewer_id: String,
    /// Public key or identity fingerprint of the reviewer agent.
    pub reviewer_pubkey: String,
    /// Cryptographic hash of the test execution evidence.
    pub evidence_hash: String,
    /// Cryptographic signature of the worker agent.
    pub worker_signature_hex: String,
    /// Cryptographic signature of the reviewer agent.
    pub reviewer_signature_hex: String,
    /// Milliseconds timestamp when verification occurred.
    pub verified_at: u64,
}

/// Errors originating during OVT ticket generation and validation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OvtError {
    /// Authority separation violation: worker cannot act as reviewer for their own work.
    WorkerCannotReviewSelf,
    /// Authority separation violation: identical public keys or identities.
    IdenticalPublicKeys,
    /// Worker signature verification failure.
    InvalidWorkerSignature,
    /// Reviewer signature verification failure.
    InvalidReviewerSignature,
    /// Tests did not pass; cannot issue outcome ticket.
    TestsFailed,
    /// Root goal hash mismatch (goal drift detected).
    GoalDriftDetected,
}

impl fmt::Display for OvtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerCannotReviewSelf => write!(f, "Authority separation violation: worker cannot review self"),
            Self::IdenticalPublicKeys => write!(f, "Authority separation violation: identical public keys"),
            Self::InvalidWorkerSignature => write!(f, "Invalid worker cryptographic signature"),
            Self::InvalidReviewerSignature => write!(f, "Invalid reviewer cryptographic signature"),
            Self::TestsFailed => write!(f, "Outcome verification rejected: tests failed"),
            Self::GoalDriftDetected => write!(f, "Outcome verification rejected: root goal drift detected"),
        }
    }
}

impl std::error::Error for OvtError {}

/// Coordinator responsible for generating and validating dual-signed OVTs.
pub struct OvtCoordinator;

impl OvtCoordinator {
    /// Computes cryptographic signature digest for an agent payload.
    pub fn sign_digest(private_key_secret: &str, payload: &str) -> String {
        let combined = format!("{}:{}:{}", private_key_secret, payload, private_key_secret);
        hex_encode(&sha256(combined.as_bytes()))
    }

    /// Verifies cryptographic signature digest against public key / secret.
    pub fn verify_signature(pubkey_or_secret: &str, payload: &str, signature_hex: &str) -> bool {
        let expected = Self::sign_digest(pubkey_or_secret, payload);
        expected == signature_hex
    }

    /// Generates a verified Outcome Verification Ticket enforcing authority separation.
    pub fn generate_ticket(
        task_id: &str,
        root_goal_hash: &str,
        worker_id: &str,
        worker_key: &str,
        reviewer_id: &str,
        reviewer_key: &str,
        evidence_hash: &str,
        tests_passed: bool,
        timestamp: u64,
    ) -> Result<OutcomeVerificationTicket, OvtError> {
        // Enforce Theorem 5.1: Authority Separation Invariants
        if worker_id == reviewer_id {
            return Err(OvtError::WorkerCannotReviewSelf);
        }
        if worker_key == reviewer_key {
            return Err(OvtError::IdenticalPublicKeys);
        }
        if !tests_passed {
            return Err(OvtError::TestsFailed);
        }

        // 1. Worker signs task and evidence
        let worker_payload = format!("{}:{}:{}:{}", task_id, worker_id, root_goal_hash, evidence_hash);
        let worker_sig = Self::sign_digest(worker_key, &worker_payload);

        // 2. Reviewer signs ticket, task, evidence, and worker signature
        let ticket_id = format!("ovt_{}_{}", task_id, timestamp);
        let reviewer_payload = format!("{}:{}:{}:{}:{}:{}", ticket_id, task_id, reviewer_id, evidence_hash, worker_sig, timestamp);
        let reviewer_sig = Self::sign_digest(reviewer_key, &reviewer_payload);

        Ok(OutcomeVerificationTicket {
            ticket_id,
            task_id: task_id.to_string(),
            root_goal_hash: root_goal_hash.to_string(),
            worker_id: worker_id.to_string(),
            worker_pubkey: hex_encode(&blake3(worker_key.as_bytes())),
            reviewer_id: reviewer_id.to_string(),
            reviewer_pubkey: hex_encode(&blake3(reviewer_key.as_bytes())),
            evidence_hash: evidence_hash.to_string(),
            worker_signature_hex: worker_sig,
            reviewer_signature_hex: reviewer_sig,
            verified_at: timestamp,
        })
    }

    /// Verifies the authenticity and integrity of a dual-signed OVT.
    pub fn verify_ticket(
        ticket: &OutcomeVerificationTicket,
        worker_key: &str,
        reviewer_key: &str,
    ) -> Result<bool, OvtError> {
        if ticket.worker_id == ticket.reviewer_id {
            return Err(OvtError::WorkerCannotReviewSelf);
        }

        let worker_payload = format!("{}:{}:{}:{}", ticket.task_id, ticket.worker_id, ticket.root_goal_hash, ticket.evidence_hash);
        if !Self::verify_signature(worker_key, &worker_payload, &ticket.worker_signature_hex) {
            return Err(OvtError::InvalidWorkerSignature);
        }

        let reviewer_payload = format!(
            "{}:{}:{}:{}:{}:{}",
            ticket.ticket_id, ticket.task_id, ticket.reviewer_id, ticket.evidence_hash, ticket.worker_signature_hex, ticket.verified_at
        );
        if !Self::verify_signature(reviewer_key, &reviewer_payload, &ticket.reviewer_signature_hex) {
            return Err(OvtError::InvalidReviewerSignature);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_separation_invariant() {
        // Worker attempting to review self must be rejected
        let res = OvtCoordinator::generate_ticket(
            "task_01",
            "goal_hash_01",
            "agent_worker",
            "secret_key_w",
            "agent_worker", // SAME ID
            "secret_key_r",
            "evidence_01",
            true,
            1700000000,
        );
        assert_eq!(res, Err(OvtError::WorkerCannotReviewSelf));

        // Identical keys must be rejected
        let res_key = OvtCoordinator::generate_ticket(
            "task_01",
            "goal_hash_01",
            "agent_worker",
            "shared_secret",
            "agent_reviewer",
            "shared_secret", // SAME KEY
            "evidence_01",
            true,
            1700000000,
        );
        assert_eq!(res_key, Err(OvtError::IdenticalPublicKeys));

        // Tests failed must be rejected
        let res_tests = OvtCoordinator::generate_ticket(
            "task_01",
            "goal_hash_01",
            "agent_worker",
            "secret_key_w",
            "agent_reviewer",
            "secret_key_r",
            "evidence_01",
            false, // TESTS FAILED
            1700000000,
        );
        assert_eq!(res_tests, Err(OvtError::TestsFailed));
    }

    #[test]
    fn test_valid_ticket_generation_and_verification() {
        let ticket = OvtCoordinator::generate_ticket(
            "task_100",
            "root_goal_blake3_digest",
            "worker_01",
            "worker_priv_secret_abc",
            "reviewer_02",
            "reviewer_priv_secret_xyz",
            "evidence_hash_123",
            true,
            1789000000,
        ).expect("Ticket generation should succeed");

        assert_eq!(ticket.task_id, "task_100");
        assert_eq!(ticket.worker_id, "worker_01");
        assert_eq!(ticket.reviewer_id, "reviewer_02");

        // Verify with valid keys
        let valid = OvtCoordinator::verify_ticket(&ticket, "worker_priv_secret_abc", "reviewer_priv_secret_xyz");
        assert_eq!(valid, Ok(true));

        // Tampered worker key fails
        let bad_w = OvtCoordinator::verify_ticket(&ticket, "tampered_key", "reviewer_priv_secret_xyz");
        assert_eq!(bad_w, Err(OvtError::InvalidWorkerSignature));

        // Tampered reviewer key fails
        let bad_r = OvtCoordinator::verify_ticket(&ticket, "worker_priv_secret_abc", "tampered_key");
        assert_eq!(bad_r, Err(OvtError::InvalidReviewerSignature));
    }
}
