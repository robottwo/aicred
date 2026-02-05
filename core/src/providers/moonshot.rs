//! Moonshot AI provider plugin for scanning Moonshot API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Moonshot AI API keys and configuration files.
pub struct MoonshotPlugin;

impl ProviderPlugin for MoonshotPlugin {
    fn name(&self) -> &str {
        "moonshot"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Moonshot keys typically start with "sk-" or are long random strings
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
                "Moonshot base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Moonshot base URL patterns
        let is_valid_url = instance.base_url.contains("moonshot.cn")
            || instance.base_url.contains("moonshot.ai");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Moonshot base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Moonshot models
        Ok(vec![
            "moonshot-v1-8k".to_string(),
            "moonshot-v1-32k".to_string(),
            "moonshot-v1-128k".to_string(),
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
    fn test_moonshot_plugin_name() {
        let plugin = MoonshotPlugin;
        assert_eq!(plugin.name(), "moonshot");
    
    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = MoonshotPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "moonshot".to_string(),
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
        let plugin = MoonshotPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "moonshot".to_string(),
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
        let plugin = MoonshotPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "moonshot".to_string(),
            base_url: "https://api.moonshot.cn".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = MoonshotPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "moonshot".to_string(),
            base_url: "https://api.moonshot.cn".to_string(),
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
        let plugin = MoonshotPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "moonshot".to_string(),
            base_url: "https://api.moonshot.cn".to_string(),
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
        let plugin = MoonshotPlugin;
        assert_eq!(plugin.confidence_score("sk-1234567890abcdef1234567890abcdef123456"), 0.90);
        assert_eq!(plugin.confidence_score("long-random-key-with-hyphens"), 0.65);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
