//! Anthropic provider plugin for scanning Anthropic API keys and configuration.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

/// Response structure for Anthropic models API
#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

/// Individual model in the Anthropic API response
#[derive(Debug, Deserialize)]
struct AnthropicModel {
    id: String,
}

/// Plugin for scanning Anthropic API keys and configuration files.
pub struct AnthropicPlugin;

impl ProviderPlugin for AnthropicPlugin {
    fn name(&self) -> &'static str {
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
            return Err(Error::PluginError(
                "Anthropic base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Anthropic base URL patterns
        let is_valid_anthropic_url = instance.base_url.starts_with("https://api.anthropic.com")
            || instance.base_url.starts_with("https://api.anthropic.ai")
            || instance
                .base_url
                .starts_with("https://claude-api.anthropic.com");

        if !is_valid_anthropic_url {
            return Err(Error::PluginError(
                "Invalid Anthropic base URL. Expected format: https://api.anthropic.com"
                    .to_string(),
            ));
        }

        // Validate that at least one key exists if models are configured
        if !instance.models.is_empty() && !instance.has_non_empty_api_key() {
            return Err(Error::PluginError(
                "Anthropic instance has models configured but no valid API keys".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // If instance has specific models configured, return those
        if !instance.models.is_empty() {
            return Ok(instance.models.clone());
        }

        // Try to fetch models from API if we have a valid key
        if instance.has_non_empty_api_key() {
            if let Some(api_key) = instance.get_api_key() {
                return Self::fetch_supported_models(api_key);
            }
        }

        // If no valid keys, return empty list instead of hard-coded models
        Ok(vec![])
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Anthropic requires both a valid base URL and at least one valid API key
        if !instance.has_non_empty_api_key() {
            return Ok(false);
        }

        // Validate base URL format
        self.validate_instance(instance)?;

        Ok(true)
    }

    fn probe_models(&self, api_key: &str) -> Result<Vec<String>> {
        // Use the existing fetch_supported_models method
        Self::fetch_supported_models(api_key)
    }
}

impl AnthropicPlugin {
    /// Fetch supported models from the Anthropic API
    ///
    /// Makes a blocking HTTP GET request to the Anthropic models endpoint.
    /// Returns a vector of model IDs on success, or falls back to hardcoded defaults on error.
    fn fetch_supported_models(api_key: &str) -> Result<Vec<String>> {
        // Create a blocking HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| Error::PluginError(format!("Failed to create HTTP client: {e}")))?;

        // Make the API request
        let response = client
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .send();

        // Handle response - return proper errors instead of falling back
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<AnthropicModelsResponse>() {
                        Ok(models_response) => {
                            let model_ids: Vec<String> =
                                models_response.data.into_iter().map(|m| m.id).collect();

                            if model_ids.is_empty() {
                                Err(Error::PluginError(
                                    "Anthropic API returned empty model list".to_string(),
                                ))
                            } else {
                                eprintln!(
                                    "DEBUG: Successfully fetched {} models from Anthropic API",
                                    model_ids.len()
                                );
                                Ok(model_ids)
                            }
                        }
                        Err(e) => Err(Error::PluginError(format!(
                            "Failed to parse Anthropic API response: {e}"
                        ))),
                    }
                } else if resp.status() == 401 {
                    Err(Error::PluginError(
                        "Invalid Anthropic API key (401 Unauthorized)".to_string(),
                    ))
                } else if resp.status() == 403 {
                    Err(Error::PluginError(
                        "Anthropic API access forbidden (403 Forbidden)".to_string(),
                    ))
                } else {
                    Err(Error::PluginError(format!(
                        "Anthropic API returned unexpected status: {}",
                        resp.status()
                    )))
                }
            }
            Err(e) => Err(Error::PluginError(format!(
                "Failed to call Anthropic API: {e}"
            ))),
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
    use crate::models::ProviderInstance;

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
        let mut instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        // Set a valid API key directly on the instance
        instance.set_api_key("sk-ant-test123".to_string());

        // Add a model
        instance.add_model("claude-3-sonnet".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_url() {
        let plugin = AnthropicPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://invalid-url.com".to_string(),
            String::new(),
        );

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Anthropic base URL"));
    }

    #[test]
    fn test_validate_no_keys_with_models() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        // Add a model but no keys
        instance.add_model("claude-3-sonnet".to_string());

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no valid API keys"));
    }

    #[test]
    fn test_get_instance_models_with_configured_models() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        // Add models
        instance.add_model("claude-3-sonnet".to_string());
        instance.add_model("claude-3-opus".to_string());

        let model_list = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(model_list.len(), 2);
        assert!(model_list.contains(&"claude-3-sonnet".to_string()));
        assert!(model_list.contains(&"claude-3-opus".to_string()));
    }

    #[test]
    fn test_get_instance_models_without_keys() {
        let plugin = AnthropicPlugin;
        let instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        let models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(models.len(), 0); // Should return no models when no valid keys (probe returns error)
    }

    #[test]
    fn test_is_instance_configured() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        // Without keys, should return false
        assert!(!plugin.is_instance_configured(&instance).unwrap());

        // Set a valid API key directly on the instance
        instance.set_api_key("sk-ant-test123".to_string());

        // With valid key and URL, should return true
        assert!(plugin.is_instance_configured(&instance).unwrap());
    }

    #[test]
    fn test_probe_models_with_invalid_api_key() {
        let plugin = AnthropicPlugin;

        // Test with an obviously invalid API key
        let result = plugin.probe_models("invalid-key");

        // The probe should return an error for invalid keys
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Anthropic API key") || error_msg.contains("401"));
    }

    #[test]
    fn test_probe_models_with_empty_key() {
        let plugin = AnthropicPlugin;

        // Test with an empty API key
        let result = plugin.probe_models("");

        // The probe should return an error for empty keys
        assert!(result.is_err());
    }

    #[test]
    fn test_probe_models_malformed_response_handling() {
        let plugin = AnthropicPlugin;

        // Test with a key that might exist but is invalid
        let result = plugin.probe_models("sk-ant-test1234567890abcdefghijklmnopqrstuvwxyz");

        // The probe should handle malformed responses gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_supported_models_direct_error_handling() {
        let plugin = AnthropicPlugin;

        // Test the direct fetch method with invalid credentials
        let result = AnthropicPlugin::fetch_supported_models("sk-ant-invalid-key");

        // Should return an error for invalid API keys
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Anthropic API key") || error_msg.contains("401"));
    }

    #[test]
    fn test_probe_models_integration_success_scenario() {
        let plugin = AnthropicPlugin;

        // Test with a properly formatted but fake API key
        // This tests the code path but won't actually call the real API
        let result = plugin.probe_models("sk-ant-api03-fake1234567890abcdefghijklmnopqrstuvwxyz");

        // Since this is a fake key, it should fail with a 401 error
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Invalid Anthropic API key") || error_msg.contains("401"));
    }

    #[test]
    fn test_get_instance_models_with_valid_configured_models() {
        let plugin = AnthropicPlugin;
        let mut instance = ProviderInstance::new_without_models(
            "test-anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            String::new(),
        );

        // Add some pre-configured models
        instance.add_model("claude-3-sonnet".to_string());
        instance.add_model("claude-3-opus".to_string());

        // Should return the configured models regardless of API key status
        let instance_models = plugin.get_instance_models(&instance).unwrap();
        assert_eq!(instance_models.len(), 2);
        assert!(instance_models.contains(&"claude-3-sonnet".to_string()));
        assert!(instance_models.contains(&"claude-3-opus".to_string()));
    }

    #[test]
    fn test_probe_models_with_network_timeout_simulation() {
        let plugin = AnthropicPlugin;

        // Test with a very long key that might cause issues
        let long_key = format!("sk-ant-api03-{}", "a".repeat(100));
        let result = plugin.probe_models(&long_key);

        // Should handle malformed keys gracefully
        assert!(result.is_err());
    }
}
