//! Phase 1 acceptance: both transports pass IDENTICAL scenarios (§R1).
//! These tests spawn a real `kinetid` process and talk DTOs over UDS,
//! then replay the same script against the in-process DirectBackend.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use kineti::ipc::{
    dto, request, DirectBackend, SocketBackend, JournalWriter, SpendService, ENV_FORCE_DIRECT,
};
use kineti::ipc::pool::{Ceilings, Pool};
use kineti::memory::journal::{compute_hash, now_iso, Journal, Record, GENESIS};

// ── helpers ──────────────────────────────────────────────────────────────────

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp1-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    d
}

struct TestDaemon {
    child: Child,
    pub root: PathBuf,
}

impl TestDaemon {
    fn start(root: PathBuf) -> Self {
        Self::start_with_cap(root, 50.0)
    }

    /// Cap flows through the project's own kineti.toml — exactly how a real
    /// deployment configures the daemon.
    fn start_with_cap(root: PathBuf, cap_usd: f64) -> Self {
        std::fs::write(
            root.join("kineti.toml"),
            format!(
                "[providers.test]\n\
                 base_url = \"http://localhost:9\"\n\
                 api_key_env = \"KINETI_TEST_KEY\"\n\
                 default_model = \"test-model\"\n\n\
                 [limits]\n\
                 global_usd = {cap_usd}\n"
            ),
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
        panic!("daemon did not come up on {}", sock.display());
    }

    fn backend(&self) -> SocketBackend {
        SocketBackend::connect(&self.root.join(".kineti/kineti.sock")).expect("connect")
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        let sock = self.root.join(".kineti/kineti.sock");
        let _ = request(&sock, dto::Req::Shutdown);
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

fn ctx(stage: &str, worker: &str, est: u64) -> dto::ReserveCtx {
    dto::ReserveCtx { stage: stage.into(), worker: worker.into(), est_micro_usd: est }
}

/// The ONE scenario both backends must survive identically.
fn exercise_spend(spend: &dyn SpendService) {
    // reserve 300k → settle 250k → total 250k
    let r = spend.reserve(&ctx("s", "w", 300_000)).unwrap();
    assert_eq!(spend.settle(&r, 250_000).unwrap(), 250_000);
    // reserve 400k → settle 500k → total 750k
    let r2 = spend.reserve(&ctx("s", "w", 400_000)).unwrap();
    assert_eq!(spend.settle(&r2, 500_000).unwrap(), 750_000);

    let snap = spend.snapshot().unwrap();
    assert_eq!(snap.total_micro_usd, 750_000);
    assert!(!snap.tripped);

    // projected 750k + 1_300_000 > $2.00 cap → denied
    let err = spend.reserve(&ctx("s", "w", 1_300_000)).unwrap_err();
    assert!(err.contains("BREAKER"), "{err}");
    assert_eq!(spend.snapshot().unwrap().total_micro_usd, 750_000);
}

const TEST_CAP_MICRO: u64 = 2_000_000; // $2.00

#[test]
fn direct_and_daemon_backends_behave_identically() {
    // direct first
    let d = tmp("parity-direct");
    exercise_spend(&DirectBackend::new(&d, Ceilings::global_only(TEST_CAP_MICRO)).unwrap());

    // then through the wire
    let t = TestDaemon::start_with_cap(tmp("parity-daemon"), 2.0);
    exercise_spend(&t.backend());
}

#[test]
fn breaker_trips_on_settled_reality_in_both_backends() {
    let d = tmp("trip-direct");
    let be = DirectBackend::new(&d, Ceilings::global_only(500_000)).unwrap(); // $0.50
    let r = be.reserve(&ctx("s", "w", 100_000)).unwrap();
    be.settle(&r, 600_000).unwrap(); // reality crosses cap
    assert!(be.snapshot().unwrap().tripped);
    assert!(be.reserve(&ctx("s", "w", 1)).is_err());

    let t = TestDaemon::start_with_cap(tmp("trip-daemon"), 0.5);
    let sb = t.backend();
    let r = sb.reserve(&ctx("s", "w", 100_000)).unwrap();
    sb.settle(&r, 600_000).unwrap();
    assert!(sb.snapshot().unwrap().tripped);
    assert!(sb.reserve(&ctx("s", "w", 1)).is_err());
}

#[test]
fn journal_writes_validate_chain_and_verify_clean() {
    let t = TestDaemon::start(tmp("journal"));
    let sb = t.backend();

    assert_eq!(sb.head("").unwrap(), GENESIS);
    let one = rec("t-0001", GENESIS, serde_json::json!({"tool": "read_file"}));
    sb.append_batch("", vec![one]).unwrap();

    let head1 = sb.head("").unwrap();
    let two = rec("t-0002", &head1, serde_json::json!({"ok": true}));
    sb.append_batch("", vec![two]).unwrap();

    // forged record must be refused by the daemon's tail check
    let forged_prev = "deadbeef".repeat(8);
    let bad = rec("t-0003", &forged_prev, serde_json::json!({"evil": true}));
    assert!(sb.append_batch("", vec![bad]).is_err());

    // branch namespaces are independent chains
    assert_eq!(sb.head("w1").unwrap(), GENESIS);

    // reload from disk through the original verifier: chain intact
    let j = Journal::load(&t.root.join(".kineti/journal.jsonl"));
    j.verify().unwrap_or_else(|e| panic!("chain broken: {e}"));
    assert_eq!(j.records.len(), 2);
}

#[test]
fn stale_socket_fails_gracefully_and_cleans() {
    let d = tmp("stale");
    let sock = d.join(".kineti/kineti.sock");
    std::fs::write(&sock, b"not a socket").unwrap(); // regular file at socket path
    assert!(!kineti::ipc::ping(&sock), "must refuse, not panic");
    let err = SocketBackend::connect(&sock).unwrap_err();
    assert!(!err.is_empty());
    assert!(kineti::ipc::clean_stale_socket(&d), "dead endpoint file removed");
    assert!(!sock.exists());
}

#[test]
fn daemon_restart_reconstructs_pool_from_snapshot() {
    let d = tmp("restart");

    // pre-existing ledger (as if an earlier run settled $1.11)
    let micro: u64 = 1_110_000;
    std::fs::write(
        d.join(".kineti/spend.json"),
        serde_json::json!({"total_usd": 1.11, "total_micro_usd": micro}).to_string(),
    )
    .unwrap();

    // direct path seeds from the same file
    let be = DirectBackend::new(&d, Ceilings::global_only(TEST_CAP_MICRO)).unwrap();
    assert_eq!(be.snapshot().unwrap().total_micro_usd, micro);
    drop(be); // release the ledger lock before the daemon takes it

    let t = TestDaemon::start(d.clone());
    let snap = t.backend().snapshot().unwrap();
    assert_eq!(snap.total_micro_usd, micro, "daemon must seed from disk");
}

#[test]
fn human_reset_file_clears_tripped_breaker_both_backends() {
    let d = tmp("reset-direct");
    let be = DirectBackend::new(&d, Ceilings::global_only(500_000)).unwrap();
    let r = be.reserve(&ctx("s", "w", 100_000)).unwrap();
    be.settle(&r, 700_000).unwrap();
    assert!(be.snapshot().unwrap().tripped);
    std::fs::write(d.join(".kineti/spend.reset"), "").unwrap();
    assert!(be.reset_if_human_requested(&d).unwrap());
    assert_eq!(be.snapshot().unwrap().total_micro_usd, 0);
    assert!(be.reserve(&ctx("s", "w", 10)).is_ok());

    let t = TestDaemon::start_with_cap(tmp("reset-daemon"), 0.5);
    let sb = t.backend();
    let r = sb.reserve(&ctx("s", "w", 100_000)).unwrap();
    sb.settle(&r, 700_000).unwrap();
    assert!(sb.snapshot().unwrap().tripped);
    std::fs::write(t.root.join(".kineti/spend.reset"), "").unwrap();
    assert!(sb.reset_if_human_requested(&t.root).unwrap());
    assert!(sb.reserve(&ctx("s", "w", 10)).is_ok());
}

#[test]
fn force_direct_env_is_honored_even_with_live_daemon() {
    let t = TestDaemon::start(tmp("forcedirect"));
    // safety: never let a stray auto-spawn happen during this test
    std::env::set_var(ENV_FORCE_DIRECT, "1");
    let direct_selected = kineti::ipc::selects_direct(&t.root);
    std::env::remove_var(ENV_FORCE_DIRECT);
    assert!(direct_selected, "FORCE_DIRECT must win over a live socket");
}

#[test]
fn warm_ping_latency_reported() {
    let t = TestDaemon::start(tmp("latency"));
    let sock = t.root.join(".kineti/kineti.sock");
    // warm-up
    for _ in 0..20 {
        request(&sock, dto::Req::Ping).unwrap();
    }
    let mut samples: Vec<u128> = Vec::new();
    for _ in 0..300 {
        let t0 = Instant::now();
        request(&sock, dto::Req::Ping).unwrap();
        samples.push(t0.elapsed().as_micros());
    }
    samples.sort_unstable();
    let p50 = samples[samples.len() / 2];
    let p95 = samples[samples.len() * 95 / 100];
    println!("warm round-trip p50={p50}µs p95={p95}µs (target ≤50µs locally)");
    // CI runners are noisy VMs — assert a generous ceiling here and treat the
    // printed number as the real benchmark.
    assert!(p50 < 5_000, "p50 {p50}µs exceeds even a generous bound");
    assert!(p95 < 50_000, "p95 {p95}µs pathological");
}

#[test]
fn pool_units_are_exact_integers() {
    let p = Pool::new(Ceilings::global_only(1_000_000));
    let r = p
        .reserve(&kineti::ipc::dto::ReserveCtx {
            stage: String::new(),
            worker: String::new(),
            est_micro_usd: 333_333,
        })
        .unwrap();
    assert_eq!(p.settle(&r, 333_333), 333_333);
    assert_eq!(p.total(), 333_333);
}
