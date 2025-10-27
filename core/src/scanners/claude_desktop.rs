//! `Claude Desktop` scanner for discovering API keys in `Claude Desktop` configuration files.

use super::{ScanResult, ScannerPlugin};
use crate::error::Result;
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey};
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
        let mut result = ScanResult::new();

        // Try to parse as JSON first
        let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) else {
            return Ok(result);
        };

        // Extract keys from JSON config
        if let Some(keys) = self.extract_keys_from_json(&json_value, path) {
            result.add_keys(keys);
        }

        // Create config instances for Claude Desktop installations
        let mut instances = Vec::new();
        let instance = Self::create_config_instance(path, &json_value);
        tracing::debug!("Created config instance");
        instances.push(instance);
        result.add_instances(instances);

        tracing::debug!(
            "Parse config result: {} keys, {} instances",
            result.keys.len(),
            result.instances.len()
        );

        Ok(result)
    }

    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Look only for ~/.claude.json
        let config_path = home_dir.join(".claude.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if Self::is_valid_claude_config(&json_value) {
                        let instance = Self::create_config_instance(&config_path, &json_value);
                        instances.push(instance);
                    }
                }
            }
        }

        Ok(instances)
    }
}

impl ClaudeDesktopScanner {
    /// Extract keys from JSON configuration.
    fn extract_keys_from_json(
        &self,
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Look for API key stored under "userID" field
        if let Some(user_id) = json_value.get("userID").and_then(|v| v.as_str()) {
            if Self::is_valid_key(user_id) {
                let discovered_key = DiscoveredKey::new(
                    "anthropic".to_string(),
                    path.display().to_string(),
                    ValueType::ApiKey,
                    Self::get_confidence(user_id),
                    user_id.to_string(),
                );
                keys.push(discovered_key);
            }
        }

        if keys.is_empty() {
            None
        } else {
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
        key.len() >= 15 && key.chars().any(char::is_alphanumeric)
    }

    /// Get confidence score for a key.
    fn get_confidence(key: &str) -> Confidence {
        if key.starts_with("sk-ant-") {
            Confidence::High
        } else if key.len() >= 30 {
            Confidence::Medium
        } else {
            Confidence::Low
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
    }

    #[test]
    fn test_is_valid_claude_config() {
        let scanner = ClaudeDesktopScanner;

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
        let scanner = ClaudeDesktopScanner;
        let config = serde_json::json!({
            "userID": "sk-ant-test1234567890abcdef"
        });

        let instance =
            ClaudeDesktopScanner::create_config_instance(Path::new("/test/config.json"), &config);
        assert_eq!(instance.app_name, "claude-desktop");
    }
}
