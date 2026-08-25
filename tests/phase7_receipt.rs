//! Phase 7 acceptance: one receipt builder aggregates coordinator + worker
//! spend, gate timeline, egress rollup (incl. preserved worker logs), DAG
//! state, and the clean-files verdict; §8.2 blocks ship on dirty trees;
//! swarm success path folds worker journals into the DAG and preserves
//! egress before tearing down worktrees.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use kineti::ipc::journal_writer_no_spawn;
use kineti::memory::journal::build as build_rec;
use kineti::memory::merge::merge_branch;
use kineti::receipt;
use kineti::stages::ship_chain_check;

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp7-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    d
}

fn rec(id: &str, prev: &str, r#type: &str, data: serde_json::Value) -> kineti::memory::journal::Record {
    build_rec(prev, id, r#type, &data)
}

/// Handcraft a coordinator chain + two worker branch chains with costs,
/// merge the workers exactly like the swarm success path does.
fn fixture_with_workers(tag: &str) -> PathBuf {
    let d = tmp(tag);
    std::fs::write(d.join("root_goal"), "build the thing").ok();
    let mut w = journal_writer_no_spawn(&d);

    // coordinator: run-record with cost + one gate, chained correctly
    let h0 = w.head("").unwrap();
    let r1 = rec("rr-0001", &h0, "run-record",
        serde_json::json!({"outcome":"complete","cost_usd":0.05,"iterations":3}));
    w.append_batch("", vec![r1.clone()]).unwrap();
    w.append_batch(
        "",
        vec![rec("gate-0001", &r1.hash, "gate", serde_json::json!({"kind":"spec-approved","detail":"human said yes"}))],
    )
    .unwrap();

    for (branch, cost) in [("w-w1", 0.10), ("w-w2", 0.20)] {
        let head0 = w.head(branch).unwrap();
        w.append_batch(
            branch,
            vec![rec(
                &format!("{branch}-rec1"),
                &head0,
                "stage-outcome",
                serde_json::json!({"stage": format!("{branch}/build"), "cost_usd": cost / 2.0, "prompt_tokens": 100, "completion_tokens": 50}),
            )],
        )
        .unwrap();
        let head1 = w.head(branch).unwrap();
        w.append_batch(
            branch,
            vec![rec(
                &format!("{branch}-rec2"),
                &head1,
                "stage-outcome",
                serde_json::json!({"stage": format!("{branch}/qa"), "cost_usd": cost / 2.0, "prompt_tokens": 200, "completion_tokens": 80}),
            )],
        )
        .unwrap();
        merge_branch(&d, w.as_mut(), branch).unwrap();
    }
    d
}

#[test]
fn receipt_aggregates_coordinator_and_worker_spend() {
    let d = fixture_with_workers("agg");
    let s = receipt::build(&d);

    assert!((s.coordinator_cost_usd - 0.05).abs() < 1e-9);
    assert_eq!(s.workers.len(), 2);
    assert!((s.total_cost_usd() - 0.35).abs() < 1e-9, "{}", s.total_cost_usd());

    let w1 = s.workers.iter().find(|w| w.branch == "w-w1").unwrap();
    assert!((w1.cost_usd - 0.10).abs() < 1e-9);
    assert_eq!(w1.prompt_tokens, 300);
    assert_eq!(w1.completion_tokens, 130);
    assert_eq!(w1.records, 2);

    assert!(s.dag.is_clean());
    assert_eq!(s.last_run.as_ref().unwrap().2, 0.05);
}

#[test]
fn receipt_gate_timeline_is_sorted_and_complete() {
    let d = fixture_with_workers("gates");
    let s = receipt::build(&d);
    assert!(!s.gates.is_empty());
    let sorted = {
        let mut g: Vec<String> = s.gates.iter().map(|x| x.at.clone()).collect();
        g.sort();
        g
    };
    let actual: Vec<String> = s.gates.iter().map(|x| x.at.clone()).collect();
    assert_eq!(actual, sorted, "timeline must be chronological");
    assert!(s.gates.iter().any(|g| g.kind == "spec-approved"));
}

#[test]
fn egress_rollup_includes_preserved_worker_logs() {
    let d = fixture_with_workers("egress");
    // main + two preserved worker logs (as teardown would leave them)
    kineti::enforce::egress::record_at(&d, "api.test", "main send", "aa");
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    std::fs::write(d.join(".kineti/egress.w-w1.jsonl"), "{\"hash\":\"x\"}\n{\"hash\":\"y\"}\n").unwrap();

    let s = receipt::build(&d);
    let tags: Vec<(String, usize)> =
        s.egress.iter().map(|e| (e.tag.clone(), e.records)).collect();
    assert!(tags.contains(&("main".into(), 1)), "{tags:?}");
    assert!(tags.contains(&("worker:w-w1".into(), 2)), "{tags:?}");
}

// ── swarm closure path: journals fold into DAG, egress survives teardown ────

struct TestDaemon {
    child: Child,
    pub root: PathBuf,
}
impl TestDaemon {
    fn start(root: PathBuf) -> Self {
        std::fs::write(
            root.join("kineti.toml"),
            "[providers.test]\nbase_url = \"http://localhost:9\"\napi_key_env = \"K\"\ndefault_model = \"m\"\n",
        )
        .unwrap();
        let exe = env!("CARGO_BIN_EXE_kineti");
        let mut child = Command::new(exe)
            .args(["serve", "--foreground"])
            .current_dir(&root)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
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
        let _ = kineti::ipc::request(
            &self.root.join(".kineti/kineti.sock"),
            kineti::ipc::dto::Req::Shutdown,
        );
        for _ in 0..50 {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn worker_journal_folds_into_dag_and_receipt_stays_clean() {
    let t = TestDaemon::start(tmp("fold"));
    let mut w = journal_writer_no_spawn(&t.root);

    // worker writes its stage-outcome onto branch w-w9 through the wire
    let h0 = w.head("w-w9").unwrap();
    w.append_batch(
        "w-w9",
        vec![rec(
            "w9-rec1",
            &h0,
            "stage-outcome",
            serde_json::json!({"cost_usd": 0.33, "prompt_tokens": 10, "completion_tokens": 5}),
        )],
    )
    .unwrap();

    // pre-merge: orphan flagged by ship gate
    assert!(ship_chain_check(&t.root).is_err());

    // the exact call integrate_workers performs after a verified merge
    merge_branch(&t.root, w.as_mut(), "w-w9").unwrap();

    let s = receipt::build(&t.root);
    assert!(s.dag.is_clean());
    let w9 = s.workers.iter().find(|w| w.branch == "w-w9").expect("w9 present");
    assert!((w9.cost_usd - 0.33).abs() < 1e-9);
    assert!(ship_chain_check(&t.root).is_ok());
}

#[test]
fn clean_files_verdict_and_ship_refusal() {
    let d = tmp("clean");
    std::fs::write(d.join("code.rs"), "let ok = true;\n").unwrap();
    let cfg = kineti::config::Config::load();
    assert!(kineti::enforce::cleanfiles::gate(&d, &cfg.clean_files.forbid).is_ok());
    let s = receipt::build(&d);
    assert!(s.clean_files.is_ok());

    // seed a home-path violation → gate refuses and receipt flags it
    std::fs::write(d.join("leak.rs"), "static P: &str = \"/Users/somebody/x\";").unwrap(); // kineti-clean-ignore (fixture)
    // note: scanner skips the ignore marker line — use an unmarked second file
    std::fs::create_dir_all(d.join("src")).unwrap();
    std::fs::write(d.join("src/leak2.rs"), "const Q: &str = concat!(\"/Users\", \"/somebody\");").unwrap();
    assert!(kineti::enforce::cleanfiles::gate(&d, &cfg.clean_files.forbid).is_err());
    let s2 = receipt::build(&d);
    assert_eq!(s2.clean_files.unwrap_err(), 1);
}

#[test]
fn live_worktree_egress_appears_as_live_tag() {
    // uses phase5 machinery directly: create tree, write egress, roll up
    let repo = tmp("live-wt");
    // minimal git repo so Auto mode picks git worktree
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["init", "-q", "-b", "main"])
        .status()
        .unwrap();
    let wt = kineti::worktree::create(&repo, "w-live", mode_auto()).unwrap();
    std::fs::create_dir_all(wt.path.join(".kineti")).unwrap();
    kineti::enforce::egress::record_at(&wt.path, "web.search", "worker lookup", "hh");

    let s = receipt::build(&repo);
    assert!(s.egress.iter().any(|e| e.tag == "live:w-live" && e.records == 1),
        "{:?}", s.egress.iter().map(|e| e.tag.clone()).collect::<Vec<_>>());

    // preservation helper semantics: copy then destroy keeps audit trail
    let dst = repo.join(".kineti/egress.w-w-live.jsonl");
    std::fs::copy(wt.path.join(".kineti/egress.jsonl"), &dst).unwrap();
    kineti::worktree::destroy(&repo, &wt).unwrap();
    assert!(dst.exists());
    assert!(!wt.path.exists());
}

fn mode_auto() -> kineti::worktree::Mode {
    kineti::worktree::Mode::Auto
}
