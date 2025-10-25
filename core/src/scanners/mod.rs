//! Scanner plugins for various applications that store API keys.

mod claude_desktop;
mod gsh;
mod langchain;
mod ragit;
mod roo_code;

pub use claude_desktop::ClaudeDesktopScanner;
pub use gsh::GshScanner;
pub use langchain::LangChainScanner;
pub use ragit::RagitScanner;
pub use roo_code::RooCodeScanner;

use crate::error::{Error, Result};
use crate::models::{ConfigInstance, DiscoveredKey};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Trait that all application scanner plugins must implement.
pub trait ScannerPlugin: Send + Sync {
    /// Returns the name of this scanner (e.g., "ragit", "claude-desktop").
    fn name(&self) -> &str;

    /// Returns the application name (e.g., "Ragit", "Claude Desktop").
    fn app_name(&self) -> &str;

    /// Returns the paths that this scanner should scan for configuration files.
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>;

    /// Parses a configuration file and extracts discovered keys and config instances.
    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult>;

    /// Validates that this scanner can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool;

    /// Returns true if this scanner can discover provider-specific configurations.
    /// This enables the scanner to look for provider keys in application configs.
    fn supports_provider_scanning(&self) -> bool {
        // Default implementation - override to enable provider scanning
        false
    }

    /// Returns a list of provider names that this scanner can discover.
    /// Only used when supports_provider_scanning() returns true.
    fn supported_providers(&self) -> Vec<String> {
        // Default implementation - override to specify supported providers
        Vec::new()
    }

    /// Scans for provider-specific configuration files (e.g., .env files, provider configs).
    /// This method enables scanners to discover provider keys in standard locations.
    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        // Default implementation - override to scan for provider-specific configs
        let mut paths = Vec::new();

        // Common provider configuration file patterns
        paths.push(home_dir.join(".env"));
        paths.push(home_dir.join(".env.local"));
        paths.push(home_dir.join(".envrc"));
        paths.push(home_dir.join("config.json"));
        paths.push(home_dir.join("config.yaml"));
        paths.push(home_dir.join("config.yml"));
        paths.push(home_dir.join("settings.json"));

        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }

    /// Scans for multiple instances of this application (e.g., multiple installations).
    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        // Default implementation - override for multi-instance applications
        let mut instances = Vec::new();

        // Scan application-specific paths
        let app_paths = self.scan_paths(home_dir);
        for path in app_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(result) = self.parse_config(&path, &content) {
                        if !result.keys.is_empty() || !result.instances.is_empty() {
                            instances.extend(result.instances);
                        }
                    }
                }
            }
        }

        // If this scanner supports provider scanning, also scan for provider configs
        if self.supports_provider_scanning() {
            if let Ok(provider_paths) = self.scan_provider_configs(home_dir) {
                for path in provider_paths {
                    if path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(result) = self.parse_config(&path, &content) {
                                if !result.keys.is_empty() || !result.instances.is_empty() {
                                    instances.extend(result.instances);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(instances)
    }
}

/// Result from scanning a configuration file.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Discovered API keys.
    pub keys: Vec<DiscoveredKey>,
    /// Configuration instances found.
    pub instances: Vec<ConfigInstance>,
}

impl ScanResult {
    /// Creates a new scan result.
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Adds a discovered key.
    pub fn add_key(&mut self, key: DiscoveredKey) {
        self.keys.push(key);
    }

    /// Adds multiple discovered keys.
    pub fn add_keys(&mut self, keys: Vec<DiscoveredKey>) {
        self.keys.extend(keys);
    }

    /// Adds a configuration instance.
    pub fn add_instance(&mut self, instance: ConfigInstance) {
        self.instances.push(instance);
    }

    /// Adds multiple configuration instances.
    pub fn add_instances(&mut self, instances: Vec<ConfigInstance>) {
        self.instances.extend(instances);
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for managing scanner plugins.
#[derive(Clone)]
pub struct ScannerRegistry {
    scanners: std::sync::Arc<std::sync::RwLock<HashMap<String, std::sync::Arc<dyn ScannerPlugin>>>>,
}

impl std::fmt::Debug for ScannerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScannerRegistry")
            .field(
                "scanners",
                &format!("<{} scanners>", self.scanners.read().unwrap().len()),
            )
            .finish()
    }
}

impl ScannerRegistry {
    /// Creates a new empty scanner registry.
    pub fn new() -> Self {
        Self {
            scanners: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new scanner.
    pub fn register(&self, scanner: std::sync::Arc<dyn ScannerPlugin>) -> Result<()> {
        let mut scanners = self.scanners.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on scanners".to_string())
        })?;

        let name = scanner.name().to_string();
        if scanners.contains_key(&name) {
            return Err(Error::PluginError(format!(
                "Scanner '{}' is already registered",
                name
            )));
        }

        scanners.insert(name, scanner);
        Ok(())
    }

    /// Gets a scanner by name.
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn ScannerPlugin>> {
        self.scanners
            .read()
            .ok()
            .and_then(|scanners| scanners.get(name).cloned())
    }

    /// Lists all registered scanner names.
    pub fn list(&self) -> Vec<String> {
        self.scanners
            .read()
            .ok()
            .map(|scanners| scanners.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets all scanners that can handle a specific file.
    pub fn get_scanners_for_file(&self, path: &Path) -> Vec<std::sync::Arc<dyn ScannerPlugin>> {
        self.scanners
            .read()
            .ok()
            .map(|scanners| {
                scanners
                    .values()
                    .filter(|scanner| scanner.can_handle_file(path))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for ScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to parse JSON config files.
pub fn parse_json_config(content: &str) -> Result<serde_json::Value> {
    serde_json::from_str(content)
        .map_err(|e| Error::ConfigError(format!("Failed to parse JSON: {}", e)))
}

/// Helper function to parse YAML config files.
pub fn parse_yaml_config(content: &str) -> Result<serde_yaml::Value> {
    serde_yaml::from_str(content)
        .map_err(|e| Error::ConfigError(format!("Failed to parse YAML: {}", e)))
}

/// Helper function to extract keys from environment variable format.
pub fn extract_env_keys(content: &str, patterns: &[(&str, &str)]) -> Vec<DiscoveredKey> {
    let mut keys = Vec::new();

    for (env_var, provider) in patterns {
        let pattern = format!(
            r"(?i){}\s*=\s*([a-zA-Z0-9_-]{{15,}})",
            regex::escape(env_var)
        );
        let regex = regex::Regex::new(&pattern).unwrap();

        for cap in regex.captures_iter(content) {
            if let Some(key_match) = cap.get(1) {
                let key_value = key_match.as_str();

                let discovered_key = DiscoveredKey::new(
                    provider.to_string(),
                    "env_file".to_string(),
                    crate::models::discovered_key::ValueType::ApiKey,
                    crate::models::discovered_key::Confidence::High,
                    key_value.to_string(),
                );

                keys.push(discovered_key);
            }
        }
    }

    keys
}

/// Registers all built-in scanner plugins.
pub fn register_builtin_scanners(registry: &ScannerRegistry) -> Result<()> {
    registry.register(std::sync::Arc::new(RagitScanner))?;
    registry.register(std::sync::Arc::new(ClaudeDesktopScanner))?;
    registry.register(std::sync::Arc::new(RooCodeScanner))?;
    registry.register(std::sync::Arc::new(LangChainScanner))?;
    registry.register(std::sync::Arc::new(GshScanner))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_registry() {
        let registry = ScannerRegistry::new();
        assert!(registry.list().is_empty());
    }
}
