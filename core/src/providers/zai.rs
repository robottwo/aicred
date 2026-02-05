//! ZAI provider plugin for scanning ZAI API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning ZAI API keys and configuration files.
pub struct ZAIPlugin;

impl ProviderPlugin for ZAIPlugin {
    fn name(&self) -> &str {
        "zai"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // ZAI keys typically follow specific patterns
        if key.starts_with("zai-") && key.len() >= 30 {
            0.90
        } else if key.starts_with("sk-") && key.len() >= 32 {
            0.75
        } else if key.len() >= 32 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.55
        } else {
            0.30
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "ZAI base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid ZAI base URL patterns
        let is_valid_url = instance.base_url.contains("z.ai")
            || instance.base_url.contains("api.z.ai");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid ZAI base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default ZAI models
        Ok(vec![
            "zai-1".to_string(),
            "zai-1-turbo".to_string(),
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
    fn test_zai_plugin_name() {
        let plugin = ZAIPlugin;
        assert_eq!(plugin.name(), "zai");
    
    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = ZAIPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "zai".to_string(),
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
        let plugin = ZAIPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "zai".to_string(),
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
        let plugin = ZAIPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "zai".to_string(),
            base_url: "https://api.zai.tools".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = ZAIPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "zai".to_string(),
            base_url: "https://api.zai.tools".to_string(),
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
        let plugin = ZAIPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "zai".to_string(),
            base_url: "https://api.zai.tools".to_string(),
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
        let plugin = ZAIPlugin;
        assert_eq!(plugin.confidence_score("zai-1234567890abcdef1234567890abcdef"), 0.90);
        assert_eq!(plugin.confidence_score("sk-1234567890abcdef1234567890abcdef"), 0.75);
        assert_eq!(plugin.confidence_score("long-random-key-with-hyphens"), 0.55);
        assert_eq!(plugin.confidence_score("short"), 0.30);
    }
}
