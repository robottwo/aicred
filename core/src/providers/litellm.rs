//! `LiteLLM` provider plugin for scanning `LiteLLM` configuration and API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning `LiteLLM` configuration files and API keys.
pub struct LiteLLMPlugin;

impl ProviderPlugin for LiteLLMPlugin {
    fn name(&self) -> &'static str {
        "litellm"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // LiteLLM keys are typically longer and more complex
        if key.len() >= 40 && key.contains('-') && key.chars().any(char::is_uppercase) {
            0.85 // High confidence for complex keys
        } else if key.len() >= 30 {
            0.85 // Medium-high confidence for longer keys (30+ chars)
        } else {
            0.50 // Lower confidence for shorter keys
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;

        // LiteLLM-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "LiteLLM base URL cannot be empty".to_string(),
            ));
        }

        // LiteLLM is flexible with base URLs since it can proxy to many providers
        // Just validate it's a valid HTTP(S) URL
        if !instance.base_url.starts_with("http://") && !instance.base_url.starts_with("https://") {
            return Err(Error::PluginError(
                "LiteLLM base URL must be a valid HTTP(S) URL".to_string(),
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_non_empty_api_key() {
            return Err(Error::PluginError(
                "LiteLLM instance has models configured but no valid API keys".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.clone());
        }

        // Otherwise, return default LiteLLM-supported models based on instance configuration
        let mut models = vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-opus-20240229".to_string(),
            "llama3-8b-8192".to_string(),
            "mixtral-8x7b-32768".to_string(),
        ];

        // If no valid keys, only return a subset of models
        if !instance.has_non_empty_api_key() {
            models.truncate(3); // Only return three models for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // LiteLLM requires both a valid base URL and at least one valid API key
        if !instance.has_non_empty_api_key() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;

        Ok(true)
    }

    fn initialize_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // LiteLLM-specific initialization logic
        // This could include testing connectivity, validating proxy configurations, etc.

        // For now, just validate the instance
        self.validate_instance(instance)?;

        // Additional LiteLLM-specific initialization could go here
        // such as testing the proxy endpoint, validating model access, etc.

        Ok(())
    }
}

impl LiteLLMPlugin {
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
    fn test_litellm_plugin_name() {
        let plugin = LiteLLMPlugin;
        assert_eq!(plugin.name(), "litellm");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = LiteLLMPlugin;

        // High confidence for complex keys
        assert_eq!(
            plugin.confidence_score("LL-ABCD-1234-EFGH-5678-IJKL-9012-MNOP"),
            0.85
        );
        assert_eq!(
            plugin.confidence_score("litellm-api-key-with-dashes-and-UPPERCASE"),
            0.85
        );

        // Medium confidence for longer keys
        assert_eq!(
            plugin.confidence_score("litellm-key-with-30-chars-exactly"),
            0.85
        );

        // Lower confidence for shorter keys
        assert_eq!(plugin.confidence_score("short-key-123"), 0.50);
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Set a valid API key directly on the instance
        instance.set_api_key("litellm-api-key-with-dashes-and-UPPERCASE".to_string());

        // Add a model
        instance.add_model("gpt-3.5-turbo".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = LiteLLMPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "not-a-url".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("must start with http:// or https://"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add a model but no keys
        
        instance.add_model("gpt-3.5-turbo".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add models
        instance.add_model("gpt-3.5-turbo".to_string());
        instance.add_model("claude-3-sonnet".to_string());

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"gpt-3.5-turbo".to_string()));
        assert!(model_list.contains(&"claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = LiteLLMPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 3); // Should return only three models when no valid keys
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"claude-3-sonnet-20240229".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Without keys, should return false
        assert!(!plugin.is_instance_configured(&instance).unwrap());

        // Set a valid API key directly on the instance
        instance.set_api_key("litellm-api-key-with-dashes-and-UPPERCASE".to_string());

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }

    #[test]
    fn test_initialize_instance() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Set a valid API key directly on the instance
        instance.set_api_key("litellm-api-key-with-dashes-and-UPPERCASE".to_string());

        let result = plugin.initialize_instance(&instance);
        assert!(result.is_ok());
    }
}
