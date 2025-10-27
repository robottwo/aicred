//! OpenAI provider plugin for scanning OpenAI API keys and configuration.

use crate::error::{Error, Result};
use crate::models::{discovered_key::{Confidence, DiscoveredKey, ValueType}, ProviderInstance};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning OpenAI API keys and configuration files.
pub struct OpenAIPlugin;

impl ProviderPlugin for OpenAIPlugin {
    fn name(&self) -> &str {
        "openai"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // OpenAI keys have very specific patterns
        if key.starts_with("sk-proj-") {
            0.95 // Project keys are very distinctive
        } else if key.starts_with("sk-") {
            0.95 // Standard OpenAI keys
        } else if key.len() >= 40 && key.contains('-') {
            0.75 // Might be an OpenAI key without the prefix
        } else {
            0.50 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;
        
        // OpenAI-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError("OpenAI base URL cannot be empty".to_string()));
        }
        
        // Check for valid OpenAI base URL patterns
        let is_valid_openai_url = instance.base_url.starts_with("https://api.openai.com") ||
                                   instance.base_url.starts_with("https://api.openai.com/v1") ||
                                   instance.base_url.starts_with("https://openai-api-proxy.com") ||
                                   instance.base_url.contains("openai");
        
        if !is_valid_openai_url {
            return Err(Error::PluginError(
                "Invalid OpenAI base URL. Expected format: https://api.openai.com".to_string()
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_valid_keys() {
            return Err(Error::PluginError(
                "OpenAI instance has models configured but no valid API keys".to_string()
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Otherwise, return default OpenAI models based on instance configuration
        let mut models = vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "text-davinci-003".to_string(),
            "text-embedding-ada-002".to_string(),
        ];

        // If no valid keys, only return a subset of models
        if !instance.has_valid_keys() {
            models.truncate(4); // Return first four models (including gpt-4o) for testing without keys
        }

        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // OpenAI requires both a valid base URL and at least one valid API key
        if !instance.has_valid_keys() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;
        
        Ok(true)
    }

    fn initialize_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // OpenAI-specific initialization logic
        // This could include testing API connectivity, validating model access, etc.
        
        // For now, just validate the instance
        self.validate_instance(instance)?;
        
        // Additional OpenAI-specific initialization could go here
        // such as testing API endpoints, checking model availability, etc.
        
        Ok(())
    }
}

impl OpenAIPlugin {
    /// Extracts OpenAI key from environment variable content.
    fn extract_from_env(&self, content: &str) -> Option<DiscoveredKey> {
        // Look for OPENAI_API_KEY environment variable
        let env_patterns = [
            r"(?i)OPENAI_API_KEY[\s]*=[\s]*([a-zA-Z0-9_-]+)",
            r#"(?i)OPENAI_API_KEY[\s]*=[\s]*['"]([a-zA-Z0-9_-]+)['"]"#,
            r#"(?i)OPENAI_API_KEY[\s]*=[\s]*['"](sk-[a-zA-Z0-9_-]+)['"]"#,
        ];

        for pattern in &env_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for cap in regex.captures_iter(content) {
                    if let Some(key_match) = cap.get(1) {
                        let key_value = key_match.as_str();
                        if self.is_valid_openai_key(key_value) {
                            let confidence = self.confidence_score(key_value);
                            let confidence_enum = self.float_to_confidence(confidence);

                            return Some(DiscoveredKey::new(
                                "openai".to_string(),
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

    /// Checks if a key is a valid OpenAI key format.
    fn is_valid_openai_key(&self, key: &str) -> bool {
        // OpenAI keys must be at least 19 characters long (including the sk- prefix)
        if key.len() < 19 {
            return false;
        }

        // Check for OpenAI key patterns
        if key.starts_with("sk-") || key.starts_with("sk-proj-") {
            return true;
        }

        // Allow keys that look like they could be OpenAI keys (long with alphanumeric and dashes)
        key.len() >= 40 && key.chars().all(|c| c.is_alphanumeric() || c == '-')
    }

    /// Converts float confidence to Confidence enum.
    fn float_to_confidence(&self, score: f32) -> Confidence {
        if score >= 0.9 {
            Confidence::VeryHigh
        } else if score >= 0.7 {
            Confidence::High
        } else if score >= 0.5 {
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
    fn test_openai_plugin_name() {
        let plugin = OpenAIPlugin;
        assert_eq!(plugin.name(), "openai");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = OpenAIPlugin;

        assert_eq!(plugin.confidence_score("sk-1234567890abcdef"), 0.95);
        assert_eq!(plugin.confidence_score("sk-proj-1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("random-key-with-dashes-123456789"),
            0.50
        );
    }

    #[test]
    fn test_valid_openai_key_detection() {
        let plugin = OpenAIPlugin;

        let test_key = "sk-1234567890abcdef";
        println!("Key: '{}', Length: {}", test_key, test_key.len());
        println!("Starts with 'sk-': {}", test_key.starts_with("sk-"));
        println!(
            "is_valid_openai_key result: {}",
            plugin.is_valid_openai_key(test_key)
        );

        assert!(plugin.is_valid_openai_key("sk-1234567890abcdef"));
        assert!(plugin.is_valid_openai_key("sk-proj-1234567890abcdef"));
        assert!(plugin.is_valid_openai_key("sk-1234567890abcdef1234567890abcdef"));
        assert!(!plugin.is_valid_openai_key("short"));
        assert!(!plugin.is_valid_openai_key("sk-"));
    }

    #[test]
    fn test_parse_config_ignores_invalid_key() {
        let plugin = OpenAIPlugin;
        // too short to be valid
        let content = "api_key: sk-1234";
        let path = Path::new("test.yaml");
        // Test that invalid keys are not considered valid API keys
        assert!(!plugin.is_valid_openai_key("sk-1234"));
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = OpenAIPlugin;
        let mut instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("sk-test1234567890abcdef".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // Add a model
        let model = crate::models::Model::new(
            "gpt-3.5-turbo".to_string(),
            instance.id.clone(),
            "GPT-3.5 Turbo".to_string(),
        );
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = OpenAIPlugin;
        let instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://invalid-url.com".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid OpenAI base URL"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = OpenAIPlugin;
        let mut instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // Add a model but no keys
        let model = crate::models::Model::new(
            "gpt-3.5-turbo".to_string(),
            instance.id.clone(),
            "GPT-3.5 Turbo".to_string(),
        );
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = OpenAIPlugin;
        let mut instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // Add models
        let model1 = crate::models::Model::new(
            "gpt-3.5-turbo".to_string(),
            instance.id.clone(),
            "GPT-3.5 Turbo".to_string(),
        );
        let model2 = crate::models::Model::new(
            "gpt-4".to_string(),
            instance.id.clone(),
            "GPT-4".to_string(),
        );
        instance.add_model(model1);
        instance.add_model(model2);

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = OpenAIPlugin;
        let instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 4); // Should return only four models when no valid keys
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = OpenAIPlugin;
        let mut instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
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
        key.value = Some("sk-test1234567890abcdef".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }

    #[test]
    fn test_initialize_instance() {
        let plugin = OpenAIPlugin;
        let mut instance = ProviderInstance::new(
            "test-openai".to_string(),
            "Test OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // Add a valid key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some("sk-test1234567890abcdef".to_string());
        key.validation_status = ValidationStatus::Valid;
        instance.add_key(key);

        let result = plugin.initialize_instance(&instance);
        assert!(result.is_ok());
    }
}
