//! Azure OpenAI provider plugin for scanning Azure OpenAI credentials.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Azure OpenAI credentials and configuration files.
pub struct AzurePlugin;

#[async_trait::async_trait]
impl ProviderPlugin for AzurePlugin {
    fn name(&self) -> &'static str {
        "azure"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Azure keys are typically 32-character hex strings
        if key.len() == 32 && key.chars().all(|c| c.is_ascii_hexdigit()) {
            0.90 // Very likely Azure key
        } else if key.len() >= 20 && key.chars().all(|c| c.is_ascii_alphanumeric()) {
            0.60 // Could be Azure key
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Azure OpenAI base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Azure base URL patterns
        let is_valid_azure_url = instance
            .base_url
            .contains(".openai.azure.com")
            || instance.base_url.contains(".azure-api.net");

        if !is_valid_azure_url {
            return Err(Error::PluginError(
                "Invalid Azure OpenAI base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        Ok(instance.has_non_empty_api_key())
    }

    async fn probe_models_async(
        &self,
        _api_key: &str,
        _base_url: Option<&str>,
    ) -> Result<Vec<crate::models::ModelMetadata>> {
        // Azure OpenAI models
        Ok(vec![
            crate::models::ModelMetadata {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                description: Some("Most capable multimodal model".to_string()),
                context_length: Some(128000),
                pricing: None,
                architecture: None,
                metadata: None,
            },
            crate::models::ModelMetadata {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                description: Some("High-performance GPT-4 model".to_string()),
                context_length: Some(128000),
                pricing: None,
                architecture: None,
                metadata: None,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_plugin_name() {
        let plugin = AzurePlugin;
        assert_eq!(plugin.name(), "azure");
    }

    #[test]
    fn test_azure_confidence_score() {
        let plugin = AzurePlugin;

        // Test Azure key format
        let score1 = plugin.confidence_score("d4d0a84e0a8944a29c8f5b2d7a4a5c2a");
        assert!(score1 > 0.8, "Expected score > 0.8, got {score1}");

        // Test generic key
        let score2 = plugin.confidence_score("sk-1234567890");
        assert!(score2 < 0.5, "Expected score < 0.5, got {score2}");
    }
}
