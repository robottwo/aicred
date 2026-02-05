//! `LangChain` scanner for discovering API keys in `LangChain` configuration files.

use super::{EnvVarDeclaration, LabelMapping, ScanResult, ScannerPlugin, ScannerPluginExt};
use crate::error::{Error, Result};
use crate::models::credentials::{Confidence, ValueType};
use crate::models::ConfigInstance;
use crate::models::credentials::DiscoveredCredential;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for `LangChain` application configuration.
pub struct LangChainScanner;

impl ScannerPlugin for LangChainScanner {
    fn name(&self) -> &'static str {
        "langchain"
    }

    fn app_name(&self) -> &'static str {
        "LangChain"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            // Global config
            home_dir.join(".langchain").join("config.yaml"),
            home_dir.join(".langchain").join("config.json"),
            home_dir.join(".langchain").join("settings.json"),
            // Project configs
            home_dir.join("langchain_config.yaml"),
            home_dir.join("langchain_config.json"),
            home_dir.join(".langchain.yaml"),
            home_dir.join(".langchain.json"),
            // Environment files
            home_dir.join(".env"),
            home_dir.join("langchain.env"),
        ]
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
                if Self::is_valid_langchain_config_yaml(&yaml_value) {
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
                if Self::is_valid_langchain_config(&json_value) {
                    let instance = Self::create_config_instance(path, &json_value);
                    result.add_instance(instance);
                }
            }
        } else if file_name == ".env" || file_name.ends_with(".env") {
            // Parse environment file
            let mut env_result = Self::parse_env_file(content);

            // Build provider instances from discovered keys
            if !env_result.keys.is_empty() {
                match self.build_instances_from_keys(
                    &env_result.keys,
                    path.to_str().unwrap_or(""),
                    None,
                ) {
                    Ok(provider_instances) => {
                        // Create a config instance for the .env file with the provider instances
                        let mut instance = ConfigInstance::new(
                            Self::generate_instance_id(path),
                            "langchain".to_string(),
                            path.to_path_buf(),
                        );

                        // Add each provider instance to the config instance
                        for provider_instance in provider_instances {
                            if let Err(e) = instance.add_provider_instance(provider_instance) {
                                tracing::warn!("Failed to add provider instance to config: {}", e);
                            }
                        }

                        env_result.add_instance(instance);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to build provider instances from .env keys: {}", e);
                    }
                }
            }

            return Ok(env_result);
        }

        Ok(result)
    }

    fn get_env_var_schema(&self) -> Vec<EnvVarDeclaration> {
        vec![
            EnvVarDeclaration::required(
                "LANGCHAIN_API_KEY".to_string(),
                "API key for LangChain application".to_string(),
                "ApiKey".to_string(),
            ),
            EnvVarDeclaration::optional(
                "LANGCHAIN_BASE_URL".to_string(),
                "Base URL for LangChain API".to_string(),
                "BaseUrl".to_string(),
                Some("https://api.langchain.com/v1".to_string()),
            ),
            EnvVarDeclaration::optional(
                "LANGCHAIN_MODEL_ID".to_string(),
                "Model ID for LangChain".to_string(),
                "ModelId".to_string(),
                Some("langchain-70b".to_string()),
            ),
        ]
    }

    fn get_label_mappings(&self) -> Vec<LabelMapping> {
        vec![]
    }
}

impl LangChainScanner {
    /// Extract keys from JSON configuration.
    fn extract_keys_from_json(
        &self,
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredCredential>> {
        let mut keys = Vec::new();

        // Look for API keys in common locations
        if let Some(api_key) = json_value.get("api_key").and_then(|v| v.as_str()) {
            if Self::is_valid_key(api_key) {
                let discovered_key = DiscoveredCredential::new(
                    "langchain".to_string(),
                    path.display().to_string(),
                    ValueType::ApiKey,
                    Self::get_confidence(api_key),
                    api_key.to_string(),
                );
                keys.push(discovered_key);
            }
        }

        // Look for provider-specific keys
        if let Some(providers) = json_value.get("providers").and_then(|v| v.as_object()) {
            for (provider_name, provider_config) in providers {
                if let Some(key) = provider_config.get("api_key").and_then(|v| v.as_str()) {
                    if Self::is_valid_key(key) {
                        let discovered_key = DiscoveredCredential::new(
                            provider_name.clone(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            Self::get_confidence(key),
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
                let env_name_lc = env_name.to_ascii_lowercase();
                if env_name_lc.contains("key") || env_name_lc.contains("token") {
                    if let Some(value) = env_value.as_str() {
                        if Self::is_valid_key(value) {
                            let provider = Self::infer_provider_from_env_name(env_name);
                            let discovered_key = DiscoveredCredential::new(
                                provider,
                                path.display().to_string(),
                                ValueType::ApiKey,
                                Self::get_confidence(value),
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
                    if Self::is_valid_key(api_key) {
                        let discovered_key = DiscoveredCredential::new(
                            provider.to_string(),
                            path.display().to_string(),
                            ValueType::ApiKey,
                            Self::get_confidence(api_key),
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
    ) -> Option<Vec<DiscoveredCredential>> {
        let keys = Vec::new();

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

    /// Check if this is a valid `LangChain` configuration.
    fn is_valid_langchain_config(json_value: &serde_json::Value) -> bool {
        // Check for LangChain-specific configuration keys
        json_value.get("langchain").is_some()
            || json_value.get("langchain_version").is_some()
            || json_value.get("llm").is_some()
            || json_value.get("chain").is_some()
            || json_value.get("agent").is_some()
            || json_value.get("retriever").is_some()
    }

    /// Check if this is a valid `LangChain` YAML configuration.
    fn is_valid_langchain_config_yaml(yaml_value: &serde_yaml::Value) -> bool {
        // Convert to JSON for consistent checking
        serde_json::to_value(yaml_value)
            .map(|json_value| Self::is_valid_langchain_config(&json_value))
            .unwrap_or(false)
    }

    /// Create a config instance from `LangChain` configuration.
    fn create_config_instance(path: &Path, json_value: &serde_json::Value) -> ConfigInstance {
        let mut metadata = HashMap::new();

        // Extract version if available
        if let Some(version) = json_value.get("langchain_version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }

        // Extract LLM configuration - avoid storing redundant model_id in metadata
        if let Some(llm) = json_value.get("llm").and_then(|v| v.as_object()) {
            if let Some(provider) = llm.get("provider").and_then(|v| v.as_str()) {
                metadata.insert("llm_provider".to_string(), provider.to_string());
            }
            // Don't store model in metadata if it will be handled by the models array
            // Only store it if it's truly metadata, not a primary model configuration
            if let Some(model) = llm.get("model").and_then(|v| v.as_str()) {
                // Only add to metadata if this is additional context, not the primary model
                if !llm.contains_key("models") {
                    metadata.insert("llm_model".to_string(), model.to_string());
                }
            }
        }

        // Extract chain configuration
        if let Some(chain) = json_value.get("chain").and_then(|v| v.as_object()) {
            if let Some(chain_type) = chain.get("type").and_then(|v| v.as_str()) {
                metadata.insert("chain_type".to_string(), chain_type.to_string());
            }
        }

        let mut instance = ConfigInstance::new(
            Self::generate_instance_id(path),
            "langchain".to_string(),
            path.to_path_buf(),
        );
        instance.metadata.extend(metadata);
        instance
    }

    /// Create a config instance from `LangChain` YAML configuration.
    fn create_config_instance_yaml(
        &self,
        path: &Path,
        yaml_value: &serde_yaml::Value,
    ) -> Result<ConfigInstance> {
        // Convert to JSON for consistent processing
        serde_json::to_value(yaml_value).map_or_else(
            |_| {
                Err(Error::ConfigError(
                    "Failed to convert YAML to JSON".to_string(),
                ))
            },
            |json_value| Ok(Self::create_config_instance(path, &json_value)),
        )
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("langchain_{:x}", hasher.finalize())
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
        if key.starts_with("sk-") || key.starts_with("sk-ant-") || key.starts_with("hf_") {
            Confidence::High
        } else if key.len() >= 30 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }

    /// Infer provider from environment variable name.
    fn infer_provider_from_env_name(env_name: &str) -> String {
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
    fn parse_env_file(content: &str) -> ScanResult {
        let mut result = ScanResult::new();

        // API key patterns
        let api_patterns = [
            ("LANGCHAIN_API_KEY", "langchain"),
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("HUGGING_FACE_HUB_TOKEN", "huggingface"),
            ("HUGGINGFACE_API_KEY", "huggingface"),
            ("GROQ_API_KEY", "groq"),
            ("OPENROUTER_API_KEY", "openrouter"),
            ("TEST_API_KEY", "test"),
        ];

        // Metadata patterns - these will be extracted as Custom value types
        // Note: BaseUrl and ModelId are handled by the ProviderInstance structure,
        // so we don't need to extract them as metadata here
        let metadata_patterns = [
            ("GROQ_TEMPERATURE", "groq", "Temperature"),
            ("OPENROUTER_TEMPERATURE", "openrouter", "Temperature"),
            ("ANTHROPIC_TEMPERATURE", "anthropic", "Temperature"),
            ("OPENAI_TEMPERATURE", "openai", "Temperature"),
            ("HUGGINGFACE_TEMPERATURE", "huggingface", "Temperature"),
            ("LANGCHAIN_TEMPERATURE", "langchain", "Temperature"),
            // Model IDs - these should be extracted as ModelId type
            ("GROQ_MODEL", "groq", "ModelId"),
            ("OPENAI_MODEL", "openai", "ModelId"),
            ("ANTHROPIC_MODEL", "anthropic", "ModelId"),
            ("HUGGINGFACE_MODEL", "huggingface", "ModelId"),
            ("OPENROUTER_MODEL", "openrouter", "ModelId"),
            // Base URLs - these should be extracted as BaseUrl type
            ("GROQ_BASE_URL", "groq", "BaseUrl"),
            ("OPENAI_BASE_URL", "openai", "BaseUrl"),
            ("ANTHROPIC_BASE_URL", "anthropic", "BaseUrl"),
            ("HUGGINGFACE_BASE_URL", "huggingface", "BaseUrl"),
            ("OPENROUTER_BASE_URL", "openrouter", "BaseUrl"),
        ];

        let keys =
            super::extract_env_keys_with_metadata(content, &api_patterns, &metadata_patterns);
        result.add_keys(keys);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Using associated functions; no scanner instance needed

        let valid_config = serde_json::json!({
            "langchain_version": "0.1.0",
            "llm": {"provider": "openai"}
        });
        assert!(LangChainScanner::is_valid_langchain_config(&valid_config));

        let invalid_config = serde_json::json!({
            "random_key": "value"
        });
        assert!(!LangChainScanner::is_valid_langchain_config(
            &invalid_config
        ));
    }

    #[test]
    fn test_create_config_instance() {
        // Using associated functions; no scanner instance needed
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

        let instance =
            LangChainScanner::create_config_instance(Path::new("/test/config.json"), &config);
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
