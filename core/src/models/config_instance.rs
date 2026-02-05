//! `ConfigInstance` model for tracking multiple instances of the same application configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::credentials::DiscoveredCredential;
use crate::models::{ProviderInstance, ProviderCollection};

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
    pub keys: Vec<DiscoveredCredential>,
    /// Provider instances for this configuration
    #[serde(default)]
    pub provider_instances: ProviderCollection,
    /// Optional metadata (version, settings, etc.)
    pub metadata: HashMap<String, String>,
}

impl ConfigInstance {
    /// Creates a new config instance.
    #[must_use]
    pub fn new(instance_id: String, app_name: String, config_path: PathBuf) -> Self {
        Self {
            instance_id,
            app_name,
            config_path,
            discovered_at: Utc::now(),
            keys: Vec::new(),
            provider_instances: ProviderCollection::new(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new config instance with provider instances.
    #[must_use]
    pub fn with_provider_instances(mut self, provider_instances: ProviderCollection) -> Self {
        self.provider_instances = provider_instances;
        self
    }

    /// Gets a provider instance by ID.
    #[must_use]
    pub fn get_provider_instance(&self, id: &str) -> Option<&ProviderInstance> {
        self.provider_instances.get_instance(id)
    }

    /// Gets a mutable reference to a provider instance by ID.
    pub fn get_provider_instance_mut(&mut self, id: &str) -> Option<&mut ProviderInstance> {
        self.provider_instances.get_instance_mut(id)
    }

    /// Adds a provider instance to this config.
    /// # Errors
    /// Returns an error if the instance cannot be added (e.g., duplicate ID).
    pub fn add_provider_instance(&mut self, instance: ProviderInstance) -> Result<(), String> {
        self.provider_instances.add_instance(instance)
    }

    /// Adds a provider instance, replacing any existing instance with the same ID.
    pub fn add_or_replace_provider_instance(&mut self, instance: ProviderInstance) {
        self.provider_instances.add_or_replace_instance(instance);
    }

    /// Removes a provider instance by ID.
    pub fn remove_provider_instance(&mut self, id: &str) -> Option<ProviderInstance> {
        self.provider_instances.remove_instance(id)
    }

    /// Gets all provider instances for this config.
    #[must_use]
    pub fn provider_instances(&self) -> Vec<&ProviderInstance> {
        self.provider_instances.all_instances()
    }

    /// Gets active provider instances for this config.
    #[must_use]
    pub fn active_provider_instances(&self) -> Vec<&ProviderInstance> {
        self.provider_instances.active_instances()
    }

    /// Gets provider instances by type.
    #[must_use]
    pub fn provider_instances_by_type(&self, provider_type: &str) -> Vec<&ProviderInstance> {
        self.provider_instances.instances_by_type(provider_type)
    }

    /// Adds a discovered key to this instance.
    pub fn add_key(&mut self, key: DiscoveredCredential) {
        self.keys.push(key);
    }

    /// Adds multiple discovered keys to this instance.
    pub fn add_keys(&mut self, keys: Vec<DiscoveredCredential>) {
        self.keys.extend(keys);
    }

    /// Gets the total number of keys for this instance.
    #[must_use]
    pub const fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Checks if this instance has any keys.
    #[must_use]
    pub const fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// Adds metadata to this instance.
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Adds multiple metadata entries from a `HashMap`.
    #[must_use]
    pub fn with_metadata_from_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Gets a mutable reference to the metadata.
    pub const fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }

    /// Gets a reference to the metadata.
    #[must_use]
    pub const fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Gets the config path as a string.
    #[must_use]
    pub fn config_path_string(&self) -> String {
        self.config_path.display().to_string()
    }

    /// Attempts to deserialize from JSON format.
    /// # Errors
    /// Returns an error if the JSON content cannot be parsed as a valid `ConfigInstance`.
    pub fn from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Self>(content)
    }

    /// Attempts to deserialize from YAML format.
    /// # Errors
    /// Returns an error if the YAML content cannot be parsed as a valid `ConfigInstance`.
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str::<Self>(content)
    }

    /// Loads a `ConfigInstance` from a file, or creates a default one if the file is invalid.
    ///
    /// This method attempts to load and validate a configuration file. If the file cannot be
    /// loaded or validated, it deletes the invalid file and creates a new default configuration.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    /// * `instance_id` - ID for the config instance
    /// * `app_name` - Name of the application
    ///
    /// # Returns
    /// * `Ok(ConfigInstance)` - Either loaded from file or newly created default
    /// * `Err(String)` - If file operations fail
    ///
    /// # Errors
    /// Returns an error if file deletion or creation fails.
    pub fn load_or_create(
        path: &std::path::Path,
        instance_id: String,
        app_name: String,
    ) -> Result<Self, String> {
        use std::fs;

        // Try to read the file
        match fs::read_to_string(path) {
            Ok(content) => {
                // Determine format based on file extension
                let result = if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    Self::from_json(&content).map_err(|e| e.to_string())
                } else {
                    Self::from_yaml(&content).map_err(|e| e.to_string())
                };

                match result {
                    Ok(instance) => Ok(instance),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to load config from {}: {}",
                            path.display(),
                            e
                        );
                        eprintln!("Deleting invalid config and creating new default");

                        // Delete invalid file
                        if let Err(del_err) = fs::remove_file(path) {
                            return Err(format!("Failed to delete invalid config file: {del_err}"));
                        }

                        // Create and return new default instance
                        Ok(Self::new(
                            instance_id,
                            app_name,
                            path.parent()
                                .unwrap_or_else(|| std::path::Path::new("/"))
                                .to_path_buf(),
                        ))
                    }
                }
            }
            Err(_) => {
                // File doesn't exist, create new default instance
                Ok(Self::new(
                    instance_id,
                    app_name,
                    path.parent()
                        .unwrap_or_else(|| std::path::Path::new("/"))
                        .to_path_buf(),
                ))
            }
        }
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
    use crate::models::credentials::{Confidence, ValueType};

    fn create_test_key() -> DiscoveredCredential {
        DiscoveredCredential::new_redacted(
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
    fn test_current_format_deserialization() {
        let current_content = r#"
instance_id: "test-instance"
app_name: "test-app"
config_path: "/test/path"
discovered_at: "2024-01-01T00:00:00Z"
keys: []
provider_instances: {}
metadata: {}
        "#;

        let result = ConfigInstance::from_yaml(current_content);
        assert!(result.is_ok());

        let instance = result.unwrap();
        assert_eq!(instance.instance_id, "test-instance");
        assert_eq!(instance.app_name, "test-app");
        assert_eq!(instance.provider_instances.len(), 0);
    }

    #[test]
    fn test_load_or_create_with_nonexistent_file() {
        use std::path::PathBuf;

        let path = PathBuf::from("/nonexistent/config.yaml");
        let result =
            ConfigInstance::load_or_create(&path, "test-id".to_string(), "test-app".to_string());

        assert!(result.is_ok());
        let instance = result.unwrap();
        assert_eq!(instance.instance_id, "test-id");
        assert_eq!(instance.app_name, "test-app");
    }
}
