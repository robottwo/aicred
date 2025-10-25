//! LiteLLM provider plugin for scanning LiteLLM configuration and API keys.

use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, DiscoveredKey, ValueType};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning LiteLLM configuration files and API keys.
pub struct LiteLLMPlugin;

impl ProviderPlugin for LiteLLMPlugin {
    fn name(&self) -> &str {
        "litellm"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // LiteLLM keys are typically longer and more complex
        if key.len() >= 40 && key.contains('-') && key.chars().any(|c| c.is_uppercase()) {
            0.85 // High confidence for complex keys
        } else if key.len() >= 30 {
            0.85 // Medium-high confidence for longer keys (30+ chars)
        } else {
            0.50 // Lower confidence for shorter keys
        }
    }
}

impl LiteLLMPlugin {
    /// Extracts LiteLLM key from environment variable content.
    fn extract_from_env(&self, content: &str) -> Option<DiscoveredKey> {
        // Look for LiteLLM-specific environment variables
        let env_patterns = [
            r"(?i)LITELLM_API_KEY[\s]*=[\s]*([a-zA-Z0-9_-]+)",
            r#"(?i)LITELLM_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]+)['"]"#,
            r#"(?i)LITELLM_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]{30,})['"]"#,
        ];

        for pattern in &env_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(key_match) = cap.get(1) {
                        let key_value = key_match.as_str();
                        if self.is_valid_litellm_key(key_value) {
                            let confidence = self.confidence_score(key_value);
                            let confidence_enum = self.float_to_confidence(confidence);

                            return Some(DiscoveredKey::new(
                                "litellm".to_string(),
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

    /// Extracts provider-specific keys that LiteLLM might use.
    fn extract_provider_keys(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Look for common provider keys that LiteLLM proxies
        let provider_patterns = [
            (
                r"(?i)openai[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "openai",
                "sk-",
            ),
            (
                r"(?i)anthropic[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "anthropic",
                "sk-ant-",
            ),
            (
                r"(?i)huggingface[_\s]*token[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "huggingface",
                "hf_",
            ),
            (
                r"(?i)cohere[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "cohere",
                "",
            ),
            (
                r"(?i)ai21[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "ai21",
                "",
            ),
        ];

        for (pattern, provider, prefix) in &provider_patterns {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| Error::PluginError(format!("Invalid regex pattern: {}", e)))?;

            for cap in regex.captures_iter(content) {
                if let Some(key_match) = cap.get(1) {
                    let key_value = key_match.as_str().trim();

                    // Clean up the key value
                    let cleaned_key = key_value.trim_matches('\'').trim_matches('"').trim();

                    // Check if it matches the expected prefix
                    if prefix.is_empty() || cleaned_key.starts_with(prefix) {
                        let confidence = if prefix.is_empty() { 0.60 } else { 0.80 };
                        let confidence_enum = self.float_to_confidence(confidence);

                        let discovered_key = DiscoveredKey::new(
                            provider.to_string(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            confidence_enum,
                            cleaned_key.to_string(),
                        );

                        keys.push(discovered_key);
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Checks if a key is a valid LiteLLM API key format.
    fn is_valid_litellm_key(&self, key: &str) -> bool {
        // LiteLLM API keys must be at least 20 characters long
        if key.len() < 20 {
            return false;
        }

        // LiteLLM keys are typically alphanumeric with dashes/underscores
        key.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Converts float confidence to Confidence enum.
    fn float_to_confidence(&self, score: f32) -> Confidence {
        if score >= 0.80 {
            Confidence::High
        } else if score >= 0.60 {
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
    fn test_litellm_plugin_name() {
        let plugin = LiteLLMPlugin;
        assert_eq!(plugin.name(), "litellm");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = LiteLLMPlugin;

        // High confidence for complex keys
        assert_eq!(
            plugin.confidence_score("LL-ABCD-1234-EFGH-5678-IJKL-9012-MNOP"),
            0.85
        );
        assert_eq!(
            plugin.confidence_score("litellm-api-key-with-dashes-and-UPPERCASE"),
            0.85
        );

        // Medium confidence for longer keys
        assert_eq!(
            plugin.confidence_score("litellm-key-with-30-chars-exactly"),
            0.85
        );

        // Lower confidence for shorter keys
        assert_eq!(plugin.confidence_score("short-key-123"), 0.50);
    }

    #[test]
    fn test_valid_litellm_key_detection() {
        let plugin = LiteLLMPlugin;

        assert!(plugin.is_valid_litellm_key("LL-ABCD-1234-EFGH-5678-IJKL-9012-MNOP"));
        assert!(plugin.is_valid_litellm_key("litellm-api-key-with-dashes-and-underscores"));
        assert!(!plugin.is_valid_litellm_key("short"));
        assert!(!plugin.is_valid_litellm_key("invalid key with spaces"));
    }

    #[test]
    fn test_extract_provider_keys() {
        let plugin = LiteLLMPlugin;
        let content = "openai_api_key: sk-1234567890abcdef";
        let path = Path::new("test.yaml");

        let keys = plugin.extract_provider_keys(path, content).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].provider, "openai");
    }
}
