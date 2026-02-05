//! Hugging Face provider plugin for scanning Hugging Face tokens and configuration.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Hugging Face tokens and configuration files.
pub struct HuggingFacePlugin;

impl ProviderPlugin for HuggingFacePlugin {
    fn name(&self) -> &'static str {
        "huggingface"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Hugging Face tokens have very specific patterns
        if key.starts_with("hf_") {
            0.95 // Very distinctive Hugging Face prefix
        } else if key.len() >= 40 && key.contains('_') {
            0.70 // Might be a Hugging Face token without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;

        // Hugging Face-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Hugging Face base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Hugging Face base URL patterns
        let is_valid_hf_url = instance.base_url.starts_with("https://huggingface.co")
            || instance
                .base_url
                .starts_with("https://api-inference.huggingface.co")
            || instance.base_url.starts_with("https://huggingface.co/api");

        if !is_valid_hf_url {
            return Err(Error::PluginError(
                "Invalid Hugging Face base URL. Expected format: https://huggingface.co"
                    .to_string(),
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_non_empty_api_key() {
            return Err(Error::PluginError(
                "Hugging Face instance has models configured but no valid API tokens".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.clone());
        }

        // Otherwise, return default Hugging Face models based on instance configuration
        let mut models = vec![
            "microsoft/DialoGPT-medium".to_string(),
            "facebook/blenderbot-400M-distill".to_string(),
            "microsoft/DialoGPT-small".to_string(),
            "facebook/blenderbot-1B-distill".to_string(),
        ];

        // If no valid keys, only return a subset of models
        if !instance.has_non_empty_api_key() {
            models.truncate(2); // Only return two models for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Hugging Face requires both a valid base URL and at least one valid API token
        if !instance.has_non_empty_api_key() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;

        Ok(true)
    }
}

impl HuggingFacePlugin {
    /// Helper method to perform base instance validation
    fn validate_base_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError("Base URL cannot be empty".to_string()));
        }
        if !instance.base_url.starts_with("http://") && !instance.base_url.starts_with("https://") {
            return Err(Error::PluginError(
                "Base URL must start with http:// or https://".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        discovered_key::Confidence, provider_key::{Environment, ValidationStatus}, ProviderInstance, ProviderKey,
    };

    #[test]
    fn test_huggingface_plugin_name() {
        let plugin = HuggingFacePlugin;
        assert_eq!(plugin.name(), "huggingface");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = HuggingFacePlugin;

        assert_eq!(plugin.confidence_score("hf_1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("hf_1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random_key_with_underscores_123456789"),
            0.30
        );
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = HuggingFacePlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://huggingface.co".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("hf_test1234567890abcdef".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.set_api_key(key.value.unwrap_or_default());

        // Add a model
        
        instance.add_model("microsoft/DialoGPT-medium".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = HuggingFacePlugin;
        let instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://invalid-url.com".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Hugging Face base URL"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = HuggingFacePlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://huggingface.co".to_string(),
        );

        // Add a model but no keys
        
        instance.add_model("microsoft/DialoGPT-medium".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API tokens"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = HuggingFacePlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://huggingface.co".to_string(),
        );

        // Add models
        
        
        instance.add_model("microsoft/DialoGPT-medium".to_string());
        instance.add_model("facebook/blenderbot-400M-distill".to_string());

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"microsoft/DialoGPT-medium".to_string()));
        assert!(model_list.contains(&"facebook/blenderbot-400M-distill".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = HuggingFacePlugin;
        let instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://huggingface.co".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 2); // Should return only two models when no valid keys
        assert!(models.contains(&"microsoft/DialoGPT-medium".to_string()));
        assert!(models.contains(&"facebook/blenderbot-400M-distill".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = HuggingFacePlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-hf".to_string(),
            "Test Hugging Face".to_string(),
            "huggingface".to_string(),
            "https://huggingface.co".to_string(),
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
        key.value = Some("hf_test1234567890abcdef".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.set_api_key(key.value.unwrap_or_default());

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }
}
