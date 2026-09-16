//! # Kineti Memory (`kineti-memory`)
//!
//! Dual-substrate associative memory system for personalized consumer AI agents.
//!
//! ## Core Architecture
//! - **[`graph`]**: Causal User Property Graph tracking personal facts, habits, and preferences with `Replaces` edges.
//! - **[`vector`]**: High-speed cosine similarity vector index for past message and knowledge recall.
//! - **[`tombstone`]**: $O(1)$ Tombstone masking for instant invalidation of superseded or deleted facts.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod graph;
pub mod storage;
pub mod tombstone;
pub mod vector;

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
    }
}
