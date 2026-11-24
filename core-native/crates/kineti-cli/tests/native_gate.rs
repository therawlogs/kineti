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
    // Fail-closed CI when no command mapping is declared (PA02)
    assert_eq!(
        p.run(&["ci", "--require-evidence", "release"])
            .status
            .code(),
        Some(1)
    );
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["release"],
            "commands": {"release": ["/bin/sh", "-c", "printf '1 pass\\n0 fail\\n'"]},
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();
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
    let tools = rows[1]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 6);
    assert!(!tools
        .iter()
        .any(|t| t["name"] == "kineti_saga_push" || t["name"] == "kineti_saga_rollback"));
    let ev = tools
        .iter()
        .find(|t| t["name"] == "kineti_evidence_record")
        .unwrap();
    assert_eq!(ev["annotations"]["destructiveHint"], true);
    assert_eq!(ev["annotations"]["openWorldHint"], true);
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
    let at = "2026-10-25T00:00:00Z";
    let id = "rr-unresolved";
    let data = json!({"costs_usd": null});
    let hash = kineti_cli::compat::digest(format!("GENESIS{at}{id}{data}"));
    fs::write(
        p.0.join(".kineti/journal.jsonl"),
        json!({
            "type": "run-record",
            "id": id,
            "at": at,
            "project": "fixture",
            "data": data,
            "prev_hash": "GENESIS",
            "hash": hash
        })
        .to_string()
            + "\n",
    )
    .unwrap();
    let out = p.run(&["spend", "check"]);
    assert_eq!(out.status.code(), Some(3));
    let err = value(&out)["error"].as_str().unwrap().to_owned();
    assert!(err.contains("rr-unresolved"), "unexpected error: {err}");
    assert!(
        err.contains("unresolved historical cost"),
        "must reach spend unresolved-cost check, not journal integrity failure: {err}"
    );

    // Malformed journal row must still fail closed on integrity check
    let bad = Project::new();
    fs::write(
        bad.0.join(".kineti/journal.jsonl"),
        json!({"type":"run-record","id":"rr-malformed","data":{"costs_usd":null}}).to_string()
            + "\n",
    )
    .unwrap();
    let bad_out = bad.run(&["spend", "check"]);
    assert_eq!(bad_out.status.code(), Some(3));
    assert!(value(&bad_out)["error"]
        .as_str()
        .unwrap()
        .contains("accounting journal integrity failed at rr-malformed"));
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
fn pa01_mcp_allowlist_saga_removal_and_jsonrpc_hardening() {
    use std::io::Write;
    let p = Project::new();
    let allowlisted = json!(["/bin/sh", "-c", "printf '1 pass\\n0 fail\\n'"]);
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["unit"],
            "commands": {"unit": allowlisted},
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();
    let marker = p.0.join("rce-marker.txt");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kineti"))
        .arg("mcp")
        .current_dir(&p.0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    // 1. Initialize without protocolVersion -> must default to non-null string
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}})
    )
    .unwrap();
    // 2. JSON-RPC batch array and non-object -> must return -32600 (not hang/drop)
    writeln!(input, "[1,2,3]").unwrap();
    writeln!(input, "\"hello\"").unwrap();
    // 3. Unallowlisted RCE command -> must be rejected with zero side effects
    writeln!(
        input,
        "{}",
        json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params":{"name":"kineti_evidence_record","arguments":{
                "label":"unit",
                "command":["/bin/sh","-c",format!("touch {}", marker.display())],
                "required_tests":["test.txt"]
            }}
        })
    )
    .unwrap();
    // 4. Removed saga MCP tools -> must be rejected
    writeln!(
        input,
        "{}",
        json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params":{"name":"kineti_saga_push","arguments":{
                "label":"evil","command":["/bin/sh","-c",format!("touch {}", marker.display())]
            }}
        })
    )
    .unwrap();
    writeln!(
        input,
        "{}",
        json!({
            "jsonrpc":"2.0","id":4,"method":"tools/call",
            "params":{"name":"kineti_saga_rollback","arguments":{"confirm":true}}
        })
    )
    .unwrap();
    // 5. Exact allowlisted command -> must succeed
    writeln!(
        input,
        "{}",
        json!({
            "jsonrpc":"2.0","id":5,"method":"tools/call",
            "params":{"name":"kineti_evidence_record","arguments":{
                "label":"unit","command":allowlisted,"required_tests":["test.txt"],"timeout":120
            }}
        })
    )
    .unwrap();
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let rows = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 7, "rows={rows:?}");
    assert_eq!(rows[0]["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(rows[1]["error"]["code"], -32600);
    assert_eq!(rows[2]["error"]["code"], -32600);
    assert_eq!(rows[3]["result"]["isError"], true);
    assert_eq!(rows[4]["result"]["isError"], true);
    assert_eq!(rows[5]["result"]["isError"], true);
    assert_eq!(rows[6]["result"]["isError"], false);
    assert!(!marker.exists(), "RCE marker must never be created");
}

#[test]
fn pa02_codex_multi_project_and_secret_backup_isolation() {
    let p1 = Project::new();
    let p2 = Project::new();
    let shared_home = p1.0.join("shared-codex-home");
    fs::create_dir_all(&shared_home).unwrap();
    let secret = "SUPER_SECRET_API_TOKEN_98765";
    fs::write(
        shared_home.join("config.toml"),
        format!("[mcp_servers.other]\nenv = {{ API_KEY = \"{secret}\" }}\n"),
    )
    .unwrap();
    let h = shared_home.to_str().unwrap();
    assert!(p1
        .run(&["init", "--codex-home", h, "--apply"])
        .status
        .success());
    // Project #2 must succeed against the same Codex home without collision
    let o2 = p2.run(&["init", "--codex-home", h, "--apply"]);
    assert!(o2.status.success(), "{}", value(&o2));
    // Secrets from global config.toml must NEVER appear inside project backups or install-audit.jsonl
    for proj in [&p1, &p2] {
        let audit = fs::read_to_string(proj.0.join(".kineti/install-audit.jsonl")).unwrap();
        assert!(!audit.contains(secret), "secret leaked into install-audit");
        let proj_backups = proj.0.join(".kineti/backups");
        if proj_backups.exists() {
            for entry in fs::read_dir(proj_backups).unwrap() {
                let text = fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
                assert!(!text.contains(secret), "secret leaked into project backups");
            }
        }
    }
    // Backup must exist in <codex-home>/.kineti-backups instead
    assert!(shared_home.join(".kineti-backups").is_dir());
}

#[test]
fn pa03_mirror_json_ceiling_override_is_ignored() {
    let p = Project::new();
    fs::write(
        p.0.join(".kineti/mirror.json"),
        json!({"ceiling": 9_000_000_000u64}).to_string(),
    )
    .unwrap();
    let status = p.run(&["spend", "status"]);
    assert!(status.status.success(), "{}", value(&status));
    assert_eq!(value(&status)["result"]["global_microcents"], 50_000_000);
}

#[test]
fn pa04_relative_executable_resolution_and_symlinked_roots() {
    use std::{
        io::Write,
        os::unix::fs::{symlink, PermissionsExt},
    };
    let p = Project::new();
    let caller = Project::new();
    fs::create_dir_all(p.0.join("scripts")).unwrap();
    fs::create_dir_all(caller.0.join("scripts")).unwrap();
    let real_script = p.0.join("scripts/test.sh");
    let decoy_script = caller.0.join("scripts/test.sh");
    fs::write(&real_script, "#!/bin/sh\nprintf '1 pass\\n0 fail\\n'\n").unwrap();
    fs::write(&decoy_script, "#!/bin/sh\necho DECOY\n").unwrap();
    fs::set_permissions(&real_script, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&decoy_script, fs::Permissions::from_mode(0o755)).unwrap();
    let real_sha = kineti_cli::store::hash_file_nofollow(&real_script).unwrap();

    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["unit"],
            "commands": {"unit": ["./scripts/test.sh"]},
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();

    // Symlinked root passed via --root must work AND hash <root>/scripts/test.sh (not caller CWD!)
    let root_link = caller.0.join("linked-project");
    symlink(&p.0, &root_link).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kineti"))
        .args(["mcp", "--root", root_link.to_str().unwrap()])
        .current_dir(&caller.0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}})
    )
    .unwrap();
    writeln!(
        input,
        "{}",
        json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params":{"name":"kineti_evidence_record","arguments":{
                "label":"unit","command":["./scripts/test.sh"],"required_tests":["test.txt"]
            }}
        })
    )
    .unwrap();
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let rows = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows[1]["result"]["isError"], false, "rows={rows:?}");
    let rec: Value =
        serde_json::from_str(rows[1]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(rec["manifest"]["runtime"]["executable_sha256"], real_sha);

    // Symlinked --codex-home directory must also succeed
    let real_codex = p.0.join("real-codex");
    fs::create_dir_all(&real_codex).unwrap();
    let link_codex = p.0.join("link-codex");
    symlink(&real_codex, &link_codex).unwrap();
    let init_link = p.run(&[
        "init",
        "--codex-home",
        link_codex.to_str().unwrap(),
        "--apply",
    ]);
    assert!(init_link.status.success(), "{}", value(&init_link));

    // Symlinked project-internal leaf file (.kineti/state.json) MUST still be refused (exit 3)
    let bad = Project::new();
    let outside = bad.0.join("outside.json");
    fs::write(&outside, "{}").unwrap();
    fs::remove_file(bad.0.join(".kineti/state.json")).unwrap();
    symlink(&outside, bad.0.join(".kineti/state.json")).unwrap();
    assert_eq!(
        bad.run(&["init", "--goal", "Fixture only", "--apply"])
            .status
            .code(),
        Some(3)
    );
}

#[test]
fn pa05_two_tier_freshness_legacy_drift_and_constant_size_receipts() {
    let p = Project::new();
    fs::create_dir_all(p.0.join("src/legacy")).unwrap();
    fs::create_dir_all(p.0.join("src/build")).unwrap();
    fs::write(
        p.0.join("src/legacy/auth.rs"),
        "pub fn ok() -> bool { false }\n",
    )
    .unwrap();
    fs::write(p.0.join("src/build/x.rs"), "pub const V: u32 = 1;\n").unwrap();

    // Creating .pytest_cache during test execution must NOT trigger mutated_during_run
    let run = p.run(&[
        "test",
        "run",
        "--label",
        "release",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "mkdir -p .pytest_cache && printf x > .pytest_cache/tag && printf '1 pass\\n0 fail\\n'",
    ]);
    assert!(run.status.success(), "{}", value(&run));
    let rec = &value(&run)["result"];
    assert!(rec["manifest"]["inventory_sha256"].is_string());
    assert!(rec["manifest"].get("inputs").is_none());

    // 1-byte drift inside src/legacy/ MUST trigger exit 4 (stale)
    fs::write(
        p.0.join("src/legacy/auth.rs"),
        "pub fn ok() -> bool { true }\n",
    )
    .unwrap();
    assert_eq!(p.check().status.code(), Some(4));

    // Re-record, then add nested src/target/backdoor.rs -> MUST trigger exit 4
    assert!(p.record().status.success());
    fs::create_dir_all(p.0.join("src/target")).unwrap();
    fs::write(p.0.join("src/target/backdoor.rs"), "// stealth").unwrap();
    assert_eq!(p.check().status.code(), Some(4));
}

#[test]
fn pa06_two_tier_parsers_multi_suite_bun_cargo_ignore_and_exit_code() {
    let p = Project::new();
    // Multi-suite bun output where suite 2 fails MUST exit 1 (never false pass!)
    let bun_fail = p.run(&[
        "test",
        "run",
        "--label",
        "bun",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf '2 pass\\n0 fail\\n1 pass\\n1 fail\\n'",
    ]);
    assert_eq!(bun_fail.status.code(), Some(1), "{}", value(&bun_fail));

    // Cargo test with 1 ignored fails by default, passes with --allow-ignored
    let cargo_ign = p.run(&[
        "test",
        "run",
        "--label",
        "cargo",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf 'test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out\\n'",
    ]);
    assert_eq!(cargo_ign.status.code(), Some(1));
    let cargo_ok = p.run(&[
        "test",
        "run",
        "--label",
        "cargo",
        "--allow-ignored",
        "--required-test",
        "test.txt",
        "--",
        "/bin/sh",
        "-c",
        "printf 'test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out\\n'",
    ]);
    assert!(cargo_ok.status.success(), "{}", value(&cargo_ok));

    // min_passed floor enforcement in gate.json
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["cargo"],
            "commands": {"cargo": ["/bin/sh", "-c", "printf 'test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out\\n'"]},
            "min_passed": {"cargo": 10},
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();
    assert_eq!(
        p.run(&["ci", "--require-evidence", "cargo"]).status.code(),
        Some(1)
    );

    // Polyglot --parser exit-code succeeds on pytest-style output and records count_observed: false
    let py_proj = Project::new();
    let py_cmd = ["/bin/sh", "-c", "printf '3 passed in 0.02s\\n'"];
    fs::write(
        py_proj.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["py"],
            "commands": {"py": py_cmd},
            "parsers": {"py": "exit-code"},
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();
    let py = py_proj.run(&[
        "test",
        "run",
        "--label",
        "py",
        "--parser",
        "exit-code",
        "--required-test",
        "test.txt",
        "--",
        py_cmd[0],
        py_cmd[1],
        py_cmd[2],
    ]);
    assert!(py.status.success(), "{}", value(&py));
    assert_eq!(
        value(&py)["result"]["verification"]["parser"],
        "kineti-exit-code-v1"
    );
    assert_eq!(
        value(&py)["result"]["verification"]["count_observed"],
        false
    );
    let chk = py_proj.run(&["test", "check", "--label", "py"]);
    assert!(chk.status.success(), "{}", value(&chk));
    let ci_ok = py_proj.run(&["ci", "--require-evidence", "py"]);
    assert!(ci_ok.status.success(), "{}", value(&ci_ok));
}

#[test]
fn pa07_pa08_pa09_pr_check_cli_end_to_end_and_audit_verification() {
    let p = Project::new();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&p.0)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    fs::create_dir_all(p.0.join("tests")).unwrap();
    fs::write(
        p.0.join("tests/test_auth.py"),
        "def test_ok():\n    assert True\n\ndef test_two():\n    assert 1 + 1 == 2\n",
    )
    .unwrap();
    let py_cmd = json!(["/bin/sh", "-c", "printf '2 passed in 0.01s\\n'"]);
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({
            "version": 1,
            "required_checks": ["unit"],
            "commands": {"unit": py_cmd},
            "required_tests": ["tests/test_auth.py"],
            "max_age_minutes": 240
        })
        .to_string(),
    )
    .unwrap();
    git(&[
        "add",
        "-f",
        ".kineti/gate.json",
        "tests/test_auth.py",
        "test.txt",
    ]);
    git(&["commit", "-m", "base"]);

    // Tamper PR: weaken gate.json AND insert @pytest.mark.skip
    git(&["checkout", "-b", "ai-tamper"]);
    fs::write(
        p.0.join(".kineti/gate.json"),
        json!({"version":1,"required_checks":[],"commands":{}}).to_string(),
    )
    .unwrap();
    fs::write(
        p.0.join("tests/test_auth.py"),
        "import pytest\n\n@pytest.mark.skip(reason='flaky')\ndef test_ok():\n    assert True\n",
    )
    .unwrap();
    git(&["add", "-f", ".kineti/gate.json", "tests/test_auth.py"]);
    git(&["commit", "-m", "ai tamper"]);

    let blocked = p.run(&[
        "pr-check",
        "--base-ref",
        "main",
        "--head-ref",
        "HEAD",
        "--audit-out",
        ".kineti/pr-audit.json",
        "--summary-out",
        ".kineti/pr-summary.md",
    ]);
    assert_eq!(blocked.status.code(), Some(1), "{}", value(&blocked));
    let audit: Value =
        serde_json::from_str(&fs::read_to_string(p.0.join(".kineti/pr-audit.json")).unwrap())
            .unwrap();
    assert_eq!(audit["schema"], "kineti-pr-audit-v1");
    assert_eq!(audit["verdict"], "blocked");
    assert_eq!(audit["policy_tamper_detected"], true);
    assert!(!audit["test_tamper_findings"].as_array().unwrap().is_empty());
    let pub_pem = fs::read_to_string(p.0.join(".kineti/evidence-issuer.pem")).unwrap();
    assert!(kineti_cli::pr_guard::verify_audit(&audit, &pub_pem));
    let mut tampered = audit.clone();
    tampered["verdict"] = json!("passed");
    assert!(!kineti_cli::pr_guard::verify_audit(&tampered, &pub_pem));
    assert!(fs::read_to_string(p.0.join(".kineti/pr-summary.md"))
        .unwrap()
        .contains("BLOCKED"));

    // Clean PR with linked passing evidence -> passes exit 0
    git(&["checkout", "main"]);
    git(&["checkout", "-b", "clean-pr"]);
    fs::write(
        p.0.join("tests/test_auth.py"),
        "def test_ok():\n    assert True\n\ndef test_two():\n    assert 1 + 1 == 2\n\ndef test_three():\n    assert 2 + 2 == 4\n",
    )
    .unwrap();
    git(&["add", "tests/test_auth.py"]);
    git(&["commit", "-m", "add third test"]);

    let rec = p.run(&[
        "test",
        "run",
        "--label",
        "unit",
        "--parser",
        "exit-code",
        "--required-test",
        "tests/test_auth.py",
        "--",
        "/bin/sh",
        "-c",
        "printf '2 passed in 0.01s\\n'",
    ]);
    assert!(rec.status.success(), "{}", value(&rec));
    let passed = p.run(&[
        "pr-check",
        "--base-ref",
        "main",
        "--head-ref",
        "HEAD",
        "--require-evidence",
        "unit",
        "--audit-out",
        ".kineti/pr-audit-pass.json",
        "--summary-out",
        ".kineti/pr-summary-pass.md",
    ]);
    assert!(passed.status.success(), "{}", value(&passed));
    assert_eq!(value(&passed)["result"]["verdict"], "passed");
    assert!(fs::read_to_string(p.0.join(".kineti/pr-summary-pass.md"))
        .unwrap()
        .contains("PASSED"));
}

#[test]
fn pa10_subcommand_help_and_status_project_only() {
    let p = Project::new();
    for cmd in ["test", "pr-check", "spend", "ci", "init", "status", "undo"] {
        let out = p.run(&[cmd, "--help"]);
        assert!(out.status.success(), "{cmd}: {}", value(&out));
        assert_eq!(value(&out)["result"]["command"], cmd);
    }
    // Corrupt spend.json -> regular status fails (exit 3), status --project-only stays readable (exit 0)
    fs::write(p.0.join(".kineti/spend.json"), "{corrupt").unwrap();
    assert_eq!(p.run(&["status"]).status.code(), Some(3));
    let po = p.run(&["status", "--project-only"]);
    assert!(po.status.success(), "{}", value(&po));
    assert_eq!(value(&po)["result"]["state"]["root_goal"], "Fixture only");
}

#[test]
fn v1_0_version_and_signature_only_trusted_key_nofollow() {
    let p = Project::new();
    let v = p.run(&["--version"]);
    assert!(v.status.success(), "{}", value(&v));
    assert_eq!(value(&v)["result"]["version"], "1.0.1");

    // Create a receipt and valid issuer key, then replace trusted-key path with a symlink
    fs::create_dir_all(p.0.join("tests")).unwrap();
    fs::write(p.0.join("tests/a.test.js"), "// disposable test\n").unwrap();
    let ok = p.run(&[
        "test",
        "run",
        "--label",
        "unit",
        "--required-test",
        "tests/a.test.js",
        "--parser",
        "exit-code",
        "--",
        "/bin/sh",
        "-c",
        "exit 0",
    ]);
    assert!(ok.status.success(), "{}", value(&ok));
    let receipt_path = p.0.join("single-receipt.json");
    let lines = fs::read_to_string(p.0.join(".kineti/evidence.jsonl")).unwrap();
    fs::write(&receipt_path, lines.lines().last().unwrap()).unwrap();

    let outside_key = p.0.join("real-pub.pem");
    fs::copy(p.0.join(".kineti/evidence-issuer.pem"), &outside_key).unwrap();
    let key_link = p.0.join("link-pub.pem");
    std::os::unix::fs::symlink(&outside_key, &key_link).unwrap();

    let blocked = p.run(&[
        "test",
        "check",
        "--signature-only",
        "--receipt",
        "single-receipt.json",
        "--trusted-key",
        "link-pub.pem",
    ]);
    assert!(!blocked.status.success(), "symlinked trusted key must fail");
    assert_eq!(blocked.status.code(), Some(3));
}
