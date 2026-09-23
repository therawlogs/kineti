//! Causal Value Graph (CVG) and Transitive Blast Radius Engine.
//!
//! Implements Directional Normalized Trust-Weighted Impact (DNTI)
//! with Kahneman-Tversky loss aversion ($\kappa \ge 2.5$).

use std::collections::HashMap;

/// Kahneman-Tversky loss aversion coefficient ($kappa \ge 2.5$).
pub const DEFAULT_LOSS_AVERSION_KAPPA: f32 = 2.5;

/// Node in the Causal Value Graph representing an outcome, component, or state.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueNode {
    /// Node identifier.
    pub id: String,
    /// Trust weight or importance weight w(v) in range [0.0, 1.0].
    pub weight: f32,
    /// Expected upside or gain in range [0.0, 1.0].
    pub expected_gain: f32,
    /// Downside risk or potential loss in range [0.0, 1.0].
    pub expected_loss: f32,
    /// Outgoing dependency IDs (transitive blast radius edges).
    pub dependencies: Vec<String>,
}

/// Blast radius assessment result.
#[derive(Debug, Clone, PartialEq)]
pub struct BlastRadiusReport {
    /// Transitive affected node count.
    pub affected_node_count: usize,
    /// Total cumulative blast radius weight.
    pub total_weight: f32,
    /// Directional Normalized Trust-Weighted Impact (DNTI).
    pub dnti: f32,
    /// Whether the action is approved or requires explicit human confirmation.
    pub requires_human_approval: bool,
}

/// Causal Value Graph engine tracking system-wide impact and blast radius.
#[derive(Debug, Default)]
pub struct CausalValueGraph {
    nodes: HashMap<String, ValueNode>,
}

impl CausalValueGraph {
    /// Creates a new empty Causal Value Graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Inserts or updates a value node in the graph.
    pub fn insert_node(&mut self, node: ValueNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Computes the transitive blast radius set starting from a modified node.
    pub fn compute_blast_radius(&self, root_node_id: &str) -> Vec<String> {
        let mut visited = Vec::new();
        let mut queue = vec![root_node_id.to_string()];

        while let Some(current) = queue.pop() {
            if !visited.contains(&current) {
                visited.push(current.clone());
                if let Some(node) = self.nodes.get(&current) {
                    for dep in &node.dependencies {
                        if !visited.contains(dep) {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }
        visited
    }

    /// Evaluates the Directional Normalized Trust-Weighted Impact (DNTI) across the blast radius.
    pub fn evaluate_impact(&self, root_node_id: &str, kappa: f32) -> BlastRadiusReport {
        let radius = self.compute_blast_radius(root_node_id);
        let mut sum_weight = 0.0f32;
        let mut weighted_impact = 0.0f32;

        for node_id in &radius {
            if let Some(node) = self.nodes.get(node_id) {
                let w = node.weight.max(0.01);
                sum_weight += w;
                let delta = node.expected_gain - (kappa * node.expected_loss);
                weighted_impact += w * delta;
            }
        }

        let dnti = if sum_weight > 0.0 {
            weighted_impact / sum_weight
        } else {
            0.0
        };

        // If DNTI is negative or blast radius affects > 5 nodes with significant loss, require human review
        let requires_human = dnti < 0.0 || (radius.len() >= 3 && dnti < 0.2);

        BlastRadiusReport {
            affected_node_count: radius.len(),
            total_weight: sum_weight,
            dnti,
            requires_human_approval: requires_human,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnti_calculation_with_loss_aversion() {
        let mut graph = CausalValueGraph::new();

        graph.insert_node(ValueNode {
            id: "auth_service".to_string(),
            weight: 0.9,
            expected_gain: 0.8,
            expected_loss: 0.1,
            dependencies: vec!["db_users".to_string(), "api_gateway".to_string()],
        });

        graph.insert_node(ValueNode {
            id: "db_users".to_string(),
            weight: 1.0,
            expected_gain: 0.5,
            expected_loss: 0.4, // Risky!
            dependencies: vec![],
        });

        graph.insert_node(ValueNode {
            id: "api_gateway".to_string(),
            weight: 0.7,
            expected_gain: 0.7,
            expected_loss: 0.05,
            dependencies: vec![],
        });

        let report = graph.evaluate_impact("auth_service", DEFAULT_LOSS_AVERSION_KAPPA);
        assert_eq!(report.affected_node_count, 3);
        assert!(report.total_weight > 2.0);

        // Due to db_users having 0.4 loss * 2.5 = 1.0 downside, DNTI should reflect Kahneman-Tversky aversion
        assert!(report.dnti < 0.3);
    }

    #[test]
    fn test_high_loss_action_requires_human_approval() {
        let mut graph = CausalValueGraph::new();
        graph.insert_node(ValueNode {
            id: "payments".to_string(),
            weight: 1.0,
            expected_gain: 0.1,
            expected_loss: 0.9,
            dependencies: vec!["ledger".to_string()],
        });
        graph.insert_node(ValueNode {
            id: "ledger".to_string(),
            weight: 1.0,
            expected_gain: 0.0,
            expected_loss: 0.8,
            dependencies: vec![],
        });
        let report = graph.evaluate_impact("payments", DEFAULT_LOSS_AVERSION_KAPPA);
        assert!(report.dnti < 0.0);
        assert!(report.requires_human_approval);
    }

    #[test]
    fn test_empty_graph_needs_no_approval() {
        let graph = CausalValueGraph::new();
        let report = graph.evaluate_impact("ghost", DEFAULT_LOSS_AVERSION_KAPPA);
        assert_eq!(report.affected_node_count, 1);
        assert_eq!(report.total_weight, 0.0);
        assert!(!report.requires_human_approval);
    }

    #[test]
    fn test_blast_radius_stops_at_visited_nodes() {
        let mut graph = CausalValueGraph::new();
        for id in ["a", "b", "c"] {
            graph.insert_node(ValueNode {
                id: id.to_string(),
                weight: 0.5,
                expected_gain: 0.5,
                expected_loss: 0.0,
                dependencies: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            });
        }
        let radius = graph.compute_blast_radius("a");
        assert_eq!(radius.len(), 3);
    }
}
