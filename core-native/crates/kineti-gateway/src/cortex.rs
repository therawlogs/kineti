//! Deliberative Cortex Orchestrator and Multi-Model Router.
//!
//! Formulates context-hydrated prompts for Claude 3.7, GPT-4o, and fast micro-models,
//! conditioning responses on the user's personal memory graph and dynamic style profile.

use kineti_core::conversation::{UserFact, UserStyleProfile};

/// Target deliberative AI model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelTier {
    /// Fast/cheapest model for lightweight classification and direct formatting.
    #[default]
    FastMicro,
    /// High-intelligence reasoning and nuanced conversational model (Claude 3.7 / GPT-4o).
    DeepReasoning,
}

impl ModelTier {
    /// String identifier of model.
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::FastMicro => "claude-3-5-haiku-20241022",
            Self::DeepReasoning => "claude-3-7-sonnet-20250219",
        }
    }
}

/// Supported LLM Provider backend.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    /// OpenCode Go subscription API (OpenAI-compatible endpoint).
    OpenCodeGo {
        /// OpenCode API Key.
        api_key: String,
        /// Base URL (defaults to https://opencode.ai/zen/go/v1/chat/completions).
        endpoint: String,
        /// Model identifier (e.g. "opencode-go/claude-3-7-sonnet" or "opencode-go/default").
        model: String,
    },
    /// Generic OpenAI-compatible gateway (e.g. DeepSeek, Groq, Ollama, OpenRouter).
    OpenAICompatible {
        /// Provider API Key.
        api_key: String,
        /// Endpoint URL.
        endpoint: String,
        /// Model identifier.
        model: String,
    },
    /// Direct Anthropic Claude API.
    Anthropic {
        /// Anthropic API Key.
        api_key: String,
        /// Model name.
        model: String,
    },
    /// Direct OpenAI API.
    OpenAI {
        /// OpenAI API Key.
        api_key: String,
        /// Model name.
        model: String,
    },
}

impl LlmProvider {
    /// Creates an OpenCode Go provider with the given API key.
    pub fn opencode_go(api_key: &str, model: Option<&str>) -> Self {
        Self::OpenCodeGo {
            api_key: api_key.to_string(),
            endpoint: "https://opencode.ai/zen/go/v1/chat/completions".to_string(),
            model: model.unwrap_or("opencode-go/claude-3-7-sonnet").to_string(),
        }
    }

    /// Automatically detects LLM provider from environment variables.
    /// Checks `OPENCODE_API_KEY` / `OPENCODE_GO_KEY` first.
    pub fn from_env() -> Option<Self> {
        if let Ok(key) = std::env::var("OPENCODE_API_KEY").or_else(|_| std::env::var("OPENCODE_GO_KEY")) {
            let model = std::env::var("OPENCODE_MODEL").ok();
            return Some(Self::opencode_go(&key, model.as_deref()));
        }
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Some(Self::Anthropic {
                api_key: key,
                model: "claude-3-7-sonnet-20250219".to_string(),
            });
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            return Some(Self::OpenAI {
                api_key: key,
                model: "gpt-4o".to_string(),
            });
        }
        None
    }

    /// Builds HTTP URL, headers, and JSON request payload for this provider.
    pub fn build_http_request(&self, prompt: &InferencePrompt) -> (String, Vec<(String, String)>, String) {
        match self {
            Self::OpenCodeGo { api_key, endpoint, model }
            | Self::OpenAICompatible { api_key, endpoint, model } => {
                let headers = vec![
                    ("Authorization".to_string(), format!("Bearer {}", api_key)),
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("User-Agent".to_string(), "Kineti-Cortex/1.0".to_string()),
                ];

                // OpenAI-compatible chat completions payload
                let body = format!(
                    r#"{{"model":{},"messages":[{{"role":"system","content":{}}},{{"role":"user","content":{}}}],"temperature":0.7}}"#,
                    kineti_core::kernel::escape_json_string(model),
                    kineti_core::kernel::escape_json_string(&prompt.system_prompt),
                    kineti_core::kernel::escape_json_string(&prompt.user_message),
                );

                (endpoint.clone(), headers, body)
            }
            Self::Anthropic { api_key, model } => {
                let headers = vec![
                    ("x-api-key".to_string(), api_key.clone()),
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ];

                let body = format!(
                    r#"{{"model":{},"system":{},"messages":[{{"role":"user","content":{}}}],"max_tokens":1024}}"#,
                    kineti_core::kernel::escape_json_string(model),
                    kineti_core::kernel::escape_json_string(&prompt.system_prompt),
                    kineti_core::kernel::escape_json_string(&prompt.user_message),
                );

                ("https://api.anthropic.com/v1/messages".to_string(), headers, body)
            }
            Self::OpenAI { api_key, model } => {
                let headers = vec![
                    ("Authorization".to_string(), format!("Bearer {}", api_key)),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ];

                let body = format!(
                    r#"{{"model":{},"messages":[{{"role":"system","content":{}}},{{"role":"user","content":{}}}],"temperature":0.7}}"#,
                    kineti_core::kernel::escape_json_string(model),
                    kineti_core::kernel::escape_json_string(&prompt.system_prompt),
                    kineti_core::kernel::escape_json_string(&prompt.user_message),
                );

                ("https://api.openai.com/v1/chat/completions".to_string(), headers, body)
            }
        }
    }
}

/// Prompt bundle ready for model inference.
#[derive(Debug, Clone)]
pub struct InferencePrompt {
    /// Selected model.
    pub model: ModelTier,
    /// System prompt embedding user facts and style rules.
    pub system_prompt: String,
    /// Current user query.
    pub user_message: String,
}

/// Cortex Orchestrator.
#[derive(Debug, Default, Clone)]
pub struct CortexOrchestrator {
    /// Optional configured LLM provider.
    pub provider: Option<LlmProvider>,
}

impl CortexOrchestrator {
    /// Creates a new cortex orchestrator.
    pub fn new() -> Self {
        Self {
            provider: LlmProvider::from_env(),
        }
    }

    /// Sets the LLM provider explicitly (e.g. OpenCode Go).
    pub fn with_provider(mut self, provider: LlmProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Selects model tier based on query complexity.
    pub fn select_model(&self, query: &str) -> ModelTier {
        let len = query.split_whitespace().count();
        if len > 30 || query.contains("compare") || query.contains("analyze") || query.contains("explain") {
            ModelTier::DeepReasoning
        } else {
            ModelTier::FastMicro
        }
    }

    /// Builds context-hydrated system prompt and inference bundle.
    pub fn build_inference_prompt(
        &self,
        query: &str,
        user_facts: &[UserFact],
        style: &UserStyleProfile,
    ) -> InferencePrompt {
        let model = self.select_model(query);

        let mut sys = String::from(
            "You are Kineti, an ultra-fast, intelligent personal companion living in iMessage and WhatsApp.\n\
             Rules:\n\
             1. Answer directly and concisely. Zero fluff, no AI pleasantries.\n\
             2. Never output complex markdown tables or broken code blocks; use clean spacing and short bullet points.\n"
        );

        // Inject tone and style directives
        if style.lowercase_preference {
            sys.push_str("3. Style rule: Use lowercase and a relaxed, fast conversational vibe matching the user.\n");
        }
        if style.formality >= 0.7 {
            sys.push_str("3. Style rule: Maintain a polished, professional, executive tone.\n");
        }

        // Inject personal facts
        if !user_facts.is_empty() {
            sys.push_str("\nKnown facts about this user:\n");
            for fact in user_facts {
                sys.push_str(&format!("- {}: {}\n", fact.key, fact.value));
            }
        }

        InferencePrompt {
            model,
            system_prompt: sys,
            user_message: query.to_string(),
        }
    }

    /// Builds context-hydrated system prompt and inference bundle from a resolved epistemic persona view.
    pub fn build_inference_prompt_from_persona(
        &self,
        query: &str,
        persona: &kineti_memory::ResolvedPersonaView,
        style: &UserStyleProfile,
    ) -> InferencePrompt {
        let model = self.select_model(query);

        let mut sys = String::from(
            "You are Kineti, an ultra-fast, intelligent personal companion living in iMessage and WhatsApp.\n\
             Rules:\n\
             1. Answer directly and concisely. Zero fluff, no AI pleasantries.\n\
             2. Never output complex markdown tables or broken code blocks; use clean spacing and short bullet points.\n"
        );

        // Inject tone and style directives
        if style.lowercase_preference {
            sys.push_str("3. Style rule: Use lowercase and a relaxed, fast conversational vibe matching the user.\n");
        }
        if style.formality >= 0.7 {
            sys.push_str("3. Style rule: Maintain a polished, professional, executive tone.\n");
        }

        let directives = persona.compile_prompt_directives();
        if !directives.is_empty() {
            sys.push_str("\n--- RESOLVED USER PERSONA DIRECTIVES ---\n");
            sys.push_str(&directives);
        }

        InferencePrompt {
            model,
            system_prompt: sys,
            user_message: query.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortex_model_selection() {
        let cortex = CortexOrchestrator::new();
        assert_eq!(cortex.select_model("what time is it"), ModelTier::FastMicro);
        assert_eq!(
            cortex.select_model("Please analyze and compare the optical characteristics of Leica lenses"),
            ModelTier::DeepReasoning
        );
    }

    #[test]
    fn test_cortex_prompt_hydration_with_style_and_facts() {
        let cortex = CortexOrchestrator::new();
        let fact = UserFact::new(
            "user_01",
            "relationship",
            "manager",
            "Sarah Chen",
            1.0,
            None,
            1710000000,
        );
        let mut style = UserStyleProfile::default();
        style.lowercase_preference = true;

        let prompt = cortex.build_inference_prompt("did sarah email me", &[fact], &style);
        assert!(prompt.system_prompt.contains("Sarah Chen"));
        assert!(prompt.system_prompt.contains("lowercase and a relaxed"));
    }

    #[test]
    fn test_opencode_go_provider_request_builder() {
        let provider = LlmProvider::opencode_go("opencode_live_12345", None);
        let fact = UserFact::new("u1", "general", "name", "Alex", 1.0, None, 1710000000);
        let style = UserStyleProfile::default();
        let cortex = CortexOrchestrator::new().with_provider(provider.clone());

        let prompt = cortex.build_inference_prompt("What is my name?", &[fact], &style);
        let (url, headers, body) = provider.build_http_request(&prompt);

        assert_eq!(url, "https://opencode.ai/zen/go/v1/chat/completions");
        assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer opencode_live_12345"));
        assert!(body.contains("opencode-go/claude-3-7-sonnet"));
        assert!(body.contains("What is my name?"));
    }

    #[test]
    fn test_cortex_prompt_hydration_from_persona() {
        use kineti_memory::{ContextScope, DomainKind, EpistemicCertainty, EpistemicFact, ResolvedPersonaView, RuleConstraintType};
        let cortex = CortexOrchestrator::new();
        let fact = EpistemicFact {
            id: "ep_01".to_string(),
            user_id: "u1".to_string(),
            scope: ContextScope::Domain(DomainKind::Health),
            attribute: "diet".to_string(),
            claim: "Vegetarian".to_string(),
            constraint_type: RuleConstraintType::BaselineRule,
            certainty: EpistemicCertainty::DirectlyKnown,
            valid_from: 1000,
            valid_until: None,
            contradiction_criteria: None,
            consequence_level: kineti_memory::ConsequenceLevel::Operational,
        };
        let persona = ResolvedPersonaView {
            baseline_rules: vec![fact],
            exceptions: vec![],
            preferences: vec![],
            safety_ceilings: vec![],
        };
        let mut style = UserStyleProfile::default();
        style.formality = 0.8;

        let prompt = cortex.build_inference_prompt_from_persona("what should I order?", &persona, &style);
        assert!(prompt.system_prompt.contains("RESOLVED USER PERSONA DIRECTIVES"));
        assert!(prompt.system_prompt.contains("Vegetarian"));
        assert!(prompt.system_prompt.contains("executive tone"));
    }
}
