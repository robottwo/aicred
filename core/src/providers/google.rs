//! Google AI provider plugin for scanning Google AI API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Google AI API keys and configuration files.
pub struct GooglePlugin;

impl ProviderPlugin for GooglePlugin {
    fn name(&self) -> &str {
        "google"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Google API keys are typically alphanumeric strings
        if key.len() == 39 && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            0.90
        } else if key.len() >= 32 && key.len() <= 50 && key.chars().all(|c| c.is_alphanumeric()) {
            0.70
        } else {
            0.35
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Google AI base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Google AI base URL patterns
        let is_valid_url = instance.base_url.contains("googleapis.com")
            || instance.base_url.contains("generativelanguage.googleapis.com");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Google AI base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Google AI models
        Ok(vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.0-pro".to_string(),
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
    fn test_google_plugin_name() {
        let plugin = GooglePlugin;
        assert_eq!(plugin.name(), "google");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = GooglePlugin;
        // 39 character alphanumeric key
        let key_39 = "AIzaSyDaGmWKa4JsXZ-HjGw7ISLn_3namBGewQe";
        assert_eq!(plugin.confidence_score(key_39), 0.90);
        assert_eq!(
            plugin.confidence_score("alphanumericstringthirtytwocharacters"),
            0.70
        );
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
