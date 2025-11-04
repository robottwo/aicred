//! Provider-Model Tuple utilities for parsing and identifying provider:model combinations.

use crate::models::{Model, ProviderInstance};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a parsed provider:model tuple identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProviderModelTuple {
    /// Provider identifier (either provider type or instance ID)
    pub provider: String,
    /// Model identifier
    pub model: String,
}

impl ProviderModelTuple {
    /// Creates a new provider:model tuple.
    #[must_use]
    pub const fn new(provider: String, model: String) -> Self {
        Self { provider, model }
    }

    /// Parses a provider:model tuple from a string in the format `{provider}:{model}`.
    ///
    /// # Arguments
    /// * `input` - String in format `{provider}:{model}` where provider can be either
    ///   a provider type (e.g., "openai") or provider instance ID (e.g., "openai-prod")
    ///
    /// # Returns
    /// Result containing the parsed tuple or an error message
    ///
    /// # Errors
    /// Returns an error if:
    /// - The input is empty
    /// - The input doesn't contain a ':' separator
    /// - The provider or model identifier is empty
    /// - The provider or model identifier contains ':'
    ///
    /// # Examples
    /// ```
    /// # use aicred_core::utils::ProviderModelTuple;
    /// let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    /// assert_eq!(tuple.provider, "openai");
    /// assert_eq!(tuple.model, "gpt-4");
    ///
    /// let tuple = ProviderModelTuple::parse("openai-prod:gpt-4").unwrap();
    /// assert_eq!(tuple.provider, "openai-prod");
    /// assert_eq!(tuple.model, "gpt-4");
    /// ```
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();

        if input.is_empty() {
            return Err("Provider:model tuple cannot be empty".to_string());
        }

        let colon_pos = input
            .find(':')
            .ok_or_else(|| "Provider:model tuple must contain ':' separator".to_string())?;

        if colon_pos == 0 {
            return Err("Provider identifier cannot be empty".to_string());
        }

        if colon_pos == input.len() - 1 {
            return Err("Model identifier cannot be empty".to_string());
        }

        let provider = input[..colon_pos].trim().to_string();
        let model = input[colon_pos + 1..].trim().to_string();

        if provider.is_empty() {
            return Err("Provider identifier cannot be empty".to_string());
        }

        if model.is_empty() {
            return Err("Model identifier cannot be empty".to_string());
        }

        // Validate that provider and model don't contain invalid characters
        if provider.contains(':') {
            return Err("Provider identifier cannot contain ':'".to_string());
        }

        if model.contains(':') {
            return Err("Model identifier cannot contain ':'".to_string());
        }

        Ok(Self { provider, model })
    }

    /// Gets the provider identifier.
    #[must_use]
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Gets the model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Checks if this tuple matches a provider instance and model.
    ///
    /// # Arguments
    /// * `instance` - The provider instance to match against
    /// * `model_id` - Optional model ID to match against (None for provider-level matching)
    ///
    /// # Returns
    /// True if this tuple matches the instance and model
    #[must_use]
    pub fn matches(&self, instance: &ProviderInstance, model_id: Option<&str>) -> bool {
        // Check if provider matches either the instance ID or provider type
        let provider_matches =
            self.provider == instance.id || self.provider == instance.provider_type;

        if !provider_matches {
            return false;
        }

        // If no model specified, this is a provider-level match
        if model_id.is_none() {
            return true;
        }

        // Check if the model exists in this instance
        instance.get_model(&self.model).is_some()
    }

    /// Creates a string representation of this tuple.
    #[must_use]
    pub fn as_str(&self) -> String {
        format!("{}:{}", self.provider, self.model)
    }

    /// Creates a human-readable description of this tuple.
    #[must_use]
    pub fn description(&self) -> String {
        format!("provider:model tuple '{}'", self.as_str())
    }
}

impl fmt::Display for ProviderModelTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.provider, self.model)
    }
}

impl From<(String, String)> for ProviderModelTuple {
    fn from((provider, model): (String, String)) -> Self {
        Self::new(provider, model)
    }
}

impl From<ProviderModelTuple> for String {
    fn from(tuple: ProviderModelTuple) -> Self {
        tuple.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProviderInstance;

    #[test]
    fn test_parse_valid_tuples() {
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        assert_eq!(tuple.provider, "openai");
        assert_eq!(tuple.model, "gpt-4");

        let tuple = ProviderModelTuple::parse("openai-prod:gpt-4").unwrap();
        assert_eq!(tuple.provider, "openai-prod");
        assert_eq!(tuple.model, "gpt-4");

        let tuple = ProviderModelTuple::parse("anthropic:claude-3-sonnet").unwrap();
        assert_eq!(tuple.provider, "anthropic");
        assert_eq!(tuple.model, "claude-3-sonnet");
    }

    #[test]
    fn test_parse_with_whitespace() {
        let tuple = ProviderModelTuple::parse(" openai : gpt-4 ").unwrap();
        assert_eq!(tuple.provider, "openai");
        assert_eq!(tuple.model, "gpt-4");
    }

    #[test]
    fn test_parse_invalid_tuples() {
        // Missing colon
        assert!(ProviderModelTuple::parse("openai").is_err());

        // Empty provider
        assert!(ProviderModelTuple::parse(":gpt-4").is_err());

        // Empty model
        assert!(ProviderModelTuple::parse("openai:").is_err());

        // Empty string
        assert!(ProviderModelTuple::parse("").is_err());

        // Provider contains colon
        assert!(ProviderModelTuple::parse("open:ai:gpt-4").is_err());

        // Model contains colon
        assert!(ProviderModelTuple::parse("openai:gpt:4").is_err());
    }

    #[test]
    fn test_display() {
        let tuple = ProviderModelTuple::new("openai".to_string(), "gpt-4".to_string());
        assert_eq!(tuple.to_string(), "openai:gpt-4");
    }

    #[test]
    fn test_matches_provider_instance() {
        let instance = ProviderInstance::new(
            "openai-prod".to_string(),
            "OpenAI Production".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // Add a model to the instance
        let model = Model::new("gpt-4".to_string(), "GPT-4".to_string());
        let mut instance = instance;
        instance.add_model(model);

        // Test provider type match
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        assert!(tuple.matches(&instance, Some("gpt-4")));

        // Test instance ID match
        let tuple = ProviderModelTuple::parse("openai-prod:gpt-4").unwrap();
        assert!(tuple.matches(&instance, Some("gpt-4")));

        // Test provider-level match (no model specified)
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        assert!(tuple.matches(&instance, None));

        // Test non-matching provider
        let tuple = ProviderModelTuple::parse("anthropic:gpt-4").unwrap();
        assert!(!tuple.matches(&instance, Some("gpt-4")));

        // Test non-matching model
        let tuple = ProviderModelTuple::parse("openai:gpt-3.5").unwrap();
        assert!(!tuple.matches(&instance, Some("gpt-3.5")));
    }

    #[test]
    fn test_from_tuple() {
        let tuple = ProviderModelTuple::from(("openai".to_string(), "gpt-4".to_string()));
        assert_eq!(tuple.provider, "openai");
        assert_eq!(tuple.model, "gpt-4");
    }

    #[test]
    fn test_into_string() {
        let tuple = ProviderModelTuple::new("openai".to_string(), "gpt-4".to_string());
        let s: String = tuple.into();
        assert_eq!(s, "openai:gpt-4");
    }

    #[test]
    fn test_as_str() {
        let tuple = ProviderModelTuple::new("openai".to_string(), "gpt-4".to_string());
        assert_eq!(tuple.as_str(), "openai:gpt-4");
    }
    #[test]
    fn test_provider_model_tuple_scenarios() {
        // Test various provider:model combinations
        let test_cases = vec![
            ("openai:gpt-4", "openai", "gpt-4"),
            ("anthropic:claude-3-sonnet", "anthropic", "claude-3-sonnet"),
            ("google:gemini-pro", "google", "gemini-pro"),
            ("openai-prod:gpt-4-turbo", "openai-prod", "gpt-4-turbo"),
            ("local-ollama:llama2", "local-ollama", "llama2"),
        ];

        for (input, expected_provider, expected_model) in test_cases {
            let tuple = ProviderModelTuple::parse(input).unwrap();
            assert_eq!(tuple.provider(), expected_provider);
            assert_eq!(tuple.model(), expected_model);
            assert_eq!(tuple.as_str(), input);
        }
    }

    #[test]
    fn test_provider_model_tuple_edge_cases() {
        // Test with special characters in provider/model names
        let tuple = ProviderModelTuple::parse("provider-name:model-name").unwrap();
        assert_eq!(tuple.provider(), "provider-name");
        assert_eq!(tuple.model(), "model-name");

        let tuple = ProviderModelTuple::parse("provider_name:model_name").unwrap();
        assert_eq!(tuple.provider(), "provider_name");
        assert_eq!(tuple.model(), "model_name");

        let tuple = ProviderModelTuple::parse("provider123:model456").unwrap();
        assert_eq!(tuple.provider(), "provider123");
        assert_eq!(tuple.model(), "model456");
    }

    #[test]
    fn test_provider_model_tuple_equality() {
        let tuple1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple2 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple3 = ProviderModelTuple::parse("anthropic:claude-3").unwrap();

        assert_eq!(tuple1, tuple2);
        assert_ne!(tuple1, tuple3);
    }

    #[test]
    fn test_provider_model_tuple_hash() {
        use std::collections::HashMap;

        let tuple1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple2 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple3 = ProviderModelTuple::parse("anthropic:claude-3").unwrap();

        let mut map = HashMap::new();
        map.insert(tuple1.clone(), "value1");
        map.insert(tuple3.clone(), "value2");

        // tuple2 should not create a new entry since it's equal to tuple1
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&tuple1), Some(&"value1"));
        assert_eq!(map.get(&tuple2), Some(&"value1")); // Should find the same value
        assert_eq!(map.get(&tuple3), Some(&"value2"));
    }

    #[test]
    fn test_provider_model_tuple_display_format() {
        let tuple = ProviderModelTuple::new("test-provider".to_string(), "test-model".to_string());

        // Test Display trait
        assert_eq!(format!("{tuple}"), "test-provider:test-model");

        // Test to_string
        assert_eq!(tuple.to_string(), "test-provider:test-model");
    }

    #[test]
    fn test_provider_model_tuple_from_tuple() {
        let input = ("openai".to_string(), "gpt-4".to_string());
        let tuple = ProviderModelTuple::from(input);

        assert_eq!(tuple.provider(), "openai");
        assert_eq!(tuple.model(), "gpt-4");
    }

    #[test]
    fn test_provider_model_tuple_into_string() {
        let tuple = ProviderModelTuple::new("openai".to_string(), "gpt-4".to_string());
        let s: String = tuple.into();

        assert_eq!(s, "openai:gpt-4");
    }
}
