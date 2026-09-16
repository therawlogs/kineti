//! High-performance tombstone mask for invalidated memories and facts.
//!
//! Provides $O(1)$ lookup and setting of tombstone bits to ensure
//! updated or deleted facts are instantly suppressed from retrieval.

use std::collections::HashSet;
use std::sync::RwLock;

/// Fast thread-safe tombstone mask tracking invalidated memory node IDs.
#[derive(Debug, Default)]
pub struct TombstoneMask {
    masked_nodes: RwLock<HashSet<String>>,
}

impl TombstoneMask {
    /// Creates a new empty tombstone mask.
    pub fn new() -> Self {
        Self {
            masked_nodes: RwLock::new(HashSet::new()),
        }
    }

    /// Masks (invalidates) a node ID.
    pub fn mask(&self, node_id: impl Into<String>) {
        let mut set = self.masked_nodes.write().unwrap();
        set.insert(node_id.into());
    }

    /// Unmasks a node ID.
    pub fn unmask(&self, node_id: &str) {
        let mut set = self.masked_nodes.write().unwrap();
        set.remove(node_id);
    }

    /// Returns true if the node ID is masked (tombstoned).
    pub fn is_masked(&self, node_id: &str) -> bool {
        let set = self.masked_nodes.read().unwrap();
        set.contains(node_id)
    }

    /// Returns the count of currently tombstoned nodes.
    pub fn count(&self) -> usize {
        let set = self.masked_nodes.read().unwrap();
        set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tombstone_lifecycle() {
        let mask = TombstoneMask::new();
        assert!(!mask.is_masked("fact_01"));
        assert_eq!(mask.count(), 0);

        mask.mask("fact_01");
        assert!(mask.is_masked("fact_01"));
        assert_eq!(mask.count(), 1);

        mask.unmask("fact_01");
        assert!(!mask.is_masked("fact_01"));
        assert_eq!(mask.count(), 0);
    }
}
