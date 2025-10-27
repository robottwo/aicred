//! Provider instance model for managing individual provider configurations with enhanced metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{Model, ProviderKey};

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
    
    /// API keys associated with this instance.
    #[serde(default)]
    pub keys: Vec<ProviderKey>,
    
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
    pub fn new(
        id: String,
        display_name: String,
        provider_type: String,
        base_url: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            display_name,
            provider_type,
            base_url,
            keys: Vec::new(),
            models: Vec::new(),
            metadata: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Adds a key to this instance.
    pub fn add_key(&mut self, key: ProviderKey) {
        self.keys.push(key);
        self.updated_at = Utc::now();
    }
    
    /// Adds multiple keys to this instance.
    pub fn add_keys(&mut self, keys: Vec<ProviderKey>) {
        self.keys.extend(keys);
        self.updated_at = Utc::now();
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
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Sets the active status.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    
    /// Gets the number of keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
    
    /// Gets the number of valid keys.
    pub fn valid_key_count(&self) -> usize {
        self.keys.iter().filter(|key| key.is_valid()).count()
    }
    
    /// Gets the number of models.
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
    
    /// Gets a key by ID.
    pub fn get_key(&self, id: &str) -> Option<&ProviderKey> {
        self.keys.iter().find(|key| key.id == id)
    }
    
    /// Gets the default key (first valid key or first key).
    pub fn default_key(&self) -> Option<&ProviderKey> {
        self.keys.iter().find(|key| key.is_valid())
            .or_else(|| self.keys.first())
    }
    
    /// Gets a model by ID.
    pub fn get_model(&self, model_id: &str) -> Option<&Model> {
        self.models.iter().find(|model| model.model_id == model_id)
    }
    
    /// Validates the instance configuration.
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
            model.validate()?;
        }
        
        Ok(())
    }
    
    /// Checks if this instance has any valid keys.
    pub fn has_valid_keys(&self) -> bool {
        self.keys.iter().any(|key| key.is_valid())
    }
    
    /// Gets active models (from active instance).
    pub fn active_models(&self) -> Vec<&Model> {
        if self.active {
            self.models.iter().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for ProviderInstance {
    fn default() -> Self {
        Self::new(
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        )
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
        
        // Migrate keys
        instance.keys = config.keys;
        
        // Convert model strings to Model objects (basic conversion)
        instance.models = config.models.into_iter()
            .map(|model_id| {
                let model_name = model_id.clone();
                Model::new(model_id, instance.id.clone(), model_name)
            })
            .collect();
        
        instance
    }
}

impl From<ProviderInstance> for crate::models::ProviderConfig {
    fn from(instance: ProviderInstance) -> Self {
        let mut config = Self::new("1.0".to_string());
        
        // Migrate keys
        config.keys = instance.keys;
        
        // Convert models back to strings
        config.models = instance.models.into_iter()
            .map(|model| model.model_id)
            .collect();
        
        config
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
        assert_eq!(instance.key_count(), 0);
        assert_eq!(instance.model_count(), 0);
    }
    
    #[test]
    fn test_provider_instance_with_data() {
        let mut instance = ProviderInstance::new(
            "anthropic-dev".to_string(),
            "Anthropic Development".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );
        
        let mut key = ProviderKey::new(
            "dev-key".to_string(),
            "/config/anthropic".to_string(),
            Confidence::High,
            Environment::Development,
        );
        key.set_validation_status(ValidationStatus::Valid);
        
        let model = Model::new(
            "claude-3-sonnet".to_string(),
            instance.id.clone(),
            "Claude 3 Sonnet".to_string(),
        );
        
        instance.add_key(key);
        instance.add_model(model);
        
        assert_eq!(instance.key_count(), 1);
        assert_eq!(instance.model_count(), 1);
        assert!(instance.has_valid_keys());
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
            "".to_string(),
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