//! # Epistemic Persona Engine (`epistemic`)
//!
//! Implements the 360º Human Model for personalized autonomous agents:
//! - **Multi-Scope Context**: Global, domain-specific (Health, Work, Finance, Schedule, Taste), and person-to-person relationships.
//! - **Epistemic Certainty Tiers**: `DirectlyKnown` (explicit) > `ObservedPattern` > `Inferred`.
//! - **Rule-Exception Hierarchies**: `BaselineRule` -> `PermittedException` -> `Preference` -> `SafetyCeiling`.
//! - **7 Epistemic Transitions**: Confirm, Narrow, Add Exception, Supersede, Conflict/Clarify, Weak Clue, Invalidate.
//! - **Strict Non-Destructive Invariants**: An inference can never override a directly known fact; conflicts require clarification.
//! - **High-Consequence Action Authorization**: Only `DirectlyKnown` facts or confirmed permissions can authorize high-consequence operations.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

/// Specific domain categories for scoped context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainKind {
    /// Physical well-being, nutrition, sleep, medical constraints.
    Health,
    /// Professional obligations, projects, team context.
    Work,
    /// Budgets, investments, payment constraints.
    Finance,
    /// Time allocations, meeting windows, availability.
    Schedule,
    /// Culinary, artistic, atmospheric preferences.
    Taste,
}

impl DomainKind {
    /// Returns the canonical string representation of this domain.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Work => "Work",
            Self::Finance => "Finance",
            Self::Schedule => "Schedule",
            Self::Taste => "Taste",
        }
    }
}

/// Context Scope partition preventing universal leakage of narrow facts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextScope {
    /// Universal baseline applying everywhere unless overridden.
    Global,
    /// Domain-specific context.
    Domain(DomainKind),
    /// Person-specific relationship context (e.g. Alok, Sarah).
    Relationship {
        /// Identifier for the contact or entity.
        contact_id: String,
        /// Role in relation to the user (e.g. "manager", "friend", "contractor").
        role: String,
    },
}

impl ContextScope {
    /// Checks if a fact in this scope is relevant to a query within `query_scope`.
    ///
    /// Strict Isolation Invariant:
    /// - Global facts are accessible from any query scope.
    /// - Domain facts are accessible ONLY when querying that exact domain.
    /// - Relationship facts are accessible ONLY when querying that exact contact ID.
    /// - All other combinations (generic -> domain, domainA -> domainB, relA -> relB) FAIL CLOSED.
    pub fn is_accessible_from(&self, query_scope: &ContextScope) -> bool {
        match (self, query_scope) {
            // Global facts are accessible everywhere
            (ContextScope::Global, _) => true,
            // Domain facts are accessible only when querying within that specific domain
            (ContextScope::Domain(d1), ContextScope::Domain(d2)) => d1 == d2,
            // Relationship facts are strictly isolated to that specific contact
            (
                ContextScope::Relationship { contact_id: c1, .. },
                ContextScope::Relationship { contact_id: c2, .. },
            ) => !c1.trim().is_empty() && c1 == c2,
            // No cross-domain, cross-relationship, or domain-into-generic leakage
            _ => false,
        }
    }

    /// Validates that this context scope is well-formed.
    pub fn validate(&self) -> Result<(), EpistemicError> {
        match self {
            ContextScope::Relationship { contact_id, .. } => {
                if contact_id.trim().is_empty() {
                    return Err(EpistemicError::InvalidScope(
                        "Relationship scope contact_id must not be empty or whitespace".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Returns true if this scope represents an un-scoped generic query.
    pub fn is_generic(&self) -> bool {
        matches!(self, ContextScope::Global)
    }
}

/// Specific origin of knowledge for auditability.
#[derive(Debug, Clone, PartialEq)]
pub enum FactSource {
    /// Directly articulated by the user in a message.
    UserStatement {
        /// Source message NodeId or UUID.
        message_id: String,
        /// Platform channel (AppleMessages, WhatsApp, Web).
        platform: String,
    },
    /// Derived from repeated system sensors or telemetry.
    DirectObservation {
        /// Telemetry or event ID.
        event_id: String,
        /// Sensor kind.
        sensor: String,
    },
    /// Model deduction or hypothesis.
    ModelInference {
        /// LLM model name (e.g. "claude-3-7-sonnet").
        model: String,
        /// BLAKE3 digest of reasoning prompt.
        prompt_digest: String,
        /// Model confidence score.
        confidence: f32,
    },
    /// Cryptographic confirmation ticket.
    ConfirmedToken {
        /// Token ID authorizing the grant.
        token_id: String,
    },
}

/// Epistemic Certainty levels governing belief precedence.
#[derive(Debug, Clone, PartialEq)]
pub enum EpistemicCertainty {
    /// Machine deduction or hypothesis (lowest precedence, rank 1).
    Inferred {
        /// Explanation for the deduction.
        reasoning: String,
    },
    /// Behavioral pattern observed multiple times across interactions (rank 2).
    ObservedPattern {
        /// Number of observations.
        observation_count: u32,
        /// Consistency score between 0.0 and 1.0.
        consistency: f32,
    },
    /// Ground truth explicitly stated by the user (highest precedence, rank 3).
    DirectlyKnown,
}

impl EpistemicCertainty {
    /// Integer rank for ordering and precedence checks:
    /// `DirectlyKnown` (3) > `ObservedPattern` (2) > `Inferred` (1).
    pub fn rank(&self) -> u8 {
        match self {
            EpistemicCertainty::Inferred { .. } => 1,
            EpistemicCertainty::ObservedPattern { .. } => 2,
            EpistemicCertainty::DirectlyKnown => 3,
        }
    }
}

/// Category of rule within the resolution hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleConstraintType {
    /// Foundational rule (e.g. "Vegetarian").
    BaselineRule,
    /// Specific permitted carve-out (e.g. "Eats eggs").
    PermittedException,
    /// Soft preference or guidance (e.g. "Prefers low dairy" - not an allergy).
    Preference,
    /// Strict, non-negotiable safety or financial ceiling (e.g. "Peanut allergy", "Max $50").
    SafetyCeiling,
}

/// Cost/risk of an action or false assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsequenceLevel {
    /// Harmless suggestion or formatting.
    Trivial = 1,
    /// Internal state or draft preparation.
    Operational = 2,
    /// External communication, financial spend, health/diet commitment.
    HighConsequence = 3,
}

/// Hybrid Logical Clock temporal validity window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HlcWindow {
    /// Monotonic timestamp when fact became valid (epoch ms or HLC physical ms).
    pub valid_from: u64,
    /// Monotonic timestamp when superseded or ended (None = active right now).
    pub valid_until: Option<u64>,
}

impl HlcWindow {
    /// Creates a new validity window.
    pub const fn new(valid_from: u64, valid_until: Option<u64>) -> Self {
        Self {
            valid_from,
            valid_until,
        }
    }

    /// Checks if this fact is active at a given timestamp.
    pub fn is_active(&self, at_timestamp: u64) -> bool {
        self.valid_from <= at_timestamp
            && match self.valid_until {
                Some(until) => at_timestamp < until,
                None => true,
            }
    }

    /// Creates a new validity window from Hybrid Logical Clock timestamps.
    pub fn from_hlc(
        valid_from: kineti_core::hlc::HlcTimestamp,
        valid_until: Option<kineti_core::hlc::HlcTimestamp>,
    ) -> Self {
        Self {
            valid_from: valid_from.physical_ms,
            valid_until: valid_until.map(|u| u.physical_ms),
        }
    }

    /// Checks if this fact is active at a given HlcTimestamp.
    pub fn is_active_at_hlc(&self, hlc: &kineti_core::hlc::HlcTimestamp) -> bool {
        self.is_active(hlc.physical_ms)
    }
}

/// Complete 7-dimensional provenance record of a persona belief.
#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceRecord {
    /// Specific factual claim.
    pub claim: String,
    /// Knowledge provenance source.
    pub source: FactSource,
    /// HLC temporal validity window.
    pub time_window: HlcWindow,
    /// Scoped boundary.
    pub scope: ContextScope,
    /// Epistemic certainty tier.
    pub certainty: EpistemicCertainty,
    /// Invalidation criteria.
    pub contradiction_criteria: Option<String>,
    /// Consequence cost if misapplied.
    pub consequence_cost: ConsequenceLevel,
}

/// A 7-dimensional context fact with epistemic provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemicFact {
    /// Unique fact identifier.
    pub id: String,
    /// User partition key.
    pub user_id: String,
    /// Scoped boundary.
    pub scope: ContextScope,
    /// Conceptual attribute (e.g. "diet", "meeting_window", "tone").
    pub attribute: String,
    /// Specific factual claim (e.g. "vegetarian", "afternoons", "eats eggs").
    pub claim: String,
    /// Behavioral hierarchy classification.
    pub constraint_type: RuleConstraintType,
    /// Epistemic certainty tier.
    pub certainty: EpistemicCertainty,
    /// Timestamp when fact became valid (HLC or epoch millis).
    pub valid_from: u64,
    /// Timestamp when superseded or ended (None = active right now).
    pub valid_until: Option<u64>,
    /// Explicit condition that would contradict this fact.
    pub contradiction_criteria: Option<String>,
    /// Consequence level if mishandled.
    pub consequence_level: ConsequenceLevel,
}

impl EpistemicFact {
    /// Checks if this fact is currently active (not superseded).
    pub fn is_active(&self, at_timestamp: u64) -> bool {
        self.valid_from <= at_timestamp
            && match self.valid_until {
                Some(until) => at_timestamp < until,
                None => true,
            }
    }

    /// Returns the HLC temporal validity window for this fact.
    pub fn time_window(&self) -> HlcWindow {
        HlcWindow::new(self.valid_from, self.valid_until)
    }

    /// Derives the knowledge source from the certainty tier and identifiers.
    pub fn source(&self) -> FactSource {
        match &self.certainty {
            EpistemicCertainty::DirectlyKnown => FactSource::UserStatement {
                message_id: self.id.clone(),
                platform: "chat".to_string(),
            },
            EpistemicCertainty::ObservedPattern {
                observation_count, ..
            } => FactSource::DirectObservation {
                event_id: self.id.clone(),
                sensor: format!("telemetry_obs_{}", observation_count),
            },
            EpistemicCertainty::Inferred { reasoning } => FactSource::ModelInference {
                model: "cortex-inference".to_string(),
                prompt_digest: reasoning.clone(),
                confidence: 0.75,
            },
        }
    }

    /// Derives the provenance record for this fact.
    pub fn provenance(&self) -> ProvenanceRecord {
        ProvenanceRecord {
            claim: self.claim.clone(),
            source: self.source(),
            time_window: self.time_window(),
            scope: self.scope.clone(),
            certainty: self.certainty.clone(),
            contradiction_criteria: self.contradiction_criteria.clone(),
            consequence_cost: self.consequence_level,
        }
    }

    /// Returns default durability based on certainty and constraint type.
    pub fn default_durability(&self) -> FactDurability {
        match self.certainty {
            EpistemicCertainty::DirectlyKnown => FactDurability::DurableFact,
            EpistemicCertainty::ObservedPattern {
                observation_count, ..
            } if observation_count >= 3 => FactDurability::DurableFact,
            _ => {
                if self.constraint_type == RuleConstraintType::SafetyCeiling
                    || self.constraint_type == RuleConstraintType::BaselineRule
                {
                    FactDurability::DurableFact
                } else {
                    FactDurability::WorkingContext
                }
            }
        }
    }
}

/// The 7 Epistemic State Transitions for non-destructive updates.
#[derive(Debug, Clone, PartialEq)]
pub enum EpistemicTransition {
    /// Existing belief reinforced with higher observation count.
    Confirmed(String),
    /// Fact scope narrowed to be more specific.
    Narrowed(String),
    /// An exception was added to a baseline rule.
    ExceptionAdded {
        /// The new exception fact ID.
        exception_id: String,
        /// The parent rule ID it qualifies.
        parent_rule_id: String,
    },
    /// Old fact superseded by a new fact from a given timestamp.
    Superseded {
        /// Previous fact ID now marked with valid_until.
        previous_id: String,
        /// Newly active fact ID.
        new_id: String,
    },
    /// Conflict detected with a high-certainty fact: requires asking the user.
    ConflictNeedsClarification {
        /// Clarification question to ask the user.
        question: String,
        /// Existing fact that conflicts.
        existing_fact_id: String,
        /// Candidate fact ID held in escrow.
        candidate_fact_id: String,
    },
    /// Weak clue recorded without altering active rules.
    WeakClueRecorded(String),
    /// Fact expired or invalidated because the situation ended.
    Invalidated(String),
}

/// Epistemic Engine error conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpistemicError {
    /// Inferred or lower-certainty fact contradicts a DirectlyKnown belief.
    ConflictNeedsClarification {
        /// Question to pose to the user.
        question: String,
        /// Existing ground truth fact ID.
        existing_fact_id: String,
        /// Proposed candidate fact ID held in escrow.
        candidate_fact_id: String,
    },
    /// Attempted HighConsequence action without explicit DirectlyKnown authorization.
    UnauthorizedHighConsequence {
        /// Explanation for the authorization failure.
        reason: String,
    },
    /// Invalid context scope specification.
    InvalidScope(String),
    /// Underlying storage or persistence failure.
    StorageError(String),
}

impl fmt::Display for EpistemicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictNeedsClarification {
                question,
                existing_fact_id,
                candidate_fact_id,
            } => write!(
                f,
                "Conflict needs clarification between existing '{}' and candidate '{}': {}",
                existing_fact_id, candidate_fact_id, question
            ),
            Self::UnauthorizedHighConsequence { reason } => {
                write!(f, "Unauthorized high consequence operation: {}", reason)
            }
            Self::InvalidScope(msg) => write!(f, "Invalid context scope: {}", msg),
            Self::StorageError(msg) => write!(f, "Epistemic storage error: {}", msg),
        }
    }
}

impl std::error::Error for EpistemicError {}

/// Helper: Checks if a word is common boilerplate/qualifier in allergy or safety claims.
fn is_allergy_boilerplate(word: &str) -> bool {
    matches!(
        word,
        "allergy"
            | "allergies"
            | "allergic"
            | "severe"
            | "mild"
            | "intolerance"
            | "intolerant"
            | "sensitive"
            | "sensitivity"
            | "restriction"
            | "avoid"
            | "free"
            | "no"
    )
}

/// Helper: Normalizes English singular and plural forms ("peanuts" -> "peanut", "tomatoes" -> "tomato").
fn stem_word(word: &str) -> &str {
    if let Some(s) = word.strip_suffix("ies") {
        if s.len() >= 2 {
            return s;
        }
    }
    if let Some(s) = word.strip_suffix("es") {
        if s.len() >= 3 {
            return s;
        }
    }
    if let Some(s) = word.strip_suffix('s') {
        if s.len() >= 3 {
            return s;
        }
    }
    word
}

/// Helper: Checks if two word tokens match (case-insensitive, singular/plural aware).
fn token_matches(t1: &str, t2: &str) -> bool {
    let t1 = t1.to_lowercase();
    let t2 = t2.to_lowercase();
    if t1 == t2 {
        return true;
    }
    if t1.strip_suffix('s') == Some(&t2) || t2.strip_suffix('s') == Some(&t1) {
        return true;
    }
    if t1.strip_suffix("es") == Some(&t2) || t2.strip_suffix("es") == Some(&t1) {
        return true;
    }
    stem_word(&t1) == stem_word(&t2)
}

/// Helper: Extracts significant non-boilerplate words from a string.
fn extract_significant_words(text: &str) -> Vec<String> {
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    let filtered: Vec<String> = words
        .iter()
        .filter(|w| !is_allergy_boilerplate(w))
        .cloned()
        .collect();

    if filtered.is_empty() {
        words
    } else {
        filtered
    }
}

/// Helper: Determines if a safety ceiling claim conflicts with an exception claim.
fn safety_ceiling_conflicts_with_exception(sc_claim: &str, exc_claim: &str) -> bool {
    let sc_lower = sc_claim.to_lowercase();
    let exc_lower = exc_claim.to_lowercase();
    if sc_lower == exc_lower || sc_lower.contains(&exc_lower) || exc_lower.contains(&sc_lower) {
        return true;
    }
    let sc_words = extract_significant_words(sc_claim);
    let exc_words = extract_significant_words(exc_claim);
    for sw in &sc_words {
        for ew in &exc_words {
            if token_matches(sw, ew) {
                return true;
            }
        }
    }
    false
}

/// Helper: Robust matching of a safety ceiling claim against candidate item name and tags.
fn matches_safety_ceiling(
    ceiling_claim: &str,
    candidate_item: &str,
    candidate_tags: &[&str],
) -> bool {
    let claim_lower = ceiling_claim.to_lowercase();
    let item_lower = candidate_item.to_lowercase();

    // 1. Direct substring match on candidate item name
    if item_lower.contains(&claim_lower) {
        return true;
    }

    // 2. Exact or substring match on tags (ignoring boilerplate)
    for &tag in candidate_tags {
        let tag_lower = tag.to_lowercase();
        if tag_lower == claim_lower
            || (!is_allergy_boilerplate(&tag_lower)
                && (tag_lower.contains(&claim_lower) || claim_lower.contains(&tag_lower)))
        {
            return true;
        }
    }

    // 3. Token- and stem-level matching across significant words
    let claim_words = extract_significant_words(ceiling_claim);
    let mut candidate_words = Vec::new();
    for &tag in candidate_tags {
        candidate_words.extend(extract_significant_words(tag));
    }
    candidate_words.extend(extract_significant_words(candidate_item));

    for cw in &claim_words {
        for cand_w in &candidate_words {
            if token_matches(cw, cand_w) {
                return true;
            }
        }
    }

    false
}

/// Resolved view of persona constraints for safe evaluation.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ResolvedPersonaView {
    /// Foundational baseline rules.
    pub baseline_rules: Vec<EpistemicFact>,
    /// Permitted exceptions to baseline rules.
    pub exceptions: Vec<EpistemicFact>,
    /// Soft preferences.
    pub preferences: Vec<EpistemicFact>,
    /// Hard non-negotiable safety ceilings (allergies, caps).
    pub safety_ceilings: Vec<EpistemicFact>,
}

impl ResolvedPersonaView {
    /// Compiles unambiguous, structured directives for LLM prompt hydration.
    pub fn compile_prompt_directives(&self) -> String {
        let mut out = String::new();

        if !self.safety_ceilings.is_empty() {
            out.push_str("STRICT SAFETY CEILINGS (ABSOLUTE PROHIBITION - NEVER VIOLATE):\n");
            for sc in &self.safety_ceilings {
                out.push_str(&format!(" - [ALLERGY/SAFETY] {}\n", sc.claim));
            }
        }

        if !self.baseline_rules.is_empty() {
            out.push_str("CORE BASELINE RULES:\n");
            for r in &self.baseline_rules {
                out.push_str(&format!(" - [RULE] {}\n", r.claim));
            }
        }

        // SafetyCeiling strictly suppresses any conflicting PermittedException
        let non_conflicting_exceptions: Vec<_> = self
            .exceptions
            .iter()
            .filter(|exc| {
                !self.safety_ceilings.iter().any(|sc| {
                    safety_ceiling_conflicts_with_exception(&sc.claim, &exc.claim)
                })
            })
            .collect();

        if !non_conflicting_exceptions.is_empty() {
            out.push_str("PERMITTED EXCEPTIONS TO BASELINE RULES:\n");
            for exc in non_conflicting_exceptions {
                out.push_str(&format!(" - [EXCEPTION] Permitted: {}\n", exc.claim));
            }
        }

        if !self.preferences.is_empty() {
            out.push_str("SOFT PREFERENCES (GUIDANCE - NOT AN ALLERGY OR HARD BLOCK):\n");
            for pref in &self.preferences {
                out.push_str(&format!(" - [PREFERENCE] {}\n", pref.claim));
            }
        }

        out
    }

    /// Returns true if no facts are present in any category.
    pub fn is_empty(&self) -> bool {
        self.baseline_rules.is_empty()
            && self.exceptions.is_empty()
            && self.preferences.is_empty()
            && self.safety_ceilings.is_empty()
    }

    /// Returns the total number of facts across all categories.
    pub fn total_facts(&self) -> usize {
        self.baseline_rules.len()
            + self.exceptions.len()
            + self.preferences.len()
            + self.safety_ceilings.len()
    }
}

/// Result of evaluating a candidate action against resolved persona constraints.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionEvaluation {
    /// Action is completely compatible with user persona.
    Permitted {
        /// Non-binding preference recommendations (e.g. "Low dairy preferred").
        recommendations: Vec<String>,
    },
    /// Blocked by a hard safety ceiling (e.g. allergy or financial cap).
    BlockedBySafetyCeiling {
        /// Reason for blocking.
        reason: String,
    },
    /// Blocked by baseline rule without an exception.
    BlockedByRule {
        /// Baseline rule that prohibits this.
        rule: String,
    },
    /// Ambiguity or potential conflict requires asking user first.
    NeedsClarification {
        /// Question to pose to the user.
        question: String,
    },
}

/// Structured advice resolved from persona hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedAdvice {
    /// Evaluation outcome against the 4-phase hierarchy.
    pub evaluation: ActionEvaluation,
    /// Compiled prompt directives for downstream cortex hydration.
    pub directives: String,
}

impl ResolvedAdvice {
    /// Returns true if the proposed action is permitted.
    pub fn is_permitted(&self) -> bool {
        matches!(self.evaluation, ActionEvaluation::Permitted { .. })
    }
}

/// Resolves behavioral advice for a candidate item against a set of persona facts.
///
/// Follows the 4-phase structured behavioral resolution:
/// 1. `SafetyCeiling` (Allergies/Medical/Hard caps) -> evaluated first as absolute veto.
/// 2. `PermittedException` (Explicit carve-outs) -> whitelists exceptions against general rules.
/// 3. `BaselineRule` (Identity & broad guidelines) -> enforced unless permitted by an exception.
/// 4. `Preference` (Soft inclinations) -> soft guidance without confusing preferences with allergies.
pub fn resolve_advice(
    facts: &[EpistemicFact],
    candidate_item: &str,
    candidate_tags: &[&str],
) -> ResolvedAdvice {
    let mut baseline_rules = Vec::new();
    let mut exceptions = Vec::new();
    let mut preferences = Vec::new();
    let mut safety_ceilings = Vec::new();

    for fact in facts {
        match fact.constraint_type {
            RuleConstraintType::BaselineRule => baseline_rules.push(fact.clone()),
            RuleConstraintType::PermittedException => {
                // Invariant: Only DirectlyKnown facts can carve out exceptions to baseline rules
                if fact.certainty == EpistemicCertainty::DirectlyKnown {
                    exceptions.push(fact.clone());
                }
            }
            RuleConstraintType::Preference => preferences.push(fact.clone()),
            RuleConstraintType::SafetyCeiling => safety_ceilings.push(fact.clone()),
        }
    }

    let view = ResolvedPersonaView {
        baseline_rules,
        exceptions,
        preferences,
        safety_ceilings,
    };

    let directives = view.compile_prompt_directives();

    // 1. Phase 1: Safety Ceilings (Absolute VETO - Allergies/Medical/Hard caps)
    for ceiling in &view.safety_ceilings {
        if matches_safety_ceiling(&ceiling.claim, candidate_item, candidate_tags) {
            return ResolvedAdvice {
                evaluation: ActionEvaluation::BlockedBySafetyCeiling {
                    reason: format!("Action violates strict safety ceiling: {}", ceiling.claim),
                },
                directives,
            };
        }
    }

    // Helper: checks if a violating ingredient or category is explicitly whitelisted by a permitted exception
    let has_exception_for = |ingredient: &str| -> bool {
        view.exceptions.iter().any(|exc| {
            // Exceptions in conflict with active safety ceilings are suppressed
            if view
                .safety_ceilings
                .iter()
                .any(|sc| safety_ceiling_conflicts_with_exception(&sc.claim, &exc.claim))
            {
                return false;
            }

            let exc_lower = exc.claim.to_lowercase();
            let ing_lower = ingredient.to_lowercase();
            if exc_lower == ing_lower
                || exc_lower.contains(&ing_lower)
                || ing_lower.contains(&exc_lower)
            {
                return true;
            }

            let exc_words = extract_significant_words(&exc.claim);
            let ing_words = extract_significant_words(ingredient);
            for ew in &exc_words {
                for iw in &ing_words {
                    if token_matches(ew, iw) {
                        return true;
                    }
                }
            }
            false
        })
    };

    // 2. Phase 2 & 3: Baseline Rules enforced UNLESS whitelisted by Permitted Exceptions
    let item_words = extract_significant_words(candidate_item);

    for rule in &view.baseline_rules {
        let rule_lower = rule.claim.to_lowercase();
        if rule_lower.contains("vegetarian") {
            let meat_indicators = [
                "meat", "chicken", "beef", "pork", "fish", "seafood", "lamb", "bacon",
                "mutton", "turkey", "duck", "veal", "steak", "poultry",
            ];

            // 1. Check all candidate tags: ANY unexcepted meat tag blocks
            for &tag in candidate_tags {
                let tag_is_meat = meat_indicators.iter().any(|&m| {
                    tag.eq_ignore_ascii_case(m)
                        || extract_significant_words(tag).iter().any(|tw| token_matches(tw, m))
                });
                if tag_is_meat && !has_exception_for(tag) {
                    return ResolvedAdvice {
                        evaluation: ActionEvaluation::BlockedByRule {
                            rule: format!("User is vegetarian and no exception permits {}", tag),
                        },
                        directives,
                    };
                }
            }

            // 2. Check candidate item name: ANY unexcepted meat indicator blocks
            for &m in &meat_indicators {
                let name_has_meat = item_words.iter().any(|iw| token_matches(iw, m));
                if name_has_meat && !has_exception_for(m) {
                    return ResolvedAdvice {
                        evaluation: ActionEvaluation::BlockedByRule {
                            rule: format!("User is vegetarian and no exception permits {}", m),
                        },
                        directives,
                    };
                }
            }
        } else if rule_lower.contains("vegan") {
            let animal_indicators = [
                "meat", "chicken", "beef", "pork", "fish", "seafood", "dairy", "cheese",
                "milk", "eggs", "egg", "honey", "gelatin",
            ];

            // 1. Check all candidate tags: ANY unexcepted animal tag blocks
            for &tag in candidate_tags {
                let tag_is_animal = animal_indicators.iter().any(|&a| {
                    tag.eq_ignore_ascii_case(a)
                        || extract_significant_words(tag).iter().any(|tw| token_matches(tw, a))
                });
                if tag_is_animal && !has_exception_for(tag) {
                    return ResolvedAdvice {
                        evaluation: ActionEvaluation::BlockedByRule {
                            rule: format!("User is vegan and no exception permits {}", tag),
                        },
                        directives,
                    };
                }
            }

            // 2. Check candidate item name: ANY unexcepted animal indicator blocks
            for &a in &animal_indicators {
                let name_has_animal = item_words.iter().any(|iw| token_matches(iw, a));
                if name_has_animal && !has_exception_for(a) {
                    return ResolvedAdvice {
                        evaluation: ActionEvaluation::BlockedByRule {
                            rule: format!("User is vegan and no exception permits {}", a),
                        },
                        directives,
                    };
                }
            }
        }
    }

    // 4. Soft Preferences (Guidance)
    let mut recommendations = Vec::new();
    for pref in &view.preferences {
        recommendations.push(format!("Preference noted: {}", pref.claim));
    }

    ResolvedAdvice {
        evaluation: ActionEvaluation::Permitted { recommendations },
        directives,
    }
}

/// High-performance Epistemic Persona Engine.
#[derive(Debug, Default)]
pub struct EpistemicEngine {
    facts: RwLock<HashMap<String, Vec<EpistemicFact>>>,
    provenance_sources: RwLock<HashMap<String, FactSource>>,
}

impl EpistemicEngine {
    /// Creates a new empty EpistemicEngine.
    pub fn new() -> Self {
        Self {
            facts: RwLock::new(HashMap::new()),
            provenance_sources: RwLock::new(HashMap::new()),
        }
    }

    /// Ingests a new fact, performing non-destructive epistemic delta resolution.
    ///
    /// If an incoming lower-certainty fact contradicts a DirectlyKnown belief,
    /// returns `Ok(EpistemicTransition::ConflictNeedsClarification)` without silently overwriting.
    pub fn ingest(
        &self,
        new_fact: EpistemicFact,
        current_time: u64,
    ) -> Result<EpistemicTransition, &'static str> {
        if let ContextScope::Relationship { contact_id, .. } = &new_fact.scope {
            if contact_id.trim().is_empty() {
                return Err("Invalid context scope: relationship contact_id cannot be empty");
            }
        }

        let mut store = self.facts.write().unwrap();
        let user_facts = store.entry(new_fact.user_id.clone()).or_default();

        // 1. Check for conflicting active facts under the same scope and attribute
        let existing_conflict = user_facts.iter_mut().find(|f| {
            f.is_active(current_time)
                && f.scope == new_fact.scope
                && f.attribute == new_fact.attribute
                && f.claim != new_fact.claim
        });

        if let Some(existing) = existing_conflict {
            // INVARIANT: An Inferred fact CANNOT override a DirectlyKnown fact
            if new_fact.certainty.rank() < existing.certainty.rank() {
                return Ok(EpistemicTransition::ConflictNeedsClarification {
                    question: format!(
                        "You previously confirmed '{}', but recent context suggests '{}'. Which should I follow?",
                        existing.claim, new_fact.claim
                    ),
                    existing_fact_id: existing.id.clone(),
                    candidate_fact_id: new_fact.id.clone(),
                });
            }

            // If the new fact is an explicit PermittedException to a BaselineRule
            if existing.constraint_type == RuleConstraintType::BaselineRule
                && new_fact.constraint_type == RuleConstraintType::PermittedException
            {
                let exc_id = new_fact.id.clone();
                let parent_id = existing.id.clone();
                user_facts.push(new_fact);
                return Ok(EpistemicTransition::ExceptionAdded {
                    exception_id: exc_id,
                    parent_rule_id: parent_id,
                });
            }

            // Higher or equal certainty explicit update -> Non-destructive supersession
            existing.valid_until = Some(current_time);
            let prev_id = existing.id.clone();
            let new_id = new_fact.id.clone();
            user_facts.push(new_fact);

            return Ok(EpistemicTransition::Superseded {
                previous_id: prev_id,
                new_id,
            });
        }

        // 2. Check for reinforcement of identical claim
        if let Some(existing) = user_facts.iter_mut().find(|f| {
            f.is_active(current_time)
                && f.scope == new_fact.scope
                && f.attribute == new_fact.attribute
                && f.claim == new_fact.claim
        }) {
            if new_fact.certainty.rank() > existing.certainty.rank() {
                existing.certainty = new_fact.certainty;
                existing.constraint_type = new_fact.constraint_type;
                existing.consequence_level = new_fact.consequence_level;
            } else if let EpistemicCertainty::ObservedPattern {
                observation_count, ..
            } = &mut existing.certainty
            {
                *observation_count += 1;
            }
            return Ok(EpistemicTransition::Confirmed(existing.id.clone()));
        }

        // 3. Normal fresh ingestion
        let fact_id = new_fact.id.clone();
        user_facts.push(new_fact);
        Ok(EpistemicTransition::Confirmed(fact_id))
    }

    /// Ingests a new fact, returning a typed `Err(EpistemicError::ConflictNeedsClarification)`
    /// when an inference or lower-certainty belief conflicts with an existing `DirectlyKnown` fact.
    pub fn try_ingest(
        &self,
        new_fact: EpistemicFact,
        current_time: u64,
    ) -> Result<EpistemicTransition, EpistemicError> {
        new_fact.scope.validate()?;

        let candidate_id = new_fact.id.clone();
        let candidate_claim = new_fact.claim.clone();

        let mut store = self.facts.write().unwrap();
        let user_facts = store.entry(new_fact.user_id.clone()).or_default();

        // 1. Check for conflicting active facts under the same scope and attribute
        let existing_conflict = user_facts.iter_mut().find(|f| {
            f.is_active(current_time)
                && f.scope == new_fact.scope
                && f.attribute == new_fact.attribute
                && f.claim != new_fact.claim
        });

        if let Some(existing) = existing_conflict {
            // INVARIANT: An Inferred fact CANNOT override a DirectlyKnown fact
            if new_fact.certainty.rank() < existing.certainty.rank() {
                return Err(EpistemicError::ConflictNeedsClarification {
                    question: format!(
                        "You previously confirmed '{}', but recent context suggests '{}'. Which should I follow?",
                        existing.claim, candidate_claim
                    ),
                    existing_fact_id: existing.id.clone(),
                    candidate_fact_id: candidate_id,
                });
            }

            // If the new fact is an explicit PermittedException to a BaselineRule
            if existing.constraint_type == RuleConstraintType::BaselineRule
                && new_fact.constraint_type == RuleConstraintType::PermittedException
            {
                let exc_id = new_fact.id.clone();
                let parent_id = existing.id.clone();
                user_facts.push(new_fact);
                return Ok(EpistemicTransition::ExceptionAdded {
                    exception_id: exc_id,
                    parent_rule_id: parent_id,
                });
            }

            // Higher or equal certainty explicit update -> Non-destructive supersession
            existing.valid_until = Some(current_time);
            let prev_id = existing.id.clone();
            let new_id = new_fact.id.clone();
            user_facts.push(new_fact);

            return Ok(EpistemicTransition::Superseded {
                previous_id: prev_id,
                new_id,
            });
        }

        // 2. Check for reinforcement of identical claim
        if let Some(existing) = user_facts.iter_mut().find(|f| {
            f.is_active(current_time)
                && f.scope == new_fact.scope
                && f.attribute == new_fact.attribute
                && f.claim == new_fact.claim
        }) {
            if new_fact.certainty.rank() > existing.certainty.rank() {
                existing.certainty = new_fact.certainty;
                existing.constraint_type = new_fact.constraint_type;
                existing.consequence_level = new_fact.consequence_level;
            } else if let EpistemicCertainty::ObservedPattern {
                observation_count, ..
            } = &mut existing.certainty
            {
                *observation_count += 1;
            }
            return Ok(EpistemicTransition::Confirmed(existing.id.clone()));
        }

        // 3. Normal fresh ingestion
        let fact_id = new_fact.id.clone();
        user_facts.push(new_fact);
        Ok(EpistemicTransition::Confirmed(fact_id))
    }

    /// Ingests a new fact with an explicit `FactSource` for provenance tracking.
    pub fn ingest_with_source(
        &self,
        new_fact: EpistemicFact,
        source: FactSource,
        current_time: u64,
    ) -> Result<EpistemicTransition, &'static str> {
        let fact_id = new_fact.id.clone();
        let transition = self.ingest(new_fact, current_time)?;
        self.provenance_sources.write().unwrap().insert(fact_id, source);
        Ok(transition)
    }

    /// Explicitly registers provenance source for an existing fact.
    pub fn set_provenance_source(&self, fact_id: &str, source: FactSource) {
        self.provenance_sources
            .write()
            .unwrap()
            .insert(fact_id.to_string(), source);
    }

    /// Retrieves the complete provenance record for a specific fact.
    pub fn get_provenance(&self, user_id: &str, fact_id: &str) -> Option<ProvenanceRecord> {
        let store = self.facts.read().unwrap();
        let user_facts = store.get(user_id)?;
        let fact = user_facts.iter().find(|f| f.id == fact_id)?;
        let mut prov = fact.provenance();

        // Check if an explicit source was registered
        if let Some(explicit_source) = self.provenance_sources.read().unwrap().get(fact_id) {
            prov.source = explicit_source.clone();
        }

        Some(prov)
    }

    /// Resolves the full persona rule-exception hierarchy for a given scope.
    pub fn resolve_scope(
        &self,
        user_id: &str,
        query_scope: &ContextScope,
        at_timestamp: u64,
    ) -> ResolvedPersonaView {
        let store = self.facts.read().unwrap();
        let mut view = ResolvedPersonaView::default();

        if let Some(user_facts) = store.get(user_id) {
            for fact in user_facts {
                if fact.is_active(at_timestamp) && fact.scope.is_accessible_from(query_scope) {
                    match fact.constraint_type {
                        RuleConstraintType::BaselineRule => view.baseline_rules.push(fact.clone()),
                        RuleConstraintType::PermittedException => view.exceptions.push(fact.clone()),
                        RuleConstraintType::Preference => view.preferences.push(fact.clone()),
                        RuleConstraintType::SafetyCeiling => view.safety_ceilings.push(fact.clone()),
                    }
                }
            }
        }

        view
    }

    /// Queries all active facts accessible within the specified scope.
    ///
    /// Enforces strict query isolation:
    /// - Queries in `Global` scope retrieve ONLY global facts (0 domain or relationship facts).
    /// - Queries in a specific domain retrieve only Global + that specific domain.
    /// - Queries in a relationship retrieve only Global + that specific relationship.
    pub fn query_facts_in_scope(
        &self,
        user_id: &str,
        query_scope: &ContextScope,
        at_timestamp: u64,
    ) -> Vec<EpistemicFact> {
        let store = self.facts.read().unwrap();
        let mut matching = Vec::new();

        if let Some(user_facts) = store.get(user_id) {
            for fact in user_facts {
                if fact.is_active(at_timestamp) && fact.scope.is_accessible_from(query_scope) {
                    matching.push(fact.clone());
                }
            }
        }

        matching
    }

    /// Evaluates a candidate action against the resolved persona hierarchy.
    pub fn evaluate_candidate(
        &self,
        user_id: &str,
        scope: &ContextScope,
        candidate_item: &str,
        candidate_tags: &[&str],
        current_time: u64,
    ) -> ActionEvaluation {
        self.resolve_advice_for_candidate(
            user_id,
            scope,
            candidate_item,
            candidate_tags,
            current_time,
        )
        .evaluation
    }

    /// Resolves behavioral advice and compiled prompt directives for candidate action.
    pub fn resolve_advice_for_candidate(
        &self,
        user_id: &str,
        scope: &ContextScope,
        candidate_item: &str,
        candidate_tags: &[&str],
        current_time: u64,
    ) -> ResolvedAdvice {
        let active_facts = self.query_facts_in_scope(user_id, scope, current_time);
        resolve_advice(&active_facts, candidate_item, candidate_tags)
    }

    /// Evaluates whether a proposed action can be authorized given persona facts and optional confirmed token.
    ///
    /// Non-negotiable invariant: HighConsequence actions MUST be authorized by DirectlyKnown facts or confirmed permissions.
    pub fn authorize_action_with_token(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        confirmed_token_id: Option<&str>,
        at_timestamp: u64,
    ) -> Result<bool, ActionEvaluation> {
        if consequence == ConsequenceLevel::HighConsequence {
            // Check if confirmed permission token is explicitly provided
            if let Some(token) = confirmed_token_id {
                if !token.trim().is_empty() {
                    return Ok(true);
                }
            }

            let view = self.resolve_scope(user_id, scope, at_timestamp);
            let prov_sources = self.provenance_sources.read().unwrap();

            let has_direct_authorization = view
                .baseline_rules
                .iter()
                .chain(view.exceptions.iter())
                .chain(view.preferences.iter())
                .chain(view.safety_ceilings.iter())
                .any(|f| {
                    let matches_claim = f.claim.eq_ignore_ascii_case(required_claim);
                    let is_directly_known = f.certainty == EpistemicCertainty::DirectlyKnown;
                    let is_confirmed_source = prov_sources
                        .get(&f.id)
                        .is_some_and(|s| matches!(s, FactSource::ConfirmedToken { .. }));

                    matches_claim && (is_directly_known || is_confirmed_source)
                });

            if !has_direct_authorization {
                return Err(ActionEvaluation::NeedsClarification {
                    question: format!(
                        "Executing '{}' has high real-world consequences. Please confirm explicitly before proceeding.",
                        required_claim
                    ),
                });
            }
        }

        Ok(true)
    }

    /// Evaluates whether a proposed action can be authorized given persona facts.
    ///
    /// Non-negotiable invariant: HighConsequence actions MUST be authorized by DirectlyKnown facts or confirmed permissions.
    pub fn authorize_action(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        at_timestamp: u64,
    ) -> Result<bool, ActionEvaluation> {
        self.authorize_action_with_token(
            user_id,
            scope,
            consequence,
            required_claim,
            None,
            at_timestamp,
        )
    }

    /// Evaluates whether a proposed action with optional confirmation token can be authorized,
    /// returning a typed `EpistemicError` on failure.
    pub fn authorize_action_with_token_strict(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        confirmed_token_id: Option<&str>,
        at_timestamp: u64,
    ) -> Result<bool, EpistemicError> {
        self.authorize_action_with_token(
            user_id,
            scope,
            consequence,
            required_claim,
            confirmed_token_id,
            at_timestamp,
        )
        .map_err(|eval| match eval {
            ActionEvaluation::NeedsClarification { question } => {
                EpistemicError::UnauthorizedHighConsequence { reason: question }
            }
            ActionEvaluation::BlockedBySafetyCeiling { reason } => {
                EpistemicError::UnauthorizedHighConsequence { reason }
            }
            ActionEvaluation::BlockedByRule { rule } => {
                EpistemicError::UnauthorizedHighConsequence { reason: rule }
            }
            ActionEvaluation::Permitted { .. } => {
                EpistemicError::UnauthorizedHighConsequence {
                    reason: "Action not authorized".to_string(),
                }
            }
        })
    }

    /// Evaluates whether a proposed action can be authorized, returning a typed `EpistemicError` on failure.
    pub fn authorize_action_strict(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        at_timestamp: u64,
    ) -> Result<bool, EpistemicError> {
        self.authorize_action_with_token_strict(
            user_id,
            scope,
            consequence,
            required_claim,
            None,
            at_timestamp,
        )
    }

    /// Evaluates incoming user input and correction count for memory promotion.
    pub fn evaluate_promotion(
        &self,
        text: &str,
        correction_count_on_attribute: usize,
    ) -> PromotionDecision {
        PromotionSignalDetector::evaluate(text, correction_count_on_attribute)
    }

    /// Verifies a fact at commitment time against perishability and live state.
    pub fn verify_commitment_fact(
        &self,
        acquired_at_ms: u64,
        perishability: Perishability,
        commit_timestamp_ms: u64,
        plan_value: &str,
        current_live_value: Option<&str>,
    ) -> CommitmentVerificationResult {
        CommitmentTimeVerifier::verify_fact(
            acquired_at_ms,
            perishability,
            commit_timestamp_ms,
            plan_value,
            current_live_value,
        )
    }

    /// Evaluates an execution gap using asymmetric gap filling.
    pub fn evaluate_gap(
        &self,
        parameter_name: &str,
        consequence: ConsequenceLevel,
        is_reversible: bool,
        cost_microcents: u64,
        suggested_default: Option<&str>,
    ) -> GapFillingDecision {
        GapFillingPolicy::evaluate_gap(
            parameter_name,
            consequence,
            is_reversible,
            cost_microcents,
            suggested_default,
        )
    }
}

// ---------------------------------------------------------------------------
// Memory Durability & Promotion Engine
// ---------------------------------------------------------------------------

/// Durability tier of a memory item, cleanly separating permanent persona ground-truth from transient scratchpad context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactDurability {
    /// Long-term durable fact that persists across sessions (e.g. food allergy, core preferences, hard rules).
    DurableFact,
    /// Ephemeral working context valid only within the current task or session (e.g. flight quote, current draft).
    WorkingContext,
}

/// Result of evaluating whether working context should be promoted to durable persona memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionDecision {
    /// Fact is promoted to durable persona memory.
    PromoteToDurable {
        /// Reason for promotion.
        reason: String,
    },
    /// Stays in working context without polluting permanent persona.
    KeepWorkingContext {
        /// Reason for keeping ephemeral.
        reason: String,
    },
}

/// Promotion Signal Detector identifying explicit cues or repeated corrections that promote context.
#[derive(Debug, Default)]
pub struct PromotionSignalDetector;

impl PromotionSignalDetector {
    /// Evaluates incoming user input and correction history to determine durability.
    pub fn evaluate(text: &str, correction_count_on_attribute: usize) -> PromotionDecision {
        let lower = text.to_lowercase();

        // 1. Strong explicit linguistic cues
        let strong_markers = [
            "always",
            "never",
            "i prefer",
            "i only",
            "from now on",
            "do not ever",
            "don't ever",
            "remember that",
            "my rule is",
            "permanently",
        ];

        for marker in &strong_markers {
            if lower.contains(marker) {
                return PromotionDecision::PromoteToDurable {
                    reason: format!("Explicit strong linguistic marker detected: '{}'", marker),
                };
            }
        }

        // 2. Repeated user corrections (>= 2 corrections on same attribute)
        if correction_count_on_attribute >= 2 {
            return PromotionDecision::PromoteToDurable {
                reason: format!(
                    "Repeated correction threshold met ({} corrections on this attribute)",
                    correction_count_on_attribute
                ),
            };
        }

        // 3. Otherwise: one-off action stays as working context
        PromotionDecision::KeepWorkingContext {
            reason: "One-off action or observation without explicit promotion signals; kept as working context"
                .to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Commitment-Time Verification Engine
// ---------------------------------------------------------------------------

/// Classification of fact perishability and freshness requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perishability {
    /// Stable fact that does not rapidly expire (e.g. food allergy, relationship identity).
    Stable,
    /// Perishable fact with a strict Time-To-Live in milliseconds (e.g. price quote, seat hold, inventory, auth token).
    Perishable {
        /// Maximum valid duration in milliseconds before re-verification is mandated.
        ttl_ms: u64,
    },
}

/// Result of evaluating a fact at the exact millisecond of external or financial commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitmentVerificationResult {
    /// Fact is fresh and verified; commitment can proceed safely.
    VerifiedFresh {
        /// Age of the fact in milliseconds.
        age_ms: u64,
    },
    /// Perishable fact has exceeded its TTL; commitment halted until re-fetched.
    StaleFactExpired {
        /// Elapsed time since acquisition.
        age_ms: u64,
        /// Configured TTL.
        ttl_ms: u64,
        /// Action required before commit.
        action_required: String,
    },
    /// Plan-time snapshot contradicts live state at commitment time.
    StateContradiction {
        /// Snapshot value assumed during planning.
        plan_value: String,
        /// Reality value discovered at commitment time.
        current_value: String,
    },
}

/// Verifier enforcing that facts are re-checked at commitment time rather than trusting cached plan snapshots.
#[derive(Debug)]
pub struct CommitmentTimeVerifier;

impl CommitmentTimeVerifier {
    /// Validates a fact at the moment of commitment.
    pub fn verify_fact(
        acquired_at_ms: u64,
        perishability: Perishability,
        commit_timestamp_ms: u64,
        plan_value: &str,
        current_live_value: Option<&str>,
    ) -> CommitmentVerificationResult {
        // 1. Check for live value contradiction
        if let Some(live) = current_live_value {
            if live != plan_value {
                return CommitmentVerificationResult::StateContradiction {
                    plan_value: plan_value.to_string(),
                    current_value: live.to_string(),
                };
            }
        }

        let age_ms = commit_timestamp_ms.saturating_sub(acquired_at_ms);

        // 2. Check perishability TTL
        match perishability {
            Perishability::Stable => CommitmentVerificationResult::VerifiedFresh { age_ms },
            Perishability::Perishable { ttl_ms } => {
                if age_ms > ttl_ms {
                    CommitmentVerificationResult::StaleFactExpired {
                        age_ms,
                        ttl_ms,
                        action_required: format!(
                            "Fact is {}ms old (exceeds TTL of {}ms). Re-verify with live source before committing.",
                            age_ms, ttl_ms
                        ),
                    }
                } else {
                    CommitmentVerificationResult::VerifiedFresh { age_ms }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Asymmetric Gap-Filling Engine
// ---------------------------------------------------------------------------

/// Decision rendered when an autonomous execution encounters missing information or parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapFillingDecision {
    /// Cheap & reversible gap: Fill with default, explicitly record and disclose the assumption.
    ProceedWithDisclosedAssumption {
        /// Selected default value.
        selected_default: String,
        /// Disclosed assumption to be recorded in audit trail / evidence.
        disclosed_assumption: String,
        /// Confirmation that the action is reversible.
        reversible: bool,
    },
    /// Expensive or irreversible gap: Escalate to user with clean question.
    EscalateToUser {
        /// Missing parameter name.
        missing_parameter: String,
        /// Clarification question for the user.
        question: String,
        /// Architectural rationale for holding execution.
        rationale: String,
    },
}

/// Policy governing asymmetric gap-filling: cheap/reversible fills and discloses; expensive/irreversible halts and asks.
#[derive(Debug)]
pub struct GapFillingPolicy;

impl GapFillingPolicy {
    /// Evaluates whether an execution gap can be filled with a disclosed assumption or must escalate.
    pub fn evaluate_gap(
        parameter_name: &str,
        consequence: ConsequenceLevel,
        is_reversible: bool,
        cost_microcents: u64,
        suggested_default: Option<&str>,
    ) -> GapFillingDecision {
        // High consequence, irreversible, or non-trivial spend (> $1.00 = 1_000_000 microcents) mandates escalation
        if consequence == ConsequenceLevel::HighConsequence || !is_reversible || cost_microcents > 1_000_000 {
            GapFillingDecision::EscalateToUser {
                missing_parameter: parameter_name.to_string(),
                question: format!(
                    "Please clarify '{}' before proceeding. This step is irreversible or high-consequence.",
                    parameter_name
                ),
                rationale: format!(
                    "Consequence is {:?}, reversible={}, cost=${:.2}",
                    consequence,
                    is_reversible,
                    cost_microcents as f64 / 1_000_000.0
                ),
            }
        } else {
            let default_val = suggested_default.unwrap_or("standard_default");
            GapFillingDecision::ProceedWithDisclosedAssumption {
                selected_default: default_val.to_string(),
                disclosed_assumption: format!(
                    "Assumed '{}' for parameter '{}' because the operation is cheap (${:.2}) and reversible.",
                    default_val,
                    parameter_name,
                    cost_microcents as f64 / 1_000_000.0
                ),
                reversible: true,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reputation & Outbound Gating Engine
// ---------------------------------------------------------------------------

/// Outbound communication gating rule: knowing an identity does not equal permission to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundGatingRule {
    /// Target contact or recipient address (e.g. email, phone number).
    pub recipient: String,
    /// Explicit permission granted to communicate externally.
    pub permission_granted: bool,
    /// Channel restricted to (e.g. "Email", "WhatsApp").
    pub channel: String,
}

impl OutboundGatingRule {
    /// Checks if an outbound transmission is authorized.
    pub fn can_send(&self) -> Result<(), String> {
        if self.permission_granted {
            Ok(())
        } else {
            Err(format!(
                "Outbound transmission to '{}' via {} blocked: knowing an identity does not grant permission to contact",
                self.recipient, self.channel
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Ingress Threat Defense Engine
// ---------------------------------------------------------------------------

/// Trust level for incoming payloads, ensuring external inputs arrive as untrusted data, never instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngressTrustLevel {
    /// External email, webhook, or peer agent message (untrusted data).
    UntrustedExternalData,
    /// Verified cryptographic user prompt from authenticated local harness.
    AuthenticatedUserInstruction,
}

impl IngressTrustLevel {
    /// Verifies that an incoming payload is permitted to issue direct agent instructions.
    pub fn allows_direct_instruction(&self) -> bool {
        matches!(self, IngressTrustLevel::AuthenticatedUserInstruction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_scope_isolation() {
        let engine = EpistemicEngine::new();

        let global_fact = EpistemicFact {
            id: "f_global".to_string(),
            user_id: "alex".to_string(),
            scope: ContextScope::Global,
            attribute: "language".to_string(),
            claim: "English".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::Trivial,
        };

        let health_fact = EpistemicFact {
            id: "f_health".to_string(),
            user_id: "alex".to_string(),
            scope: ContextScope::Domain(DomainKind::Health),
            attribute: "diet".to_string(),
            claim: "Vegetarian".to_string(),
            constraint_type: RuleConstraintType::BaselineRule,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::HighConsequence,
        };

        let work_fact = EpistemicFact {
            id: "f_work".to_string(),
            user_id: "alex".to_string(),
            scope: ContextScope::Domain(DomainKind::Work),
            attribute: "focus".to_string(),
            claim: "Systems Architecture".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::Operational,
        };

        let rel_fact = EpistemicFact {
            id: "f_rel_alok".to_string(),
            user_id: "alex".to_string(),
            scope: ContextScope::Relationship {
                contact_id: "alok".to_string(),
                role: "collaborator".to_string(),
            },
            attribute: "meeting_time".to_string(),
            claim: "Afternoons".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::Operational,
        };

        engine.ingest(global_fact, 1000).unwrap();
        engine.ingest(health_fact, 1000).unwrap();
        engine.ingest(work_fact, 1000).unwrap();
        engine.ingest(rel_fact, 1000).unwrap();

        // 1. Strict Query Isolation: Generic/Global scope returns ONLY global facts (0 domain, 0 relationship)
        let global_facts = engine.query_facts_in_scope("alex", &ContextScope::Global, 1050);
        assert_eq!(global_facts.len(), 1);
        assert_eq!(global_facts[0].claim, "English");

        let global_view = engine.resolve_scope("alex", &ContextScope::Global, 1050);
        assert_eq!(global_view.preferences.len(), 1);
        assert_eq!(global_view.preferences[0].claim, "English");
        assert_eq!(global_view.baseline_rules.len(), 0);
        assert_eq!(global_view.exceptions.len(), 0);
        assert_eq!(global_view.safety_ceilings.len(), 0);

        // 2. Querying Health scope should contain Global and Health, but NEVER Work or Alok relationship
        let health_view =
            engine.resolve_scope("alex", &ContextScope::Domain(DomainKind::Health), 1050);
        assert_eq!(health_view.baseline_rules.len(), 1);
        assert_eq!(health_view.baseline_rules[0].claim, "Vegetarian");
        assert_eq!(health_view.preferences.len(), 1); // Global language
        assert_eq!(health_view.preferences[0].claim, "English");

        // Verify zero work facts and zero Alok relationship facts in Health scope
        let health_facts = engine.query_facts_in_scope(
            "alex",
            &ContextScope::Domain(DomainKind::Health),
            1050,
        );
        assert_eq!(health_facts.len(), 2);
        assert!(!health_facts.iter().any(|f| f.claim == "Systems Architecture"));
        assert!(!health_facts.iter().any(|f| f.claim == "Afternoons"));

        // 3. Querying Sarah relationship scope should NEVER see Alok's meeting preference
        let sarah_scope = ContextScope::Relationship {
            contact_id: "sarah".to_string(),
            role: "collaborator".to_string(),
        };
        let sarah_view = engine.resolve_scope("alex", &sarah_scope, 1050);
        let has_alok_fact = sarah_view
            .preferences
            .iter()
            .any(|f| f.claim == "Afternoons");
        assert!(!has_alok_fact, "Alok's relationship facts leaked to Sarah");

        // Sarah scope should only see Global (English)
        let sarah_facts = engine.query_facts_in_scope("alex", &sarah_scope, 1050);
        assert_eq!(sarah_facts.len(), 1);
        assert_eq!(sarah_facts[0].claim, "English");

        // 4. Querying Alok relationship scope sees Global + Alok facts only
        let alok_scope = ContextScope::Relationship {
            contact_id: "alok".to_string(),
            role: "collaborator".to_string(),
        };
        let alok_facts = engine.query_facts_in_scope("alex", &alok_scope, 1050);
        assert_eq!(alok_facts.len(), 2);
        assert!(alok_facts.iter().any(|f| f.claim == "English"));
        assert!(alok_facts.iter().any(|f| f.claim == "Afternoons"));
    }

    #[test]
    fn test_dietary_rule_exception_and_safety_hierarchy() {
        let engine = EpistemicEngine::new();
        let health_scope = ContextScope::Domain(DomainKind::Health);

        // 1. Baseline Rule: Vegetarian
        engine
            .ingest(
                EpistemicFact {
                    id: "diet_base".to_string(),
                    user_id: "alex".to_string(),
                    scope: health_scope.clone(),
                    attribute: "diet".to_string(),
                    claim: "Vegetarian".to_string(),
                    constraint_type: RuleConstraintType::BaselineRule,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1000,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::HighConsequence,
                },
                1000,
            )
            .unwrap();

        // 2. Permitted Exception: Eats eggs
        engine
            .ingest(
                EpistemicFact {
                    id: "diet_exc_eggs".to_string(),
                    user_id: "alex".to_string(),
                    scope: health_scope.clone(),
                    attribute: "diet_exception".to_string(),
                    claim: "eggs".to_string(),
                    constraint_type: RuleConstraintType::PermittedException,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1005,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::HighConsequence,
                },
                1005,
            )
            .unwrap();

        // 3. Preference: Low dairy (NOT an allergy)
        engine
            .ingest(
                EpistemicFact {
                    id: "diet_pref_dairy".to_string(),
                    user_id: "alex".to_string(),
                    scope: health_scope.clone(),
                    attribute: "dairy_preference".to_string(),
                    claim: "low_dairy".to_string(),
                    constraint_type: RuleConstraintType::Preference,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1010,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::Operational,
                },
                1010,
            )
            .unwrap();

        // 4. Safety Ceiling: Peanut Allergy (strict non-negotiable)
        engine
            .ingest(
                EpistemicFact {
                    id: "diet_allergy_peanuts".to_string(),
                    user_id: "alex".to_string(),
                    scope: health_scope.clone(),
                    attribute: "allergy".to_string(),
                    claim: "peanut".to_string(),
                    constraint_type: RuleConstraintType::SafetyCeiling,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1015,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::HighConsequence,
                },
                1015,
            )
            .unwrap();

        // Test A: Chicken biryani -> MUST BE BLOCKED by Vegetarian rule
        let chicken_eval = engine.evaluate_candidate(
            "alex",
            &health_scope,
            "Chicken Biryani",
            &["meat", "chicken"],
            1020,
        );
        match chicken_eval {
            ActionEvaluation::BlockedByRule { rule } => {
                assert!(rule.contains("vegetarian"));
            }
            _ => panic!("Expected Chicken Biryani to be blocked by rule"),
        }

        // Test B: Egg omelette -> MUST BE PERMITTED due to explicit exception
        let egg_eval = engine.evaluate_candidate(
            "alex",
            &health_scope,
            "Vegetable Omelette",
            &["eggs", "breakfast"],
            1020,
        );
        match egg_eval {
            ActionEvaluation::Permitted { recommendations } => {
                assert_eq!(recommendations.len(), 1);
                assert!(recommendations[0].contains("low_dairy"));
            }
            _ => panic!("Expected Vegetable Omelette with eggs to be permitted"),
        }

        // Test C: Peanut pad thai -> MUST BE BLOCKED by Safety Ceiling (Allergy)
        let peanut_eval = engine.evaluate_candidate(
            "alex",
            &health_scope,
            "Tofu Pad Thai with Peanuts",
            &["vegetarian", "peanut"],
            1020,
        );
        match peanut_eval {
            ActionEvaluation::BlockedBySafetyCeiling { reason } => {
                assert!(reason.contains("peanut"));
            }
            _ => panic!("Expected Peanut Pad Thai to be blocked by safety ceiling"),
        }

        // Test D: Cheese Pasta -> MUST BE PERMITTED with preference note (NOT false allergy)
        let cheese_eval = engine.evaluate_candidate(
            "alex",
            &health_scope,
            "Cheese Tortellini",
            &["vegetarian", "dairy", "cheese"],
            1020,
        );
        match cheese_eval {
            ActionEvaluation::Permitted { recommendations } => {
                assert_eq!(recommendations.len(), 1);
                assert!(recommendations[0].contains("low_dairy"));
            }
            _ => panic!("Expected Cheese Tortellini to be permitted without false allergy"),
        }

        // Test E: Chicken & Egg Scramble -> MUST BE BLOCKED by rule (chicken is meat, even though eggs are permitted)
        let chicken_egg_eval = engine.evaluate_candidate(
            "alex",
            &health_scope,
            "Chicken & Egg Scramble",
            &["chicken", "eggs", "meat"],
            1020,
        );
        match chicken_egg_eval {
            ActionEvaluation::BlockedByRule { rule } => {
                assert!(rule.contains("vegetarian"));
            }
            _ => panic!("Expected Chicken & Egg Scramble to be blocked because chicken is meat"),
        }
    }

    #[test]
    fn test_epistemic_conflict_blocks_inference_from_overwriting_ground_truth() {
        let engine = EpistemicEngine::new();
        let work_scope = ContextScope::Domain(DomainKind::Work);

        // Ground truth: DirectlyKnown meeting preference (Rank 3)
        engine
            .ingest(
                EpistemicFact {
                    id: "mtg_morning".to_string(),
                    user_id: "alex".to_string(),
                    scope: work_scope.clone(),
                    attribute: "meeting_preference".to_string(),
                    claim: "Only mornings".to_string(),
                    constraint_type: RuleConstraintType::Preference,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1000,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::Operational,
                },
                1000,
            )
            .unwrap();

        // System deduction / inference claiming user likes late nights (Rank 1)
        let inference = EpistemicFact {
            id: "mtg_night_inference".to_string(),
            user_id: "alex".to_string(),
            scope: work_scope.clone(),
            attribute: "meeting_preference".to_string(),
            claim: "Late nights".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::Inferred {
                reasoning: "Observed one message sent at 11 PM".to_string(),
            },
            valid_from: 1050,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::Operational,
        };

        // 1. Ingest via `ingest`: returns ConflictNeedsClarification without silent overwrite
        let result = engine.ingest(inference.clone(), 1050).unwrap();
        match result {
            EpistemicTransition::ConflictNeedsClarification {
                question,
                existing_fact_id,
                candidate_fact_id,
            } => {
                assert_eq!(existing_fact_id, "mtg_morning");
                assert_eq!(candidate_fact_id, "mtg_night_inference");
                assert!(question.contains("Only mornings"));
                assert!(question.contains("Late nights"));
            }
            _ => panic!("Inference should have triggered ConflictNeedsClarification"),
        }

        // 2. Ingest via `try_ingest`: returns Err(EpistemicError::ConflictNeedsClarification)
        let strict_result = engine.try_ingest(inference, 1050);
        match strict_result {
            Err(EpistemicError::ConflictNeedsClarification {
                question,
                existing_fact_id,
                candidate_fact_id,
            }) => {
                assert_eq!(existing_fact_id, "mtg_morning");
                assert_eq!(candidate_fact_id, "mtg_night_inference");
                assert!(question.contains("Only mornings"));
                assert!(question.contains("Late nights"));
            }
            _ => panic!("try_ingest should return Err(EpistemicError::ConflictNeedsClarification)"),
        }

        // 3. Ground truth is NOT overwritten: active preference remains "Only mornings"
        let active_facts = engine.query_facts_in_scope("alex", &work_scope, 1050);
        assert_eq!(active_facts.len(), 1);
        assert_eq!(active_facts[0].claim, "Only mornings");
    }

    #[test]
    fn test_high_consequence_action_authorization_barrier() {
        let engine = EpistemicEngine::new();
        let finance_scope = ContextScope::Domain(DomainKind::Finance);

        // Fact 1: An Inferred fact indicating budget preference (Rank 1)
        engine
            .ingest(
                EpistemicFact {
                    id: "inf_budget".to_string(),
                    user_id: "alex".to_string(),
                    scope: finance_scope.clone(),
                    attribute: "auto_spend".to_string(),
                    claim: "Approve transactions up to $500".to_string(),
                    constraint_type: RuleConstraintType::Preference,
                    certainty: EpistemicCertainty::Inferred {
                        reasoning: "User browsed high-end electronics".to_string(),
                    },
                    valid_from: 1000,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::HighConsequence,
                },
                1000,
            )
            .unwrap();

        // An Inferred fact CANNOT authorize a HighConsequence action
        let auth_res = engine.authorize_action(
            "alex",
            &finance_scope,
            ConsequenceLevel::HighConsequence,
            "Approve transactions up to $500",
            1050,
        );
        match auth_res {
            Err(ActionEvaluation::NeedsClarification { question }) => {
                assert!(question.contains("high real-world consequences"));
            }
            _ => panic!("Expected NeedsClarification for unauthorized HighConsequence action"),
        }

        // Strict authorization also fails closed
        let strict_auth = engine.authorize_action_strict(
            "alex",
            &finance_scope,
            ConsequenceLevel::HighConsequence,
            "Approve transactions up to $500",
            1050,
        );
        assert!(matches!(
            strict_auth,
            Err(EpistemicError::UnauthorizedHighConsequence { .. })
        ));

        // Operational actions pass through without requiring DirectlyKnown authorization
        let op_auth = engine.authorize_action(
            "alex",
            &finance_scope,
            ConsequenceLevel::Operational,
            "Draft spend summary",
            1050,
        );
        assert!(op_auth.unwrap());

        // Now user directly confirms the rule (Rank 3)
        engine
            .ingest(
                EpistemicFact {
                    id: "direct_budget".to_string(),
                    user_id: "alex".to_string(),
                    scope: finance_scope.clone(),
                    attribute: "auto_spend".to_string(),
                    claim: "Approve transactions up to $500".to_string(),
                    constraint_type: RuleConstraintType::BaselineRule,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1100,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::HighConsequence,
                },
                1100,
            )
            .unwrap();

        // DirectlyKnown fact NOW successfully authorizes the HighConsequence action!
        let confirmed_auth = engine.authorize_action(
            "alex",
            &finance_scope,
            ConsequenceLevel::HighConsequence,
            "Approve transactions up to $500",
            1150,
        );
        assert!(confirmed_auth.unwrap());

        // Confirmed action authorization token also authorizes high-consequence actions
        let token_auth = engine.authorize_action_with_token(
            "alex",
            &finance_scope,
            ConsequenceLevel::HighConsequence,
            "Wire transfer $10000",
            Some("action_token_sec_999"),
            1150,
        );
        assert!(token_auth.unwrap());
    }

    #[test]
    fn test_explicit_supersession_preserves_temporal_history() {
        let engine = EpistemicEngine::new();
        let alok_scope = ContextScope::Relationship {
            contact_id: "alok".to_string(),
            role: "friend".to_string(),
        };

        // Fact 1: Alok prefers afternoons on Sept 1 (t=1000)
        engine
            .ingest(
                EpistemicFact {
                    id: "alok_pref_v1".to_string(),
                    user_id: "alex".to_string(),
                    scope: alok_scope.clone(),
                    attribute: "meeting_slot".to_string(),
                    claim: "Afternoons".to_string(),
                    constraint_type: RuleConstraintType::Preference,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 1000,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::Operational,
                },
                1000,
            )
            .unwrap();

        // Fact 2: Alok can only do mornings now on Sept 10 (t=2000)
        let result = engine
            .ingest(
                EpistemicFact {
                    id: "alok_pref_v2".to_string(),
                    user_id: "alex".to_string(),
                    scope: alok_scope.clone(),
                    attribute: "meeting_slot".to_string(),
                    claim: "Mornings".to_string(),
                    constraint_type: RuleConstraintType::Preference,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: 2000,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: ConsequenceLevel::Operational,
                },
                2000,
            )
            .unwrap();

        match result {
            EpistemicTransition::Superseded {
                previous_id,
                new_id,
            } => {
                assert_eq!(previous_id, "alok_pref_v1");
                assert_eq!(new_id, "alok_pref_v2");
            }
            _ => panic!("Expected Superseded transition"),
        }

        // Looking back at past timestamp (t=1500) -> Active preference was "Afternoons"
        let past_view = engine.resolve_scope("alex", &alok_scope, 1500);
        assert_eq!(past_view.preferences.len(), 1);
        assert_eq!(past_view.preferences[0].claim, "Afternoons");

        // Looking at current timestamp (t=2500) -> Active preference is "Mornings"
        let current_view = engine.resolve_scope("alex", &alok_scope, 2500);
        assert_eq!(current_view.preferences.len(), 1);
        assert_eq!(current_view.preferences[0].claim, "Mornings");
    }

    #[test]
    fn test_provenance_tracking_and_hlc_window() {
        let engine = EpistemicEngine::new();
        let fact = EpistemicFact {
            id: "fact_prov_1".to_string(),
            user_id: "alex".to_string(),
            scope: ContextScope::Domain(DomainKind::Taste),
            attribute: "coffee".to_string(),
            claim: "Cortado with oat milk".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: Some("Switches to black drip coffee".to_string()),
            consequence_level: ConsequenceLevel::Trivial,
        };

        let source = FactSource::UserStatement {
            message_id: "msg_99".to_string(),
            platform: "AppleMessages".to_string(),
        };

        engine.ingest_with_source(fact, source.clone(), 1000).unwrap();

        let prov = engine.get_provenance("alex", "fact_prov_1").unwrap();
        assert_eq!(prov.claim, "Cortado with oat milk");
        assert_eq!(prov.source, source);
        assert_eq!(prov.time_window.valid_from, 1000);
        assert_eq!(prov.time_window.valid_until, None);
        assert_eq!(prov.certainty, EpistemicCertainty::DirectlyKnown);
        assert_eq!(
            prov.contradiction_criteria,
            Some("Switches to black drip coffee".to_string())
        );
        assert_eq!(prov.consequence_cost, ConsequenceLevel::Trivial);
    }

    #[test]
    fn test_prompt_directives_compilation() {
        let view = ResolvedPersonaView {
            baseline_rules: vec![EpistemicFact {
                id: "r1".to_string(),
                user_id: "u1".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 100,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            }],
            exceptions: vec![EpistemicFact {
                id: "e1".to_string(),
                user_id: "u1".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "exc".to_string(),
                claim: "eggs".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 100,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            }],
            preferences: vec![EpistemicFact {
                id: "p1".to_string(),
                user_id: "u1".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "pref".to_string(),
                claim: "low dairy".to_string(),
                constraint_type: RuleConstraintType::Preference,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 100,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::Operational,
            }],
            safety_ceilings: vec![EpistemicFact {
                id: "s1".to_string(),
                user_id: "u1".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "allergy".to_string(),
                claim: "peanut".to_string(),
                constraint_type: RuleConstraintType::SafetyCeiling,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 100,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            }],
        };

        let directives = view.compile_prompt_directives();
        assert!(directives.contains("STRICT SAFETY CEILINGS"));
        assert!(directives.contains("[ALLERGY/SAFETY] peanut"));
        assert!(directives.contains("CORE BASELINE RULES"));
        assert!(directives.contains("[RULE] Vegetarian"));
        assert!(directives.contains("PERMITTED EXCEPTIONS"));
        assert!(directives.contains("[EXCEPTION] Permitted: eggs"));
        assert!(directives.contains("SOFT PREFERENCES"));
        assert!(directives.contains("[PREFERENCE] low dairy"));
    }

    #[test]
    fn test_resolve_advice_standalone() {
        let facts = vec![
            EpistemicFact {
                id: "1".to_string(),
                user_id: "u".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 10,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            EpistemicFact {
                id: "2".to_string(),
                user_id: "u".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet_exc".to_string(),
                claim: "eggs".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 10,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
        ];

        let adv1 = resolve_advice(&facts, "Chicken Sandwich", &["meat"]);
        assert!(!adv1.is_permitted());

        let adv2 = resolve_advice(&facts, "Egg Salad", &["eggs"]);
        assert!(adv2.is_permitted());
    }

    #[test]
    fn test_context_scope_validation_and_rejection() {
        let empty_rel = ContextScope::Relationship {
            contact_id: "".to_string(),
            role: "colleague".to_string(),
        };
        let whitespace_rel = ContextScope::Relationship {
            contact_id: "   \t".to_string(),
            role: "colleague".to_string(),
        };
        let valid_rel = ContextScope::Relationship {
            contact_id: "alice".to_string(),
            role: "colleague".to_string(),
        };

        assert!(matches!(
            empty_rel.validate(),
            Err(EpistemicError::InvalidScope(_))
        ));
        assert!(matches!(
            whitespace_rel.validate(),
            Err(EpistemicError::InvalidScope(_))
        ));
        assert!(valid_rel.validate().is_ok());
        assert!(ContextScope::Global.validate().is_ok());
        assert!(ContextScope::Domain(DomainKind::Health).validate().is_ok());

        let engine = EpistemicEngine::new();
        let invalid_fact = EpistemicFact {
            id: "bad_scope_fact".to_string(),
            user_id: "u_test".to_string(),
            scope: whitespace_rel,
            attribute: "secret".to_string(),
            claim: "hidden".to_string(),
            constraint_type: RuleConstraintType::Preference,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 100,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: ConsequenceLevel::Trivial,
        };

        // ingest should fail with string error
        assert!(engine.ingest(invalid_fact.clone(), 100).is_err());

        // try_ingest should return typed EpistemicError::InvalidScope
        assert!(matches!(
            engine.try_ingest(invalid_fact, 100),
            Err(EpistemicError::InvalidScope(_))
        ));
    }

    #[test]
    fn test_inferred_exception_does_not_whitelist_in_resolve_advice() {
        let facts = vec![
            EpistemicFact {
                id: "b1".to_string(),
                user_id: "u".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 10,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            EpistemicFact {
                id: "e1".to_string(),
                user_id: "u".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet_exc".to_string(),
                claim: "chicken".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::Inferred {
                    reasoning: "weak guess".to_string(),
                },
                valid_from: 10,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
        ];

        let advice = resolve_advice(&facts, "Chicken Sandwich", &["chicken"]);
        assert!(!advice.is_permitted());
        assert!(matches!(
            advice.evaluation,
            ActionEvaluation::BlockedByRule { .. }
        ));
    }

    #[test]
    fn test_fact_durability_and_promotion_signals() {
        // 1. Explicit linguistic marker -> promote
        let dec1 = PromotionSignalDetector::evaluate("I always prefer aisle seats on flights", 0);
        assert!(matches!(dec1, PromotionDecision::PromoteToDurable { .. }));

        let dec2 = PromotionSignalDetector::evaluate("Never schedule meetings before 10 AM", 0);
        assert!(matches!(dec2, PromotionDecision::PromoteToDurable { .. }));

        // 2. Repeated corrections -> promote
        let dec3 = PromotionSignalDetector::evaluate("No, please use Helvetica", 2);
        assert!(matches!(dec3, PromotionDecision::PromoteToDurable { .. }));

        // 3. One-off action -> keep working context
        let dec4 = PromotionSignalDetector::evaluate("Order an iced latte today", 0);
        assert!(matches!(dec4, PromotionDecision::KeepWorkingContext { .. }));
    }

    #[test]
    fn test_commitment_time_verifier_detects_stale_and_contradiction() {
        let acquired_at = 1_000;
        let ttl_ms = 5_000; // 5 seconds
        let perishability = Perishability::Perishable { ttl_ms };

        // Fresh within TTL -> VerifiedFresh
        let fresh = CommitmentTimeVerifier::verify_fact(
            acquired_at,
            perishability,
            3_000,
            "$199.00",
            Some("$199.00"),
        );
        assert!(matches!(fresh, CommitmentVerificationResult::VerifiedFresh { .. }));

        // Expired -> StaleFactExpired
        let expired = CommitmentTimeVerifier::verify_fact(
            acquired_at,
            perishability,
            7_000,
            "$199.00",
            Some("$199.00"),
        );
        assert!(matches!(expired, CommitmentVerificationResult::StaleFactExpired { .. }));

        // Contradiction -> StateContradiction
        let contradiction = CommitmentTimeVerifier::verify_fact(
            acquired_at,
            perishability,
            2_000,
            "$199.00",
            Some("$250.00"),
        );
        match contradiction {
            CommitmentVerificationResult::StateContradiction {
                plan_value,
                current_value,
            } => {
                assert_eq!(plan_value, "$199.00");
                assert_eq!(current_value, "$250.00");
            }
            _ => panic!("Expected StateContradiction"),
        }
    }

    #[test]
    fn test_gap_filling_policy_asymmetric_decisions() {
        // Cheap & reversible -> Proceed with disclosed assumption
        let cheap_dec = GapFillingPolicy::evaluate_gap(
            "table_sort_column",
            ConsequenceLevel::Trivial,
            true,
            0,
            Some("created_at_desc"),
        );
        match cheap_dec {
            GapFillingDecision::ProceedWithDisclosedAssumption {
                selected_default,
                reversible,
                ..
            } => {
                assert_eq!(selected_default, "created_at_desc");
                assert!(reversible);
            }
            _ => panic!("Expected ProceedWithDisclosedAssumption"),
        }

        // Expensive / irreversible -> Escalate to user
        let expensive_dec = GapFillingPolicy::evaluate_gap(
            "wire_transfer_amount",
            ConsequenceLevel::HighConsequence,
            false,
            50_000_000, // $50.00
            Some("$50.00"),
        );
        assert!(matches!(
            expensive_dec,
            GapFillingDecision::EscalateToUser { .. }
        ));
    }

    #[test]
    fn test_outbound_gating_and_ingress_trust() {
        // Outbound gating: knowing identity != permission to contact
        let unpermitted_outbound = OutboundGatingRule {
            recipient: "partner@example.com".to_string(),
            permission_granted: false,
            channel: "Email".to_string(),
        };
        assert!(unpermitted_outbound.can_send().is_err());

        let permitted_outbound = OutboundGatingRule {
            recipient: "partner@example.com".to_string(),
            permission_granted: true,
            channel: "Email".to_string(),
        };
        assert!(permitted_outbound.can_send().is_ok());

        // Ingress threat defense: untrusted external data cannot issue instructions
        let external_webhook = IngressTrustLevel::UntrustedExternalData;
        assert!(!external_webhook.allows_direct_instruction());

        let local_prompt = IngressTrustLevel::AuthenticatedUserInstruction;
        assert!(local_prompt.allows_direct_instruction());
    }
}
