//! # Kineti Harness (`kineti-harness`)
//!
//! Developer safety harness, shadow workspace isolation, and native Outcome Verification Tickets.
//!
//! ## Core Primitives
//! - **[`ovt`]**: Ed25519 dual-signed tickets; public-key trust remains the caller's responsibility.
//! - **[`shadow`]**: Git worktree separation for file edits, not a security sandbox.
//! - **[`cvg`]**: Causal Value Graph and Directional Normalized Trust-Weighted Impact (DNTI) with loss aversion.
//! - **[`signer`]**: In-memory Ed25519 signer; no hardware key backend is included.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod cvg;
pub mod dnti;
pub mod ovt;
pub mod shadow;
pub mod signer;

pub use cvg::{BlastRadiusReport, CausalValueGraph, ValueNode, DEFAULT_LOSS_AVERSION_KAPPA};
pub use dnti::{
    calculate_dnti, calculate_phi, compute_dnti, semantic_entropy_attenuation, DntiInput,
    DntiResult, MetricDirection, TestIntegrityPredicate, DEFAULT_EPSILON, DEFAULT_MIN_ASSERTIONS,
    DEFAULT_MIN_COVERAGE_PCT, DEFAULT_TAU,
};
pub use ovt::{OutcomeVerificationTicket, OvtCoordinator, OvtError};
pub use shadow::ShadowWorkspace;
pub use signer::{
    confirm_prompt, enclave_available, needs_human_confirm, SignerBackend, SoftwareSigner,
};
