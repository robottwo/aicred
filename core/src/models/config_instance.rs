//! ConfigInstance model for tracking multiple instances of the same application configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::{DiscoveredKey, ProviderInstances, ProviderInstance, ProviderConfig, ProviderConfigMigrator, MigrationConfig};

/// Represents a specific instance of an application configuration
/// For example, multiple Roo Code installations in different directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInstance {
    /// Unique identifier for this instance
    pub instance_id: String,
    /// The application/tool name (e.g., "roo-code", "claude-desktop")
    pub app_name: String,
    /// Installation/config directory path
    pub config_path: PathBuf,
    /// When this instance was discovered
    pub discovered_at: DateTime<Utc>,
    /// Associated discovered keys for this instance
    pub keys: Vec<DiscoveredKey>,
    /// Provider instances for this configuration
    #[serde(default)]
    pub provider_instances: ProviderInstances,
    /// Optional metadata (version, settings, etc.)
    pub metadata: HashMap<String, String>,
}

/// Helper structure for backward compatibility with old format
#[derive(Debug, Clone, Deserialize)]
struct LegacyConfigInstance {
    instance_id: String,
    app_name: String,
    config_path: PathBuf,
    discovered_at: DateTime<Utc>,
    keys: Vec<DiscoveredKey>,
    #[serde(default)]
    providers: Vec<ProviderConfig>,
    metadata: HashMap<String, String>,
}

/// Helper structure for current format without provider_instances
#[derive(Debug, Clone, Deserialize)]
struct CurrentConfigInstanceWithoutProviders {
    instance_id: String,
    app_name: String,
    config_path: PathBuf,
    discovered_at: DateTime<Utc>,
    keys: Vec<DiscoveredKey>,
    metadata: HashMap<String, String>,
}

impl ConfigInstance {
    /// Creates a new config instance.
    pub fn new(instance_id: String, app_name: String, config_path: PathBuf) -> Self {
        Self {
            instance_id,
            app_name,
            config_path,
            discovered_at: Utc::now(),
            keys: Vec::new(),
            provider_instances: ProviderInstances::new(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new config instance with provider instances.
    pub fn with_provider_instances(mut self, provider_instances: ProviderInstances) -> Self {
        self.provider_instances = provider_instances;
        self
    }

    /// Gets a provider instance by ID.
    pub fn get_provider_instance(&self, id: &str) -> Option<&ProviderInstance> {
        self.provider_instances.get_instance(id)
    }

    /// Gets a mutable reference to a provider instance by ID.
    pub fn get_provider_instance_mut(&mut self, id: &str) -> Option<&mut ProviderInstance> {
        self.provider_instances.get_instance_mut(id)
    }

    /// Adds a provider instance to this config.
    pub fn add_provider_instance(&mut self, instance: ProviderInstance) -> Result<(), String> {
        self.provider_instances.add_instance(instance)
    }

    /// Adds a provider instance, replacing any existing instance with the same ID.
    pub fn add_or_replace_provider_instance(&mut self, instance: ProviderInstance) {
        self.provider_instances.add_or_replace_instance(instance)
    }

    /// Removes a provider instance by ID.
    pub fn remove_provider_instance(&mut self, id: &str) -> Option<ProviderInstance> {
        self.provider_instances.remove_instance(id)
    }

    /// Gets all provider instances for this config.
    pub fn provider_instances(&self) -> Vec<&ProviderInstance> {
        self.provider_instances.all_instances()
    }

    /// Gets active provider instances for this config.
    pub fn active_provider_instances(&self) -> Vec<&ProviderInstance> {
        self.provider_instances.active_instances()
    }

    /// Gets provider instances by type.
    pub fn provider_instances_by_type(&self, provider_type: &str) -> Vec<&ProviderInstance> {
        self.provider_instances.instances_by_type(provider_type)
    }

    /// Migrates from old format with Vec<ProviderConfig> to new format with ProviderInstances.
    fn from_legacy_format(legacy: LegacyConfigInstance) -> Self {
        let mut config_instance = Self {
            instance_id: legacy.instance_id,
            app_name: legacy.app_name,
            config_path: legacy.config_path,
            discovered_at: legacy.discovered_at,
            keys: legacy.keys,
            provider_instances: ProviderInstances::new(),
            metadata: legacy.metadata,
        };

        // Convert ProviderConfig instances to ProviderInstance using the migrator
        let migration_config = MigrationConfig::default();
        
        for (index, provider_config) in legacy.providers.into_iter().enumerate() {
            // Try to detect provider type and base URL from the config
            let provider_type = Self::detect_provider_type_from_config(&provider_config)
                .unwrap_or_else(|| "unknown".to_string());
            let base_url = Self::detect_base_url_from_config(&provider_config)
                .unwrap_or_else(|| "https://api.example.com".to_string());

            match ProviderConfigMigrator::migrate_config(
                provider_config,
                &provider_type,
                &base_url,
                index,
                &migration_config,
            ) {
                Ok(instance) => {
                    let _ = config_instance.provider_instances.add_instance(instance);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to migrate provider config {}: {}", index, e);
                }
            }
        }

        config_instance
    }

    /// Detects provider type from ProviderConfig metadata
    fn detect_provider_type_from_config(config: &ProviderConfig) -> Option<String> {
        // Check if metadata contains provider type information
        if let Some(metadata) = &config.metadata {
            if let Some(provider_type) = metadata.get("provider_type") {
                if let Some(type_str) = provider_type.as_str() {
                    return Some(type_str.to_string());
                }
            }
        }
        
        // Default detection based on version or other heuristics
        None
    }

    /// Detects base URL from ProviderConfig metadata
    fn detect_base_url_from_config(config: &ProviderConfig) -> Option<String> {
        // Check if metadata contains base URL information
        if let Some(metadata) = &config.metadata {
            if let Some(base_url) = metadata.get("base_url") {
                if let Some(url_str) = base_url.as_str() {
                    return Some(url_str.to_string());
                }
            }
        }
        
        None
    }

    /// Adds a discovered key to this instance.
    pub fn add_key(&mut self, key: DiscoveredKey) {
        self.keys.push(key);
    }

    /// Adds multiple discovered keys to this instance.
    pub fn add_keys(&mut self, keys: Vec<DiscoveredKey>) {
        self.keys.extend(keys);
    }

    /// Gets the total number of keys for this instance.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Checks if this instance has any keys.
    pub fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// Adds metadata to this instance.
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Adds multiple metadata entries from a HashMap.
    pub fn with_metadata_from_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Gets a mutable reference to the metadata.
    pub fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }

    /// Gets a reference to the metadata.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Gets the config path as a string.
    pub fn config_path_string(&self) -> String {
        self.config_path.display().to_string()
    }

    /// Attempts to deserialize from either old or new format with automatic migration.
    pub fn from_json(content: &str) -> Result<Self, serde_json::Error> {
        // First try to deserialize as current format
        if let Ok(current) = serde_json::from_str::<ConfigInstance>(content) {
            return Ok(current);
        }

        // Try to deserialize as legacy format with Vec<ProviderConfig>
        if let Ok(legacy) = serde_json::from_str::<LegacyConfigInstance>(content) {
            return Ok(Self::from_legacy_format(legacy));
        }

        // Try to deserialize as format without provider_instances field
        if let Ok(without_providers) = serde_json::from_str::<CurrentConfigInstanceWithoutProviders>(content) {
            return Ok(Self {
                instance_id: without_providers.instance_id,
                app_name: without_providers.app_name,
                config_path: without_providers.config_path,
                discovered_at: without_providers.discovered_at,
                keys: without_providers.keys,
                provider_instances: ProviderInstances::new(),
                metadata: without_providers.metadata,
            });
        }

        // If all else fails, try standard deserialization
        serde_json::from_str::<ConfigInstance>(content)
    }

    /// Attempts to deserialize from YAML format with automatic migration.
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        // Check if content contains legacy 'providers' field
        if content.contains("providers:") && !content.contains("provider_instances:") {
            eprintln!("DEBUG: Detected legacy format with 'providers' field");
            // Try to deserialize as legacy format with Vec<ProviderConfig>
            match serde_yaml::from_str::<LegacyConfigInstance>(content) {
                Ok(legacy) => {
                    eprintln!("DEBUG: Successfully parsed as LegacyConfigInstance");
                    return Ok(Self::from_legacy_format(legacy));
                }
                Err(e) => {
                    eprintln!("DEBUG: Failed to parse as LegacyConfigInstance: {}", e);
                }
            }
        }

        // First try to deserialize as current format
        match serde_yaml::from_str::<ConfigInstance>(content) {
            Ok(current) => {
                eprintln!("DEBUG: Successfully parsed as current ConfigInstance format");
                return Ok(current);
            }
            Err(e) => {
                eprintln!("DEBUG: Failed to parse as current ConfigInstance: {}", e);
            }
        }

        // Try to deserialize as format without provider_instances field
        if let Ok(without_providers) = serde_yaml::from_str::<CurrentConfigInstanceWithoutProviders>(content) {
            return Ok(Self {
                instance_id: without_providers.instance_id,
                app_name: without_providers.app_name,
                config_path: without_providers.config_path,
                discovered_at: without_providers.discovered_at,
                keys: without_providers.keys,
                provider_instances: ProviderInstances::new(),
                metadata: without_providers.metadata,
            });
        }

        // If all else fails, try standard deserialization
        serde_yaml::from_str::<ConfigInstance>(content)
    }

    /// Checks if the provided content contains legacy format that needs migration.
    pub fn needs_migration(content: &str) -> bool {
        ProviderConfigMigrator::is_legacy_format(content)
    }

    /// Performs migration on the provided content and returns a new ConfigInstance.
    pub fn migrate_from_legacy(content: &str) -> Result<Self, String> {
        // Try to deserialize as legacy format first
        if let Ok(legacy) = serde_yaml::from_str::<LegacyConfigInstance>(content) {
            return Ok(Self::from_legacy_format(legacy));
        }
        
        if let Ok(legacy) = serde_json::from_str::<LegacyConfigInstance>(content) {
            return Ok(Self::from_legacy_format(legacy));
        }
        
        Err("Content is not in a recognized legacy format".to_string())
    }
}

impl Default for ConfigInstance {
    fn default() -> Self {
        Self::new(
            "default".to_string(),
            "unknown".to_string(),
            PathBuf::from("/"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::{Confidence, ValueType};

    fn create_test_key() -> DiscoveredKey {
        DiscoveredKey::new_redacted(
            "test-provider".to_string(),
            "/test/path".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "sk-test123",
        )
    }

    #[test]
    fn test_config_instance_creation() {
        let instance = ConfigInstance::new(
            "test-123".to_string(),
            "test-app".to_string(),
            PathBuf::from("/test/path"),
        );

        assert_eq!(instance.instance_id, "test-123");
        assert_eq!(instance.app_name, "test-app");
        assert_eq!(instance.config_path, PathBuf::from("/test/path"));
        assert_eq!(instance.key_count(), 0);
        assert!(!instance.has_keys());
    }

    #[test]
    fn test_adding_keys() {
        let mut instance = ConfigInstance::new(
            "test-123".to_string(),
            "test-app".to_string(),
            PathBuf::from("/test/path"),
        );

        let key1 = create_test_key();
        let key2 = create_test_key();

        instance.add_key(key1);
        instance.add_key(key2);

        assert_eq!(instance.key_count(), 2);
        assert!(instance.has_keys());
    }

    #[test]
    fn test_adding_multiple_keys() {
        let mut instance = ConfigInstance::new(
            "test-123".to_string(),
            "test-app".to_string(),
            PathBuf::from("/test/path"),
        );

        let keys = vec![create_test_key(), create_test_key(), create_test_key()];
        instance.add_keys(keys);

        assert_eq!(instance.key_count(), 3);
    }

    #[test]
    fn test_metadata() {
        let mut instance = ConfigInstance::new(
            "test-123".to_string(),
            "test-app".to_string(),
            PathBuf::from("/test/path"),
        );

        instance.add_metadata("version".to_string(), "1.0.0".to_string());
        instance.add_metadata("platform".to_string(), "macos".to_string());

        assert_eq!(instance.metadata().len(), 2);
        assert_eq!(
            instance.metadata().get("version"),
            Some(&"1.0.0".to_string())
        );
        assert_eq!(
            instance.metadata().get("platform"),
            Some(&"macos".to_string())
        );
    }

    #[test]
    fn test_config_path_string() {
        let instance = ConfigInstance::new(
            "test-123".to_string(),
            "test-app".to_string(),
            PathBuf::from("/test/path/config.json"),
        );

        assert_eq!(instance.config_path_string(), "/test/path/config.json");
    }

    #[test]
    fn test_legacy_format_migration() {
        let legacy_content = r#"
        instance_id: "test-instance"
        app_name: "test-app"
        config_path: "/test/path"
        discovered_at: "2024-01-01T00:00:00Z"
        keys: []
        providers:
          - keys: []
            models: ["gpt-4", "gpt-3.5-turbo"]
            version: "1.0"
            schema_version: "3.0"
            created_at: "2024-01-01T00:00:00Z"
            updated_at: "2024-01-01T00:00:00Z"
        metadata: {}
        "#;

        let result = ConfigInstance::from_yaml(legacy_content);
        assert!(result.is_ok());
        
        let instance = result.unwrap();
        assert_eq!(instance.instance_id, "test-instance");
        assert_eq!(instance.app_name, "test-app");
        assert!(instance.provider_instances.len() > 0);
    }

    #[test]
    fn test_needs_migration_detection() {
        let legacy_content = r#"
        api_key: sk-test123
        models: ["gpt-4"]
        version: "1.0"
        "#;

        assert!(ConfigInstance::needs_migration(legacy_content));
    }
}
