//! # Kineti Memory (`kineti-memory`)
//!
//! Dual-substrate associative memory system for personalized consumer AI agents.
//!
//! ## Core Architecture
//! - **[`epistemic`]**: 360º Human Model Epistemic Persona Engine with multi-scope isolation, certainty tiers, and rule-exception hierarchies.
//! - **[`graph`]**: Causal User Property Graph tracking personal facts, habits, and preferences with `Replaces` edges.
//! - **[`vector`]**: High-speed cosine similarity vector index for past message and knowledge recall.
//! - **[`tombstone`]**: $O(1)$ Tombstone masking for instant invalidation of superseded or deleted facts.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod epistemic;
pub mod graph;
pub mod storage;
pub mod tombstone;
pub mod vector;

pub use epistemic::{
    resolve_advice, ActionEvaluation, CommitmentTimeVerifier, CommitmentVerificationResult,
    ConsequenceLevel, ContextScope, DomainKind, EpistemicCertainty, EpistemicEngine,
    EpistemicError, EpistemicFact, EpistemicTransition, FactDurability, FactSource,
    GapFillingDecision, GapFillingPolicy, HlcWindow, IngressTrustLevel, OutboundGatingRule,
    Perishability, PromotionDecision, PromotionSignalDetector, ProvenanceRecord,
    ResolvedAdvice, ResolvedPersonaView, RuleConstraintType,
};

use graph::UserPropertyGraph;
use kineti_core::conversation::{UserFact, UserStyleProfile};
use kineti_core::current_epoch_millis;
use kineti_core::kernel::NodeId;
use kineti_reflex::StyleAnalyzer;
use std::collections::HashMap;
use std::sync::RwLock;
use tombstone::TombstoneMask;
use vector::{SearchMatch, VectorIndex};

/// Unified Memory Engine coordinating vector search, causal graph traversal, and style profiles.
#[derive(Debug, Default)]
pub struct MemoryEngine {
    /// In-memory vector embedding index.
    pub vector_index: VectorIndex,
    /// Causal user property graph.
    pub property_graph: UserPropertyGraph,
    /// Fast tombstone mask for invalidated memories.
    pub tombstones: TombstoneMask,
    /// High-performance Epistemic Persona Engine.
    pub epistemic: EpistemicEngine,
    /// Dynamic user style profiles.
    pub style_profiles: RwLock<HashMap<String, UserStyleProfile>>,
    style_analyzer: StyleAnalyzer,
}

impl MemoryEngine {
    /// Creates a new MemoryEngine instance.
    pub fn new() -> Self {
        Self {
            vector_index: VectorIndex::new(),
            property_graph: UserPropertyGraph::new(),
            tombstones: TombstoneMask::new(),
            epistemic: EpistemicEngine::new(),
            style_profiles: RwLock::new(HashMap::new()),
            style_analyzer: StyleAnalyzer::new(),
        }
    }

    /// Stores a new personal memory fact for a user.
    pub fn remember_fact(
        &self,
        user_id: &str,
        category: &str,
        key: &str,
        value: &str,
        confidence: f32,
        source_msg_id: Option<String>,
    ) -> NodeId {
        let fact = UserFact::new(
            user_id,
            category,
            key,
            value,
            confidence,
            source_msg_id,
            current_epoch_millis(),
        );
        self.property_graph.insert_fact(fact)
    }

    /// Queries active facts for a user.
    pub fn query_facts(&self, user_id: &str, category: Option<&str>) -> Vec<UserFact> {
        self.property_graph.query_active_facts(user_id, category, &self.tombstones)
    }

    /// Deletes a fact (for privacy or 'forget me' settings).
    pub fn forget_fact(&self, fact_id: &str) -> bool {
        self.property_graph.delete_fact(fact_id, &self.tombstones)
    }

    /// Updates the dynamic style profile of a user based on an incoming message.
    pub fn observe_user_style(&self, user_id: &str, incoming_text: &str) -> UserStyleProfile {
        let mut profiles = self.style_profiles.write().unwrap();
        let current = profiles.entry(user_id.to_string()).or_default();
        let updated = self.style_analyzer.update_profile(current, incoming_text);
        *current = updated.clone();
        updated
    }

    /// Retrieves the current style profile for a user.
    pub fn get_user_style(&self, user_id: &str) -> UserStyleProfile {
        let profiles = self.style_profiles.read().unwrap();
        profiles.get(user_id).cloned().unwrap_or_default()
    }

    /// Searches vector index for top-K matching memories.
    pub fn search_vectors(&self, query_vec: &[f32], top_k: usize) -> Vec<SearchMatch> {
        self.vector_index.search(query_vec, top_k, &self.tombstones)
    }

    /// Ingests an epistemic fact with full provenance and non-destructive delta resolution.
    pub fn ingest_epistemic_fact(
        &self,
        fact: EpistemicFact,
        current_time: u64,
    ) -> Result<EpistemicTransition, &'static str> {
        self.epistemic.ingest(fact, current_time)
    }

    /// Ingests an epistemic fact, returning a typed `EpistemicError` on conflict.
    pub fn try_ingest_epistemic_fact(
        &self,
        fact: EpistemicFact,
        current_time: u64,
    ) -> Result<EpistemicTransition, EpistemicError> {
        self.epistemic.try_ingest(fact, current_time)
    }

    /// Resolves the scoped persona view for prompt hydration.
    pub fn resolve_scoped_persona(
        &self,
        user_id: &str,
        query_scope: &ContextScope,
        at_timestamp: u64,
    ) -> ResolvedPersonaView {
        self.epistemic.resolve_scope(user_id, query_scope, at_timestamp)
    }

    /// Queries all active facts for a user strictly matching the scope isolation boundary.
    pub fn query_scoped_facts(
        &self,
        user_id: &str,
        query_scope: &ContextScope,
        at_timestamp: u64,
    ) -> Vec<EpistemicFact> {
        self.epistemic.query_facts_in_scope(user_id, query_scope, at_timestamp)
    }

    /// Scoped hybrid retrieval combining vector similarity with epistemic scope masking.
    pub fn hybrid_scoped_retrieval(
        &self,
        user_id: &str,
        query_vec: &[f32],
        query_scope: &ContextScope,
        at_timestamp: u64,
        top_k: usize,
    ) -> (ResolvedPersonaView, Vec<SearchMatch>) {
        let persona_view = self.resolve_scoped_persona(user_id, query_scope, at_timestamp);
        let vector_matches =
            self.vector_index
                .search_for_user(user_id, query_vec, top_k, &self.tombstones);
        (persona_view, vector_matches)
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
        self.epistemic.evaluate_candidate(
            user_id,
            scope,
            candidate_item,
            candidate_tags,
            current_time,
        )
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
        self.epistemic.resolve_advice_for_candidate(
            user_id,
            scope,
            candidate_item,
            candidate_tags,
            current_time,
        )
    }

    /// Evaluates whether a proposed action can be authorized given persona facts.
    pub fn authorize_action(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        at_timestamp: u64,
    ) -> Result<bool, ActionEvaluation> {
        self.epistemic.authorize_action(
            user_id,
            scope,
            consequence,
            required_claim,
            at_timestamp,
        )
    }

    /// Evaluates whether a proposed action with optional confirmation token can be authorized.
    pub fn authorize_action_with_token(
        &self,
        user_id: &str,
        scope: &ContextScope,
        consequence: ConsequenceLevel,
        required_claim: &str,
        confirmed_token_id: Option<&str>,
        at_timestamp: u64,
    ) -> Result<bool, ActionEvaluation> {
        self.epistemic.authorize_action_with_token(
            user_id,
            scope,
            consequence,
            required_claim,
            confirmed_token_id,
            at_timestamp,
        )
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
        self.epistemic.authorize_action_strict(
            user_id,
            scope,
            consequence,
            required_claim,
            at_timestamp,
        )
    }

    /// Retrieves the complete provenance record for a specific fact.
    pub fn get_provenance(&self, user_id: &str, fact_id: &str) -> Option<ProvenanceRecord> {
        self.epistemic.get_provenance(user_id, fact_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_memory_engine_e2e() {
        let engine = MemoryEngine::new();

        // 1. Store user facts
        let fact_id = engine.remember_fact(
            "user_01",
            "relationship",
            "manager",
            "Sarah Chen",
            0.98,
            None,
        );
        let facts = engine.query_facts("user_01", Some("relationship"));
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].value, "Sarah Chen");

        // 2. Observe and calibrate user style
        let style = engine.observe_user_style("user_01", "yo can u check my inbox ngl rushing");
        assert!(style.lowercase_preference);
        assert!(style.slang_affinity > 0.1);

        // 3. Forget fact via settings privacy flow
        let deleted = engine.forget_fact(fact_id.as_str());
        assert!(deleted);
        let facts_after = engine.query_facts("user_01", Some("relationship"));
        assert_eq!(facts_after.len(), 0);

        // 4. Ingest and query epistemic facts via MemoryEngine facade
        let epistemic_fact = EpistemicFact {
            id: "ep_01".to_string(),
            user_id: "user_01".to_string(),
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
        let res = engine.ingest_epistemic_fact(epistemic_fact, 1000);
        assert!(res.is_ok());

        let scoped_facts = engine.query_scoped_facts(
            "user_01",
            &ContextScope::Domain(DomainKind::Health),
            1050,
        );
        assert_eq!(scoped_facts.len(), 1);
        assert_eq!(scoped_facts[0].claim, "Vegetarian");

        // 5. Scoped hybrid retrieval
        let query_v = vec![0.1; 128];
        let (persona, matches) = engine.hybrid_scoped_retrieval(
            "user_01",
            &query_v,
            &ContextScope::Domain(DomainKind::Health),
            1050,
            5,
        );
        assert_eq!(persona.baseline_rules.len(), 1);
        assert_eq!(persona.baseline_rules[0].claim, "Vegetarian");
        assert_eq!(matches.len(), 0);

        // 6. Test evaluate_candidate and resolve_advice_for_candidate facade
        let eval_blocked = engine.evaluate_candidate(
            "user_01",
            &ContextScope::Domain(DomainKind::Health),
            "Beef Burger",
            &["meat", "beef"],
            1050,
        );
        assert!(matches!(eval_blocked, ActionEvaluation::BlockedByRule { .. }));

        let advice = engine.resolve_advice_for_candidate(
            "user_01",
            &ContextScope::Domain(DomainKind::Health),
            "Green Salad",
            &["vegetable", "healthy"],
            1050,
        );
        assert!(advice.is_permitted());

        // 7. Test authorize_action facade
        let auth_ok = engine.authorize_action(
            "user_01",
            &ContextScope::Domain(DomainKind::Health),
            ConsequenceLevel::HighConsequence,
            "Vegetarian",
            1050,
        );
        assert!(auth_ok.unwrap());

        let auth_with_tok = engine.authorize_action_with_token(
            "user_01",
            &ContextScope::Domain(DomainKind::Health),
            ConsequenceLevel::HighConsequence,
            "Non-Vegetarian Dinner Party",
            Some("special_perm_token_888"),
            1050,
        );
        assert!(auth_with_tok.unwrap());

        // 8. Test get_provenance facade
        let prov = engine.get_provenance("user_01", "ep_01").unwrap();
        assert_eq!(prov.claim, "Vegetarian");
        assert_eq!(prov.certainty, EpistemicCertainty::DirectlyKnown);
    }
}
