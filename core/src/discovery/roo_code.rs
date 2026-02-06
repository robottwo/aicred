//! Roo Code scanner for discovering API keys in `VSCode` extension configurations.

use super::{EnvVarDeclaration, LabelMapping, ScanResult, ScannerPlugin, ScannerPluginExt};
use crate::error::Result;
use crate::models::credentials::DiscoveredCredential;
use crate::models::credentials::{Confidence, ValueType};
use crate::models::ConfigInstance;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for Roo Code `VSCode` extension configuration.
pub struct RooCodeScanner;

impl ScannerPlugin for RooCodeScanner {
    fn name(&self) -> &'static str {
        "roo-code"
    }

    fn app_name(&self) -> &'static str {
        "Roo Code"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // VSCode globalStorage directories (where Roo Code actually stores configs)
        #[cfg(target_os = "macos")]
        {
            // macOS: ~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline
            if let Some(app_support) = dirs_next::data_dir() {
                let app_support_path = app_support.join("Code").join("User").join("globalStorage");
                paths.push(app_support_path.join("rooveterinaryinc.roo-cline"));
                paths.push(
                    app_support_path
                        .join("rooveterinaryinc.roo-cline")
                        .join("tasks"),
                );
            }

            // Also check for VSCode Insiders
            if let Some(app_support) = dirs_next::data_dir() {
                let app_support_path = app_support
                    .join("Code - Insiders")
                    .join("User")
                    .join("globalStorage");
                paths.push(app_support_path.join("rooveterinaryinc.roo-cline"));
                paths.push(
                    app_support_path
                        .join("rooveterinaryinc.roo-cline")
                        .join("tasks"),
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: ~/.vscode-server/data/User/globalStorage/rooveterinaryinc.roo-cline
            paths.push(
                home_dir
                    .join(".vscode-server")
                    .join("data")
                    .join("User")
                    .join("globalStorage")
                    .join("rooveterinaryinc.roo-cline"),
            );
            paths.push(
                home_dir
                    .join(".vscode-server")
                    .join("data")
                    .join("User")
                    .join("globalStorage")
                    .join("rooveterinaryinc.roo-cline")
                    .join("tasks"),
            );
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(app_data) = std::env::var_os("APPDATA") {
                let app_data_path = PathBuf::from(app_data);
                // Windows: %APPDATA%\Code\User\globalStorage\rooveterinaryinc.roo-cline
                paths.push(
                    app_data_path
                        .join("Code")
                        .join("User")
                        .join("globalStorage")
                        .join("rooveterinaryinc.roo-cline"),
                );
                paths.push(
                    app_data_path
                        .join("Code")
                        .join("User")
                        .join("globalStorage")
                        .join("rooveterinaryinc.roo-cline")
                        .join("tasks"),
                );
                paths.push(
                    app_data_path
                        .join("Code - Insiders")
                        .join("User")
                        .join("globalStorage")
                        .join("rooveterinaryinc.roo-cline"),
                );
                paths.push(
                    app_data_path
                        .join("Code - Insiders")
                        .join("User")
                        .join("globalStorage")
                        .join("rooveterinaryinc.roo-cline")
                        .join("tasks"),
                );
            }
        }

        // VSCode extensions (available on all platforms)
        paths.push(
            home_dir
                .join(".vscode")
                .join("extensions")
                .join("rooveterinaryinc.roo-cline-*"),
        );
        paths.push(
            home_dir
                .join(".vscode-insiders")
                .join("extensions")
                .join("rooveterinaryinc.roo-cline-*"),
        );
        paths.push(
            home_dir
                .join(".vscode-oss")
                .join("extensions")
                .join("rooveterinaryinc.roo-cline-*"),
        );

        // Settings files (may contain Roo Code configuration)
        paths.push(home_dir.join(".vscode").join("settings.json"));
        paths.push(home_dir.join(".vscode-insiders").join("settings.json"));

        // Roo Code specific config files
        paths.push(home_dir.join(".roo-code").join("config.json"));
        paths.push(home_dir.join(".roo_code").join("config.json"));
        paths.push(home_dir.join("roo-code.json"));
        paths.push(home_dir.join("roo_code.json"));

        tracing::debug!(
            "RooCodeScanner scan_paths generated {} paths from home_dir: {}",
            paths.len(),
            home_dir.display()
        );
        for path in &paths {
            tracing::debug!("  - {}", path.display());
        }

        paths
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let path_str = path.to_string_lossy().to_lowercase();

        tracing::debug!(
            "RooCodeScanner checking if can handle file: {}",
            path.display()
        );

        // Handle files in Roo Code globalStorage directory
        if path_str.contains("rooveterinaryinc.roo-cline") {
            tracing::debug!("  - Matched Roo Code globalStorage directory");
            return true;
        }

        // Handle extension manifest files
        if file_name == "package.json"
            && (path_str.contains("roo-cline") || path_str.contains("roo-code"))
        {
            tracing::debug!("  - Matched package.json with roo pattern");
            return true;
        }

        // Handle settings files
        if file_name == "settings.json"
            && (path_str.contains("vscode") || path_str.contains("globalstorage"))
        {
            tracing::debug!("  - Matched settings.json in VSCode directory");
            return true;
        }

        // Handle any JSON file in Roo Code directories
        if file_name.ends_with(".json")
            && (path_str.contains("roo-cline")
                || path_str.contains("roo-code")
                || path_str.contains("rooveterinaryinc"))
        {
            tracing::debug!("  - Matched JSON file with roo pattern");
            return true;
        }

        // Handle Roo Code specific config files
        if file_name == "config.json"
            && (path_str.contains("roo-code") || path_str.contains("roo_code"))
        {
            tracing::debug!("  - Matched Roo Code config.json");
            return true;
        }

        if file_name == "roo-code.json" || file_name == "roo_code.json" {
            tracing::debug!("  - Matched Roo Code JSON file");
            return true;
        }

        tracing::debug!("  - File not handled by RooCodeScanner");
        false
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        self.parse_config_with_registry(path, content, None)
    }

    fn get_env_var_schema(&self) -> Vec<EnvVarDeclaration> {
        vec![
            EnvVarDeclaration::required(
                "ROOCODE_API_KEY".to_string(),
                "API key for Roo Code extension".to_string(),
                "ApiKey".to_string(),
            ),
            EnvVarDeclaration::optional(
                "ROOCODE_BASE_URL".to_string(),
                "Base URL for Roo Code API".to_string(),
                "BaseUrl".to_string(),
                Some("https://api.roocode.com/v1".to_string()),
            ),
            EnvVarDeclaration::optional(
                "ROOCODE_MODEL_ID".to_string(),
                "Model ID for Roo Code".to_string(),
                "ModelId".to_string(),
                Some("roocode-70b".to_string()),
            ),
        ]
    }

    fn get_label_mappings(&self) -> Vec<LabelMapping> {
        vec![]
    }
}

impl RooCodeScanner {
    /// Parse config with optional plugin registry for model auto-detection
    /// Parse Roo Code configuration with optional plugin registry for model auto-detection
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

        tracing::debug!("RooCodeScanner parsing config file: {}", path.display());
        tracing::debug!("Content length: {} bytes", content.len());

        // Try to parse as JSON first
        let json_value = match serde_json::from_str::<serde_json::Value>(content) {
            Ok(value) => {
                tracing::debug!("Successfully parsed as JSON");
                value
            }
            Err(e) => {
                tracing::debug!("JSON parsing failed: {}, trying .env format", e);
                // If JSON parsing fails, try to extract from .env format
                // Safe file_name -> &str conversion and check for .env variants
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();
                if file_name.starts_with(".env") {
                    return Ok(Self::parse_env_file(content));
                }
                return Ok(result);
            }
        };

        // Extract keys from JSON config
        let discovered_keys = if let Some(keys) = self.extract_keys_from_json(&json_value, path) {
            tracing::debug!("Extracted {} keys from JSON", keys.len());
            result.add_keys(keys.clone());
            keys
        } else {
            tracing::debug!("No keys extracted from JSON");
            Vec::new()
        };

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
                    "Successfully built {} provider instances for Roo Code config",
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

    /// Scan for Roo Code instances
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be accessed or if the scan fails
    pub fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Look for VSCode extension directories
        let vscode_extensions = home_dir.join(".vscode").join("extensions");
        if vscode_extensions.exists() {
            self.scan_extension_directory(&vscode_extensions, &mut instances)?;
        }

        // Look for VSCode Insiders extension directories
        let insiders_extensions = home_dir.join(".vscode-insiders").join("extensions");
        if insiders_extensions.exists() {
            self.scan_extension_directory(&insiders_extensions, &mut instances)?;
        }

        // Look for settings files that might contain Roo Code configuration
        self.scan_settings_files(home_dir, &mut instances)?;

        Ok(instances)
    }
}

impl RooCodeScanner {
    /// Extract keys from JSON configuration.
    #[allow(clippy::cognitive_complexity)]
    fn extract_keys_from_json(
        &self,
        json_value: &serde_json::Value,
        path: &Path,
    ) -> Option<Vec<DiscoveredCredential>> {
        let mut keys = Vec::new();

        // Look for API keys in VSCode settings
        if let Some(settings) = json_value.as_object() {
            for (key, value) in settings {
                if key.contains("roo") && key.contains("api") && key.contains("key") {
                    if let Some(api_key) = value.as_str() {
                        if Self::is_valid_key(api_key) {
                            let discovered_key = DiscoveredCredential::new(
                                "roo-code".to_string(),
                                path.display().to_string(),
                                ValueType::ApiKey,
                                Self::get_confidence(api_key),
                                api_key.to_string(),
                            );
                            keys.push(discovered_key);
                        }
                    }
                }

                // Extract model ID
                if key.contains("roo") && key.contains("model") {
                    if let Some(model_id) = value.as_str() {
                        let discovered_key = DiscoveredCredential::new(
                            "roo-code".to_string(),
                            path.display().to_string(),
                            ValueType::ModelId,
                            Confidence::High,
                            model_id.to_string(),
                        );
                        keys.push(discovered_key);
                    }
                }

                // Extract temperature
                if key.contains("roo") && key.contains("temperature") {
                    if let Some(temp) = value.as_f64() {
                        let discovered_key = DiscoveredCredential::new(
                            "roo-code".to_string(),
                            path.display().to_string(),
                            ValueType::Temperature,
                            Confidence::High,
                            temp.to_string(),
                        );
                        keys.push(discovered_key);
                    }
                }

                // Extract max_tokens
                if key.contains("roo") && key.contains("max_tokens") {
                    if let Some(max_tokens) = value.as_i64() {
                        let discovered_key = DiscoveredCredential::new(
                            "roo-code".to_string(),
                            path.display().to_string(),
                            ValueType::Custom("max_tokens".to_string()),
                            Confidence::High,
                            max_tokens.to_string(),
                        );
                        keys.push(discovered_key);
                    }
                }
            }
        }

        // Look for extension configuration
        if let Some(extension_config) = json_value.get("roo-cline") {
            if let Some(keys_obj) = extension_config.get("keys").and_then(|v| v.as_object()) {
                for (provider, key_value) in keys_obj {
                    if let Some(key) = key_value.as_str() {
                        if Self::is_valid_key(key) {
                            let discovered_key = DiscoveredCredential::new(
                                provider.clone(),
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

            // Extract settings from nested configuration
            if let Some(settings_obj) = extension_config.get("settings").and_then(|v| v.as_object())
            {
                for (setting_key, setting_value) in settings_obj {
                    let value_str = match setting_value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => continue,
                    };

                    let discovered_key = DiscoveredCredential::new(
                        "roo-code".to_string(),
                        path.display().to_string(),
                        ValueType::Custom(setting_key.clone()),
                        Confidence::High,
                        value_str,
                    );
                    keys.push(discovered_key);
                }
            }
        }

        // Look for provider-specific keys in extension manifest
        if let Some(contributes) = json_value.get("contributes") {
            if let Some(configuration) = contributes.get("configuration") {
                if let Some(properties) =
                    configuration.get("properties").and_then(|v| v.as_object())
                {
                    for (prop_name, prop_config) in properties {
                        if prop_name.contains("api") && prop_name.contains("key") {
                            if let Some(default_value) =
                                prop_config.get("default").and_then(|v| v.as_str())
                            {
                                if Self::is_valid_key(default_value) {
                                    let provider =
                                        Self::infer_provider_from_property_name(prop_name);
                                    let discovered_key = DiscoveredCredential::new(
                                        provider,
                                        path.display().to_string(),
                                        ValueType::ApiKey,
                                        Self::get_confidence(default_value),
                                        default_value.to_string(),
                                    );
                                    keys.push(discovered_key);
                                }
                            }
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

    /// Scan extension directory for Roo Code installations.
    ///
    /// # Errors
    ///
    /// Returns an error if the extensions directory cannot be read.
    pub fn scan_extension_directory(
        &self,
        extensions_dir: &Path,
        instances: &mut Vec<ConfigInstance>,
    ) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(extensions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();

                    // Check if this is a Roo Code extension
                    if dir_name.contains("roo-cline") {
                        // Look for package.json in the extension
                        let package_json = path.join("package.json");
                        if package_json.exists() {
                            if let Ok(content) = std::fs::read_to_string(&package_json) {
                                if let Ok(json_value) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    let instance =
                                        Self::create_extension_instance(&path, &json_value);
                                    instances.push(instance);
                                }
                            }
                        }

                        // Look for configuration files
                        let config_files = ["settings.json", "config.json", "storage.json"];
                        for config_file in &config_files {
                            let config_path = path.join(config_file);
                            if config_path.exists() {
                                if let Ok(content) = std::fs::read_to_string(&config_path) {
                                    if let Ok(json_value) =
                                        serde_json::from_str::<serde_json::Value>(&content)
                                    {
                                        let instance =
                                            Self::create_config_instance(&config_path, &json_value);
                                        instances.push(instance);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Scan settings files for Roo Code configuration.
    fn scan_settings_files(
        &self,
        home_dir: &Path,
        instances: &mut Vec<ConfigInstance>,
    ) -> Result<()> {
        let settings_paths = [
            home_dir.join(".vscode").join("settings.json"),
            home_dir.join(".vscode-insiders").join("settings.json"),
        ];

        for settings_path in &settings_paths {
            if settings_path.exists() {
                if let Ok(content) = std::fs::read_to_string(settings_path) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Check if this settings file contains Roo Code configuration
                        if Self::has_roo_code_settings(&json_value) {
                            let instance = Self::create_config_instance(settings_path, &json_value);
                            instances.push(instance);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if settings contain Roo Code configuration.
    fn has_roo_code_settings(json_value: &serde_json::Value) -> bool {
        if let Some(settings) = json_value.as_object() {
            for key in settings.keys() {
                if key.contains("roo") && key.contains("cline") {
                    return true;
                }
            }
        }
        false
    }

    /// Create a config instance from extension directory.
    fn create_extension_instance(
        extension_path: &Path,
        package_json: &serde_json::Value,
    ) -> ConfigInstance {
        let mut metadata = HashMap::new();

        // Extract extension information
        if let Some(name) = package_json.get("name").and_then(|v| v.as_str()) {
            metadata.insert("extension_name".to_string(), name.to_string());
        }

        if let Some(version) = package_json.get("version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }

        if let Some(display_name) = package_json.get("displayName").and_then(|v| v.as_str()) {
            metadata.insert("display_name".to_string(), display_name.to_string());
        }

        if let Some(publisher) = package_json.get("publisher").and_then(|v| v.as_str()) {
            metadata.insert("publisher".to_string(), publisher.to_string());
        }

        let mut instance = ConfigInstance::new(
            Self::generate_instance_id(extension_path),
            "roo-code".to_string(),
            extension_path.to_path_buf(),
        );
        instance.metadata.extend(metadata);
        instance
    }

    /// Create a config instance from configuration.
    fn create_config_instance(path: &Path, json_value: &serde_json::Value) -> ConfigInstance {
        let mut metadata = HashMap::new();

        // Extract VSCode settings
        if let Some(settings) = json_value.as_object() {
            for (key, value) in settings {
                if key.contains("roo") && key.contains("cline") {
                    if let Some(value_str) = value.as_str() {
                        metadata.insert(key.clone(), value_str.to_string());
                    } else if let Some(value_bool) = value.as_bool() {
                        metadata.insert(key.clone(), value_bool.to_string());
                    } else if let Some(value_num) = value.as_f64() {
                        metadata.insert(key.clone(), value_num.to_string());
                    }
                }
            }
        }

        let mut instance = ConfigInstance::new(
            Self::generate_instance_id(path),
            "roo-code".to_string(),
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
        format!("roo_{:x}", hasher.finalize())
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

    /// Infer provider from property name.
    fn infer_provider_from_property_name(prop_name: &str) -> String {
        let prop_name_lower = prop_name.to_lowercase();
        if prop_name_lower.contains("openai") {
            "openai".to_string()
        } else if prop_name_lower.contains("anthropic") {
            "anthropic".to_string()
        } else if prop_name_lower.contains("google") || prop_name_lower.contains("gemini") {
            "google".to_string()
        } else if prop_name_lower.contains("huggingface") || prop_name_lower.contains("hf") {
            "huggingface".to_string()
        } else {
            "roo-code".to_string()
        }
    }

    /// Parse .env file format.
    fn parse_env_file(content: &str) -> ScanResult {
        let mut result = ScanResult::new();
        let env_patterns = [
            ("ROO_CODE_API_KEY", "roo-code"),
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
        ];

        let keys = super::extract_env_keys(content, &env_patterns);
        result.add_keys(keys);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roo_code_scanner_name() {
        let scanner = RooCodeScanner;
        assert_eq!(scanner.name(), "roo-code");
        assert_eq!(scanner.app_name(), "Roo Code");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = RooCodeScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);
        let normalized_paths: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"))
            .collect();

        assert!(!normalized_paths.is_empty());
        assert!(normalized_paths
            .iter()
            .any(|p| p.contains(".vscode/extensions")));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = RooCodeScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let vscode_ext_path = temp_dir
            .path()
            .join(".vscode")
            .join("extensions")
            .join("roo-cline-1.0.0")
            .join("package.json");
        let vscode_settings_path = temp_dir.path().join(".vscode").join("settings.json");

        assert!(scanner.can_handle_file(&vscode_ext_path));
        assert!(scanner.can_handle_file(&vscode_settings_path));
        assert!(!scanner.can_handle_file(Path::new("/random/config.json")));
    }

    #[test]
    fn test_parse_valid_config() {
        let scanner = RooCodeScanner;
        let config = r#"{
            "roo-cline.api.key": "sk-test1234567890abcdef",
            "roo-cline.model": "gpt-4",
            "roo-cline.enable": true
        }"#;

        let result = scanner
            .parse_config(Path::new("settings.json"), config)
            .unwrap();
        // Key extraction requires "roo", "api", and "key" in the property name
        assert!(!result.keys.is_empty()); // Should find the API key
        assert!(!result.instances.is_empty()); // Should create instance

        if !result.keys.is_empty() {
            // Check key if found
            assert_eq!(result.keys[0].provider, "roo-code");
            assert_eq!(result.keys[0].value_type, ValueType::ApiKey);
        }

        if !result.instances.is_empty() {
            // Check instance if found
            assert_eq!(result.instances[0].app_name, "roo-code");
        }
    }

    #[test]
    fn test_has_roo_code_settings() {
        let valid_settings = serde_json::json!({
            "roo-cline.apiKey": "test",
            "editor.fontSize": 14
        });
        assert!(RooCodeScanner::has_roo_code_settings(&valid_settings));

        let invalid_settings = serde_json::json!({
            "editor.fontSize": 14,
            "workbench.colorTheme": "dark"
        });
        assert!(!RooCodeScanner::has_roo_code_settings(&invalid_settings));
    }

    #[test]
    fn test_create_config_instance() {
        let config = serde_json::json!({
            "roo-cline.apiKey": "sk-test1234567890abcdef",
            "roo-cline.model": "gpt-4",
            "roo-cline.enable": true,
            "roo-cline.temperature": 0.7
        });

        let instance =
            RooCodeScanner::create_config_instance(Path::new("/test/settings.json"), &config);
        assert_eq!(instance.app_name, "roo-code");
        assert_eq!(
            instance.metadata.get("roo-cline.apiKey"),
            Some(&"sk-test1234567890abcdef".to_string())
        );
        assert_eq!(
            instance.metadata.get("roo-cline.model"),
            Some(&"gpt-4".to_string())
        );
        assert_eq!(
            instance.metadata.get("roo-cline.enable"),
            Some(&"true".to_string())
        );
        assert_eq!(
            instance.metadata.get("roo-cline.temperature"),
            Some(&"0.7".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_json_returns_empty() {
        let scanner = RooCodeScanner;
        let content = "{ this is not valid json";
        let result = scanner
            .parse_config(Path::new("package.json"), content)
            .unwrap();
        assert_eq!(result.keys.len(), 0);
        assert_eq!(result.instances.len(), 0);
    }
}
