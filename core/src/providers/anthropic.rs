//! Anthropic provider plugin for scanning Anthropic API keys and configuration.

use crate::error::{Error, Result};
use crate::models::{discovered_key::{Confidence, DiscoveredKey, ValueType}, ProviderInstance};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning Anthropic API keys and configuration files.
pub struct AnthropicPlugin;

impl ProviderPlugin for AnthropicPlugin {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Anthropic keys have very specific patterns
        if key.starts_with("sk-ant-") {
            0.95 // Very distinctive Anthropic prefix
        } else if key.len() >= 40 && key.contains('-') {
            0.70 // Might be an Anthropic key without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;
        
        // Anthropic-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError("Anthropic base URL cannot be empty".to_string()));
        }
        
        // Check for valid Anthropic base URL patterns
        let is_valid_anthropic_url = instance.base_url.starts_with("https://api.anthropic.com") ||
                                     instance.base_url.starts_with("https://api.anthropic.ai") ||
                                     instance.base_url.starts_with("https://claude-api.anthropic.com");
        
        if !is_valid_anthropic_url {
            return Err(Error::PluginError(
                "Invalid Anthropic base URL. Expected format: https://api.anthropic.com".to_string()
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_valid_keys() {
            return Err(Error::PluginError(
                "Anthropic instance has models configured but no valid API keys".to_string()
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Otherwise, return default Anthropic models based on instance configuration
        let mut models = vec![
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ];

        // If no valid keys, only return a subset of models
        if !instance.has_valid_keys() {
            models.truncate(1); // Only return one model for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Anthropic requires both a valid base URL and at least one valid API key
        if !instance.has_valid_keys() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;
        
        Ok(true)
    }
}

impl AnthropicPlugin {
    /// Helper method to perform base instance validation
    fn validate_base_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError("Base URL cannot be empty".to_string()));
        }
        if !instance.base_url.starts_with("http://") && !instance.base_url.starts_with("https://") {
            return Err(Error::PluginError("Base URL must start with http:// or https://".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{discovered_key::Confidence, ProviderInstance, ProviderKey, Environment, ValidationStatus};

    #[test]
    fn test_anthropic_plugin_name() {
        let plugin = AnthropicPlugin;
        assert_eq!(plugin.name(), "anthropic");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = AnthropicPlugin;

        assert_eq!(plugin.confidence_score("sk-ant-1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("sk-ant-1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random-key-with-dashes-123456789"),
            0.30
        );
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("sk-ant-test123".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // Add a model
        let model = crate::models::Model::new(
            "claude-3-sonnet".to_string(),
            instance.id.clone(),
            "Claude 3 Sonnet".to_string(),
        );
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = AnthropicPlugin;
        let instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://invalid-url.com".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Anthropic base URL"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        // Add a model but no keys
        let model = crate::models::Model::new(
            "claude-3-sonnet".to_string(),
            instance.id.clone(),
            "Claude 3 Sonnet".to_string(),
        );
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        // Add models
        let model1 = crate::models::Model::new(
            "claude-3-sonnet".to_string(),
            instance.id.clone(),
            "Claude 3 Sonnet".to_string(),
        );
        let model2 = crate::models::Model::new(
            "claude-3-opus".to_string(),
            instance.id.clone(),
            "Claude 3 Opus".to_string(),
        );
        instance.add_model(model1);
        instance.add_model(model2);

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"claude-3-sonnet".to_string()));
        assert!(models.contains(&"claude-3-opus".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = AnthropicPlugin;
        let instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 1); // Should return only one model when no valid keys
        assert!(models.contains(&"claude-3-sonnet-20240229".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new(
            "test-anthropic".to_string(),
            "Test Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        // Without keys, should return false
        assert!(!plugin.is_instance_configured(&instance).unwrap());

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("sk-ant-test123".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }
}
