//! Provider instance model for managing individual provider configurations with enhanced metadata.

use crate::models::{Model, ProviderKey};
use chrono::{DateTime, Utc};
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

/// Provider instance configuration with enhanced metadata and model management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInstance {
    /// Unique identifier for this instance.
    pub id: String,

    /// Human-readable display name.
    pub display_name: String,

    /// Provider type (e.g., "openai", "anthropic", "groq").
    pub provider_type: String,

    /// Base URL for API requests.
    pub base_url: String,

    /// API key associated with this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Instance-specific model configurations.
    #[serde(default)]
    pub models: Vec<Model>,

    /// Additional metadata for this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Whether this instance is active and should be used.
    pub active: bool,

    /// When this instance was created.
    pub created_at: DateTime<Utc>,

    /// When this instance was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ProviderInstance {
    /// Creates a new provider instance with required fields.
    #[must_use]
    pub fn new(id: String, display_name: String, provider_type: String, base_url: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            display_name,
            provider_type,
            base_url,
            api_key: None,
            models: Vec::new(),
            metadata: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new provider instance with cleaned metadata.
    /// This method ensures that redundant fields like `model_id` and `base_url` are not stored in metadata.
    #[must_use]
    pub fn new_with_cleaned_metadata(
        id: String,
        display_name: String,
        provider_type: String,
        base_url: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        let mut instance = Self::new(id, display_name, provider_type, base_url);

        // Clean metadata by removing redundant fields
        if let Some(meta) = metadata {
            let mut cleaned_meta = HashMap::new();
            for (key, value) in meta {
                // Skip redundant fields that should not be in metadata
                if key != "model_id" && key != "base_url" {
                    cleaned_meta.insert(key, value);
                }
            }
            if !cleaned_meta.is_empty() {
                instance.metadata = Some(cleaned_meta);
            }
        }

        instance
    }

    /// Adds a model to this instance.
    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
        self.updated_at = Utc::now();
    }

    /// Adds multiple models to this instance.
    pub fn add_models(&mut self, models: Vec<Model>) {
        self.models.extend(models);
        self.updated_at = Utc::now();
    }

    /// Sets metadata for this instance.
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the active status.
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets the API key for this instance.
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
        self.updated_at = Utc::now();
    }

    /// Gets the API key if present.
    #[must_use]
    pub const fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Checks if this instance has an API key (presence check only).
    #[must_use]
    pub const fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Checks if this instance has a non-empty API key.
    #[must_use]
    pub fn has_non_empty_api_key(&self) -> bool {
        self.api_key.as_ref().is_some_and(|key| !key.is_empty())
    }

    /// Gets the number of models.
    #[must_use]
    pub const fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Gets a model by ID.
    #[must_use]
    pub fn get_model(&self, model_id: &str) -> Option<&Model> {
        self.models.iter().find(|model| model.model_id() == model_id)
    }

    /// Validates the instance configuration.
    ///
    /// # Errors
    /// Returns an error if the instance ID, display name, or provider type is empty, or if any model validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Instance ID cannot be empty".to_string());
        }
        if self.display_name.is_empty() {
            return Err("Display name cannot be empty".to_string());
        }
        if self.provider_type.is_empty() {
            return Err("Provider type cannot be empty".to_string());
        }
        if self.base_url.is_empty() {
            return Err("Base URL cannot be empty".to_string());
        }

        // Validate models
        for model in &self.models {
            model.validate().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Gets active models (from active instance).
    #[must_use]
    pub fn active_models(&self) -> Vec<&Model> {
        if self.active {
            self.models.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Gets the name that should be used for the --key argument.
    /// This returns the instance ID which is used to identify the instance
    /// in commands like `aicred instances get <name>`.
    #[must_use]
    pub fn key_name(&self) -> &str {
        &self.id
    }

    /// Gets all tags assigned to this provider instance.
    /// This is a placeholder method that would be implemented with actual tag storage.
    #[must_use]
    pub const fn get_tags(&self) -> Vec<String> {
        // TODO: Implement actual tag retrieval from storage
        Vec::new()
    }

    /// Gets all labels assigned to this provider instance.
    /// This is a placeholder method that would be implemented with actual label storage.
    #[must_use]
    pub const fn get_labels(&self) -> Vec<String> {
        // TODO: Implement actual label retrieval from storage
        Vec::new()
    }

    /// Checks if this provider instance has a specific tag.
    /// This is a placeholder method that would be implemented with actual tag storage.
    #[must_use]
    pub const fn has_tag(&self, _tag_id: &str) -> bool {
        // TODO: Implement actual tag checking
        false
    }

    /// Checks if this provider instance has a specific label.
    /// This is a placeholder method that would be implemented with actual label storage.
    #[must_use]
    pub const fn has_label(&self, _label_id: &str) -> bool {
        // TODO: Implement actual label checking
        false
    }

    /// Gets tags assigned to a specific model in this provider instance.
    /// This is a placeholder method that would be implemented with actual tag storage.
    #[must_use]
    pub const fn get_model_tags(&self, _model_id: &str) -> Vec<String> {
        // TODO: Implement actual model tag retrieval
        Vec::new()
    }

    /// Gets labels assigned to a specific model in this provider instance.
    /// This is a placeholder method that would be implemented with actual label storage.
    #[must_use]
    pub const fn get_model_labels(&self, _model_id: &str) -> Vec<String> {
        // TODO: Implement actual model label retrieval
        Vec::new()
    }

    /// Checks if a specific model in this provider instance has a tag.
    /// This is a placeholder method that would be implemented with actual tag storage.
    #[must_use]
    pub const fn model_has_tag(&self, _model_id: &str, _tag_id: &str) -> bool {
        // TODO: Implement actual model tag checking
        false
    }

    /// Checks if a specific model in this provider instance has a label.
    /// This is a placeholder method that would be implemented with actual label storage.
    #[must_use]
    pub const fn model_has_label(&self, _model_id: &str, _label_id: &str) -> bool {
        // TODO: Implement actual model label checking
        false
    }

    /// Validates that this provider instance can accept a tag assignment.
    /// Always returns true for tags since they can be shared.
    #[must_use]
    pub const fn can_accept_tag(&self, _tag_id: &str) -> bool {
        true
    }

    /// Validates that this provider instance can accept a label assignment.
    /// Always returns true - actual uniqueness validation would be done at the storage level.
    #[must_use]
    pub const fn can_accept_label(&self, _label_id: &str) -> bool {
        true
    }

    /// Gets a display-friendly summary of tags and labels for this instance.
    #[must_use]
    pub fn get_tag_label_summary(&self) -> String {
        let tags = self.get_tags();
        let labels = self.get_labels();

        let tag_summary = if tags.is_empty() {
            "no tags".to_string()
        } else {
            format!("{} tag(s)", tags.len())
        };

        let label_summary = if labels.is_empty() {
            "no labels".to_string()
        } else {
            format!("{} label(s)", labels.len())
        };

        format!("{tag_summary} and {label_summary}")
    }
}

impl Default for ProviderInstance {
    fn default() -> Self {
        Self::new(String::new(), String::new(), String::new(), String::new())
    }
}

// From/Into traits for backward compatibility
impl From<crate::models::ProviderConfig> for ProviderInstance {
    fn from(config: crate::models::ProviderConfig) -> Self {
        let mut instance = Self::new(
            "default".to_string(),
            "Default Instance".to_string(),
            "unknown".to_string(),
            "https://api.example.com".to_string(),
        );

        // Extract the first valid key value and metadata if available
        if let Some(key) = config.keys.first() {
            if let Some(value) = &key.value {
                instance.api_key = Some(value.clone());
            }

            // Store key metadata in the instance metadata
            let mut metadata = std::collections::HashMap::new();

            // Store environment
            metadata.insert("environment".to_string(), key.environment.to_string());

            // Store confidence
            metadata.insert("confidence".to_string(), key.confidence.to_string());

            // Store validation status
            metadata.insert(
                "validation_status".to_string(),
                key.validation_status.to_string(),
            );

            // Store discovered_at timestamp
            metadata.insert("discovered_at".to_string(), key.discovered_at.to_rfc3339());

            // Store source and line number if available
            metadata.insert("source".to_string(), key.source.clone());
            if let Some(line) = key.line_number {
                metadata.insert("line_number".to_string(), line.to_string());
            }

            // Store additional key metadata if present - only if JSON serialization succeeds
            if let Some(ref key_metadata) = key.metadata {
                match serde_json::to_string(key_metadata) {
                    Ok(metadata_str) => {
                        metadata.insert("key_metadata".to_string(), metadata_str);
                    }
                    Err(e) => {
                        tracing::debug!("Failed to serialize key metadata during ProviderConfig->ProviderInstance conversion: {}", e);
                        // Omit the field rather than storing corrupted data
                    }
                }
            }

            instance.metadata = Some(metadata);
        }

        // Convert model strings to Model objects (basic conversion)
        instance.models = config
            .models
            .into_iter()
            .map(|model_id| {
                let model_name = model_id.clone();
                Model::new(model_id, model_name)
            })
            .collect();

        instance
    }
}

impl From<ProviderInstance> for crate::models::ProviderConfig {
    fn from(instance: ProviderInstance) -> Self {
        let mut config = Self::new("1.0".to_string());

        // Create ProviderKey from the api_key and preserved metadata if present
        if let Some(api_key_value) = &instance.api_key {
            let mut key = ProviderKey::new(
                "default".to_string(),
                "converted".to_string(),
                crate::models::discovered_key::Confidence::Medium,
                crate::models::provider_key::Environment::Production,
            )
            .with_value(api_key_value.clone());

            // Restore metadata from instance if available
            if let Some(ref instance_metadata) = instance.metadata {
                restore_metadata_from_instance(&mut key, instance_metadata);
            }

            config.keys = vec![key];
        }

        // Convert models back to strings
        config.models = instance
            .models
            .into_iter()
            .map(|model| model.id.clone())
            .collect();

        config
    }
}

/// Restores metadata from `ProviderInstance` to `ProviderKey`
fn restore_metadata_from_instance(
    key: &mut ProviderKey,
    instance_metadata: &std::collections::HashMap<String, String>,
) {
    // Restore environment with safe default
    if let Some(env_str) = instance_metadata.get("environment") {
        key.environment = parse_environment(env_str);
    }

    // Restore confidence with safe default
    if let Some(conf_str) = instance_metadata.get("confidence") {
        key.confidence = parse_confidence(conf_str);
    }

    // Restore validation status with safe default
    if let Some(status_str) = instance_metadata.get("validation_status") {
        key.validation_status = parse_validation_status(status_str);
    }

    // Restore discovered_at timestamp with safe fallback
    if let Some(discovered_str) = instance_metadata.get("discovered_at") {
        parse_timestamp(key, discovered_str);
    }

    // Restore source
    if let Some(source) = instance_metadata.get("source") {
        key.source.clone_from(source);
    }

    // Restore line number with safe parsing
    if let Some(line_str) = instance_metadata.get("line_number") {
        parse_line_number(key, line_str);
    }

    // Restore additional key metadata with safe JSON parsing
    if let Some(key_metadata_str) = instance_metadata.get("key_metadata") {
        parse_key_metadata(key, key_metadata_str);
    }
}

/// Parses environment string to Environment enum
fn parse_environment(env_str: &str) -> crate::models::provider_key::Environment {
    match env_str {
        "development" => crate::models::provider_key::Environment::Development,
        "staging" => crate::models::provider_key::Environment::Staging,
        "production" => crate::models::provider_key::Environment::Production,
        "testing" => crate::models::provider_key::Environment::Testing,
        custom => {
            // Handle custom environments, including malformed ones
            if custom.is_empty() {
                tracing::debug!("Empty environment string found, defaulting to Production");
                crate::models::provider_key::Environment::Production
            } else {
                crate::models::provider_key::Environment::Custom(custom.to_string())
            }
        }
    }
}

/// Parses confidence string to Confidence enum
fn parse_confidence(conf_str: &str) -> crate::models::discovered_key::Confidence {
    match conf_str {
        "Low" => crate::models::discovered_key::Confidence::Low,
        "Medium" => crate::models::discovered_key::Confidence::Medium,
        "High" => crate::models::discovered_key::Confidence::High,
        "Very High" => crate::models::discovered_key::Confidence::VeryHigh,
        unknown => {
            tracing::debug!(
                "Unknown confidence level '{}', defaulting to Medium",
                unknown
            );
            crate::models::discovered_key::Confidence::Medium
        }
    }
}

/// Parses validation status string to `ValidationStatus` enum
fn parse_validation_status(status_str: &str) -> crate::models::provider_key::ValidationStatus {
    match status_str {
        "Unknown" => crate::models::provider_key::ValidationStatus::Unknown,
        "Valid" => crate::models::provider_key::ValidationStatus::Valid,
        "Invalid" => crate::models::provider_key::ValidationStatus::Invalid,
        "Expired" => crate::models::provider_key::ValidationStatus::Expired,
        "Revoked" => crate::models::provider_key::ValidationStatus::Revoked,
        "Rate Limited" => crate::models::provider_key::ValidationStatus::RateLimited,
        unknown => {
            tracing::debug!(
                "Unknown validation status '{}', defaulting to Unknown",
                unknown
            );
            crate::models::provider_key::ValidationStatus::Unknown
        }
    }
}

/// Parses timestamp string and updates `key.discovered_at`
fn parse_timestamp(key: &mut ProviderKey, discovered_str: &str) {
    match chrono::DateTime::parse_from_rfc3339(discovered_str) {
        Ok(discovered_at) => {
            key.discovered_at = discovered_at.with_timezone(&chrono::Utc);
        }
        Err(e) => {
            tracing::debug!(
                "Failed to parse discovered_at timestamp '{}': {}, using current time",
                discovered_str,
                e
            );
            // Keep the default timestamp from ProviderKey::new
        }
    }
}

/// Parses line number string and updates `key.line_number`
fn parse_line_number(key: &mut ProviderKey, line_str: &str) {
    match line_str.parse::<u32>() {
        Ok(line) => {
            key.line_number = Some(line);
        }
        Err(e) => {
            tracing::debug!(
                "Failed to parse line_number '{}': {}, omitting field",
                line_str,
                e
            );
            // Leave line_number as None
        }
    }
}

/// Parses key metadata JSON string and updates key.metadata
fn parse_key_metadata(key: &mut ProviderKey, key_metadata_str: &str) {
    match serde_json::from_str::<serde_json::Value>(key_metadata_str) {
        Ok(key_metadata) => {
            key.metadata = Some(key_metadata);
        }
        Err(e) => {
            tracing::debug!(
                "Failed to parse key_metadata JSON '{}': {}, omitting field",
                key_metadata_str,
                e
            );
            // Leave metadata as None rather than storing corrupted data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;
    use crate::models::provider_key::{Environment, ValidationStatus};

    #[test]
    fn test_provider_instance_creation() {
        let instance = ProviderInstance::new(
            "openai-prod".to_string(),
            "OpenAI Production".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        assert_eq!(instance.id, "openai-prod");
        assert_eq!(instance.display_name, "OpenAI Production");
        assert_eq!(instance.provider_type, "openai");
        assert_eq!(instance.base_url, "https://api.openai.com");
        assert!(instance.active);
        assert!(!instance.has_non_empty_api_key());
        assert_eq!(instance.model_count(), 0);
    }

    #[test]
    fn test_provider_instance_with_api_key() {
        let mut instance = ProviderInstance::new(
            "anthropic-dev".to_string(),
            "Anthropic Development".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        instance.set_api_key("sk-ant-test123".to_string());
        let model = Model::new("claude-3-sonnet".to_string(), "Claude 3 Sonnet".to_string());
        instance.add_model(model);

        assert!(instance.has_api_key());
        assert!(instance.has_non_empty_api_key());
        assert_eq!(instance.get_api_key(), Some(&"sk-ant-test123".to_string()));
        assert_eq!(instance.model_count(), 1);
    }

    #[test]
    fn test_provider_instance_validation() {
        let valid_instance = ProviderInstance::new(
            "valid-instance".to_string(),
            "Valid Instance".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );
        assert!(valid_instance.validate().is_ok());

        let invalid_instance = ProviderInstance::new(
            String::new(),
            "Invalid Instance".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );
        assert!(invalid_instance.validate().is_err());
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
}
