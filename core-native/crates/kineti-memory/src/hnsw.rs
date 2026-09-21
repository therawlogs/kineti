//! # Layered Navigable Small World Index (`hnsw`)
//!
//! A std-only Hierarchical Navigable Small World graph for approximate
//! nearest-neighbor candidate generation over dense embedding vectors.
//!
//! Design notes:
//! - Deterministic layer assignment from a hash of the node id (no RNG,
//!   stable across runs for the same insert order).
//! - Cosine similarity is the distance metric, matching `vector.rs`.
//! - The graph proposes candidates; exact re-ranking plus tenant and
//!   tombstone filters in `vector.rs` stay authoritative for correctness.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Cosine similarity between two vectors over their shared prefix.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..len {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct NodeKey(u32);

#[derive(Clone, Debug)]
struct Node {
    layer: usize,
    vector: Vec<f32>,
    /// Neighbors per layer, index = layer.
    neighbors: Vec<Vec<u32>>,
}

#[derive(PartialEq)]
struct Scored {
    score: f32,
    id: u32,
}

impl Eq for Scored {}

impl PartialOrd for Scored {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for Scored {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Layered graph index. `max_layer` caps memory; `m` caps neighbors per layer.
#[derive(Debug, Default)]
pub struct HnswGraph {
    max_layer: usize,
    m: usize,
    entry: Option<u32>,
    nodes: HashMap<u32, Node>,
}

impl HnswGraph {
    /// Creates a graph with the given layer cap and neighbor budget.
    pub fn new(max_layer: usize, m: usize) -> Self {
        Self {
            max_layer: if max_layer == 0 { 3 } else { max_layer },
            m: if m == 0 { 8 } else { m },
            entry: None,
            nodes: HashMap::new(),
        }
    }

    /// Number of indexed nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True when no nodes are indexed.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Deterministic layer from node id hash (no RNG).
    fn layer_for(id: u32, max_layer: usize) -> usize {
        let mut h = id.wrapping_mul(0x9E37_79B9).wrapping_add(0x85EB_CA6B);
        h ^= h >> 13;
        h = h.wrapping_mul(0xC2B2_AE35);
        h ^= h >> 16;
        (h as usize) % (max_layer + 1)
    }

    /// Inserts a node and links it to its nearest neighbors per layer.
    pub fn insert(&mut self, id: u32, vector: Vec<f32>) {
        let layer = Self::layer_for(id, self.max_layer);
        let mut neighbors: Vec<Vec<u32>> = vec![Vec::new(); layer + 1];
        if let Some(entry_id) = self.entry {
            for l in (0..=layer).rev() {
                let cands = self.nearest_in_layer(&vector, entry_id, l, self.m * 2);
                neighbors[l] = cands.into_iter().take(self.m).collect();
            }
            // Back-link from existing neighbors.
            for l in 0..=layer {
                let targets = neighbors[l].clone();
                for t in targets {
                    let needs_prune = {
                        match self.nodes.get_mut(&t) {
                            Some(node) if l < node.neighbors.len() && !node.neighbors[l].contains(&id) => {
                                node.neighbors[l].push(id);
                                node.neighbors[l].len() > self.m * 2
                            }
                            _ => false,
                        }
                    };
                    if needs_prune {
                        let (nvec, nlist) = match self.nodes.get(&t) {
                            Some(n) => (n.vector.clone(), n.neighbors[l].clone()),
                            None => continue,
                        };
                        let keep = self.prune(&nvec, &nlist);
                        if let Some(n) = self.nodes.get_mut(&t) {
                            if l < n.neighbors.len() {
                                n.neighbors[l] = keep;
                            }
                        }
                    }
                }
            }
        }
        self.nodes.insert(id, Node { layer, vector, neighbors });
        let promote = match self.entry {
            None => true,
            Some(e) => self.nodes.get(&id).map(|n| n.layer).unwrap_or(0)
                > self.nodes.get(&e).map(|n| n.layer).unwrap_or(0),
        };
        if promote {
            self.entry = Some(id);
        }
    }

    fn prune(&self, base: &[f32], ids: &[u32]) -> Vec<u32> {
        let mut scored: Vec<(u32, f32)> = ids
            .iter()
            .filter_map(|id| self.nodes.get(id).map(|n| (*id, cosine_similarity(base, &n.vector))))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        scored.into_iter().take(self.m).map(|(id, _)| id).collect()
    }

    fn nearest_in_layer(&self, query: &[f32], start: u32, layer: usize, ef: usize) -> Vec<u32> {
        let mut visited: HashSet<u32> = HashSet::new();
        let mut heap: BinaryHeap<Scored> = BinaryHeap::new();
        visited.insert(start);
        if let Some(n) = self.nodes.get(&start) {
            heap.push(Scored { score: cosine_similarity(query, &n.vector), id: start });
        }
        let mut best: Vec<Scored> = Vec::new();
        while let Some(top) = heap.pop() {
            best.push(Scored { score: top.score, id: top.id });
            if let Some(node) = self.nodes.get(&top.id) {
                if layer < node.neighbors.len() {
                    for nb in &node.neighbors[layer] {
                        if visited.insert(*nb) {
                            if let Some(nn) = self.nodes.get(nb) {
                                heap.push(Scored {
                                    score: cosine_similarity(query, &nn.vector),
                                    id: *nb,
                                });
                            }
                        }
                    }
                }
            }
            if visited.len() > ef * 4 + 16 {
                break;
            }
        }
        best.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        best.into_iter().take(ef).map(|s| s.id).collect()
    }

    /// Proposes up to `ef` candidate ids via greedy descent from the entry point.
    pub fn search(&self, query: &[f32], ef: usize) -> Vec<u32> {
        let entry = match self.entry {
            Some(e) => e,
            None => return Vec::new(),
        };
        let top_layer = self.nodes.get(&entry).map(|n| n.layer).unwrap_or(0);
        let mut current = entry;
        // Greedy descent through upper layers.
        for l in (1..=top_layer).rev() {
            let cands = self.nearest_in_layer(query, current, l, 1);
            if let Some(&better) = cands.first() {
                current = better;
            }
        }
        self.nearest_in_layer(query, current, 0, ef.max(1))
    }

    /// Removes a node and all references to it (tombstone support).
    pub fn remove(&mut self, id: u32) {
        self.nodes.remove(&id);
        for node in self.nodes.values_mut() {
            for layer in node.neighbors.iter_mut() {
                layer.retain(|n| *n != id);
            }
        }
        if self.entry == Some(id) {
            self.entry = self.nodes.keys().next().copied();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster_vec(base: [f32; 3], jitter: f32, i: usize) -> Vec<f32> {
        vec![
            base[0] + jitter * ((i % 5) as f32 * 0.01),
            base[1] + jitter * ((i % 3) as f32 * 0.01),
            base[2],
        ]
    }

    #[test]
    fn test_empty_search_returns_nothing() {
        let g = HnswGraph::new(3, 8);
        assert!(g.is_empty());
        assert!(g.search(&[1.0, 0.0, 0.0], 5).is_empty());
    }

    #[test]
    fn test_insert_and_recall_nearest() {
        let mut g = HnswGraph::new(3, 8);
        for i in 0..20 {
            g.insert(i, cluster_vec([1.0, 0.0, 0.0], 1.0, i as usize));
        }
        for i in 20..40 {
            g.insert(i, cluster_vec([0.0, 1.0, 0.0], 1.0, i as usize));
        }
        let cands = g.search(&[1.0, 0.0, 0.0], 5);
        assert!(!cands.is_empty());
        assert!(cands.iter().all(|c| *c < 20));
    }

    #[test]
    fn test_layers_spread_deterministically() {
        let mut g = HnswGraph::new(4, 8);
        for i in 0..50 {
            g.insert(i, vec![i as f32, 0.0, 1.0]);
        }
        let layers: HashSet<usize> = g.nodes.values().map(|n| n.layer).collect();
        assert!(layers.len() > 1);
        // Same insert order rebuilds identical layers.
        let mut g2 = HnswGraph::new(4, 8);
        for i in 0..50 {
            g2.insert(i, vec![i as f32, 0.0, 1.0]);
        }
        for i in 0..50 {
            assert_eq!(g.nodes[&i].layer, g2.nodes[&i].layer);
        }
    }

    #[test]
    fn test_remove_cleans_neighbors() {
        let mut g = HnswGraph::new(2, 4);
        for i in 0..10 {
            g.insert(i, vec![1.0, i as f32 * 0.01, 0.0]);
        }
        g.remove(0);
        assert!(!g.nodes.contains_key(&0));
        for n in g.nodes.values() {
            for layer in &n.neighbors {
                assert!(!layer.contains(&0));
            }
        }
        assert!(!g.search(&[1.0, 0.0, 0.0], 5).contains(&0));
    }

    #[test]
    fn test_recall_stays_high_with_noise() {
        let mut g = HnswGraph::new(3, 8);
        for i in 0..100 {
            let v = if i < 50 {
                cluster_vec([1.0, 0.1, 0.0], 1.0, i)
            } else {
                cluster_vec([0.1, 1.0, 0.0], 1.0, i)
            };
            g.insert(i as u32, v);
        }
        let cands = g.search(&[1.0, 0.1, 0.0], 10);
        let hits = cands.iter().filter(|c| **c < 50).count();
        assert!(hits >= 8, "expected >=8/10 from correct cluster, got {hits}");
    }
}
