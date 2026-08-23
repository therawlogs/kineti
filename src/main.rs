mod anchor;
mod config;
mod agent_loop;
mod integrity;
mod memory;
mod provider;
mod quarantine;
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
    /// Run the governed agent loop on a freeform task
    Run {
        /// The task for the agent
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
    },
    /// Resume from last recorded stage
    Resume,
    /// Print the hash-chained run record
    Receipt {
        #[arg(long)]
        last: bool,
    },
    /// Verify the journal hash chain and print the head
    Verify,
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
        Cmd::Run { task, provider, model } => cmd_run(&task, &provider, model.as_deref()),
        Cmd::Resume => {
            eprintln!("not built yet (day-3 milestone)");
            2
        }
        Cmd::Receipt { .. } => {
            eprintln!("not built yet (day-3 milestone)");
            2
        }
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
