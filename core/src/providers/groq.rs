//! Groq provider plugin for scanning Groq API keys and configuration.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Groq API keys and configuration files.
pub struct GroqPlugin;

impl ProviderPlugin for GroqPlugin {
    fn name(&self) -> &'static str {
        "groq"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Groq keys have very specific patterns
        if key.starts_with("gsk_") {
            0.95 // Very distinctive Groq prefix
        } else if key.starts_with("gsk-") {
            0.95 // Alternative Groq prefix format
        } else if key.len() >= 40 && key.contains('_') {
            0.70 // Might be a Groq key without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;

        // Groq-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Groq base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Groq base URL patterns
        let is_valid_groq_url = instance.base_url.starts_with("https://api.groq.com")
            || instance.base_url.starts_with("https://groq.com");

        if !is_valid_groq_url {
            return Err(Error::PluginError(
                "Invalid Groq base URL. Expected format: https://api.groq.com".to_string(),
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_non_empty_api_key() {
            return Err(Error::PluginError(
                "Groq instance has models configured but no valid API keys".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.clone());
        }

        // Otherwise, return default Groq models based on instance configuration
        let mut models = vec![
            "llama3-8b-8192".to_string(),
            "llama3-70b-8192".to_string(),
            "mixtral-8x7b-32768".to_string(),
            "gemma-7b-it".to_string(),
            "gemma2-9b-it".to_string(),
        ];

        // If no valid keys, only return a subset of models
        if !instance.has_non_empty_api_key() {
            models.truncate(2); // Only return two models for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Groq requires both a valid base URL and at least one valid API key
        if !instance.has_non_empty_api_key() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;

        Ok(true)
    }
}

impl GroqPlugin {
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
    use crate::models::ProviderInstance;

    #[test]
    fn test_groq_plugin_name() {
        let plugin = GroqPlugin;
        assert_eq!(plugin.name(), "groq");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = GroqPlugin;

        assert_eq!(
            plugin.confidence_score("gsk_test1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("gsk-1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random_key_with_underscores_123456789"),
            0.30
        );
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = GroqPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://api.groq.com".to_string(),
        );

        // Set a valid API key directly on the instance
        instance.set_api_key("gsk_test1234567890abcdef1234567890abcdef".to_string());

        // Add a model
        instance.add_model("llama3-8b-8192".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = GroqPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://invalid-url.com".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Groq base URL"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = GroqPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://api.groq.com".to_string(),
        );

        // Add a model but no keys
        
        instance.add_model("llama3-8b-8192".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = GroqPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://api.groq.com".to_string(),
        );

        // Add models
        instance.add_model("llama3-8b-8192".to_string());
        instance.add_model("mixtral-8x7b-32768".to_string());

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"llama3-8b-8192".to_string()));
        assert!(model_list.contains(&"mixtral-8x7b-32768".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = GroqPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://api.groq.com".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 2); // Should return only two models when no valid keys
        assert!(models.contains(&"llama3-8b-8192".to_string()));
        assert!(models.contains(&"llama3-70b-8192".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = GroqPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-groq".to_string(),
            "Test Groq".to_string(),
            "groq".to_string(),
            "https://api.groq.com".to_string(),
        );

        // Without keys, should return false
        assert!(!plugin.is_instance_configured(&instance).unwrap());

        // Set a valid API key directly on the instance
        instance.set_api_key("gsk_test1234567890abcdef1234567890abcdef".to_string());

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }
}
