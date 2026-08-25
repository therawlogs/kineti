//! Phase 4 acceptance: per-branch chains join the main chain through
//! deterministic 2-parent merge records; tamper is isolated AND attributed;
//! orphan branches block; old linear journals verify unchanged.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use kineti::ipc::dto::ReserveCtx;
use kineti::ipc::pool::Ceilings;
use kineti::ipc::{journal_writer_no_spawn, DirectBackend, JournalWriter};
use kineti::memory::journal::{build as build_rec, now_iso, Journal, Record, GENESIS};
use kineti::memory::merge::{build_merge_record, merge_branch, verify_project, branch_rel_file};
use kineti::stages::ship_chain_check;

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp4-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    d
}

struct TestDaemon {
    child: Child,
    pub root: PathBuf,
}

impl TestDaemon {
    fn start(root: PathBuf) -> Self {
        std::fs::write(
            root.join("kineti.toml"),
            "[providers.test]\n\
             base_url = \"http://localhost:9\"\n\
             api_key_env = \"KINETI_TEST_KEY\"\n\
             default_model = \"test-model\"\n",
        )
        .expect("write kineti.toml");
        let exe = env!("CARGO_BIN_EXE_kineti");
        let mut child = Command::new(exe)
            .args(["serve", "--foreground"])
            .current_dir(&root)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn kineti serve");
        let sock = root.join(".kineti/kineti.sock");
        for _ in 0..600 {
            if kineti::ipc::ping(&sock) {
                return TestDaemon { child, root };
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = child.kill();
        let _ = child.wait();
        panic!("daemon did not come up");
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        let sock = self.root.join(".kineti/kineti.sock");
        let _ = kineti::ipc::request(&sock, kineti::ipc::dto::Req::Shutdown);
        for _ in 0..50 {
            if let Ok(Some(_)) = self.child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn rec(id: &str, prev: &str, data: serde_json::Value) -> Record {
    build_rec(prev, id, "action", &data)
}

/// Write `n` chained records onto a branch through the boundary.
fn fill_branch(w: &mut dyn JournalWriter, branch: &str, n: usize) -> String {
    let mut head = w.head(branch).unwrap();
    for i in 0..n {
        let r = rec(&format!("{branch}-{i:03}"), &head, serde_json::json!({"i": i}));
        w.append_batch(branch, vec![r.clone()]).unwrap();
        head = r.hash;
    }
    head
}

// ── backward compatibility ───────────────────────────────────────────────────

#[test]
fn linear_journal_without_branches_is_a_clean_dag() {
    let d = tmp("compat");
    {
        let w = journal_writer_no_spawn(&d);
        w.append_batch("", vec![rec("a-1", GENESIS, serde_json::json!({"x": 1}))])
            .unwrap();
    }
    let rep = verify_project(&d);
    assert!(rep.is_clean(), "{rep:?}");
    assert_eq!(rep.main_records, 1);
    assert!(rep.branches.is_empty());
    assert!(ship_chain_check(&d).is_ok());
}

#[test]
fn legacy_pre_dag_records_still_deserialize_with_empty_parents() {
    // a hand-written pre-Phase-4 record line (no parents field)
    let d = tmp("legacy");
    let line = r#"{"at":"2026-08-23T00:00:00.000Z","type":"action","state":"active","project":"kineti","id":"action-0001","data":{"tool":"read_file"},"links":[],"prev_hash":"GENESIS","hash":"deadbeef"}"#;
    std::fs::write(d.join(".kineti/journal.jsonl"), format!("{line}\n")).unwrap();
    let j = Journal::load(&d.join(".kineti/journal.jsonl"));
    assert_eq!(j.records.len(), 1);
    assert!(j.records[0].parents.is_empty());
    // hash won't match (fake hash above) — but prev-chain parse must succeed
}

// ── merge mechanics ──────────────────────────────────────────────────────────

#[test]
fn two_branches_merge_into_main_and_verify_clean() {
    let d = tmp("merge2");
    let mut w = journal_writer_no_spawn(&d);

    // main gets one ordinary record first
    w.append_batch("", vec![rec("m-000", GENESIS, serde_json::json!({"start": true}))])
        .unwrap();

    let h_w1 = fill_branch(w.as_mut(), "w1", 3);
    let h_w2 = fill_branch(w.as_mut(), "w2", 2);

    let m1 = merge_branch(&d, w.as_mut(), "w1").unwrap().expect("w1 non-empty");
    let m2 = merge_branch(&d, w.as_mut(), "w2").unwrap().expect("w2 non-empty");

    // merge records are chained onto each other on MAIN
    assert_eq!(m2.prev_hash, m1.hash, "second merge builds on first");
    assert_eq!(m1.parents, vec![h_w1]);
    assert_eq!(m2.parents, vec![h_w2]);
    assert_eq!(m1.data["file"], "journal.w1.jsonl");

    // empty branch is a no-op
    assert!(merge_branch(&d, w.as_mut(), "w3").unwrap().is_none());

    // main chain itself must still verify as a plain chain
    let main = Journal::load(&d.join(".kineti/journal.jsonl"));
    main.verify().expect("main spine intact after merges");

    let rep = verify_project(&d);
    assert!(rep.is_clean(), "{rep:?}");
    assert_eq!(rep.branches.len(), 2);
    assert_eq!(rep.main_records, 3); // m-000 + two merges
    assert!(ship_chain_check(&d).is_ok());
}

#[test]
fn merging_refuses_unverified_branch_history() {
    let d = tmp("badbranch");
    let mut w = journal_writer_no_spawn(&d);
    let _ = fill_branch(w.as_mut(), "w1", 2);

    // tamper a byte inside the branch file directly
    let bf = d.join(".kineti").join(branch_rel_file("w1"));
    let raw = std::fs::read_to_string(&bf).unwrap();
    std::fs::write(&bf, raw.replacen("\"i\":0", "\"i\":9", 1)).unwrap();

    let err = merge_branch(&d, w.as_mut(), "w1").unwrap_err();
    assert!(err.contains("'w1'") && err.contains("verification"), "{err}");
    // and nothing was committed to main
    assert_eq!(verify_project(&d).main_records, 0);
}

#[test]
fn tampered_branch_attributed_by_name_in_verify_project() {
    let d = tmp("attrib");
    let mut w = journal_writer_no_spawn(&d);
    let h1 = fill_branch(w.as_mut(), "w1", 2);
    let _h2 = fill_branch(w.as_mut(), "w2", 2);
    merge_branch(&d, w.as_mut(), "w1").unwrap();
    merge_branch(&d, w.as_mut(), "w2").unwrap();

    // tamper ONLY w2's file
    let bf = d.join(".kineti").join(branch_rel_file("w2"));
    let raw = std::fs::read_to_string(&bf).unwrap();
    std::fs::write(&bf, raw.replacen("\"i\":1", "\"i\":7", 1)).unwrap();

    let rep = verify_project(&d);
    assert!(!rep.is_clean());
    assert!(
        rep.errors.iter().any(|e| e.contains("'w2'") && e.contains("TAMPERED")),
        "must attribute w2: {:?}",
        rep.errors
    );
    // w1 remains clean and listed
    assert!(rep.branches.iter().any(|(b, _, _)| b == "w1"));
    let _ = h1;
}

#[test]
fn branch_extended_after_merge_is_caught() {
    let d = tmp("extend");
    let mut w = journal_writer_no_spawn(&d);
    fill_branch(w.as_mut(), "w1", 1);
    merge_branch(&d, w.as_mut(), "w1").unwrap();

    // sneak an EXTRA record onto the merged branch afterwards
    let head_now = w.head("w1").unwrap();
    w.append_batch("w1", vec![rec("w1-sneak", &head_now, serde_json::json!({"sneaky": true}))])
        .unwrap();

    let rep = verify_project(&d);
    assert!(
        rep.errors.iter().any(|e| e.contains("'w1'") && e.contains("AFTER")),
        "post-merge extension must be flagged: {:?}",
        rep.errors
    );
}

#[test]
fn orphan_branch_blocks_ship_until_merged() {
    let d = tmp("orphan");
    let mut w = journal_writer_no_spawn(&d);
    fill_branch(w.as_mut(), "worker-x", 1);
    // deliberately NOT merged

    let rep = verify_project(&d);
    assert_eq!(rep.orphans, vec!["journal.worker-x.jsonl".to_string()]);
    let ship_err = ship_chain_check(&d).unwrap_err();
    assert!(ship_err.contains("orphan"), "{ship_err}");

    // merging clears it
    merge_branch(&d, w.as_mut(), "worker-x").unwrap();
    assert!(verify_project(&d).is_clean());
    assert!(ship_chain_check(&d).is_ok());
}

#[test]
fn forged_merge_head_detected() {
    let d = tmp("forged");
    let mut w = journal_writer_no_spawn(&d);
    fill_branch(w.as_mut(), "w1", 1);

    // craft a merge record claiming a head the branch never reached
    let tail = w.head("").unwrap();
    let fake =
        build_merge_record(&tail, &now_iso(), "w1", "0000fabricated-head", None);
    w.append_batch("", vec![fake]).unwrap();

    let rep = verify_project(&d);
    assert!(
        rep.errors.iter().any(|e| e.contains("'w1'") && e.contains("merged head")),
        "fabricated head must be caught: {:?}",
        rep.errors
    );
    assert!(ship_chain_check(&d).is_err());
}

// ── wire parity: same flow against a live daemon ─────────────────────────────

#[test]
fn daemon_backed_branches_merge_identically() {
    let t = TestDaemon::start(tmp("daemon"));
    let mut sb = journal_writer_no_spawn(&t.root); // attaches to live daemon

    sb.append_batch(
        "",
        vec![rec("m-000", GENESIS, serde_json::json!({"start": true}))],
    )
    .unwrap();
    let hw = fill_branch(sb.as_mut(), "worker-a", 2);

    // spend still flows through the daemon while we're at it.
    // The suite may run under KINETI_FORCE_DIRECT=1 — clear it JUST here so
    // selection can see the live socket (restore afterwards).
    let forced = std::env::var("KINETI_FORCE_DIRECT").ok();
    std::env::remove_var("KINETI_FORCE_DIRECT");
    let sel = kineti::ipc::select_backends(&t.root, Ceilings::global_only(10_000_000));
    if forced.is_some() {
        std::env::set_var("KINETI_FORCE_DIRECT", "1");
    }
    let (svc, _) = match sel {
        Ok(p) => p,
        Err(_) => panic!("backend selection with live daemon"),
    };
    svc.reserve(&ReserveCtx { stage: "t".into(), worker: String::new(), est_micro_usd: 100 })
        .unwrap();

    let m = merge_branch(&t.root, sb.as_mut(), "worker-a")
        .unwrap()
        .expect("non-empty");
    assert_eq!(m.parents, vec![hw]);

    let rep = verify_project(&t.root);
    assert!(rep.is_clean(), "{rep:?}");
    assert!(rep.branches.iter().any(|(b, _, _)| b == "worker-a"));
    assert_eq!(svc.snapshot().unwrap().total_micro_usd, 100);
}

// ── direct-mode coexistence: spend lock + journal-only writer ────────────────

#[test]
fn journal_only_writer_never_contends_for_ledger_lock() {
    let d = tmp("coexist");
    let _be = DirectBackend::new(&d, Ceilings::global_only(1_000_000)).unwrap(); // holds lock

    // journal-only construction must work WHILE the ledger is locked
    let w = journal_writer_no_spawn(&d);
    w.append_batch("", vec![rec("j-1", GENESIS, serde_json::json!({"ok": true}))])
        .unwrap();
    let rep = verify_project(&d);
    assert!(rep.is_clean());
}
