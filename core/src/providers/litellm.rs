//! `LiteLLM` provider plugin for scanning `LiteLLM` configuration and API keys.

use crate::error::{Error, Result};
use crate::models::{
    discovered_key::{Confidence, DiscoveredKey, ValueType},
    ProviderInstance,
};
use crate::plugins::ProviderPlugin;
use std::path::Path;

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
        if !instance.models.is_empty() && !instance.has_valid_keys() {
            return Err(Error::PluginError(
                "LiteLLM instance has models configured but no valid API keys".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
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
        if !instance.has_valid_keys() {
            models.truncate(3); // Only return three models for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // LiteLLM requires both a valid base URL and at least one valid API key
        if !instance.has_valid_keys() {
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
    /// Extracts `LiteLLM` key from environment variable content.
    fn extract_from_env(&self, content: &str) -> Option<DiscoveredKey> {
        // Look for LiteLLM-specific environment variables
        let env_patterns = [
            r"(?i)LITELLM_API_KEY[\s]*=[\s]*([a-zA-Z0-9_-]+)",
            r#"(?i)LITELLM_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]+)['"]"#,
            r#"(?i)LITELLM_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]{30,})['"]"#,
        ];

        for pattern in &env_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(key_match) = cap.get(1) {
                        let key_value = key_match.as_str();
                        if self.is_valid_litellm_key(key_value) {
                            let confidence = self.confidence_score(key_value);
                            let confidence_enum = self.float_to_confidence(confidence);

                            return Some(DiscoveredKey::new(
                                "litellm".to_string(),
                                "environment".to_string(),
                                ValueType::ApiKey,
                                confidence_enum,
                                key_value.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    /// Extracts provider-specific keys that `LiteLLM` might use.
    fn extract_provider_keys(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Look for common provider keys that LiteLLM proxies
        let provider_patterns = [
            (
                r"(?i)openai[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "openai",
                "sk-",
            ),
            (
                r"(?i)anthropic[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "anthropic",
                "sk-ant-",
            ),
            (
                r"(?i)huggingface[_\s]*token[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "huggingface",
                "hf_",
            ),
            (
                r"(?i)cohere[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "cohere",
                "",
            ),
            (
                r"(?i)ai21[_\s]*api[_\s]*key[\s]*[:=][\s]*([a-zA-Z0-9_-]{15,})",
                "ai21",
                "",
            ),
        ];

        for (pattern, provider, prefix) in &provider_patterns {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| Error::PluginError(format!("Invalid regex pattern: {e}")))?;

            for cap in regex.captures_iter(content) {
                if let Some(key_match) = cap.get(1) {
                    let key_value = key_match.as_str().trim();

                    // Clean up the key value
                    let cleaned_key = key_value.trim_matches('\'').trim_matches('"').trim();

                    // Check if it matches the expected prefix
                    if prefix.is_empty() || cleaned_key.starts_with(prefix) {
                        let confidence = if prefix.is_empty() { 0.60 } else { 0.80 };
                        let confidence_enum = self.float_to_confidence(confidence);

                        let discovered_key = DiscoveredKey::new(
                            (*provider).to_string(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            confidence_enum,
                            cleaned_key.to_string(),
                        );

                        keys.push(discovered_key);
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Checks if a key is a valid `LiteLLM` API key format.
    fn is_valid_litellm_key(&self, key: &str) -> bool {
        // LiteLLM API keys must be at least 20 characters long
        if key.len() < 20 {
            return false;
        }

        // LiteLLM keys are typically alphanumeric with dashes/underscores
        key.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Converts float confidence to Confidence enum.
    fn float_to_confidence(&self, score: f32) -> Confidence {
        if score >= 0.80 {
            Confidence::High
        } else if score >= 0.60 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }

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
        discovered_key::Confidence, Environment, ProviderInstance, ProviderKey, ValidationStatus,
    };

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
    fn test_valid_litellm_key_detection() {
        let plugin = LiteLLMPlugin;

        assert!(plugin.is_valid_litellm_key("LL-ABCD-1234-EFGH-5678-IJKL-9012-MNOP"));
        assert!(plugin.is_valid_litellm_key("litellm-api-key-with-dashes-and-underscores"));
        assert!(!plugin.is_valid_litellm_key("short"));
        assert!(!plugin.is_valid_litellm_key("invalid key with spaces"));
    }

    #[test]
    fn test_extract_provider_keys() {
        let plugin = LiteLLMPlugin;
        let content = "openai_api_key: sk-1234567890abcdef";
        let path = Path::new("test.yaml");

        let keys = plugin.extract_provider_keys(path, content).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].provider, "openai");
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("litellm-api-key-with-dashes-and-UPPERCASE".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // Add a model
        let model =
            crate::models::Model::new("gpt-3.5-turbo".to_string(), "GPT-3.5 Turbo".to_string());
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = LiteLLMPlugin;
        let instance = ProviderInstance::new(
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
        let mut instance = ProviderInstance::new(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add a model but no keys
        let model =
            crate::models::Model::new("gpt-3.5-turbo".to_string(), "GPT-3.5 Turbo".to_string());
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add models
        let model1 =
            crate::models::Model::new("gpt-3.5-turbo".to_string(), "GPT-3.5 Turbo".to_string());
        let model2 =
            crate::models::Model::new("claude-3-sonnet".to_string(), "Claude 3 Sonnet".to_string());
        instance.add_model(model1);
        instance.add_model(model2);

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"gpt-3.5-turbo".to_string()));
        assert!(model_list.contains(&"claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = LiteLLMPlugin;
        let instance = ProviderInstance::new(
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
        let mut instance = ProviderInstance::new(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
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
        key.value = Some("litellm-api-key-with-dashes-and-UPPERCASE".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }

    #[test]
    fn test_initialize_instance() {
        let plugin = LiteLLMPlugin;
        let mut instance = ProviderInstance::new(
            "test-litellm".to_string(),
            "Test LiteLLM".to_string(),
            "litellm".to_string(),
            "https://api.litellm.ai".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("litellm-api-key-with-dashes-and-UPPERCASE".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        let result = plugin.initialize_instance(&instance);
        assert!(result.is_ok());
    }
}
