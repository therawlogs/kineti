//! Reflex Circuit for executing fixed action patterns.
//!
//! Evaluates classified sensory perceptions and executes immediate
//! reflexes (emoji reactions, greetings, memory saves) without invoking
//! expensive deliberative LLMs, achieving sub-5ms response loops.

use crate::sensor::{IntentCategory, SensoryPerception};
use crate::style::StyleAnalyzer;
use kineti_core::conversation::UserStyleProfile;

/// Action emitted by the reflex circuit.
#[derive(Debug, Clone, PartialEq)]
pub enum ReflexAction {
    /// Fired for low-information messages ("ok", "got it", "thanks", "leaving now").
    /// Triggers a native WhatsApp reaction (👍, ❤️, ⚡) on the user's message bubble with zero text clutter.
    ReactWithEmoji {
        /// Unicode emoji to attach as reaction.
        emoji: &'static str,
    },
    /// Direct instant text reply generated locally with 0 LLM tokens.
    FastReply {
        /// The synthesized message text.
        text: String,
    },
    /// Fast memory write returning immediate acknowledgment.
    StoreMemoryAndAck {
        /// Subject or key of fact.
        subject: String,
        /// Value of fact.
        content: String,
        /// Response to return to user.
        ack_text: String,
    },
    /// Requires deliberative LLM synthesis or tool execution.
    RouteToCortex {
        /// The classified intent.
        intent: IntentCategory,
        /// Processed prompt.
        prompt: String,
        /// Calibrated style profile.
        style: UserStyleProfile,
    },
}

/// The Reflex Circuit engine.
#[derive(Debug, Default, Clone)]
pub struct ReflexCircuit {
    style_analyzer: StyleAnalyzer,
}

impl ReflexCircuit {
    /// Creates a new reflex circuit.
    pub fn new() -> Self {
        Self {
            style_analyzer: StyleAnalyzer::new(),
        }
    }

    /// Evaluates a sensory perception and decides the immediate action pattern.
    pub fn evaluate(
        &self,
        perception: &SensoryPerception,
        raw_text: &str,
        user_style: &UserStyleProfile,
    ) -> ReflexAction {
        match &perception.intent {
            IntentCategory::LowInfoStatusAck => {
                let lower = raw_text.to_lowercase();
                let emoji = if lower.contains("thanks") || lower.contains("ty") || lower.contains("thx") {
                    "❤️"
                } else if lower.contains("leaving") || lower.contains("heading out") || lower.contains("omw") {
                    "⚡"
                } else {
                    "👍"
                };
                ReflexAction::ReactWithEmoji { emoji }
            }
            IntentCategory::FastReflexGreeting => {
                let greeting = if user_style.lowercase_preference {
                    "hey! what can i help you with today?"
                } else if user_style.formality >= 0.7 {
                    "Good day. How may I assist you?"
                } else {
                    "Hey! What can I help with?"
                };
                let calibrated = self.style_analyzer.calibrate_output(user_style, greeting);
                ReflexAction::FastReply {
                    text: calibrated,
                }
            }
            IntentCategory::MemoryStore { subject, content } => {
                let ack = if user_style.lowercase_preference {
                    format!("got it locked in 👍\n{}", content)
                } else {
                    format!("Got it locked in.\n• {}: {}\nI'll remember this.", subject, content)
                };
                let calibrated = self.style_analyzer.calibrate_output(user_style, &ack);
                ReflexAction::StoreMemoryAndAck {
                    subject: subject.clone(),
                    content: content.clone(),
                    ack_text: calibrated,
                }
            }
            _ => ReflexAction::RouteToCortex {
                intent: perception.intent.clone(),
                prompt: raw_text.to_string(),
                style: user_style.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensor::SensoryClassifier;

    #[test]
    fn test_low_info_triggers_emoji_reaction() {
        let classifier = SensoryClassifier::new();
        let circuit = ReflexCircuit::new();
        let style = UserStyleProfile::default();

        let p1 = classifier.classify("ok got it", None, None);
        let a1 = circuit.evaluate(&p1, "ok got it", &style);
        assert_eq!(a1, ReflexAction::ReactWithEmoji { emoji: "👍" });

        let p2 = classifier.classify("thanks so much!", None, None);
        let a2 = circuit.evaluate(&p2, "thanks so much!", &style);
        assert_eq!(a2, ReflexAction::ReactWithEmoji { emoji: "❤️" });
    }

    #[test]
    fn test_fast_greeting_style_adaptation() {
        let classifier = SensoryClassifier::new();
        let circuit = ReflexCircuit::new();
        let mut style = UserStyleProfile::default();
        style.lowercase_preference = true;

        let p = classifier.classify("yo", None, None);
        let a = circuit.evaluate(&p, "yo", &style);
        if let ReflexAction::FastReply { text } = a {
            assert!(text.contains("hey!"));
            assert_eq!(text, text.to_lowercase());
        } else {
            panic!("Expected FastReply");
        }
    }

    #[test]
    fn test_memory_store_instant_ack() {
        let classifier = SensoryClassifier::new();
        let circuit = ReflexCircuit::new();
        let style = UserStyleProfile::default();

        let p = classifier.classify("my manager is Sarah Chen", None, None);
        let a = circuit.evaluate(&p, "my manager is Sarah Chen", &style);
        if let ReflexAction::StoreMemoryAndAck { subject, content, ack_text } = a {
            assert_eq!(subject, "manager");
            assert_eq!(content, "Sarah Chen");
            assert!(ack_text.contains("Sarah Chen"));
        } else {
            panic!("Expected StoreMemoryAndAck");
        }
    }
}
