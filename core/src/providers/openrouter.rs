//! `OpenRouter` provider plugin implementation.

use crate::error::{Error, Result};
use crate::models::ModelMetadata;
use crate::plugins::ProviderPlugin;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// `OpenRouter` provider plugin.
///
/// This plugin provides integration with `OpenRouter`'s API, which aggregates
/// multiple AI model providers into a single unified interface.
pub struct OpenRouterPlugin;

impl OpenRouterPlugin {
    /// Default base URL for `OpenRouter` API
    const DEFAULT_BASE_URL: &'static str = "https://openrouter.ai/api/v1";
}

/// Response structure from `OpenRouter`'s /models endpoint
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

/// Model information from `OpenRouter` API
#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    description: Option<String>,
    context_length: Option<u32>,
    pricing: Option<OpenRouterPricing>,
    architecture: Option<OpenRouterArchitecture>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// Pricing information from `OpenRouter` API
#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: Option<String>,
    completion: Option<String>,
    image: Option<String>,
    request: Option<String>,
}

/// Architecture information from `OpenRouter` API
#[derive(Debug, Deserialize)]
struct OpenRouterArchitecture {
    modality: Option<String>,
    tokenizer: Option<String>,
    instruct_type: Option<String>,
}

impl OpenRouterPlugin {
    /// Converts `OpenRouter` pricing string to f64
    fn parse_price(price_str: Option<String>) -> Option<f64> {
        price_str.and_then(|s| s.parse::<f64>().ok())
    }

    /// Transforms `OpenRouter` model to `ModelMetadata`
    fn transform_model(model: OpenRouterModel) -> ModelMetadata {
        let metadata = ModelMetadata {
            id: Some(model.id.clone()),
            name: model.name.clone().or_else(|| Some("Unknown".to_string())),
            architecture: model.architecture.as_ref().and_then(|a| a.modality.clone()),
            parameter_count: None,
            training_cutoff: None,
            release_date: None,
            notes: model.description,
        };

        metadata
    }
}

#[async_trait]
impl ProviderPlugin for OpenRouterPlugin {
    fn name(&self) -> &'static str {
        "openrouter"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // OpenRouter keys typically start with "sk-or-"
        let mut score: f32 = 0.3;

        if key.starts_with("sk-or-") {
            score += 0.5;
        }

        // Length-based scoring
        if key.len() >= 40 {
            score += 0.1;
        }

        // Character diversity
        let has_uppercase = key.chars().any(char::is_uppercase);
        let has_lowercase = key.chars().any(char::is_lowercase);
        let has_digits = key.chars().any(|c| c.is_ascii_digit());

        if has_uppercase && has_lowercase && has_digits {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name.ends_with(".env")
            || file_name.ends_with(".json")
            || file_name.ends_with(".yaml")
            || file_name.ends_with(".yml")
    }

    async fn probe_models_async(
        &self,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<Vec<ModelMetadata>> {
        let url = format!("{}/models", base_url.unwrap_or(Self::DEFAULT_BASE_URL));

        // Create HTTP client
        let client = reqwest::Client::new();

        // Make API request
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        // Check for authentication errors
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::ApiError(
                "Authentication failed: Invalid API key".to_string(),
            ));
        }

        // Check for other HTTP errors
        if !response.status().is_success() {
            return Err(Error::ApiError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        // Parse response
        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .map_err(|e| Error::SerializationError(format!("Failed to parse API response: {e}")))?;

        // Transform models
        let models = models_response
            .data
            .into_iter()
            .map(Self::transform_model)
            .collect();

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::no_effect_underscore_binding)]
    use super::*;

    #[test]
    fn test_openrouter_plugin_name() {
        let plugin = OpenRouterPlugin;
        assert_eq!(plugin.name(), "openrouter");
    }

    #[test]
    fn test_openrouter_confidence_score() {
        let plugin = OpenRouterPlugin;

        // Test OpenRouter-specific key format with sufficient length and diversity
        let score1 = plugin.confidence_score("sk-or-v1-1234567890abcdefABCDEF1234567890");
        assert!(score1 > 0.8, "Expected score > 0.8, got {score1}");

        // Test OpenRouter prefix but shorter
        let score2 = plugin.confidence_score("sk-or-v1-1234567890abcdef");
        assert!(score2 > 0.7, "Expected score > 0.7, got {score2}");

        // Test generic key
        let score3 = plugin.confidence_score("sk-1234567890abcdef");
        assert!(score3 < 0.8, "Expected score < 0.8, got {score3}");

        // Test short key
        let score4 = plugin.confidence_score("short");
        assert!(score4 < 0.5, "Expected score < 0.5, got {score4}");
    }

    #[test]
    fn test_openrouter_can_handle_file() {
        let plugin = OpenRouterPlugin;

        assert!(plugin.can_handle_file(Path::new(".env")));
        assert!(plugin.can_handle_file(Path::new("config.json")));
        assert!(plugin.can_handle_file(Path::new("config.yaml")));
        assert!(!plugin.can_handle_file(Path::new("script.py")));
    }

    #[test]
    fn test_parse_price() {
        assert_eq!(
            OpenRouterPlugin::parse_price(Some("0.0001".to_string())),
            Some(0.0001)
        );
        assert_eq!(
            OpenRouterPlugin::parse_price(Some("invalid".to_string())),
            None
        );
        assert_eq!(OpenRouterPlugin::parse_price(None), None);
    }

    #[test]
    fn test_transform_model() {
        let openrouter_model = OpenRouterModel {
            id: "test-model".to_string(),
            name: Some("Test Model".to_string()),
            description: Some("A test model".to_string()),
            context_length: Some(4096),
            pricing: Some(OpenRouterPricing {
                prompt: Some("0.0001".to_string()),
                completion: Some("0.0002".to_string()),
                image: None,
                request: None,
            }),
            architecture: Some(OpenRouterArchitecture {
                modality: Some("text".to_string()),
                tokenizer: Some("cl100k_base".to_string()),
                instruct_type: Some("chatml".to_string()),
            }),
            extra: HashMap::new(),
        };

        let metadata = OpenRouterPlugin::transform_model(openrouter_model);

        assert_eq!(metadata.id, Some("test-model".to_string()));
        assert_eq!(metadata.name, Some("Test Model".to_string()));
        // New ModelMetadata structure doesn't have context_length or pricing fields
        assert!(metadata.architecture.is_some());
    }
}
