//! Fast vector index and similarity retrieval for user messages and facts.
//!
//! Implements normalized cosine similarity search with automatic tombstone masking,
//! multi-tenant user isolation, and calibrated hybrid fusion scoring.

use crate::tombstone::TombstoneMask;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;

/// An individual vector embedding record with tenant isolation and sequence index.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorRecord {
    /// Associated entity NodeId.
    pub id: String,
    /// Integer index ID for compressed bitset masking.
    pub index_id: u32,
    /// Tenant isolation partition key.
    pub user_id: String,
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

/// In-memory vector similarity index with tenant partitioning.
#[derive(Debug, Default)]
pub struct VectorIndex {
    records: RwLock<Vec<VectorRecord>>,
    next_index: AtomicU32,
}

impl VectorIndex {
    /// Creates a new vector index.
    pub fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            next_index: AtomicU32::new(0),
        }
    }

    /// Inserts a new vector record for a specific user tenant. Returns the assigned index_id.
    pub fn insert_for_user(
        &self,
        user_id: impl Into<String>,
        id: impl Into<String>,
        vector: Vec<f32>,
        snippet: impl Into<String>,
    ) -> u32 {
        let index_id = self.next_index.fetch_add(1, Ordering::Relaxed);
        let mut records = self.records.write().unwrap();
        records.push(VectorRecord {
            id: id.into(),
            index_id,
            user_id: user_id.into(),
            vector,
            snippet: snippet.into(),
        });
        index_id
    }

    /// Inserts a new vector embedding record into the index with global user scope.
    pub fn insert(&self, id: impl Into<String>, vector: Vec<f32>, snippet: impl Into<String>) -> u32 {
        self.insert_for_user("", id, vector, snippet)
    }

    /// Performs top-K cosine similarity search strictly partitioned by user_id, excluding tombstoned nodes.
    pub fn search_for_user(
        &self,
        user_id: &str,
        query: &[f32],
        top_k: usize,
        tombstones: &TombstoneMask,
    ) -> Vec<SearchMatch> {
        let records = self.records.read().unwrap();
        let query_norm = norm(query);
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut matches: Vec<SearchMatch> = records
            .iter()
            .filter(|rec| rec.user_id == user_id)
            .filter(|rec| !tombstones.is_masked_index(rec.index_id) && !tombstones.is_masked(&rec.id))
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

    /// Performs top-K cosine similarity search across all records, excluding tombstoned nodes.
    pub fn search(&self, query: &[f32], top_k: usize, tombstones: &TombstoneMask) -> Vec<SearchMatch> {
        let records = self.records.read().unwrap();
        let query_norm = norm(query);
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut matches: Vec<SearchMatch> = records
            .iter()
            .filter(|rec| !tombstones.is_masked_index(rec.index_id) && !tombstones.is_masked(&rec.id))
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

/// Computes calibrated temperature-scaled hybrid fusion score combining vector cosine
/// similarity and graph geodesic hop distance (Equation 14, Paper 3).
///
/// Constants: alpha = 0.55, gamma = 0.85, tau_cos = 0.70, temperature = 0.15.
pub fn compute_hybrid_fusion_score(
    cosine_sim: f32,
    graph_hop_distance: usize,
    alpha: f32,
    gamma: f32,
    tau_cos: f32,
    temperature: f32,
) -> f32 {
    let tau = if tau_cos <= 0.0 { 0.70 } else { tau_cos };
    let temp = if temperature <= 0.0 { 0.15 } else { temperature };
    let normalized_cos = cosine_sim / tau;
    let vector_score = 1.0 / (1.0 + (-normalized_cos / temp).exp());
    let graph_score = gamma.powi(graph_hop_distance as i32);
    alpha * vector_score + (1.0 - alpha) * graph_score
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

    #[test]
    fn test_multi_tenant_vector_isolation() {
        let index = VectorIndex::new();
        let tombstones = TombstoneMask::new();

        index.insert_for_user("tenant_alice", "alice_doc_1", vec![1.0, 0.0, 0.0], "Alice confidential");
        index.insert_for_user("tenant_bob", "bob_doc_1", vec![1.0, 0.0, 0.0], "Bob confidential");

        let query = vec![1.0, 0.0, 0.0];

        // Searching as Alice must NEVER return Bob's document
        let alice_results = index.search_for_user("tenant_alice", &query, 10, &tombstones);
        assert_eq!(alice_results.len(), 1);
        assert_eq!(alice_results[0].id, "alice_doc_1");

        // Searching as Bob must NEVER return Alice's document
        let bob_results = index.search_for_user("tenant_bob", &query, 10, &tombstones);
        assert_eq!(bob_results.len(), 1);
        assert_eq!(bob_results[0].id, "bob_doc_1");
    }

    #[test]
    fn test_hybrid_fusion_score() {
        let score_1hop = compute_hybrid_fusion_score(0.95, 1, 0.55, 0.85, 0.70, 0.15);
        let score_3hop = compute_hybrid_fusion_score(0.95, 3, 0.55, 0.85, 0.70, 0.15);
        assert!(score_1hop > score_3hop);
        assert!(score_1hop > 0.0 && score_1hop <= 1.0);
    }
}
