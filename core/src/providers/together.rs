//! Together AI provider plugin for scanning Together AI API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Together AI API keys and configuration files.
pub struct TogetherPlugin;

impl ProviderPlugin for TogetherPlugin {
    fn name(&self) -> &str {
        "together"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Together AI keys typically start with "e5f" or are long random strings
        if key.starts_with("e5f") && key.len() >= 30 {
            0.90
        } else if key.len() >= 32 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.60
        } else {
        0.35
    }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Together AI base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Together AI base URL patterns
        let is_valid_url = instance.base_url.contains("together.xyz")
            || instance.base_url.contains("api.together");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Together AI base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Together AI models
        Ok(vec![
            "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo".to_string(),
            "meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo".to_string(),
            "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string(),
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
    fn test_together_plugin_name() {
        let plugin = TogetherPlugin;
        assert_eq!(plugin.name(), "together");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = TogetherPlugin;
        assert_eq!(plugin.confidence_score("e5f1234567890abcdef1234567890abcdef"), 0.90);
        assert_eq!(plugin.confidence_score("long-random-key-with-hyphens"), 0.60);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
