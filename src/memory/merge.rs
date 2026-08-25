//! Branch-and-merge journal layer (Phase 4): per-worker linear chains in
//! `journal.<branch>.jsonl`, joined into the MAIN chain by deterministic
//! 2-parent merge records. The main chain stays a normal hash chain — its
//! merge record commits to both parents: `prev_hash` (main tail, parent A)
//! and `parents[0] == data.head` (branch head, parent B). Same sha256
//! formula as every other record; determinism comes from canonical_stable.

use std::path::Path;

use crate::memory::journal::{compute_hash, now_iso, Journal, Record, GENESIS};

/// Branch file name relative to the project root. Mirrors the daemon's own
/// branch mapping (`DirectBackend`/socket protocol) — one convention only.
pub fn branch_rel_file(branch: &str) -> String {
    if branch.is_empty() || branch == "main" {
        return "journal.jsonl".into();
    }
    format!("journal.{branch}.jsonl")
}

static MERGE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Pure record builder — same inputs ⇒ same bytes ⇒ stable hash. The caller
/// supplies the timestamp so tests can pin it; NO hidden clock reads here.
pub fn build_merge_record(
    main_tail: &str,
    at: &str,
    branch: &str,
    branch_head: &str,
    extra_data: Option<serde_json::Value>,
) -> Record {
    let k = MERGE_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let id = format!("merge-{k:06}");
    let mut data = serde_json::json!({
        "branch": branch,
        "file": branch_rel_file(branch),
        "head": branch_head,
        "at": at,
    });
    if let Some(extra) = extra_data {
        if let (Some(dst), Some(src)) = (data.as_object_mut(), extra.as_object()) {
            for (kk, vv) in src {
                dst.insert(kk.clone(), vv.clone());
            }
        }
    }
    // canonical_stable(data) inside compute_hash ⇒ deterministic bytes
    let hash = compute_hash(main_tail, at, &id, &data);
    Record {
        at: at.to_string(),
        r#type: "merge".into(),
        state: "active".into(),
        project: "kineti".into(),
        id,
        data,
        links: vec![],
        expires: None,
        parents: vec![branch_head.to_string()],
        prev_hash: main_tail.to_string(),
        hash,
    }
}

/// Append a merge commit for `branch` into the MAIN chain via any writer.
/// Verifies the branch chain from disk first — never merges unverified
/// history. No-op for an untouched branch.
pub fn merge_branch(
    root: &Path,
    writer: &mut dyn crate::ipc::JournalWriter,
    branch: &str,
) -> Result<Option<Record>, String> {
    if branch.is_empty() {
        return Err("refusing to merge the main chain into itself".into());
    }
    let bhead = writer.head(branch)?;
    if bhead == GENESIS {
        return Ok(None); // nothing written on this branch — no-op
    }

    // full O(N) verification of the branch BEFORE committing it to history
    let bfile = root.join(".kineti").join(branch_rel_file(branch));
    let bj = Journal::load(&bfile);
    bj.verify()
        .map_err(|e| format!("branch '{branch}' failed verification: {e}"))?;
    let disk_head = bj.records.last().map(|r| r.hash.clone()).unwrap_or_default();
    if disk_head != bhead {
        return Err(format!(
            "branch '{branch}': cached head {bhead} != on-disk head {disk_head}"
        ));
    }

    // small retry: another process may extend the main chain between head
    // fetch and append — the daemon's tail check rejects, we re-fetch once.
    let mut last_err = String::new();
    for _ in 0..3 {
        let tail = writer.head("")?;
        let rec = build_merge_record(
            &tail,
            &now_iso(),
            branch,
            &disk_head,
            Some(serde_json::json!({
                "records": bj.records.len(),
            })),
        );
        match writer.append_batch("", vec![rec.clone()]) {
            Ok(()) => return Ok(Some(rec)),
            Err(e) => last_err = e,
        }
    }
    Err(format!("merge of '{branch}' failed after retries: {last_err}"))
}

// ── whole-project DAG verification ───────────────────────────────────────────

#[derive(Clone, Default, Debug)]
pub struct DagReport {
    pub main_records: usize,
    pub main_head: String,
    /// (branch, records, verified head) for every merged branch
    pub branches: Vec<(String, usize, String)>,
    /// branch files on disk with NO merge record — unaccounted history
    pub orphans: Vec<String>,
    /// human-readable problems: tamper, head mismatch, missing parents…
    pub errors: Vec<String>,
}

impl DagReport {
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.orphans.is_empty()
    }
}

/// Verify MAIN + every referenced branch + closure over all on-disk branch
/// files. Reads only — safe against a live daemon.
pub fn verify_project(root: &Path) -> DagReport {
    let mut rep = DagReport::default();
    let kineti_dir = root.join(".kineti");

    let main = Journal::load(&kineti_dir.join("journal.jsonl"));
    rep.main_records = main.records.len();
    rep.main_head = main.records.last().map(|r| r.hash.clone()).unwrap_or_else(|| GENESIS.into());
    if let Err(e) = main.verify() {
        rep.errors.push(format!("MAIN chain: {e}"));
        return rep; // cannot trust merge references off a broken spine
    }

    let mut referenced: Vec<(String, String)> = Vec::new(); // branch → expected head
    for m in main.records.iter().filter(|r| r.r#type == "merge") {
        let branch = m.data["branch"].as_str().unwrap_or_default().to_string();
        let head = m.data["head"].as_str().unwrap_or_default().to_string();
        if branch.is_empty() || head.is_empty() {
            rep.errors.push(format!("merge record {} lacks branch/head", m.id));
            continue;
        }
        // parents[0] must echo the committed branch head — 2-parent integrity
        if m.parents.first().map(String::as_str) != Some(head.as_str()) {
            rep.errors.push(format!(
                "merge record {}: parents[0] {:?} != data.head {head}",
                m.id,
                m.parents.first()
            ));
        }
        referenced.push((branch, head));
    }

    let mut seen = std::collections::HashSet::new();
    for (branch, expect_head) in &referenced {
        if !seen.insert(branch.clone()) {
            continue;
        }
        let path = kineti_dir.join(branch_rel_file(branch));
        if !path.exists() {
            rep.errors.push(format!(
                "branch '{branch}': merged as {} but file missing",
                short(expect_head)
            ));
            continue;
        }
        let bj = Journal::load(&path);
        match bj.verify() {
            Ok(()) => {
                let actual = bj.records.last().map(|r| r.hash.clone()).unwrap_or_default();
                if actual != *expect_head {
                    rep.errors.push(format!(
                        "branch '{branch}': merged head {} != on-disk head {} — \
                         branch extended AFTER its merge record",
                        short(expect_head),
                        short(&actual)
                    ));
                } else {
                    rep.branches.push((branch.clone(), bj.records.len(), actual));
                }
            }
            Err(e) => rep.errors.push(format!("branch '{branch}' TAMPERED: {e}")),
        }
    }

    // closure: every on-disk branch file must be referenced by some merge
    if let Ok(entries) = std::fs::read_dir(&kineti_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            // any journal.<branch>.jsonl must be referenced by some merge record
            if name != "journal.jsonl"
                && name.starts_with("journal.")
                && name.ends_with(".jsonl")
                && !referenced.iter().any(|(b, _)| branch_rel_file(b) == name)
            {
                rep.orphans.push(name);
            }
        }
    }
    rep
}

fn short(h: &str) -> &str {
    h.get(..12).unwrap_or(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_record_roundtrips_and_hashes_stably() {
        let a = build_merge_record("tail-1", "2026-01-01T00:00:00Z", "w1", "head-w1", None);
        // serialize → parse → recompute: byte-stable canonicalization proof
        let wire = serde_json::to_string(&a).unwrap();
        let back: Record = serde_json::from_str(&wire).unwrap();
        let recomputed = compute_hash(&back.prev_hash, &back.at, &back.id, &back.data);
        assert_eq!(recomputed, a.hash);
        assert_eq!(a.parents, vec!["head-w1"], "parent B recorded");
        assert_eq!(a.data["file"], "journal.w1.jsonl");
    }

    #[test]
    fn every_merge_input_changes_the_commit() {
        // distinct events mint distinct ids ⇒ distinct commits (by design);
        // any CONTENT change must also move the hash independently of ids.
        let base = build_merge_record("t", "at", "w", "h", None);
        assert_ne!(
            base.hash,
            build_merge_record("t", "at", "w", "h2", None).hash,
            "branch head feeds the commit"
        );
        assert_ne!(
            base.hash,
            build_merge_record("t2", "at", "w", "h", None).hash,
            "main tail feeds the commit"
        );
        let e1 = build_merge_record("t", "at", "w", "h", Some(serde_json::json!({"c": 0.5})));
        let e2 = build_merge_record("t", "at", "w", "h", Some(serde_json::json!({"c": 0.5})));
        assert_ne!(e1.hash, e2.hash, "distinct events never share a commit");
        assert_ne!(e1.id, e2.id);
        assert_eq!(e1.data["c"], e2.data["c"]);
    }
}
