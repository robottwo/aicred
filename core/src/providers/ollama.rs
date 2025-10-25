//! Ollama provider plugin for scanning Ollama configuration.

use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, DiscoveredKey, ValueType};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning Ollama configuration files.
pub struct OllamaPlugin;

impl ProviderPlugin for OllamaPlugin {
    fn name(&self) -> &str {
        "ollama"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Ollama configuration is less critical than API keys, so lower confidence
        if key.starts_with("http://") || key.starts_with("https://") {
            0.85 // Server URL
        } else if key.contains('/') && key.len() >= 7 {
            0.80 // Model name (e.g., "llama2/7b")
        } else {
            0.70 // Other configuration
        }
    }
}

impl OllamaPlugin {
    /// Extracts Ollama configuration from environment variable content.
    fn extract_from_env(&self, content: &str) -> Option<DiscoveredKey> {
        // Look for OLLAMA_HOST environment variable
        let env_patterns = [
            r"(?i)OLLAMA_HOST[\s]*=[\s]*([a-zA-Z0-9._:/-]+)",
            r#"(?i)OLLAMA_HOST[\s]*=[\s]*['"]([a-zA-Z0-9._:/-]+)['"]"#,
        ];

        for pattern in &env_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(host_match) = cap.get(1) {
                        let host_value = host_match.as_str();

                        let confidence = self.confidence_score(host_value);
                        let confidence_enum = self.float_to_confidence(confidence);

                        return Some(DiscoveredKey::new(
                            "ollama".to_string(),
                            "environment".to_string(),
                            ValueType::Custom("ServerURL".to_string()),
                            confidence_enum,
                            host_value.to_string(),
                        ));
                    }
                }
            }
        }

        None
    }

    /// Extracts Ollama server URL from configuration content.
    fn extract_server_url(&self, content: &str, path: &Path) -> Option<DiscoveredKey> {
        let url_patterns = [
            r#"(?i)server[\s]*[:=][\s]*['"]([a-zA-Z0-9._:/-]+)['"]"#,
            r#"(?i)host[\s]*[:=][\s]*['"]([a-zA-Z0-9._:/-]+)['"]"#,
            r#"(?i)url[\s]*[:=][\s]*['"]([a-zA-Z0-9._:/-]+)['"]"#,
        ];

        for pattern in &url_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(url_match) = cap.get(1) {
                        let url_value = url_match.as_str();

                        let confidence = self.confidence_score(url_value);
                        let confidence_enum = self.float_to_confidence(confidence);

                        return Some(DiscoveredKey::new(
                            "ollama".to_string(),
                            path.display().to_string(),
                            ValueType::Custom("ServerURL".to_string()),
                            confidence_enum,
                            url_value.to_string(),
                        ));
                    }
                }
            }
        }

        None
    }

    /// Converts float confidence to Confidence enum.
    fn float_to_confidence(&self, score: f32) -> Confidence {
        if score >= 0.85 {
            Confidence::High
        } else if score >= 0.70 {
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
    fn test_ollama_plugin_name() {
        let plugin = OllamaPlugin;
        assert_eq!(plugin.name(), "ollama");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = OllamaPlugin;

        assert_eq!(plugin.confidence_score("http://localhost:11434"), 0.85);
        assert_eq!(plugin.confidence_score("https://ollama.example.com"), 0.85);
        assert_eq!(plugin.confidence_score("llama2/7b"), 0.80);
        assert_eq!(plugin.confidence_score("some-config-value"), 0.70);
    }
}
