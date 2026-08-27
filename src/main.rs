use clap::{Parser, Subcommand};

use kineti::{agent_loop, config, daemon, enforce, ipc, light, memory, provider, stages};

#[derive(Parser)]
#[command(
    name = "kineti",
    version,
    about = "Ship proof + spend fuse for any agent — gateway meter and merge stamp"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold .kineti/ state and kineti.toml in the current directory
    Init,
    /// Run a command and bind its result to a code fingerprint (default product)
    Evidence {
        /// The verification command, e.g. "cargo test"
        #[arg(long)]
        cmd: String,
    },
    /// Run the ship proof gate — refuse if proof stale/missing (exit 1 = stale, 2 = missing, 3 = chain broken)
    ShipCheck,
    /// Verify the journal hash chain and print the head
    Verify {
        /// Full DAG check: main chain + every merged branch + orphan closure
        #[arg(long)]
        all: bool,
    },
    /// Print the hash-chained receipt summary (spend + hashes + gates)
    Receipt {
        #[arg(long)]
        last: bool,
    },
    /// Scan project files for names/home-paths/secrets — zero matches required
    CleanCheck,
    /// Run the kinetid governance daemon on .kineti/kineti.sock
    Serve {
        /// Accepted for symmetry; serve always runs in-process (spawners detach it).
        #[arg(long)]
        foreground: bool,
    },
    /// Wrap an external agent under the spend cap: kineti wrap -- claude -p "..."
    Wrap {
        /// The command to run under the cap (after --)
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// OpenAI-compatible gateway proxy with reserve/settle (demo; hosted in kineti-pro)
    Gateway {
        /// Port to listen on
        #[arg(long, default_value = "8787")]
        port: u16,
    },
    /// One freeform governed task (legacy thin wrapper)
    Task {
        /// The task for the agent
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Send one tiny completion to a provider (smoke test)
    ProviderTest {
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// OAuth2/PKCE login for a configured provider (opens browser)
    Login {
        /// provider name as keyed in kineti.toml [providers.<name>]
        #[arg(long)]
        provider: String,
    },
    /// Remove a stored OAuth token
    Logout {
        #[arg(long)]
        provider: String,
    },
    /// List stored OAuth tokens and expiry state
    AuthStatus,
    // ── legacy (hidden from default help) ──
    /// Legacy 13-stage pipeline — frozen at v0.1.0, see docs/v0.1.md (use `kineti run --legacy --goal "..."`)
    #[command(hide = true)]
    Run {
        /// Legacy flag — use `kineti run --legacy --goal "..."` (no effect, kept for docs parity)
        #[arg(long)]
        legacy: bool,
        /// The locked root goal
        #[arg(long)]
        goal: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        cap: Option<f64>,
        #[arg(long)]
        mode: Option<String>,
    },
    #[command(hide = true)]
    Resume {
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    #[command(hide = true)]
    Undo,
    #[command(hide = true)]
    Merge {
        #[arg(long)]
        branch: String,
    },
}

fn main() {
    // Tier-2 fast path: exact-match light flags never build the clap tree.
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Some(reply) = light::reply(&argv) {
        println!("{reply}");
        return;
    }
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::Init => cmd_init(),
        Cmd::Evidence { cmd } => cmd_evidence(&cmd),
        Cmd::ShipCheck => cmd_ship_check(),
        Cmd::Verify { all } => cmd_verify(all),
        Cmd::Receipt { .. } => cmd_receipt(),
        Cmd::CleanCheck => cmd_clean_check(),
        Cmd::Serve { foreground } => daemon::serve(std::path::Path::new("."), foreground),
        Cmd::Wrap { command } => cmd_wrap(&command),
        Cmd::Gateway { port } => kineti::gateway::serve(port),
        Cmd::Task { task, provider, model } => cmd_run(&task, &provider, model.as_deref()),
        Cmd::ProviderTest { provider, model } => cmd_provider_test(&provider, model.as_deref()),
        Cmd::Login { provider } => cmd_login(&provider),
        Cmd::Logout { provider } => cmd_logout(&provider),
        Cmd::AuthStatus => cmd_auth_status(),
        Cmd::Run { legacy: _, goal, provider, model, cap, mode } => {
            eprintln!("⚠ `kineti run --legacy` is legacy — frozen at v0.1.0. See docs/v0.1.md");
            eprintln!("  The product is now: evidence → ship-check → verify (gateway meter + stamp).");
            cmd_pipeline(&goal, &provider, model.as_deref(), cap, mode)
        }
        Cmd::Resume { provider, model } => {
            eprintln!("⚠ `kineti resume` is legacy — see docs/v0.1.md (frozen at v0.1.0)");
            match std::fs::read_to_string(".kineti/root_goal") {
                Ok(goal) => resume_pipeline(goal.trim(), &provider, model.as_deref()),
                Err(_) => {
                    eprintln!("no root goal found — `kineti run --legacy` was the legacy pipeline");
                    1
                }
            }
        }
        Cmd::Undo => cmd_undo(),
        Cmd::Merge { branch } => cmd_merge(&branch),
    };
    std::process::exit(code);
}


fn ceilings_from(cfg: &config::Config, cap_override: Option<f64>) -> ipc::pool::Ceilings {
    let pos = |v: f64| (v > 0.0).then(|| (v * 1_000_000.0).round() as u64);
    ipc::pool::Ceilings {
        global_micro: (cap_override.unwrap_or(cfg.limits.global_usd).max(0.0) * 1_000_000.0)
            .round() as u64,
        stage_micro: pos(cfg.limits.per_stage_usd),
        worker_micro: pos(cfg.limits.max_worker_usd),
    }
}

/// Human-only reset (ETHOS §3.3), routed through the selected backend so a
/// live daemon's pool state is reset too — never a divergent direct write.
/// Fails closed when the ledger lock cannot be taken.
fn human_reset_via_backend(root: &std::path::Path) -> Result<bool, String> {
    let cfg = config::Config::load();
    let (svc, _) = ipc::select_backends(root, ceilings_from(&cfg, None))?;
    svc.reset_if_human_requested(root)
}

fn cmd_init() -> i32 {
    if std::path::Path::new(".kineti").exists() {
        println!(".kineti/ already exists — nothing to do");
        return 0;
    }
    if let Err(e) = std::fs::create_dir_all(".kineti") {
        eprintln!("init failed: {e}");
        return 1;
    }
    if !std::path::Path::new("kineti.toml").exists() {
        if let Err(e) = std::fs::write("kineti.toml", config::DEFAULT_TOML.trim_start()) {
            eprintln!("write kineti.toml failed: {e}");
            return 1;
        }
    }
    println!("initialized .kineti/ + kineti.toml");
    0
}

fn cmd_run(task: &str, provider_name: &str, model: Option<&str>) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider_name);
    let m = model.unwrap_or(&p.default_model).to_string();
    let root = std::env::current_dir().expect("cwd");

    println!("kineti run ── {provider_name}/{m}");
    println!("goal  : {task}\n");

    let r = agent_loop::run_task(&root, &p, &m, task, ceilings_from(&cfg, None));
    println!("\n═══════════════════════════════");
    if let Some(h) = &r.halted {
        println!("HALTED: {h}");
    }
    println!("iterations : {}", r.iterations);
    println!("tokens     : {} in / {} out", r.prompt_tokens, r.completion_tokens);
    println!("cost       : ${:.6}", r.cost_usd);
    if !r.answer.is_empty() {
        println!("answer     :\n{}", r.answer.trim());
    }
    if r.halted.is_some() {
        1
    } else {
        0
    }
}

fn cmd_pipeline(
    goal: &str,
    provider: &str,
    model: Option<&str>,
    cap: Option<f64>,
    mode_override: Option<String>,
) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider);
    let m = model.unwrap_or(&p.default_model).to_string();
    // human reset check (ETHOS §3.3) — through the backend, never a side-write
    match human_reset_via_backend(std::path::Path::new(".")) {
        Ok(true) => println!("⚡ spend breaker reset via human flag file — counter zeroed"),
        Ok(false) => {}
        Err(e) => {
            eprintln!("⛔ {e}");
            return 1;
        }
    }
    let ceilings = ceilings_from(&cfg, cap);
    println!(
        "kineti pipeline ── {provider}/{m} | spend cap ${:.2} | per-stage {}",
        ceilings.global_micro as f64 / 1_000_000.0,
        ceilings
            .stage_micro
            .map(|m| format!("${:.2}", m as f64 / 1_000_000.0))
            .unwrap_or_else(|| "off".into())
    );
    let mut exec = cfg.execution.clone();
    if let Some(mo) = mode_override {
        exec.mode = mo;
    }
    stages::drive(&std::env::current_dir().expect("cwd"), &p, &m, goal, ceilings, &exec)
}

fn resume_pipeline(goal: &str, provider: &str, model: Option<&str>) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider);
    let m = model.unwrap_or(&p.default_model).to_string();
    if let Err(e) = human_reset_via_backend(std::path::Path::new(".")) {
        eprintln!("⛔ {e}");
        return 1;
    }
    println!("kineti resume ── {provider}/{m}");
    stages::drive(
        &std::env::current_dir().expect("cwd"),
        &p,
        &m,
        goal,
        ceilings_from(&cfg, None),
        &cfg.execution.clone(),
    )
}

fn cmd_undo() -> i32 {
    let saga = enforce::saga::Saga::load(std::path::Path::new("."));
    println!("rolling back {} registered step(s), newest-first…", saga.steps.len());
    let n = saga.rollback_all();
    println!("{n} step(s) undone");
    0
}

fn cmd_evidence(cmd: &str) -> i32 {
    let root = std::env::current_dir().expect("cwd");
    println!("evidence: running `{cmd}`…");
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .current_dir(&root)
        .env_remove("GEMINI_API_KEY")
        .env_remove("XAI_API_KEY")
        .status();
    let code = match out {
        Ok(s) => s.code().unwrap_or(-1),
        Err(e) => {
            eprintln!("spawn failed: {e}");
            return 1;
        }
    };
    let proof = enforce::evidence::record(&root, cmd, code == 0, code);
    println!(
        "proof bound: passed={} fingerprint={}… ({})",
        proof.passed,
        &proof.fingerprint[..12],
        proof.at
    );
    if code == 0 { 0 } else { 1 }
}


fn cmd_login(provider: &str) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider);
    let oa = match &p.auth {
        Some(a) => a,
        None => {
            eprintln!(
                "⛔ provider '{provider}' has no [providers.{provider}.auth] block in kineti.toml"
            );
            return 1;
        }
    };
    let sess = match kineti::auth::prepare_login(&p.name, oa) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⛔ login failed to start: {e}");
            return 1;
        }
    };
    println!("╔═══════════════ KINETI LOGIN — {provider} ═══════════════");
    println!("║ open this URL in your browser:");
    println!("║ {}", sess.auth_url);
    println!("║ waiting for redirect on 127.0.0.1:{} (5 min timeout)…", sess.port);
    println!("╚═════════════════════════════════════════════════════════");

    let code = match kineti::auth::await_callback(&sess, std::time::Duration::from_secs(300)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⛔ {e}");
            return 1;
        }
    };
    match kineti::auth::exchange_code(&p.name, oa, &code, &sess.verifier, sess.port) {
        Ok(tok) => {
            if let Err(e) = kineti::auth::save_token(&tok) {
                eprintln!("⛔ token save failed: {e}");
                return 1;
            }
            let exp = tok
                .expires_at
                .map(|e| format!("expires at unix {}", e))
                .unwrap_or_else(|| "no expiry".into());
            println!("✔ logged in as '{provider}' ({exp}) — env key no longer required");
            0
        }
        Err(e) => {
            eprintln!("⛔ token exchange failed: {e}");
            1
        }
    }
}

fn cmd_logout(provider: &str) -> i32 {
    if kineti::auth::logout(provider) {
        println!("✔ removed stored token for '{provider}'");
        0
    } else {
        println!("no stored token for '{provider}'");
        1
    }
}

fn cmd_auth_status() -> i32 {
    let tokens = kineti::auth::status_all();
    if tokens.is_empty() {
        println!("no stored OAuth tokens (env keys still work)");
        return 0;
    }
    for t in &tokens {
        println!(
            "• {} : {}{}{}",
            t.provider,
            if t.expired { "EXPIRED" } else { "valid" },
            t.expires_at.map(|e| format!(", expires unix {e}")).unwrap_or_default(),
            if t.has_refresh { ", refresh available" } else { "" },
        );
    }
    0
}

fn root_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

fn cmd_receipt() -> i32 {
    let root = root_dir();
    let r = kineti::receipt::build(&root);

    println!("╔═══════════════ KINETI RECEIPT ═══════════════╗");
    println!("goal       : {}", r.goal);

    if let Some((id, outcome, cost)) = &r.last_run {
        println!("last run   : {id} · outcome {outcome} · ${cost:.4}");
    }
    println!(
        "spend      : ${:.4} total (coordinator ${:.4} + workers {})",
        r.total_cost_usd(),
        r.coordinator_cost_usd,
        if r.workers.is_empty() {
            "none".into()
        } else {
            format!(
                "${:.4}",
                r.workers.iter().map(|w| w.cost_usd).sum::<f64>()
            )
        }
    );
    for w in &r.workers {
        println!(
            "   • {} : ${:.4} · {}→{} tok · {} records",
            w.branch, w.cost_usd, w.prompt_tokens, w.completion_tokens, w.records
        );
    }

    println!("history    : {} records chained · head {}", r.records, &r.chain_head[..16.min(r.chain_head.len())]);
    println!("causal     : {} edges", r.causal_edges);

    if !r.dag.branches.is_empty() || !r.dag.orphans.is_empty() {
        println!(
            "branches   : {} merged{}",
            r.dag.branches.len(),
            if r.dag.orphans.is_empty() {
                String::new()
            } else {
                format!(", ⚠ ORPHANS: {}", r.dag.orphans.join(", "))
            }
        );
    }
    for e in &r.dag.errors {
        println!("   ⛔ {e}");
    }

    if !r.gates.is_empty() {
        println!("gate timeline:");
        for g in &r.gates {
            let t = g.at.get(11..19).unwrap_or(&g.at);
            println!("   • [{t}] {} {}", g.kind, g.detail);
        }
    }

    if !r.egress.is_empty() {
        print!("egress     : ");
        let parts: Vec<String> =
            r.egress.iter().map(|e| format!("{}: {} send(s)", e.tag, e.records)).collect();
        println!("{}", parts.join(" · "));
    } else {
        println!("egress     : no outbound sends recorded");
    }

    match &r.clean_files {
        Ok(()) => println!("clean-files: ✔ 0 findings"),
        Err(n) => println!("clean-files: ⚠ {n} finding(s) — ship will refuse"),
    }
    if !r.dag.is_clean() {
        println!("⚠ history not clean — `kineti verify --all` for details");
    }
    println!("╚══════════════════════════════════════════════╝");

    // receipt stays informational; gates enforce. Nonzero only on broken history.
    if r.dag.is_clean() { 0 } else { 1 }
}


fn cmd_verify(all: bool) -> i32 {
    let root = std::path::Path::new(".");
    if !all {
        let j = memory::journal::Journal::load(std::path::Path::new(".kineti/journal.jsonl"));
        return match j.verify() {
            Ok(()) => {
                println!(
                    "journal OK: {} records, chain head {}",
                    j.records.len(),
                    j.records.last().map(|r| r.hash.get(..16).unwrap_or(&r.hash)).unwrap_or("GENESIS")
                );
                0
            }
            Err(e) => {
                eprintln!("TAMPERED: {e}");
                1
            }
        };
    }

    let report = memory::merge::verify_project(root);
    if !report.errors.is_empty() {
        for e in &report.errors {
            eprintln!("⛔ DAG: {e}");
        }
    }
    for o in &report.orphans {
        eprintln!("⛔ ORPHAN branch file {o} — never merged; run `kineti merge --branch <name>`");
    }
    println!(
        "DAG {}: main {} records (head {}), branches merged: {}, orphans: {}",
        if report.is_clean() { "OK" } else { "REFUSED" },
        report.main_records,
        report.main_head.get(..16).unwrap_or(&report.main_head),
        report.branches.len(),
        report.orphans.len(),
    );
    for (b, n, h) in &report.branches {
        println!("   • {b}: {n} records, head {}", h.get(..16).unwrap_or(h));
    }
    if report.is_clean() { 0 } else { 1 }
}

fn cmd_merge(branch: &str) -> i32 {
    let root = std::env::current_dir().expect("cwd");
    let mut w = ipc::journal_writer_no_spawn(&root);
    match memory::merge::merge_branch(&root, w.as_mut(), branch) {
        Ok(Some(rec)) => {
            println!(
                "✔ merged '{}' → main (records chained through {})",
                branch,
                rec.data["head"].as_str().and_then(|h| h.get(..16)).unwrap_or("?")
            );
            let rep = memory::merge::verify_project(&root);
            if rep.is_clean() {
                println!("✔ DAG clean: {} merged branch(es)", rep.branches.len());
                0
            } else {
                for e in &rep.errors { eprintln!("⛔ {e}"); }
                for o in &rep.orphans { eprintln!("⛔ ORPHAN {o}"); }
                1
            }
        }
        Ok(None) => {
            eprintln!("branch '{branch}' is empty — nothing to merge");
            1
        }
        Err(e) => {
            eprintln!("⛔ merge failed: {e}");
            1
        }
    }
}

fn cmd_ship_check() -> i32 {
    match enforce::evidence::check_ship(std::path::Path::new(".")) {
        Ok(proof) => {
            println!("✔ proofs FRESH ({} at {})", proof.command, proof.at);
            0
        }
        Err(e) => {
            // exit codes: 2 = missing, 1 = stale/failed, 3 = chain broken (via verify)
            let code = if e.contains("MISSING") { 2 } else { 1 };
            eprintln!("⛔ {e}");
            code
        }
    }
}

fn cmd_wrap(command: &[String]) -> i32 {
    if command.is_empty() {
        eprintln!("usage: kineti wrap -- <command> [args...]");
        return 1;
    }
    let cfg = config::Config::load();
    let ceilings = ceilings_from(&cfg, None);
    let root = std::env::current_dir().expect("cwd");
    // reserve a small estimate for the wrapped command's potential model calls
    // (actual spend tracked inside agent_loop; this is just a cap check)
    let ctx = crate::ipc::dto::ReserveCtx {
        stage: "wrap".into(),
        worker: String::new(),
        est_micro_usd: 0,
    };
    match crate::ipc::select_backends(&root, ceilings) {
        Ok((svc, _)) => {
            if let Err(e) = svc.reserve(&ctx) {
                eprintln!("⛔ {e}");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("⛔ ledger: {e}");
            return 1;
        }
    }
    let status = std::process::Command::new(&command[0])
        .args(&command[1..])
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("wrap spawn failed: {e}");
            1
        }
    }
}

fn cmd_clean_check() -> i32 {
    let cfg = config::Config::load();
    let root = std::env::current_dir().expect("cwd");
    let findings = enforce::cleanfiles::scan(&root, &cfg.clean_files.forbid);
    if findings.is_empty() {
        println!("✔ clean-files scan: 0 findings");
        return 0;
    }
    eprintln!("⛔ clean-files scan: {} finding(s) — zero matches required", findings.len());
    for f in &findings {
        eprintln!("   {}:{} [{}] {}", f.path, f.line, f.kind, f.snippet);
    }
    1
}

fn cmd_provider_test(name: &str, model: Option<&str>) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(name);
    let m = model.unwrap_or(&p.default_model).to_string();
    println!(
        "provider-test → {} ({})\nendpoint: {}",
        name, m, p.base_url
    );
    match provider::chat(
        &p,
        &m,
        &[provider::Msg::user("Reply with exactly: KINETI-OK")],
        &[],
    ) {
        Ok(ok) => {
            println!("reply   : {}", ok.content.trim());
            println!("tokens  : {} in / {} out", ok.usage.prompt_tokens, ok.usage.completion_tokens);
            println!("cost    : ${:.6}", ok.cost_usd);
            0
        }
        Err(e) => {
            eprintln!("FAILED: {e}");
            1
        }
    }
}
