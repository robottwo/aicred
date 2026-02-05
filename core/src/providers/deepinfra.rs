//! DeepInfra provider plugin for scanning DeepInfra API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning DeepInfra API keys and configuration files.
pub struct DeepInfraPlugin;

impl ProviderPlugin for DeepInfraPlugin {
    fn name(&self) -> &str {
        "deepinfra"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // DeepInfra keys typically start with "Bearer " or are long random strings
        if key.starts_with("Bearer ") {
            0.90
        } else if key.len() >= 32 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.75
        } else {
            0.40
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "DeepInfra base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid DeepInfra base URL patterns
        let is_valid_url = instance.base_url.contains("deepinfra.com")
            || instance.base_url.contains("deepinfra.ai");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid DeepInfra base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default DeepInfra models
        Ok(vec![
            "meta-llama/Meta-Llama-3.1-70B-Instruct".to_string(),
            "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
            "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
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
    fn test_deepinfra_plugin_name() {
        let plugin = DeepInfraPlugin;
        assert_eq!(plugin.name(), "deepinfra");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = DeepInfraPlugin;
        assert_eq!(plugin.confidence_score("Bearer abc123"), 0.90);
        assert_eq!(
            plugin.confidence_score("long-random-key-with-hyphens"),
            0.75
        );
        assert_eq!(plugin.confidence_score("short"), 0.40);
    }
}
