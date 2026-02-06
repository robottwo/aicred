//! `Claude Desktop` scanner for discovering API keys in `Claude Desktop` configuration files.

use super::{EnvVarDeclaration, LabelMapping, ScanResult, ScannerPlugin, ScannerPluginExt};
use crate::error::Result;
use crate::models::credentials::{Confidence, DiscoveredCredential, ValueType};
use crate::models::ConfigInstance;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for `Claude Desktop` application configuration.
pub struct ClaudeDesktopScanner;

impl ScannerPlugin for ClaudeDesktopScanner {
    fn name(&self) -> &'static str {
        "claude-desktop"
    }

    fn app_name(&self) -> &'static str {
        "Claude Desktop"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![home_dir.join(".claude.json")]
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let path_str = path.to_string_lossy();

        file_name.ends_with(".json")
            && (path_str.contains("Claude")
                || path_str.contains("claude")
                || path_str.contains(".claude"))
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        self.parse_config_with_registry(path, content, None)
    }

    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        self.scan_instances_with_registry(home_dir, None)
    }

    fn get_env_var_schema(&self) -> Vec<EnvVarDeclaration> {
        vec![
            EnvVarDeclaration::required(
                "CLAUDE_DESKTOP_API_KEY".to_string(),
                "API key for Claude Desktop".to_string(),
                "ApiKey".to_string(),
            ),
            EnvVarDeclaration::optional(
                "CLAUDE_DESKTOP_BASE_URL".to_string(),
                "Base URL for Claude Desktop API".to_string(),
                "BaseUrl".to_string(),
                Some("https://api.anthropic.com/v1".to_string()),
            ),
            EnvVarDeclaration::optional(
                "CLAUDE_DESKTOP_MODEL_ID".to_string(),
                "Model ID for Claude Desktop".to_string(),
                "ModelId".to_string(),
                Some("claude-3-opus-20240229".to_string()),
            ),
        ]
    }

    fn get_label_mappings(&self) -> Vec<LabelMapping> {
        vec![]
    }
}

impl ClaudeDesktopScanner {
    /// Parse config with optional plugin registry for model auto-detection
    /// Parse Claude Desktop configuration with optional plugin registry for model auto-detection
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be parsed or if the plugin registry is invalid
    pub fn parse_config_with_registry(
        &self,
        path: &Path,
        content: &str,
        plugin_registry: Option<&crate::plugins::ProviderRegistry>,
    ) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Try to parse as JSON first
        let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) else {
            return Ok(result);
        };

        // Extract keys from JSON config
        let discovered_keys =
            Self::extract_keys_from_json(&json_value, path).map_or_else(Vec::new, |keys| {
                result.add_keys(keys.clone());
                keys
            });

        // Build provider instances from discovered keys using the helper function
        tracing::info!(
            "Building provider instances from {} discovered keys in {}",
            discovered_keys.len(),
            path.display()
        );

        let provider_instances = match self.build_instances_from_keys(
            &discovered_keys,
            &path.display().to_string(),
            plugin_registry,
        ) {
            Ok(instances) => {
                tracing::info!(
                    "Successfully built {} provider instances for Claude Desktop config",
                    instances.len()
                );
                instances
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to build provider instances from keys: {}. Creating empty instance.",
                    e
                );
                Vec::new()
            }
        };

        // Create config instance with provider instances
        let mut config_instance = Self::create_config_instance(path, &json_value);

        // Populate provider_instances field
        for provider_instance in provider_instances {
            if let Err(e) = config_instance.add_provider_instance(provider_instance) {
                tracing::warn!("Failed to add provider instance to config: {}", e);
            }
        }

        // Update application instance metadata with models from provider instances
        Self::sync_models_to_metadata(&mut config_instance);

        tracing::debug!(
            "Created config instance with {} provider instances",
            config_instance.provider_instances.len()
        );

        result.add_instances(vec![config_instance]);

        tracing::debug!(
            "Parse config result: {} keys, {} instances, {} provider instances",
            result.keys.len(),
            result.instances.len(),
            result
                .instances
                .first()
                .map_or(0, |i| i.provider_instances.len())
        );

        Ok(result)
    }

    /// Scan instances with optional plugin registry for model auto-detection
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be read or if configuration parsing fails.
    pub fn scan_instances_with_registry(
        &self,
        home_dir: &Path,
        plugin_registry: Option<&crate::plugins::ProviderRegistry>,
    ) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Look only for ~/.claude.json
        let config_path = home_dir.join(".claude.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if Self::is_valid_claude_config(&json_value) {
                        // Extract keys from the config
                        let discovered_keys =
                            Self::extract_keys_from_json(&json_value, &config_path)
                                .unwrap_or_default();

                        // Build provider instances from keys
                        let provider_instances = match self.build_instances_from_keys(
                            &discovered_keys,
                            &config_path.display().to_string(),
                            plugin_registry,
                        ) {
                            Ok(instances) => {
                                tracing::info!(
                                    "Built {} provider instances for Claude Desktop at {}",
                                    instances.len(),
                                    config_path.display()
                                );
                                instances
                            }
                            Err(e) => {
                                tracing::warn!("Failed to build provider instances: {}", e);
                                Vec::new()
                            }
                        };

                        // Create config instance with provider instances
                        let mut config_instance =
                            Self::create_config_instance(&config_path, &json_value);
                        for provider_instance in provider_instances {
                            if let Err(e) = config_instance.add_provider_instance(provider_instance)
                            {
                                tracing::warn!("Failed to add provider instance to config: {}", e);
                            }
                        }

                        // Update application instance metadata with models from provider instances
                        Self::sync_models_to_metadata(&mut config_instance);

                        instances.push(config_instance);
                    }
                }
            }
        }

        Ok(instances)
    }

    /// Extract keys from JSON configuration.
    fn extract_keys_from_json(
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredCredential>> {
        let mut keys = Vec::new();

        // Look for API key stored under "userID" field
        if let Some(user_id) = json_value.get("userID").and_then(|v| v.as_str()) {
            if Self::is_valid_key(user_id) {
                let discovered_key = DiscoveredCredential::new(
                    "anthropic".to_string(),
                    path.display().to_string(),
                    ValueType::ApiKey,
                    Self::get_confidence(user_id),
                    user_id.to_string(),
                );
                keys.push(discovered_key);

                tracing::debug!(
                    "Discovered Anthropic API key in Claude Desktop config at {}",
                    path.display()
                );
            }
        }

        // Extract model information as a discovered key
        if let Some(model) = json_value.get("model").and_then(|v| v.as_str()) {
            let model_key = DiscoveredCredential::new(
                "anthropic".to_string(),
                path.display().to_string(),
                ValueType::ModelId,
                Confidence::High,
                model.to_string(),
            );
            keys.push(model_key);

            tracing::info!("Discovered model '{}' in Claude Desktop config", model);
        }

        // Extract temperature as a discovered key
        if let Some(temperature) = json_value
            .get("temperature")
            .and_then(serde_json::Value::as_f64)
        {
            let temp_key = DiscoveredCredential::new(
                "anthropic".to_string(),
                path.display().to_string(),
                ValueType::Temperature,
                Confidence::High,
                temperature.to_string(),
            );
            keys.push(temp_key);

            tracing::debug!(
                "Discovered temperature {} in Claude Desktop config",
                temperature
            );
        }

        // Extract max_tokens as custom metadata
        if let Some(max_tokens) = json_value
            .get("max_tokens")
            .and_then(serde_json::Value::as_u64)
        {
            let max_tokens_key = DiscoveredCredential::new(
                "anthropic".to_string(),
                path.display().to_string(),
                ValueType::Custom("max_tokens".to_string()),
                Confidence::High,
                max_tokens.to_string(),
            );
            keys.push(max_tokens_key);

            tracing::debug!(
                "Discovered max_tokens {} in Claude Desktop config",
                max_tokens
            );
        }

        if keys.is_empty() {
            None
        } else {
            tracing::info!(
                "Extracted {} configuration values from Claude Desktop config at {}, including {} ModelId keys",
                keys.len(),
                path.display(),
                keys.iter().filter(|k| matches!(k.value_type, ValueType::ModelId)).count()
            );
            Some(keys)
        }
    }

    /// Check if this is a valid Claude Desktop configuration.
    fn is_valid_claude_config(json_value: &serde_json::Value) -> bool {
        // Check for userID field which indicates a valid Claude Desktop config
        json_value.get("userID").is_some()
    }

    /// Create a config instance from Claude configuration.
    fn create_config_instance(path: &Path, json_value: &serde_json::Value) -> ConfigInstance {
        let mut metadata = HashMap::new();

        // Extract version if available
        if let Some(version) = json_value.get("claude_version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }

        // Extract model configuration
        if let Some(model) = json_value.get("model").and_then(|v| v.as_str()) {
            metadata.insert("model".to_string(), model.to_string());
        }

        // Extract max tokens
        if let Some(max_tokens) = json_value
            .get("max_tokens")
            .and_then(serde_json::Value::as_u64)
        {
            metadata.insert("max_tokens".to_string(), max_tokens.to_string());
        }

        // Extract other settings
        if let Some(temperature) = json_value
            .get("temperature")
            .and_then(serde_json::Value::as_f64)
        {
            metadata.insert("temperature".to_string(), temperature.to_string());
        }

        let mut instance = ConfigInstance::new(
            Self::generate_instance_id(path),
            "claude-desktop".to_string(),
            path.to_path_buf(),
        );
        instance.metadata.extend(metadata);
        instance
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("claude_{:x}", hasher.finalize())
            .chars()
            .take(16)
            .collect()
    }

    /// Check if a key is valid.
    fn is_valid_key(key: &str) -> bool {
        // For Anthropic API keys, require the sk-ant- prefix and reasonable length
        key.starts_with("sk-ant-") && key.len() >= 15 && key.chars().any(char::is_alphanumeric)
    }

    /// Get confidence score for a key.
    fn get_confidence(key: &str) -> Confidence {
        if key.starts_with("sk-ant-") {
            Confidence::High
        } else {
            // Don't assign confidence to keys that don't start with sk-ant-
            // This prevents the userID hash from being treated as an API key
            Confidence::Low
        }
    }

    /// Synchronize models from provider instances to application instance metadata.
    /// This ensures that auto-detected models (e.g., from Anthropic API) are reflected
    /// in the application instance's metadata, not just in the provider instances.
    fn sync_models_to_metadata(config_instance: &mut ConfigInstance) {
        // Collect all unique model IDs from all provider instances
        let mut all_models = Vec::new();
        for provider_instance in config_instance.provider_instances.all_instances() {
            for model in &provider_instance.models {
                if !all_models.contains(model) {
                    all_models.push(model.clone());
                }
            }
        }

        // Update metadata with the models
        if !all_models.is_empty() {
            // If there's only one model, store it as "model"
            // If there are multiple, store them as a comma-separated list
            let models_value = if all_models.len() == 1 {
                all_models[0].clone()
            } else {
                all_models.join(", ")
            };

            config_instance
                .metadata
                .insert("models".to_string(), models_value);

            tracing::debug!(
                "Synced {} model(s) to application instance metadata",
                all_models.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_desktop_scanner_name() {
        let scanner = ClaudeDesktopScanner;
        assert_eq!(scanner.name(), "claude-desktop");
        assert_eq!(scanner.app_name(), "Claude Desktop");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = ClaudeDesktopScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);

        // Should only include ~/.claude.json
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains(".claude.json"));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = ClaudeDesktopScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let claude_lib_path = temp_dir
            .path()
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("config.json");
        let claude_profile_path = temp_dir
            .path()
            .join(".claude")
            .join("profiles")
            .join("default.json");

        assert!(scanner.can_handle_file(&claude_lib_path));
        assert!(scanner.can_handle_file(&claude_profile_path));
        assert!(!scanner.can_handle_file(Path::new("/random/config.json")));
    }

    #[test]
    fn test_parse_valid_config() {
        let scanner = ClaudeDesktopScanner;
        let config = r#"{
            "userID": "sk-ant-test1234567890abcdef"
        }"#;

        let result = scanner
            .parse_config(Path::new("test.json"), config)
            .unwrap();
        assert_eq!(result.keys.len(), 1);
        assert_eq!(result.instances.len(), 1);

        // Check key
        assert_eq!(result.keys[0].provider, "anthropic");
        assert_eq!(result.keys[0].value_type, ValueType::ApiKey);
        assert_eq!(result.keys[0].confidence, Confidence::High);

        // Check instance
        assert_eq!(result.instances[0].app_name, "claude-desktop");

        // Check provider instances are populated
        assert_eq!(result.instances[0].provider_instances.len(), 1);
        let provider_instances = result.instances[0].provider_instances.all_instances();
        assert_eq!(provider_instances.len(), 1);
        let provider_instance = provider_instances[0];
        assert_eq!(provider_instance.provider_type, "anthropic");
        assert!(provider_instance.has_non_empty_api_key());
    }

    #[test]
    fn test_is_valid_claude_config() {
        let valid_config = serde_json::json!({
            "userID": "sk-ant-test1234567890abcdef"
        });
        assert!(ClaudeDesktopScanner::is_valid_claude_config(&valid_config));

        let invalid_config = serde_json::json!({
            "random_key": "value"
        });
        assert!(!ClaudeDesktopScanner::is_valid_claude_config(
            &invalid_config
        ));
    }

    #[test]
    fn test_create_config_instance() {
        let config = serde_json::json!({
            "userID": "sk-ant-test1234567890abcdef"
        });

        let instance =
            ClaudeDesktopScanner::create_config_instance(Path::new("/test/config.json"), &config);
        assert_eq!(instance.app_name, "claude-desktop");
    }

    #[test]
    fn test_parse_config_with_metadata() {
        let scanner = ClaudeDesktopScanner;
        let config = r#"{
            "userID": "sk-ant-test1234567890abcdef",
            "model": "claude-3-opus-20240229",
            "temperature": 0.7,
            "max_tokens": 4096
        }"#;

        let result = scanner
            .parse_config(Path::new("test.json"), config)
            .unwrap();

        // Verify all configuration values are discovered (API key + model + temperature + max_tokens)
        assert_eq!(result.keys.len(), 4);

        // Verify we have an API key
        let api_keys: Vec<_> = result
            .keys
            .iter()
            .filter(|k| matches!(k.value_type, ValueType::ApiKey))
            .collect();
        assert_eq!(api_keys.len(), 1);
        assert_eq!(api_keys[0].provider, "anthropic");

        // Verify config instance is created
        assert_eq!(result.instances.len(), 1);
        let config_instance = &result.instances[0];

        // Verify provider instances are populated
        assert_eq!(config_instance.provider_instances.len(), 1);
        let provider_instances = config_instance.provider_instances.all_instances();
        assert_eq!(provider_instances.len(), 1);
        let provider_instance = provider_instances[0];

        // Verify provider instance details
        assert_eq!(provider_instance.provider_type, "anthropic");
        assert!(!provider_instance.id.is_empty()); // ID is now a hash, not the provider name
        assert!(provider_instance.has_non_empty_api_key());

        // Verify model was added to provider instance
        assert_eq!(provider_instance.model_count(), 1);
        assert_eq!(provider_instance.models[0], "claude-3-opus-20240229");

        // Verify temperature and max_tokens are in provider instance metadata
        assert!(!provider_instance.metadata.is_empty());
        let metadata = &provider_instance.metadata;
        assert_eq!(metadata.get("temperature"), Some(&"0.7".to_string()));
        assert_eq!(metadata.get("max_tokens"), Some(&"4096".to_string()));

        // Verify metadata is preserved in config instance
        assert!(config_instance.metadata.contains_key("model"));
        assert_eq!(
            config_instance.metadata.get("model").unwrap(),
            "claude-3-opus-20240229"
        );
        assert!(config_instance.metadata.contains_key("temperature"));
        assert_eq!(config_instance.metadata.get("temperature").unwrap(), "0.7");

        // Verify that models from provider instances are synced to application instance metadata
        assert!(config_instance.metadata.contains_key("models"));
        assert_eq!(
            config_instance.metadata.get("models").unwrap(),
            "claude-3-opus-20240229"
        );
    }

    #[test]
    fn test_parse_config_without_api_key() {
        let scanner = ClaudeDesktopScanner;
        let config = r#"{
            "model": "claude-3-opus-20240229"
        }"#;

        let result = scanner
            .parse_config(Path::new("test.json"), config)
            .unwrap();

        // Should discover model as a key even without API key
        assert_eq!(result.keys.len(), 1);
        assert_eq!(result.keys[0].value_type, ValueType::ModelId);

        // Config instance should still be created
        assert_eq!(result.instances.len(), 1);

        // But no provider instances should be created (no API keys)
        assert_eq!(result.instances[0].provider_instances.len(), 0);
    }
}
