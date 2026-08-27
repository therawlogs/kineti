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

# Optional OAuth2/PKCE per provider — `kineti login grok` then skips env keys:
# [providers.grok.auth]
# client_id = "kineti-cli"
# authorize_url = "https://idp.example.com/authorize"
# token_url = "https://idp.example.com/token"
# scopes = "openid profile"

[clean_files]
# extra case-insensitive forbidden substrings (team/client/project names)
forbid = []

[execution]
# "single" = linear 13-stage pipeline (default) | "swarm" = parallel workers (legacy, use `kineti run --legacy`)
mode = "single"
max_parallel_workers = 4
# auto | git | scratchpad — how worker trees are isolated (Phase 5, legacy swarm only)
worker_isolation = "auto"

[limits]
global_usd = 50.0
# per-stage ceiling, ENFORCED (ETHOS §3.1). Set 0 to disable.
per_stage_usd = 10.0
# per-worker ceiling for swarm mode. 0 = disabled.
# max_worker_usd = 25.0
context_char_budget = 24000
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

#[derive(Deserialize, Clone)]
pub struct Config {
    pub providers: std::collections::HashMap<String, ProviderCfg>,
    #[serde(default = "Limits::defaults")]
    pub limits: Limits,
    #[serde(default)]
    pub clean_files: CleanFiles,
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
    pub fn load() -> Self {
        match std::fs::read_to_string("kineti.toml") {
            Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
                eprintln!("kineti.toml parse error ({e}); using defaults");
                toml::from_str(DEFAULT_TOML).expect("built-in default toml must parse")
            }),
            Err(_) => toml::from_str(DEFAULT_TOML).expect("built-in default toml must parse"),
        }
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
