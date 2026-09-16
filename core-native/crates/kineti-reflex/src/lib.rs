//! # Kineti Reflex (`kineti-reflex`)
//!
//! Sub-millisecond sensory triage, dynamic style analysis, and reflex circuit routing.
//!
//! ## Core Modules
//! - **[`sensor`]**: Fast classification of modalities (Text, Audio, Photo) and intents in `< 0.2ms`.
//! - **[`style`]**: User communication style analysis (formality, brevity, slang, lowercase, dialect) and dynamic output tone calibration.
//! - **[`circuit`]**: Fixed action pattern execution (native WhatsApp emoji reactions, instant greetings, memory store acks) with 0 LLM tokens.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod circuit;
pub mod sensor;
pub mod style;

pub use circuit::{ReflexAction, ReflexCircuit};
pub use sensor::{IntentCategory, Modality, SensoryClassifier, SensoryPerception};
pub use style::StyleAnalyzer;

/// Prelude module for convenient access to kineti-reflex types.
pub mod prelude {
    pub use crate::circuit::{ReflexAction, ReflexCircuit};
    pub use crate::sensor::{IntentCategory, Modality, SensoryClassifier, SensoryPerception};
    pub use crate::style::StyleAnalyzer;
}
