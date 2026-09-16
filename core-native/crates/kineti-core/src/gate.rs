//! Sub-50ms 3-Way Graph Commit Gate.
//!
//! Enforces:
//! 1. Topological Rank Acyclicity ($L(P) < L(C)$ and cycle detection).
//! 2. HLC Causal Monotonicity ($HLC(P) < HLC(C)$).
//! 3. Content-Addressed Merkle DAG Edge Lineage (cryptographic tamper detection).

use crate::hlc::{current_physical_ms, HlcTimestamp};
use crate::kernel::{KernelError, NodeId, ProvenanceNode};
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;

/// Errors returned by the 3-Way Graph Commit Gate.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CommitGateError {
    /// Parent node referenced by edge is not present in committed graph.
    MissingParent {
        /// Missing parent node ID.
        parent_id: String,
    },

    /// Cyclic causal dependency detected in graph.
    CyclicDependency {
        /// Candidate node ID.
        node_id: String,
        /// Parent node ID involved in cycle.
        parent_id: String,
    },

    /// Topological rank invariant violated (L(P) >= L(C)).
    TopologicalViolation {
        /// Offending parent ID.
        parent_id: String,
        /// Parent topological rank.
        parent_rank: u64,
        /// Candidate topological rank.
        candidate_rank: u64,
    },

    /// Causal monotonicity violated (HLC(P) >= HLC(C)).
    CausalInversion {
        /// Candidate HLC.
        candidate_hlc: HlcTimestamp,
        /// Parent HLC.
        parent_hlc: HlcTimestamp,
        /// Offending parent ID.
        parent_id: String,
    },

    /// Cryptographic tamper detected: single-byte corruption or hash mismatch.
    TamperDetected {
        /// Declared node ID.
        declared_id: String,
        /// Computed content hash.
        computed_id: String,
    },

    /// Underlying entity validation failed.
    ValidationFailure(KernelError),
}

impl fmt::Display for CommitGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParent { parent_id } => {
                write!(f, "Missing parent node in DAG: {}", parent_id)
            }
            Self::CyclicDependency { node_id, parent_id } => write!(
                f,
                "Cyclic dependency detected: node {} would form cycle with parent {}",
                node_id, parent_id
            ),
            Self::TopologicalViolation {
                parent_id,
                parent_rank,
                candidate_rank,
            } => write!(
                f,
                "Topological rank violation: parent {} rank {} >= candidate rank {}",
                parent_id, parent_rank, candidate_rank
            ),
            Self::CausalInversion {
                candidate_hlc,
                parent_hlc,
                parent_id,
            } => write!(
                f,
                "Causal inversion error: candidate HLC {} is not strictly greater than parent {} HLC {}",
                candidate_hlc, parent_id, parent_hlc
            ),
            Self::TamperDetected {
                declared_id,
                computed_id,
            } => write!(
                f,
                "Tamper detected: declared ID {} != computed content hash {}",
                declared_id, computed_id
            ),
            Self::ValidationFailure(err) => write!(f, "Kernel validation failure: {}", err),
        }
    }
}

impl Error for CommitGateError {}

impl From<KernelError> for CommitGateError {
    fn from(err: KernelError) -> Self {
        Self::ValidationFailure(err)
    }
}

/// A node that has successfully passed the 3-way commit gate and been stored.
#[derive(Clone, Debug)]
pub struct CommittedNode {
    /// Provenance node data.
    pub node: ProvenanceNode,
    /// Calculated topological layer/rank in the DAG.
    pub topological_rank: u64,
    /// System physical timestamp when committed.
    pub committed_at_ms: u64,
}

/// Cryptographic receipt returned upon successful graph commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitReceipt {
    /// Content-addressed ID of committed node.
    pub node_id: NodeId,
    /// Topological rank assigned to node.
    pub topological_rank: u64,
    /// Verified HLC timestamp of node.
    pub hlc: HlcTimestamp,
    /// Number of verified parent edges.
    pub parent_count: usize,
    /// Microseconds taken to verify all 3 commit invariants.
    pub latency_micros: u64,
}

/// Sub-50ms 3-Way Graph Commit Gate.
pub struct CommitGate {
    nodes: RwLock<HashMap<NodeId, CommittedNode>>,
    reverse_edges: RwLock<HashMap<NodeId, Vec<NodeId>>>,
    commit_counter: AtomicU64,
}

impl Default for CommitGate {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitGate {
    /// Creates a new empty commit gate.
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            reverse_edges: RwLock::new(HashMap::new()),
            commit_counter: AtomicU64::new(0),
        }
    }

    /// Helper evaluating all 3 commit gate invariants against existing nodes map.
    fn verify_invariants(
        nodes: &HashMap<NodeId, CommittedNode>,
        candidate: &ProvenanceNode,
    ) -> Result<u64, CommitGateError> {
        // Gate 3: Content-addressing integrity & tamper detection
        candidate.verify_content_address().map_err(|e| match e {
            KernelError::ContentAddressMismatch { declared, computed } => {
                CommitGateError::TamperDetected {
                    declared_id: declared,
                    computed_id: computed,
                }
            }
            other => CommitGateError::ValidationFailure(other),
        })?;

        // Duplicate node check: duplicate node creates cycle with itself
        if nodes.contains_key(&candidate.id) {
            return Err(CommitGateError::CyclicDependency {
                node_id: candidate.id.0.clone(),
                parent_id: candidate.id.0.clone(),
            });
        }

        // Roots: no parents, rank 0
        if candidate.parents.is_empty() {
            return Ok(0);
        }

        let mut max_parent_rank: u64 = 0;

        for parent_id in &candidate.parents {
            let parent_node = nodes.get(parent_id).ok_or_else(|| {
                CommitGateError::MissingParent {
                    parent_id: parent_id.0.clone(),
                }
            })?;

            // Gate 2: Causal monotonicity HLC(P) < HLC(C)
            if parent_node.node.hlc >= candidate.hlc {
                return Err(CommitGateError::CausalInversion {
                    candidate_hlc: candidate.hlc,
                    parent_hlc: parent_node.node.hlc,
                    parent_id: parent_id.0.clone(),
                });
            }

            // Gate 3 (Parents): Re-verify parent content-address in memory
            parent_node
                .node
                .verify_content_address()
                .map_err(|e| match e {
                    KernelError::ContentAddressMismatch { declared, computed } => {
                        CommitGateError::TamperDetected {
                            declared_id: declared,
                            computed_id: computed,
                        }
                    }
                    other => CommitGateError::ValidationFailure(other),
                })?;

            if parent_node.topological_rank > max_parent_rank {
                max_parent_rank = parent_node.topological_rank;
            }

            // Gate 1: Cycle detection via ancestor BFS
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(parent_id.clone());

            while let Some(current_id) = queue.pop_front() {
                if current_id == candidate.id {
                    return Err(CommitGateError::CyclicDependency {
                        node_id: candidate.id.0.clone(),
                        parent_id: parent_id.0.clone(),
                    });
                }

                if visited.insert(current_id.clone()) {
                    if let Some(current_node) = nodes.get(&current_id) {
                        for grand_parent in &current_node.node.parents {
                            queue.push_back(grand_parent.clone());
                        }
                    }
                }
            }
        }

        let candidate_rank = max_parent_rank + 1;
        Ok(candidate_rank)
    }

    /// Verifies candidate node against all 3 commit gate invariants without committing.
    pub fn verify(&self, candidate: &ProvenanceNode) -> Result<u64, CommitGateError> {
        let nodes = self.nodes.read().expect("CommitGate nodes read lock poisoned");
        Self::verify_invariants(&*nodes, candidate)
    }

    /// Validates all 3 invariants and atomically commits node to Merkle DAG.
    pub fn commit(&self, candidate: ProvenanceNode) -> Result<CommitReceipt, CommitGateError> {
        let start = Instant::now();

        let mut nodes = self.nodes.write().expect("CommitGate nodes write lock poisoned");
        let rank = Self::verify_invariants(&*nodes, &candidate)?;

        let node_id = candidate.id.clone();
        let hlc = candidate.hlc;
        let parent_count = candidate.parents.len();
        let parents = candidate.parents.clone();

        let committed = CommittedNode {
            node: candidate,
            topological_rank: rank,
            committed_at_ms: current_physical_ms(),
        };

        nodes.insert(node_id.clone(), committed);

        // Update reverse edges index
        {
            let mut reverse = self
                .reverse_edges
                .write()
                .expect("CommitGate reverse_edges write lock poisoned");
            for p in &parents {
                reverse.entry(p.clone()).or_default().push(node_id.clone());
            }
        }

        self.commit_counter.fetch_add(1, Ordering::Relaxed);
        let latency_micros = start.elapsed().as_micros() as u64;

        Ok(CommitReceipt {
            node_id,
            topological_rank: rank,
            hlc,
            parent_count,
            latency_micros,
        })
    }

    /// Retrieves a committed node by its NodeId.
    pub fn get(&self, id: &NodeId) -> Option<CommittedNode> {
        let nodes = self.nodes.read().expect("CommitGate nodes read lock poisoned");
        nodes.get(id).cloned()
    }

    /// Returns the total count of committed nodes in the DAG.
    pub fn count(&self) -> u64 {
        self.commit_counter.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{Actor, ActorType, HashAlgorithm, KernelEntity};
    use std::collections::BTreeMap;

    fn make_test_node(name: &str, parents: Vec<NodeId>, hlc: HlcTimestamp) -> ProvenanceNode {
        let entity = KernelEntity::Actor(Actor {
            id: name.into(),
            name: format!("Node {}", name),
            actor_type: ActorType::Agent,
            public_key: None,
            role_id: None,
        });

        ProvenanceNode::new(
            entity,
            parents,
            hlc,
            BTreeMap::new(),
            HashAlgorithm::Blake3,
        )
        .expect("Valid node creation")
    }

    #[test]
    fn test_commit_gate_acyclicity_and_rank() {
        let gate = CommitGate::new();

        // 1. Commit root A
        let node_a = make_test_node("A", vec![], HlcTimestamp::new(1000, 0, 1));
        let receipt_a = gate.commit(node_a.clone()).expect("Root A commits");
        assert_eq!(receipt_a.topological_rank, 0);

        // 2. Commit child B (parent A)
        let node_b = make_test_node("B", vec![node_a.id.clone()], HlcTimestamp::new(1000, 1, 1));
        let receipt_b = gate.commit(node_b.clone()).expect("Child B commits");
        assert_eq!(receipt_b.topological_rank, 1);

        // 3. Commit child C (parent B)
        let node_c = make_test_node("C", vec![node_b.id.clone()], HlcTimestamp::new(1000, 2, 1));
        let receipt_c = gate.commit(node_c.clone()).expect("Child C commits");
        assert_eq!(receipt_c.topological_rank, 2);

        // 4. Duplicate node ID rejected as cycle/duplicate
        let duplicate = make_test_node("A", vec![], HlcTimestamp::new(1000, 0, 1));
        let err = gate.commit(duplicate).unwrap_err();
        assert!(matches!(err, CommitGateError::CyclicDependency { .. }));
    }

    #[test]
    fn test_commit_gate_diamond_dag() {
        let gate = CommitGate::new();

        let a = make_test_node("Root", vec![], HlcTimestamp::new(1000, 0, 1));
        let rec_a = gate.commit(a.clone()).unwrap();
        assert_eq!(rec_a.topological_rank, 0);

        let b = make_test_node("Left", vec![a.id.clone()], HlcTimestamp::new(1000, 1, 1));
        let rec_b = gate.commit(b.clone()).unwrap();
        assert_eq!(rec_b.topological_rank, 1);

        let c = make_test_node("Right", vec![a.id.clone()], HlcTimestamp::new(1000, 2, 1));
        let rec_c = gate.commit(c.clone()).unwrap();
        assert_eq!(rec_c.topological_rank, 1);

        let d = make_test_node("Join", vec![b.id.clone(), c.id.clone()], HlcTimestamp::new(1000, 3, 1));
        let rec_d = gate.commit(d.clone()).unwrap();
        assert_eq!(rec_d.topological_rank, 2);
        assert_eq!(rec_d.parent_count, 2);
    }

    #[test]
    fn test_commit_gate_causal_inversion_rejection() {
        let gate = CommitGate::new();

        let parent = make_test_node("Parent", vec![], HlcTimestamp::new(2000, 5, 1));
        gate.commit(parent.clone()).expect("Parent commits");

        // Candidate with lower HLC
        let child_lower = make_test_node("ChildLower", vec![parent.id.clone()], HlcTimestamp::new(2000, 4, 1));
        let err_lower = gate.commit(child_lower).unwrap_err();
        assert!(matches!(err_lower, CommitGateError::CausalInversion { .. }));

        // Candidate with equal HLC
        let child_equal = make_test_node("ChildEqual", vec![parent.id.clone()], HlcTimestamp::new(2000, 5, 1));
        let err_equal = gate.commit(child_equal).unwrap_err();
        assert!(matches!(err_equal, CommitGateError::CausalInversion { .. }));
    }

    #[test]
    fn test_commit_gate_single_byte_tamper_detection() {
        let gate = CommitGate::new();

        let node = make_test_node("TamperTarget", vec![], HlcTimestamp::new(3000, 0, 1));
        let mut tampered_node = node.clone();

        // Mutate single character in declared NodeId
        let mut id_bytes = tampered_node.id.0.into_bytes();
        let last_idx = id_bytes.len() - 1;
        id_bytes[last_idx] = if id_bytes[last_idx] == b'a' { b'b' } else { b'a' };
        tampered_node.id = NodeId::new(String::from_utf8(id_bytes).unwrap());

        let err = gate.commit(tampered_node).unwrap_err();
        assert!(matches!(err, CommitGateError::TamperDetected { .. }));
    }

    #[test]
    fn test_commit_gate_missing_parent() {
        let gate = CommitGate::new();
        let orphan = make_test_node("Orphan", vec![NodeId::new("non_existent_parent")], HlcTimestamp::new(1000, 1, 1));
        let err = gate.commit(orphan).unwrap_err();
        assert!(matches!(err, CommitGateError::MissingParent { .. }));
    }
}
