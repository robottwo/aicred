//! Migration utilities for converting from legacy `ProviderConfig` format to new `ProviderInstance` format.

use crate::models::{Model, ProviderConfig, ProviderInstance, ProviderInstances};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Configuration for the migration process
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to generate unique IDs for instances
    pub generate_unique_ids: bool,
    /// Prefix for generated instance IDs
    pub instance_id_prefix: String,
    /// Whether to set instances as active if they have valid keys
    pub auto_activate_instances: bool,
    /// Whether to preserve metadata from `ProviderConfig`
    pub preserve_metadata: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            generate_unique_ids: true,
            instance_id_prefix: "migrated".to_string(),
            auto_activate_instances: true,
            preserve_metadata: true,
        }
    }
}

impl MigrationConfig {
    /// Creates a new migration configuration with default settings
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to generate unique IDs for instances
    #[must_use] pub const fn with_unique_ids(mut self, generate: bool) -> Self {
        self.generate_unique_ids = generate;
        self
    }

    /// Sets the instance ID prefix
    #[must_use] pub fn with_instance_prefix(mut self, prefix: String) -> Self {
        self.instance_id_prefix = prefix;
        self
    }

    /// Sets whether to auto-activate instances with valid keys
    #[must_use] pub const fn with_auto_activation(mut self, auto_activate: bool) -> Self {
        self.auto_activate_instances = auto_activate;
        self
    }

    /// Sets whether to preserve metadata from `ProviderConfig`
    #[must_use] pub const fn with_metadata_preservation(mut self, preserve: bool) -> Self {
        self.preserve_metadata = preserve;
        self
    }
}

/// Result of a migration operation
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Number of `ProviderConfig` instances migrated
    pub configs_migrated: usize,
    /// Number of `ProviderInstance` instances created
    pub instances_created: usize,
    /// Number of keys migrated
    pub keys_migrated: usize,
    /// Number of models migrated
    pub models_migrated: usize,
    /// Any warnings or issues encountered
    pub warnings: Vec<String>,
    /// Timestamp when migration completed
    pub completed_at: DateTime<Utc>,
}

impl Default for MigrationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationResult {
    /// Creates a new migration result
    #[must_use] pub fn new() -> Self {
        Self {
            configs_migrated: 0,
            instances_created: 0,
            keys_migrated: 0,
            models_migrated: 0,
            warnings: Vec::new(),
            completed_at: Utc::now(),
        }
    }

    /// Adds a warning to the migration result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Utility for migrating from `ProviderConfig` to `ProviderInstance` format
pub struct ProviderConfigMigrator;

impl ProviderConfigMigrator {
    /// Migrates a single `ProviderConfig` to a `ProviderInstance`
    pub fn migrate_config(
        config: ProviderConfig,
        provider_type: &str,
        base_url: &str,
        config_index: usize,
        migration_config: &MigrationConfig,
    ) -> Result<ProviderInstance, String> {
        // Generate unique ID for the instance
        let instance_id = if migration_config.generate_unique_ids {
            format!(
                "{}-{}-{}",
                migration_config.instance_id_prefix, provider_type, config_index
            )
        } else {
            format!("{provider_type}-{config_index}")
        };

        // Create display name
        let display_name = format!("{} Instance {}", provider_type, config_index + 1);

        // Create the provider instance with cleaned metadata
        let mut instance = ProviderInstance::new_with_cleaned_metadata(
            instance_id,
            display_name,
            provider_type.to_string(),
            base_url.to_string(),
            None, // We'll set metadata separately after cleaning
        );

        // Migrate keys from ProviderConfig
        instance.keys = config.keys;

        // Convert model strings to Model objects
        for model_id in config.models {
            let model = Model::new(model_id.clone(), model_id);
            instance.add_model(model);
        }

        // Migrate metadata if configured, but filter out redundant fields
        if migration_config.preserve_metadata {
            if let Some(config_metadata) = config.metadata {
                let mut instance_metadata = HashMap::new();
                for (key, value) in config_metadata {
                    // Skip redundant fields that should not be in metadata
                    if key != "model_id" && key != "base_url" {
                        instance_metadata
                            .insert(key, serde_yaml::to_string(&value).unwrap_or_default());
                    }
                }
                if !instance_metadata.is_empty() {
                    instance.metadata = Some(instance_metadata);
                }
            }
        }

        // Set active status based on configuration and key validity
        if migration_config.auto_activate_instances {
            instance.active = instance.has_valid_keys();
        } else {
            instance.active = false;
        }

        Ok(instance)
    }

    /// Migrates multiple `ProviderConfig` instances to `ProviderInstances`
    pub fn migrate_configs(
        configs: Vec<ProviderConfig>,
        provider_type: &str,
        base_url: &str,
        migration_config: &MigrationConfig,
    ) -> Result<(ProviderInstances, MigrationResult), String> {
        let mut result = MigrationResult::new();
        let mut instances = ProviderInstances::new();

        result.configs_migrated = configs.len();

        for (index, config) in configs.into_iter().enumerate() {
            match Self::migrate_config(config, provider_type, base_url, index, migration_config) {
                Ok(instance) => {
                    // Count keys and models before adding
                    result.keys_migrated += instance.key_count();
                    result.models_migrated += instance.model_count();

                    if let Err(e) = instances.add_instance(instance) {
                        result.add_warning(format!("Failed to add instance {index}: {e}"));
                    } else {
                        result.instances_created += 1;
                    }
                }
                Err(e) => {
                    result.add_warning(format!("Failed to migrate config {index}: {e}"));
                }
            }
        }

        result.completed_at = Utc::now();
        Ok((instances, result))
    }

    /// Detects if a configuration contains legacy `ProviderConfig` format
    #[must_use] pub fn is_legacy_format(config_content: &str) -> bool {
        // Check for old format indicators
        config_content.contains("api_key:") && !config_content.contains("provider_instances:")
    }

    /// Detects if a configuration contains the new `ProviderInstance` format
    #[must_use] pub fn is_new_format(config_content: &str) -> bool {
        config_content.contains("provider_instances:")
            || config_content.contains("ProviderInstance")
    }

    /// Attempts to extract provider type from configuration content
    #[must_use] pub fn detect_provider_type(config_content: &str) -> Option<String> {
        // Common provider type detection patterns
        let patterns = [
            ("openai|gpt|chatgpt", "openai"),
            ("anthropic|claude", "anthropic"),
            ("groq", "groq"),
            ("ollama", "ollama"),
            ("huggingface|hugging-face", "huggingface"),
            ("litellm", "litellm"),
        ];

        for (pattern, provider_type) in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(config_content) {
                    return Some((*provider_type).to_string());
                }
            }
        }

        None
    }

    /// Attempts to extract base URL from configuration content
    #[must_use] pub fn detect_base_url(config_content: &str) -> Option<String> {
        // Common base URL patterns
        let patterns = [
            r"https?://api\.openai\.com",
            r"https?://api\.anthropic\.com",
            r"https?://api\.groq\.com",
            r"https?://api\.ollama\.ai",
            r"https?://huggingface\.co/api",
            r"https?://api\.litellm\.ai",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(url_match) = re.find(config_content) {
                    return Some(url_match.as_str().to_string());
                }
            }
        }

        // Default base URL if none found
        Some("https://api.example.com".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::provider_key::{Environment, ValidationStatus};
    use crate::models::{discovered_key::Confidence, ProviderKey};

    fn create_test_provider_config() -> ProviderConfig {
        let mut config = ProviderConfig::new("1.0".to_string());

        // Add a test key
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.set_validation_status(ValidationStatus::Valid);
        key.value = Some("sk-test123".to_string());
        config.add_key(key);

        // Add some models
        config.models = vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()];

        config
    }

    #[test]
    fn test_migrate_single_config() {
        let config = create_test_provider_config();
        let migration_config = MigrationConfig::default();

        let result = ProviderConfigMigrator::migrate_config(
            config,
            "openai",
            "https://api.openai.com",
            0,
            &migration_config,
        );

        assert!(result.is_ok());
        let instance = result.unwrap();
        assert_eq!(instance.provider_type, "openai");
        assert_eq!(instance.base_url, "https://api.openai.com");
        assert_eq!(instance.key_count(), 1);
        assert_eq!(instance.model_count(), 2);
        assert!(instance.active); // Should be active due to valid key
    }

    #[test]
    fn test_migrate_multiple_configs() {
        let configs = vec![create_test_provider_config(), create_test_provider_config()];
        let migration_config = MigrationConfig::default();

        let result = ProviderConfigMigrator::migrate_configs(
            configs,
            "openai",
            "https://api.openai.com",
            &migration_config,
        );

        assert!(result.is_ok());
        let (instances, migration_result) = result.unwrap();
        assert_eq!(instances.len(), 2);
        assert_eq!(migration_result.configs_migrated, 2);
        assert_eq!(migration_result.instances_created, 2);
        assert_eq!(migration_result.keys_migrated, 2); // 1 key per config
        assert_eq!(migration_result.models_migrated, 4); // 2 models per config
    }

    #[test]
    fn test_legacy_format_detection() {
        let legacy_content = r#"
        api_key: sk-test123
        models: ["gpt-4", "gpt-3.5-turbo"]
        version: "1.0"
        "#;

        assert!(ProviderConfigMigrator::is_legacy_format(legacy_content));
        assert!(!ProviderConfigMigrator::is_new_format(legacy_content));
    }

    #[test]
    fn test_new_format_detection() {
        let new_content = r#"
        provider_instances:
          openai-1:
            id: "openai-1"
            display_name: "OpenAI Instance 1"
            provider_type: "openai"
            base_url: "https://api.openai.com"
        "#;

        assert!(!ProviderConfigMigrator::is_legacy_format(new_content));
        assert!(ProviderConfigMigrator::is_new_format(new_content));
    }

    #[test]
    fn test_provider_type_detection() {
        let openai_content = "This contains gpt-4 and openai references";
        let anthropic_content = "This contains claude-3 and anthropic references";

        assert_eq!(
            ProviderConfigMigrator::detect_provider_type(openai_content),
            Some("openai".to_string())
        );
        assert_eq!(
            ProviderConfigMigrator::detect_provider_type(anthropic_content),
            Some("anthropic".to_string())
        );
    }
}
