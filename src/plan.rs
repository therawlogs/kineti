//! Task Partition (Phase 6): the coordinator's stage-4 architecture document
//! MUST declare disjoint worker scopes. Parsing and validation are MECHANICAL
//! — no model judgment decides whether the swarm is well-formed.
//!
//! Contract inside architecture.md:
//!
//!   ## Task Partition
//!   - T1: title text | scope: src/db/** ; tests/db/** | deps: -
//!   - T2: title text | scope: src/api/**            | deps: T1
//!
//! Overlap between any two scopes bounces the pipeline back to stage 4;
//! unknown or cyclic deps likewise. The approved table becomes .kineti/plan.json.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const PARTITION_HEADER: &str = "## Task Partition";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub scopes: Vec<String>,
    pub deps: Vec<String>,
}

pub fn plan_path(root: &Path) -> std::path::PathBuf {
    root.join(".kineti/plan.json")
}

pub fn save(root: &Path, plan: &Plan) -> Result<(), String> {
    let p = plan_path(root);
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    std::fs::write(&p, serde_json::to_string_pretty(plan).unwrap())
        .map_err(|e| format!("plan write: {e}"))
}

pub fn load(root: &Path) -> Result<Plan, String> {
    let raw = std::fs::read_to_string(plan_path(root))
        .map_err(|_| "no .kineti/plan.json — approve a swarm spec first".to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("plan parse: {e}"))
}

// ── parsing ──────────────────────────────────────────────────────────────────

/// Extract + parse the partition from an architecture document. Returns
/// human-readable bounce reasons instead of a Plan when anything is off.
pub fn parse_partition(architecture_md: &str) -> Result<Plan, Vec<String>> {
    let mut errors = Vec::new();
    let section = match section_after(architecture_md, PARTITION_HEADER) {
        Some(s) => s,
        None => return Err(vec![format!(
            "missing '{PARTITION_HEADER}' section in architecture.md — declare one task per line: \
             `- ID: title | scope: glob;glob | deps: -`"
        )]),
    };

    let mut tasks = Vec::new();
    for (ln, raw) in section.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.starts_with('-') {
            continue; // prose around the table tolerated
        }
        let body = line.trim_start_matches('-').trim();
        let parts: Vec<&str> = body.split('|').map(str::trim).collect();
        if parts.len() != 3 {
            errors.push(format!("line {}: expected `- ID: title | scope: … | deps: …`", ln + 1));
            continue;
        }
        let id_part = parts[0];
        let id = match id_part.split_once(':') {
            Some((id, _)) => id.trim().to_string(),
            None => {
                errors.push(format!("line {}: missing `ID:` prefix", ln + 1));
                continue;
            }
        };
        let title = match id_part.split_once(':') {
            Some((_, t)) => t.trim().to_string(),
            None => String::new(),
        };
        let scopes: Vec<String> = parts[1]
            .strip_prefix("scope:")
            .unwrap_or(parts[1])
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if scopes.is_empty() {
            errors.push(format!("line {ln}: task {id} has no scope globs"));
            continue;
        }
        let deps: Vec<String> = parts[2]
            .strip_prefix("deps:")
            .unwrap_or(parts[2])
            .split(',')
            .map(str::trim)
            .filter(|d| !d.is_empty() && *d != "-")
            .map(String::from)
            .collect();
        tasks.push(Task { id, title, scopes, deps });
    }

    if tasks.is_empty() && errors.is_empty() {
        errors.push("Task Partition declares no tasks".into());
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    match validate(&tasks) {
        Ok(()) => Ok(Plan { tasks }),
        Err(e) => Err(e),
    }
}

fn validate(tasks: &[Task]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    if ids.len() != HashSet::<&str>::from_iter(ids.iter().copied()).len() {
        errors.push("duplicate task ids".into());
    }
    // pairwise scope overlap — mechanical, conservative
    for i in 0..tasks.len() {
        for j in i + 1..tasks.len() {
            for a in &tasks[i].scopes {
                for b in &tasks[j].scopes {
                    if globs_overlap(a, b) {
                        errors.push(format!(
                            "scope overlap: {}({a}) vs {}({b})",
                            tasks[i].id, tasks[j].id
                        ));
                    }
                }
            }
        }
    }
    // deps: known ids, acyclic
    let by_id: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    for t in tasks {
        for d in &t.deps {
            if !by_id.contains_key(d.as_str()) {
                errors.push(format!("{} depends on unknown task {d}", t.id));
            }
        }
    }
    if cycle_exists(tasks) {
        errors.push("dependency cycle among tasks".into());
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Topological order for execution waves. Errors on cycles/unknown deps.
pub fn topo_waves(plan: &Plan) -> Result<Vec<Vec<Task>>, String> {
    let by_id: HashMap<&str, &Task> = plan.tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut done: HashSet<String> = HashSet::new();
    let mut waves = Vec::new();
    while done.len() < plan.tasks.len() {
        let mut wave = Vec::new();
        for t in &plan.tasks {
            if done.contains(&t.id) {
                continue;
            }
            if t.deps.iter().all(|d| done.contains(d)) && by_id.contains_key(t.id.as_str()) {
                wave.push(t.clone());
            }
        }
        if wave.is_empty() {
            return Err("dependency deadlock (cycle or unknown dep)".into());
        }
        for t in &wave {
            done.insert(t.id.clone());
        }
        waves.push(wave);
    }
    Ok(waves)
}

fn cycle_exists(tasks: &[Task]) -> bool {
    // Kahn's algorithm: if we cannot consume everything there is a cycle
    let mut indeg: HashMap<&str, usize> = tasks.iter().map(|t| (t.id.as_str(), 0)).collect();
    let by_id: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    for t in tasks {
        for d in &t.deps {
            if by_id.contains_key(d.as_str()) {
                *indeg.get_mut(t.id.as_str()).unwrap() += 1;
            }
        }
    }
    let mut queue: Vec<&str> = indeg.iter().filter(|(_, v)| **v == 0).map(|(k, _)| *k).collect();
    let mut consumed = 0;
    while let Some(id) = queue.pop() {
        consumed += 1;
        for t in tasks {
            if t.deps.iter().any(|d| d == id) {
                let e = indeg.get_mut(t.id.as_str()).unwrap();
                *e -= 1;
                if *e == 0 {
                    queue.push(t.id.as_str());
                }
            }
        }
    }
    consumed != tasks.len()
}

fn section_after<'a>(doc: &'a str, header: &str) -> Option<&'a str> {
    let start = doc.find(header)? + header.len();
    let rest = &doc[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    Some(&rest[..end])
}

// ── glob overlap solver ──────────────────────────────────────────────────────

/// True when some path could match BOTH globs. Segments: literal, `*`
/// (exactly one segment), `**` (any number of segments incl. zero).
pub fn globs_overlap(a: &str, b: &str) -> bool {
    let sa: Vec<&str> = a.split('/').filter(|s| !s.is_empty()).collect();
    let sb: Vec<&str> = b.split('/').filter(|s| !s.is_empty()).collect();
    overlap_from(&sa, &sb)
}

/// Do two single-path-segment patterns share at least one concrete string?
/// Handles in-segment stars (`*.rs`, `lib*`) via fixed prefix/suffix checks;
/// conservative when both sides carry stars.
fn seg_intersects(p: &str, q: &str) -> bool {
    match (p.find('*'), q.find('*')) {
        (None, None) => p == q,
        (Some(_), None) => star_segment_covers(p, q),
        (None, Some(_)) => star_segment_covers(q, p),
        (Some(_), Some(_)) => {
            let p_suf = &p[p.rfind('*').unwrap() + 1..];
            let q_suf = &q[q.rfind('*').unwrap() + 1..];
            let common = p_suf.len().min(q_suf.len());
            p_suf[p_suf.len() - common..] == q_suf[q_suf.len() - common..]
        }
    }
}

fn star_segment_covers(pat: &str, lit: &str) -> bool {
    let first = pat.find('*').unwrap();
    let last = pat.rfind('*').unwrap();
    let pre = &pat[..first];
    let suf = &pat[last + 1..];
    lit.len() >= pre.len() + suf.len()
        && lit.starts_with(pre)
        && lit.ends_with(suf)
}

fn overlap_from(a: &[&str], b: &[&str]) -> bool {
    match (a.first(), b.first()) {
        (None, None) => true,
        (Some(&"**"), _) => overlap_from(&a[1..], b) || (!b.is_empty() && overlap_from(a, &b[1..])),
        (_, Some(&"**")) => !a.is_empty() && (overlap_from(a, &b[1..]) || overlap_from(&a[1..], b)),
        (None, Some(_)) | (Some(_), None) => false,
        (Some(pa), Some(pb)) => seg_intersects(pa, pb) && overlap_from(&a[1..], &b[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_solver_agrees_with_intuition() {
        assert!(globs_overlap("src/**", "src/db/**"));
        assert!(globs_overlap("src/**", "src/main.rs"));
        assert!(globs_overlap("src/*.rs", "src/lib.rs"));
        assert!(globs_overlap("**", "anything/at/all.rs"));
        assert!(globs_overlap("src/**", "src/**"));
        assert!(!globs_overlap("src/db/**", "src/api/**"));
        assert!(!globs_overlap("src/lib.rs", "src/bin.rs"));
        assert!(!globs_overlap("tests/**", "src/**"));
        assert!(globs_overlap("a/*/c", "a/b/c")); // * covers one segment
        assert!(!globs_overlap("a/*/c", "a/x/y/c")); // * ≠ two segments
    }

    fn mk(id: &str, scopes: &[&str], deps: &[&str]) -> Task {
        Task {
            id: id.into(),
            title: format!("task {id}"),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn parses_valid_table_and_rejects_overlaps_and_cycles() {
        let doc = "intro text\n\n## Task Partition\n\
                   - T1: database layer | scope: src/db/** | deps: -\n\
                   - T2: api routes | scope: src/api/** | deps: T1\n\
                   trailing notes\n";
        let plan = parse_partition(doc).expect("valid partition");
        assert_eq!(plan.tasks.len(), 2);
        let waves = topo_waves(&plan).unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0][0].id, "T1");
        assert_eq!(waves[1][0].id, "T2");

        // overlapping scopes bounce
        let bad = "## Task Partition\n- A: x | scope: src/** | deps: -\n- B: y | scope: src/db/** | deps: -\n";
        assert!(parse_partition(bad).unwrap_err().iter().any(|e| e.contains("overlap")));

        // duplicate scope exact
        let dup = "## Task Partition\n- A: x | scope: src/util.rs | deps: -\n- B: y | scope: src/util.rs | deps: -\n";
        assert!(parse_partition(dup).is_err());

        // cycle
        let cyc = "## Task Partition\n- A: x | scope: a/** | deps: B\n- B: y | scope: b/** | deps: A\n";
        assert!(parse_partition(cyc).unwrap_err().iter().any(|e| e.contains("cycle")));

        // unknown dep
        let unk = "## Task Partition\n- A: x | scope: a/** | deps: ZZZ\n";
        assert!(parse_partition(unk).unwrap_err().iter().any(|e| e.contains("unknown")));

        // missing section entirely
        assert!(parse_partition("no partition here").is_err());
    }

    #[test]
    fn plan_json_roundtrip() {
        let dir = std::env::temp_dir().join(format!("kpl-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = Plan { tasks: vec![mk("T9", &["x/**"], &[])] };
        save(&dir, &p).unwrap();
        let loaded = load(&dir).unwrap();
        assert_eq!(loaded.tasks[0].id, "T9");
        let _ = std::fs::remove_file(plan_path(&dir));
    }
}
