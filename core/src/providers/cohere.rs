//! Cohere provider plugin for scanning Cohere API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Cohere API keys and configuration files.
pub struct CoherePlugin;

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
}

#[async_trait::async_trait]
impl crate::plugins::ProviderPlugin for CoherePlugin {
    async fn validate_instance_async(
        &self,
        instance: &ProviderInstance,
    ) -> crate::validation::Result<crate::validation::ValidationResult> {
        let client = reqwest::Client::new();
        let base_url = instance.base_url.trim_end_matches('/');

        let api_key = match instance.api_key.as_ref() {
            Some(key) if !key.is_empty() => key,
            _ => {
                return Ok(crate::validation::ValidationResult::failure(
                    crate::validation::ValidationError::InvalidApiKey {
                        details: Some("No API key provided".to_string()),
                    },
                ));
            }
        };

        let confidence = self.confidence_score(api_key);
        if confidence < 0.5 {
            return Ok(crate::validation::ValidationResult::failure(
                crate::validation::ValidationError::InvalidKeyFormat {
                    expected: "Cohere API key".to_string(),
                    actual: format!("Low confidence score ({:.2})", confidence),
                },
            ));
        }

        // Try to list models
        let url = format!("{}/v1/models", base_url);
        let resp = client
            .get(&url)
            .bearer_auth(api_key)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        crate::providers::validation_helper::ValidationHandler::handle_response(
            resp,
            |response| {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        json["models"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }
                    Err(_) => vec![],
                }
            },
            crate::providers::validation_helper::RateLimitParser::parse_generic,
            10,
        )
    }

    async fn quick_validate(
        &self,
        api_key: &str,
        _base_url: Option<&str>,
    ) -> Result<bool> {
        Ok(self.confidence_score(api_key) >= 0.5)
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
