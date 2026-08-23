use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Words that assert causality — subject to time-order enforcement.
pub const CAUSAL_WORDS: [&str; 3] = ["caused", "triggers", "blocks"];

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub from: String, // cause
    pub to: String,   // effect
    pub word: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub proof_id: Option<String>,
}
fn default_status() -> String { "candidate".into() }

/// Pure validators — no IO. `times` maps record id → ISO timestamp.
pub fn validate_commit(
    edges: &[Edge],
    candidate: &Edge,
    times: &HashMap<String, String>,
) -> Result<(), String> {
    // 1. cycle detection: can we reach candidate.from starting at candidate.to?
    if reaches(&merge(edges, candidate), &candidate.to, &candidate.from, &mut HashSet::new()) {
        return Err(format!(
            "DAG REJECTED: edge {}→{} would create a cycle",
            candidate.from, candidate.to
        ));
    }
    // 2. temporal order for causal words: cause must not be after effect
    if CAUSAL_WORDS.contains(&candidate.word.as_str()) {
        let t_from = times.get(&candidate.from);
        let t_to = times.get(&candidate.to);
        if let (Some(a), Some(b)) = (t_from, t_to) {
            if a > b {
                return Err(format!(
                    "DAG REJECTED: time order violated — cause {} ({}) is after effect {} ({})",
                    candidate.from, a, candidate.to, b
                ));
            }
        }
    }
    Ok(())
}

fn merge<'a>(edges: &'a [Edge], candidate: &'a Edge) -> Vec<&'a Edge> {
    edges.iter().chain(std::iter::once(candidate)).collect()
}

fn reaches(
    edges: &[&Edge],
    current: &str,
    target: &str,
    seen: &mut HashSet<String>,
) -> bool {
    if current == target {
        return true;
    }
    if !seen.insert(current.to_string()) {
        return false;
    }
    for e in edges {
        if e.from == current && reaches(edges, &e.to, target, seen) {
            return true;
        }
    }
    false
}

/// Keyword recall over journal data + one-hop neighborhood expansion.
/// Fixes the retrieval ceiling for *causally related but lexically different* context — D1.
pub struct Graph {
    pub path: std::path::PathBuf,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn load(path: &std::path::Path) -> Self {
        let mut edges = Vec::new();
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Ok(e) = serde_json::from_str::<Edge>(line) {
                    edges.push(e);
                }
            }
        }
        Graph { path: path.to_path_buf(), edges }
    }

    pub fn commit(
        &mut self,
        candidate: Edge,
        times: &HashMap<String, String>,
    ) -> Result<(), String> {
        crate::memory::dag::validate_commit(&self.edges.clone(), &candidate, times)?;
        use std::io::Write;
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&self.path)
        {
            let _ = writeln!(f, "{}", serde_json::to_string(&candidate).unwrap_or_default());
        }
        self.edges.push(candidate);
        Ok(())
    }

    /// Score records by query-token overlap in their canonical data, then pull
    /// linked neighbors of the top hits.
    #[allow(dead_code)] // invoked from context assembly in v0.2
    pub fn recall(
        &self,
        records: &[crate::memory::journal::Record],
        query: &str,
        top_k: usize,
    ) -> Vec<crate::memory::journal::Record> {
        let tokens: Vec<String> = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 4)
            .map(String::from)
            .collect();
        if tokens.is_empty() {
            return vec![];
        }

        let mut scored: Vec<(usize, &crate::memory::journal::Record)> = records
            .iter()
            .map(|r| {
                let hay = crate::memory::journal::canonical(&r.data).to_lowercase();
                let score = tokens.iter().filter(|t| hay.contains(t.as_str())).count();
                (score, r)
            })
            .filter(|(s, _)| *s > 0)
            .collect();
        scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));

        let mut picked: Vec<&crate::memory::journal::Record> =
            scored.iter().take(top_k).map(|(_, r)| *r).collect();

        // one-hop expansion along causal edges
        let ids: HashSet<String> = picked.iter().map(|r| r.id.clone()).collect();
        let mut neighbor_ids: HashSet<String> = HashSet::new();
        for e in &self.edges {
            if ids.contains(&e.from) {
                neighbor_ids.insert(e.to.clone());
            }
            if ids.contains(&e.to) {
                neighbor_ids.insert(e.from.clone());
            }
        }
        for r in records {
            if neighbor_ids.contains(&r.id) && !ids.contains(&r.id) {
                picked.push(r);
            }
        }
        picked.into_iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: &str, secs: u64) -> (String, String) {
        (id.into(), format!("2026-08-23T00:{:02}:{:02}Z", secs / 60, secs % 60))
    }

    #[test]
    fn planted_cycle_rejected() {
        let edges = vec![
            Edge { from: "a".into(), to: "b".into(), word: "caused".into(), status: "candidate".into(), proof_id: None },
            Edge { from: "b".into(), to: "c".into(), word: "caused".into(), status: "candidate".into(), proof_id: None },
        ];
        let candidate = Edge { from: "c".into(), to: "a".into(), word: "caused".into(), status: "candidate".into(), proof_id: None };
        let times = [t("a", 0), t("b", 1), t("c", 2)].into_iter().collect();
        assert!(validate_commit(&edges, &candidate, &times).is_err());
    }

    #[test]
    fn time_order_violation_rejected() {
        // cause recorded AFTER its effect → reject
        let candidate = Edge { from: "late".into(), to: "early".into(), word: "caused".into(), status: "candidate".into(), proof_id: None };
        let times = [t("early", 5), t("late", 9)].into_iter().collect();
        assert!(validate_commit(&[], &candidate, &times).is_err());
    }

    #[test]
    fn valid_chain_accepted() {
        let edges = vec![Edge { from: "a".into(), to: "b".into(), word: "caused".into(), status: "candidate".into(), proof_id: None }];
        let candidate = Edge { from: "b".into(), to: "c".into(), word: "triggers".into(), status: "candidate".into(), proof_id: None };
        let times = [t("a", 0), t("b", 1), t("c", 2)].into_iter().collect();
        assert!(validate_commit(&edges, &candidate, &times).is_ok());
    }
}
