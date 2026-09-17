//! The Universal 20-Entity Provenance Kernel.
//!
//! Implements strongly typed data models for all 20 provenance entities,
//! canonical RFC 8785 JSON serialization, content-addressed node ID computation
//! (BLAKE3 and SHA-256), and domain invariant validation.

use crate::hlc::HlcTimestamp;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Errors arising during provenance kernel operations.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KernelError {
    /// Content address mismatch: calculated digest does not match declared node ID.
    ContentAddressMismatch {
        /// Declared node ID.
        declared: String,
        /// Computed digest.
        computed: String,
    },

    /// Validation error for required fields or domain invariants.
    ValidationError {
        /// Entity type name.
        entity_type: &'static str,
        /// Description of validation failure.
        reason: String,
    },

    /// JSON serialization or canonicalization failure.
    SerializationError(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentAddressMismatch { declared, computed } => write!(
                f,
                "Content address mismatch: declared={}, computed={}",
                declared, computed
            ),
            Self::ValidationError {
                entity_type,
                reason,
            } => write!(
                f,
                "Validation failure for entity '{}': {}",
                entity_type, reason
            ),
            Self::SerializationError(msg) => {
                write!(f, "Canonical serialization failure: {}", msg)
            }
        }
    }
}

impl Error for KernelError {}

/// Cryptographic hash algorithm used for content-addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// BLAKE3 256-bit cryptographic digest (default high-performance).
    Blake3,
    /// SHA-256 standard cryptographic digest.
    Sha256,
}

/// Content-addressed identifier for a provenance node.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NodeId(pub String);

impl NodeId {
    /// Creates a NodeId from an existing string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the string slice representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

// ---------------------------------------------------------------------------
// Pure Rust JSON & RFC 8785 Canonicalization
// ---------------------------------------------------------------------------

/// Lightweight JSON Value tree using standard Rust types.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// JSON number (integer or floating-point with exact representation)
    Number(JsonNumber),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<JsonValue>),
    /// JSON object (ordered lexicographically by key for RFC 8785)
    Object(BTreeMap<String, JsonValue>),
}

/// A numeric value in JSON preserving exact formatting.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonNumber(String);

impl JsonNumber {
    /// Creates a JsonNumber from raw string representation.
    pub fn from_raw(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Formats as string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<i64> for JsonNumber {
    fn from(v: i64) -> Self {
        Self(v.to_string())
    }
}

impl From<i32> for JsonNumber {
    fn from(v: i32) -> Self {
        Self(v.to_string())
    }
}

impl From<u64> for JsonNumber {
    fn from(v: u64) -> Self {
        Self(v.to_string())
    }
}

impl From<u32> for JsonNumber {
    fn from(v: u32) -> Self {
        Self(v.to_string())
    }
}

impl From<f64> for JsonNumber {
    fn from(v: f64) -> Self {
        if v.is_nan() || v.is_infinite() {
            Self("0".to_string())
        } else {
            let mut s = format!("{}", v);
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                s.push_str(".0");
            }
            Self(s)
        }
    }
}

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i32> for JsonValue {
    fn from(n: i32) -> Self {
        Self::Number(JsonNumber::from_raw(n.to_string()))
    }
}

impl From<i64> for JsonValue {
    fn from(n: i64) -> Self {
        Self::Number(JsonNumber::from_raw(n.to_string()))
    }
}

impl From<u64> for JsonValue {
    fn from(n: u64) -> Self {
        Self::Number(JsonNumber::from_raw(n.to_string()))
    }
}

impl From<f64> for JsonValue {
    fn from(n: f64) -> Self {
        Self::Number(JsonNumber::from(n))
    }
}

/// Macro for creating JSON values ergonomically without external crates.
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::kernel::JsonValue::Null
    };
    (true) => {
        $crate::kernel::JsonValue::Bool(true)
    };
    (false) => {
        $crate::kernel::JsonValue::Bool(false)
    };
    ([ $($elem:tt),* $(,)? ]) => {
        $crate::kernel::JsonValue::Array(vec![ $( $crate::json!($elem) ),* ])
    };
    ({ $($key:tt : $val:tt),* $(,)? }) => {
        {
            let mut map = std::collections::BTreeMap::new();
            $(
                let k = stringify!($key);
                let cleaned_k = if k.starts_with('"') && k.ends_with('"') && k.len() >= 2 {
                    k[1..k.len()-1].to_string()
                } else {
                    k.to_string()
                };
                map.insert(cleaned_k, $crate::json!($val));
            )*
            $crate::kernel::JsonValue::Object(map)
        }
    };
    ($other:expr) => {
        $crate::kernel::JsonValue::from($other)
    };
}

/// Escapes a string slice for RFC 8785 JSON output.
pub fn escape_json_string(s: &str) -> String {
    let mut buf = String::with_capacity(s.len() + 8);
    buf.push('"');
    for c in s.chars() {
        match c {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\x08' => buf.push_str("\\b"),
            '\x0C' => buf.push_str("\\f"),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                buf.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => buf.push(c),
        }
    }
    buf.push('"');
    buf
}

/// Canonicalizes a `JsonValue` according to RFC 8785 (JCS):
/// 1. Lexicographical key sorting (guaranteed by BTreeMap).
/// 2. Zero extraneous whitespace.
/// 3. Deterministic string escaping.
pub fn canonicalize_json(val: &JsonValue) -> String {
    match val {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(true) => "true".to_string(),
        JsonValue::Bool(false) => "false".to_string(),
        JsonValue::Number(num) => num.as_str().to_string(),
        JsonValue::String(s) => escape_json_string(s),
        JsonValue::Array(arr) => {
            let mut buf = String::from("[");
            for (idx, item) in arr.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                buf.push_str(&canonicalize_json(item));
            }
            buf.push(']');
            buf
        }
        JsonValue::Object(map) => {
            let mut buf = String::from("{");
            for (idx, (k, v)) in map.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                buf.push_str(&escape_json_string(k));
                buf.push(':');
                buf.push_str(&canonicalize_json(v));
            }
            buf.push('}');
            buf
        }
    }
}

/// Converts a BTreeMap of metadata key-values into canonical RFC 8785 JSON.
pub fn to_canonical_rfc8785_metadata(metadata: &BTreeMap<String, String>) -> String {
    let mut buf = String::from("{");
    for (idx, (k, v)) in metadata.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push_str(&escape_json_string(k));
        buf.push(':');
        buf.push_str(&escape_json_string(v));
    }
    buf.push('}');
    buf
}

// ---------------------------------------------------------------------------
// 20 Domain Entity Structs
// ---------------------------------------------------------------------------

/// Entity 1: Actor - Autonomous agent, human developer, or external service.
#[derive(Clone, Debug, PartialEq)]
pub struct Actor {
    /// Unique actor identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Classification of actor.
    pub actor_type: ActorType,
    /// Optional Ed25519 public key hex for cryptographic identity.
    pub public_key: Option<String>,
    /// Assigned role identifier.
    pub role_id: Option<String>,
}

/// Classification of an Actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorType {
    /// AI Agent.
    Agent,
    /// Human User.
    Human,
    /// Background Service / Daemon.
    Service,
    /// Core System Kernel.
    System,
}

impl ActorType {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Human => "Human",
            Self::Service => "Service",
            Self::System => "System",
        }
    }
}

/// Entity 2: Role - Permissions and trust level assigned to actors.
#[derive(Clone, Debug, PartialEq)]
pub struct Role {
    /// Unique role identifier.
    pub id: String,
    /// Role name (e.g., "Engineer", "Reviewer", "Orchestrator").
    pub name: String,
    /// Allowed capability identifiers.
    pub permissions: Vec<String>,
    /// Maximum spend authorized in microcents ($1 = 1,000,000 microcents).
    pub max_spend_microcents: u64,
    /// Trust score from 0 to 100.
    pub trust_level: u8,
}

/// Entity 3: Authority - Delegated authority grants between actors.
#[derive(Clone, Debug, PartialEq)]
pub struct Authority {
    /// Unique grant ID.
    pub id: String,
    /// Actor granting authority.
    pub granter_id: String,
    /// Actor receiving authority.
    pub grantee_id: String,
    /// Scopes granted.
    pub scope: Vec<String>,
    /// Validity window start (Unix ms).
    pub valid_from_ms: u64,
    /// Validity window end (Unix ms).
    pub valid_until_ms: u64,
    /// Optional Ed25519 delegation signature.
    pub signature: Option<String>,
}

/// Entity 4: Intent - High-level user prompt or autonomous objective.
#[derive(Clone, Debug, PartialEq)]
pub struct Intent {
    /// Unique intent ID.
    pub id: String,
    /// Parent session identifier.
    pub session_id: String,
    /// Natural language prompt.
    pub prompt: String,
    /// Optional raw input context.
    pub raw_input: Option<String>,
    /// Optional embedding reference ID.
    pub embedding_id: Option<String>,
}

/// Entity 5: Goal - Structured goal decomposed from an intent.
#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    /// Unique goal ID.
    pub id: String,
    /// Parent intent reference.
    pub intent_id: String,
    /// Goal description.
    pub description: String,
    /// Success verification criteria.
    pub success_criteria: Vec<String>,
    /// Lifecycle status.
    pub status: GoalStatus,
}

/// Status of a Goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalStatus {
    /// Goal active.
    Active,
    /// Goal completed.
    Completed,
    /// Goal abandoned.
    Abandoned,
    /// Goal failed.
    Failed,
}

impl GoalStatus {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::Abandoned => "Abandoned",
            Self::Failed => "Failed",
        }
    }
}

/// Entity 6: Task - Concrete unit of work assigned to an actor.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Unique task ID.
    pub id: String,
    /// Parent goal reference.
    pub goal_id: String,
    /// Task description.
    pub description: String,
    /// Assigned actor ID.
    pub assigned_actor_id: Option<String>,
    /// Execution deadline (Unix ms).
    pub deadline_ms: Option<u64>,
    /// Lifecycle status.
    pub status: TaskStatus,
}

/// Status of a Task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// Pending dispatch.
    Pending,
    /// Currently running.
    InProgress,
    /// Succeeded.
    Succeeded,
    /// Failed.
    Failed,
    /// Blocked by dependency.
    Blocked,
}

impl TaskStatus {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "InProgress",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::Blocked => "Blocked",
        }
    }
}

/// Entity 7: Action - An atomic action executed by an agent.
#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    /// Unique action ID.
    pub id: String,
    /// Parent task reference.
    pub task_id: String,
    /// Action category/type.
    pub action_type: String,
    /// Description of action.
    pub description: String,
    /// Action parameters.
    pub parameters: JsonValue,
}

/// Entity 8: ToolCall - External tool invocation record.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// Unique tool call ID.
    pub id: String,
    /// Parent action reference.
    pub action_id: String,
    /// Name of tool invoked.
    pub tool_name: String,
    /// Arguments passed to tool.
    pub arguments: JsonValue,
    /// Execution timeout in milliseconds.
    pub timeout_ms: u64,
    /// Applied sandbox isolation.
    pub sandbox_level: SandboxLevel,
}

/// Sandbox isolation level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxLevel {
    /// No isolation.
    None,
    /// Read-only filesystem.
    ReadOnly,
    /// Git worktree isolated shadow workspace.
    WorktreeIsolated,
    /// Full containerized sandbox.
    FullSandbox,
}

impl SandboxLevel {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::ReadOnly => "ReadOnly",
            Self::WorktreeIsolated => "WorktreeIsolated",
            Self::FullSandbox => "FullSandbox",
        }
    }
}

/// Entity 9: RollbackStep - SAGA compensation step.
#[derive(Clone, Debug, PartialEq)]
pub struct RollbackStep {
    /// Unique rollback step ID.
    pub id: String,
    /// Parent task reference.
    pub task_id: String,
    /// Sequence index in LIFO rollback stack.
    pub step_index: u32,
    /// Shell/API command to reverse state.
    pub compensation_command: String,
    /// Affected file or resource path.
    pub affected_resource: String,
    /// Execution status.
    pub status: RollbackStatus,
}

/// Status of a rollback step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackStatus {
    /// Ready for execution if abort triggers.
    Pending,
    /// Successfully executed during rollback.
    Executed,
    /// Failed during rollback.
    Failed,
    /// Skipped.
    Skipped,
}

impl RollbackStatus {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Executed => "Executed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }
}

/// Entity 10: Observation - Observed output from an action or tool.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// Unique observation ID.
    pub id: String,
    /// Action observed.
    pub action_id: String,
    /// Captured stdout.
    pub stdout: Option<String>,
    /// Captured stderr.
    pub stderr: Option<String>,
    /// Subprocess exit code.
    pub exit_code: Option<i32>,
    /// Execution duration in ms.
    pub latency_ms: u64,
}

/// Entity 11: Evidence - Cryptographic proof or attestation.
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    /// Unique evidence ID.
    pub id: String,
    /// Optional observation reference.
    pub observation_id: Option<String>,
    /// Evidence classification.
    pub evidence_type: String,
    /// Cryptographic hash or signature digest.
    pub digest: String,
    /// Verification method used.
    pub verification_method: String,
}

/// Entity 12: StateChange - Recorded mutation on an external resource.
#[derive(Clone, Debug, PartialEq)]
pub struct StateChange {
    /// Unique change ID.
    pub id: String,
    /// Entity or resource mutated.
    pub entity_ref: String,
    /// Pre-state cryptographic hash.
    pub before_hash: String,
    /// Post-state cryptographic hash.
    pub after_hash: String,
    /// Relative path of affected file.
    pub path: String,
    /// Optional unified diff or patch.
    pub patch: Option<String>,
}

/// Entity 13: Metric - Quantitative performance or cost metric.
#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    /// Unique metric ID.
    pub id: String,
    /// Metric name.
    pub metric_name: String,
    /// Recorded value.
    pub value: f64,
    /// Metric unit.
    pub unit: String,
    /// Associated spend in microcents.
    pub spend_microcents: Option<u64>,
}

/// Entity 14: Decision - Agent choice between competing options.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// Unique decision ID.
    pub id: String,
    /// Parent task reference.
    pub task_id: Option<String>,
    /// Option selected by agent.
    pub chosen_option: String,
    /// Rejected alternative options.
    pub rejected_options: Vec<String>,
    /// Justification or reasoning.
    pub rationale: String,
}

/// Entity 15: Dependency - Causal or temporal dependency edge.
#[derive(Clone, Debug, PartialEq)]
pub struct Dependency {
    /// Unique dependency ID.
    pub id: String,
    /// Prerequisite node ID.
    pub source_node_id: String,
    /// Dependent node ID.
    pub target_node_id: String,
    /// Type of dependency.
    pub dependency_type: DependencyType,
}

/// Classification of dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyType {
    /// Hard execution blocker.
    Hard,
    /// Advisory dependency.
    Soft,
    /// Causal graph edge.
    Causal,
    /// Temporal ordering edge.
    Temporal,
}

impl DependencyType {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hard => "Hard",
            Self::Soft => "Soft",
            Self::Causal => "Causal",
            Self::Temporal => "Temporal",
        }
    }
}

/// Entity 16: Constraint - Safety policy or spend limit rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Constraint {
    /// Unique constraint ID.
    pub id: String,
    /// Constraint name.
    pub name: String,
    /// Invariant expression or rule.
    pub expression: String,
    /// Enforcement behavior on breach.
    pub enforcement_level: EnforcementLevel,
    /// Whether constraint is currently active.
    pub is_active: bool,
}

/// Enforcement action when constraint is violated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnforcementLevel {
    /// Reject operation and abort.
    Strict,
    /// Log warning but permit operation.
    WarnOnly,
    /// Terminate process immediately.
    Halt,
}

impl EnforcementLevel {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "Strict",
            Self::WarnOnly => "WarnOnly",
            Self::Halt => "Halt",
        }
    }
}

/// Entity 17: Approval - Human or supervisor authorization.
#[derive(Clone, Debug, PartialEq)]
pub struct Approval {
    /// Unique approval ID.
    pub id: String,
    /// Task approved.
    pub task_id: String,
    /// Approving actor ID.
    pub approver_id: String,
    /// Decision (true = approved).
    pub approved: bool,
    /// Approval rationale.
    pub reason: Option<String>,
    /// Optional cryptographic signature.
    pub signature: Option<String>,
}

/// Entity 18: Exception - Runtime error or anomaly during execution.
#[derive(Clone, Debug, PartialEq)]
pub struct Exception {
    /// Unique exception ID.
    pub id: String,
    /// Standard error code.
    pub error_code: String,
    /// Diagnostic error message.
    pub message: String,
    /// Optional stack trace.
    pub stack_trace: Option<String>,
    /// Whether exception is fatal.
    pub fatal: bool,
}

/// Entity 19: Outcome - Final verified result of a goal.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// Unique outcome ID.
    pub id: String,
    /// Goal evaluated.
    pub goal_id: String,
    /// Verification status.
    pub status: OutcomeStatus,
    /// Optional OVT ticket reference.
    pub ovt_ticket: Option<String>,
    /// Verification timestamp (Unix ms).
    pub verified_at_ms: Option<u64>,
}

/// Status of an Outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeStatus {
    /// Goal fully achieved.
    Success,
    /// Goal failed.
    Failure,
    /// Partial progress achieved.
    Partial,
}

impl OutcomeStatus {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Failure => "Failure",
            Self::Partial => "Partial",
        }
    }
}

/// Entity 20: ReviewRequired - Human review gate trigger.
#[derive(Clone, Debug, PartialEq)]
pub struct ReviewRequired {
    /// Unique review request ID.
    pub id: String,
    /// Trigger reason.
    pub reason: String,
    /// Required reviewer role.
    pub required_role: String,
    /// Urgency classification.
    pub urgency: ReviewUrgency,
    /// Gate review status.
    pub status: ReviewStatus,
}

/// Urgency of human review.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewUrgency {
    /// Low priority.
    Low,
    /// Normal priority.
    Medium,
    /// Urgent attention required.
    High,
    /// Critical blocker.
    Critical,
}

impl ReviewUrgency {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

/// Status of review gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewStatus {
    /// Awaiting human action.
    Pending,
    /// Currently being reviewed.
    InReview,
    /// Approved by human.
    Approved,
    /// Rejected by human.
    Rejected,
}

impl ReviewStatus {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InReview => "InReview",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
        }
    }
}

// ---------------------------------------------------------------------------
// 20-Entity Kernel Enum & Validation
// ---------------------------------------------------------------------------

/// Universal tagged union of all 20 provenance kernel entities.
#[derive(Clone, Debug, PartialEq)]
pub enum KernelEntity {
    /// Actor
    Actor(Actor),
    /// Role
    Role(Role),
    /// Authority
    Authority(Authority),
    /// Intent
    Intent(Intent),
    /// Goal
    Goal(Goal),
    /// Task
    Task(Task),
    /// Action
    Action(Action),
    /// ToolCall
    ToolCall(ToolCall),
    /// RollbackStep
    RollbackStep(RollbackStep),
    /// Observation
    Observation(Observation),
    /// Evidence
    Evidence(Evidence),
    /// StateChange
    StateChange(StateChange),
    /// Metric
    Metric(Metric),
    /// Decision
    Decision(Decision),
    /// Dependency
    Dependency(Dependency),
    /// Constraint
    Constraint(Constraint),
    /// Approval
    Approval(Approval),
    /// Exception
    Exception(Exception),
    /// Outcome
    Outcome(Outcome),
    /// ReviewRequired
    ReviewRequired(ReviewRequired),
}

impl KernelEntity {
    /// Returns the variant type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Actor(_) => "Actor",
            Self::Role(_) => "Role",
            Self::Authority(_) => "Authority",
            Self::Intent(_) => "Intent",
            Self::Goal(_) => "Goal",
            Self::Task(_) => "Task",
            Self::Action(_) => "Action",
            Self::ToolCall(_) => "ToolCall",
            Self::RollbackStep(_) => "RollbackStep",
            Self::Observation(_) => "Observation",
            Self::Evidence(_) => "Evidence",
            Self::StateChange(_) => "StateChange",
            Self::Metric(_) => "Metric",
            Self::Decision(_) => "Decision",
            Self::Dependency(_) => "Dependency",
            Self::Constraint(_) => "Constraint",
            Self::Approval(_) => "Approval",
            Self::Exception(_) => "Exception",
            Self::Outcome(_) => "Outcome",
            Self::ReviewRequired(_) => "ReviewRequired",
        }
    }

    /// Returns the unique ID of the wrapped entity.
    pub fn id(&self) -> &str {
        match self {
            Self::Actor(e) => &e.id,
            Self::Role(e) => &e.id,
            Self::Authority(e) => &e.id,
            Self::Intent(e) => &e.id,
            Self::Goal(e) => &e.id,
            Self::Task(e) => &e.id,
            Self::Action(e) => &e.id,
            Self::ToolCall(e) => &e.id,
            Self::RollbackStep(e) => &e.id,
            Self::Observation(e) => &e.id,
            Self::Evidence(e) => &e.id,
            Self::StateChange(e) => &e.id,
            Self::Metric(e) => &e.id,
            Self::Decision(e) => &e.id,
            Self::Dependency(e) => &e.id,
            Self::Constraint(e) => &e.id,
            Self::Approval(e) => &e.id,
            Self::Exception(e) => &e.id,
            Self::Outcome(e) => &e.id,
            Self::ReviewRequired(e) => &e.id,
        }
    }

    /// Validates domain invariants for the entity.
    pub fn validate(&self) -> Result<(), KernelError> {
        let err = |reason: String| KernelError::ValidationError {
            entity_type: self.type_name(),
            reason,
        };

        if self.id().is_empty() {
            return Err(err("Entity ID must not be empty".to_string()));
        }

        match self {
            Self::Actor(_) => Ok(()),
            Self::Role(r) => {
                if r.trust_level > 100 {
                    return Err(err(format!("Trust level {} exceeds 100", r.trust_level)));
                }
                Ok(())
            }
            Self::Authority(a) => {
                if a.granter_id.is_empty() {
                    return Err(err("Granter ID must not be empty".to_string()));
                }
                if a.grantee_id.is_empty() {
                    return Err(err("Grantee ID must not be empty".to_string()));
                }
                if a.valid_from_ms > a.valid_until_ms {
                    return Err(err("valid_from_ms must be <= valid_until_ms".to_string()));
                }
                Ok(())
            }
            Self::Intent(i) => {
                if i.prompt.trim().is_empty() {
                    return Err(err("Intent prompt must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Goal(g) => {
                if g.description.trim().is_empty() {
                    return Err(err("Goal description must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Task(t) => {
                if t.description.trim().is_empty() {
                    return Err(err("Task description must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Action(a) => {
                if a.action_type.trim().is_empty() {
                    return Err(err("Action type must not be empty".to_string()));
                }
                Ok(())
            }
            Self::ToolCall(tc) => {
                if tc.tool_name.trim().is_empty() {
                    return Err(err("Tool name must not be empty".to_string()));
                }
                Ok(())
            }
            Self::RollbackStep(rs) => {
                if rs.compensation_command.trim().is_empty() {
                    return Err(err("Compensation command must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Observation(o) => {
                if o.action_id.is_empty() {
                    return Err(err("Observation action_id must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Evidence(e) => {
                if e.digest.is_empty() {
                    return Err(err("Evidence digest must not be empty".to_string()));
                }
                Ok(())
            }
            Self::StateChange(sc) => {
                if sc.path.is_empty() {
                    return Err(err("StateChange path must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Metric(m) => {
                if m.metric_name.trim().is_empty() {
                    return Err(err("Metric name must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Decision(d) => {
                if d.chosen_option.trim().is_empty() {
                    return Err(err("Chosen option must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Dependency(dep) => {
                if dep.source_node_id.is_empty() || dep.target_node_id.is_empty() {
                    return Err(err("Dependency source and target IDs must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Constraint(c) => {
                if c.name.trim().is_empty() || c.expression.trim().is_empty() {
                    return Err(err("Constraint name and expression must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Approval(app) => {
                if app.approver_id.is_empty() {
                    return Err(err("Approver ID must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Exception(exc) => {
                if exc.error_code.trim().is_empty() {
                    return Err(err("Exception error_code must not be empty".to_string()));
                }
                Ok(())
            }
            Self::Outcome(out) => {
                if out.goal_id.is_empty() {
                    return Err(err("Outcome goal_id must not be empty".to_string()));
                }
                Ok(())
            }
            Self::ReviewRequired(rev) => {
                if rev.reason.trim().is_empty() || rev.required_role.trim().is_empty() {
                    return Err(err("Review reason and required_role must not be empty".to_string()));
                }
                Ok(())
            }
        }
    }

    /// Serializes this entity into canonical RFC 8785 JSON representation.
    pub fn to_canonical_json(&self) -> String {
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), JsonValue::String(self.type_name().to_string()));

        let mut data_map = BTreeMap::new();
        data_map.insert("id".to_string(), JsonValue::String(self.id().to_string()));

        match self {
            Self::Actor(a) => {
                data_map.insert("name".to_string(), JsonValue::String(a.name.clone()));
                data_map.insert("actor_type".to_string(), JsonValue::String(a.actor_type.as_str().to_string()));
                if let Some(ref pk) = a.public_key {
                    data_map.insert("public_key".to_string(), JsonValue::String(pk.clone()));
                }
                if let Some(ref r) = a.role_id {
                    data_map.insert("role_id".to_string(), JsonValue::String(r.clone()));
                }
            }
            Self::Role(r) => {
                data_map.insert("name".to_string(), JsonValue::String(r.name.clone()));
                data_map.insert(
                    "permissions".to_string(),
                    JsonValue::Array(r.permissions.iter().map(|p| JsonValue::String(p.clone())).collect()),
                );
                data_map.insert("max_spend_microcents".to_string(), JsonValue::from(r.max_spend_microcents));
                data_map.insert("trust_level".to_string(), JsonValue::from(r.trust_level as u64));
            }
            Self::Authority(a) => {
                data_map.insert("granter_id".to_string(), JsonValue::String(a.granter_id.clone()));
                data_map.insert("grantee_id".to_string(), JsonValue::String(a.grantee_id.clone()));
                data_map.insert(
                    "scope".to_string(),
                    JsonValue::Array(a.scope.iter().map(|s| JsonValue::String(s.clone())).collect()),
                );
                data_map.insert("valid_from_ms".to_string(), JsonValue::from(a.valid_from_ms));
                data_map.insert("valid_until_ms".to_string(), JsonValue::from(a.valid_until_ms));
                if let Some(ref s) = a.signature {
                    data_map.insert("signature".to_string(), JsonValue::String(s.clone()));
                }
            }
            Self::Intent(i) => {
                data_map.insert("session_id".to_string(), JsonValue::String(i.session_id.clone()));
                data_map.insert("prompt".to_string(), JsonValue::String(i.prompt.clone()));
                if let Some(ref r) = i.raw_input {
                    data_map.insert("raw_input".to_string(), JsonValue::String(r.clone()));
                }
                if let Some(ref e) = i.embedding_id {
                    data_map.insert("embedding_id".to_string(), JsonValue::String(e.clone()));
                }
            }
            Self::Goal(g) => {
                data_map.insert("intent_id".to_string(), JsonValue::String(g.intent_id.clone()));
                data_map.insert("description".to_string(), JsonValue::String(g.description.clone()));
                data_map.insert(
                    "success_criteria".to_string(),
                    JsonValue::Array(g.success_criteria.iter().map(|s| JsonValue::String(s.clone())).collect()),
                );
                data_map.insert("status".to_string(), JsonValue::String(g.status.as_str().to_string()));
            }
            Self::Task(t) => {
                data_map.insert("goal_id".to_string(), JsonValue::String(t.goal_id.clone()));
                data_map.insert("description".to_string(), JsonValue::String(t.description.clone()));
                if let Some(ref a) = t.assigned_actor_id {
                    data_map.insert("assigned_actor_id".to_string(), JsonValue::String(a.clone()));
                }
                if let Some(d) = t.deadline_ms {
                    data_map.insert("deadline_ms".to_string(), JsonValue::from(d));
                }
                data_map.insert("status".to_string(), JsonValue::String(t.status.as_str().to_string()));
            }
            Self::Action(a) => {
                data_map.insert("task_id".to_string(), JsonValue::String(a.task_id.clone()));
                data_map.insert("action_type".to_string(), JsonValue::String(a.action_type.clone()));
                data_map.insert("description".to_string(), JsonValue::String(a.description.clone()));
                data_map.insert("parameters".to_string(), a.parameters.clone());
            }
            Self::ToolCall(tc) => {
                data_map.insert("action_id".to_string(), JsonValue::String(tc.action_id.clone()));
                data_map.insert("tool_name".to_string(), JsonValue::String(tc.tool_name.clone()));
                data_map.insert("arguments".to_string(), tc.arguments.clone());
                data_map.insert("timeout_ms".to_string(), JsonValue::from(tc.timeout_ms));
                data_map.insert("sandbox_level".to_string(), JsonValue::String(tc.sandbox_level.as_str().to_string()));
            }
            Self::RollbackStep(rs) => {
                data_map.insert("task_id".to_string(), JsonValue::String(rs.task_id.clone()));
                data_map.insert("step_index".to_string(), JsonValue::from(rs.step_index as u64));
                data_map.insert("compensation_command".to_string(), JsonValue::String(rs.compensation_command.clone()));
                data_map.insert("affected_resource".to_string(), JsonValue::String(rs.affected_resource.clone()));
                data_map.insert("status".to_string(), JsonValue::String(rs.status.as_str().to_string()));
            }
            Self::Observation(o) => {
                data_map.insert("action_id".to_string(), JsonValue::String(o.action_id.clone()));
                if let Some(ref s) = o.stdout {
                    data_map.insert("stdout".to_string(), JsonValue::String(s.clone()));
                }
                if let Some(ref s) = o.stderr {
                    data_map.insert("stderr".to_string(), JsonValue::String(s.clone()));
                }
                if let Some(ec) = o.exit_code {
                    data_map.insert("exit_code".to_string(), JsonValue::from(ec));
                }
                data_map.insert("latency_ms".to_string(), JsonValue::from(o.latency_ms));
            }
            Self::Evidence(e) => {
                if let Some(ref obs) = e.observation_id {
                    data_map.insert("observation_id".to_string(), JsonValue::String(obs.clone()));
                }
                data_map.insert("evidence_type".to_string(), JsonValue::String(e.evidence_type.clone()));
                data_map.insert("digest".to_string(), JsonValue::String(e.digest.clone()));
                data_map.insert("verification_method".to_string(), JsonValue::String(e.verification_method.clone()));
            }
            Self::StateChange(sc) => {
                data_map.insert("entity_ref".to_string(), JsonValue::String(sc.entity_ref.clone()));
                data_map.insert("before_hash".to_string(), JsonValue::String(sc.before_hash.clone()));
                data_map.insert("after_hash".to_string(), JsonValue::String(sc.after_hash.clone()));
                data_map.insert("path".to_string(), JsonValue::String(sc.path.clone()));
                if let Some(ref p) = sc.patch {
                    data_map.insert("patch".to_string(), JsonValue::String(p.clone()));
                }
            }
            Self::Metric(m) => {
                data_map.insert("metric_name".to_string(), JsonValue::String(m.metric_name.clone()));
                data_map.insert("value".to_string(), JsonValue::from(m.value));
                data_map.insert("unit".to_string(), JsonValue::String(m.unit.clone()));
                if let Some(sp) = m.spend_microcents {
                    data_map.insert("spend_microcents".to_string(), JsonValue::from(sp));
                }
            }
            Self::Decision(d) => {
                if let Some(ref t) = d.task_id {
                    data_map.insert("task_id".to_string(), JsonValue::String(t.clone()));
                }
                data_map.insert("chosen_option".to_string(), JsonValue::String(d.chosen_option.clone()));
                data_map.insert(
                    "rejected_options".to_string(),
                    JsonValue::Array(d.rejected_options.iter().map(|r| JsonValue::String(r.clone())).collect()),
                );
                data_map.insert("rationale".to_string(), JsonValue::String(d.rationale.clone()));
            }
            Self::Dependency(dep) => {
                data_map.insert("source_node_id".to_string(), JsonValue::String(dep.source_node_id.clone()));
                data_map.insert("target_node_id".to_string(), JsonValue::String(dep.target_node_id.clone()));
                data_map.insert("dependency_type".to_string(), JsonValue::String(dep.dependency_type.as_str().to_string()));
            }
            Self::Constraint(c) => {
                data_map.insert("name".to_string(), JsonValue::String(c.name.clone()));
                data_map.insert("expression".to_string(), JsonValue::String(c.expression.clone()));
                data_map.insert("enforcement_level".to_string(), JsonValue::String(c.enforcement_level.as_str().to_string()));
                data_map.insert("is_active".to_string(), JsonValue::Bool(c.is_active));
            }
            Self::Approval(app) => {
                data_map.insert("task_id".to_string(), JsonValue::String(app.task_id.clone()));
                data_map.insert("approver_id".to_string(), JsonValue::String(app.approver_id.clone()));
                data_map.insert("approved".to_string(), JsonValue::Bool(app.approved));
                if let Some(ref r) = app.reason {
                    data_map.insert("reason".to_string(), JsonValue::String(r.clone()));
                }
                if let Some(ref s) = app.signature {
                    data_map.insert("signature".to_string(), JsonValue::String(s.clone()));
                }
            }
            Self::Exception(exc) => {
                data_map.insert("error_code".to_string(), JsonValue::String(exc.error_code.clone()));
                data_map.insert("message".to_string(), JsonValue::String(exc.message.clone()));
                if let Some(ref st) = exc.stack_trace {
                    data_map.insert("stack_trace".to_string(), JsonValue::String(st.clone()));
                }
                data_map.insert("fatal".to_string(), JsonValue::Bool(exc.fatal));
            }
            Self::Outcome(out) => {
                data_map.insert("goal_id".to_string(), JsonValue::String(out.goal_id.clone()));
                data_map.insert("status".to_string(), JsonValue::String(out.status.as_str().to_string()));
                if let Some(ref o) = out.ovt_ticket {
                    data_map.insert("ovt_ticket".to_string(), JsonValue::String(o.clone()));
                }
                if let Some(v) = out.verified_at_ms {
                    data_map.insert("verified_at_ms".to_string(), JsonValue::from(v));
                }
            }
            Self::ReviewRequired(rev) => {
                data_map.insert("reason".to_string(), JsonValue::String(rev.reason.clone()));
                data_map.insert("required_role".to_string(), JsonValue::String(rev.required_role.clone()));
                data_map.insert("urgency".to_string(), JsonValue::String(rev.urgency.as_str().to_string()));
                data_map.insert("status".to_string(), JsonValue::String(rev.status.as_str().to_string()));
            }
        }

        map.insert("data".to_string(), JsonValue::Object(data_map));
        canonicalize_json(&JsonValue::Object(map))
    }
}

/// Helper that converts any `JsonValue` to canonical RFC 8785 JSON.
pub fn to_canonical_rfc8785_json(val: &JsonValue) -> Result<String, KernelError> {
    Ok(canonicalize_json(val))
}

// ---------------------------------------------------------------------------
// Pure Rust Self-Contained Cryptographic Hash Algorithms
// ---------------------------------------------------------------------------

/// Pure Rust SHA-256 implementation conforming to FIPS 180-4.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(data.len() + 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Computes RFC 2104 HMAC-SHA256 over arbitrary message bytes using a secret key.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hashed = sha256(key);
        k[..32].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut inner = Vec::with_capacity(64 + message.len());
    for b in &k {
        inner.push(b ^ 0x36);
    }
    inner.extend_from_slice(message);
    let inner_hash = sha256(&inner);

    let mut outer = Vec::with_capacity(64 + 32);
    for b in &k {
        outer.push(b ^ 0x5c);
    }
    outer.extend_from_slice(&inner_hash);
    sha256(&outer)
}

/// Constant-time byte slice comparison to prevent timing attacks.
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Pure Rust BLAKE3 cryptographic hash implementation.
pub fn blake3(data: &[u8]) -> [u8; 32] {
    const IV: [u32; 8] = [
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
        0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
    ];

    const CHUNK_START: u32 = 1;
    const CHUNK_END: u32 = 2;
    const ROOT: u32 = 8;

    #[inline(always)]
    fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
        state[a] = state[a].wrapping_add(state[b]).wrapping_add(mx);
        state[d] = (state[d] ^ state[a]).rotate_right(16);
        state[c] = state[c].wrapping_add(state[d]);
        state[b] = (state[b] ^ state[c]).rotate_right(12);
        state[a] = state[a].wrapping_add(state[b]).wrapping_add(my);
        state[d] = (state[d] ^ state[a]).rotate_right(8);
        state[c] = state[c].wrapping_add(state[d]);
        state[b] = (state[b] ^ state[c]).rotate_right(7);
    }

    #[inline(always)]
    fn round(state: &mut [u32; 16], m: &[u32; 16]) {
        g(state, 0, 4, 8, 12, m[0], m[1]);
        g(state, 1, 5, 9, 13, m[2], m[3]);
        g(state, 2, 6, 10, 14, m[4], m[5]);
        g(state, 3, 7, 11, 15, m[6], m[7]);
        g(state, 0, 5, 10, 15, m[8], m[9]);
        g(state, 1, 6, 11, 12, m[10], m[11]);
        g(state, 2, 7, 8, 13, m[12], m[13]);
        g(state, 3, 4, 9, 14, m[14], m[15]);
    }

    const SIGMA: [[usize; 16]; 7] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8],
        [3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1],
        [10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6],
        [12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4],
        [9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7],
        [11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13],
    ];

    fn compress(
        cv: &[u32; 8],
        block: &[u8],
        counter: u64,
        block_len: u32,
        flags: u32,
    ) -> [u32; 16] {
        let mut m = [0u32; 16];
        for i in 0..16 {
            if (i * 4 + 4) <= block.len() {
                m[i] = u32::from_le_bytes([block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3]]);
            } else if (i * 4) < block.len() {
                let mut b = [0u8; 4];
                let rem = block.len() - i * 4;
                b[..rem].copy_from_slice(&block[i * 4..]);
                m[i] = u32::from_le_bytes(b);
            }
        }

        let mut state = [
            cv[0], cv[1], cv[2], cv[3], cv[4], cv[5], cv[6], cv[7],
            IV[0], IV[1], IV[2], IV[3],
            counter as u32,
            (counter >> 32) as u32,
            block_len,
            flags,
        ];

        for r in 0..7 {
            let mut perm = [0u32; 16];
            for i in 0..16 {
                perm[i] = m[SIGMA[r][i]];
            }
            round(&mut state, &perm);
        }

        let mut out = [0u32; 16];
        for i in 0..8 {
            out[i] = state[i] ^ state[i + 8];
            out[i + 8] = state[i + 8] ^ cv[i];
        }
        out
    }

    if data.is_empty() {
        let out = compress(&IV, &[], 0, 0, CHUNK_START | CHUNK_END | ROOT);
        let mut bytes = [0u8; 32];
        for i in 0..8 {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&out[i].to_le_bytes());
        }
        return bytes;
    }

    // Process single chunk (data <= 1024 bytes) or multi-chunk
    let total_chunks = (data.len() + 1023) / 1024;
    let mut chunk_cvs = Vec::with_capacity(total_chunks);

    for (chunk_idx, chunk) in data.chunks(1024).enumerate() {
        let is_root = total_chunks == 1;
        let mut cur_cv = IV;
        let block_count = (chunk.len() + 63) / 64;

        for (b_idx, block) in chunk.chunks(64).enumerate() {
            let mut flags = 0u32;
            if b_idx == 0 {
                flags |= CHUNK_START;
            }
            if b_idx == block_count - 1 {
                flags |= CHUNK_END;
                if is_root {
                    flags |= ROOT;
                }
            }
            let out = compress(&cur_cv, block, chunk_idx as u64, block.len() as u32, flags);
            for i in 0..8 {
                cur_cv[i] = out[i];
            }
        }
        chunk_cvs.push(cur_cv);
    }

    if total_chunks == 1 {
        let mut bytes = [0u8; 32];
        for i in 0..8 {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&chunk_cvs[0][i].to_le_bytes());
        }
        return bytes;
    }

    // Parent tree reduction
    let mut current_level = chunk_cvs;
    while current_level.len() > 1 {
        let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
        for pair in current_level.chunks(2) {
            if pair.len() == 2 {
                let mut block = [0u8; 64];
                for i in 0..8 {
                    block[i * 4..i * 4 + 4].copy_from_slice(&pair[0][i].to_le_bytes());
                    block[32 + i * 4..32 + i * 4 + 4].copy_from_slice(&pair[1][i].to_le_bytes());
                }
                let is_root = current_level.len() == 2;
                let flags = 4 | (if is_root { ROOT } else { 0 }); // 4 = PARENT
                let out = compress(&IV, &block, 0, 64, flags);
                let mut parent_cv = [0u32; 8];
                for i in 0..8 {
                    parent_cv[i] = out[i];
                }
                next_level.push(parent_cv);
            } else {
                next_level.push(pair[0]);
            }
        }
        current_level = next_level;
    }

    let mut bytes = [0u8; 32];
    for i in 0..8 {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&current_level[0][i].to_le_bytes());
    }
    bytes
}

/// Hexadecimal encoding of a byte slice.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------------
// Provenance Node & Content-Addressed ID Computation
// ---------------------------------------------------------------------------

/// A content-addressed node in the Universal Provenance Merkle DAG.
#[derive(Clone, Debug, PartialEq)]
pub struct ProvenanceNode {
    /// Content-addressed node identifier (BLAKE3 or SHA-256 digest).
    pub id: NodeId,
    /// Strongly typed 20-entity kernel payload.
    pub entity: KernelEntity,
    /// Content-addressed IDs of direct causal parents.
    pub parents: Vec<NodeId>,
    /// Hybrid Logical Clock timestamp.
    pub hlc: HlcTimestamp,
    /// Canonical metadata key-value pairs.
    pub metadata: BTreeMap<String, String>,
}

impl ProvenanceNode {
    /// Constructs a new ProvenanceNode and calculates its content-addressed `id`.
    pub fn new(
        entity: KernelEntity,
        parents: Vec<NodeId>,
        hlc: HlcTimestamp,
        metadata: BTreeMap<String, String>,
        algo: HashAlgorithm,
    ) -> Result<Self, KernelError> {
        entity.validate()?;

        let mut node = Self {
            id: NodeId::new(""),
            entity,
            parents,
            hlc,
            metadata,
        };

        node.id = node.compute_content_id(algo)?;
        Ok(node)
    }

    /// Computes the content-addressed ID based on canonical RFC 8785 representation.
    pub fn compute_content_id(&self, algo: HashAlgorithm) -> Result<NodeId, KernelError> {
        let mut sorted_parents = self.parents.clone();
        sorted_parents.sort();

        let canonical_entity = self.entity.to_canonical_json();
        let canonical_metadata = to_canonical_rfc8785_metadata(&self.metadata);

        let preimage = format!(
            "{{\"entity\":{},\"hlc\":\"{}\",\"metadata\":{},\"parents\":[{}]}}",
            canonical_entity,
            self.hlc,
            canonical_metadata,
            sorted_parents
                .iter()
                .map(|p| format!("\"{}\"", p.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        );

        let digest_str = match algo {
            HashAlgorithm::Blake3 => {
                let digest = blake3(preimage.as_bytes());
                format!("blake3:{}", hex_encode(&digest))
            }
            HashAlgorithm::Sha256 => {
                let digest = sha256(preimage.as_bytes());
                format!("sha256:{}", hex_encode(&digest))
            }
        };

        Ok(NodeId::new(digest_str))
    }

    /// Verifies that the node's declared `id` matches its content-addressed digest.
    pub fn verify_content_address(&self) -> Result<(), KernelError> {
        let algo = if self.id.as_str().starts_with("sha256:") {
            HashAlgorithm::Sha256
        } else {
            HashAlgorithm::Blake3
        };

        let computed = self.compute_content_id(algo)?;
        if self.id != computed {
            return Err(KernelError::ContentAddressMismatch {
                declared: self.id.0.clone(),
                computed: computed.0,
            });
        }

        self.entity.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_golden_vector() {
        let empty_hash = sha256(b"");
        assert_eq!(
            hex_encode(&empty_hash),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let hello_hash = sha256(b"hello world");
        assert_eq!(
            hex_encode(&hello_hash),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_blake3_golden_vector() {
        let empty_hash = blake3(b"");
        assert_eq!(
            hex_encode(&empty_hash),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_rfc8785_canonical_json_sorting_and_compactness() {
        let mut inner_banana = BTreeMap::new();
        inner_banana.insert(
            "array".to_string(),
            JsonValue::Array(vec![
                JsonValue::from(3),
                JsonValue::from(2),
                JsonValue::from(1),
            ]),
        );
        inner_banana.insert("nested_a".to_string(), JsonValue::Null);
        inner_banana.insert("nested_z".to_string(), JsonValue::Bool(true));

        let mut root_map = BTreeMap::new();
        root_map.insert("zebra".to_string(), JsonValue::from(1));
        root_map.insert("apple".to_string(), JsonValue::from("pie"));
        root_map.insert("banana".to_string(), JsonValue::Object(inner_banana));
        root_map.insert("car".to_string(), JsonValue::from(42.0));

        let canonical = canonicalize_json(&JsonValue::Object(root_map));
        let expected = "{\"apple\":\"pie\",\"banana\":{\"array\":[3,2,1],\"nested_a\":null,\"nested_z\":true},\"car\":42.0,\"zebra\":1}";
        assert_eq!(canonical, expected, "Canonical RFC 8785 JSON output mismatch!");
    }

    #[test]
    fn test_all_20_entity_variants_validation() {
        let entities = vec![
            KernelEntity::Actor(Actor {
                id: "actor_01".into(),
                name: "Agent Smith".into(),
                actor_type: ActorType::Agent,
                public_key: None,
                role_id: Some("role_eng".into()),
            }),
            KernelEntity::Role(Role {
                id: "role_eng".into(),
                name: "Lead Engineer".into(),
                permissions: vec!["git:write".into()],
                max_spend_microcents: 5_000_000,
                trust_level: 95,
            }),
            KernelEntity::Authority(Authority {
                id: "auth_01".into(),
                granter_id: "human_admin".into(),
                grantee_id: "actor_01".into(),
                scope: vec!["repo:commit".into()],
                valid_from_ms: 1000,
                valid_until_ms: 2000,
                signature: None,
            }),
            KernelEntity::Intent(Intent {
                id: "intent_01".into(),
                session_id: "sess_01".into(),
                prompt: "Build kineti-core".into(),
                raw_input: None,
                embedding_id: None,
            }),
            KernelEntity::Goal(Goal {
                id: "goal_01".into(),
                intent_id: "intent_01".into(),
                description: "Complete testing strategy".into(),
                success_criteria: vec!["tests pass".into()],
                status: GoalStatus::Active,
            }),
            KernelEntity::Task(Task {
                id: "task_01".into(),
                goal_id: "goal_01".into(),
                description: "Write test_strategy.md".into(),
                assigned_actor_id: Some("actor_01".into()),
                deadline_ms: None,
                status: TaskStatus::InProgress,
            }),
            KernelEntity::Action(Action {
                id: "action_01".into(),
                task_id: "task_01".into(),
                action_type: "file_write".into(),
                description: "Write markdown file".into(),
                parameters: json!({"path": "test_strategy.md"}),
            }),
            KernelEntity::ToolCall(ToolCall {
                id: "tool_01".into(),
                action_id: "action_01".into(),
                tool_name: "write_to_file".into(),
                arguments: json!({"target": "test_strategy.md"}),
                timeout_ms: 5000,
                sandbox_level: SandboxLevel::WorktreeIsolated,
            }),
            KernelEntity::RollbackStep(RollbackStep {
                id: "rb_01".into(),
                task_id: "task_01".into(),
                step_index: 0,
                compensation_command: "git checkout -- test_strategy.md".into(),
                affected_resource: "test_strategy.md".into(),
                status: RollbackStatus::Pending,
            }),
            KernelEntity::Observation(Observation {
                id: "obs_01".into(),
                action_id: "action_01".into(),
                stdout: Some("File written".into()),
                stderr: None,
                exit_code: Some(0),
                latency_ms: 12,
            }),
            KernelEntity::Evidence(Evidence {
                id: "ev_01".into(),
                observation_id: Some("obs_01".into()),
                evidence_type: "hash_attestation".into(),
                digest: "blake3:abc12345".into(),
                verification_method: "blake3_sum".into(),
            }),
            KernelEntity::StateChange(StateChange {
                id: "sc_01".into(),
                entity_ref: "file:test_strategy.md".into(),
                before_hash: "00000000".into(),
                after_hash: "11111111".into(),
                path: "test_strategy.md".into(),
                patch: Some("+ # Testing Strategy".into()),
            }),
            KernelEntity::Metric(Metric {
                id: "metric_01".into(),
                metric_name: "test_coverage".into(),
                value: 99.8,
                unit: "percent".into(),
                spend_microcents: None,
            }),
            KernelEntity::Decision(Decision {
                id: "dec_01".into(),
                task_id: Some("task_01".into()),
                chosen_option: "EBR ArcSwap".into(),
                rejected_options: vec!["Mutex".into(), "RwLock".into()],
                rationale: "Wait-free reads under high writer contention".into(),
            }),
            KernelEntity::Dependency(Dependency {
                id: "dep_01".into(),
                source_node_id: "node_parent".into(),
                target_node_id: "node_child".into(),
                dependency_type: DependencyType::Causal,
            }),
            KernelEntity::Constraint(Constraint {
                id: "const_01".into(),
                name: "SpendCeiling".into(),
                expression: "spend <= 50.00".into(),
                enforcement_level: EnforcementLevel::Halt,
                is_active: true,
            }),
            KernelEntity::Approval(Approval {
                id: "app_01".into(),
                task_id: "task_01".into(),
                approver_id: "lead_reviewer".into(),
                approved: true,
                reason: Some("Specification verified".into()),
                signature: Some("sig_ed25519".into()),
            }),
            KernelEntity::Exception(Exception {
                id: "exc_01".into(),
                error_code: "ERR_SPEND_BREACH".into(),
                message: "Ceiling exceeded".into(),
                stack_trace: None,
                fatal: true,
            }),
            KernelEntity::Outcome(Outcome {
                id: "out_01".into(),
                goal_id: "goal_01".into(),
                status: OutcomeStatus::Success,
                ovt_ticket: Some("ovt_12345678".into()),
                verified_at_ms: Some(1726410000000),
            }),
            KernelEntity::ReviewRequired(ReviewRequired {
                id: "rev_01".into(),
                reason: "High spend reservation requested".into(),
                required_role: "FinanceAdmin".into(),
                urgency: ReviewUrgency::High,
                status: ReviewStatus::Pending,
            }),
        ];

        for (idx, entity) in entities.into_iter().enumerate() {
            assert!(
                entity.validate().is_ok(),
                "Validation failed for entity #{} ({})",
                idx + 1,
                entity.type_name()
            );
        }
    }

    #[test]
    fn test_content_address_tamper_detection() {
        let entity = KernelEntity::Actor(Actor {
            id: "actor_tamper_test".into(),
            name: "Original Name".into(),
            actor_type: ActorType::Agent,
            public_key: None,
            role_id: None,
        });

        let mut node = ProvenanceNode::new(
            entity,
            vec![],
            HlcTimestamp::new(1000, 1, 1),
            BTreeMap::new(),
            HashAlgorithm::Blake3,
        )
        .expect("Valid node creation");

        // Initial content address must verify
        assert!(node.verify_content_address().is_ok());

        // Tamper: Modify payload without recomputing ID
        if let KernelEntity::Actor(ref mut a) = node.entity {
            a.name = "Tampered Name".into();
        }

        // Verification must detect mismatch
        let res = node.verify_content_address();
        assert!(res.is_err());
        match res.unwrap_err() {
            KernelError::ContentAddressMismatch { declared, computed } => {
                assert_ne!(declared, computed);
            }
            other => panic!("Expected ContentAddressMismatch, got {:?}", other),
        }
    }
}
