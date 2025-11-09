//! Model metadata structures for provider probing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a model available from a provider.
///
/// This structure contains comprehensive information about a model,
/// including its capabilities, pricing, and architecture details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique identifier for the model (e.g., "gpt-4", "claude-3-opus")
    pub id: String,

    /// Human-readable name of the model
    pub name: String,

    /// Description of the model's capabilities and use cases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Maximum context length in tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,

    /// Pricing information for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,

    /// Architecture details of the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ModelArchitecture>,

    /// Additional provider-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Pricing information for a model.
///
/// All prices are in USD per token or per request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per prompt token in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<f64>,

    /// Cost per completion token in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<f64>,

    /// Cost per image in USD (for vision models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<f64>,

    /// Cost per request in USD (for models with per-request pricing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<f64>,
}

/// Architecture details of a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    /// Modality of the model (e.g., "text", "text+image", "multimodal")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,

    /// Tokenizer used by the model (e.g., "`cl100k_base`", "claude")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,

    /// Instruction format type (e.g., "chatml", "alpaca", "vicuna")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruct_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: Some("A test model".to_string()),
            context_length: Some(4096),
            pricing: Some(ModelPricing {
                prompt: Some(0.0001),
                completion: Some(0.0002),
                image: None,
                request: None,
            }),
            architecture: Some(ModelArchitecture {
                modality: Some("text".to_string()),
                tokenizer: Some("cl100k_base".to_string()),
                instruct_type: Some("chatml".to_string()),
            }),
            metadata: None,
        };

        assert_eq!(metadata.id, "test-model");
        assert_eq!(metadata.name, "Test Model");
        assert_eq!(metadata.context_length, Some(4096));
    }

    #[test]
    fn test_model_metadata_serialization() {
        let metadata = ModelMetadata {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: None,
            context_length: Some(8192),
            pricing: None,
            architecture: None,
            metadata: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ModelMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, metadata.id);
        assert_eq!(deserialized.name, metadata.name);
        assert_eq!(deserialized.context_length, metadata.context_length);
    }

    #[test]
    fn test_model_pricing_creation() {
        let pricing = ModelPricing {
            prompt: Some(0.0001),
            completion: Some(0.0002),
            image: Some(0.01),
            request: None,
        };

        assert_eq!(pricing.prompt, Some(0.0001));
        assert_eq!(pricing.completion, Some(0.0002));
        assert_eq!(pricing.image, Some(0.01));
        assert_eq!(pricing.request, None);
    }

    #[test]
    fn test_model_architecture_creation() {
        let architecture = ModelArchitecture {
            modality: Some("multimodal".to_string()),
            tokenizer: Some("gpt2".to_string()),
            instruct_type: Some("alpaca".to_string()),
        };

        assert_eq!(architecture.modality, Some("multimodal".to_string()));
        assert_eq!(architecture.tokenizer, Some("gpt2".to_string()));
        assert_eq!(architecture.instruct_type, Some("alpaca".to_string()));
    }
}
