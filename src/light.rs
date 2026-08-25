//! Pre-clap fast path (startup budget, Tier 2): exact-match light commands
//! answer without building clap's full Command tree — no heap-heavy arg
//! parsing on the hot "am I alive?" paths. Anything not matched here falls
//! through to normal clap handling unchanged.

/// Returns a ready-to-print reply for exact-match light invocations.
/// Input is argv WITHOUT the binary name. `None` = defer to clap.
pub fn reply(args: &[String]) -> Option<String> {
    if args.len() != 1 {
        return None; // only unambiguous single-flag forms short-circuit
    }
    match args[0].as_str() {
        "-V" | "--version" => Some(format!("kineti {}", env!("CARGO_PKG_VERSION"))),
        "-h" | "--help" => Some(SHORT_HELP.to_string()),
        _ => None,
    }
}

pub const SHORT_HELP: &str =
    "kineti — agent harness: governed single-agent runs and parallel swarms.\n\
     Try `kineti --help` for the full command list.";
