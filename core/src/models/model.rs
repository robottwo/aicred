//! Model data structure representing AI models with their capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token cost tracking for model usage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenCost {
    /// Cost per million input tokens in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cost_per_million: Option<f64>,

    /// Cost per million output tokens in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_cost_per_million: Option<f64>,

    /// Cached input cost modifier (0.1 = 90% discount).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_input_cost_modifier: Option<f64>,
}

/// Model capabilities and features.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    /// Supports text generation.
    pub text_generation: bool,
    /// Supports image generation.
    pub image_generation: bool,
    /// Supports audio processing.
    pub audio_processing: bool,
    /// supports video processing.
    pub video_processing: bool,
    /// Supports code generation.
    pub code_generation: bool,
    /// Supports function calling.
    pub function_calling: bool,
    /// Supports fine-tuning.
    pub fine_tuning: bool,
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports multi-modal inputs.
    pub multimodal: bool,
    /// Supports tool use.
    pub tool_use: bool,
    /// Additional custom capabilities.
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// AI model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Model {
    /// Unique identifier for the model.
    pub model_id: String,
    /// Human-readable name for the model.
    pub name: String,
    /// Model quantization information (e.g., "fp16", "int8", "fp32").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    /// Maximum context window size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    /// Model capabilities and features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    /// Optional temperature parameter (0.0-2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional tags for categorization and filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Token cost tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<TokenCost>,
    /// Additional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Model {
    /// Creates a new model with required fields.
    #[must_use] pub const fn new(model_id: String, name: String) -> Self {
        Self {
            model_id,
            name,
            quantization: None,
            context_window: None,
            capabilities: None,
            temperature: None,
            tags: None,
            cost: None,
            metadata: None,
        }
    }

    /// Sets the quantization for the model.
    #[must_use] pub fn with_quantization(mut self, quantization: String) -> Self {
        self.quantization = Some(quantization);
        self
    }

    /// Sets the context window size.
    #[must_use] pub const fn with_context_window(mut self, size: u32) -> Self {
        self.context_window = Some(size);
        self
    }

    /// Sets the capabilities for the model.
    #[must_use] pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Sets the temperature parameter.
    #[must_use] pub const fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the tags for the model.
    #[must_use] pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the cost tracking for the model.
    #[must_use] pub const fn with_cost(mut self, cost: TokenCost) -> Self {
        self.cost = Some(cost);
        self
    }

    /// Sets additional metadata for the model.
    #[must_use] pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Validates the model configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.model_id.is_empty() {
            return Err("Model ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }
        if let Some(window) = self.context_window {
            if window == 0 {
                return Err("Context window cannot be zero".to_string());
            }
        }
        Ok(())
    }

    /// Checks if the model supports text generation.
    #[must_use] pub fn supports_text_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .is_some_and(|caps| caps.text_generation)
    }

    /// Checks if the model supports image generation.
    #[must_use] pub fn supports_image_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .is_some_and(|caps| caps.image_generation)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = Model::new("gpt-4".to_string(), "GPT-4".to_string());

        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
        assert!(model.quantization.is_none());
        assert!(model.temperature.is_none());
        assert!(model.tags.is_none());
        assert!(model.cost.is_none());
    }

    #[test]
    fn test_model_builder() {
        let capabilities = Capabilities {
            text_generation: true,
            code_generation: true,
            ..Default::default()
        };

        let model = Model::new("claude-3".to_string(), "Claude 3".to_string())
            .with_quantization("fp16".to_string())
            .with_context_window(200_000)
            .with_capabilities(capabilities)
            .with_temperature(0.7)
            .with_tags(vec!["text-generation".to_string(), "code".to_string()]);

        assert_eq!(model.quantization, Some("fp16".to_string()));
        assert_eq!(model.context_window, Some(200_000));
        assert!(model.capabilities.is_some());
        assert_eq!(model.temperature, Some(0.7));
        assert_eq!(
            model.tags,
            Some(vec!["text-generation".to_string(), "code".to_string()])
        );
        assert!(model.supports_text_generation());
    }

    #[test]
    fn test_model_validation() {
        let valid_model = Model::new("valid".to_string(), "Valid Model".to_string());
        assert!(valid_model.validate().is_ok());

        let invalid_model = Model::new(String::new(), "Invalid Model".to_string());
        assert!(invalid_model.validate().is_err());

        let temp_model =
            Model::new("temp-test".to_string(), "Temp Model".to_string()).with_temperature(3.0);
        assert!(temp_model.validate().is_err());
    }

    #[test]
    fn test_token_cost() {
        let cost = TokenCost {
            input_cost_per_million: Some(0.001),
            output_cost_per_million: Some(0.002),
            cached_input_cost_modifier: Some(0.1),
        };

        assert_eq!(cost.input_cost_per_million, Some(0.001));
        assert_eq!(cost.output_cost_per_million, Some(0.002));
        assert_eq!(cost.cached_input_cost_modifier, Some(0.1));
    }

    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::default();
        assert!(!caps.text_generation);
        assert!(!caps.image_generation);
        assert!(!caps.code_generation);
    }
}
