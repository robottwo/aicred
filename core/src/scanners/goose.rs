//! Goose scanner for discovering API keys in Goose configuration files.

use super::{ScanResult, ScannerPlugin};
use crate::error::Result;
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey};
use std::path::{Path, PathBuf};

/// Scanner for Goose application configuration.
pub struct GooseScanner;

impl ScannerPlugin for GooseScanner {
    fn name(&self) -> &str {
        "goose"
    }

    fn app_name(&self) -> &str {
        "Goose"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Platform-specific global configs (highest priority)
        // XDG Base Directory Specification: ~/.config/goose/
        paths.push(home_dir.join(".config").join("goose").join("config.yaml"));
        paths.push(home_dir.join(".config").join("goose").join(".gooseignore"));
        paths.push(home_dir.join(".config").join("goose").join(".gdrive-server-credentials.json"));
        paths.push(home_dir.join(".config").join("goose").join("gcp-oauth.keys.json"));

        // macOS Application Support
        if cfg!(target_os = "macos") {
            paths.push(home_dir.join("Library").join("Application Support").join("Goose").join("config.yaml"));
            paths.push(home_dir.join("Library").join("Application Support").join("Goose").join(".gooseignore"));
            paths.push(home_dir.join("Library").join("Application Support").join("Goose").join(".gdrive-server-credentials.json"));
            paths.push(home_dir.join("Library").join("Application Support").join("Goose").join("gcp-oauth.keys.json"));
        }

        // Windows APPDATA
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_path = PathBuf::from(appdata).join("Block").join("goose");
            paths.push(appdata_path.join("config.yaml"));
            paths.push(appdata_path.join(".gooseignore"));
            paths.push(appdata_path.join(".gdrive-server-credentials.json"));
            paths.push(appdata_path.join("gcp-oauth.keys.json"));
        }

        // Environment files
        paths.push(home_dir.join(".goosebench.env"));
        paths.push(PathBuf::from(".goosebench.env"));

        // Recipe files
        paths.push(PathBuf::from("recipes").join("*.yaml"));
        paths.push(PathBuf::from("goose-recipes").join("*.yaml"));

        paths
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let path_str = path.to_string_lossy();
        
        // Check for Goose-specific config files
        file_name == "config.yaml" && (
            path_str.contains("goose") ||
            path_str.contains(".config/goose") ||
            path_str.contains("Block/goose")
        ) ||
        file_name == ".gooseignore" ||
        file_name == ".goosebench.env" ||
        // Check for benchmark configs
        (file_name.ends_with("config.json") && path_str.contains("goose")) ||
        // Check for recipe files
        (file_name.ends_with(".yaml") && (
            path_str.contains("recipes") ||
            path_str.contains("goose-recipes")
        )) ||
        // Environment files in goose contexts
        ((file_name == ".env" || file_name.ends_with(".env")) && 
        (path_str.contains("goose") || path_str.contains("benchmark")))
    }

    fn supports_provider_scanning(&self) -> bool {
        true
    }

    fn supported_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "google".to_string(),
            "databricks".to_string(),
            "groq".to_string(),
            "ollama".to_string(),
            "bedrock".to_string(),
            "azure-openai".to_string(),
            "xai".to_string(),
            "venice".to_string(),
            "openrouter".to_string(),
            "litellm".to_string(),
            "tetrate".to_string(),
            "sagemaker".to_string(),
            "gcp-vertex-ai".to_string(),
            "huggingface".to_string(),
        ]
    }

    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Global provider configs (highest priority)
        // XDG Base Directory Specification: ~/.config/goose/
        let global_config_dir = home_dir.join(".config").join("goose");
        paths.push(global_config_dir.join("config.yaml"));

        // macOS Application Support
        if cfg!(target_os = "macos") {
            let macos_config_dir = home_dir.join("Library").join("Application Support").join("Goose");
            paths.push(macos_config_dir.join("config.yaml"));
        }

        // Windows APPDATA
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_path = PathBuf::from(appdata).join("Block").join("goose");
            paths.push(appdata_path.join("config.yaml"));
        }

        // Environment files (lowest priority)
        paths.push(home_dir.join(".env"));
        paths.push(home_dir.join(".env.local"));
        paths.push(PathBuf::from(".env"));
        paths.push(PathBuf::from(".env.local"));
        paths.push(PathBuf::from(".goosebench.env"));

        // Provider-specific environment files
        paths.push(PathBuf::from("openai.env"));
        paths.push(PathBuf::from("anthropic.env"));
        paths.push(PathBuf::from("google.env"));
        paths.push(PathBuf::from("databricks.env"));
        paths.push(PathBuf::from("groq.env"));

        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Try to parse as YAML first
        let yaml_value = match super::parse_yaml_config(content) {
            Ok(value) => value,
            Err(_) => {
                // If YAML parsing fails, try to parse as JSON (for benchmark configs)
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename == "config.json" {
                        return self.parse_json_config(path, content);
                    }
                }
                // If JSON parsing fails, try to extract from .env format
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename == ".env" || filename == ".goosebench.env" {
                        return self.parse_env_file(content);
                    }
                }
                return Ok(result);
            }
        };

        // Extract keys from YAML config
        if let Some(keys) = self.extract_keys_from_yaml(&yaml_value, path) {
            result.add_keys(keys);
        }

        // Create config instance if this is a valid Goose config
        if self.is_valid_goose_config(&yaml_value) {
            let instance = self.create_config_instance(path, &yaml_value)?;
            result.add_instance(instance);
        }

        Ok(result)
    }

    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Priority order: global → user → project → alternative
        // 1. Global configs (highest priority)
        let global_config_paths = vec![
            home_dir.join(".config").join("goose").join("config.yaml"),
        ];

        for config_path in global_config_paths {
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        if self.is_valid_goose_config(&yaml_value) {
                            let instance = self.create_config_instance(&config_path, &yaml_value)?;
                            instances.push(instance);
                        }
                    }
                }
            }
        }

        // 2. macOS Application Support
        if cfg!(target_os = "macos") {
            let macos_config_paths = vec![
                home_dir.join("Library").join("Application Support").join("Goose").join("config.yaml"),
            ];

            for config_path in macos_config_paths {
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            if self.is_valid_goose_config(&yaml_value) {
                                let instance = self.create_config_instance(&config_path, &yaml_value)?;
                                instances.push(instance);
                            }
                        }
                    }
                }
            }
        }

        // 3. Windows APPDATA
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_path = PathBuf::from(appdata).join("Block").join("goose");
            let appdata_paths = vec![
                appdata_path.join("config.yaml"),
            ];

            for config_path in appdata_paths {
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            if self.is_valid_goose_config(&yaml_value) {
                                let instance = self.create_config_instance(&config_path, &yaml_value)?;
                                instances.push(instance);
                            }
                        }
                    }
                }
            }
        }

        // 4. Environment files (lowest priority)
        let env_paths = vec![
            home_dir.join(".goosebench.env"),
            PathBuf::from(".goosebench.env"),
        ];

        for env_path in env_paths {
            if env_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&env_path) {
                    if content.contains("GOOSE_") || content.contains("OPENAI_API_KEY") {
                        let instance = ConfigInstance::new(
                            self.generate_instance_id(&env_path),
                            "goose".to_string(),
                            env_path.to_path_buf(),
                        );
                        instances.push(instance);
                    }
                }
            }
        }

        Ok(instances)
    }
}

impl GooseScanner {
    /// Extract keys from YAML configuration.
    pub fn extract_keys_from_yaml(
        &self,
        yaml_value: &serde_yaml::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();
        
        // Look for environment variables in extensions
        if let Some(extensions) = yaml_value.get("extensions").and_then(|v| v.as_mapping()) {
            for (_, extension_config) in extensions {
                if let Some(envs) = extension_config.get("envs").and_then(|v| v.as_mapping()) {
                    for (env_name, env_value) in envs {
                        if let Some(env_name_str) = env_name.as_str() {
                            if env_name_str.contains("token") || 
                               env_name_str.contains("key") || 
                               env_name_str.contains("secret") ||
                               env_name_str.contains("api") {
                                if let Some(value) = env_value.as_str() {
                                    if self.is_valid_key(value) {
                                        let provider = self.infer_provider_from_env_name(env_name_str);
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
                }
            }
        }
        
        // Look for top-level environment variables
        for (key, value) in yaml_value.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
            if let Some(key_str) = key.as_str() {
                if key_str.contains("api_key") || key_str.contains("token") || key_str.contains("secret") {
                    if let Some(value_str) = value.as_str() {
                        if self.is_valid_key(value_str) {
                            let provider = self.infer_provider_from_key_name(key_str);
                            let discovered_key = DiscoveredKey::new(
                                provider,
                                path.display().to_string(),
                                ValueType::ApiKey,
                                self.get_confidence(value_str),
                                value_str.to_string(),
                            );
                            keys.push(discovered_key);
                        }
                    }
                }
            }
        }
        
        // Also check for direct API keys in the YAML (like GITHUB_PERSONAL_ACCESS_TOKEN)
        if let Some(github_token) = yaml_value.get("GITHUB_PERSONAL_ACCESS_TOKEN").and_then(|v| v.as_str()) {
            if self.is_valid_key(github_token) {
                let discovered_key = DiscoveredKey::new(
                    "github".to_string(),
                    path.display().to_string(),
                    ValueType::ApiKey,
                    self.get_confidence(github_token),
                    github_token.to_string(),
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

    /// Parse JSON configuration (for benchmark configs).
    fn parse_json_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        
        let json_value = match serde_json::from_str::<serde_json::Value>(content) {
            Ok(value) => value,
            Err(_) => return Ok(result),
        };

        // Extract keys from JSON config
        if let Some(keys) = self.extract_keys_from_json(&json_value, path) {
            result.add_keys(keys);
        }

        // Create config instance if this is a valid Goose benchmark config
        if self.is_valid_goose_benchmark_config(&json_value) {
            let instance = ConfigInstance::new(
                self.generate_instance_id(path),
                "goose".to_string(),
                path.to_path_buf(),
            );
            result.add_instance(instance);
        }

        Ok(result)
    }

    /// Extract keys from JSON configuration.
    fn extract_keys_from_json(
        &self,
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredKey>> {
        let mut keys = Vec::new();

        // Look for models array in benchmark configs
        if let Some(models) = json_value.get("models").and_then(|v| v.as_array()) {
            for model in models {
                if let Some(provider) = model.get("provider").and_then(|v| v.as_str()) {
                    // Look for provider-specific keys in environment or config
                    if let Some(env_file) = json_value.get("env_file").and_then(|v| v.as_str()) {
                        if env_file.contains("goosebench") {
                            // This indicates a Goose benchmark config
                            let discovered_key = DiscoveredKey::new(
                                provider.to_string(),
                                path.display().to_string(),
                                ValueType::ApiKey,
                                Confidence::Medium,
                                format!("env_file: {}", env_file),
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

    /// Check if this is a valid Goose configuration.
    fn is_valid_goose_config(&self, yaml_value: &serde_yaml::Value) -> bool {
        // Check for Goose-specific configuration keys
        yaml_value.get("GOOSE_PROVIDER").is_some() ||
        yaml_value.get("GOOSE_MODEL").is_some() ||
        yaml_value.get("extensions").is_some() ||
        yaml_value.get("GOOSE_MODE").is_some() ||
        yaml_value.get("GOOSE_RECIPE_GITHUB_REPO").is_some() ||
        yaml_value.get("ALPHA_FEATURES").is_some()
    }

    /// Check if this is a valid Goose benchmark configuration.
    fn is_valid_goose_benchmark_config(&self, json_value: &serde_json::Value) -> bool {
        // Check for Goose benchmark-specific configuration keys
        json_value.get("models").is_some() && json_value.get("evals").is_some() ||
        json_value.get("env_file").is_some() && json_value.get("env_file").unwrap().as_str().unwrap_or("").contains("goosebench")
    }

    /// Create a config instance from Goose configuration.
    fn create_config_instance(
        &self,
        path: &Path,
        yaml_value: &serde_yaml::Value,
    ) -> Result<ConfigInstance> {
        let mut metadata = std::collections::HashMap::new();

        // Extract provider if available
        if let Some(provider) = yaml_value.get("GOOSE_PROVIDER").and_then(|v| v.as_str()) {
            metadata.insert("provider".to_string(), provider.to_string());
        }

        // Extract model if available
        if let Some(model) = yaml_value.get("GOOSE_MODEL").and_then(|v| v.as_str()) {
            metadata.insert("model".to_string(), model.to_string());
        }

        // Extract mode if available
        if let Some(mode) = yaml_value.get("GOOSE_MODE").and_then(|v| v.as_str()) {
            metadata.insert("mode".to_string(), mode.to_string());
        }

        let mut instance = ConfigInstance::new(
            self.generate_instance_id(path),
            "goose".to_string(),
            path.to_path_buf(),
        );
        instance.metadata.extend(metadata);
        Ok(instance)
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(&self, path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("goose_{:x}", hasher.finalize())
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
        if key.starts_with("sk-") || key.starts_with("sk-ant-") || key.starts_with("hf_") || 
           key.starts_with("gsk_") || key.starts_with("dapi_") || key.starts_with("rlc-") {
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
        } else if env_name_lower.contains("databricks") {
            "databricks".to_string()
        } else if env_name_lower.contains("groq") {
            "groq".to_string()
        } else if env_name_lower.contains("ollama") {
            "ollama".to_string()
        } else if env_name_lower.contains("bedrock") {
            "bedrock".to_string()
        } else if env_name_lower.contains("azure") {
            "azure-openai".to_string()
        } else if env_name_lower.contains("xai") {
            "xai".to_string()
        } else if env_name_lower.contains("venice") {
            "venice".to_string()
        } else if env_name_lower.contains("openrouter") {
            "openrouter".to_string()
        } else if env_name_lower.contains("litellm") {
            "litellm".to_string()
        } else if env_name_lower.contains("tetrate") {
            "tetrate".to_string()
        } else if env_name_lower.contains("sagemaker") {
            "sagemaker".to_string()
        } else if env_name_lower.contains("vertex") {
            "gcp-vertex-ai".to_string()
        } else if env_name_lower.contains("huggingface") || env_name_lower.contains("hf_") {
            "huggingface".to_string()
        } else if env_name_lower.contains("github") {
            "github".to_string()
        } else if env_name_lower.contains("cloudflare") {
            "cloudflare".to_string()
        } else if env_name_lower.contains("brave") {
            "brave".to_string()
        } else if env_name_lower.contains("asana") {
            "asana".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Infer provider from key name.
    fn infer_provider_from_key_name(&self, key_name: &str) -> String {
        let key_name_lower = key_name.to_lowercase();
        if key_name_lower.contains("openai") {
            "openai".to_string()
        } else if key_name_lower.contains("anthropic") {
            "anthropic".to_string()
        } else if key_name_lower.contains("google") || key_name_lower.contains("gemini") {
            "google".to_string()
        } else if key_name_lower.contains("databricks") {
            "databricks".to_string()
        } else if key_name_lower.contains("groq") {
            "groq".to_string()
        } else if key_name_lower.contains("ollama") {
            "ollama".to_string()
        } else if key_name_lower.contains("bedrock") {
            "bedrock".to_string()
        } else if key_name_lower.contains("azure") {
            "azure-openai".to_string()
        } else if key_name_lower.contains("xai") {
            "xai".to_string()
        } else if key_name_lower.contains("venice") {
            "venice".to_string()
        } else if key_name_lower.contains("openrouter") {
            "openrouter".to_string()
        } else if key_name_lower.contains("litellm") {
            "litellm".to_string()
        } else if key_name_lower.contains("tetrate") {
            "tetrate".to_string()
        } else if key_name_lower.contains("sagemaker") {
            "sagemaker".to_string()
        } else if key_name_lower.contains("vertex") {
            "gcp-vertex-ai".to_string()
        } else if key_name_lower.contains("huggingface") || key_name_lower.contains("hf_") {
            "huggingface".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Parse .env file format.
    fn parse_env_file(&self, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        let env_patterns = [
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("GOOGLE_API_KEY", "google"),
            ("GEMINI_API_KEY", "google"),
            ("DATABRICKS_TOKEN", "databricks"),
            ("GROQ_API_KEY", "groq"),
            ("HUGGING_FACE_HUB_TOKEN", "huggingface"),
            ("GITHUB_PERSONAL_ACCESS_TOKEN", "github"),
            ("CLOUDFLARE_API_TOKEN", "cloudflare"),
            ("BRAVE_API_KEY", "brave"),
            ("ASANA_ACCESS_TOKEN", "asana"),
        ];

        let keys = super::extract_env_keys(content, &env_patterns);
        result.add_keys(keys);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goose_scanner_name() {
        let scanner = GooseScanner;
        assert_eq!(scanner.name(), "goose");
        assert_eq!(scanner.app_name(), "Goose");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = GooseScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);

        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".config/goose/config.yaml")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".goosebench.env")));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = GooseScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().join(".config").join("goose").join("config.yaml");

        assert!(scanner.can_handle_file(&home_path));
        assert!(scanner.can_handle_file(Path::new("/config/goose/config.yaml")));
        assert!(!scanner.can_handle_file(Path::new("/random/config.yaml")));
        assert!(scanner.can_handle_file(Path::new("/recipes/test.yaml")));
    }

    #[test]
    fn test_parse_valid_config() {
        let scanner = GooseScanner;
        let config = r#"
GOOSE_PROVIDER: "anthropic"
GOOSE_MODEL: "claude-3.5-sonnet"
GITHUB_PERSONAL_ACCESS_TOKEN: "ghp_test1234567890abcdef"
extensions:
  github:
    name: GitHub
    envs:
      GITHUB_PERSONAL_ACCESS_TOKEN: "ghp_test1234567890abcdef"
  google-drive:
    name: Google Drive
    envs:
      GDRIVE_CREDENTIALS_PATH: "~/.config/credentials.json"
"#;

        let result = scanner
            .parse_config(Path::new("config.yaml"), config)
            .unwrap();
        assert_eq!(result.keys.len(), 1); // One from top-level GITHUB_PERSONAL_ACCESS_TOKEN
        assert_eq!(result.instances.len(), 1);

        // Check keys
        assert!(result.keys.iter().any(|k| k.provider == "github"));
    }

    #[test]
    fn test_parse_env_file() {
        let scanner = GooseScanner;
        let env_content = r#"
OPENAI_API_KEY=sk-test1234567890abcdef
ANTHROPIC_API_KEY=sk-ant-test1234567890abcdef
GITHUB_PERSONAL_ACCESS_TOKEN=ghp_test1234567890abcdef
"#;

        let result = scanner.parse_env_file(env_content).unwrap();
        assert_eq!(result.keys.len(), 3);
    }

    #[test]
    fn test_is_valid_goose_config() {
        let scanner = GooseScanner;

        let valid_config = serde_yaml::from_str::<serde_yaml::Value>(r#"
GOOSE_PROVIDER: "anthropic"
GOOSE_MODEL: "claude-3.5-sonnet"
"#).unwrap();
        assert!(scanner.is_valid_goose_config(&valid_config));

        let invalid_config = serde_yaml::from_str::<serde_yaml::Value>(r#"
random_key: "value"
"#).unwrap();
        assert!(!scanner.is_valid_goose_config(&invalid_config));
    }

    #[test]
    fn test_create_config_instance() {
        let scanner = GooseScanner;
        let config = serde_yaml::from_str::<serde_yaml::Value>(r#"
GOOSE_PROVIDER: "anthropic"
GOOSE_MODEL: "claude-3.5-sonnet"
GOOSE_MODE: "smart_approve"
"#).unwrap();

        let instance = scanner
            .create_config_instance(Path::new("/test/config.yaml"), &config)
            .unwrap();
        assert_eq!(instance.app_name, "goose");
        assert_eq!(instance.metadata.get("provider"), Some(&"anthropic".to_string()));
        assert_eq!(instance.metadata.get("model"), Some(&"claude-3.5-sonnet".to_string()));
        assert_eq!(instance.metadata.get("mode"), Some(&"smart_approve".to_string()));
    }

    #[test]
    fn test_key_validation() {
        let scanner = GooseScanner;

        // Valid keys
        assert!(scanner.is_valid_key("sk-test1234567890abcdef"));
        assert!(scanner.is_valid_key("ghp_test1234567890abcdef"));
        assert!(scanner.is_valid_key("sk-ant-test1234567890abcdef"));

        // Invalid keys (too short)
        assert!(!scanner.is_valid_key("short-key"));
        assert!(!scanner.is_valid_key("12345678901234")); // 14 chars
    }

    #[test]
    fn test_confidence_scoring() {
        let scanner = GooseScanner;

        // High confidence keys
        assert_eq!(scanner.get_confidence("sk-test1234567890abcdef"), Confidence::High);
        assert_eq!(scanner.get_confidence("sk-ant-test1234567890abcdef"), Confidence::High);
        assert_eq!(scanner.get_confidence("hf_test1234567890abcdef"), Confidence::High);
        assert_eq!(scanner.get_confidence("gsk_test1234567890abcdef"), Confidence::High);

        // Medium confidence keys
        assert_eq!(scanner.get_confidence("test1234567890abcdef1234567890abcdef"), Confidence::Medium);

        // Low confidence keys
        assert_eq!(scanner.get_confidence("test1234567890abcdef"), Confidence::Low);
    }
}