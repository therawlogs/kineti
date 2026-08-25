//! Guarantee net (Phase 0): every ETHOS standing rule as an executable test.
//! This suite is backend-blind by design — from Phase 1 it must stay green in
//! BOTH direct and daemon modes (CI matrix, §R1).

use kineti::enforce::{cleanfiles, egress, evidence, saga};
use kineti::ipc::dto::ReserveCtx;
use kineti::ipc::pool::Ceilings;
use kineti::ipc::{DirectBackend, SpendService};
use kineti::memory::journal::Journal;
use kineti::tools;

fn tmp(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kig-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

// ── G1: hash chain is tamper-evident and attributes the break ────────────────

#[test]
fn g1_empty_journal_verifies_at_genesis() {
    let path = tmp("g1a").join("journal.jsonl");
    assert!(Journal::load(&path).verify().is_ok());
}

#[test]
fn g1_chain_tamper_detected_and_attributed() {
    let dir = tmp("g1b");
    let path = dir.join("journal.jsonl");
    let mut j = Journal::load(&path);
    j.append("action", serde_json::json!({"tool": "read_file"}), vec![], "kineti");
    j.append("observation", serde_json::json!({"ok": true}), vec![], "kineti");
    assert!(j.verify().is_ok());

    // flip one byte of history on disk
    let raw = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, raw.replacen("read_file", "Xead_file", 1)).unwrap();

    let err = Journal::load(&path).verify().unwrap_err();
    assert!(err.contains("HASH MISMATCH"), "wrong error: {err}");
    assert!(err.starts_with("record 1"), "must attribute record index: {err}");
}

#[test]
fn g1_float_costs_survive_disk_roundtrip() {
    let path = tmp("g1c").join("journal.jsonl");
    let mut j = Journal::load(&path);
    let cost = 0.0125_f64 + 0.000975; // ugly accumulated float
    j.append("stage-outcome", serde_json::json!({"cost_usd": cost}), vec![], "kineti");
    assert!(Journal::load(&path).verify().is_ok(), "roundtrip must not flip bytes");
}

// ── G2: spend breaker trips hard; only the human reset file clears it ───────

#[test]
fn g2_breaker_trips_and_only_human_reset_clears() {
    let d = tmp("g2");
    let cap = Ceilings::global_only(50_000_000); // $50
    let ctx50 = ReserveCtx { stage: "s".into(), worker: String::new(), est_micro_usd: 49_990_000 };

    // under cap → ok
    let be = DirectBackend::new(&d, cap.clone()).unwrap();
    assert!(be.reserve(&ctx50).is_ok());

    // settled reality crosses the cap → breaker message halts everything
    let r = be
        .reserve(&ReserveCtx { stage: "s".into(), worker: String::new(), est_micro_usd: 10 })
        .unwrap();
    be.settle(&r, 60_000_000).unwrap();
    let snap = be.snapshot().unwrap();
    assert!(snap.tripped);
    let err = be.reserve(&ctx50).unwrap_err();
    assert!(err.contains("SPEND BREAKER"), "{err}");

    // without the human flag file, nothing resets
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    match be.reset_if_human_requested(&d) {
        Ok(false) => {}
        other => panic!("expected false, got {other:?}"),
    }
    assert!(be.snapshot().unwrap().tripped);

    // human creates .kineti/spend.reset → counter zeroed exactly once
    std::fs::write(d.join(".kineti/spend.reset"), "").unwrap();
    assert!(be.reset_if_human_requested(&d).unwrap());
    let snap = be.snapshot().unwrap();
    assert_eq!(snap.total_micro_usd, 0);
    assert!(!snap.tripped);
    assert!(be
        .reserve(&ReserveCtx { stage: "s".into(), worker: String::new(), est_micro_usd: 100 })
        .is_ok());

    // reset file was consumed — second call is a no-op
    assert!(!be.reset_if_human_requested(&d).unwrap());
}

// ── G3: saga rollback newest-first, failed undo logged not fatal ────────────

#[test]
fn g3_rolls_back_newest_first_across_files() {
    let d = tmp("g3a");
    let f = d.join("f.txt"); // mutated twice: proves ordering
    let a = d.join("a.txt");
    let b = d.join("b.txt");
    std::fs::write(&f, "v0").unwrap();
    std::fs::write(&a, "A0").unwrap();
    std::fs::write(&b, "B0").unwrap();

    let mut sg = saga::Saga::load(&d);
    sg.register_file_backup(&f);
    std::fs::write(&f, "v1").unwrap();
    sg.register_file_backup(&f);
    std::fs::write(&f, "v2").unwrap();
    sg.register_file_backup(&a);
    std::fs::write(&a, "A1").unwrap();
    sg.register_file_backup(&b);
    std::fs::write(&b, "B1").unwrap();

    assert_eq!(sg.rollback_all(), 4);
    // double-mutated file must unwind v2→v1→v0; oldest-first would strand v1
    assert_eq!(std::fs::read_to_string(&f).unwrap(), "v0");
    assert_eq!(std::fs::read_to_string(&a).unwrap(), "A0");
    assert_eq!(std::fs::read_to_string(&b).unwrap(), "B0");
}

#[test]
fn g3_failed_undo_logged_rest_continue() {
    use std::os::unix::fs::PermissionsExt;
    let d = tmp("g3b");
    let ro_dir = d.join("ro");
    std::fs::create_dir_all(&ro_dir).unwrap();

    let free = d.join("free.txt");
    let locked = ro_dir.join("locked.txt");
    std::fs::write(&free, "F0").unwrap();
    std::fs::write(&locked, "L0").unwrap();

    let mut sg = saga::Saga::load(&d);
    sg.register_file_backup(&free); // older
    sg.register_file_backup(&locked); // NEWER — attempted first

    // mutate both, then make locked.txt's undo impossible (read-only parent)
    std::fs::write(&free, "F1").unwrap();
    std::fs::remove_file(&locked).unwrap();
    std::fs::set_permissions(&ro_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let n = sg.rollback_all();

    // restore perms so temp cleanup can succeed
    let _ = std::fs::set_permissions(&ro_dir, std::fs::Permissions::from_mode(0o755));

    assert_eq!(n, 1, "locked undo fails (logged), free undo must still run");
    assert_eq!(std::fs::read_to_string(&free).unwrap(), "F0");
}

// ── G4: tool fence rejects every escape shape ────────────────────────────────

#[test]
fn g4_fence_rejects_every_escape_shape() {
    let d = tmp("g4");
    std::fs::write(d.join("inside.txt"), "x").unwrap();
    assert!(tools::resolve_in_root(&d, "inside.txt").is_ok());
    assert!(tools::resolve_in_root(&d, "../outside.txt").is_err());
    assert!(tools::resolve_in_root(&d, "sub/../../outside.txt").is_err());
    assert!(tools::resolve_in_root(&d, "/etc/passwd").is_err());

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", d.join("link")).unwrap();
        let err = tools::resolve_in_root(&d, "link").unwrap_err();
        assert!(err.contains("symlink escapes"), "{err}");
    }
}

// ── G5: ship refuses MISSING, FAILED, and STALE proofs ──────────────────────

#[test]
fn g5_ship_refuses_missing_failed_and_stale() {
    let d = tmp("g5");

    let e = evidence::check_ship(&d).unwrap_err();
    assert!(e.contains("MISSING"), "{e}");

    std::fs::write(d.join("code.txt"), "v1").unwrap();
    evidence::record(&d, "false", false, 1);
    let e = evidence::check_ship(&d).unwrap_err();
    assert!(e.contains("FAILED"), "{e}");

    evidence::record(&d, "true", true, 0);
    assert!(evidence::check_ship(&d).is_ok());

    std::fs::write(d.join("code.txt"), "v2").unwrap(); // fingerprint flips
    let e = evidence::check_ship(&d).unwrap_err();
    assert!(e.contains("STALE"), "{e}");
}

// ── G6: egress is hash-chained, ordered, and redacted ───────────────────────

#[test]
fn g6_egress_records_chain_in_order() {
    let d = tmp("g6a");
    egress::record_at(&d, "unit.test", "first send", "aa");
    egress::record_at(&d, "unit.test", "second send", "bb");

    let raw = std::fs::read_to_string(d.join(".kineti/egress.jsonl")).unwrap();
    let recs: Vec<serde_json::Value> =
        raw.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[1]["prev_hash"], recs[0]["hash"], "chain must link");
    assert_ne!(recs[0]["hash"], recs[1]["hash"]);
}

#[test]
fn g6_egress_redacts_key_shapes() {
    assert!(egress::redact("key is xai-AbCdEf123456789012345678 done").contains("[REDACTED]")); // kineti-clean-ignore (fixture)
    assert!(egress::redact("Bearer abcdefghijklmnopqrstuvwxyz12345").contains("[REDACTED]"));
    assert!(egress::redact("mail me at a@b.com now").contains("[REDACTED]"));
    assert_eq!(egress::redact("clean text"), "clean text");
}

// ── G7: clean-files scan — zero matches required ─────────────────────────────

#[test]
fn g7_scan_finds_home_paths_secrets_and_extra_terms() {
    let d = tmp("g7a");
    std::fs::create_dir_all(d.join("src")).unwrap();
    std::fs::write(d.join("src/a.rs"), "let p = \"/Users/someone/x\";").unwrap(); // kineti-clean-ignore (fixture)
    std::fs::write(d.join("b.md"), "token: sk-abcdefghijklmnopqrstuvwx").unwrap(); // kineti-clean-ignore (fixture)
    std::fs::write(d.join("c.md"), "all clear here").unwrap();

    let found = cleanfiles::scan(&d, &["Acme Client".to_string()]);
    assert!(found.iter().any(|f| f.kind == "home-path"));
    assert!(found.iter().any(|f| f.kind == "secret"));

    // scan output itself leaks neither names nor keys
    assert!(found.iter().all(|f| !f.snippet.contains("someone")));
    assert!(found.iter().all(|f| !f.snippet.contains("sk-abcdefgh")));

    std::fs::write(d.join("c.md"), "proposal prepared for Acme Client").unwrap();
    let found2 = cleanfiles::scan(&d, &["Acme Client".to_string()]);
    assert!(found2.iter().any(|f| f.kind == "forbidden-term"));

    // clean tree passes with zero findings
    assert!(cleanfiles::scan(&tmp("g7b"), &[]).is_empty());
}
