//! Model data structure representing AI models with their capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct Model {
    /// Unique identifier for the model.
    pub model_id: String,
    /// Reference to the provider this model belongs to.
    pub provider_id: String,
    /// Human-readable name for the model.
    pub name: String,
    /// Model quantization information (e.g., "fp16", "int8", "fp32").
    pub quantization: Option<String>,
    /// Cost per token in USD (input tokens).
    pub cost_per_token_input: Option<f64>,
    /// Cost per token in USD (output tokens).
    pub cost_per_token_output: Option<f64>,
    /// Maximum context window size in tokens.
    pub context_window: Option<u32>,
    /// Model capabilities and features.
    pub capabilities: Option<Capabilities>,
    /// Additional metadata.
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Model {
    /// Creates a new model with required fields.
    pub fn new(model_id: String, provider_id: String, name: String) -> Self {
        Self {
            model_id,
            provider_id,
            name,
            quantization: None,
            cost_per_token_input: None,
            cost_per_token_output: None,
            context_window: None,
            capabilities: None,
            metadata: None,
        }
    }

    /// Sets the quantization for the model.
    pub fn with_quantization(mut self, quantization: String) -> Self {
        self.quantization = Some(quantization);
        self
    }

    /// Sets the cost per token for input tokens.
    pub fn with_cost_per_token_input(mut self, cost: f64) -> Self {
        self.cost_per_token_input = Some(cost);
        self
    }

    /// Sets the cost per token for output tokens.
    pub fn with_cost_per_token_output(mut self, cost: f64) -> Self {
        self.cost_per_token_output = Some(cost);
        self
    }

    /// Sets the context window size.
    pub fn with_context_window(mut self, size: u32) -> Self {
        self.context_window = Some(size);
        self
    }

    /// Sets the capabilities for the model.
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Sets additional metadata for the model.
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Validates the model configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.model_id.is_empty() {
            return Err("Model ID cannot be empty".to_string());
        }
        if self.provider_id.is_empty() {
            return Err("Provider ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        if let Some(cost) = self.cost_per_token_input {
            if cost < 0.0 {
                return Err("Input cost per token cannot be negative".to_string());
            }
        }
        if let Some(cost) = self.cost_per_token_output {
            if cost < 0.0 {
                return Err("Output cost per token cannot be negative".to_string());
            }
        }
        if let Some(window) = self.context_window {
            if window == 0 {
                return Err("Context window cannot be zero".to_string());
            }
        }
        Ok(())
    }

    /// Gets the total cost per token (input + output).
    pub fn total_cost_per_token(&self) -> Option<f64> {
        match (self.cost_per_token_input, self.cost_per_token_output) {
            (Some(input), Some(output)) => Some(input + output),
            (Some(input), None) => Some(input),
            (None, Some(output)) => Some(output),
            (None, None) => None,
        }
    }

    /// Checks if the model supports text generation.
    pub fn supports_text_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .map(|caps| caps.text_generation)
            .unwrap_or(false)
    }

    /// Checks if the model supports image generation.
    pub fn supports_image_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .map(|caps| caps.image_generation)
            .unwrap_or(false)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            provider_id: String::new(),
            name: String::new(),
            quantization: None,
            cost_per_token_input: None,
            cost_per_token_output: None,
            context_window: None,
            capabilities: None,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = Model::new(
            "gpt-4".to_string(),
            "openai".to_string(),
            "GPT-4".to_string(),
        );

        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.provider_id, "openai");
        assert_eq!(model.name, "GPT-4");
        assert!(model.quantization.is_none());
    }

    #[test]
    fn test_model_builder() {
        let capabilities = Capabilities {
            text_generation: true,
            code_generation: true,
            ..Default::default()
        };

        let model = Model::new(
            "claude-3".to_string(),
            "anthropic".to_string(),
            "Claude 3".to_string(),
        )
        .with_quantization("fp16".to_string())
        .with_context_window(200000)
        .with_capabilities(capabilities);

        assert_eq!(model.quantization, Some("fp16".to_string()));
        assert_eq!(model.context_window, Some(200000));
        assert!(model.capabilities.is_some());
        assert!(model.supports_text_generation());
    }

    #[test]
    fn test_model_validation() {
        let valid_model = Model::new(
            "valid".to_string(),
            "provider".to_string(),
            "Valid Model".to_string(),
        );
        assert!(valid_model.validate().is_ok());

        let invalid_model = Model::new(
            "".to_string(),
            "provider".to_string(),
            "Invalid Model".to_string(),
        );
        assert!(invalid_model.validate().is_err());

        let negative_cost_model = Model::new(
            "cost".to_string(),
            "provider".to_string(),
            "Cost Model".to_string(),
        )
        .with_cost_per_token_input(-0.01);
        assert!(negative_cost_model.validate().is_err());
    }

    #[test]
    fn test_total_cost_calculation() {
        let model = Model::new(
            "test".to_string(),
            "provider".to_string(),
            "Test Model".to_string(),
        )
        .with_cost_per_token_input(0.001)
        .with_cost_per_token_output(0.002);

        assert_eq!(model.total_cost_per_token(), Some(0.003));
    }
}
