use serde::Deserialize;

pub const DEFAULT_TOML: &str = r#"
[providers.gemini]
base_url = "https://generativelanguage.googleapis.com/v1beta/openai"
api_key_env = "GEMINI_API_KEY"
default_model = "gemini-3.6-flash"
price_per_1m_input = 0.0
price_per_1m_output = 0.0

[providers.grok]
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
default_model = "grok-4.20-non-reasoning"
price_per_1m_input = 12.5
price_per_1m_output = 25.0

# Optional OAuth2/PKCE per provider — `kineti login --provider grok` then skips env keys:
# [providers.grok.auth]
# client_id = "kineti-cli"
# authorize_url = "https://idp.example.com/authorize"
# token_url = "https://idp.example.com/token"
# scopes = "openid profile"

[clean_files]
# extra case-insensitive forbidden substrings (team/client/project names)
forbid = []

# ── Generic artifact fingerprint (any agent) ──────────────────────────────
# Which files count toward the proof fingerprint. Works for code, docs,
# datasets, configs — any work product an agent produces.
[artifacts]
# glob patterns relative to repo root; first match wins. Empty = defaults.
include = ["**/*"]
exclude = [".git", ".kineti", "target", "node_modules", "dist", "build", ".next", "coverage", "tmp", ".cache", "legacy"]
# skip files larger than this (bytes) — avoids hashing large binaries/datasets; 0 = no limit
max_file_bytes = 4194304
follow_symlinks = false

[proof]
# default verification command for `evidence` / `ship-check` when --cmd not given.
# Works for any agent: "cargo test", "pytest", "npm test", "make check", "./verify.sh"
command = ""
# legacy alias still honored: [limits].verify_command

[execution]
# v0.2 product is evidence → ship-check → verify (no mode). This only matters for `kineti run --legacy`.
# "single" = linear pipeline | "swarm" = parallel workers
mode = "single"
max_parallel_workers = 4
# swarm only: auto | git | scratchpad — how worker trees are isolated (Phase 5)
worker_isolation = "auto"

[limits]
global_usd = 50.0
# per-scope ceiling, ENFORCED (ETHOS §3.1). Applies to any agent spend via gateway/wrap. 0 = disabled.
per_stage_usd = 10.0
# legacy alias: per_stage_usd = per-scope cap; keep name for compat
# per-worker ceiling for swarm mode. 0 = disabled.
# max_worker_usd = 25.0
context_char_budget = 24000
# legacy: use [proof].command instead
# verify_command = "cargo test"
"#;

/// Optional per-provider OAuth2 (PKCE) endpoints. When a token exists for
/// the provider it wins over the env-var key (Phase 8).
#[derive(Deserialize, Clone, Debug)]
pub struct OAuthCfg {
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
    #[serde(default)]
    pub scopes: String,
}

#[derive(Deserialize, Clone)]
pub struct ProviderCfg {
    pub base_url: String,
    pub api_key_env: String,
    pub default_model: String,
    /// Injected by Config::provider() so deeper layers can key token storage
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub auth: Option<OAuthCfg>,
    #[serde(default)]
    pub price_per_1m_input: f64,
    #[serde(default)]
    pub price_per_1m_output: f64,
}

#[derive(Deserialize, Clone)]
pub struct Limits {
    #[serde(default = "d_global")]
    pub global_usd: f64,
    /// Per-stage ceiling, ENFORCED since Phase 2 (ETHOS §3.1). 0 disables.
    #[serde(default = "d_stage")]
    pub per_stage_usd: f64,
    /// Per-worker ceiling for swarm mode (Phase 6). 0 = disabled.
    #[serde(default)]
    pub max_worker_usd: f64,
    #[serde(default = "d_budget")]
    pub context_char_budget: usize,
    #[serde(default)]
    pub verify_command: String,
}
fn d_global() -> f64 { 50.0 }
fn d_stage() -> f64 { 10.0 }
fn d_budget() -> usize { 24_000 }

#[derive(Deserialize, Clone, Debug)]
pub struct Execution {
    #[serde(default = "d_mode")]
    pub mode: String,
    #[serde(default = "d_par")]
    pub max_parallel_workers: usize,
    #[serde(default = "d_iso")]
    pub worker_isolation: String,
}
fn d_mode() -> String { "single".into() }
fn d_par() -> usize { 4 }
fn d_iso() -> String { "auto".into() }
impl Default for Execution {
    fn default() -> Self {
        Execution { mode: d_mode(), max_parallel_workers: d_par(), worker_isolation: d_iso() }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct CleanFiles {
    #[serde(default)]
    pub forbid: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Artifacts {
    #[serde(default = "d_art_include")]
    pub include: Vec<String>,
    #[serde(default = "d_art_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "d_art_max_bytes")]
    pub max_file_bytes: usize,
    #[serde(default)]
    pub follow_symlinks: bool,
}
fn d_art_include() -> Vec<String> { vec!["**/*".into()] }
fn d_art_exclude() -> Vec<String> {
    vec![".git".into(), ".kineti".into(), "target".into(), "node_modules".into(),
         "dist".into(), "build".into(), ".next".into(), "coverage".into(),
         "tmp".into(), ".cache".into(), "legacy".into()]
}
fn d_art_max_bytes() -> usize { 4 * 1024 * 1024 }
impl Default for Artifacts {
    fn default() -> Self {
        Artifacts { include: d_art_include(), exclude: d_art_exclude(), max_file_bytes: d_art_max_bytes(), follow_symlinks: false }
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ProofCfg {
    #[serde(default)]
    pub command: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub providers: std::collections::HashMap<String, ProviderCfg>,
    #[serde(default = "Limits::defaults")]
    pub limits: Limits,
    #[serde(default)]
    pub clean_files: CleanFiles,
    #[serde(default)]
    pub artifacts: Artifacts,
    #[serde(default)]
    pub proof: ProofCfg,
    #[serde(default)]
    pub execution: Execution,
}
impl Limits {
    fn defaults() -> Self {
        Limits {
            global_usd: 50.0,
            per_stage_usd: 10.0,
            max_worker_usd: 0.0,
            context_char_budget: 24_000,
            verify_command: String::new(),
        }
    }
}

impl Config {
    /// Load kineti.toml from cwd; fall back to built-in defaults when absent.
    pub fn load() -> Self { Self::load_from(std::path::Path::new(".")) }

    pub fn load_from(root: &std::path::Path) -> Self {
        let path = root.join("kineti.toml");
        let mut cfg: Self = match std::fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
                eprintln!("kineti.toml parse error at {} ({e}); using defaults", path.display());
                toml::from_str(DEFAULT_TOML).expect("built-in default toml must parse")
            }),
            Err(_) => {
                // fallback to cwd for backwards compat when root != cwd but file only at cwd
                if root != std::path::Path::new(".") {
                    if let Ok(s) = std::fs::read_to_string("kineti.toml") {
                        if let Ok(c) = toml::from_str::<Self>(&s) { return { let mut c2 = c; if c2.proof.command.trim().is_empty() && !c2.limits.verify_command.trim().is_empty() { c2.proof.command = c2.limits.verify_command.clone(); } c2 } }
                    }
                }
                toml::from_str(DEFAULT_TOML).expect("built-in default toml must parse")
            },
        };
        // legacy alias: [limits].verify_command → [proof].command
        if cfg.proof.command.trim().is_empty() && !cfg.limits.verify_command.trim().is_empty() {
            cfg.proof.command = cfg.limits.verify_command.clone();
        }
        cfg
    }

    /// Unified proof command — honors [proof].command first, then legacy [limits].verify_command.
    pub fn proof_command(&self) -> String {
        if !self.proof.command.trim().is_empty() { self.proof.command.clone() } else { self.limits.verify_command.clone() }
    }

    pub fn provider(&self, name: &str) -> ProviderCfg {
        let mut cfg = self.providers.get(name).cloned().unwrap_or_else(|| {
            eprintln!("unknown provider '{name}', falling back to gemini");
            self.providers["gemini"].clone()
        });
        cfg.name = name.to_string();
        cfg
    }
}
