//! OpenAI provider plugin for scanning OpenAI API keys and configuration.

use crate::error::{Error, Result};
use crate::models::{discovered_key::{Confidence, DiscoveredKey, ValueType}, ProviderInstance};
use crate::plugins::ProviderPlugin;
use url::Url;

/// Configuration for OpenAI provider defaults
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// Default chat completion models
    pub chat_models: Vec<String>,
    /// Default embedding models
    pub embedding_models: Vec<String>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            chat_models: vec![
                "gpt-3.5-turbo".to_string(), // Legacy fallback only
            ],
            embedding_models: vec![
                "text-embedding-3-small".to_string(), // Modern replacement for ada-002
                "text-embedding-3-large".to_string(), // Alternative option
            ],
        }
    }
}

impl OpenAIConfig {
    /// Load configuration from environment variables with fallbacks
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Override chat models if specified in environment
        if let Ok(chat_models_str) = std::env::var("OPENAI_CHAT_MODELS") {
            if !chat_models_str.is_empty() {
                config.chat_models = chat_models_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        // Override embedding models if specified in environment
        if let Ok(embedding_models_str) = std::env::var("OPENAI_EMBEDDING_MODELS") {
            if !embedding_models_str.is_empty() {
                config.embedding_models = embedding_models_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        config
    }
    
    /// Get all models (chat + embedding)
    pub fn all_models(&self) -> Vec<String> {
        let mut all = self.chat_models.clone();
        all.extend(self.embedding_models.clone());
        all
    }
}

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
        
        // Check for valid OpenAI base URL patterns by parsing and validating hostname
        let is_valid_openai_url = match Url::parse(&instance.base_url) {
            Ok(parsed_url) => {
                let host = parsed_url.host_str().unwrap_or("");
                let allowed_hosts = ["api.openai.com", "openai-api-proxy.com"];
                allowed_hosts.contains(&host)
            }
            Err(_) => false,
        };
        
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

        // Load configuration from environment or use defaults
        let config = OpenAIConfig::from_env();
        
        // Get all models from configuration
        let mut models = config.all_models();

        // If no valid keys, only return a subset of models (mainly for testing/demo purposes)
        if !instance.has_valid_keys() {
            // Return only the first few models to avoid exposing all capabilities without keys
            models.truncate(3); // Return first 3 models for testing without keys
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
    use std::path::Path;
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
        let _content = "api_key: sk-1234";
        let _path = Path::new("test.yaml");
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
            "text-embedding-3-small".to_string(),
            instance.id.clone(),
            "Text Embedding 3 Small".to_string(),
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
            "text-embedding-3-large".to_string(),
            instance.id.clone(),
            "Text Embedding 3 Large".to_string(),
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
            "text-embedding-3-small".to_string(),
            instance.id.clone(),
            "Text Embedding 3 Small".to_string(),
        );
        instance.add_model(model1);
        instance.add_model(model2);

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"text-embedding-3-small".to_string()));
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
        assert_eq!(models.len(), 3); // Should return only 3 models when no valid keys (configurable via env)
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"text-embedding-3-small".to_string()));
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

    #[test]
    fn test_openai_config_defaults() {
        let config = OpenAIConfig::default();
        
        // Should only have gpt-3.5-turbo as chat model
        assert_eq!(config.chat_models.len(), 1);
        assert_eq!(config.chat_models[0], "gpt-3.5-turbo");
        
        // Should have modern embedding models
        assert_eq!(config.embedding_models.len(), 2);
        assert!(config.embedding_models.contains(&"text-embedding-3-small".to_string()));
        assert!(config.embedding_models.contains(&"text-embedding-3-large".to_string()));
        
        // All models should include both chat and embedding
        let all_models = config.all_models();
        assert_eq!(all_models.len(), 3);
        assert!(all_models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(all_models.contains(&"text-embedding-3-small".to_string()));
        assert!(all_models.contains(&"text-embedding-3-large".to_string()));
    }

    #[test]
    fn test_openai_config_from_env() {
        // Set environment variables for testing
        std::env::set_var("OPENAI_CHAT_MODELS", "gpt-4o-mini, gpt-4o");
        std::env::set_var("OPENAI_EMBEDDING_MODELS", "text-embedding-3-small");
        
        let config = OpenAIConfig::from_env();
        
        // Should use environment variables
        assert_eq!(config.chat_models.len(), 2);
        assert!(config.chat_models.contains(&"gpt-4o-mini".to_string()));
        assert!(config.chat_models.contains(&"gpt-4o".to_string()));
        
        assert_eq!(config.embedding_models.len(), 1);
        assert_eq!(config.embedding_models[0], "text-embedding-3-small");
        
        // Clean up
        std::env::remove_var("OPENAI_CHAT_MODELS");
        std::env::remove_var("OPENAI_EMBEDDING_MODELS");
    }

    #[test]
    fn test_hostname_validation_valid_urls() {
        let plugin = OpenAIPlugin;
        
        let valid_urls = vec![
            "https://api.openai.com",
            "https://api.openai.com/v1",
            "https://api.openai.com/v1/chat/completions",
            "https://openai-api-proxy.com",
            "https://openai-api-proxy.com/v1",
            "https://api.openai.com:443",
            "https://openai-api-proxy.com:8080",
        ];
        
        for url in valid_urls {
            let instance = ProviderInstance::new(
                "test-openai".to_string(),
                "Test OpenAI".to_string(),
                "openai".to_string(),
                url.to_string(),
            );
            
            let result = plugin.validate_instance(&instance);
            assert!(result.is_ok(), "URL '{}' should be valid but got error: {:?}", url, result.err());
        }
    }

    #[test]
    fn test_hostname_validation_invalid_urls() {
        let plugin = OpenAIPlugin;
        
        let invalid_urls = vec![
            "https://malicious-openai.com",
            "https://openai-proxy.com",
            "https://my-openai-api.com",
            "https://openai.evil.com",
            "https://not-openai.com",
            "https://api.openai.org",  // Wrong TLD
            "https://api.openai.net",  // Wrong TLD
            "https://api.openai.com.evil.com",  // Subdomain attack
        ];
        
        for url in invalid_urls {
            let instance = ProviderInstance::new(
                "test-openai".to_string(),
                "Test OpenAI".to_string(),
                "openai".to_string(),
                url.to_string(),
            );
            
            let result = plugin.validate_instance(&instance);
            assert!(result.is_err(), "URL '{}' should be invalid but was accepted", url);
        }
    }
}
