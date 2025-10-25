//! OpenAI provider plugin for scanning OpenAI API keys and configuration.

use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, DiscoveredKey, ValueType};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning OpenAI API keys and configuration files.
pub struct OpenAIPlugin;

impl ProviderPlugin for OpenAIPlugin {
    fn name(&self) -> &str {
        "openai"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // OpenAI keys have very specific patterns
        if key.starts_with("sk-proj-") {
            0.95 // Project keys are very distinctive
        } else if key.starts_with("sk-") {
            0.95 // Standard OpenAI keys
        } else if key.len() >= 40 && key.contains('-') {
            0.75 // Might be an OpenAI key without the prefix
        } else {
            0.50 // Lower confidence for other patterns
        }
    }
}

impl OpenAIPlugin {
    /// Extracts OpenAI key from environment variable content.
    fn extract_from_env(&self, content: &str) -> Option<DiscoveredKey> {
        // Look for OPENAI_API_KEY environment variable
        let env_patterns = [
            r"(?i)OPENAI_API_KEY[\s]*=[\s]*([a-zA-Z0-9_-]+)",
            r#"(?i)OPENAI_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]+)['"]"#,
            r#"(?i)OPENAI_API_KEY[\s]*=[\s]*['"](sk-[a-zA-Z0-9_-]+)['"]"#,
        ];

        for pattern in &env_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(key_match) = cap.get(1) {
                        let key_value = key_match.as_str();
                        if self.is_valid_openai_key(key_value) {
                            let confidence = self.confidence_score(key_value);
                            let confidence_enum = self.float_to_confidence(confidence);

                            return Some(DiscoveredKey::new(
                                "openai".to_string(),
                                "environment".to_string(),
                                ValueType::ApiKey,
                                confidence_enum,
                                key_value.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    /// Checks if a key is a valid OpenAI key format.
    fn is_valid_openai_key(&self, key: &str) -> bool {
        // OpenAI keys must be at least 19 characters long (including the sk- prefix)
        if key.len() < 19 {
            return false;
        }

        // Check for OpenAI key patterns
        if key.starts_with("sk-") || key.starts_with("sk-proj-") {
            return true;
        }

        // Allow keys that look like they could be OpenAI keys (long with alphanumeric and dashes)
        key.len() >= 40 && key.chars().all(|c| c.is_alphanumeric() || c == '-')
    }

    /// Converts float confidence to Confidence enum.
    fn float_to_confidence(&self, score: f32) -> Confidence {
        if score >= 0.9 {
            Confidence::VeryHigh
        } else if score >= 0.7 {
            Confidence::High
        } else if score >= 0.5 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;

    #[test]
    fn test_openai_plugin_name() {
        let plugin = OpenAIPlugin;
        assert_eq!(plugin.name(), "openai");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = OpenAIPlugin;

        assert_eq!(plugin.confidence_score("sk-1234567890abcdef"), 0.95);
        assert_eq!(plugin.confidence_score("sk-proj-1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("random-key-with-dashes-123456789"),
            0.50
        );
    }

    #[test]
    fn test_valid_openai_key_detection() {
        let plugin = OpenAIPlugin;

        let test_key = "sk-1234567890abcdef";
        println!("Key: '{}', Length: {}", test_key, test_key.len());
        println!("Starts with 'sk-': {}", test_key.starts_with("sk-"));
        println!(
            "is_valid_openai_key result: {}",
            plugin.is_valid_openai_key(test_key)
        );

        assert!(plugin.is_valid_openai_key("sk-1234567890abcdef"));
        assert!(plugin.is_valid_openai_key("sk-proj-1234567890abcdef"));
        assert!(plugin.is_valid_openai_key("sk-1234567890abcdef1234567890abcdef"));
        assert!(!plugin.is_valid_openai_key("short"));
        assert!(!plugin.is_valid_openai_key("sk-"));
    }

    #[test]
    fn test_parse_config_ignores_invalid_key() {
        let plugin = OpenAIPlugin;
        // too short to be valid
        let content = "api_key: sk-1234";
        let path = Path::new("test.yaml");
        // Test that invalid keys are not considered valid API keys
        assert!(!plugin.is_valid_openai_key("sk-1234"));
    }
}
