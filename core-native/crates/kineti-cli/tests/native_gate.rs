//! End-to-end native behavior tests use disposable project-local directories.
use serde_json::{json, Value};
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
    assert_eq!(rows[1]["result"]["tools"].as_array().unwrap().len(), 8);
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
