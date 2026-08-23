mod agent_loop;
mod anchor;
mod config;
mod enforce;
mod integrity;
mod memory;
mod provider;
mod quarantine;
mod stages;
mod tools;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "kineti",
    version,
    about = "Agent harness — context integrity, mechanical governance, chained memory"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold .kineti/ state and kineti.toml in the current directory
    Init,
    /// Run the full 13-stage governed pipeline on a goal
    Run {
        /// The locked root goal
        #[arg(long)]
        goal: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
        /// Override global spend cap (demo aid)
        #[arg(long)]
        cap: Option<f64>,
    },
    /// One freeform governed task (no stage machine)
    Task {
        /// The task for the agent
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Resume the pipeline from the last saved stage
    Resume {
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Roll back saga-registered file changes, newest-first
    Undo,
    /// Run a command and bind its result to a code fingerprint
    Evidence {
        /// The verification command, e.g. "cargo test"
        #[arg(long)]
        cmd: String,
    },
    /// Run the ship proof gate only (fresh-fingerprint check)
    ShipCheck,
    /// Verify the journal hash chain and print the head
    Verify,
    /// Print the hash-chained run record summary
    Receipt {
        #[arg(long)]
        last: bool,
    },
    /// Send one tiny completion to a provider (smoke test)
    ProviderTest {
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::Init => cmd_init(),
        Cmd::Run { goal, provider, model, cap } => {
            cmd_pipeline(&goal, &provider, model.as_deref(), cap)
        }
        Cmd::Task { task, provider, model } => cmd_run(&task, &provider, model.as_deref()),
        Cmd::Resume { provider, model } => {
            match std::fs::read_to_string(".kineti/root_goal") {
                Ok(goal) => resume_pipeline(goal.trim(), &provider, model.as_deref()),
                Err(_) => {
                    eprintln!("no root goal found — run `kineti run --goal …` first");
                    1
                }
            }
        }
        Cmd::Undo => cmd_undo(),
        Cmd::Evidence { cmd } => cmd_evidence(&cmd),
        Cmd::Receipt { .. } => cmd_receipt(),
        Cmd::ShipCheck => match enforce::evidence::check_ship(std::path::Path::new(".")) {
            Ok(proof) => {
                println!("✔ proofs FRESH ({} at {})", proof.command, proof.at);
                0
            }
            Err(e) => {
                eprintln!("⛔ {e}");
                1
            }
        },
        Cmd::Verify => cmd_verify(),
        Cmd::ProviderTest { provider, model } => cmd_provider_test(&provider, model.as_deref()),
    };
    std::process::exit(code);
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

    let r = agent_loop::run_task(&root, &p, &m, task, cfg.limits.global_usd);
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

fn cmd_pipeline(goal: &str, provider: &str, model: Option<&str>, cap: Option<f64>) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider);
    let m = model.unwrap_or(&p.default_model).to_string();
    // human reset check (ETHOS §3.3)
    if enforce::spend::Spend::human_reset_if_requested(std::path::Path::new(".")) {
        println!("⚡ spend breaker reset via human flag file — counter zeroed");
    }
    let g = cap.unwrap_or(cfg.limits.global_usd);
    println!("kineti pipeline ── {provider}/{m} | spend cap ${g:.2}");
    stages::drive(&std::env::current_dir().expect("cwd"), &p, &m, goal, g)
}

fn resume_pipeline(goal: &str, provider: &str, model: Option<&str>) -> i32 {
    let cfg = config::Config::load();
    let p = cfg.provider(provider);
    let m = model.unwrap_or(&p.default_model).to_string();
    enforce::spend::Spend::human_reset_if_requested(std::path::Path::new("."));
    println!("kineti resume ── {provider}/{m}");
    stages::drive(&std::env::current_dir().expect("cwd"), &p, &m, goal, cfg.limits.global_usd)
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

fn cmd_receipt() -> i32 {
    let j = memory::journal::Journal::load(std::path::Path::new(".kineti/journal.jsonl"));
    let g = memory::graph::Graph::load(std::path::Path::new(".kineti/graph.jsonl"));
    if let Err(e) = j.verify() {
        eprintln!("⛔ chain TAMPERED: {e}");
        return 1;
    }

    let goal = std::fs::read_to_string(".kineti/root_goal").unwrap_or_default();
    let last_run = j.records.iter().rev().find(|r| r.r#type == "run-record");
    let gates: Vec<&memory::journal::Record> =
        j.records.iter().filter(|r| r.r#type == "gate").collect();

    println!("╔═══════════════ KINETI RECEIPT ═══════════════╗");
    println!("goal       : {}", goal.trim());
    if let Some(r) = last_run {
        println!("last run   : {} · {}", r.id, r.at);
        println!(
            "outcome    : {} | iters {} | ${:.4} | {}→{} tok",
            r.data["outcome"].as_str().unwrap_or("?"),
            r.data["iterations"].as_u64().unwrap_or(0),
            r.data["cost_usd"].as_f64().unwrap_or(0.0),
            r.data["prompt_tokens"].as_u64().unwrap_or(0),
            r.data["completion_tokens"].as_u64().unwrap_or(0),
        );
    }
    println!("records    : {} chained", j.records.len());
    println!("causal edges: {}", g.edges.len());
    println!("gates hit:");
    for gate in gates {
        println!(
            "   • {} {}",
            gate.data["kind"].as_str().unwrap_or("?"),
            gate.data["detail"].as_str().unwrap_or("")
        );
    }
    let head = j.records.last().map(|r| r.hash.clone()).unwrap_or_default();
    println!("chain head : {}", head.get(..16).unwrap_or(head.as_str()));
    println!("╚══════════════════════════════════════════════╝");
    0
}

fn cmd_verify() -> i32 {
    let j = memory::journal::Journal::load(std::path::Path::new(".kineti/journal.jsonl"));
    match j.verify() {
        Ok(()) => {
            println!(
                "journal OK: {} records, chain head {}",
                j.records.len(),
                j.records.last().map(|r| &r.hash[..16]).unwrap_or("GENESIS")
            );
            0
        }
        Err(e) => {
            eprintln!("TAMPERED: {e}");
            1
        }
    }
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
