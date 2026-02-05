//! Ollama provider plugin for scanning Ollama configuration.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Ollama configuration files.
pub struct OllamaPlugin;

impl ProviderPlugin for OllamaPlugin {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Ollama configuration is less critical than API keys, so lower confidence
        if key.starts_with("http://") || key.starts_with("https://") {
            0.85 // Server URL
        } else if key.contains('/') && key.len() >= 7 {
            0.80 // Model name (e.g., "llama2/7b")
        } else {
            0.70 // Other configuration
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // First perform base validation
        self.validate_base_instance(instance)?;

        // Parse and enforce loopback/local + default port 11434
        let url = url::Url::parse(&instance.base_url)
            .map_err(|e| Error::PluginError(format!("Invalid Ollama base URL: {e}")))?;
        let host_ok = match url.host_str() {
            Some("localhost") => true,
            Some(h) if h == "127.0.0.1" || h == "0.0.0.0" => true,
            Some(h) => h
                .parse::<std::net::IpAddr>()
                .map(|ip| ip.is_loopback())
                .unwrap_or(false),
            None => false,
        } || matches!(url.host(), Some(url::Host::Ipv6(addr)) if addr.is_loopback());
        let port_ok = url.port() == Some(11434);
        if !(host_ok && port_ok) {
            return Err(Error::PluginError(
                "Invalid Ollama base URL. Expected local loopback host and port 11434 (e.g., http://localhost:11434)".to_string()
            ));
        }

        // Ollama doesn't require API keys, so we don't check for them
        // But if models are configured, we should validate them
        if !instance.models.is_empty() {
            for model in &instance.models {
                if model.is_empty() {
                    return Err(Error::PluginError(
                        "Ollama instance has empty model ID".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.clone());
        }

        // Otherwise, return default Ollama models
        let models = vec![
            "llama2".to_string(),
            "llama3".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
            "phi".to_string(),
            "gemma".to_string(),
        ];

        // Ollama doesn't require keys, so we return all models regardless
        Ok(models)
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Ollama only requires a valid base URL, no API keys needed
        // Validate base URL format
        match self.validate_instance(instance) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn initialize_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // Ollama-specific initialization logic
        // This could include testing connectivity to the Ollama server

        // For now, just validate the instance
        self.validate_instance(instance)?;

        // Additional Ollama-specific initialization could go here
        // such as testing server connectivity, checking available models, etc.

        Ok(())
    }
}

impl OllamaPlugin {
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
    fn test_ollama_plugin_name() {
        let plugin = OllamaPlugin;
        assert_eq!(plugin.name(), "ollama");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = OllamaPlugin;

        assert_eq!(plugin.confidence_score("http://localhost:11434"), 0.85);
        assert_eq!(plugin.confidence_score("https://ollama.example.com"), 0.85);
        assert_eq!(plugin.confidence_score("llama2/7b"), 0.80);
        assert_eq!(plugin.confidence_score("some-config-value"), 0.70);
    }

    #[test]
    fn test_validate_valid_instance() {
        let plugin = OllamaPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );

        // Add a model (Ollama doesn't require keys)
        let model = crate::models::Model::new("llama2".to_string(), "Llama 2".to_string());
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = OllamaPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "https://invalid-url.com".to_string(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Ollama base URL"));
    }

    #[test]
    fn test_validate_empty_model_id() {
        let plugin = OllamaPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );

        // Add a model with empty ID
        let model = crate::models::Model::new(String::new(), "Empty Model".to_string());
        instance.add_model(model);

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("empty model ID"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = OllamaPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );

        // Add models
        let model1 = crate::models::Model::new("llama2".to_string(), "Llama 2".to_string());
        let model2 = crate::models::Model::new("mistral".to_string(), "Mistral".to_string());
        instance.add_model(model1);
        instance.add_model(model2);

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"llama2".to_string()));
        assert!(model_list.contains(&"mistral".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = OllamaPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 6); // Should return all models (Ollama doesn't require keys)
        assert!(models.contains(&"llama2".to_string()));
        assert!(models.contains(&"llama3".to_string()));
        assert!(models.contains(&"mistral".to_string()));
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = OllamaPlugin;

        // With valid URL, should return true (no keys required)
        let instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );
        assert!(plugin.is_instance_configured(&instance).unwrap());

        // With invalid URL, should return false
        let invalid_instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "https://invalid-url.com".to_string(),
        );
        assert!(!plugin.is_instance_configured(&invalid_instance).unwrap());
    }

    #[test]
    fn test_initialize_instance() {
        let plugin = OllamaPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-ollama".to_string(),
            "Test Ollama".to_string(),
            "ollama".to_string(),
            "http://localhost:11434".to_string(),
        );

        let result = plugin.initialize_instance(&instance);
        assert!(result.is_ok());
    }
}
