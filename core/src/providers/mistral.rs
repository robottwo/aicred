//! Mistral AI provider plugin for scanning Mistral API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Mistral AI API keys and configuration files.
pub struct MistralPlugin;

impl ProviderPlugin for MistralPlugin {
    fn name(&self) -> &str {
        "mistral"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Mistral keys typically don't have a prefix but follow specific patterns
        if key.len() == 32 && key.chars().all(|c| c.is_alphanumeric()) {
            0.95
        } else if key.len() >= 24 && key.len() <= 40 && key.chars().all(|c| c.is_alphanumeric()) {
            0.65
        } else {
            0.35
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Mistral base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Mistral base URL patterns
        let is_valid_url = instance.base_url.contains("api.mistral.ai")
            || instance.base_url.contains("mistral.ai");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Mistral base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Mistral models
        Ok(vec![
            "mistral-large-latest".to_string(),
            "mistral-medium-latest".to_string(),
            "mistral-small-latest".to_string(),
            "open-mistral-7b".to_string(),
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
    fn test_mistral_plugin_name() {
        let plugin = MistralPlugin;
        assert_eq!(plugin.name(), "mistral");
    
    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = MistralPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "mistral".to_string(),
            base_url: "".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_instance_invalid_url() {
        let plugin = MistralPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "mistral".to_string(),
            base_url: "https://example.com".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_instance_valid() {
        let plugin = MistralPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "mistral".to_string(),
            base_url: "https://api.mistral.ai".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = MistralPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "mistral".to_string(),
            base_url: "https://api.mistral.ai".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_instance_configured_without_key() {
        let plugin = MistralPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "mistral".to_string(),
            base_url: "https://api.mistral.ai".to_string(),
            api_key: None,
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

    #[test]
    fn test_confidence_scoring() {
        let plugin = MistralPlugin;
        // 32 character alphanumeric key
        let key_32 = "abcd1234efgh5678ijkl9012mnop3456";
        assert_eq!(plugin.confidence_score(key_32), 0.95);
        assert_eq!(plugin.confidence_score("alphanumericstring24chars"), 0.65);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
