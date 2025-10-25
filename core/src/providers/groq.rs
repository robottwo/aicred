//! Groq provider plugin for scanning Groq API keys and configuration.

use crate::plugins::ProviderPlugin;

/// Plugin for scanning Groq API keys and configuration files.
pub struct GroqPlugin;

impl ProviderPlugin for GroqPlugin {
    fn name(&self) -> &str {
        "groq"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Groq keys have very specific patterns
        if key.starts_with("gsk_") {
            0.95 // Very distinctive Groq prefix
        } else if key.starts_with("gsk-") {
            0.95 // Alternative Groq prefix format
        } else if key.len() >= 40 && key.contains('_') {
            0.70 // Might be a Groq key without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_plugin_name() {
        let plugin = GroqPlugin;
        assert_eq!(plugin.name(), "groq");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = GroqPlugin;

        assert_eq!(
            plugin.confidence_score("gsk_test1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("gsk-1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random_key_with_underscores_123456789"),
            0.30
        );
    }
}
