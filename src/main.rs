mod config;
mod provider;

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
    /// Run the governed agent loop
    Run {
        #[arg(long)]
        stage: Option<u8>,
    },
    /// Resume from last recorded stage
    Resume,
    /// Print the hash-chained run record
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
        Cmd::Run { .. } | Cmd::Resume => {
            eprintln!("not built yet (day-1 milestone)");
            2
        }
        Cmd::Receipt { .. } => {
            eprintln!("not built yet (day-3 milestone)");
            2
        }
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
    ) {
        Ok(ok) => {
            println!("reply   : {}", ok.text.trim());
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
