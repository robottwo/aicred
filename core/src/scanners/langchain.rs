//! LangChain scanner for discovering API keys in LangChain configuration files.

use super::{ScanResult, ScannerPlugin};
use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for LangChain application configuration.
pub struct LangChainScanner;

impl ScannerPlugin for LangChainScanner {
    fn name(&self) -> &str {
        "langchain"
    }

    fn app_name(&self) -> &str {
        "LangChain"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global config
        paths.push(home_dir.join(".langchain").join("config.yaml"));
        paths.push(home_dir.join(".langchain").join("config.json"));
        paths.push(home_dir.join(".langchain").join("settings.json"));

        // Project configs
        paths.push(PathBuf::from("langchain_config.yaml"));
        paths.push(PathBuf::from("langchain_config.json"));
        paths.push(PathBuf::from(".langchain.yaml"));
        paths.push(PathBuf::from(".langchain.json"));

        // Environment files
        paths.push(PathBuf::from(".env"));
        paths.push(PathBuf::from("langchain.env"));

        paths
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let path_str = path.to_string_lossy();

        file_name.ends_with(".yaml")
            || file_name.ends_with(".yml")
            || file_name.ends_with(".json")
            || file_name == ".env"
            || file_name.ends_with(".env")
            || path_str.contains("langchain")
    }

    fn supports_provider_scanning(&self) -> bool {
        true
    }

    fn supported_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "google".to_string(),
            "huggingface".to_string(),
            "langchain".to_string(),
        ]
    }

    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Common provider configuration files
        paths.push(home_dir.join(".env"));
        paths.push(home_dir.join(".env.local"));
        paths.push(home_dir.join(".envrc"));
        paths.push(PathBuf::from(".env"));
        paths.push(PathBuf::from(".env.local"));

        // Provider-specific environment files
        paths.push(PathBuf::from("openai.env"));
        paths.push(PathBuf::from("anthropic.env"));
        paths.push(PathBuf::from("huggingface.env"));
        paths.push(PathBuf::from("google.env"));

        // Configuration directories
        paths.push(home_dir.join(".config").join("openai"));
        paths.push(home_dir.join(".config").join("anthropic"));
        paths.push(home_dir.join(".config").join("huggingface"));

        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Determine file type and parse accordingly
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
            // Parse YAML
            if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(content) {
                if let Some(keys) = self.extract_keys_from_yaml(&yaml_value, path) {
                    result.add_keys(keys);
                }
                if self.is_valid_langchain_config_yaml(&yaml_value) {
                    let instance = self.create_config_instance_yaml(path, &yaml_value)?;
                    result.add_instance(instance);
                }
            }
        } else if file_name.ends_with(".json") {
            // Parse JSON
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                if let Some(keys) = self.extract_keys_from_json(&json_value, path) {
                    result.add_keys(keys);
                }
                if self.is_valid_langchain_config(&json_value) {
                    let instance = self.create_config_instance(path, &json_value)?;
                    result.add_instance(instance);
                }
            }
        } else if file_name == ".env" || file_name.ends_with(".env") {
            // Parse environment file
            return self.parse_env_file(content);
        }

        Ok(result)
    }
}

impl LangChainScanner {
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
                    "langchain".to_string(),
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

        // Look for LangChain-specific configuration
        if let Some(llm) = json_value.get("llm").and_then(|v| v.as_object()) {
            if let Some(provider) = llm.get("provider").and_then(|v| v.as_str()) {
                if let Some(api_key) = llm.get("api_key").and_then(|v| v.as_str()) {
                    if self.is_valid_key(api_key) {
                        let discovered_key = DiscoveredKey::new(
                            provider.to_string(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            self.get_confidence(api_key),
                            api_key.to_string(),
                        );
                        keys.push(discovered_key);
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

    /// Extract keys from YAML configuration.
    fn extract_keys_from_yaml(
        &self,
        yaml_value: &serde_yaml::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Convert YAML to JSON-like structure for easier processing
        if let Ok(json_value) = serde_json::to_value(yaml_value) {
            return self.extract_keys_from_json(&json_value, path);
        }

        if keys.is_empty() {
            None
        } else {
            Some(keys)
        }
    }

    /// Check if this is a valid LangChain configuration.
    fn is_valid_langchain_config(&self, json_value: &serde_json::Value) -> bool {
        // Check for LangChain-specific configuration keys
        json_value.get("langchain").is_some()
            || json_value.get("langchain_version").is_some()
            || json_value.get("llm").is_some()
            || json_value.get("chain").is_some()
            || json_value.get("agent").is_some()
            || json_value.get("retriever").is_some()
    }

    /// Check if this is a valid LangChain YAML configuration.
    fn is_valid_langchain_config_yaml(&self, yaml_value: &serde_yaml::Value) -> bool {
        // Convert to JSON for consistent checking
        if let Ok(json_value) = serde_json::to_value(yaml_value) {
            self.is_valid_langchain_config(&json_value)
        } else {
            false
        }
    }

    /// Create a config instance from LangChain configuration.
    fn create_config_instance(
        &self,
        path: &Path,
        json_value: &serde_json::Value,
    ) -> Result<ConfigInstance> {
        let mut metadata = HashMap::new();

        // Extract version if available
        if let Some(version) = json_value.get("langchain_version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }

        // Extract LLM configuration
        if let Some(llm) = json_value.get("llm").and_then(|v| v.as_object()) {
            if let Some(provider) = llm.get("provider").and_then(|v| v.as_str()) {
                metadata.insert("llm_provider".to_string(), provider.to_string());
            }
            if let Some(model) = llm.get("model").and_then(|v| v.as_str()) {
                metadata.insert("llm_model".to_string(), model.to_string());
            }
        }

        // Extract chain configuration
        if let Some(chain) = json_value.get("chain").and_then(|v| v.as_object()) {
            if let Some(chain_type) = chain.get("type").and_then(|v| v.as_str()) {
                metadata.insert("chain_type".to_string(), chain_type.to_string());
            }
        }

        let mut instance = ConfigInstance::new(
            self.generate_instance_id(path),
            "langchain".to_string(),
            path.to_path_buf(),
        );
        instance.metadata.extend(metadata);
        Ok(instance)
    }

    /// Create a config instance from LangChain YAML configuration.
    fn create_config_instance_yaml(
        &self,
        path: &Path,
        yaml_value: &serde_yaml::Value,
    ) -> Result<ConfigInstance> {
        // Convert to JSON for consistent processing
        if let Ok(json_value) = serde_json::to_value(yaml_value) {
            self.create_config_instance(path, &json_value)
        } else {
            Err(Error::ConfigError(
                "Failed to convert YAML to JSON".to_string(),
            ))
        }
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(&self, path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("langchain_{:x}", hasher.finalize())
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
        if key.starts_with("sk-") || key.starts_with("sk-ant-") || key.starts_with("hf_") {
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
            ("LANGCHAIN_API_KEY", "langchain"),
            ("LANGCHAIN_API_KEY", "langchain"),
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("HUGGING_FACE_HUB_TOKEN", "huggingface"),
        ];

        let keys = super::extract_env_keys(content, &env_patterns);
        result.add_keys(keys);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_langchain_scanner_name() {
        let scanner = LangChainScanner;
        assert_eq!(scanner.name(), "langchain");
        assert_eq!(scanner.app_name(), "LangChain");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = LangChainScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);

        assert!(!paths.is_empty());
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".langchain")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("langchain_config")));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = LangChainScanner;

        assert!(scanner.can_handle_file(Path::new("langchain_config.yaml")));
        assert!(scanner.can_handle_file(Path::new("langchain_config.json")));
        assert!(scanner.can_handle_file(Path::new(".env")));
        assert!(scanner.can_handle_file(Path::new("config.yaml")));
        assert!(!scanner.can_handle_file(Path::new("random.txt")));
    }

    #[test]
    fn test_parse_valid_json_config() {
        let scanner = LangChainScanner;
        let config = r#"{
            "langchain_version": "0.1.0",
            "api_key": "sk-test1234567890abcdef",
            "llm": {
                "provider": "openai",
                "model": "gpt-4",
                "api_key": "sk-openai1234567890abcdef"
            },
            "chain": {
                "type": "conversational"
            }
        }"#;

        let result = scanner
            .parse_config(Path::new("config.json"), config)
            .unwrap();
        assert_eq!(result.keys.len(), 2);
        assert_eq!(result.instances.len(), 1);

        // Check keys
        assert_eq!(result.keys[0].provider, "langchain");
        assert_eq!(result.keys[1].provider, "openai");

        // Check instance
        assert_eq!(result.instances[0].app_name, "langchain");
        assert_eq!(
            result.instances[0].metadata.get("version"),
            Some(&"0.1.0".to_string())
        );
        assert_eq!(
            result.instances[0].metadata.get("llm_provider"),
            Some(&"openai".to_string())
        );
        assert_eq!(
            result.instances[0].metadata.get("llm_model"),
            Some(&"gpt-4".to_string())
        );
    }

    #[test]
    fn test_parse_valid_yaml_config() {
        let scanner = LangChainScanner;
        let config = r#"
langchain_version: "0.1.0"
api_key: sk-test1234567890abcdef
llm:
  provider: openai
  model: gpt-4
  api_key: sk-openai1234567890abcdef
chain:
  type: conversational
"#;

        let result = scanner
            .parse_config(Path::new("config.yaml"), config)
            .unwrap();
        assert_eq!(result.keys.len(), 2);
        assert_eq!(result.instances.len(), 1);
    }

    #[test]
    fn test_is_valid_langchain_config() {
        let scanner = LangChainScanner;

        let valid_config = serde_json::json!({
            "langchain_version": "0.1.0",
            "llm": {"provider": "openai"}
        });
        assert!(scanner.is_valid_langchain_config(&valid_config));

        let invalid_config = serde_json::json!({
            "random_key": "value"
        });
        assert!(!scanner.is_valid_langchain_config(&invalid_config));
    }

    #[test]
    fn test_create_config_instance() {
        let scanner = LangChainScanner;
        let config = serde_json::json!({
            "langchain_version": "0.2.0",
            "llm": {
                "provider": "openai",
                "model": "gpt-4"
            },
            "chain": {
                "type": "conversational"
            }
        });

        let instance = scanner
            .create_config_instance(Path::new("/test/config.json"), &config)
            .unwrap();
        assert_eq!(instance.app_name, "langchain");
        assert_eq!(instance.metadata.get("version"), Some(&"0.2.0".to_string()));
        assert_eq!(
            instance.metadata.get("llm_provider"),
            Some(&"openai".to_string())
        );
        assert_eq!(
            instance.metadata.get("llm_model"),
            Some(&"gpt-4".to_string())
        );
        assert_eq!(
            instance.metadata.get("chain_type"),
            Some(&"conversational".to_string())
        );
    }
}
