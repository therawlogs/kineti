//! # Kineti Core (`kineti-core`)
//!
//! Foundational native Rust nervous system and context integrity harness for autonomous agents.
//!
//! ## Core Primitives
//! - **[`snapshot`]**: Lock-free atomic state snapshots with wait-free reads and zero torn reads.
//! - **[`kernel`]**: Universal 20-Entity Provenance Kernel, RFC 8785 JSON canonicalization, and BLAKE3/SHA-256 content addressing.
//! - **[`hlc`]**: Monotonic Hybrid Logical Clock (HLC) tracking physical timestamp (ms) and logical sequence under clock skew.
//! - **[`gate`]**: Sub-50ms 3-Way Graph Commit Gate enforcing topological rank acyclicity, HLC causal monotonicity, and Merkle DAG integrity.
//! - **[`spend`]**: Atomic fast-path spend circuit breaker ($50 ceiling, $47.50 95% trip) with two-phase commit pre-allocations and partial spend settlement.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod conversation;
pub mod gate;
pub mod hlc;
pub mod kernel;
pub mod snapshot;
pub mod spend;

use std::error::Error;
use std::fmt;

// ---------------------------------------------------------------------------
// Canonical Type Aliases
// ---------------------------------------------------------------------------

/// Financial amount represented in microcents ($1.00 USD = 1,000,000 microcents).
pub type Microcents = u64;

/// Milliseconds since Unix epoch (January 1, 1970 UTC).
pub type MillisecondTimestamp = u64;

/// Logical sequence counter for HLC timestamps within the same physical millisecond.
pub type LogicalCounter = u32;

/// Hexadecimal string representing a cryptographic digest (BLAKE3 or SHA-256).
pub type DigestHex = String;

/// Standard Result alias for kineti-core operations.
pub type CoreResult<T> = Result<T, CoreError>;

// ---------------------------------------------------------------------------
// Unified Error Taxonomy
// ---------------------------------------------------------------------------

/// Top-level unified error taxonomy for `kineti-core`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CoreError {
    /// Error originating in the Universal Provenance Kernel.
    Kernel(kernel::KernelError),

    /// Error originating in the Hybrid Logical Clock.
    Hlc(hlc::HlcError),

    /// Error originating in the 3-Way Graph Commit Gate.
    Gate(gate::CommitGateError),

    /// Error originating in the Spend Circuit Breaker.
    Spend(spend::SpendError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(err) => write!(f, "Kernel error: {}", err),
            Self::Hlc(err) => write!(f, "HLC error: {}", err),
            Self::Gate(err) => write!(f, "Commit gate error: {}", err),
            Self::Spend(err) => write!(f, "Spend circuit breaker error: {}", err),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(err) => Some(err),
            Self::Hlc(err) => Some(err),
            Self::Gate(err) => Some(err),
            Self::Spend(err) => Some(err),
        }
    }
}

impl From<kernel::KernelError> for CoreError {
    fn from(err: kernel::KernelError) -> Self {
        Self::Kernel(err)
    }
}

impl From<hlc::HlcError> for CoreError {
    fn from(err: hlc::HlcError) -> Self {
        Self::Hlc(err)
    }
}

impl From<gate::CommitGateError> for CoreError {
    fn from(err: gate::CommitGateError) -> Self {
        Self::Gate(err)
    }
}

impl From<spend::SpendError> for CoreError {
    fn from(err: spend::SpendError) -> Self {
        Self::Spend(err)
    }
}

// ---------------------------------------------------------------------------
// Public Re-Exports
// ---------------------------------------------------------------------------

// 3-Way Commit Gate
pub use gate::{CommitGate, CommitGateError, CommitReceipt, CommittedNode};

// Hybrid Logical Clock
pub use hlc::{current_physical_ms, HlcError, HlcTimestamp, HybridLogicalClock};

// Universal Provenance Kernel (all 20 entities, enums, JSON, hashing)
pub use kernel::{
    blake3, canonicalize_json, hex_encode, sha256, to_canonical_rfc8785_json, to_canonical_rfc8785_metadata,
    Action, Actor, ActorType, Approval, Authority, Constraint, Decision, Dependency,
    DependencyType, EnforcementLevel, Evidence, Exception, Goal, GoalStatus, HashAlgorithm,
    Intent, JsonNumber, JsonValue, KernelEntity, KernelError, Metric, NodeId, Observation,
    Outcome, OutcomeStatus, ProvenanceNode, ReviewRequired, ReviewStatus, ReviewUrgency, Role,
    RollbackStatus, RollbackStep, SandboxLevel, StateChange, Task, TaskStatus, ToolCall,
};

// Conversation Models
pub use conversation::{
    ConversationThread, Message, MessageSender, Platform, User, UserFact, UserStyleProfile,
    UserTier,
};

// Snapshot Engine
pub use snapshot::{current_epoch_millis, SnapshotEngine, SnapshotHandle, SnapshotNode};

// Spend Circuit Breaker
pub use spend::{
    microcents_to_usd, usd_to_microcents, Reservation, SpendCircuitBreaker, SpendError,
    UserQuotaExceeded, UserSpendQuota, UserSpendStatus, DEFAULT_CEILING_MICROCENTS,
    DEFAULT_CEILING_USD, DEFAULT_SAFETY_FACTOR, DEFAULT_STAGE_CEILING_MICROCENTS,
    DEFAULT_STAGE_CEILING_USD, DEFAULT_STAGE_TRIP_MICROCENTS, DEFAULT_TRIP_THRESHOLD_MICROCENTS,
    HALT_EXIT_CODE, MICROCENTS_PER_USD,
};

/// Prelude module for convenient single-import access to essential types.
pub mod prelude {
    pub use crate::gate::{CommitGate, CommitGateError, CommitReceipt, CommittedNode};
    pub use crate::hlc::{current_physical_ms, HlcError, HlcTimestamp, HybridLogicalClock};
    pub use crate::json;
    pub use crate::kernel::{
        blake3, canonicalize_json, sha256, to_canonical_rfc8785_json, Action, Actor, ActorType,
        Approval, Authority, Constraint, Decision, Dependency, DependencyType, EnforcementLevel,
        Evidence, Exception, Goal, GoalStatus, HashAlgorithm, Intent, JsonNumber, JsonValue,
        KernelEntity, KernelError, Metric, NodeId, Observation, Outcome, OutcomeStatus,
        ProvenanceNode, ReviewRequired, ReviewStatus, ReviewUrgency, Role, RollbackStatus,
        RollbackStep, SandboxLevel, StateChange, Task, TaskStatus, ToolCall,
    };
    pub use crate::snapshot::{SnapshotEngine, SnapshotHandle, SnapshotNode};
    pub use crate::spend::{
        microcents_to_usd, usd_to_microcents, Reservation, SpendCircuitBreaker, SpendError,
        DEFAULT_CEILING_MICROCENTS, DEFAULT_CEILING_USD, DEFAULT_SAFETY_FACTOR,
        DEFAULT_STAGE_CEILING_MICROCENTS, DEFAULT_STAGE_TRIP_MICROCENTS,
        DEFAULT_TRIP_THRESHOLD_MICROCENTS, HALT_EXIT_CODE, MICROCENTS_PER_USD,
    };
    pub use crate::{
        CoreError, CoreResult, DigestHex, LogicalCounter, Microcents, MillisecondTimestamp,
    };
}
