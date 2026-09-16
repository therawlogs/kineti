//! Conversation and Messaging domain models for Kineti.
//!
//! Strongly typed structures representing users, chat threads, messages,
//! personalized user style profiles, and long-term memory facts.

use crate::hlc::HlcTimestamp;
use crate::kernel::{blake3, escape_json_string, hex_encode, NodeId};
use std::fmt;

/// Messaging channel platform through which a user interacts with Kineti.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Apple iMessage / Apple Messages for Business.
    AppleMessages,
    /// Meta WhatsApp Cloud API.
    WhatsApp,
    /// Mobile Web Companion (getkineti.com/settings).
    Web,
}

impl Platform {
    /// Returns string identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AppleMessages => "AppleMessages",
            Self::WhatsApp => "WhatsApp",
            Self::Web => "Web",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// User subscription and quota tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UserTier {
    /// Free tier: Daily budget allowance ($0.20/day).
    #[default]
    Free,
    /// Pro tier: Unlimited high-speed reasoning and image generation ($2.00/day allowance).
    Pro,
    /// Enterprise tier: Dedicated high-volume quota.
    Enterprise,
}

impl UserTier {
    /// Returns the daily microcent spend quota for this tier.
    pub fn daily_quota_microcents(&self) -> u64 {
        match self {
            Self::Free => 200_000,       // $0.20 USD = 200,000 microcents
            Self::Pro => 2_000_000,      // $2.00 USD = 2,000,000 microcents
            Self::Enterprise => 10_000_000, // $10.00 USD
        }
    }

    /// String representation of tier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Pro => "Pro",
            Self::Enterprise => "Enterprise",
        }
    }
}

/// Style and tone characteristics learned from a user's conversational behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct UserStyleProfile {
    /// Brevity and verbosity preference: 0.0 (ultra-terse) to 1.0 (detailed explanation).
    pub verbosity: f32,
    /// Whether user predominantly communicates without capital letters.
    pub lowercase_preference: bool,
    /// Emoji usage frequency: 0.0 (zero emojis) to 1.0 (emoji-rich).
    pub emoji_density: f32,
    /// Formality score: 0.0 (slang/casual) to 1.0 (executive/formal).
    pub formality: f32,
    /// Slang affinity: 0.0 (formal speech) to 1.0 (heavy slang like 'ngl', 'bet', 'w').
    pub slang_affinity: f32,
    /// Primary language or code-switching dialect (e.g. "en", "en-slang", "hi-en").
    pub language_dialect: String,
}

impl Default for UserStyleProfile {
    fn default() -> Self {
        Self {
            verbosity: 0.5,
            lowercase_preference: false,
            emoji_density: 0.2,
            formality: 0.5,
            slang_affinity: 0.2,
            language_dialect: "en".to_string(),
        }
    }
}

impl UserStyleProfile {
    /// Generates canonical JSON representation of the style profile.
    pub fn to_canonical_json(&self) -> String {
        format!(
            "{{\"emoji_density\":{:.4},\"formality\":{:.4},\"language_dialect\":{},\"lowercase_preference\":{},\"slang_affinity\":{:.4},\"verbosity\":{:.4}}}",
            self.emoji_density,
            self.formality,
            escape_json_string(&self.language_dialect),
            self.lowercase_preference,
            self.slang_affinity,
            self.verbosity
        )
    }
}

/// A unique user registered in the Kineti nervous system.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// Content-addressed or cryptographic user ID.
    pub id: NodeId,
    /// One-way BLAKE3 hash of phone number (E.164 format) for privacy.
    pub phone_hash: String,
    /// Primary onboarding platform.
    pub primary_platform: Platform,
    /// Subscription tier.
    pub tier: UserTier,
    /// Timestamp of user creation (Unix ms).
    pub created_at_ms: u64,
    /// Dynamic style profile of the user.
    pub style: UserStyleProfile,
}

impl User {
    /// Creates a new User instance and calculates content-addressed NodeId.
    pub fn new(
        raw_phone: &str,
        primary_platform: Platform,
        tier: UserTier,
        created_at_ms: u64,
    ) -> Self {
        let phone_hash = hex_encode(&blake3(raw_phone.as_bytes()));
        let style = UserStyleProfile::default();
        let payload = format!(
            "{{\"created_at_ms\":{},\"phone_hash\":\"{}\",\"platform\":\"{}\",\"tier\":\"{}\"}}",
            created_at_ms,
            phone_hash,
            primary_platform.as_str(),
            tier.as_str()
        );
        let id = NodeId::new(hex_encode(&blake3(payload.as_bytes())));
        Self {
            id,
            phone_hash,
            primary_platform,
            tier,
            created_at_ms,
            style,
        }
    }
}

/// Sender of a conversation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageSender {
    /// End human user.
    User,
    /// Autonomous assistant (Kineti).
    Assistant,
    /// System or Tool execution record.
    System,
}

impl MessageSender {
    /// String identifier for serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Assistant => "Assistant",
            Self::System => "System",
        }
    }
}

/// An individual message in a chat thread.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Content-addressed node identifier.
    pub id: NodeId,
    /// Parent conversation thread ID.
    pub thread_id: String,
    /// Authoring user ID.
    pub user_id: String,
    /// Sender role.
    pub sender: MessageSender,
    /// Text body of message.
    pub body: String,
    /// Optional media URL (image, voice note, document).
    pub media_url: Option<String>,
    /// Hybrid Logical Clock timestamp guaranteeing causal ordering.
    pub hlc: HlcTimestamp,
    /// Pre-cognitive classified intent tag (e.g. "greeting", "email_search", "image_create").
    pub intent_tag: Option<String>,
}

impl Message {
    /// Creates a new Message and computes its BLAKE3 content-addressed NodeId.
    pub fn new(
        thread_id: impl Into<String>,
        user_id: impl Into<String>,
        sender: MessageSender,
        body: impl Into<String>,
        media_url: Option<String>,
        hlc: HlcTimestamp,
        intent_tag: Option<String>,
    ) -> Self {
        let thread_id = thread_id.into();
        let user_id = user_id.into();
        let body = body.into();
        let payload = format!(
            "{{\"body\":\"{}\",\"hlc\":\"{}\",\"sender\":\"{}\",\"thread_id\":\"{}\",\"user_id\":\"{}\"}}",
            escape_json_string(&body),
            hlc,
            sender.as_str(),
            thread_id,
            user_id
        );
        let id = NodeId::new(hex_encode(&blake3(payload.as_bytes())));
        Self {
            id,
            thread_id,
            user_id,
            sender,
            body,
            media_url,
            hlc,
            intent_tag,
        }
    }
}

/// A permanent personal memory fact extracted from conversations.
#[derive(Debug, Clone, PartialEq)]
pub struct UserFact {
    /// Unique fact ID.
    pub id: NodeId,
    /// User ID this fact belongs to.
    pub user_id: String,
    /// Fact category (e.g. "relationship", "preference", "schedule", "work").
    pub category: String,
    /// Specific key or subject (e.g. "manager", "dietary", "meeting_time").
    pub key: String,
    /// Extracted fact content (e.g. "Sarah Chen", "no morning meetings", "hates cilantro").
    pub value: String,
    /// Confidence score from 0.0 to 1.0.
    pub confidence: f32,
    /// Source message NodeId that provided this fact.
    pub source_message_id: Option<String>,
    /// Creation timestamp (Unix ms).
    pub created_at_ms: u64,
    /// Whether this fact has been invalidated/tombstoned by a newer fact.
    pub tombstoned: bool,
}

impl UserFact {
    /// Creates a new UserFact with content-addressed ID.
    pub fn new(
        user_id: impl Into<String>,
        category: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
        confidence: f32,
        source_message_id: Option<String>,
        created_at_ms: u64,
    ) -> Self {
        let user_id = user_id.into();
        let category = category.into();
        let key = key.into();
        let value = value.into();
        let payload = format!(
            "{{\"category\":\"{}\",\"created_at_ms\":{},\"key\":\"{}\",\"user_id\":\"{}\",\"value\":\"{}\"}}",
            category, created_at_ms, key, user_id, escape_json_string(&value)
        );
        let id = NodeId::new(hex_encode(&blake3(payload.as_bytes())));
        Self {
            id,
            user_id,
            category,
            key,
            value,
            confidence,
            source_message_id,
            created_at_ms,
            tombstoned: false,
        }
    }
}

/// A conversation thread grouping related messages.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationThread {
    /// Content-addressed thread ID.
    pub id: NodeId,
    /// User ID owning this thread.
    pub user_id: String,
    /// Communication platform.
    pub platform: Platform,
    /// Thread creation timestamp (Unix ms).
    pub started_at_ms: u64,
    /// Last message timestamp (Unix ms).
    pub last_active_at_ms: u64,
    /// Optional thread title or topic.
    pub title: Option<String>,
}

impl ConversationThread {
    /// Creates a new ConversationThread.
    pub fn new(
        user_id: impl Into<String>,
        platform: Platform,
        started_at_ms: u64,
        title: Option<String>,
    ) -> Self {
        let user_id = user_id.into();
        let payload = format!(
            "{{\"platform\":\"{}\",\"started_at_ms\":{},\"user_id\":\"{}\"}}",
            platform.as_str(),
            started_at_ms,
            user_id
        );
        let id = NodeId::new(hex_encode(&blake3(payload.as_bytes())));
        Self {
            id,
            user_id,
            platform,
            started_at_ms,
            last_active_at_ms: started_at_ms,
            title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation_and_content_addressing() {
        let user = User::new("+15551234567", Platform::WhatsApp, UserTier::Pro, 1710000000);
        assert!(!user.id.as_str().is_empty());
        assert_eq!(user.tier, UserTier::Pro);
        assert_eq!(user.tier.daily_quota_microcents(), 2_000_000);
    }

    #[test]
    fn test_message_creation_and_causal_id() {
        let hlc = HlcTimestamp::new(1710000000, 1, 0);
        let msg = Message::new(
            "thread_01",
            "user_01",
            MessageSender::User,
            "Did Sarah send the contract?",
            None,
            hlc,
            Some("email_search".to_string()),
        );
        assert!(!msg.id.as_str().is_empty());
        assert_eq!(msg.sender, MessageSender::User);
        assert_eq!(msg.intent_tag.as_deref(), Some("email_search"));
    }

    #[test]
    fn test_user_fact_lifecycle() {
        let mut fact = UserFact::new(
            "user_01",
            "preference",
            "meeting_time",
            "before 11am",
            0.95,
            Some("msg_123".to_string()),
            1710000000,
        );
        assert!(!fact.tombstoned);
        fact.tombstoned = true;
        assert!(fact.tombstoned);
    }

    #[test]
    fn test_style_profile_canonical_json() {
        let style = UserStyleProfile {
            verbosity: 0.2,
            lowercase_preference: true,
            emoji_density: 0.8,
            formality: 0.1,
            slang_affinity: 0.9,
            language_dialect: "en-slang".to_string(),
        };
        let json = style.to_canonical_json();
        assert!(json.contains("\"lowercase_preference\":true"));
        assert!(json.contains("\"language_dialect\":\"en-slang\""));
    }
}
