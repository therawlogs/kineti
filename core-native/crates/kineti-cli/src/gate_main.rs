//! Native gate command entrypoint.
use kineti_cli::{
    gate, host_control, init, mcp, receipt, self_test, spend,
    store::{self, Error, Result},
    task, undo,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};

struct Args {
    values: BTreeMap<String, Vec<String>>,
    position: Vec<String>,
    command: Vec<String>,
    separator: bool,
}
impl Args {
    fn parse(argv: &[String], values: &[&str], flags: &[&str]) -> Result<Self> {
        let mut a = Self {
            values: BTreeMap::new(),
            position: vec![],
            command: vec![],
            separator: false,
        };
        let mut i = 0;
        while i < argv.len() {
            let s = &argv[i];
            if s == "--" {
                a.separator = true;
                a.command = argv[i + 1..].to_vec();
                break;
            }
            if let Some((name, value)) = s.strip_prefix("--").and_then(|_| s.split_once('=')) {
                if !values.contains(&name) || value.is_empty() {
                    return store::fail(2, format!("invalid valued option {name}"));
                }
                a.values.entry(name.into()).or_default().push(value.into());
            } else if values.contains(&s.as_str()) {
                i += 1;
                let v = argv
                    .get(i)
                    .filter(|s| !s.is_empty() && !s.starts_with("--"))
                    .ok_or_else(|| Error(2, format!("missing value for {s}")))?;
                a.values.entry(s.clone()).or_default().push(v.clone());
            } else if flags.contains(&s.as_str()) {
                if a.values.insert(s.clone(), vec!["true".into()]).is_some() {
                    return store::fail(2, format!("duplicate flag {s}"));
                }
            } else if s.starts_with('-') {
                return store::fail(2, format!("unknown option {s}"));
            } else {
                a.position.push(s.clone());
            }
            i += 1;
        }
        Ok(a)
    }
    fn one(&self, k: &str) -> Result<Option<&str>> {
        match self.values.get(k) {
            Some(v) if v.len() == 1 => Ok(Some(&v[0])),
            None => Ok(None),
            _ => store::fail(2, format!("duplicate option {k}")),
        }
    }
    fn text(&self, k: &str, default: &str) -> Result<String> {
        Ok(self.one(k)?.unwrap_or(default).into())
    }
    fn yes(&self, k: &str) -> bool {
        self.values.contains_key(k)
    }
    fn list(&self, k: &str) -> Vec<String> {
        self.values.get(k).cloned().unwrap_or_default()
    }
    fn number(&self, k: &str, default: u64) -> Result<u64> {
        self.one(k)?.map_or(Ok(default), |v| {
            v.parse()
                .map_err(|_| Error(2, format!("invalid integer {k}")))
        })
    }
    fn clean(&self) -> Result<()> {
        if !self.position.is_empty() || self.separator {
            return store::fail(2, "unexpected positional arguments or command separator");
        }
        Ok(())
    }
}
fn help() -> Value {
    json!({"version":env!("CARGO_PKG_VERSION"),"claim":"Record checks, detect changed inputs, and block participating CI commands when evidence or reported spending fails policy.","commands":["init","test run/check","evidence","verify","ci","spend","undo","saga","status","mcp","self-test","host status/inspect/revoke (owner lifecycle primitive)","task plan/inspect/approve/serve/status/run/check/recover/cancel/pause/revoke (experimental Linux owner service)","spend assessment (read-only; no grant or release authority)"],"status":"candidate; publication requires reviewed artifacts and accounting approval"})
}
fn dispatch(root: &Path, args: &[String]) -> Result<Value> {
    let Some(cmd) = args.first().map(String::as_str) else {
        return Ok(help());
    };
    if ["--help", "-h", "help"].contains(&cmd) {
        return Ok(help());
    }
    if ["--version", "-v", "version"].contains(&cmd) {
        return Ok(json!({"version":env!("CARGO_PKG_VERSION")}));
    }
    let rest = &args[1..];
    match cmd {
        "task" => {
            let sub = rest.first().map(String::as_str).unwrap_or("status");
            let a = Args::parse(
                if rest.is_empty() { rest } else { &rest[1..] },
                &[
                    "--owner-store",
                    "--request",
                    "--id",
                    "--session",
                    "--socket",
                    "--read-fd",
                    "--write-fd",
                ],
                &[],
            )?;
            a.clean()?;
            let required = |k| a.one(k)?.ok_or_else(|| Error(2, format!("required {k}")));
            match sub {
                "plan" => task::plan(Path::new(required("--request")?)),
                "inspect" => {
                    task::inspect(Path::new(required("--owner-store")?), required("--id")?)
                }
                "approve" => task::approve(
                    Path::new(required("--owner-store")?),
                    Path::new(required("--request")?),
                ),
                "serve-model" => {
                    task::serve_model(Path::new(required("--owner-store")?), required("--id")?)
                }
                "serve" => task::serve(Path::new(required("--owner-store")?), required("--id")?),
                "status" | "run" | "check" | "recover" | "cancel" | "revoke" | "pause" => {
                    if a.one("--read-fd")?.is_some() || a.one("--write-fd")?.is_some() {
                        if a.one("--socket")?.is_some() {
                            return store::fail(2, "select one task transport");
                        }
                        let read_fd = required("--read-fd")?
                            .parse::<i32>()
                            .map_err(|_| Error(2, "invalid read descriptor".into()))?;
                        let write_fd = required("--write-fd")?
                            .parse::<i32>()
                            .map_err(|_| Error(2, "invalid write descriptor".into()))?;
                        return kineti_cli::task_pipe::Channel::inherited(read_fd, write_fd)?
                            .request(required("--id")?, required("--session")?, sub, root);
                    }
                    task::request(
                        Path::new(required("--socket")?),
                        required("--id")?,
                        required("--session")?,
                        sub,
                        root,
                    )
                }
                _ => store::fail(2, "unknown task operation"),
            }
        }
        "host" => {
            let sub = rest.first().map(String::as_str).unwrap_or("status");
            let args = if rest.is_empty() { rest } else { &rest[1..] };
            if ["guest-worker", "guest-connect"].contains(&sub) {
                let a = Args::parse(args, &[], &[])?;
                a.clean()?;
                return if sub == "guest-worker" {
                    kineti_cli::task_guest::worker_serve()
                } else {
                    kineti_cli::task_guest::worker_connect()
                };
            }
            if sub == "guest-cold" {
                let a = Args::parse(args, &["--phase"], &[])?;
                a.clean()?;
                let phase = a
                    .one("--phase")?
                    .ok_or_else(|| Error(2, "guest-cold requires --phase".into()))?;
                if !["prepare", "shutdown", "export"].contains(&phase) {
                    return store::fail(2, "unknown guest cold-checkpoint phase");
                }
                use std::io::Read;
                let mut wire = vec![];
                std::io::stdin().take(32_000_001).read_to_end(&mut wire)?;
                if wire.len() > 32_000_000 {
                    return store::fail(3, "guest checkpoint input exceeds transfer bound");
                }
                let q = serde_json::from_slice(&wire)
                    .map_err(|_| Error(3, "invalid guest checkpoint envelope".into()))?;
                return kineti_cli::task_guest::cold(phase, q);
            }
            if sub == "guest-preflight" {
                let a = Args::parse(args, &[], &[])?;
                a.clean()?;
                return kineti_cli::task_guest::preflight();
            }
            if sub == "status" {
                let a = Args::parse(args, &[], &[])?;
                a.clean()?;
                return Ok(host_control::status());
            }
            if ["vm-inspect", "vm-revoke", "vm-supervise", "vm-run"].contains(&sub) {
                let a = Args::parse(args, &["--binding"], &[])?;
                a.clean()?;
                let binding = Path::new(
                    a.one("--binding")?
                        .ok_or_else(|| Error(2, "VM lifecycle requires exact --binding".into()))?,
                );
                return match sub {
                    "vm-inspect" => kineti_cli::vm_control::inspect(binding),
                    "vm-revoke" => kineti_cli::vm_control::revoke(binding),
                    "vm-run" => kineti_cli::vm_control::run(binding),
                    _ => kineti_cli::vm_control::supervise(binding),
                };
            }
            if !["inspect", "revoke", "supervise"].contains(&sub) {
                return store::fail(2, "host supports status, inspect and owner-only revoke");
            }
            let a = Args::parse(args, &["--cgroup", "--task-uid", "--wait-ms"], &[])?;
            a.clean()?;
            let path = a
                .one("--cgroup")?
                .ok_or_else(|| Error(2, "host control requires exact --cgroup".into()))?;
            let uid: u32 = a
                .one("--task-uid")?
                .ok_or_else(|| Error(2, "host control requires --task-uid".into()))?
                .parse()
                .map_err(|_| Error(2, "task UID must be a non-root u32".into()))?;
            if sub == "supervise" {
                if a.one("--wait-ms")?.is_some() {
                    return store::fail(2, "supervisor wait is fixed");
                }
                return host_control::supervise(Path::new(path), uid);
            }
            if sub == "inspect" {
                if a.one("--wait-ms")?.is_some() {
                    return store::fail(2, "--wait-ms applies only to revoke");
                }
                host_control::inspect(Path::new(path), uid)
            } else {
                host_control::revoke(Path::new(path), uid, a.number("--wait-ms", 1000)?)
            }
        }
        "self-test" => {
            let a = Args::parse(rest, &[], &[])?;
            a.clean()?;
            self_test::run(root)
        }
        "mcp" => {
            let a = Args::parse(
                rest,
                &[
                    "--root",
                    "--task-socket",
                    "--task-id",
                    "--task-session",
                    "--task-read-fd",
                    "--task-write-fd",
                ],
                &[],
            )?;
            a.clean()?;
            let r = a
                .one("--root")?
                .map(std::path::PathBuf::from)
                .unwrap_or(root.into());
            if a.one("--task-read-fd")?.is_some() || a.one("--task-write-fd")?.is_some() {
                if a.one("--task-socket")?.is_some() {
                    return store::fail(2, "select one task MCP transport");
                }
                let required = |k| a.one(k)?.ok_or_else(|| Error(2, format!("required {k}")));
                let read_fd = required("--task-read-fd")?
                    .parse::<i32>()
                    .map_err(|_| Error(2, "invalid read descriptor".into()))?;
                let write_fd = required("--task-write-fd")?
                    .parse::<i32>()
                    .map_err(|_| Error(2, "invalid write descriptor".into()))?;
                mcp::serve_task_pipe(
                    &r,
                    read_fd,
                    write_fd,
                    required("--task-id")?,
                    required("--task-session")?,
                )?;
                return Ok(json!({"server":"stopped"}));
            }
            match (
                a.one("--task-socket")?,
                a.one("--task-id")?,
                a.one("--task-session")?,
            ) {
                (None, None, None) => mcp::serve(&r)?,
                (Some(socket), Some(id), Some(session)) => {
                    mcp::serve_task(&r, Path::new(socket), id, session)?
                }
                _ => {
                    return store::fail(
                        2,
                        "task MCP binding requires socket, ID and session together",
                    )
                }
            }
            Ok(json!({"server":"stopped"}))
        }
        "init" => {
            let a = Args::parse(
                rest,
                &["--goal", "--codex-home"],
                &["--apply", "--uninstall"],
            )?;
            a.clean()?;
            let detected = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .any(|p| p.join("codex").is_file());
            let home = a
                .one("--codex-home")?
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    if detected {
                        std::env::var_os("CODEX_HOME")
                            .map(std::path::PathBuf::from)
                            .or_else(|| {
                                std::env::var_os("HOME")
                                    .map(|p| std::path::PathBuf::from(p).join(".codex"))
                            })
                    } else {
                        None
                    }
                });
            init::run(
                root,
                a.one("--goal")?,
                home.as_deref(),
                a.yes("--apply"),
                a.yes("--uninstall"),
            )
        }
        "ci" => {
            let a = Args::parse(
                rest,
                &["--require-evidence", "--evidence-commands", "--max-age-min"],
                &[],
            )?;
            a.clean()?;
            let commands = a
                .one("--evidence-commands")?
                .map(|p| store::read(&root.join(p)))
                .transpose()?;
            gate::ci(
                root,
                a.list("--require-evidence"),
                commands,
                a.number("--max-age-min", 240)?,
            )
        }
        "verify" => {
            let a = Args::parse(rest, &[], &["--trust", "--status"])?;
            a.clean()?;
            if a.yes("--trust") && a.yes("--status") {
                return store::fail(2, "choose --trust or --status");
            }
            gate::verify(
                root,
                if a.yes("--trust") {
                    "trust"
                } else if a.yes("--status") {
                    "status"
                } else {
                    "run"
                },
            )
        }
        "status" => {
            let a = Args::parse(rest, &[], &[])?;
            a.clean()?;
            Ok(json!({"state":store::state(root)?,"spend":spend::inspect(root,false)?}))
        }
        "test" | "evidence" => {
            let sub = rest.first().map(String::as_str).unwrap_or("check");
            let args = if rest.is_empty() { rest } else { &rest[1..] };
            if sub == "fingerprint" {
                let a = Args::parse(args, &[], &[])?;
                a.clean()?;
                Ok(
                    json!({"fingerprint":store::inventory_digest(&store::inventory(root)?)?,"format":"native-v3-full-byte-inventory"}),
                )
            } else if sub == "run" {
                let a = Args::parse(
                    args,
                    &[
                        "--label",
                        "--kind",
                        "--required-test",
                        "--env-name",
                        "--artifact",
                        "--timeout",
                    ],
                    &[],
                )?;
                if !a.position.is_empty() {
                    return store::fail(2, "unexpected positional arguments before command");
                }
                receipt::run(
                    root,
                    receipt::Run {
                        label: a.text("--label", "")?,
                        argv: a.command.clone(),
                        tests: a.list("--required-test"),
                        env: a.list("--env-name"),
                        artifacts: a.list("--artifact"),
                        timeout: a.number("--timeout", 60)?,
                        kind: a.text("--kind", "test")?,
                    },
                )
            } else if sub == "check" {
                let a = Args::parse(
                    args,
                    &[
                        "--label",
                        "--max-age-min",
                        "--expect-argv",
                        "--receipt",
                        "--history",
                        "--trusted-key",
                    ],
                    &["--signature-only"],
                )?;
                a.clean()?;
                if a.yes("--signature-only") {
                    let trusted = std::fs::read_to_string(
                        root.join(a.text("--trusted-key", ".kineti/evidence-issuer.pem")?),
                    )?;
                    if let Some(path) = a.one("--history")? {
                        let rows = store::lines(&root.join(path))?;
                        return Ok(
                            json!({"signature_valid":rows.iter().map(|r|receipt::signature(r,&trusted)).collect::<Vec<_>>(),"independent_test_execution":false,"gate_authorized":false}),
                        );
                    }
                    let r = store::read(&root.join(a.text("--receipt", "")?))?;
                    if !receipt::signature(&r, &trusted) {
                        return store::fail(7, "invalid signature or issuer");
                    }
                    return Ok(
                        json!({"signature_valid":true,"independent_test_execution":false,"gate_authorized":false}),
                    );
                }
                let expected = a
                    .one("--expect-argv")?
                    .map(serde_json::from_str::<Value>)
                    .transpose()
                    .map_err(|e| Error(2, e.to_string()))?;
                receipt::check_kind(
                    root,
                    &a.text("--label", "test")?,
                    a.number("--max-age-min", 240)?,
                    expected.as_ref(),
                    cmd == "evidence",
                )
            } else {
                store::fail(2, "test supports run or check")
            }
        }
        "spend" => {
            let sub = rest.first().map(String::as_str).unwrap_or("status");
            let args = if rest.is_empty() { rest } else { &rest[1..] };
            match sub {
                "assessment" => {
                    let a = Args::parse(args, &[], &[])?;
                    a.clean()?;
                    spend::assessment(root)
                }
                "status" | "check" | "economics" => {
                    let a = Args::parse(args, &[], &[])?;
                    a.clean()?;
                    spend::inspect(root, sub != "status")
                }
                "log" => {
                    let a = Args::parse(
                        args,
                        &["--stage", "--model", "--tokens-in", "--tokens-out", "--usd"],
                        &[],
                    )?;
                    a.clean()?;
                    let usd = a.one("--usd")?.ok_or_else(|| {
                        Error(
                            2,
                            "explicit reported --usd required; no inferred provider prices".into(),
                        )
                    })?;
                    spend::charge(
                        root,
                        &a.text("--stage", "")?,
                        &a.text("--model", "reported")?,
                        a.number("--tokens-in", 0)?,
                        a.number("--tokens-out", 0)?,
                        usd,
                    )
                }
                "reset" => {
                    let a = Args::parse(args, &["--extend-usd"], &["--i-am-human"])?;
                    a.clean()?;
                    if a.one("--extend-usd")?
                        .is_some_and(|v| spend::money(v).is_ok_and(|n| n == 0))
                    {
                        return store::fail(2, "explicit extension must be positive");
                    }
                    spend::reset(root, a.yes("--i-am-human"), &a.text("--extend-usd", "0")?)
                }
                "recover" => {
                    let a = Args::parse(
                        args,
                        &["--source", "--proposal-sha256", "--evidence"],
                        &["--apply", "--i-am-human"],
                    )?;
                    a.clean()?;
                    if let Some(source) = a.one("--source")? {
                        let source = root.join(source);
                        if a.yes("--apply") {
                            spend::recover_from(
                                root,
                                &source,
                                &a.text("--proposal-sha256", "")?,
                                Path::new(&a.text("--evidence", "")?),
                                a.yes("--i-am-human"),
                            )
                        } else if let Some(e) = a.one("--evidence")? {
                            spend::propose_review(root, &source, Path::new(e))
                        } else {
                            spend::propose_from(root, &source)
                        }
                    } else if a.yes("--apply") {
                        store::fail(
                            2,
                            "recovery apply needs an evidence-backed --source snapshot",
                        )
                    } else {
                        spend::recovery_proposal(root)
                    }
                }
                _ => store::fail(2, "unknown spend subcommand"),
            }
        }
        "undo" | "saga" => {
            let sub = rest.first().map(String::as_str).unwrap_or("rollback");
            let args = if rest.is_empty() { rest } else { &rest[1..] };
            match sub {
                "begin" | "commit" => {
                    let a = Args::parse(args, &["--run-id"], &[])?;
                    a.clean()?;
                    undo::transaction(root, &a.text("--run-id", "")?, sub == "commit")
                }
                "push" | "register" => {
                    let a = Args::parse(
                        args,
                        &["--label", "--run-id", "--inverse-argv", "--inverse"],
                        &[],
                    )?;
                    let modes = usize::from(a.separator)
                        + usize::from(a.one("--inverse-argv")?.is_some())
                        + usize::from(a.one("--inverse")?.is_some())
                        + usize::from(!a.position.is_empty());
                    if modes != 1 || a.position.len() > 1 {
                        return store::fail(2, "provide exactly one inverse command");
                    }
                    let argv = if !a.command.is_empty() {
                        a.command.clone()
                    } else if let Some(v) = a.one("--inverse-argv")? {
                        serde_json::from_str(v).map_err(|e| Error(2, e.to_string()))?
                    } else if let Some(v) = a
                        .one("--inverse")?
                        .or(a.position.last().map(String::as_str))
                    {
                        shell_words::split(v).map_err(|e| Error(2, e.to_string()))?
                    } else {
                        vec![]
                    };
                    undo::push(
                        root,
                        &a.text("--label", &format!("undo-{}", store::nonce()?))?,
                        &a.text("--run-id", "default")?,
                        argv,
                    )
                }
                "rollback" => {
                    let a = Args::parse(args, &["--run-id"], &["--yes"])?;
                    a.clean()?;
                    undo::rollback(root, &a.text("--run-id", "default")?, a.yes("--yes"))
                }
                "inspect" => {
                    let a = Args::parse(args, &["--expected-sha256"], &["--yes"])?;
                    a.clean()?;
                    undo::inspect(root, a.one("--expected-sha256")?, a.yes("--yes"))
                }
                "resolve" => {
                    let a =
                        Args::parse(args, &["--run-id", "--action-id", "--outcome"], &["--yes"])?;
                    a.clean()?;
                    undo::resolve(
                        root,
                        &a.text("--run-id", "default")?,
                        &a.text("--action-id", "")?,
                        &a.text("--outcome", "")?,
                        a.yes("--yes"),
                    )
                }
                _ => store::fail(2, "unknown undo subcommand"),
            }
        }
        _ => store::fail(
            2,
            format!("command '{cmd}' is removed from the native gate; see native gate contract"),
        ),
    }
}
fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // Only gate-level --json is consumed; command arguments after -- are untouched.
    let stop = args.iter().position(|s| s == "--").unwrap_or(args.len());
    let mut n = 0;
    args.retain(|s| {
        let keep = n >= stop || s != "--json";
        n += 1;
        keep
    });
    let result = dispatch(&std::env::current_dir().unwrap(), &args);
    match result {
        Ok(v) => {
            if args.first().is_some_and(|s| s == "mcp")
                || args.first().is_some_and(|s| s == "host")
                    && args.get(1).is_some_and(|s| s == "supervise")
            {
                return;
            }
            let blocked = task::blocked(&v);
            println!(
                "{}",
                json!({"schema":"kineti-cli-v1","ok":!blocked,"result":v})
            );
            if blocked {
                std::process::exit(3);
            }
        }
        Err(e) => {
            println!(
                "{}",
                json!({"schema":"kineti-cli-v1","ok":false,"code":e.0,"error":e.1})
            );
            std::process::exit(e.0);
        }
    }
}
