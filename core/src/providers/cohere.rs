//! Cohere provider plugin for scanning Cohere API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Cohere API keys and configuration files.
pub struct CoherePlugin;

#[async_trait::async_trait]
impl ProviderPlugin for CoherePlugin {
    fn name(&self) -> &'static str {
        "cohere"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Cohere keys have specific patterns
        if key.starts_with("cohere-") {
            0.95 // Very distinctive Cohere prefix
        } else if key.len() >= 32 && key.chars().all(|c| c.is_ascii_alphanumeric()) {
            0.60 // Could be Cohere key
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Cohere base URL cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        Ok(instance.has_non_empty_api_key())
    }

    async fn probe_models_async(
        &self,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<Vec<crate::models::ModelMetadata>> {
        let client = reqwest::Client::new();
        let base_url = base_url.unwrap_or("https://api.cohere.ai").trim_end_matches('/');

        let url = format!("{}/v1/models", base_url);
        let resp = client
            .get(&url)
            .bearer_auth(api_key)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| Error::HttpError(format!("Failed to fetch models: {}", e)))?;

        if !resp.status().is_success() {
            return Err(Error::ApiError(format!(
                "API returned status {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::SerializationError(format!("Failed to parse response: {}", e)))?;

        let models = json["models"]
            .as_array()
            .ok_or_else(|| Error::SerializationError("Expected 'models' array in response".to_string()))?
            .iter()
            .filter_map(|m| {
                let name = m["name"].as_str()?;
                Some(crate::models::ModelMetadata {
                    id: name.to_string(),
                    name: name.to_string(),
                    description: None,
                    context_length: None,
                    pricing: None,
                    architecture: None,
                    metadata: None,
                })
            })
            .collect();

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohere_plugin_name() {
        let plugin = CoherePlugin;
        assert_eq!(plugin.name(), "cohere");
    }

    #[test]
    fn test_cohere_confidence_score() {
        let plugin = CoherePlugin;

        // Test Cohere-prefixed key
        let score1 = plugin.confidence_score("cohere-1234567890abcdef");
        assert!(score1 > 0.9, "Expected score > 0.9 for cohere- prefix, got {score1}");

        // Test alphanumeric key (32+ chars)
        let score2 = plugin.confidence_score("abcdef1234567890abcdef1234567890");
        assert!(score2 > 0.5, "Expected score > 0.5 for long alphanumeric, got {score2}");

        // Test generic key
        let score3 = plugin.confidence_score("sk-1234");
        assert!(score3 < 0.5, "Expected score < 0.5 for generic key, got {score3}");
    }

    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = CoherePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "cohere".to_string(),
            base_url: "".to_string(),
            api_key: Some("cohere-test123".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_instance_valid() {
        let plugin = CoherePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "cohere".to_string(),
            base_url: "https://api.cohere.ai".to_string(),
            api_key: Some("cohere-test123".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = CoherePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "cohere".to_string(),
            base_url: "https://api.cohere.ai".to_string(),
            api_key: Some("cohere-test123".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_instance_configured_without_key() {
        let plugin = CoherePlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "cohere".to_string(),
            base_url: "https://api.cohere.ai".to_string(),
            api_key: None,
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
