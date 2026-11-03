//! End-to-end native behavior tests use disposable project-local directories.
use serde_json::{json, Value};
use std::path::Path;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../.kineti/native-gate-tests")
            .join(kineti_cli::store::nonce().unwrap());
        fs::create_dir_all(&p).unwrap();
        let s = Self(p.canonicalize().unwrap());
        assert!(s
            .run(&["init", "--goal", "Fixture only", "--apply"])
            .status
            .success());
        fs::write(s.0.join("test.txt"), "declared fixture").unwrap();
        s
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kineti"))
            .args(args)
            .env("HOME", self.0.join("home"))
            .env("CODEX_HOME", self.0.join(".kineti/codex-home"))
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
    fn record(&self) -> Output {
        self.run(&[
            "test",
            "run",
            "--label",
            "release",
            "--required-test",
            "test.txt",
            "--",
            "/bin/sh",
            "-c",
            "printf '1 pass\\n0 fail\\n'",
        ])
    }
    fn check(&self) -> Output {
        self.run(&["test", "check", "--label", "release"])
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn value(o: &Output) -> Value {
    serde_json::from_slice(&o.stdout)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&o.stdout)))
}

#[test]
fn native_self_test_demonstrates_all_four_outcomes() {
    let p = Project::new();
    let out = p.run(&["self-test"]);
    assert!(out.status.success(), "{}", value(&out));
    let r = &value(&out)["result"];
    assert_eq!(r["stale_release_blocked"], true);
    assert_eq!(r["fresh_release_passed"], true);
    assert_eq!(r["invalid_signature_rejected"], true);
    assert_eq!(r["fabricated_output_accepted"], true);
}
#[test]
fn malformed_options_do_not_write() {
    let p = Project::new();
    let before = fs::read_dir(p.0.join(".kineti")).unwrap().count();
    for args in [
        vec!["test", "run", "--unknown"],
        vec!["spend", "log", "--usd", "not-money"],
        vec![
            "spend", "log", "--usd", "0.01", "--stage", "build", "--", "/bin/sh",
        ],
        vec!["undo", "push", "unexpected", "--", "/bin/sh"],
        vec!["init", "--goal"],
        vec!["undo", "rollback", "--oops"],
    ] {
        let o = p.run(&args);
        assert_eq!(o.status.code(), Some(2), "{}", value(&o));
    }
    assert_eq!(before, fs::read_dir(p.0.join(".kineti")).unwrap().count());
}
#[test]
fn fresh_then_changed_inputs_block() {
    let p = Project::new();
    assert!(p.record().status.success());
    assert!(p.check().status.success());
    fs::write(p.0.join("test.txt"), "changed").unwrap();
    assert_eq!(p.check().status.code(), Some(4));
}
#[test]
fn exact_empty_and_spaced_arguments_are_preserved() {
    let p = Project::new();
    let o = p.run(&[
        "test",
        "run",
        "--label",
        "args",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "test \"$1\" = '' && test \"$2\" = 'two words' && printf '1 pass\\n0 fail\\n'",
        "fixture",
        "",
        "two words",
    ]);
    assert!(o.status.success(), "{}", value(&o));
    assert_eq!(value(&o)["result"]["manifest"]["command"][4], "");
    assert_eq!(value(&o)["result"]["manifest"]["command"][5], "two words");
}
#[test]
fn environment_changes_invalidate() {
    let p = Project::new();
    let mut c = Command::new(env!("CARGO_BIN_EXE_kineti"));
    let o = c
        .current_dir(&p.0)
        .env("FIXTURE_MODE", "first")
        .args([
            "test",
            "run",
            "--label",
            "env",
            "--env-name",
            "FIXTURE_MODE",
            "--required-test",
            "test.txt",
            "--",
            "/bin/sh",
            "-c",
            "printf '1 pass\\n0 fail\\n'",
        ])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", value(&o));
    let o = Command::new(env!("CARGO_BIN_EXE_kineti"))
        .current_dir(&p.0)
        .env("FIXTURE_MODE", "second")
        .args(["test", "check", "--label", "env"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(4));
}
#[test]
fn signature_alteration_and_issuer_substitution_fail() {
    for field in ["signature", "issuer_key", "label"] {
        let p = Project::new();
        assert!(p.record().status.success());
        let path = p.0.join(".kineti/evidence.jsonl");
        let mut r = kineti_cli::store::lines(&path).unwrap();
        r[0][field] = json!("altered");
        fs::write(path, r[0].to_string() + "\n").unwrap();
        assert!(!p.check().status.success());
    }
}
#[test]
fn missing_checks_and_corrupt_history_fail_closed() {
    let p = Project::new();
    assert_eq!(p.check().status.code(), Some(5));
    assert!(!p.run(&["ci"]).status.success());
    assert!(p.record().status.success());
    fs::write(p.0.join(".kineti/evidence.jsonl"), "{broken}\n").unwrap();
    assert!(!p.check().status.success());
}
#[test]
fn exact_ci_command_map_rejects_substitution() {
    let p = Project::new();
    assert!(p.record().status.success());
    assert!(p
        .run(&["ci", "--require-evidence", "release"])
        .status
        .success());
    let o = p.run(&[
        "test",
        "check",
        "--label",
        "release",
        "--expect-argv",
        "[\"different-command\"]",
    ]);
    assert_eq!(o.status.code(), Some(7));
}
#[test]
fn spending_threshold_and_reset_need_headroom() {
    let p = Project::new();
    let o = p.run(&["spend", "log", "--stage", "build", "--usd", "9.5"]);
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    assert_eq!(
        p.run(&["spend", "reset", "--i-am-human"]).status.code(),
        Some(3)
    );
    let tighter = Project::new();
    fs::write(tighter.0.join("kineti.config.json"),json!({"settings":{"spend_limit_usd":{"global":50,"per_stage_default":10,"safety_factor":0.5}}}).to_string()).unwrap();
    assert_eq!(
        tighter
            .run(&["spend", "log", "--stage", "build", "--usd", "5"])
            .status
            .code(),
        Some(3)
    );
    assert_eq!(
        tighter
            .run(&["spend", "reset", "--i-am-human"])
            .status
            .code(),
        Some(3)
    );
}
#[test]
fn concurrent_accounting_does_not_lose_entries() {
    let p = Project::new();
    let children = (0..12)
        .map(|_| {
            Command::new(env!("CARGO_BIN_EXE_kineti"))
                .current_dir(&p.0)
                .args(["spend", "log", "--stage", "build", "--usd", "0.01"])
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap()
        })
        .collect::<Vec<_>>();
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    let o = p.run(&["spend", "check"]);
    assert!(o.status.success(), "{}", value(&o));
    assert_eq!(value(&o)["result"]["total_microcents"], 120000);
}
#[test]
fn corrupt_accounting_and_unresolved_reservations_block() {
    let p = Project::new();
    fs::write(
        p.0.join(".kineti/spend.reservations.jsonl"),
        json!({"id":"pending","status":"reserved"}).to_string() + "\n",
    )
    .unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    fs::remove_file(p.0.join(".kineti/spend.reservations.jsonl")).unwrap();
    fs::write(p.0.join(".kineti/spend.json"), "{bad}").unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    let o = p.run(&["spend", "recover"]);
    assert!(o.status.success());
    assert_eq!(value(&o)["result"]["apply_allowed"], false);
    assert_eq!(
        fs::read_to_string(p.0.join(".kineti/spend.json")).unwrap(),
        "{bad}"
    );
}
#[test]
fn undo_runs_newest_first_and_continues_definite_failure() {
    let p = Project::new();
    for (label, script) in [
        ("old", "printf old >> order.txt"),
        ("failure", "exit 1"),
        ("new", "printf new >> order.txt"),
    ] {
        let o = p.run(&[
            "undo", "push", "--label", label, "--", "/bin/sh", "-c", script,
        ]);
        assert!(o.status.success(), "{}", value(&o));
    }
    assert!(p
        .run(&[
            "undo",
            "push",
            "--label",
            "missing-executable",
            "--",
            "./does-not-exist"
        ])
        .status
        .success());
    let o = p.run(&["undo", "rollback", "--yes"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(fs::read_to_string(p.0.join("order.txt")).unwrap(), "newold");
}
#[test]
fn uncertain_undo_requires_inspection() {
    let p = Project::new();
    let file = p.0.join(".kineti/undo.native.jsonl");
    fs::write(
        &file,
        json!({"kind":"intent","run_id":"default","label":"unknown","action_id":"interrupted"})
            .to_string()
            + "\n",
    )
    .unwrap();
    kineti_cli::store::write(
        &p.0.join(".kineti/undo.native.guard.json"),
        &json!({"sha256":kineti_cli::compat::digest(fs::read(&file).unwrap())}),
    )
    .unwrap();
    assert_eq!(p.run(&["undo", "rollback", "--yes"]).status.code(), Some(3));
    assert!(p
        .run(&[
            "undo",
            "resolve",
            "--action-id",
            "interrupted",
            "--outcome",
            "not-applied",
            "--yes"
        ])
        .status
        .success());
    assert!(p.run(&["undo", "rollback", "--yes"]).status.success());
}
#[test]
fn init_preserves_locked_goal_and_rejects_damage() {
    let p = Project::new();
    let before = fs::read(p.0.join(".kineti/state.json")).unwrap();
    assert!(p.run(&["init", "--apply"]).status.success());
    assert_eq!(before, fs::read(p.0.join(".kineti/state.json")).unwrap());
    assert_eq!(
        p.run(&["init", "--goal", "different", "--apply"])
            .status
            .code(),
        Some(2)
    );
    fs::write(p.0.join(".kineti/state.json"), "broken").unwrap();
    assert_eq!(p.run(&["init", "--apply"]).status.code(), Some(3));
}
#[test]
fn codex_install_preview_backup_repetition_uninstall_preserves_settings() {
    let p = Project::new();
    let home = p.0.join("codex-home");
    fs::create_dir(&home).unwrap();
    let cfg = home.join("config.toml");
    fs::write(&cfg, "# user comment\nmodel = \"custom\"\n").unwrap();
    let h = home.to_str().unwrap();
    let preview = p.run(&["init", "--codex-home", h]);
    assert!(preview.status.success(), "{}", value(&preview));
    assert!(!fs::read_to_string(&cfg).unwrap().contains("mcp_servers"));
    assert!(p
        .run(&["init", "--codex-home", h, "--apply"])
        .status
        .success());
    assert!(p
        .run(&["init", "--codex-home", h, "--apply"])
        .status
        .success());
    let s = fs::read_to_string(&cfg).unwrap();
    fs::write(&cfg, format!("{s}\n[other]\nvalue = true\n")).unwrap();
    assert!(p
        .run(&["init", "--codex-home", h, "--uninstall", "--apply"])
        .status
        .success());
    let s = fs::read_to_string(&cfg).unwrap();
    assert!(s.contains("# user comment"));
    assert!(s.contains("[other]"));
    assert!(!s.contains("mcp_servers.kineti"));
}
#[test]
fn edited_owned_config_is_preserved() {
    let p = Project::new();
    let home = p.0.join("codex-home");
    let h = home.to_str().unwrap();
    assert!(p
        .run(&["init", "--codex-home", h, "--apply"])
        .status
        .success());
    let cfg = home.join("config.toml");
    let edited = fs::read_to_string(&cfg)
        .unwrap()
        .replace("mcp_servers.kineti", "mcp_servers.kineti_custom");
    fs::write(&cfg, &edited).unwrap();
    assert_eq!(
        p.run(&["init", "--codex-home", h, "--apply"]).status.code(),
        Some(3)
    );
    assert_eq!(fs::read_to_string(cfg).unwrap(), edited);
}

#[test]
fn removed_command_and_mcp_inventory_is_explicitly_rejected() {
    let p = Project::new();
    let inventory: Value =
        serde_json::from_str(include_str!("fixtures/command-inventory.json")).unwrap();
    for row in inventory["commands"].as_array().unwrap() {
        if row["disposition"] == "removed" {
            let name = row["name"].as_str().unwrap();
            let o = p.run(&[name]);
            assert_eq!(o.status.code(), Some(2), "{name}");
            assert!(value(&o)["error"].as_str().unwrap().contains("removed"));
            for sub in row["discovered_subcommands"].as_array().unwrap() {
                assert_eq!(p.run(&[name, sub.as_str().unwrap()]).status.code(), Some(2));
            }
        }
    }
    for name in inventory["native_assistant_removed"].as_array().unwrap() {
        assert_eq!(p.run(&[name.as_str().unwrap()]).status.code(), Some(2));
    }
    for row in inventory["mcp"].as_array().unwrap() {
        if row["disposition"] == "removed" {
            assert!(
                kineti_cli::mcp::call(&p.0, row["name"].as_str().unwrap(), &json!({})).is_err()
            );
        }
    }
}
#[test]
fn legacy_receipts_verify_but_do_not_gain_native_authority() {
    let p = Project::new();
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/legacy-golden.json")).unwrap();
    let r = &fixtures["fixtures"][0]["record"];
    fs::write(p.0.join(".kineti/evidence.jsonl"), r.to_string() + "\n").unwrap();
    fs::write(
        p.0.join(".kineti/evidence-issuer.pem"),
        fixtures["public_key"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        p.run(&["test", "check", "--label", "fixture"])
            .status
            .code(),
        Some(6)
    );
}
#[test]
fn execution_detects_input_mutation_and_missing_summary() {
    let p = Project::new();
    let o = p.run(&[
        "test",
        "run",
        "--label",
        "mutated",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf changed > test.txt; printf '1 pass\\n0 fail\\n'",
    ]);
    assert_eq!(o.status.code(), Some(1));
    let o = p.run(&[
        "test",
        "run",
        "--label",
        "silent",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "exit 0",
    ]);
    assert_eq!(o.status.code(), Some(1));
}
#[test]
fn native_accounting_chain_corruption_is_not_reset() {
    let p = Project::new();
    assert!(p
        .run(&["spend", "log", "--stage", "build", "--usd", "0.01"])
        .status
        .success());
    let f = p.0.join(".kineti/spend.native.jsonl");
    let mut v: Value = serde_json::from_str(fs::read_to_string(&f).unwrap().trim()).unwrap();
    v["microcents"] = json!(0);
    fs::write(&f, v.to_string() + "\n").unwrap();
    assert_eq!(
        p.run(&["spend", "reset", "--i-am-human"]).status.code(),
        Some(3)
    );
}
#[test]
fn legacy_spend_is_read_without_rewriting() {
    let p = Project::new();
    let state =
        json!({"tripped":false,"reason":null,"total_usd":1,"by_stage":{"build":1},"entries":1});
    let entry = json!({"at":"2026-10-16T17:00:00.000Z","stage":"build","model":"reported","tokens_in":0,"tokens_out":0,"usd":1});
    fs::write(p.0.join(".kineti/spend.json"), state.to_string()).unwrap();
    fs::write(
        p.0.join(".kineti/spend.log.jsonl"),
        entry.to_string() + "\n",
    )
    .unwrap();
    let o = p.run(&["spend", "check"]);
    assert!(o.status.success(), "{}", value(&o));
    assert_eq!(value(&o)["result"]["total_microcents"], 1000000);
    assert_eq!(
        fs::read_to_string(p.0.join(".kineti/spend.json")).unwrap(),
        state.to_string()
    );
    let golden: Value =
        serde_json::from_str(include_str!("fixtures/legacy-spend-golden.json")).unwrap();
    for case in golden["cases"].as_array().unwrap() {
        let fixture = Project::new();
        let state = case["state"].to_string();
        let log = case["entry"].to_string() + "\n";
        fs::write(fixture.0.join(".kineti/spend.json"), &state).unwrap();
        fs::write(fixture.0.join(".kineti/spend.log.jsonl"), &log).unwrap();
        let out = fixture.run(&["spend", "check"]);
        if case["original_accepts"] == false {
            assert_eq!(out.status.code(), Some(3), "{}", value(&out));
            assert_eq!(
                fs::read_to_string(fixture.0.join(".kineti/spend.json")).unwrap(),
                state
            );
            assert_eq!(
                fs::read_to_string(fixture.0.join(".kineti/spend.log.jsonl")).unwrap(),
                log
            );
            continue;
        }
        assert!(out.status.success(), "{}", value(&out));
        assert_eq!(
            value(&out)["result"]["total_microcents"],
            case["expected_microcents"]
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join(".kineti/spend.json")).unwrap(),
            state
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join(".kineti/spend.log.jsonl")).unwrap(),
            log
        );
        let mut damaged = case["state"].clone();
        damaged.as_object_mut().unwrap().remove("reason");
        fs::write(fixture.0.join(".kineti/spend.json"), damaged.to_string()).unwrap();
        assert_eq!(fixture.run(&["spend", "check"]).status.code(), Some(3));
        fs::write(fixture.0.join(".kineti/spend.json"), &state).unwrap();
        let mut damaged = case["entry"].clone();
        damaged["model"] = Value::Null;
        fs::write(
            fixture.0.join(".kineti/spend.log.jsonl"),
            damaged.to_string() + "\n",
        )
        .unwrap();
        assert_eq!(fixture.run(&["spend", "check"]).status.code(), Some(3));
        fs::write(fixture.0.join(".kineti/spend.log.jsonl"), &log).unwrap();
        let mut damaged = case["state"].clone();
        damaged["by_stage_microcents"] = json!({});
        fs::write(fixture.0.join(".kineti/spend.json"), damaged.to_string()).unwrap();
        assert_eq!(fixture.run(&["spend", "check"]).status.code(), Some(3));
    }
    let mut altered = state.clone();
    altered["total_usd"] = json!(1.004);
    altered["total_microcents"] = json!(1000000);
    fs::write(p.0.join(".kineti/spend.json"), altered.to_string()).unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
}
#[test]
fn native_mcp_protocol_discovers_and_calls() {
    use std::io::Write;
    let p = Project::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kineti"))
        .args(["mcp"])
        .current_dir(&p.0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for req in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"kineti_status","arguments":{}}}),
    ] {
        writeln!(input, "{req}").unwrap();
    }
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let rows = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["result"]["tools"].as_array().unwrap().len(), 9);
    assert_eq!(rows[2]["result"]["isError"], false);
}

#[test]
fn reviewed_recovery_preserves_damaged_original_and_binds_evidence() {
    let p = Project::new();
    let source = p.0.join(".kineti/verified-snapshot");
    fs::create_dir_all(source.join(".kineti")).unwrap();
    let s = json!({"tripped":false,"reason":null,"total_usd":1,"by_stage":{"build":1},"entries":1});
    fs::write(source.join(".kineti/spend.json"), s.to_string()).unwrap();
    fs::write(
        source.join(".kineti/spend.log.jsonl"),
        json!({"stage":"build","model":"reported","tokens_in":0,"tokens_out":0,"usd":1})
            .to_string()
            + "\n",
    )
    .unwrap();
    fs::write(p.0.join(".kineti/spend.json"), "damaged original").unwrap();
    fs::write(
        p.0.join("proof.txt"),
        "Reviewer-supplied fixture evidence of the exact snapshot, not a real bill.",
    )
    .unwrap();
    let proposal = p.run(&[
        "spend",
        "recover",
        "--source",
        ".kineti/verified-snapshot",
        "--evidence",
        "proof.txt",
    ]);
    assert!(proposal.status.success(), "{}", value(&proposal));
    let hash = value(&proposal)["result"]["proposal_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    let o = p.run(&[
        "spend",
        "recover",
        "--source",
        ".kineti/verified-snapshot",
        "--evidence",
        "proof.txt",
        "--proposal-sha256",
        &hash,
        "--apply",
        "--i-am-human",
    ]);
    assert!(o.status.success(), "{}", value(&o));
    assert_eq!(
        fs::read_to_string(p.0.join(".kineti/spend.json")).unwrap(),
        "damaged original"
    );
    assert!(p.run(&["spend", "check"]).status.success());
    let view_path = p.0.join(".kineti/spend.recovery-view.json");
    let original_view = fs::read(&view_path).unwrap();
    let mut changed: Value = serde_json::from_slice(&original_view).unwrap();
    changed["proposal_sha256"] = json!("0".repeat(64));
    fs::write(&view_path, changed.to_string()).unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    fs::write(&view_path, &original_view).unwrap();
    assert!(p.run(&["spend", "check"]).status.success());
    let audit = p.0.join(".kineti/accounting-recovery.jsonl");
    let original_audit = fs::read(&audit).unwrap();
    kineti_cli::store::append(
        &audit,
        &json!({"kind":"recovery-intent","id":"interrupted-after-view"}),
    )
    .unwrap();
    let guard = p.0.join(".kineti/accounting-recovery.guard.json");
    let original_guard = fs::read(&guard).unwrap();
    // Represent a crash after a fully durable intent, before completion.
    kineti_cli::store::write(
        &guard,
        &json!({"sha256":kineti_cli::compat::digest(fs::read(&audit).unwrap())}),
    )
    .unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    fs::write(&audit, &original_audit).unwrap();
    fs::write(&guard, &original_guard).unwrap();
    assert!(p.run(&["spend", "check"]).status.success());
    fs::write(source.join(".kineti/spend.json"), "altered snapshot").unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
}
#[test]
fn journal_unknown_costs_remain_blocking() {
    let p = Project::new();
    fs::write(
        p.0.join(".kineti/journal.jsonl"),
        json!({"type":"run-record","id":"rr-unresolved","data":{"costs_usd":null}}).to_string()
            + "\n",
    )
    .unwrap();
    let out = p.run(&["spend", "check"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(value(&out)["error"]
        .as_str()
        .unwrap()
        .contains("rr-unresolved"));
}
#[test]
fn verify_missing_or_untrusted_command_preserves_exit_nine() {
    let p = Project::new();
    assert_eq!(p.run(&["verify"]).status.code(), Some(9));
    fs::write(
        p.0.join("kineti.config.json"),
        json!({"settings":{"verify_command":["/bin/sh","-c","exit 0"]}}).to_string(),
    )
    .unwrap();
    assert_eq!(p.run(&["verify"]).status.code(), Some(9));
    assert!(p.run(&["verify", "--status"]).status.success());
}
#[test]
fn named_saga_begin_and_commit_are_preserved() {
    let p = Project::new();
    assert!(p
        .run(&["saga", "begin", "--run-id", "named"])
        .status
        .success());
    assert!(p
        .run(&[
            "saga", "push", "--run-id", "named", "--label", "inverse", "--", "/bin/sh", "-c",
            "exit 0"
        ])
        .status
        .success());
    assert!(p
        .run(&["saga", "commit", "--run-id", "named"])
        .status
        .success());
    assert_eq!(
        p.run(&["saga", "rollback", "--run-id", "named", "--yes"])
            .status
            .code(),
        Some(2)
    );
}

fn resign(p: &Project, r: &mut Value) {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use ed25519_dalek::{Signer, SigningKey};
    let pem = fs::read_to_string(p.0.join(".kineti/evidence-issuer.pem.private")).unwrap();
    let encoded = pem
        .lines()
        .filter(|s| !s.starts_with("-----"))
        .collect::<String>();
    let der = STANDARD.decode(encoded).unwrap();
    let key = SigningKey::from_bytes(der[16..].try_into().unwrap());
    r.as_object_mut().unwrap().shift_remove("signature");
    let bytes = kineti_cli::compat::canonical(r).unwrap();
    r["signature"] = json!(STANDARD.encode(key.sign(bytes.as_bytes()).to_bytes()));
}

#[test]
fn signed_expired_and_future_receipts_are_stale() {
    for time in ["2000-01-01T00:00:00.000Z", "2099-01-01T00:00:00.000Z"] {
        let p = Project::new();
        assert!(p.record().status.success());
        let file = p.0.join(".kineti/evidence.jsonl");
        let mut rows = kineti_cli::store::lines(&file).unwrap();
        rows[0]["at"] = json!(time);
        resign(&p, &mut rows[0]);
        fs::write(file, rows[0].to_string() + "\n").unwrap();
        assert_eq!(p.check().status.code(), Some(4));
    }
}

#[test]
fn dependency_and_runtime_identity_changes_are_stale() {
    let p = Project::new();
    fs::write(p.0.join("Cargo.lock"), "fixture dependency").unwrap();
    assert!(p.record().status.success());
    fs::write(p.0.join("Cargo.lock"), "changed dependency").unwrap();
    assert_eq!(p.check().status.code(), Some(4));
    assert!(p.record().status.success());
    let file = p.0.join(".kineti/evidence.jsonl");
    let mut rows = kineti_cli::store::lines(&file).unwrap();
    let last = rows.last_mut().unwrap();
    last["manifest"]["runtime"]["executable_sha256"] = json!("0".repeat(64));
    resign(&p, last);
    fs::write(
        file,
        rows.iter()
            .map(|v| v.to_string() + "\n")
            .collect::<String>(),
    )
    .unwrap();
    assert_eq!(p.check().status.code(), Some(4));
}

#[test]
fn missing_state_has_repair_instruction_and_symlink_inputs_are_rejected() {
    use std::os::unix::fs::symlink;
    let p = Project::new();
    symlink(&p.0, p.0.join("cycle")).unwrap();
    let out = p.run(&[
        "test",
        "run",
        "--label",
        "cycle",
        "--required-test",
        "cycle",
        "--",
        "/bin/sh",
        "-c",
        "exit 0",
    ]);
    assert_eq!(out.status.code(), Some(2));
    fs::remove_file(p.0.join(".kineti/state.json")).unwrap();
    let out = p.run(&["status"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(value(&out)["error"]
        .as_str()
        .unwrap()
        .contains("init --goal"));
}

#[test]
fn skipped_summary_cannot_authorize_release() {
    let p = Project::new();
    let out = p.run(&[
        "test",
        "run",
        "--label",
        "release",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf '1 pass\\n0 fail\\n1 skip\\n'",
    ]);
    assert!(!out.status.success());
    assert!(!p.check().status.success());
}

#[test]
fn excessive_token_integer_is_rejected_before_accounting_writes() {
    let p = Project::new();
    let out = p.run(&[
        "spend",
        "log",
        "--stage",
        "build",
        "--usd",
        "0.01",
        "--tokens-in",
        "9007199254740992",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!p.0.join(".kineti/spend.native.jsonl").exists());
}

#[test]
fn undo_guard_inspection_preserves_exact_interrupted_bytes() {
    let p = Project::new();
    assert!(p
        .run(&["undo", "push", "--label", "inspect", "--", "/bin/sh", "-c", "exit 0"])
        .status
        .success());
    fs::remove_file(p.0.join(".kineti/undo.native.guard.json")).unwrap();
    assert_eq!(p.run(&["undo", "rollback", "--yes"]).status.code(), Some(3));
    let before = fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap();
    let preview = value(&p.run(&["undo", "inspect"]));
    let hash = preview["result"]["sha256"].as_str().unwrap();
    assert_eq!(
        p.run(&["undo", "inspect", "--expected-sha256", "wrong", "--yes"])
            .status
            .code(),
        Some(3)
    );
    assert!(p
        .run(&["undo", "inspect", "--expected-sha256", hash, "--yes"])
        .status
        .success());
    assert_eq!(
        fs::read(p.0.join(".kineti/undo-inspection").join(hash)).unwrap(),
        before
    );
    assert_eq!(
        fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap(),
        before
    );
    assert!(p.run(&["undo", "rollback", "--yes"]).status.success());
}

#[test]
fn interrupted_recovery_blocks_before_any_view_is_installed() {
    let p = Project::new();
    fs::write(p.0.join(".kineti/accounting-recovery.initialized"), "1\n").unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    let audit = p.0.join(".kineti/accounting-recovery.jsonl");
    kineti_cli::store::append(
        &audit,
        &json!({"kind":"recovery-intent","id":"interrupted-before-view"}),
    )
    .unwrap();
    kineti_cli::store::write(
        &p.0.join(".kineti/accounting-recovery.guard.json"),
        &json!({"sha256":kineti_cli::compat::digest(fs::read(&audit).unwrap())}),
    )
    .unwrap();
    let out = p.run(&["spend", "check"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(value(&out)["error"]
        .as_str()
        .unwrap()
        .contains("unfinished recovery"));
    assert!(!p.0.join(".kineti/spend.recovery-view.json").exists());
}

#[test]
fn legacy_accounting_journal_encodings_and_checkpoint_are_preserved() {
    let p = Project::new();
    let golden: Value =
        serde_json::from_str(include_str!("fixtures/legacy-journal-golden.json")).unwrap();
    let text = golden["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.to_string() + "\n")
        .collect::<String>();
    let file = p.0.join(".kineti/journal.jsonl");
    fs::write(&file, &text).unwrap();
    let out = p.run(&["spend", "check"]);
    assert!(out.status.success(), "{}", value(&out));
    let checkpoint = p.0.join(".kineti/journal.jsonl.checkpoint.json");
    let expected =
        json!({"version":1,"bytes":text.len(),"sha256":kineti_cli::compat::digest(&text)});
    kineti_cli::store::write(&checkpoint, &expected).unwrap();
    assert!(p.run(&["spend", "check"]).status.success());
    let altered = text.replace("Fixture only", "Changed inputs");
    fs::write(&file, &altered).unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    fs::remove_file(checkpoint).unwrap();
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    fs::write(file, &text).unwrap();
    assert!(p.run(&["spend", "check"]).status.success());
}

#[test]
fn legacy_snapshot_only_reservation_cannot_pass_or_write() {
    let p = Project::new();
    let state = json!({"tripped":false,"reason":null,"total_usd":0,"total_microcents":0,"by_stage":{},"by_stage_microcents":{},"entries":0,"routing_reservations":{"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa":{"session_id":"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb","stage":"build","microcents":1000000,"status":"unknown"}}});
    fs::write(p.0.join(".kineti/spend.json"), state.to_string()).unwrap();
    let before = fs::read(p.0.join(".kineti/spend.json")).unwrap();
    for args in [
        vec!["spend", "check"],
        vec!["spend", "log", "--stage", "build", "--usd", "0.01"],
        vec!["spend", "reset", "--i-am-human"],
    ] {
        let out = p.run(&args);
        assert_eq!(out.status.code(), Some(3), "{}", value(&out));
    }
    assert_eq!(before, fs::read(p.0.join(".kineti/spend.json")).unwrap());
    assert!(!p.0.join(".kineti/spend.native.jsonl").exists());
}

#[test]
fn equals_options_and_empty_delimiter_validate_before_writes() {
    let p = Project::new();
    let good = p.run(&[
        "test",
        "run",
        "--label=equals",
        "--required-test=test.txt",
        "--",
        "/bin/sh",
        "-c",
        "test \"$1\" = '' && printf '1 pass\\n0 fail\\n'",
        "fixture",
        "",
    ]);
    assert!(good.status.success(), "{}", value(&good));
    assert!(p.run(&["test", "check", "--label=equals"]).status.success());
    for args in [
        vec!["spend", "check", "--"],
        vec!["spend", "log", "--usd=0.01", "--stage=   "],
        vec!["spend", "log", "--usd=0.01", "--model=  "],
        vec!["spend", "reset", "--i-am-human", "--extend-usd=0"],
        vec!["test", "check", "--label=equals", "--label", "equals"],
        vec!["spend", "reset", "--i-am-human=true"],
    ] {
        let out = p.run(&args);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", value(&out));
    }
    assert!(!p.0.join(".kineti/spend.lock").exists());
    assert!(!p.0.join(".kineti/spend.native.jsonl").exists());
}

#[test]
fn operational_evidence_remains_distinct_from_test_authority() {
    for kind in ["check", "package"] {
        let p = Project::new();
        fs::write(
            p.0.join("kineti.config.json"),
            json!({"settings":{"verify_command":"different default test"}}).to_string(),
        )
        .unwrap();
        let record = p.run(&[
            "evidence",
            "run",
            "--label",
            "operation",
            "--kind",
            kind,
            "--",
            "/bin/sh",
            "-c",
            "exit 0",
        ]);
        assert!(record.status.success(), "{}", value(&record));
        let checked = p.run(&["evidence", "check", "--label", "operation"]);
        assert!(checked.status.success(), "{}", value(&checked));
        assert_eq!(value(&checked)["result"]["gate_authorized"], false);
        assert_eq!(
            p.run(&["test", "check", "--label", "operation"])
                .status
                .code(),
            Some(7)
        );
        assert_eq!(
            p.run(&["ci", "--require-evidence", "operation"])
                .status
                .code(),
            Some(1)
        );
    }
}

#[test]
fn completion_persistence_failure_does_not_replay_undo_effect() {
    let p = Project::new();
    let script="printf x >> .kineti/effect; rm .kineti/undo.native.guard.json; mkdir .kineti/undo.native.guard.json";
    assert!(p
        .run(&[
            "undo",
            "push",
            "--run-id",
            "interrupted",
            "--label",
            "effect",
            "--",
            "/bin/sh",
            "-c",
            script
        ])
        .status
        .success());
    let first = p.run(&["undo", "rollback", "--run-id", "interrupted", "--yes"]);
    assert_eq!(first.status.code(), Some(3), "{}", value(&first));
    assert_eq!(fs::read(p.0.join(".kineti/effect")).unwrap(), b"x");
    let history = fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap();
    assert_eq!(
        p.run(&["undo", "rollback", "--run-id", "interrupted", "--yes"])
            .status
            .code(),
        Some(3)
    );
    assert_eq!(fs::read(p.0.join(".kineti/effect")).unwrap(), b"x");
    assert_eq!(
        history,
        fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap()
    );
}

#[test]
fn main_legacy_receipts_keep_signatures_without_native_authority() {
    let f: Value = serde_json::from_str(include_str!("fixtures/main-legacy-golden.json")).unwrap();
    let key = f["public_key"].as_str().unwrap();
    for case in f["receipts"].as_array().unwrap() {
        let r = &case["record"];
        assert!(kineti_cli::receipt::signature(r, key), "{}", case["name"]);
        assert_eq!(
            kineti_cli::compat::legacy_unsigned(r).unwrap(),
            case["unsigned"].as_str().unwrap()
        );
        for field in [
            "session_id",
            "parent_run_id",
            "issuer_key",
            "signature",
            "encoding",
        ] {
            let mut altered = r.clone();
            altered[field] = json!("altered");
            assert!(!kineti_cli::receipt::signature(&altered, key), "{field}");
        }
        assert!(!kineti_cli::receipt::signature(r, "untrusted key"));
        let p = Project::new();
        fs::write(p.0.join(".kineti/evidence-issuer.pem"), key).unwrap();
        let bytes = r.to_string() + "\n";
        fs::write(p.0.join(".kineti/evidence.jsonl"), &bytes).unwrap();
        assert_eq!(
            p.run(&["evidence", "check", "--label", r["label"].as_str().unwrap()])
                .status
                .code(),
            Some(6)
        );
        assert_eq!(
            p.run(&["ci", "--require-evidence", r["label"].as_str().unwrap()])
                .status
                .code(),
            Some(1)
        );
        assert_eq!(
            fs::read_to_string(p.0.join(".kineti/evidence.jsonl")).unwrap(),
            bytes
        );
    }
}

#[test]
fn main_legacy_accounting_matches_original_decisions_without_rewrites() {
    let f: Value = serde_json::from_str(include_str!("fixtures/main-legacy-golden.json")).unwrap();
    for case in f["histories"].as_array().unwrap() {
        let p = Project::new();
        let state = case["state"].to_string();
        let log = case["log"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.to_string() + "\n")
            .collect::<String>();
        fs::write(p.0.join(".kineti/spend.json"), &state).unwrap();
        fs::write(p.0.join(".kineti/spend.log.jsonl"), &log).unwrap();
        let out = p.run(&["spend", "check"]);
        assert_eq!(
            out.status.success(),
            case["original_check_passes"].as_bool().unwrap(),
            "{}: {}",
            case["name"],
            value(&out)
        );
        if out.status.success() {
            assert_eq!(
                value(&out)["result"]["total_microcents"],
                case["total_microcents"]
            );
        } else {
            assert_eq!(out.status.code(), Some(3));
        }
        assert_eq!(
            fs::read_to_string(p.0.join(".kineti/spend.json")).unwrap(),
            state
        );
        assert_eq!(
            fs::read_to_string(p.0.join(".kineti/spend.log.jsonl")).unwrap(),
            log
        );
    }
}

#[test]
fn ci_requires_complete_exact_current_and_legacy_command_declarations() {
    let command = json!(["/bin/sh", "-c", "printf '1 pass\\n0 fail\\n'"]);
    for (settings, map, explicit, passes) in [
        (json!({}), json!({"release":command}), None, true),
        (json!({}), json!({"release":["wrong"]}), None, false),
        (json!({}), json!({"other":command}), None, false),
        (json!({}), json!({"release":"not-array"}), None, false),
        (json!({}), json!({}), Some(json!({"release":command})), true),
        (json!({}), json!({}), Some(json!({})), false),
        (
            json!({"verify_commands":{"release":command},"verify_command_argv":["wrong"],"verify_command":"wrong"}),
            json!({}),
            None,
            true,
        ),
        (
            json!({"verify_command_argv":command,"verify_command":"wrong"}),
            json!({}),
            None,
            true,
        ),
        (
            json!({"verify_command":"/bin/sh -c \"printf '1 pass\\n0 fail\\n'\""}),
            json!({}),
            None,
            true,
        ),
        (
            json!({"verify_commands":{"release":["wrong"]}}),
            json!({"release":command}),
            None,
            true,
        ),
    ] {
        let p = Project::new();
        fs::write(
            p.0.join(".kineti/gate.json"),
            json!({"required_checks":["release"],"commands":map}).to_string(),
        )
        .unwrap();
        fs::write(
            p.0.join("kineti.config.json"),
            json!({"settings":settings}).to_string(),
        )
        .unwrap();
        if let Some(value) = &explicit {
            fs::write(p.0.join("mapping.json"), value.to_string()).unwrap();
        }
        assert!(p.record().status.success());
        let args = if explicit.is_some() {
            vec!["ci", "--evidence-commands", "mapping.json"]
        } else {
            vec!["ci"]
        };
        let out = p.run(&args);
        assert_eq!(
            out.status.success(),
            passes,
            "settings={settings}, map={map}, explicit={explicit:?}: {}",
            value(&out)
        );
    }
}

#[test]
fn operational_label_mapping_and_explicit_override_preserve_exact_argv() {
    for kind in ["check", "package"] {
        let p = Project::new();
        let argv = json!(["/bin/sh", "-c", "exit 0", "fixture", ""]);
        fs::write(
            p.0.join("kineti.config.json"),
            json!({"settings":{"verify_command":"wrong default","verify_commands":{"op":argv}}})
                .to_string(),
        )
        .unwrap();
        assert!(p
            .run(&[
                "evidence",
                "run",
                "--label",
                "op",
                "--kind",
                kind,
                "--env-name",
                "LANG",
                "--",
                "/bin/sh",
                "-c",
                "exit 0",
                "fixture",
                ""
            ])
            .status
            .success());
        assert!(p
            .run(&["evidence", "check", "--label", "op"])
            .status
            .success());
        assert_eq!(
            p.run(&[
                "evidence",
                "check",
                "--label",
                "op",
                "--expect-argv",
                "[\"wrong\"]"
            ])
            .status
            .code(),
            Some(7)
        );
        assert!(p
            .run(&[
                "evidence",
                "check",
                "--label",
                "op",
                "--expect-argv",
                &argv.to_string()
            ])
            .status
            .success());
        let mut changed = std::process::Command::new(env!("CARGO_BIN_EXE_kineti"));
        let out = changed
            .args(["evidence", "check", "--label", "op"])
            .env("LANG", "changed-fixture")
            .current_dir(&p.0)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4));
        fs::write(p.0.join("test.txt"), "changed").unwrap();
        assert_eq!(
            p.run(&["evidence", "check", "--label", "op"]).status.code(),
            Some(4)
        );
    }
}

fn exact_tree(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn walk(
        base: &std::path::Path,
        at: &std::path::Path,
        out: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
    ) {
        for entry in fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.insert(path.strip_prefix(base).unwrap().to_path_buf(), vec![]);
                walk(base, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(base).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    walk(root, root, &mut result);
    result
}
#[test]
fn malformed_spending_leaves_exact_tree_unchanged_before_and_after_initialization() {
    for initialized in [false, true] {
        let p = Project::new();
        if initialized {
            let out = p.run(&["spend", "log", "--stage", "build", "--usd", "48"]);
            assert_eq!(out.status.code(), Some(3));
        }
        let before = exact_tree(&p.0);
        for args in [
            vec![
                "spend",
                "log",
                "--stage",
                "build",
                "--usd",
                "0.01",
                "--unknown",
            ],
            vec!["spend", "check", "--unknown"],
            vec!["spend", "status", "--unknown"],
            vec!["spend", "reset", "--i-am-human", "--unknown"],
            vec!["spend", "check", "--"],
            vec!["spend", "log", "--stage", "build", "--usd", "0.01", "--"],
            vec!["spend", "log", "--stage", " build ", "--usd", "0.01"],
            vec![
                "spend", "log", "--stage", "build", "--model", " ", "--usd", "0.01",
            ],
            vec!["spend", "log", "--stage", "build", "--usd", "NaN"],
            vec![
                "spend",
                "log",
                "--stage",
                "build",
                "--usd",
                "0.01",
                "--tokens-in",
                "9007199254740992",
            ],
            vec!["spend", "reset", "--i-am-human", "--extend-usd", "0"],
        ] {
            let out = p.run(&args);
            assert_eq!(out.status.code(), Some(2), "{args:?}: {}", value(&out));
            assert_eq!(before, exact_tree(&p.0), "{args:?}");
        }
        assert!(kineti_cli::spend::charge(&p.0, "build\0", "reported", 0, 0, "0.01").is_err());
        assert!(kineti_cli::mcp::call(
            &p.0,
            "kineti_spend_record",
            &json!({"stage":"bad\0","usd":"0.01"})
        )
        .is_err());
        assert_eq!(before, exact_tree(&p.0));
    }
}

#[test]
fn native_receipt_encoding_and_session_cannot_be_substituted() {
    let p = Project::new();
    assert!(p.record().status.success());
    let raw = fs::read_to_string(p.0.join(".kineti/evidence.jsonl")).unwrap();
    let r: Value = serde_json::from_str(raw.trim()).unwrap();
    let key = fs::read_to_string(p.0.join(".kineti/evidence-issuer.pem")).unwrap();
    assert!(kineti_cli::receipt::signature(&r, &key));
    for field in ["encoding", "receipt_version"] {
        let mut altered = r.clone();
        altered.as_object_mut().unwrap().shift_remove(field);
        assert!(!kineti_cli::receipt::signature(&altered, &key));
    }
    let mut altered = r.clone();
    altered["manifest"]["session_ref"] = json!("different");
    assert!(!kineti_cli::receipt::signature(&altered, &key));
}

#[test]
fn verify_declaration_is_independent_from_required_ci_labels() {
    let p = Project::new();
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({"commands":{"release":["release-check"]}}).to_string(),
    )
    .unwrap();
    fs::write(
        p.0.join("kineti.config.json"),
        json!({"settings":{"verify_command_argv":["/bin/sh","-c","exit 0"]}}).to_string(),
    )
    .unwrap();
    let out = p.run(&["verify", "--status"]);
    assert!(out.status.success(), "{}", value(&out));
    assert_eq!(
        p.run(&["ci", "--require-evidence", "missing"])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn positional_inverse_equals_and_literal_tail_arguments_are_preserved() {
    let p = Project::new();
    assert!(p
        .run(&[
            "undo",
            "push",
            "--run-id",
            "eq",
            "--label",
            "eq",
            "printf a=b"
        ])
        .status
        .success());
    assert_eq!(
        p.run(&["undo", "push", "--inverse", "echo x", "--"])
            .status
            .code(),
        Some(2)
    );
    let argv = [
        "/bin/sh",
        "-c",
        "printf '1 pass\\n0 fail\\n'",
        "fixture",
        "",
        "--json",
        "--x=y",
        "--",
    ];
    let mut args = vec![
        "test",
        "run",
        "--label=tail",
        "--required-test=test.txt",
        "--",
    ];
    args.extend_from_slice(&argv);
    let out = p.run(&args);
    assert!(out.status.success(), "{}", value(&out));
    assert_eq!(value(&out)["result"]["manifest"]["command"], json!(argv));
}

#[test]
fn pending_original_reservations_cannot_be_hidden_by_recovery_snapshot() {
    let f: Value = serde_json::from_str(include_str!("fixtures/main-legacy-golden.json")).unwrap();
    for name in ["pending-reserved", "pending-unknown-partial", "log-only"] {
        let p = Project::new();
        let c = f["histories"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        fs::write(p.0.join(".kineti/spend.json"), c["state"].to_string()).unwrap();
        fs::write(
            p.0.join(".kineti/spend.log.jsonl"),
            c["log"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.to_string() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        let source = p.0.join(".kineti/clean-candidate");
        fs::create_dir_all(source.join(".kineti")).unwrap();
        fs::write(
            source.join(".kineti/spend.json"),
            json!({"tripped":false,"reason":null,"total_usd":0,"by_stage":{},"entries":0})
                .to_string(),
        )
        .unwrap();
        fs::write(
            p.0.join("proof.txt"),
            "Clean snapshot is not proof resolving an existing unknown.",
        )
        .unwrap();
        let before = exact_tree(&p.0);
        let out = p.run(&[
            "spend",
            "recover",
            "--source",
            ".kineti/clean-candidate",
            "--evidence",
            "proof.txt",
        ]);
        assert_eq!(out.status.code(), Some(3), "{}", value(&out));
        assert_eq!(before, exact_tree(&p.0));
        // Even a previously created view must not hide original obligations.
        fs::write(p.0.join(".kineti/spend.recovery-view.json"),json!({"original_sources":[],"candidate_sources":[],"verified_source":".kineti/clean-candidate"}).to_string()).unwrap();
        assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    }
}

#[test]
fn malformed_spending_never_initializes_an_empty_workspace() {
    let p = Project::new();
    fs::remove_dir_all(p.0.join(".kineti")).unwrap();
    fs::remove_file(p.0.join("test.txt")).unwrap();
    let before = exact_tree(&p.0);
    for args in [
        vec!["spend", "check", "--"],
        vec!["spend", "check", "--unknown"],
        vec!["spend", "log", "--usd", "0.01", "--stage", " "],
        vec!["spend", "reset", "--i-am-human", "--unknown"],
    ] {
        assert_eq!(p.run(&args).status.code(), Some(2));
        assert_eq!(before, exact_tree(&p.0));
    }
}

#[test]
fn verify_missing_and_malformed_declarations_keep_exit_nine() {
    let p = Project::new();
    for declaration in [json!(""), json!([]), json!(17), json!([""])] {
        fs::write(
            p.0.join("kineti.config.json"),
            json!({"settings":{"verify_command":declaration}}).to_string(),
        )
        .unwrap();
        assert_eq!(p.run(&["verify", "--status"]).status.code(), Some(9));
    }
}

#[test]
fn shared_runner_callers_preserve_definite_results_and_truncation() {
    use std::os::unix::fs::PermissionsExt;
    for (name, command, exit, success) in [
        ("zero", "exit 0", json!(0), true),
        ("nonzero", "exit 19", json!(19), false),
        ("signal", "kill -TERM $$", Value::Null, false),
        ("spawn-failure", "", json!(127), false),
        (
            "truncated",
            "printf '1 pass\\n0 fail\\n'; dd if=/dev/zero bs=8000001 count=1 2>/dev/null",
            json!(0),
            false,
        ),
    ] {
        let p = Project::new();
        let argv: Vec<String> = if name == "spawn-failure" {
            let path = p.0.join("not-executable");
            fs::write(&path, "not an executable").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            vec!["./not-executable".into()]
        } else {
            vec!["/bin/sh".into(), "-c".into(), command.into()]
        };
        let mut args = vec!["test", "run", "--label", name, "--kind", "check", "--"];
        args.extend(argv.iter().map(String::as_str));
        assert_eq!(p.run(&args).status.success(), success, "receipt {name}");
        let records = kineti_cli::store::lines(&p.0.join(".kineti/evidence.jsonl")).unwrap();
        let receipt = records.last().unwrap();
        assert_eq!(receipt["exit_code"], exit, "{name}: {receipt}");
        assert_eq!(receipt["verification"]["truncated"], name == "truncated");
        fs::write(
            p.0.join("kineti.config.json"),
            json!({"settings":{"verify_command_argv":argv}}).to_string(),
        )
        .unwrap();
        kineti_cli::store::write(&p.0.join(".kineti/verify-trust.json"),
            &json!({"argv":argv,"sha256":kineti_cli::compat::digest(serde_json::to_vec(&argv).unwrap()),"fixture_only":true})).unwrap();
        assert_eq!(
            p.run(&["verify"]).status.success(),
            success,
            "verify {name}"
        );
    }
    let p = Project::new();
    let argv = [
        "/bin/sh",
        "-c",
        "printf x >> .kineti/effect; dd if=/dev/zero bs=8000001 count=1 2>/dev/null",
    ];
    let mut args = vec!["undo", "push", "--label", "complete-truncated", "--"];
    args.extend(argv);
    assert!(p.run(&args).status.success());
    assert!(p.run(&["undo", "rollback", "--yes"]).status.success());
    assert!(p.run(&["undo", "rollback", "--yes"]).status.success());
    assert_eq!(fs::read(p.0.join(".kineti/effect")).unwrap(), b"x");
    let records = kineti_cli::store::lines(&p.0.join(".kineti/undo.native.jsonl")).unwrap();
    assert!(records
        .iter()
        .any(|r| r["kind"] == "completion" && r["exit_code"] == 0));
}

#[test]
fn undo_pipe_holder_fixture() {
    use std::os::fd::AsRawFd;
    let root = std::env::current_dir().unwrap();
    if !root.join(".kineti/undo-holder-fixture").exists() {
        return;
    }
    let control = fs::File::open(root.join(".kineti/undo-release")).unwrap();
    use std::io::Write;
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(".kineti/undo-effect"))
        .unwrap()
        .write_all(b"x")
        .unwrap();
    let mut handshake = [0; 2];
    assert_eq!(unsafe { libc::pipe(handshake.as_mut_ptr()) }, 0);
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        unsafe {
            libc::close(handshake[0]);
            libc::write(handshake[1], b"R".as_ptr().cast(), 1);
            libc::close(handshake[1]);
            let mut poll = libc::pollfd {
                fd: control.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // Independent holder lifetime, even if its supervisor is defective.
            libc::poll(&mut poll, 1, 70_000);
            libc::_exit(0);
        }
    }
    let mut ready = [0u8];
    unsafe {
        libc::close(handshake[1]);
        assert_eq!(libc::read(handshake[0], ready.as_mut_ptr().cast(), 1), 1);
        libc::close(handshake[0]);
    }
    assert_eq!(&ready, b"R");
    std::process::exit(0);
}

#[test]
fn incomplete_capture_keeps_undo_intent_and_blocks_repeated_effect() {
    use std::{
        io::Write,
        os::unix::{ffi::OsStrExt, process::CommandExt},
        process::Stdio,
        time::{Duration, Instant},
    };
    let p = Project::new();
    fs::write(p.0.join(".kineti/undo-holder-fixture"), b"fixture only").unwrap();
    let fifo =
        std::ffi::CString::new(p.0.join(".kineti/undo-release").as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let mut control = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(p.0.join(".kineti/undo-release"))
        .unwrap();
    let test = std::env::current_exe().unwrap();
    assert!(p
        .run(&[
            "undo",
            "push",
            "--label",
            "incomplete-capture",
            "--",
            test.to_str().unwrap(),
            "--exact",
            "undo_pipe_holder_fixture",
            "--nocapture"
        ])
        .status
        .success());
    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kineti"))
        .args(["undo", "rollback", "--yes"])
        .current_dir(&p.0)
        .env("HOME", p.0.join("home"))
        .env("CODEX_HOME", p.0.join(".kineti/codex-home"))
        .process_group(0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let watchdog = start + Duration::from_secs(65);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= watchdog {
            // Child has not been reaped: the CLI group number is still owned.
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = control.write_all(b"Q");
            let _ = child.wait();
            panic!("independent undo watchdog expired");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let elapsed = start.elapsed();
    let _ = control.write_all(b"Q");
    assert_eq!(output.status.code(), Some(3), "{}", value(&output));
    assert!(elapsed >= Duration::from_secs(60), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(63), "{elapsed:?}");
    let before = fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap();
    let records = kineti_cli::store::lines(&p.0.join(".kineti/undo.native.jsonl")).unwrap();
    assert!(records.iter().any(|r| r["kind"] == "intent"));
    assert!(!records.iter().any(|r| r["kind"] == "completion"));
    assert_eq!(p.run(&["undo", "rollback", "--yes"]).status.code(), Some(3));
    assert_eq!(
        fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap(),
        before
    );
    assert_eq!(fs::read(p.0.join(".kineti/undo-effect")).unwrap(), b"x");
}

#[test]
fn signal_undo_preserves_intent_and_blocks_replay() {
    let p = Project::new();
    assert!(p
        .run(&[
            "undo",
            "push",
            "--label",
            "signal",
            "--",
            "/bin/sh",
            "-c",
            "printf x >> .kineti/effect; kill -TERM $$"
        ])
        .status
        .success());
    assert_eq!(p.run(&["undo", "rollback", "--yes"]).status.code(), Some(3));
    let before = fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap();
    let records = kineti_cli::store::lines(&p.0.join(".kineti/undo.native.jsonl")).unwrap();
    assert!(records.iter().any(|r| r["kind"] == "intent"));
    assert!(!records.iter().any(|r| r["kind"] == "completion"));
    assert_eq!(p.run(&["undo", "rollback", "--yes"]).status.code(), Some(3));
    assert_eq!(
        fs::read(p.0.join(".kineti/undo.native.jsonl")).unwrap(),
        before
    );
    assert_eq!(fs::read(p.0.join(".kineti/effect")).unwrap(), b"x");
}

#[test]
fn incomplete_capture_cannot_record_passed_receipt() {
    let p = Project::new();
    let output = p.run(&[
        "test",
        "run",
        "--label",
        "deadline",
        "--timeout",
        "1",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf '1 pass\\n0 fail\\n'; sleep 3 &",
    ]);
    assert!(!output.status.success(), "{}", value(&output));
    let records = kineti_cli::store::lines(&p.0.join(".kineti/evidence.jsonl")).unwrap();
    if let Some(receipt) = records.last() {
        assert_eq!(receipt["outcome"], "failed");
        assert_eq!(receipt["exit_code"], Value::Null);
    } else {
        assert_eq!(output.status.code(), Some(3), "{}", value(&output));
    }
    assert!(!p
        .run(&["test", "check", "--label", "deadline"])
        .status
        .success());
}

#[test]
fn owner_host_control_never_grants_task_authority_or_uses_process_fallback() {
    let p = Project::new();
    let out = p.run(&["host", "status"]);
    assert!(out.status.success());
    let status = value(&out)["result"].clone();
    assert_eq!(status["enforced"], false);
    assert_eq!(status["task_authority"], false);
    assert_eq!(
        kineti_cli::mcp::call(&p.0, "kineti_host_status", &json!({})).unwrap(),
        status
    );
    assert!(kineti_cli::mcp::call(&p.0, "kineti_host_revoke", &json!({})).is_err());
    for args in [
        vec![
            "host",
            "revoke",
            "--cgroup",
            "/sys/fs/cgroup",
            "--task-uid",
            "2501",
        ],
        vec![
            "host",
            "revoke",
            "--cgroup",
            "/sys/fs/cgroup/kineti-task-x",
            "--task-uid",
            "0",
        ],
        vec![
            "host",
            "revoke",
            "--cgroup",
            "/sys/fs/cgroup/kineti-task-x",
            "--task-uid",
            "2501",
            "--wait-ms",
            "0",
        ],
    ] {
        assert_eq!(p.run(&args).status.code(), Some(2));
    }
    assert_eq!(
        p.run(&[
            "host",
            "inspect",
            "--cgroup",
            "/sys/fs/cgroup/kineti-task-x",
            "--task-uid",
            "2501"
        ])
        .status
        .code(),
        Some(3)
    );
}
#[test]
fn task_catalog_cannot_escape_binding_or_reach_legacy_executor() {
    let p = Project::new();
    let names = kineti_cli::mcp::task_tools();
    assert_eq!(names.as_array().unwrap().len(), 5);
    for name in [
        "kineti_evidence_record",
        "kineti_saga_rollback",
        "kineti_task_revoke",
        "kineti_task_approve",
    ] {
        assert!(kineti_cli::mcp::task_call(
            &p.0,
            Path::new("missing.sock"),
            "fixture",
            "session",
            name,
            &json!({})
        )
        .is_err());
    }
    assert!(kineti_cli::mcp::task_call(
        &p.0,
        Path::new("missing.sock"),
        "fixture",
        "session",
        "kineti_task_run",
        &json!({"command":["/bin/sh"]})
    )
    .is_err());
}
#[test]
fn assessment_preserves_unknowns_and_blocks_uncertain_reservations() {
    // Historical records predate initial accounting anchors. Editing anchored
    // history after initialization must remain an integrity failure.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.kineti/native-gate-tests")
        .join(kineti_cli::store::nonce().unwrap());
    fs::create_dir_all(path.join(".kineti")).unwrap();
    let p = Project(path.canonicalize().unwrap());
    let journal = p.0.join(".kineti/journal.jsonl");
    let data = json!({"costs_usd":null});
    let at = "2026-10-31T00:00:00Z";
    let hash = kineti_cli::compat::digest(format!("GENESIS{at}rr-historical{}", data));
    let bytes = json!({"type":"run-record","id":"rr-historical","at":at,"project":"fixture","data":data,"prev_hash":"GENESIS","hash":hash}).to_string()+"\n";
    fs::write(&journal, &bytes).unwrap();
    assert!(p
        .run(&["init", "--goal", "Historical fixture only", "--apply"])
        .status
        .success());
    assert_eq!(p.run(&["spend", "check"]).status.code(), Some(3));
    let assessment = p.run(&["spend", "assessment"]);
    assert!(assessment.status.success());
    assert_eq!(
        value(&assessment)["result"]["historical_valuation"]["status"],
        "unknown"
    );
    assert_eq!(
        value(&assessment)["result"]["accounting_allows_zero_incremental_local"],
        true
    );
    assert_eq!(value(&assessment)["result"]["task_authority"], false);
    assert_eq!(fs::read_to_string(&journal).unwrap(), bytes);
    fs::write(&journal,json!({"type":"run-record","id":"rr-pending","data":{"costs_usd":null,"pending_charge":true}}).to_string()+"\n").unwrap();
    assert_eq!(
        value(&p.run(&["spend", "assessment"]))["result"]
            ["accounting_allows_zero_incremental_local"],
        false
    );
}
#[test]
fn snapshot_preserves_dirty_untracked_modes_and_conflicting_recovery() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let p = Project::new();
    fs::write(p.0.join("dirty.txt"), "dirty bytes\n").unwrap();
    fs::write(p.0.join("untracked.txt"), "untracked bytes\n").unwrap();
    fs::set_permissions(p.0.join("dirty.txt"), fs::Permissions::from_mode(0o750)).unwrap();
    std::os::unix::fs::symlink("dirty.txt", p.0.join("link")).unwrap();
    fs::write(p.0.join(".env.private"), "exclude me").unwrap();
    let work = p.0.join(".kineti/work");
    let blobs = p.0.join(".kineti/blobs");
    let uid = fs::metadata(&p.0).unwrap().uid();
    let gid = fs::metadata(&p.0).unwrap().gid();
    let baseline =
        kineti_cli::task_snapshot::prepare(&p.0, &work, &blobs, &[], 1000000, uid, gid).unwrap();
    assert_eq!(fs::read(work.join("dirty.txt")).unwrap(), b"dirty bytes\n");
    assert_eq!(
        fs::metadata(work.join("dirty.txt"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o750
    );
    assert_eq!(
        fs::read_link(work.join("link")).unwrap(),
        Path::new("dirty.txt")
    );
    assert!(!work.join(".env.private").exists());
    fs::write(work.join("dirty.txt"), "task output").unwrap();
    let expected = kineti_cli::task_snapshot::inventory(&work, &[], 1000000).unwrap();
    fs::write(work.join("untracked.txt"), "later user work").unwrap();
    assert!(kineti_cli::task_snapshot::restore(
        &work,
        &blobs,
        &baseline,
        &expected,
        &[],
        1000000,
        &p.0.join(".kineti/quarantine"),
        uid,
        gid
    )
    .is_err());
    assert_eq!(
        fs::read(work.join("untracked.txt")).unwrap(),
        b"later user work"
    );
    assert_eq!(fs::read(p.0.join("dirty.txt")).unwrap(), b"dirty bytes\n");
}
#[test]
fn task_request_rejects_paid_network_identity_and_unknown_scope() {
    let p = Project::new();
    let request = json!({"schema":"kineti-task-request-v1","id":"fixture","session":"session","project_root":p.0,"project_identity":kineti_cli::task_grant::project_identity(&p.0).unwrap(),"purpose":"fixture","task_uid":2501,"task_user":"kineti-test-one","host":{"path":"/var/lib/codex","sha256":"a".repeat(64),"version":"0.166.0"},"checks":[{"label":"test","kind":"test","argv":["/bin/sh","-c","printf '1 pass'"],"inputs":["."]}],"repair_argv":["/bin/true"],"exclusions":[],"runtime_inputs":["/bin/sh"],"limits":{"expires_at":2000000000,"wall_seconds":120,"command_seconds":20,"repair_attempts":1,"verification_attempts":3,"snapshot_bytes":1000000,"concurrency":1,"new_paid_usd_micros":0,"memory_bytes":268435456,"pids":64},"network_destinations":[],"credential_handles":[],"resource_class":"owned-local","automatic_rollback":false});
    assert!(kineti_cli::task_grant::Grant::parse(request.clone()).is_ok());
    let mut model = request.clone();
    model["resource_class"] = json!("included-subscription");
    model["repair_argv"] = json!([]);
    model["model_repair"] = json!({"model":"gpt-6.1-sol","prompt":"Repair only the approved fixture","max_turns":2,"provider":"codex-chatgpt-subscription","data_scope":"approved-task-workspace","helper_sha256":"a".repeat(64)});
    assert!(kineti_cli::task_grant::Grant::parse(model.clone()).is_ok());
    for (pointer, bad) in [
        ("/model_repair/max_turns", json!(0)),
        ("/model_repair/helper_sha256", json!("invalid")),
        ("/model_repair/model", json!("fallback/any")),
        ("/model_repair/prompt", json!("")),
        ("/repair_argv", json!(["/bin/true"])),
        ("/resource_class", json!("owned-local")),
    ] {
        let mut changed = model.clone();
        *changed.pointer_mut(pointer).unwrap() = bad;
        assert!(
            kineti_cli::task_grant::Grant::parse(changed).is_err(),
            "{pointer}"
        );
    }
    model["model_repair"]["token"] = json!("fixture-secret");
    assert!(kineti_cli::task_grant::Grant::parse(model).is_err());

    for (pointer, bad) in [
        ("/limits/new_paid_usd_micros", json!(1)),
        ("/network_destinations", json!(["https://example.com"])),
        ("/task_uid", json!(0)),
        ("/repair_argv", json!(["git", "push"])),
        ("/credential_handles", json!(["secret"])),
        ("/limits/concurrency", json!(2)),
        ("/project_identity", json!("forged")),
    ] {
        let mut v = request.clone();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(
            kineti_cli::task_grant::Grant::parse(v).is_err(),
            "{pointer}"
        );
    }
    let mut v = request;
    v["approved"] = json!(true);
    assert!(kineti_cli::task_grant::Grant::parse(v).is_err());
}

#[test]
fn cancelled_restoration_preserves_displaced_and_remaining_work() {
    use std::os::unix::fs::MetadataExt;
    let p = Project::new();
    fs::write(p.0.join("a.txt"), "original a").unwrap();
    fs::write(p.0.join("b.txt"), "original b").unwrap();
    let work = p.0.join(".kineti/work");
    let blobs = p.0.join(".kineti/blobs");
    let m = fs::metadata(&p.0).unwrap();
    let baseline =
        kineti_cli::task_snapshot::prepare(&p.0, &work, &blobs, &[], 1000000, m.uid(), m.gid())
            .unwrap();
    fs::write(work.join("a.txt"), "task a").unwrap();
    fs::write(work.join("b.txt"), "task b").unwrap();
    let expected = kineti_cli::task_snapshot::inventory(&work, &[], 1000000).unwrap();
    let quarantine = p.0.join(".kineti/quarantine");
    let mut checkpoints = 0;
    let result = kineti_cli::task_snapshot::restore_checked(
        &work,
        &blobs,
        &baseline,
        &expected,
        &[],
        1000000,
        &quarantine,
        m.uid(),
        m.gid(),
        || {
            checkpoints += 1;
            if checkpoints == 4 {
                return kineti_cli::store::fail(
                    3,
                    "inspect-uncertain: injected owner cancellation",
                );
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(checkpoints, 4);
    assert_eq!(fs::read(work.join("b.txt")).unwrap(), b"task b");
    let displaced = fs::read_dir(&quarantine)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(displaced.len(), 1);
    assert_eq!(fs::read(displaced[0].path()).unwrap(), b"task a");
    assert_eq!(fs::read(p.0.join("a.txt")).unwrap(), b"original a");
}

#[test]
fn cancellation_during_restore_copy_preserves_full_task_output() {
    use std::os::unix::fs::MetadataExt;
    let p = Project::new();
    let original = vec![b'o'; 200000];
    let output = vec![b't'; 200000];
    fs::write(p.0.join("large.txt"), &original).unwrap();
    let work = p.0.join(".kineti/work");
    let blobs = p.0.join(".kineti/blobs");
    let m = fs::metadata(&p.0).unwrap();
    let baseline =
        kineti_cli::task_snapshot::prepare(&p.0, &work, &blobs, &[], 1000000, m.uid(), m.gid())
            .unwrap();
    fs::write(work.join("large.txt"), &output).unwrap();
    let expected = kineti_cli::task_snapshot::inventory(&work, &[], 1000000).unwrap();
    let quarantine = p.0.join(".kineti/quarantine");
    let result = kineti_cli::task_snapshot::restore_checked(
        &work,
        &blobs,
        &baseline,
        &expected,
        &[],
        1000000,
        &quarantine,
        m.uid(),
        m.gid(),
        || {
            if fs::metadata(work.join("large.txt")).is_ok_and(|m| m.len() == 65536) {
                return kineti_cli::store::fail(3, "inspect-uncertain: cancellation during copy");
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(fs::read(work.join("large.txt")).unwrap(), original[..65536]);
    let displaced = fs::read_dir(&quarantine)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // Project::new also contributes test.txt; both displaced files survive.
    assert_eq!(displaced.len(), 2);
    let retained = displaced
        .iter()
        .map(|p| fs::read(p.path()).unwrap())
        .collect::<Vec<_>>();
    assert!(retained.contains(&output));
    assert!(retained.contains(&b"declared fixture".to_vec()));
    assert_eq!(fs::read(p.0.join("large.txt")).unwrap(), original);
}

#[test]
fn cancelled_snapshot_setup_preserves_source_and_partial_checkpoint() {
    use std::os::unix::fs::MetadataExt;
    for cancel_blob in [true, false] {
        let p = Project::new();
        let original = vec![b's'; 200000];
        fs::write(p.0.join("large.txt"), &original).unwrap();
        let work = p.0.join(".kineti/work");
        let blobs = p.0.join(".kineti/blobs");
        let m = fs::metadata(&p.0).unwrap();
        let result = kineti_cli::task_snapshot::prepare_checked(
            &p.0,
            &work,
            &blobs,
            &[],
            1000000,
            m.uid(),
            m.gid(),
            || {
                let partial = if cancel_blob {
                    blobs.is_dir()
                        && fs::read_dir(&blobs)
                            .unwrap()
                            .any(|e| e.unwrap().metadata().unwrap().len() == 65536)
                } else {
                    fs::metadata(work.join("large.txt")).is_ok_and(|m| m.len() == 65536)
                };
                if partial {
                    return kineti_cli::store::fail(
                        3,
                        "inspect-uncertain: injected setup cancellation",
                    );
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read(p.0.join("large.txt")).unwrap(), original);
        let retained = fs::read_dir(&blobs)
            .unwrap()
            .map(|e| fs::read(e.unwrap().path()).unwrap())
            .collect::<Vec<_>>();
        assert!(retained.contains(&if cancel_blob {
            original[..65536].to_vec()
        } else {
            original.clone()
        }));
        if !cancel_blob {
            assert_eq!(fs::read(work.join("large.txt")).unwrap(), original[..65536]);
        }
        // A partial projection cannot silently become a fresh completed setup.
        assert!(kineti_cli::task_snapshot::prepare(
            &p.0,
            &work,
            &blobs,
            &[],
            1000000,
            m.uid(),
            m.gid()
        )
        .is_err());
    }
}
