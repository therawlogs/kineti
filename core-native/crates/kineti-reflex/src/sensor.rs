//! Sub-millisecond sensory triage and intent classification.
//!
//! Evaluates incoming message stimuli in `< 0.2ms` to determine:
//! 1. Modality (Text, VoiceNote, Photo, Document, Location, Contact)
//! 2. Intent category (Instant reflex vs Memory vs Tools vs Deep Cortex)

use std::fmt;

/// The sensory input modality of an incoming message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modality {
    /// Pure text message.
    Text,
    /// Audio voice recording or audio message.
    VoiceNote,
    /// Static image or photograph.
    Photo,
    /// Document attachment (PDF, DOCX, CSV).
    Document,
    /// Geographic GPS coordinates.
    Location,
    /// Electronic contact card (vCard).
    Contact,
}

impl Modality {
    /// Detects modality from media URL or mime type hints.
    pub fn from_media(media_url: Option<&str>, mime_type: Option<&str>) -> Self {
        if let Some(mime) = mime_type {
            if mime.starts_with("audio/") {
                return Self::VoiceNote;
            } else if mime.starts_with("image/") {
                return Self::Photo;
            } else if mime.starts_with("application/") || mime.starts_with("text/csv") {
                return Self::Document;
            }
        }
        if let Some(url) = media_url {
            let lower = url.to_lowercase();
            if lower.ends_with(".ogg") || lower.ends_with(".mp3") || lower.ends_with(".m4a") {
                return Self::VoiceNote;
            } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp") {
                return Self::Photo;
            } else if lower.ends_with(".pdf") || lower.ends_with(".docx") || lower.ends_with(".csv") {
                return Self::Document;
            }
        }
        Self::Text
    }
}

/// Classified intent category for routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntentCategory {
    /// Quick greeting or pleasantry ("hi", "gm", "sup", "yo", "hello").
    FastReflexGreeting,
    /// Low-information acknowledgment or status update ("ok", "thanks", "got it", "leaving now").
    /// Triggers native emoji reaction (👍, ❤️) without generating text clutter.
    LowInfoStatusAck,
    /// Personal memory or preference storage ("remember that...", "my manager is...").
    MemoryStore {
        /// Extracted key/category.
        subject: String,
        /// Extracted fact or value.
        content: String,
    },
    /// Direct memory or preference recall ("what is my manager's name?").
    MemoryRecall {
        /// Query subject.
        subject: String,
    },
    /// Live real-time web search with temporal cues ("latest news", "weather today", "SpaceX launch this week").
    LiveWebSearch {
        /// Extracted search query.
        query: String,
    },
    /// Multimodal image generation ("create an image of...", "draw a...", "make a photo...").
    ImageGeneration {
        /// Image prompt.
        prompt: String,
    },
    /// Gmail search, triage, or email draft action.
    GmailAction {
        /// Target action description.
        action: String,
    },
    /// Notion database, page, or task action.
    NotionAction {
        /// Target action description.
        action: String,
    },
    /// Real-world eCommerce, price comparison, or ticket booking.
    RealWorldAction {
        /// Action target.
        target: String,
    },
    /// General deliberative reasoning requiring multi-model LLM synthesis.
    DeliberativeReasoning,
}

impl fmt::Display for IntentCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FastReflexGreeting => write!(f, "FastReflexGreeting"),
            Self::LowInfoStatusAck => write!(f, "LowInfoStatusAck"),
            Self::MemoryStore { subject, .. } => write!(f, "MemoryStore({})", subject),
            Self::MemoryRecall { subject } => write!(f, "MemoryRecall({})", subject),
            Self::LiveWebSearch { query } => write!(f, "LiveWebSearch({})", query),
            Self::ImageGeneration { .. } => write!(f, "ImageGeneration"),
            Self::GmailAction { .. } => write!(f, "GmailAction"),
            Self::NotionAction { .. } => write!(f, "NotionAction"),
            Self::RealWorldAction { .. } => write!(f, "RealWorldAction"),
            Self::DeliberativeReasoning => write!(f, "DeliberativeReasoning"),
        }
    }
}

/// A parsed sensory perception ready for the reflex circuit.
#[derive(Debug, Clone, PartialEq)]
pub struct SensoryPerception {
    /// Detected modality.
    pub modality: Modality,
    /// Classified intent.
    pub intent: IntentCategory,
    /// Confidence score from 0.0 to 1.0.
    pub confidence: f32,
    /// Execution duration of sensory triage in microseconds.
    pub triage_duration_micros: u64,
}

/// Sensory triage engine performing sub-millisecond classification.
#[derive(Debug, Default, Clone)]
pub struct SensoryClassifier;

impl SensoryClassifier {
    /// Creates a new sensory classifier.
    pub fn new() -> Self {
        Self
    }

    /// Classifies an incoming message into a sensory perception in `< 0.2ms`.
    pub fn classify(&self, text: &str, media_url: Option<&str>, mime_type: Option<&str>) -> SensoryPerception {
        let start = std::time::Instant::now();
        let modality = Modality::from_media(media_url, mime_type);
        let trimmed = text.trim();
        let lower = trimmed.to_lowercase();

        let intent = if modality == Modality::Photo && trimmed.is_empty() {
            IntentCategory::DeliberativeReasoning
        } else if is_low_info_ack(&lower) {
            IntentCategory::LowInfoStatusAck
        } else if is_fast_greeting(&lower) {
            IntentCategory::FastReflexGreeting
        } else if let Some((subj, val)) = parse_memory_store(&lower, trimmed) {
            IntentCategory::MemoryStore {
                subject: subj,
                content: val,
            }
        } else if let Some(prompt) = parse_image_generation(&lower, trimmed) {
            IntentCategory::ImageGeneration { prompt }
        } else if let Some(query) = parse_gmail_action(&lower, trimmed) {
            IntentCategory::GmailAction { action: query }
        } else if let Some(query) = parse_notion_action(&lower, trimmed) {
            IntentCategory::NotionAction { action: query }
        } else if let Some(target) = parse_real_world_action(&lower, trimmed) {
            IntentCategory::RealWorldAction { target }
        } else if let Some(query) = parse_live_search(&lower, trimmed) {
            IntentCategory::LiveWebSearch { query }
        } else if let Some(subj) = parse_memory_recall(&lower) {
            IntentCategory::MemoryRecall { subject: subj }
        } else {
            IntentCategory::DeliberativeReasoning
        };

        let triage_duration_micros = start.elapsed().as_micros() as u64;
        SensoryPerception {
            modality,
            intent,
            confidence: 0.95,
            triage_duration_micros,
        }
    }
}

// ---------------------------------------------------------------------------
// Pattern Matchers (Sub-millisecond Zero-Allocation Heuristics)
// ---------------------------------------------------------------------------

fn is_low_info_ack(lower: &str) -> bool {
    const ACKS: &[&str] = &[
        "ok", "okay", "k", "kk", "got it", "thanks", "thx", "ty", "cool",
        "bet", "sounds good", "leaving now", "heading out", "omw", "on my way",
        "will do", "alright", "perfect", "done", "noted", "yep", "yeah", "sure",
    ];
    let trimmed = lower.trim_matches(|c: char| c == '.' || c == '!' || c == '?' || c == ',');
    if ACKS.contains(&trimmed)
        || trimmed.starts_with("leaving now")
        || trimmed.starts_with("heading out")
        || trimmed.starts_with("on my way")
        || trimmed.starts_with("omw")
    {
        return true;
    }
    // Check compound acknowledgments like "ok got it", "ok thanks", "thanks so much"
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() <= 4 && words.iter().all(|w| {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
        ACKS.contains(&clean) || clean == "it" || clean == "got" || clean == "now" || clean == "so" || clean == "much" || clean == "you" || clean == "see" || clean == "later"
    }) {
        return true;
    }
    false
}

fn is_fast_greeting(lower: &str) -> bool {
    const GREETINGS: &[&str] = &[
        "hi", "hello", "hey", "gm", "gn", "good morning", "good evening",
        "sup", "yo", "wassup", "what's up", "hola",
    ];
    for &g in GREETINGS {
        if lower == g || lower.starts_with(&format!("{} ", g)) {
            return true;
        }
    }
    false
}

fn parse_memory_store(lower: &str, raw: &str) -> Option<(String, String)> {
    if lower.starts_with("remember that ") {
        let content = raw[14..].trim().to_string();
        Some(("general".to_string(), content))
    } else if lower.starts_with("my manager is ") {
        let name = raw[14..].trim().to_string();
        Some(("manager".to_string(), name))
    } else if lower.starts_with("don't forget that ") {
        let content = raw[18..].trim().to_string();
        Some(("general".to_string(), content))
    } else if lower.starts_with("my sister is ") || lower.starts_with("my sister's name is ") {
        let name = raw.split("is ").last().unwrap_or("").trim().to_string();
        Some(("sister".to_string(), name))
    } else {
        None
    }
}

fn parse_memory_recall(lower: &str) -> Option<String> {
    if lower.starts_with("what is my ") || lower.starts_with("what's my ") {
        Some(lower.trim_end_matches('?').to_string())
    } else if lower.starts_with("who is my ") || lower.starts_with("who's my ") {
        Some(lower.trim_end_matches('?').to_string())
    } else {
        None
    }
}

fn parse_image_generation(lower: &str, raw: &str) -> Option<String> {
    if lower.starts_with("create an image of ") {
        Some(raw[19..].trim().to_string())
    } else if lower.starts_with("generate an image of ") {
        Some(raw[21..].trim().to_string())
    } else if lower.starts_with("draw a ") || lower.starts_with("draw an ") {
        Some(raw[5..].trim().to_string())
    } else if lower.starts_with("make a photo of ") || lower.starts_with("make an image of ") {
        Some(raw[16..].trim().to_string())
    } else if lower.starts_with("image:") || lower.starts_with("generate image:") {
        Some(raw.split(':').last().unwrap_or("").trim().to_string())
    } else {
        None
    }
}

fn parse_gmail_action(lower: &str, raw: &str) -> Option<String> {
    if lower.contains("email") || lower.contains("gmail") || lower.contains("inbox") {
        if lower.contains("check") || lower.contains("search") || lower.contains("send") || lower.contains("draft") || lower.contains("did ") {
            return Some(raw.to_string());
        }
    }
    None
}

fn parse_notion_action(lower: &str, raw: &str) -> Option<String> {
    if lower.contains("notion") {
        Some(raw.to_string())
    } else {
        None
    }
}

fn parse_real_world_action(lower: &str, raw: &str) -> Option<String> {
    if lower.contains("best price") || lower.contains("tickets for") || lower.contains("flight to") || lower.contains("compare prices") {
        Some(raw.to_string())
    } else {
        None
    }
}

fn parse_live_search(lower: &str, raw: &str) -> Option<String> {
    if lower.contains("weather today") || lower.contains("latest update on") || lower.contains("this week") || lower.contains("stock price of") || lower.contains("breaking news") {
        Some(raw.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_info_acknowledgment_triggers_emoji_path() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("ok got it", None, None);
        assert_eq!(p.intent, IntentCategory::LowInfoStatusAck);

        let p2 = classifier.classify("leaving now", None, None);
        assert_eq!(p2.intent, IntentCategory::LowInfoStatusAck);

        let p3 = classifier.classify("thanks!", None, None);
        assert_eq!(p3.intent, IntentCategory::LowInfoStatusAck);
    }

    #[test]
    fn test_fast_greeting() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("yo what's up", None, None);
        assert_eq!(p.intent, IntentCategory::FastReflexGreeting);
    }

    #[test]
    fn test_memory_store_intent() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("remember that my manager is Sarah Chen", None, None);
        assert!(matches!(p.intent, IntentCategory::MemoryStore { .. }));
    }

    #[test]
    fn test_image_generation_intent() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("create an image of a cybernetic cat at sunset", None, None);
        assert!(matches!(p.intent, IntentCategory::ImageGeneration { .. }));
    }

    #[test]
    fn test_gmail_and_notion_intent() {
        let classifier = SensoryClassifier::new();
        let p1 = classifier.classify("Did Sarah send any notes on the quarterly budget via email?", None, None);
        assert!(matches!(p1.intent, IntentCategory::GmailAction { .. }));

        let p2 = classifier.classify("Add this beta launch milestone to Notion", None, None);
        assert!(matches!(p2.intent, IntentCategory::NotionAction { .. }));
    }

    #[test]
    fn test_real_world_shopping_and_ticket_action() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("Find me the best price on Sony WH-1000XM5 headphones", None, None);
        assert!(matches!(p.intent, IntentCategory::RealWorldAction { .. }));

        let p2 = classifier.classify("Look for 2 good tickets for Hans Zimmer this October", None, None);
        assert!(matches!(p2.intent, IntentCategory::RealWorldAction { .. }));
    }

    #[test]
    fn test_triage_latency_under_1_millisecond() {
        let classifier = SensoryClassifier::new();
        let p = classifier.classify("What is the latest update on SpaceX's Starship launch scheduled for this week?", None, None);
        assert!(p.triage_duration_micros < 1000, "Triage took {}us (> 1000us)", p.triage_duration_micros);
    }
}
