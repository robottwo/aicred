//! Fireworks AI provider plugin for scanning Fireworks API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Fireworks AI API keys and configuration files.
pub struct FireworksPlugin;

impl ProviderPlugin for FireworksPlugin {
    fn name(&self) -> &str {
        "fireworks"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Fireworks keys typically start with "fw_" or are long random strings
        if key.starts_with("fw_") {
            0.95
        } else if key.len() >= 32 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.60
        } else {
            0.30
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Fireworks base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Fireworks base URL patterns
        let is_valid_url = instance.base_url.contains("fireworks.ai")
            || instance.base_url.contains("fireworks.com");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Fireworks base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Fireworks models
        Ok(vec![
            "accounts/fireworks/models/llama-v3p1-70b-instruct".to_string(),
            "accounts/fireworks/models/mixtral-8x7b-instruct".to_string(),
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
    fn test_fireworks_plugin_name() {
        let plugin = FireworksPlugin;
        assert_eq!(plugin.name(), "fireworks");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = FireworksPlugin;
        assert_eq!(plugin.confidence_score("fw_1234567890abcdef"), 0.95);
        assert_eq!(plugin.confidence_score("long-random-key-with-hyphens"), 0.60);
        assert_eq!(plugin.confidence_score("short"), 0.30);
    }
}
