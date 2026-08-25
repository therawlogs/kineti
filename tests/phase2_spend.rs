//! Phase 2 acceptance: scoped ceilings (per-stage / per-worker) enforced at
//! reserve, tripped on settled reality, cleared only by the human reset file;
//! estimation math matches wire prices; direct-mode ledger lock fails closed.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use kineti::ipc::dto::ReserveCtx;
use kineti::ipc::pool::Ceilings;
use kineti::ipc::{DirectBackend, SocketBackend, SpendService};
use kineti::provider;

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp2-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    d
}

struct TestDaemon {
    child: Child,
    pub root: PathBuf,
}

impl TestDaemon {
    /// Cap config flows through the project's own kineti.toml — exactly how a
    /// real deployment configures the daemon.
    fn start(root: PathBuf, global: f64, per_stage: f64, per_worker: f64) -> Self {
        std::fs::write(
            root.join("kineti.toml"),
            format!(
                "[providers.test]\n\
                 base_url = \"http://localhost:9\"\n\
                 api_key_env = \"KINETI_TEST_KEY\"\n\
                 default_model = \"test-model\"\n\n\
                 [limits]\n\
                 global_usd = {global}\n\
                 per_stage_usd = {per_stage}\n\
                 max_worker_usd = {per_worker}\n"
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

impl TestDaemon {
    fn backend(&self) -> SocketBackend {
        SocketBackend::connect(&self.root.join(".kineti/kineti.sock")).expect("connect")
    }
}

fn ctx(stage: &str, worker: &str, est_micro: u64) -> ReserveCtx {
    ReserveCtx { stage: stage.into(), worker: worker.into(), est_micro_usd: est_micro }
}

fn scoped(global_m: u64, stage_m: u64, worker_m: u64) -> Ceilings {
    Ceilings { global_micro: global_m, stage_micro: Some(stage_m), worker_micro: Some(worker_m) }
}

// ── ceilings through the SpendService boundary ──────────────────────────────

#[test]
fn per_stage_and_per_worker_ceilings_enforced_at_backend() {
    let d = tmp("scoped");
    let be = DirectBackend::new(&d, scoped(100_000_000, 10_000_000, 25_000_000)).unwrap();

    assert!(be.reserve(&ctx("build", "", 9_000_000)).is_ok());
    let err = be.reserve(&ctx("build", "", 2_000_000)).unwrap_err();
    assert!(err.contains("SPEND BREAKER") && err.contains("per-stage 'build'"), "{err}");

    assert!(be.reserve(&ctx("", "w1", 24_000_000)).is_ok());
    let err = be.reserve(&ctx("", "w1", 2_000_000)).unwrap_err();
    assert!(err.contains("per-worker 'w1'"), "{err}");

    // untouched scopes still have headroom
    assert!(be.reserve(&ctx("qa", "w2", 9_000_000)).is_ok());
}

#[test]
fn settled_stage_crossing_trips_whole_pool_until_human_reset() {
    let d = tmp("trip");
    let be = DirectBackend::new(&d, scoped(100_000_000, 10_000_000, 25_000_000)).unwrap();

    let r = be.reserve(&ctx("build", "", 8_000_000)).unwrap();
    be.settle(&r, 11_000_000).unwrap(); // reality crosses the stage cap

    assert!(be.snapshot().unwrap().tripped);
    // even an unrelated stage is denied — "halts everything immediately" (§3.1)
    let err = be.reserve(&ctx("qa", "", 1)).unwrap_err();
    assert!(err.contains("SPEND BREAKER"), "{err}");

    // human reset clears scoped ledgers along with everything else (§3.3)
    std::fs::write(d.join(".kineti/spend.reset"), "").unwrap();
    assert!(be.reset_if_human_requested(&d).unwrap());
    assert!(!be.snapshot().unwrap().tripped);
    assert!(be.reserve(&ctx("build", "", 5_000_000)).is_ok());
}

#[test]
fn daemon_parity_identical_ceiling_behavior() {
    exercise(&DirectBackend::new(&tmp("parity-direct"), scoped(2_000_000, 500_000, 500_000)).unwrap());

    // identical script over the wire, caps read from its own kineti.toml
    let t = TestDaemon::start(tmp("parity-daemon"), 2.0, 0.5, 0.5);
    exercise(&t.backend());
}

fn exercise(spend: &dyn SpendService) {
    assert!(spend.reserve(&ctx("build", "", 400_000)).is_ok());
    let err = spend.reserve(&ctx("build", "", 300_000)).unwrap_err();
    assert!(err.contains("per-stage 'build'"), "{err}");
    assert!(spend.reserve(&ctx("qa", "", 400_000)).is_ok()); // own ledger
    assert_eq!(spend.snapshot().unwrap().total_micro_usd, 800_000);

    // settle actual crossing qa's stage cap trips everything
    let r = spend.reserve(&ctx("qa", "", 50_000)).unwrap();
    spend.settle(&r, 600_000).unwrap();
    assert!(spend.snapshot().unwrap().tripped);
    assert!(spend.reserve(&ctx("build", "", 1)).is_err());
}

// ── estimation math ──────────────────────────────────────────────────────────

#[test]
fn estimator_matches_wire_prices() {
    // grok pricing: $12.5 in / $25 out per 1M tokens
    let micro = provider::estimate_cost_micro(12.5, 25.0, 40_000); // ≈10k in-tokens
    let expect_in = (10_000.0_f64 / 1e6 * 12.5 * 1e6).round() as u64; // 125_000
    let expect_out = (8_192.0_f64 / 1e6 * 25.0 * 1e6).round() as u64; // 204_800
    assert_eq!(micro, expect_in + expect_out);

    // free tier → zero estimate → passthrough reservation upstream
    assert_eq!(provider::estimate_cost_micro(0.0, 0.0, 999_999), 0);
}

// ── direct-mode ledger lock fails closed ────────────────────────────────────

#[test]
fn second_direct_backend_fails_closed_while_first_holds_lock() {
    let d = tmp("lock");
    let a = DirectBackend::new(&d, scoped(100, 50, 50)).unwrap();

    // same-process contention must also fail (flock is per open-file-description)
    let err = match DirectBackend::new_with_lock_timeout(
        &d,
        scoped(100, 50, 50),
        Duration::from_millis(300),
    ) {
        Ok(_) => panic!("second backend must NOT acquire the held ledger lock"),
        Err(e) => e,
    };
    assert!(err.contains("LEDGER LOCK"), "{err}");

    // releasing the holder frees the ledger immediately
    drop(a);
    assert!(
        DirectBackend::new_with_lock_timeout(&d, scoped(100, 50, 50), Duration::from_secs(2))
            .is_ok()
    );
}
