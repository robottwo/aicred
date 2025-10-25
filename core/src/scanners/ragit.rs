//! Ragit scanner for discovering API keys in Ragit configuration files.

use super::{ScanResult, ScannerPlugin};
use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for Ragit application configuration.
pub struct RagitScanner;

impl ScannerPlugin for RagitScanner {
    fn name(&self) -> &str {
        "ragit"
    }

    fn app_name(&self) -> &str {
        "Ragit"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global config
        paths.push(home_dir.join(".ragit").join("config.json"));

        // Project configs
        paths.push(PathBuf::from(".ragit").join("config.json"));
        paths.push(PathBuf::from("ragit_config.json"));

        paths
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name == "config.json"
            && (path.to_string_lossy().contains("ragit")
                || path
                    .parent()
                    .map(|p| p.ends_with(".ragit"))
                    .unwrap_or(false))
    }

    fn supports_provider_scanning(&self) -> bool {
        true
    }

    fn supported_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "huggingface".to_string(),
            "google".to_string(),
            "ragit".to_string(),
        ]
    }

    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Ragit-specific provider configs
        paths.push(home_dir.join(".ragit").join("providers.json"));
        paths.push(home_dir.join(".ragit").join("llm_config.json"));

        // Environment files
        paths.push(home_dir.join(".env"));
        paths.push(home_dir.join(".env.local"));
        paths.push(PathBuf::from(".env"));

        // Provider-specific environment files
        paths.push(PathBuf::from("ragit.env"));
        paths.push(PathBuf::from("openai.env"));
        paths.push(PathBuf::from("anthropic.env"));
        paths.push(PathBuf::from("huggingface.env"));

        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Try to parse as JSON first
        let json_value = match serde_json::from_str::<serde_json::Value>(content) {
            Ok(value) => value,
            Err(_) => {
                // If JSON parsing fails, try to extract from .env format
                if path.file_name().unwrap_or_default() == ".env" {
                    return self.parse_env_file(content);
                }
                return Ok(result);
            }
        };

        // Extract keys from JSON config
        if let Some(keys) = self.extract_keys_from_json(&json_value, path) {
            result.add_keys(keys);
        }

        // Create config instance if this is a valid Ragit config
        if self.is_valid_ragit_config(&json_value) {
            let instance = self.create_config_instance(path, &json_value)?;
            result.add_instance(instance);
        }

        Ok(result)
    }

    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Look for global config
        let global_path = home_dir.join(".ragit").join("config.json");
        if global_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&global_path) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if self.is_valid_ragit_config(&json_value) {
                        let instance = self.create_config_instance(&global_path, &json_value)?;
                        instances.push(instance);
                    }
                }
            }
        }

        // Look for project configs in current directory and subdirectories
        self.scan_project_configs(Path::new("."), &mut instances)?;

        Ok(instances)
    }
}

impl RagitScanner {
    /// Extract keys from JSON configuration.
    fn extract_keys_from_json(
        &self,
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Look for API keys in common locations
        if let Some(api_key) = json_value.get("api_key").and_then(|v| v.as_str()) {
            if self.is_valid_key(api_key) {
                let discovered_key = DiscoveredKey::new(
                    "ragit".to_string(),
                    path.display().to_string(),
                    ValueType::ApiKey,
                    self.get_confidence(api_key),
                    api_key.to_string(),
                );
                keys.push(discovered_key);
            }
        }

        // Look for provider-specific keys
        if let Some(providers) = json_value.get("providers").and_then(|v| v.as_object()) {
            for (provider_name, provider_config) in providers {
                if let Some(key) = provider_config.get("api_key").and_then(|v| v.as_str()) {
                    if self.is_valid_key(key) {
                        let discovered_key = DiscoveredKey::new(
                            provider_name.clone(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            self.get_confidence(key),
                            key.to_string(),
                        );
                        keys.push(discovered_key);
                    }
                }
            }
        }

        // Look for environment variables
        if let Some(env_vars) = json_value.get("env").and_then(|v| v.as_object()) {
            for (env_name, env_value) in env_vars {
                if env_name.contains("key") || env_name.contains("token") {
                    if let Some(value) = env_value.as_str() {
                        if self.is_valid_key(value) {
                            let provider = self.infer_provider_from_env_name(env_name);
                            let discovered_key = DiscoveredKey::new(
                                provider,
                                path.display().to_string(),
                                ValueType::ApiKey,
                                self.get_confidence(value),
                                value.to_string(),
                            );
                            keys.push(discovered_key);
                        }
                    }
                }
            }
        }

        if keys.is_empty() {
            None
        } else {
            Some(keys)
        }
    }

    /// Check if this is a valid Ragit configuration.
    fn is_valid_ragit_config(&self, json_value: &serde_json::Value) -> bool {
        // Check for Ragit-specific configuration keys
        json_value.get("ragit_version").is_some()
            || json_value.get("ragit").is_some()
            || json_value.get("vector_store").is_some()
            || json_value.get("chunking").is_some()
    }

    /// Create a config instance from Ragit configuration.
    fn create_config_instance(
        &self,
        path: &Path,
        json_value: &serde_json::Value,
    ) -> Result<ConfigInstance> {
        let mut metadata = HashMap::new();

        // Extract version if available
        if let Some(version) = json_value.get("ragit_version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }

        // Extract other metadata
        if let Some(default_model) = json_value.get("default_model").and_then(|v| v.as_str()) {
            metadata.insert("default_model".to_string(), default_model.to_string());
        }

        let instance = ConfigInstance {
            instance_id: self.generate_instance_id(path),
            app_name: "ragit".to_string(),
            config_path: path.to_path_buf(),
            discovered_at: Utc::now(),
            keys: Vec::new(), // Will be populated separately
            metadata,
        };

        Ok(instance)
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(&self, path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("ragit_{:x}", hasher.finalize())
            .chars()
            .take(16)
            .collect()
    }

    /// Check if a key is valid.
    fn is_valid_key(&self, key: &str) -> bool {
        key.len() >= 15 && key.chars().any(|c| c.is_alphanumeric())
    }

    /// Get confidence score for a key.
    fn get_confidence(&self, key: &str) -> Confidence {
        if key.starts_with("sk-") || key.starts_with("hf_") || key.starts_with("sk-ant-") {
            Confidence::High
        } else if key.len() >= 30 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }

    /// Infer provider from environment variable name.
    fn infer_provider_from_env_name(&self, env_name: &str) -> String {
        let env_name_lower = env_name.to_lowercase();
        if env_name_lower.contains("openai") {
            "openai".to_string()
        } else if env_name_lower.contains("anthropic") {
            "anthropic".to_string()
        } else if env_name_lower.contains("google") || env_name_lower.contains("gemini") {
            "google".to_string()
        } else if env_name_lower.contains("huggingface") || env_name_lower.contains("hf_") {
            "huggingface".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Parse .env file format.
    fn parse_env_file(&self, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        let env_patterns = [
            ("RAGIT_API_KEY", "ragit"),
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("HUGGING_FACE_HUB_TOKEN", "huggingface"),
        ];

        let keys = super::extract_env_keys(content, &env_patterns);
        result.add_keys(keys);
        Ok(result)
    }

    /// Scan for project configurations.
    fn scan_project_configs(&self, dir: &Path, instances: &mut Vec<ConfigInstance>) -> Result<()> {
        // Look for .ragit/config.json in current directory
        let project_config = dir.join(".ragit").join("config.json");
        if project_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&project_config) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if self.is_valid_ragit_config(&json_value) {
                        let instance = self.create_config_instance(&project_config, &json_value)?;
                        instances.push(instance);
                    }
                }
            }
        }

        // Look for ragit_config.json
        let alt_config = dir.join("ragit_config.json");
        if alt_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&alt_config) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if self.is_valid_ragit_config(&json_value) {
                        let instance = self.create_config_instance(&alt_config, &json_value)?;
                        instances.push(instance);
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ragit_scanner_name() {
        let scanner = RagitScanner;
        assert_eq!(scanner.name(), "ragit");
        assert_eq!(scanner.app_name(), "Ragit");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = RagitScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);

        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".ragit/config.json")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().ends_with(".ragit/config.json")));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = RagitScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().join(".ragit").join("config.json");

        assert!(scanner.can_handle_file(&home_path));
        assert!(scanner.can_handle_file(Path::new("/project/.ragit/config.json")));
        assert!(!scanner.can_handle_file(Path::new("/random/config.json")));
        assert!(!scanner.can_handle_file(
            &temp_dir
                .path()
                .join(".config")
                .join("other")
                .join("config.json")
        ));
    }

    #[test]
    fn test_parse_valid_config() {
        let scanner = RagitScanner;
        let config = r#"{
            "ragit_version": "1.0.0",
            "api_key": "sk-test1234567890abcdef",
            "providers": {
                "openai": {
                    "api_key": "sk-openai1234567890abcdef"
                }
            },
            "vector_store": {
                "type": "chroma"
            }
        }"#;

        let result = scanner
            .parse_config(Path::new("test.json"), config)
            .unwrap();
        assert_eq!(result.keys.len(), 2);
        assert_eq!(result.instances.len(), 1);

        // Check first key
        assert_eq!(result.keys[0].provider, "ragit");
        assert_eq!(result.keys[0].value_type, ValueType::ApiKey);

        // Check second key (OpenAI provider)
        assert_eq!(result.keys[1].provider, "openai");
    }

    #[test]
    fn test_parse_env_file() {
        let scanner = RagitScanner;
        let env_content = r#"
RAGIT_API_KEY=sk-test1234567890abcdef
OPENAI_API_KEY=sk-openai1234567890abcdef
"#;

        let result = scanner.parse_env_file(env_content).unwrap();
        assert_eq!(result.keys.len(), 2);
    }

    #[test]
    fn test_is_valid_ragit_config() {
        let scanner = RagitScanner;

        let valid_config = serde_json::json!({
            "ragit_version": "1.0.0",
            "vector_store": {"type": "chroma"}
        });
        assert!(scanner.is_valid_ragit_config(&valid_config));

        let invalid_config = serde_json::json!({
            "random_key": "value"
        });
        assert!(!scanner.is_valid_ragit_config(&invalid_config));
    }

    #[test]
    fn test_create_config_instance() {
        let scanner = RagitScanner;
        let config = serde_json::json!({
            "ragit_version": "1.2.0",
            "default_model": "gpt-4"
        });

        let instance = scanner
            .create_config_instance(Path::new("/test/config.json"), &config)
            .unwrap();
        assert_eq!(instance.app_name, "ragit");
        assert_eq!(instance.metadata.get("version"), Some(&"1.2.0".to_string()));
        assert_eq!(
            instance.metadata.get("default_model"),
            Some(&"gpt-4".to_string())
        );
    }
}
