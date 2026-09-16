//! Causal User Property Graph.
//!
//! Stores user facts, preferences, relationships, and habits, maintaining
//! causal provenance links and temporal validity.

use crate::tombstone::TombstoneMask;
use kineti_core::conversation::UserFact;
use kineti_core::kernel::NodeId;
use std::collections::HashMap;
use std::sync::RwLock;

/// Causal edge relationship between two facts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactRelation {
    /// New fact supersedes / invalidates the old fact (e.g. moved to a new city).
    Replaces,
    /// Fact is causally related to another fact (e.g. manager -> company).
    RelatesTo,
    /// Fact was derived from a specific message interaction.
    DerivedFrom,
}

/// A causal link between two memory facts.
#[derive(Debug, Clone, PartialEq)]
pub struct FactEdge {
    /// Source fact NodeId.
    pub from_id: String,
    /// Target fact NodeId.
    pub to_id: String,
    /// Type of causal relation.
    pub relation: FactRelation,
}

/// In-memory causal property graph for user memories.
#[derive(Debug, Default)]
pub struct UserPropertyGraph {
    facts: RwLock<HashMap<String, UserFact>>,
    user_index: RwLock<HashMap<String, Vec<String>>>,
    edges: RwLock<Vec<FactEdge>>,
}

impl UserPropertyGraph {
    /// Creates a new property graph.
    pub fn new() -> Self {
        Self {
            facts: RwLock::new(HashMap::new()),
            user_index: RwLock::new(HashMap::new()),
            edges: RwLock::new(Vec::new()),
        }
    }

    /// Inserts a fact for a user.
    pub fn insert_fact(&self, fact: UserFact) -> NodeId {
        let id = fact.id.clone();
        let id_str = id.as_str().to_string();
        let user_id = fact.user_id.clone();

        {
            let mut facts = self.facts.write().unwrap();
            facts.insert(id_str.clone(), fact);
        }

        {
            let mut user_idx = self.user_index.write().unwrap();
            user_idx.entry(user_id).or_default().push(id_str);
        }

        id
    }

    /// Replaces an existing fact with a new fact, creating a `Replaces` edge and masking the old fact.
    pub fn replace_fact(&self, old_fact_id: &NodeId, new_fact: UserFact, tombstones: &TombstoneMask) -> NodeId {
        let new_id = self.insert_fact(new_fact);

        // Tombstone the old fact
        tombstones.mask(old_fact_id.as_str());

        // Mark old fact as tombstoned in graph
        {
            let mut facts = self.facts.write().unwrap();
            if let Some(old) = facts.get_mut(old_fact_id.as_str()) {
                old.tombstoned = true;
            }
        }

        // Add causal Replaces edge
        {
            let mut edges = self.edges.write().unwrap();
            edges.push(FactEdge {
                from_id: new_id.as_str().to_string(),
                to_id: old_fact_id.as_str().to_string(),
                relation: FactRelation::Replaces,
            });
        }

        new_id
    }

    /// Queries all active (non-tombstoned) facts for a user, optionally filtered by category.
    pub fn query_active_facts(&self, user_id: &str, category: Option<&str>, tombstones: &TombstoneMask) -> Vec<UserFact> {
        let user_idx = self.user_index.read().unwrap();
        let fact_ids = match user_idx.get(user_id) {
            Some(ids) => ids,
            None => return Vec::new(),
        };

        let facts = self.facts.read().unwrap();
        fact_ids
            .iter()
            .filter(|id| !tombstones.is_masked(id))
            .filter_map(|id| facts.get(id).cloned())
            .filter(|f| !f.tombstoned)
            .filter(|f| {
                if let Some(cat) = category {
                    f.category == cat
                } else {
                    true
                }
            })
            .collect()
    }

    /// Deletes a fact permanently and masks it in the tombstone set.
    pub fn delete_fact(&self, fact_id: &str, tombstones: &TombstoneMask) -> bool {
        tombstones.mask(fact_id);
        let mut facts = self.facts.write().unwrap();
        if let Some(fact) = facts.get_mut(fact_id) {
            fact.tombstoned = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_graph_insert_and_replace() {
        let graph = UserPropertyGraph::new();
        let tombstones = TombstoneMask::new();

        let fact1 = UserFact::new(
            "user_01",
            "location",
            "home_city",
            "New York",
            0.9,
            None,
            1710000000,
        );
        let id1 = graph.insert_fact(fact1);

        let active = graph.query_active_facts("user_01", Some("location"), &tombstones);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].value, "New York");

        // User moves to London -> replaces old fact
        let fact2 = UserFact::new(
            "user_01",
            "location",
            "home_city",
            "London",
            0.95,
            None,
            1710500000,
        );
        let id2 = graph.replace_fact(&id1, fact2, &tombstones);
        assert_ne!(id1, id2);

        let active2 = graph.query_active_facts("user_01", Some("location"), &tombstones);
        assert_eq!(active2.len(), 1);
        assert_eq!(active2[0].value, "London");
        assert!(tombstones.is_masked(id1.as_str()));
    }
}
