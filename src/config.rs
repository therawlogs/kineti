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

[limits]
global_usd = 50.0
per_stage_usd = 10.0
context_char_budget = 24000
"#;

#[derive(Deserialize, Clone)]
pub struct ProviderCfg {
    pub base_url: String,
    pub api_key_env: String,
    pub default_model: String,
    #[serde(default)]
    pub price_per_1m_input: f64,
    #[serde(default)]
    pub price_per_1m_output: f64,
}

#[derive(Deserialize, Clone)]
pub struct Limits {
    #[serde(default = "d_global")]
    pub global_usd: f64,
    #[serde(default = "d_stage")]
    #[allow(dead_code)] // enforced per-stage in v0.2 stage budgeting
    pub per_stage_usd: f64,
    #[serde(default = "d_budget")]
    pub context_char_budget: usize,
    #[serde(default)]
    pub verify_command: String,
}
fn d_global() -> f64 { 50.0 }
fn d_stage() -> f64 { 10.0 }
fn d_budget() -> usize { 24_000 }

#[derive(Deserialize, Clone)]
pub struct Config {
    pub providers: std::collections::HashMap<String, ProviderCfg>,
    #[serde(default = "Limits::defaults")]
    pub limits: Limits,
}
impl Limits {
    fn defaults() -> Self {
        Limits { global_usd: 50.0, per_stage_usd: 10.0, context_char_budget: 24_000, verify_command: String::new() }
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
        self.providers.get(name).cloned().unwrap_or_else(|| {
            eprintln!("unknown provider '{name}', falling back to gemini");
            self.providers["gemini"].clone()
        })
    }
}
