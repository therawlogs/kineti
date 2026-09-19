//! # Kineti Harness (`kineti-harness`)
//!
//! Developer safety harness, shadow workspace isolation, and native Outcome Verification Tickets.
//!
//! ## Core Primitives
//! - **[`ovt`]**: Asymmetric dual-signed Outcome Verification Tickets with authority separation ($id_w \neq id_r$).
//! - **[`shadow`]**: Git worktree shadow workspace isolation.
//! - **[`cvg`]**: Causal Value Graph and Directional Normalized Trust-Weighted Impact (DNTI) with loss aversion.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod cvg;
pub mod dnti;
pub mod ovt;
pub mod shadow;

pub use cvg::{BlastRadiusReport, CausalValueGraph, ValueNode, DEFAULT_LOSS_AVERSION_KAPPA};
pub use dnti::{
    calculate_dnti, calculate_phi, compute_dnti, semantic_entropy_attenuation, DntiInput,
    DntiResult, MetricDirection, TestIntegrityPredicate, DEFAULT_EPSILON, DEFAULT_MIN_ASSERTIONS,
    DEFAULT_MIN_COVERAGE_PCT, DEFAULT_TAU,
};
pub use ovt::{OutcomeVerificationTicket, OvtCoordinator, OvtError};
pub use shadow::ShadowWorkspace;
