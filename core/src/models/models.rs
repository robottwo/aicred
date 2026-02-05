//! LLM model definitions and metadata.

use serde::{Deserialize, Serialize};

/// An LLM model with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    /// Unique model identifier
    pub id: String,
    /// Provider name
    pub provider: String,
    /// Human-readable model name
    pub name: String,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Context window size (in tokens)
    pub context_window: Option<u32>,
    /// Pricing information
    pub pricing: Option<ModelPricing>,
    /// Extended metadata
    pub metadata: ModelMetadata,
}

/// Model capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Supports chat/conversation
    pub chat: bool,
    /// Supports text completion
    pub completion: bool,
    /// Supports embeddings
    pub embedding: bool,
    /// Supports function/tool calling
    pub function_calling: bool,
    /// Supports vision/image inputs
    pub vision: bool,
    /// Supports JSON mode output
    pub json_mode: bool,
}

/// Pricing information for a model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPricing {
    /// Cost per input token
    pub input_cost_per_token: f64,
    /// Cost per output token
    pub output_cost_per_token: f64,
    /// Currency code (e.g., "USD")
    pub currency: String,
}

/// Extended model metadata.
///
/// Note: For backward compatibility with probe_models_async, this also includes
/// id, name, and pricing fields. In the future, consider separating probed
/// model info from static metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelMetadata {
    /// Model ID (for backward compatibility with probing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Model name (for backward compatibility with probing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Model architecture (e.g., "transformer", "diffusion")
    pub architecture: Option<String>,
    /// Number of parameters (e.g., 7B, 70B)
    pub parameter_count: Option<u64>,
    /// Training data cutoff date
    pub training_cutoff: Option<String>,
    /// Model release date
    pub release_date: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
}

/// Token cost calculation result.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenCost {
    /// Total cost in the model's currency
    pub total_cost: f64,
    /// Input token cost
    pub input_cost: f64,
    /// Output token cost
    pub output_cost: f64,
    /// Currency code
    pub currency: String,
}

impl Model {
    /// Creates a new Model with basic information (for backward compatibility).
    #[must_use]
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            provider: String::new(),
            name,
            capabilities: ModelCapabilities::default(),
            context_window: None,
            pricing: None,
            metadata: ModelMetadata::default(),
        }
    }

    /// Validates the model configuration (stub for backward compatibility).
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.id.is_empty() {
            return Err(crate::error::Error::ValidationError(
                "Model ID cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculates the cost for a given number of input and output tokens
    #[must_use]
    pub fn token_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<TokenCost> {
        self.pricing.as_ref().map(|p| {
            let input_cost = f64::from(input_tokens) * p.input_cost_per_token;
            let output_cost = f64::from(output_tokens) * p.output_cost_per_token;
            
            TokenCost {
                total_cost: input_cost + output_cost,
                input_cost,
                output_cost,
                currency: p.currency.clone(),
            }
        })
    }
    
    /// Checks if the model supports a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        match capability.to_lowercase().as_str() {
            "chat" => self.capabilities.chat,
            "completion" => self.capabilities.completion,
            "embedding" => self.capabilities.embedding,
            "function_calling" | "functions" | "tools" => self.capabilities.function_calling,
            "vision" => self.capabilities.vision,
            "json_mode" | "json" => self.capabilities.json_mode,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_cost_calculation() {
        let model = Model {
            id: "gpt-4".to_string(),
            provider: "openai".to_string(),
            name: "GPT-4".to_string(),
            capabilities: ModelCapabilities::default(),
            context_window: Some(8192),
            pricing: Some(ModelPricing {
                input_cost_per_token: 0.00003,
                output_cost_per_token: 0.00006,
                currency: "USD".to_string(),
            }),
            metadata: ModelMetadata::default(),
        };

        let cost = model.token_cost(1000, 500);
        assert!(cost.is_some());
        
        let cost_value = cost.unwrap();
        assert!((cost_value.input_cost - 0.03).abs() < 0.0001);
        assert!((cost_value.output_cost - 0.03).abs() < 0.0001);
        assert!((cost_value.total_cost - 0.06).abs() < 0.0001);
    }

    #[test]
    fn test_capability_check() {
        let model = Model {
            id: "test".to_string(),
            provider: "test".to_string(),
            name: "Test Model".to_string(),
            capabilities: ModelCapabilities {
                chat: true,
                function_calling: true,
                ..Default::default()
            },
            context_window: None,
            pricing: None,
            metadata: ModelMetadata::default(),
        };

        assert!(model.has_capability("chat"));
        assert!(model.has_capability("functions"));
        assert!(model.has_capability("tools"));
        assert!(!model.has_capability("vision"));
    }
}
