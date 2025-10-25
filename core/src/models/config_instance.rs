//! ConfigInstance model for tracking multiple instances of the same application configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::DiscoveredKey;

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
    /// Optional metadata (version, settings, etc.)
    pub metadata: HashMap<String, String>,
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
            metadata: HashMap::new(),
        }
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
}
