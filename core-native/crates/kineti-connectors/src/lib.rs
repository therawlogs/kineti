//! # Kineti Connectors (`kineti-connectors`)
//!
//! High-speed connectors for external platforms, tools, and auth:
//! - **[`otp`]**: WhatsApp & SMS phone number verification codes (5-min expiry, rate limiting).
//! - **[`brave`]**: Real-time Brave Search API for live web updates.
//! - **[`flux`]**: FLUX.1 photorealistic image generation with prompt optimization.
//! - **[`gmail`]**: Google Workspace & Gmail inbox search and draft creation.
//! - **[`notion`]**: Notion database query and page/task management.
//! - **[`opencode`]**: OpenCode Go LLM inference connector (OpenAI-compatible).
//! - **[`vault`]**: Authenticated encrypted vault for storing user OAuth tokens.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod brave;
pub mod flux;
pub mod gmail;
pub mod notion;
pub mod opencode;
pub mod otp;
pub mod vault;

pub use brave::{BraveSearchClient, SearchHit};
pub use flux::{AspectRatio, FluxClient, FluxGenerationRequest, FluxImageResult};
pub use gmail::{DraftEmailRequest, EmailSummary, GmailClient};
pub use notion::{CreateNotionPageRequest, NotionClient, NotionPageItem};
pub use opencode::{ChatMessage, InferenceRequest, InferenceResponse, OpenCodeClient};
pub use otp::{OtpChallenge, OtpManager};
pub use vault::{CredentialVault, EncryptedCredential};

/// Prelude module for convenient single-import access to connectors.
pub mod prelude {
    pub use crate::brave::{BraveSearchClient, SearchHit};
    pub use crate::flux::{AspectRatio, FluxClient, FluxGenerationRequest, FluxImageResult};
    pub use crate::gmail::{DraftEmailRequest, EmailSummary, GmailClient};
    pub use crate::notion::{CreateNotionPageRequest, NotionClient, NotionPageItem};
    pub use crate::opencode::{ChatMessage, InferenceRequest, InferenceResponse, OpenCodeClient};
    pub use crate::otp::{OtpChallenge, OtpManager};
    pub use crate::vault::{CredentialVault, EncryptedCredential};
}

