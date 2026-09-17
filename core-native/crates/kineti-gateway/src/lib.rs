//! # Kineti Gateway (`kineti-gateway`)
//!
//! Production messaging gateway and multi-channel orchestrator:
//! - **[`whatsapp`]**: Meta WhatsApp Cloud API webhooks, reactions, and native photo messages.
//! - **[`imessage`]**: macOS native iMessage AppleScript bridge.
//! - **[`cortex`]**: Deliberative multi-model prompt formulation and context hydration.
//! - **[`router`]**: End-to-end message routing across sensory triage, spend quotas, memory, tools, and actions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod cortex;
pub mod imessage;
pub mod router;
pub mod web_api;
pub mod whatsapp;

pub use cortex::{CortexOrchestrator, InferencePrompt, LlmProvider, ModelTier};
pub use imessage::IMessageBridge;
pub use router::{DispatchReceipt, EventSource, GatewayRouter, IncomingStimulusEvent, OutboundReply};
pub use web_api::{ConnectedAppStatus, UserSettingsPayload, VibePreferences, WebCompanionService};
pub use whatsapp::{InboundWhatsAppMessage, OutboundWhatsAppPayload, WhatsAppGateway};

/// Prelude module for convenient access to kineti-gateway types.
pub mod prelude {
    pub use crate::cortex::{CortexOrchestrator, InferencePrompt, LlmProvider, ModelTier};
    pub use crate::imessage::IMessageBridge;
    pub use crate::router::{DispatchReceipt, EventSource, GatewayRouter, IncomingStimulusEvent, OutboundReply};
    pub use crate::web_api::{ConnectedAppStatus, UserSettingsPayload, VibePreferences, WebCompanionService};
    pub use crate::whatsapp::{InboundWhatsAppMessage, OutboundWhatsAppPayload, WhatsAppGateway};
}
