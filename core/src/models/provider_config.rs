//! Provider configuration model supporting multiple API keys.
//! 
//! # Deprecated
//! This module is deprecated in favor of [`ProviderInstance`](crate::models::ProviderInstance) and [`ProviderInstances`](crate::models::ProviderInstances).
//! Use the new provider instance system for enhanced metadata and model management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::provider_key::{Environment, ProviderKey, ValidationStatus};

/// Default schema version for backward compatibility.
fn default_schema_version() -> String {
    "3.0".to_string()
}

/// Provider configuration supporting multiple API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// List of API keys for this provider.
    #[serde(default)]
    pub keys: Vec<ProviderKey>,
    
    /// List of models available for this provider.
    pub models: Vec<String>,
    
    /// Additional provider metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
    
    /// Configuration version.
    pub version: String,
    
    /// Schema version for migration tracking.
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    
    /// When this configuration was created.
    pub created_at: DateTime<Utc>,
    
    /// When this configuration was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ProviderConfig {
    /// Creates a new provider configuration with default values.
    pub fn new(version: String) -> Self {
        let now = Utc::now();
        Self {
            keys: Vec::new(),
            models: Vec::new(),
            metadata: None,
            version,
            schema_version: "3.0".to_string(), // New schema version
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Adds a key to the provider configuration.
    pub fn add_key(&mut self, key: ProviderKey) {
        self.keys.push(key);
        self.updated_at = Utc::now();
    }
    
    /// Adds multiple keys to the provider configuration.
    pub fn add_keys(&mut self, keys: Vec<ProviderKey>) {
        self.keys.extend(keys);
        self.updated_at = Utc::now();
    }
    
    /// Gets the number of keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
    
    /// Gets the number of valid keys.
    pub fn valid_key_count(&self) -> usize {
        self.keys.iter().filter(|key| key.is_valid()).count()
    }
    
    /// Gets keys by environment.
    pub fn keys_by_environment(&self, env: &Environment) -> Vec<&ProviderKey> {
        self.keys.iter().filter(|key| &key.environment == env).collect()
    }
    
    /// Gets the default key (first key with id "default" or first key).
    pub fn default_key(&self) -> Option<&ProviderKey> {
        self.keys.iter().find(|key| key.id == "default")
            .or_else(|| self.keys.first())
    }
    
    /// Gets a key by ID.
    pub fn get_key(&self, id: &str) -> Option<&ProviderKey> {
        self.keys.iter().find(|key| key.id == id)
    }
    
    /// Gets keys by validation status.
    pub fn keys_by_status(&self, status: ValidationStatus) -> Vec<&ProviderKey> {
        self.keys.iter().filter(|key| key.validation_status == status).collect()
    }
    
    /// Converts from old single-key format to new multi-key format.
    pub fn from_old_format(
        api_key: Option<String>,
        models: Vec<String>,
        metadata: Option<HashMap<String, serde_yaml::Value>>,
        version: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        let mut new_config = Self::new(version);
        new_config.models = models;
        new_config.metadata = metadata;
        new_config.created_at = created_at;
        new_config.updated_at = updated_at;
        
        // Convert single api_key to keys list
        if let Some(api_key_value) = api_key {
            let mut key = ProviderKey::new(
                "default".to_string(),
                "migrated".to_string(),
                crate::models::discovered_key::Confidence::High,
                Environment::Production,
            );
            key.value = Some(api_key_value);
            key.discovered_at = created_at;
            key.validation_status = ValidationStatus::Valid;
            new_config.add_key(key);
        }
        
        new_config
    }
}

/// Helper structure for reading old format configurations.
#[derive(Debug, Clone, Deserialize)]
struct OldProviderConfig {
    api_key: Option<String>,
    models: Vec<String>,
    metadata: Option<HashMap<String, serde_yaml::Value>>,
    version: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProviderConfig {
    /// Attempts to deserialize from either old or new format.
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        // First check if this is old format by looking for api_key field
        if content.contains("api_key:") && !content.contains("keys:") {
            // This is old format, deserialize and convert
            let old_config: OldProviderConfig = serde_yaml::from_str(content)?;
            return Ok(Self::from_old_format(
                old_config.api_key,
                old_config.models,
                old_config.metadata,
                old_config.version,
                old_config.created_at,
                old_config.updated_at,
            ));
        }
        
        // Try to deserialize as new format
        serde_yaml::from_str::<ProviderConfig>(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new("1.0".to_string());
        
        assert_eq!(config.version, "1.0");
        assert_eq!(config.schema_version, "3.0");
        assert_eq!(config.key_count(), 0);
        assert_eq!(config.valid_key_count(), 0);
    }
    
    #[test]
    fn test_add_keys() {
        let mut config = ProviderConfig::new("1.0".to_string());
        
        let key1 = ProviderKey::new(
            "key1".to_string(),
            "/test/path1".to_string(),
            Confidence::High,
            Environment::Production,
        );
        
        let key2 = ProviderKey::new(
            "key2".to_string(),
            "/test/path2".to_string(),
            Confidence::Medium,
            Environment::Development,
        );
        
        config.add_key(key1);
        config.add_key(key2);
        
        assert_eq!(config.key_count(), 2);
    }
    
    #[test]
    fn test_keys_by_environment() {
        let mut config = ProviderConfig::new("1.0".to_string());
        
        let prod_key = ProviderKey::new(
            "prod-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        
        let dev_key = ProviderKey::new(
            "dev-key".to_string(),
            "/test/path2".to_string(),
            Confidence::High,
            Environment::Development,
        );
        
        config.add_key(prod_key);
        config.add_key(dev_key);
        
        let prod_keys = config.keys_by_environment(&Environment::Production);
        let dev_keys = config.keys_by_environment(&Environment::Development);
        
        assert_eq!(prod_keys.len(), 1);
        assert_eq!(dev_keys.len(), 1);
    }
    
    #[test]
    fn test_default_key() {
        let mut config = ProviderConfig::new("1.0".to_string());
        
        let default_key = ProviderKey::new(
            "default".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        
        let other_key = ProviderKey::new(
            "other".to_string(),
            "/test/path2".to_string(),
            Confidence::High,
            Environment::Development,
        );
        
        config.add_key(other_key);
        config.add_key(default_key);
        
        let default_key_ref = config.default_key();
        assert!(default_key_ref.is_some());
        assert_eq!(default_key_ref.unwrap().id, "default");
    }

}