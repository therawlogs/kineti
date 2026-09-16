//! User communication style analyzer and dynamic tone calibrator.
//!
//! Analyzes user messages across 5 dimensions:
//! - Lowercase preference (e.g. "yo can u check..." -> lowercase output)
//! - Brevity / Verbosity (terse one-liners vs structured paragraphs)
//! - Slang & Acronym density (ngl, tbh, bet, w, fr, yo, bhai, yaar)
//! - Formality (formal greetings, complete punctuation vs casual)
//! - Emoji affinity (emoji-rich celebratory vs clean minimal text)

use kineti_core::conversation::UserStyleProfile;

/// Dynamic style analyzer that updates user profile from incoming messages.
#[derive(Debug, Default, Clone)]
pub struct StyleAnalyzer;

impl StyleAnalyzer {
    /// Creates a new style analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a single message and returns an updated style profile.
    pub fn update_profile(&self, current: &UserStyleProfile, incoming_text: &str) -> UserStyleProfile {
        let trimmed = incoming_text.trim();
        if trimmed.is_empty() {
            return current.clone();
        }

        let is_all_lowercase = trimmed.chars().any(|c| c.is_alphabetic())
            && !trimmed.chars().any(|c| c.is_uppercase());

        let emoji_count = trimmed.chars().filter(|&c| is_emoji(c)).count();
        let char_count = trimmed.chars().count();
        let message_emoji_density = if char_count > 0 {
            (emoji_count as f32 / char_count as f32).min(1.0)
        } else {
            0.0
        };

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let word_count = words.len();

        let slang_tokens = ["ngl", "tbh", "bet", "w", "fr", "yo", "sup", "bhai", "yaar", "vamos", "kinda", "gonna"];
        let slang_count = words
            .iter()
            .filter(|w| {
                let clean = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                slang_tokens.contains(&clean.as_str())
            })
            .count();
        let message_slang_affinity = (slang_count as f32 / (word_count.max(1) as f32) * 3.0).min(1.0);

        let formal_markers = ["dear", "sincerely", "regarding", "please", "kindly", "furthermore", "apologies"];
        let formal_count = words
            .iter()
            .filter(|w| {
                let clean = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                formal_markers.contains(&clean.as_str())
            })
            .count();
        let message_formality = if formal_count > 0 || (!is_all_lowercase && trimmed.ends_with('.')) {
            0.8
        } else if is_all_lowercase || message_slang_affinity > 0.2 {
            0.2
        } else {
            0.5
        };

        let detected_dialect = if words.iter().any(|w| {
            let c = w.to_lowercase();
            c == "bhai" || c == "yaar" || c == "karo" || c == "hai" || c == "thoda"
        }) {
            "hi-en".to_string()
        } else if message_slang_affinity > 0.3 {
            "en-slang".to_string()
        } else {
            "en".to_string()
        };

        // Exponential moving average update (alpha = 0.25)
        let alpha = 0.25;
        UserStyleProfile {
            verbosity: current.verbosity * (1.0 - alpha) + (word_count as f32 / 50.0).min(1.0) * alpha,
            lowercase_preference: if is_all_lowercase { true } else { current.lowercase_preference },
            emoji_density: current.emoji_density * (1.0 - alpha) + message_emoji_density * alpha,
            formality: current.formality * (1.0 - alpha) + message_formality * alpha,
            slang_affinity: current.slang_affinity * (1.0 - alpha) + message_slang_affinity * alpha,
            language_dialect: detected_dialect,
        }
    }

    /// Calibrates an output response to match the user's style profile.
    pub fn calibrate_output(&self, profile: &UserStyleProfile, raw_text: &str) -> String {
        let mut output = raw_text.to_string();

        // 1. If user has strong lowercase preference (casual Gen-Z/slang vibe), lowercase the response
        if profile.lowercase_preference && profile.formality < 0.6 {
            output = output.to_lowercase();
        }

        // 2. If user is extremely terse/executive, strip opening pleasantries
        if profile.verbosity < 0.3 && profile.formality >= 0.5 {
            if output.starts_with("Sure, ") || output.starts_with("Certainly! ") {
                if let Some(rest) = output.splitn(2, ' ').nth(1) {
                    output = rest.to_string();
                }
            }
        }

        output
    }
}

fn is_emoji(c: char) -> bool {
    matches!(
        c,
        '\u{1F300}'..='\u{1F5FF}' | // Symbols & Pictographs
        '\u{1F600}'..='\u{1F64F}' | // Emoticons
        '\u{1F680}'..='\u{1F6FF}' | // Transport & Map
        '\u{1F900}'..='\u{1F9FF}' | // Supplemental Symbols
        '\u{2600}'..='\u{26FF}'   | // Misc Symbols
        '\u{2700}'..='\u{27BF}'     // Dingbats
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase_and_slang_detection() {
        let analyzer = StyleAnalyzer::new();
        let current = UserStyleProfile::default();
        let updated = analyzer.update_profile(&current, "yo can u check if sarah sent the deck ngl kinda stressing");

        assert!(updated.lowercase_preference);
        assert!(updated.slang_affinity > 0.2);
        assert_eq!(updated.language_dialect, "en-slang");

        // Verify calibrated output is lowercased
        let raw = "Sarah sent the slides at 4:15 PM.";
        let calibrated = analyzer.calibrate_output(&updated, raw);
        assert_eq!(calibrated, "sarah sent the slides at 4:15 pm.");
    }

    #[test]
    fn test_executive_formal_detection() {
        let analyzer = StyleAnalyzer::new();
        let current = UserStyleProfile::default();
        let updated = analyzer.update_profile(&current, "Good morning. Please review the quarterly financial deck.");

        assert!(!updated.lowercase_preference);
        assert!(updated.formality > 0.5);
    }

    #[test]
    fn test_bilingual_code_switching_detection() {
        let analyzer = StyleAnalyzer::new();
        let current = UserStyleProfile::default();
        let updated = analyzer.update_profile(&current, "bhai check karo if the meeting is still on thoda urgent hai");

        assert_eq!(updated.language_dialect, "hi-en");
    }
}
