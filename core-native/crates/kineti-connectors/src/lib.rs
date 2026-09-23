//! # Kineti Connectors (`kineti-connectors`)
//!
//! High-speed connectors for external platforms, tools, and auth:
//! - **[`protocol`]**: Unified Non-Virtual Interface (NVI) protocol and single-use permission tokens.
//! - **[`otp`]**: WhatsApp & SMS phone number verification codes (5-min expiry, rate limiting).
//! - **[`brave`]**: Real-time Brave Search API for live web updates.
//! - **[`flux`]**: FLUX.1 photorealistic image generation with prompt optimization.
//! - **[`gmail`]**: Google Workspace & Gmail inbox search and draft creation.
//! - **[`notion`]**: Notion database query and page/task management.
//! - **[`opencode`]**: OpenCode Go LLM inference connector (OpenAI-compatible).
//! - **[`vault`]**: Authenticated encrypted vault for storing user OAuth tokens.

#![deny(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::manual_is_multiple_of)]

pub mod agent_email;
pub mod brave;
pub mod engineering;
pub mod flux;
pub mod gmail;
pub mod google_workspace;
pub mod microsoft_graph;
pub mod notion;
pub mod onepassword;
pub mod opencode;
pub mod otp;
pub mod protocol;
pub mod team_comms;
pub mod totp;
pub mod vault;

pub use protocol::{
    compute_bytes_sha256, compute_payload_sha256, get_str_property, ActionAuthorizationToken,
    ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
pub use agent_email::{AgentEmailClient, InboundAgentEmail, OutboundAgentEmail};
pub use brave::{BraveSearchClient, SearchHit};
pub use engineering::{GitHubClient, LinearClient};
pub use flux::{AspectRatio, FluxClient, FluxGenerationRequest, FluxImageResult};
pub use gmail::{DraftEmailRequest, EmailSummary, GmailClient};
pub use google_workspace::GoogleWorkspaceClient;
pub use microsoft_graph::MicrosoftGraphClient;
pub use notion::{CreateNotionPageRequest, NotionClient, NotionPageItem};
pub use onepassword::{
    OnePasswordAuditReceipt, OnePasswordBrokerClient, OnePasswordCategory, OnePasswordItem,
};
pub use opencode::{ChatMessage, InferenceRequest, InferenceResponse, OpenCodeClient};
pub use otp::{OtpChallenge, OtpManager};
pub use team_comms::{GranolaClient, SlackClient, WisprFlowClient};
pub use totp::{
    decode_base32, generate_hotp_code, parse_otpauth_uri, TotpAuthenticator,
    TotpAuthenticatorConnector, TotpParams,
};
pub use vault::{
    AgentItemEntry, CredentialVault, EncryptedCredential, PaymentCardEntry, PersonalInfoEntry,
    TotpSeedEntry, VaultBackend, WebLoginEntry,
};

/// Prelude module for convenient single-import access to connectors.
pub mod prelude {
    pub use crate::brave::{BraveSearchClient, SearchHit};
    pub use crate::flux::{AspectRatio, FluxClient, FluxGenerationRequest, FluxImageResult};
    pub use crate::gmail::{DraftEmailRequest, EmailSummary, GmailClient};
    pub use crate::notion::{CreateNotionPageRequest, NotionClient, NotionPageItem};
    pub use crate::opencode::{ChatMessage, InferenceRequest, InferenceResponse, OpenCodeClient};
    pub use crate::otp::{OtpChallenge, OtpManager};
    pub use crate::protocol::{
        compute_bytes_sha256, compute_payload_sha256, get_str_property,
        ActionAuthorizationToken, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
    };
    pub use crate::vault::{CredentialVault, EncryptedCredential};
}
