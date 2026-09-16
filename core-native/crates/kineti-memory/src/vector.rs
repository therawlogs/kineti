//! Fast vector index and similarity retrieval for user messages and facts.
//!
//! Implements normalized cosine similarity search with automatic tombstone masking.

use crate::tombstone::TombstoneMask;
use std::sync::RwLock;

/// An individual vector embedding record.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorRecord {
    /// Associated entity NodeId.
    pub id: String,
    /// High-dimensional embedding vector (e.g. 384, 768, or 1536 dimensions).
    pub vector: Vec<f32>,
    /// Associated text or snippet.
    pub snippet: String,
}

/// A search match returned by the vector index.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    /// NodeId of matching record.
    pub id: String,
    /// Cosine similarity score in range [-1.0, 1.0].
    pub score: f32,
    /// Snippet content.
    pub snippet: String,
}

/// In-memory vector similarity index.
#[derive(Debug, Default)]
pub struct VectorIndex {
    records: RwLock<Vec<VectorRecord>>,
}

impl VectorIndex {
    /// Creates a new vector index.
    pub fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
        }
    }

    /// Inserts a new vector embedding record into the index.
    pub fn insert(&self, id: impl Into<String>, vector: Vec<f32>, snippet: impl Into<String>) {
        let mut records = self.records.write().unwrap();
        records.push(VectorRecord {
            id: id.into(),
            vector,
            snippet: snippet.into(),
        });
    }

    /// Performs top-K cosine similarity search, excluding any tombstoned nodes.
    pub fn search(&self, query: &[f32], top_k: usize, tombstones: &TombstoneMask) -> Vec<SearchMatch> {
        let records = self.records.read().unwrap();
        let query_norm = norm(query);
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut matches: Vec<SearchMatch> = records
            .iter()
            .filter(|rec| !tombstones.is_masked(&rec.id))
            .filter_map(|rec| {
                let rec_norm = norm(&rec.vector);
                if rec_norm == 0.0 {
                    return None;
                }
                let dot = dot_product(query, &rec.vector);
                let score = dot / (query_norm * rec_norm);
                Some(SearchMatch {
                    id: rec.id.clone(),
                    score,
                    snippet: rec.snippet.clone(),
                })
            })
            .collect();

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(top_k);
        matches
    }

    /// Returns the number of vectors stored in the index.
    pub fn len(&self) -> usize {
        let records = self.records.read().unwrap();
        records.len()
    }

    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[inline(always)]
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let mut sum = 0.0;
    for i in 0..len {
        sum += a[i] * b[i];
    }
    sum
}

#[inline(always)]
fn norm(v: &[f32]) -> f32 {
    let mut sum = 0.0;
    for &x in v {
        sum += x * x;
    }
    sum.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_similarity_and_tombstone_filtering() {
        let index = VectorIndex::new();
        let tombstones = TombstoneMask::new();

        index.insert("node_01", vec![1.0, 0.0, 0.0], "Fact about dogs");
        index.insert("node_02", vec![0.9, 0.1, 0.0], "Fact about puppies");
        index.insert("node_03", vec![0.0, 1.0, 0.0], "Fact about cats");

        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 2, &tombstones);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "node_01");
        assert!((results[0].score - 1.0).abs() < 1e-5);
        assert_eq!(results[1].id, "node_02");

        // Now tombstone node_01 -> must not appear in top results
        tombstones.mask("node_01");
        let results2 = index.search(&query, 2, &tombstones);
        assert_eq!(results2.len(), 2);
        assert_eq!(results2[0].id, "node_02");
        assert_eq!(results2[1].id, "node_03");
    }
}
