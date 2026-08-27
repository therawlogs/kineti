//! Unified receipt (Phase 7): one builder, one truth source. Aggregates the
//! main chain, every merged worker branch, the causal graph, the gate
//! timeline, egress rollup, and the clean-files verdict into a single
//! summary that both `kineti receipt` and tests consume.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::memory::journal::{Journal, GENESIS};
use crate::memory::merge::verify_project;

#[derive(Clone, Debug, Default)]
pub struct WorkerCost {
    pub branch: String,
    pub cost_usd: f64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub records: usize,
}

#[derive(Clone, Debug, Default)]
pub struct GateEntry {
    pub at: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct ReceiptSummary {
    pub goal: String,
    pub records: usize,
    pub chain_head: String,
    pub causal_edges: usize,
    pub last_run: Option<(String, String, f64)>, // id, outcome, cost_usd
    /// coordinator spend = sum of run-record costs on the MAIN chain
    pub coordinator_cost_usd: f64,
    /// per-branch worker spend from stage-outcome records
    pub workers: Vec<WorkerCost>,
    pub gates: Vec<GateEntry>,
    pub dag: crate::memory::merge::DagReport,
    pub egress: Vec<crate::enforce::egress::EgressSource>,
    /// Ok(()) = zero forbidden matches; Err(findings)
    pub clean_files: Result<(), usize>,
}

impl ReceiptSummary {
    pub fn total_cost_usd(&self) -> f64 {
        self.coordinator_cost_usd + self.workers.iter().map(|w| w.cost_usd).sum::<f64>()
    }

    pub fn is_clean_history(&self) -> bool {
        self.dag.is_clean()
    }
}

fn branch_file(kineti_dir: &Path, branch: &str) -> PathBuf {
    if branch.is_empty() || branch == "main" {
        kineti_dir.join("journal.jsonl")
    } else {
        kineti_dir.join(format!("journal.{branch}.jsonl"))
    }
}

/// Build the full summary. Read-only; safe against a live daemon.
pub fn build(root: &Path) -> ReceiptSummary {
    let kineti_dir = root.join(".kineti");
    let goal = std::fs::read_to_string(kineti_dir.join("root_goal"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let j = Journal::load(&kineti_dir.join("journal.jsonl"));
    let g = crate::memory::graph::Graph::load(&kineti_dir.join("graph.jsonl"));

    let mut coordinator_cost = 0.0f64;
    let mut last_run = None;
    for r in &j.records {
        if r.r#type == "run-record" {
            let c = r.data["cost_usd"].as_f64().unwrap_or(0.0);
            coordinator_cost += c;
            last_run = Some((
                r.id.clone(),
                r.data["outcome"].as_str().unwrap_or("?").to_string(),
                c,
            ));
        }
    }

    let mut gates: Vec<GateEntry> = j
        .records
        .iter()
        .filter(|r| r.r#type == "gate")
        .map(|r| GateEntry {
            at: r.at.clone(),
            kind: r.data["kind"].as_str().unwrap_or("?").to_string(),
            detail: r.data["detail"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    gates.sort_by(|a, b| a.at.cmp(&b.at));

    let dag = verify_project(root);

    // worker spend: walk each MERGED branch's own chain
    let mut workers = Vec::new();
    for (branch, _records, _head) in &dag.branches {
        let bj = Journal::load(&branch_file(&kineti_dir, branch));
        let mut cost = 0.0f64;
        let (mut tin, mut tout) = (0u64, 0u64);
        for r in &bj.records {
            if r.r#type == "stage-outcome" {
                cost += r.data["cost_usd"].as_f64().unwrap_or(0.0);
                tin += r.data["prompt_tokens"].as_u64().unwrap_or(0);
                tout += r.data["completion_tokens"].as_u64().unwrap_or(0);
            }
        }
        workers.push(WorkerCost {
            branch: branch.clone(),
            cost_usd: cost,
            prompt_tokens: tin,
            completion_tokens: tout,
            records: bj.records.len(),
        });
    }
    workers.sort_by(|a, b| a.branch.cmp(&b.branch));

    let egress = crate::enforce::egress::summarize(root);

    let cfg = crate::config::Config::load();
    let clean_files = match crate::enforce::cleanfiles::scan(root, &cfg.clean_files.forbid) {
        findings if findings.is_empty() => Ok(()),
        findings => Err(findings.len()),
    };

    ReceiptSummary {
        goal,
        records: j.records.len(),
        chain_head: j
            .records
            .last()
            .map(|r| r.hash.clone())
            .unwrap_or_else(|| GENESIS.into()),
        causal_edges: g.edges.len(),
        last_run,
        coordinator_cost_usd: coordinator_cost,
        workers,
        gates,
        dag,
        egress,
        clean_files,
    }
}

/// Compact per-source egress counts for the receipt.
pub fn egress_map(summary: &ReceiptSummary) -> BTreeMap<String, usize> {
    summary
        .egress
        .iter()
        .map(|e| (e.tag.clone(), e.records))
        .collect()
}

/// JSON form for the C-ABI and machine consumers. Manual construction keeps
/// the wire shape stable regardless of internal struct churn.
pub fn to_json(s: &ReceiptSummary) -> serde_json::Value {
    serde_json::json!({
        "v": "1",
        "goal": s.goal,
        "records": s.records,
        "chain_head": s.chain_head,
        "causal_edges": s.causal_edges,
        "last_run": s.last_run.as_ref().map(|(id, outcome, cost)| {
            serde_json::json!({"id": id, "outcome": outcome, "cost_usd": cost})
        }),
        "spend": {
            "coordinator_usd": s.coordinator_cost_usd,
            "workers_usd": s.workers.iter().map(|w| w.cost_usd).sum::<f64>(),
            "total_usd": s.total_cost_usd(),
            "workers": s.workers.iter().map(|w| serde_json::json!({
                "branch": w.branch,
                "cost_usd": w.cost_usd,
                "prompt_tokens": w.prompt_tokens,
                "completion_tokens": w.completion_tokens,
                "records": w.records,
            })).collect::<Vec<_>>(),
        },
        "gates": s.gates.iter().map(|g| serde_json::json!({
            "at": g.at, "kind": g.kind, "detail": g.detail,
        })).collect::<Vec<_>>(),
        "dag": {
            "main_records": s.dag.main_records,
            "main_head": s.dag.main_head,
            "branches_merged": s.dag.branches.iter().map(|(b, n, h)| serde_json::json!({
                "branch": b, "records": n, "head": h,
            })).collect::<Vec<_>>(),
            "orphans": s.dag.orphans,
            "errors": s.dag.errors,
        },
        "egress": s.egress.iter().map(|e| serde_json::json!({
            "tag": e.tag, "records": e.records,
        })).collect::<Vec<_>>(),
        "clean_files_violations": match &s.clean_files {
            Ok(()) => serde_json::Value::Null,
            Err(n) => serde_json::json!(n),
        },
        "history_clean": s.is_clean_history(),
    })
}
