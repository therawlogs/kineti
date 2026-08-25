//! Phase 3 acceptance: journal writes flow through the JournalWriter
//! boundary (O(1) head-cache appends), one-shot paths never spawn daemons,
//! and the ship gate refuses a tampered chain.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use kineti::memory::journal::{build, compute_hash, now_iso, Journal, Record, GENESIS};
use kineti::stages::{log_gate, ship_chain_check};

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp3-{tag}-{}-{nanos}", std::process::id()));
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
    let at = now_iso();
    let hash = compute_hash(prev, &at, id, &data);
    Record {
        at,
        r#type: "action".into(),
        state: "active".into(),
        project: "test".into(),
        id: id.into(),
        data,
        links: vec![],
        expires: None,
        parents: vec![],
        prev_hash: prev.into(),
        hash,
    }
}

#[test]
fn build_is_deterministic_and_parent_sensitive() {
    let a = build(GENESIS, "x-1", "action", &serde_json::json!({"n": 1}));
    // byte-stability: serialize → parse → recompute (clock-independent proof)
    let wire = serde_json::to_string(&a).unwrap();
    let back: kineti::memory::journal::Record =
        serde_json::from_str(&wire).unwrap();
    let recomputed =
        kineti::memory::journal::compute_hash(&back.prev_hash, &back.at, &back.id, &back.data);
    assert_eq!(recomputed, a.hash);

    // parent sensitivity: changing prev_hash must move the commit
    let other_parent =
        build("different-parent", "x-1", "action", &serde_json::json!({"n": 1}));
    assert_ne!(a.hash, other_parent.hash);
}

#[test]
fn boundary_roundtrip_direct_and_daemon_with_forge_rejection() {
    // ── direct ──
    let d = tmp("rt-direct");
    {
        let w = kineti::ipc::journal_writer_no_spawn(&d);
        assert_eq!(w.head("").unwrap(), GENESIS);
        w.append_batch("", vec![rec("t-0001", GENESIS, serde_json::json!({"k": 1}))])
            .unwrap();
        let h1 = w.head("").unwrap();
        w.append_batch("", vec![rec("t-0002", &h1, serde_json::json!({"k": 2}))])
            .unwrap();
        assert!(w
            .append_batch("", vec![rec("bad", "forged".repeat(4).as_str(), serde_json::json!({}))])
            .is_err());
    }
    let j = Journal::load(&d.join(".kineti/journal.jsonl"));
    j.verify().unwrap();
    assert_eq!(j.records.len(), 2);

    // ── through the daemon (no_spawn must ATTACH when one is live) ──
    let t = TestDaemon::start(tmp("rt-daemon"));
    {
        let w = kineti::ipc::journal_writer_no_spawn(&t.root);
        assert_eq!(w.head("").unwrap(), GENESIS);
        w.append_batch("", vec![rec("t-0001", GENESIS, serde_json::json!({"k": 1}))])
            .unwrap();
        let h1 = w.head("").unwrap();
        w.append_batch("", vec![rec("t-0002", &h1, serde_json::json!({"k": 2}))])
            .unwrap();
        assert!(w
            .append_batch("", vec![rec("bad", "forged".repeat(4).as_str(), serde_json::json!({}))])
            .is_err(), "daemon tail check must reject forged parents");
    }
    let j2 = Journal::load(&t.root.join(".kineti/journal.jsonl"));
    j2.verify().unwrap();
    assert_eq!(j2.records.len(), 2);
}

#[test]
fn no_spawn_journal_never_creates_daemon_artifacts() {
    let d = tmp("nospawn");
    {
        let w = kineti::ipc::journal_writer_no_spawn(&d);
        w.append_batch("", vec![rec("t-0001", GENESIS, serde_json::json!({"q": true}))])
            .unwrap();
    }
    assert!(!d.join(".kineti/kineti.sock").exists(), "must not spawn a daemon");
    assert!(!d.join(".kineti/daemon.json").exists());
    let j = Journal::load(&d.join(".kineti/journal.jsonl"));
    j.verify().unwrap();
}

#[test]
fn ship_chain_check_refuses_tampered_history() {
    let clean = tmp("chain-clean");
    assert!(ship_chain_check(&clean).is_ok(), "empty chain verifies at genesis");

    let dirty = tmp("chain-dirty");
    let path = dirty.join(".kineti/journal.jsonl");
    let mut j = Journal::load(&path);
    j.append("action", serde_json::json!({"tool": "read_file"}), vec![], "kineti");
    j.append("observation", serde_json::json!({"ok": true}), vec![], "kineti");
    drop(j);
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, raw.replacen("read_file", "Xead_file", 1)).unwrap();

    let err = ship_chain_check(&dirty).unwrap_err();
    assert!(err.contains("SHIP REFUSED"), "{err}");
    assert!(err.contains("HASH MISMATCH"), "{err}");
}

#[test]
fn gate_records_flow_through_boundary_and_verify() {
    let t = TestDaemon::start(tmp("gates"));
    log_gate(&t.root, "spec-approved", "human said yes");
    log_gate(&t.root, "shipped", "");

    let j = Journal::load(&t.root.join(".kineti/journal.jsonl"));
    j.verify().unwrap_or_else(|e| panic!("gate chain broken: {e}"));
    let gates: Vec<&Record> = j.records.iter().filter(|r| r.r#type == "gate").collect();
    assert_eq!(gates.len(), 2);
    assert_eq!(gates[0].data["kind"], "spec-approved");

    // second write chained onto the first even across separate calls
    assert_eq!(gates[1].prev_hash, gates[0].hash);
}
