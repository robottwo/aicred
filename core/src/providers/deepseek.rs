//! DeepSeek provider plugin for scanning DeepSeek API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning DeepSeek API keys and configuration files.
pub struct DeepSeekPlugin;

impl ProviderPlugin for DeepSeekPlugin {
    fn name(&self) -> &str {
        "deepseek"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // DeepSeek keys typically follow a specific pattern
        if key.starts_with("sk-") && key.len() >= 40 {
            0.90
        } else if key.len() >= 32 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.65
        } else {
            0.35
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "DeepSeek base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid DeepSeek base URL patterns
        let is_valid_url = instance.base_url.contains("deepseek.com")
            || instance.base_url.contains("deepseek.ai");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid DeepSeek base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default DeepSeek models
        Ok(vec![
            "deepseek-chat".to_string(),
            "deepseek-coder".to_string(),
        ])
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        Ok(instance.has_non_empty_api_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_plugin_name() {
        let plugin = DeepSeekPlugin;
        assert_eq!(plugin.name(), "deepseek");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = DeepSeekPlugin;
        assert_eq!(plugin.confidence_score("sk-1234567890abcdef1234567890abcdef123456"), 0.90);
        assert_eq!(plugin.confidence_score("long-random-key-with-hyphens"), 0.65);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
