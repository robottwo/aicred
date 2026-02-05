//! Replicate provider plugin for scanning Replicate API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Replicate API keys and configuration files.
pub struct ReplicatePlugin;

impl ProviderPlugin for ReplicatePlugin {
    fn name(&self) -> &str {
        "replicate"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Replicate keys start with "r8_"
        if key.starts_with("r8_") && key.len() >= 30 {
            0.95
        } else if key.starts_with("r8") && key.len() >= 20 {
            0.70
        } else {
            0.35
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Replicate base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Replicate base URL patterns
        let is_valid_url = instance.base_url.contains("replicate.com")
            || instance.base_url.contains("api.replicate.com");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Replicate base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Replicate models
        Ok(vec![
            "meta/meta-llama-3.1-405b-instruct".to_string(),
            "meta/meta-llama-3.1-70b-instruct".to_string(),
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
    fn test_replicate_plugin_name() {
        let plugin = ReplicatePlugin;
        assert_eq!(plugin.name(), "replicate");
    
    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = ReplicatePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "replicate".to_string(),
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
        let plugin = ReplicatePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "replicate".to_string(),
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
        let plugin = ReplicatePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "replicate".to_string(),
            base_url: "https://api.replicate.com".to_string(),
            api_key: Some("test-key-12345678901234567890".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = ReplicatePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "replicate".to_string(),
            base_url: "https://api.replicate.com".to_string(),
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
        let plugin = ReplicatePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "replicate".to_string(),
            base_url: "https://api.replicate.com".to_string(),
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
        let plugin = ReplicatePlugin;
        assert_eq!(plugin.confidence_score("r8_1234567890abcdef1234567890abcdef"), 0.95);
        assert_eq!(plugin.confidence_score("r81234567890abcdef123456"), 0.70);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
