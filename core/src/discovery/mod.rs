//! Discovery system for finding AI credentials across applications.
//!
//! This module contains application-specific scanners that know how to find
//! credentials in various tools' configuration files (Claude Desktop, Roo Code, etc.).
//!
//! Previously named `scanners`, renamed to `discovery` in v0.2.0 for clarity.

// Allow clippy lints for the discovery module

/// Default maximum file size to scan (1MB).
pub const DEFAULT_MAX_FILE_SIZE: usize = 1024 * 1024;

/// Scanner configuration.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Maximum file size to scan in bytes.
    pub max_file_size: usize,
    /// Whether to follow symbolic links.
    pub follow_symlinks: bool,
    /// File extensions to include.
    pub include_extensions: Option<Vec<String>>,
    /// File extensions to exclude.
    pub exclude_extensions: Option<Vec<String>>,
    /// Specific files to exclude.
    pub exclude_files: Option<Vec<String>>,
    /// Whether to scan hidden files/directories.
    pub scan_hidden: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            follow_symlinks: false,
            include_extensions: None,
            exclude_extensions: Some(vec![".log".to_string(), ".tmp".to_string()]),
            exclude_files: Some(vec![".DS_Store".to_string(), "Thumbs.db".to_string()]),
            scan_hidden: true,
        }
    }
}

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
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey, Model, ProviderInstance};
use sha2::Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Environment variable declaration for scanner plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarDeclaration {
    /// Name of the environment variable (e.g., `OPENAI_API_KEY`).
    pub name: String,
    /// Description of what this variable does.
    pub description: String,
    /// Type of value expected (e.g., "string", "number", "boolean").
    pub value_type: String,
    /// Whether this environment variable is required.
    pub required: bool,
    /// Default value if not provided (None if no default).
    pub default_value: Option<String>,
}

impl EnvVarDeclaration {
    /// Creates a new environment variable declaration.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const because it takes owned String parameters
    pub fn new(
        name: String,
        description: String,
        value_type: String,
        required: bool,
        default_value: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            value_type,
            required,
            default_value,
        }
    }

    /// Creates a new required environment variable declaration.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const because it takes owned String parameters
    pub fn required(name: String, description: String, value_type: String) -> Self {
        Self::new(name, description, value_type, true, None)
    }

    /// Creates a new optional environment variable declaration.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const because it takes owned String parameters
    pub fn optional(
        name: String,
        description: String,
        value_type: String,
        default_value: Option<String>,
    ) -> Self {
        Self::new(name, description, value_type, false, default_value)
    }
}

/// Label mapping for scanner plugins that associate labels with environment variable groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelMapping {
    /// Name of the label (e.g., "fast", "smart").
    pub label_name: String,
    /// Name of the environment variable group this label maps to.
    pub env_var_group: String,
    /// Description of what this label represents.
    pub description: String,
}

impl LabelMapping {
    /// Creates a new label mapping.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const because it takes owned String parameters
    pub fn new(label_name: String, env_var_group: String, description: String) -> Self {
        Self {
            label_name,
            env_var_group,
            description,
        }
    }
}

/// Trait that all application scanner plugins must implement.
pub trait ScannerPlugin: Send + Sync {
    /// Returns the name of this scanner (e.g., "ragit", "claude-desktop").
    fn name(&self) -> &str;

    /// Returns the application name (e.g., "Ragit", "Claude Desktop").
    fn app_name(&self) -> &str;

    /// Returns the paths that this scanner should scan for configuration files.
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>;

    /// Parses a configuration file and extracts discovered keys and config instances.
    /// # Errors
    /// Returns an error if the configuration file cannot be parsed or is invalid.
    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult>;

    /// Validates that this scanner can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool;

    /// Returns the environment variable schema for this scanner.
    /// Default implementation returns empty vector for backward compatibility.
    fn get_env_var_schema(&self) -> Vec<EnvVarDeclaration> {
        Vec::new()
    }

    /// Returns the label mappings for this scanner.
    /// Default implementation returns empty vector for backward compatibility.
    fn get_label_mappings(&self) -> Vec<LabelMapping> {
        Vec::new()
    }

    /// Scans for multiple instances of this application (e.g., multiple installations).
    /// # Errors
    /// Returns an error if scanning fails or configuration files cannot be read.
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
    #[must_use]
    pub const fn new() -> Self {
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            scanners: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new scanner.
    /// # Errors
    /// Returns an error if the scanner cannot be registered (e.g., already exists).
    pub fn register(&self, scanner: std::sync::Arc<dyn ScannerPlugin>) -> Result<()> {
        let mut scanners = self.scanners.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on scanners".to_string())
        })?;

        let name = scanner.name().to_string();
        if scanners.contains_key(&name) {
            return Err(Error::PluginError(format!(
                "Scanner '{name}' is already registered"
            )));
        }

        scanners.insert(name, scanner);
        drop(scanners); // Explicitly drop the lock to avoid significant_drop_tightening warning
        Ok(())
    }

    /// Gets a scanner by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn ScannerPlugin>> {
        self.scanners
            .read()
            .ok()
            .and_then(|scanners| scanners.get(name).cloned())
    }

    /// Lists all registered scanner names.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.scanners
            .read()
            .ok()
            .map(|scanners| scanners.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets all scanners that can handle a specific file.
    #[must_use]
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

// =============================================================================
// Discovery Helper Functions
// =============================================================================
// These functions help reduce code duplication across scanner implementations

/// Helper function to parse JSON config files.
/// # Errors
/// Returns an error if the JSON content cannot be parsed.
pub fn parse_json_config(content: &str) -> Result<serde_json::Value> {
    serde_json::from_str(content)
        .map_err(|e| Error::ConfigError(format!("Failed to parse JSON: {e}")))
}

/// Helper function to parse YAML config files.
/// # Errors
/// Returns an error if the YAML content cannot be parsed.
pub fn parse_yaml_config(content: &str) -> Result<serde_yaml::Value> {
    serde_yaml::from_str(content)
        .map_err(|e| Error::ConfigError(format!("Failed to parse YAML: {e}")))
}

/// Helper function to read and parse a JSON file.
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn read_json_file(path: &Path) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::ConfigError(format!("Failed to read {}: {e}", path.display())))?;
    parse_json_config(&content)
}

/// Helper function to read and parse a YAML file.
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn read_yaml_file(path: &Path) -> Result<serde_yaml::Value> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::ConfigError(format!("Failed to read {}: {e}", path.display())))?;
    parse_yaml_config(&content)
}

/// Helper to find config files that exist from a list of potential paths.
#[must_use] 
pub fn find_existing_configs(home_dir: &Path, relative_paths: &[&str]) -> Vec<PathBuf> {
    relative_paths
        .iter()
        .map(|p| home_dir.join(p))
        .filter(|p| p.exists())
        .collect()
}

/// Helper function to extract keys from environment variable format.
/// # Errors
/// Returns an error if regex pattern compilation fails.
///
/// # Panics
/// Panics if some regex patterns are invalid.
#[must_use]
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
                    (*provider).to_string(),
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

/// Helper function to extract keys and metadata from environment variable format.
/// This function extracts both API keys and metadata (`base_url`, `model_id`, etc.)
/// # Errors
/// Returns an error if regex pattern compilation fails.
///
/// # Panics
/// Panics if some regex patterns are invalid.
#[must_use]
pub fn extract_env_keys_with_metadata(
    content: &str,
    api_patterns: &[(&str, &str)],
    metadata_patterns: &[(&str, &str, &str)],
) -> Vec<DiscoveredKey> {
    let mut keys = Vec::new();

    // First, extract API keys
    for (env_var, provider) in api_patterns {
        let pattern = format!(r"(?i){}\s*=\s*(.+)", regex::escape(env_var));
        let regex = regex::Regex::new(&pattern).unwrap();

        for cap in regex.captures_iter(content) {
            if let Some(key_match) = cap.get(1) {
                let key_value = key_match.as_str().trim_matches('"').trim();

                // Only add if it's a reasonable API key length
                if key_value.len() >= 8
                    && key_value
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    let discovered_key = DiscoveredKey::new(
                        (*provider).to_string(),
                        "env_file".to_string(),
                        crate::models::discovered_key::ValueType::ApiKey,
                        crate::models::discovered_key::Confidence::High,
                        key_value.to_string(),
                    );

                    keys.push(discovered_key);
                }
            }
        }
    }

    // Then, extract metadata
    for (env_var, provider, custom_type) in metadata_patterns {
        let pattern = format!(r"(?i){}\s*=\s*(.+)", regex::escape(env_var));
        let regex = regex::Regex::new(&pattern).unwrap();

        for cap in regex.captures_iter(content) {
            if let Some(value_match) = cap.get(1) {
                let value = value_match.as_str().trim_matches('"').trim();

                if !value.is_empty() {
                    // Map special custom types to their proper ValueType variants
                    let value_type = match *custom_type {
                        "ModelId" => crate::models::discovered_key::ValueType::ModelId,
                        "BaseUrl" => crate::models::discovered_key::ValueType::BaseUrl,
                        "Temperature" => crate::models::discovered_key::ValueType::Temperature,
                        _ => crate::models::discovered_key::ValueType::Custom(
                            (*custom_type).to_string(),
                        ),
                    };

                    let discovered_key = DiscoveredKey::new(
                        (*provider).to_string(),
                        "env_file".to_string(),
                        value_type,
                        crate::models::discovered_key::Confidence::High,
                        value.to_string(),
                    );

                    keys.push(discovered_key);
                }
            }
        }
    }

    keys
}

/// Extension trait for `ScannerPlugin` providing helper functions to build `ProviderInstance` objects.
pub trait ScannerPluginExt: ScannerPlugin {
    /// Groups discovered keys by provider.
    ///
    /// This function takes a list of discovered keys and groups them by their provider name,
    /// returning a `HashMap` where the key is the provider name and the value is a vector of
    /// discovered keys for that provider.
    ///
    /// # Arguments
    /// * `keys` - A slice of `DiscoveredKey` objects to group
    ///
    /// # Returns
    /// A `HashMap` mapping provider names to their associated discovered keys
    fn group_keys_by_provider(
        &self,
        keys: &[DiscoveredKey],
    ) -> HashMap<String, Vec<DiscoveredKey>> {
        let mut grouped: HashMap<String, Vec<DiscoveredKey>> = HashMap::new();

        for key in keys {
            grouped
                .entry(key.provider.clone())
                .or_default()
                .push(key.clone());
        }

        tracing::debug!(
            "Grouped {} keys into {} providers",
            keys.len(),
            grouped.len()
        );

        grouped
    }

    /// Builds `ProviderInstance` objects from discovered keys.
    ///
    /// This function takes discovered keys grouped by provider and creates `ProviderInstance`
    /// objects with proper API keys, models, and settings. It handles the mapping from
    /// DiscoveredKey.ValueType to `ProviderInstance` fields.
    ///
    /// # Arguments
    /// * `grouped_keys` - A `HashMap` of provider names to their discovered keys
    /// * `source_path` - The source file path where keys were discovered
    /// * `plugin_registry` - Optional plugin registry for API-based model discovery
    ///
    /// # Returns
    /// A Result containing a vector of `ProviderInstance` objects
    ///
    /// # Errors
    /// Returns an error if:
    /// - Provider instance creation fails
    /// - Key validation fails
    /// - Required fields are missing
    #[allow(clippy::cognitive_complexity)]
    fn build_provider_instances(
        &self,
        grouped_keys: HashMap<String, Vec<DiscoveredKey>>,
        source_path: &str,
        plugin_registry: Option<&crate::plugins::PluginRegistry>,
    ) -> Result<Vec<ProviderInstance>> {
        let mut instances = Vec::new();

        for (provider_name, keys) in grouped_keys {
            tracing::info!(
                "Building provider instance for '{}' with {} keys",
                provider_name,
                keys.len()
            );

            // Separate keys by their value type
            let mut api_keys = Vec::new();
            let mut base_url: Option<String> = None;
            let mut model_ids = Vec::new();
            let mut temperature: Option<f32> = None;
            let mut metadata: HashMap<String, String> = HashMap::new();

            for key in &keys {
                match &key.value_type {
                    ValueType::ApiKey
                    | ValueType::AccessToken
                    | ValueType::SecretKey
                    | ValueType::BearerToken => {
                        if let Some(value) = key.full_value() {
                            api_keys.push((key, value.to_string()));
                        } else {
                            tracing::warn!(
                                "Skipping key without full value for provider '{}' from {}",
                                provider_name,
                                source_path
                            );
                        }
                    }
                    ValueType::BaseUrl => {
                        if let Some(value) = key.full_value() {
                            base_url = Some(value.to_string());
                            tracing::debug!("Found base URL for '{}': {}", provider_name, value);
                        }
                    }
                    ValueType::ModelId => {
                        if let Some(value) = key.full_value() {
                            model_ids.push(value.to_string());
                            tracing::debug!("Found model ID for '{}': {}", provider_name, value);
                        }
                    }
                    ValueType::Temperature => {
                        if let Some(value) = key.full_value() {
                            if let Ok(temp) = value.parse::<f32>() {
                                temperature = Some(temp);
                                tracing::debug!(
                                    "Found temperature for '{}': {}",
                                    provider_name,
                                    temp
                                );
                            } else {
                                tracing::warn!(
                                    "Invalid temperature value '{}' for provider '{}'",
                                    value,
                                    provider_name
                                );
                            }
                        }
                    }
                    ValueType::ParallelToolCalls => {
                        if let Some(value) = key.full_value() {
                            metadata.insert("parallel_tool_calls".to_string(), value.to_string());
                        }
                    }
                    ValueType::Headers => {
                        if let Some(value) = key.full_value() {
                            metadata.insert("headers".to_string(), value.to_string());
                        }
                    }
                    ValueType::Custom(custom_type) => {
                        if let Some(value) = key.full_value() {
                            metadata.insert(custom_type.clone(), value.to_string());
                            tracing::debug!(
                                "Found custom metadata '{}' for '{}': {}",
                                custom_type,
                                provider_name,
                                value
                            );
                        }
                    }
                }
            }

            // Validate that we have at least one API key
            if api_keys.is_empty() {
                tracing::warn!(
                    "No API keys found for provider '{}', skipping instance creation",
                    provider_name
                );
                continue;
            }

            // Use default base URL if not provided
            let final_base_url = base_url.unwrap_or_else(|| {
                let default_url = format!("https://api.{}.com", provider_name.to_lowercase());
                tracing::debug!(
                    "No base URL found for '{}', using default: {}",
                    provider_name,
                    default_url
                );
                default_url
            });

            // Create instance ID using SHA-256 hash for consistency
            let instance_id_source = format!("{provider_name}:{source_path}");
            let mut hasher = sha2::Sha256::new();
            hasher.update(instance_id_source.as_bytes());
            let hash_result = hasher.finalize();
            let full_hash = format!("{hash_result:x}");
            let instance_id = full_hash[..4].to_string();

            // Create the provider instance
            let mut instance = ProviderInstance::new_without_models(
                instance_id.clone(),
                provider_name.clone(),
                provider_name.to_lowercase(),
                final_base_url,
            );

            // Set the API key from the first discovered key
            if let Some((discovered_key, key_value)) = api_keys.first() {
                instance.set_api_key(key_value.clone());
                tracing::debug!(
                    "Set API key for instance '{}' (confidence: {})",
                    instance_id,
                    discovered_key.confidence
                );
            }

            // Add models if any were discovered
            for model_id in &model_ids {
                instance.add_model(model_id.clone());
                tracing::debug!("Added model '{}' to instance '{}'", model_id, instance_id);
            }

            // If no models were discovered and we have a plugin registry, try to probe for models
            if model_ids.is_empty() && plugin_registry.is_some() {
                if let Some(registry) = plugin_registry {
                    // Check if this is the anthropic provider
                    if provider_name.to_lowercase() == "anthropic" {
                        if let Some(plugin) = registry.get("anthropic") {
                            // Get the API key
                            if let Some(api_key) = instance.get_api_key() {
                                tracing::info!(
                                    "No models configured for Anthropic instance '{}', attempting to probe API",
                                    instance_id
                                );
                                // Try to fetch models from the API
                                match plugin.probe_models(api_key) {
                                    Ok(probed_models) if !probed_models.is_empty() => {
                                        tracing::info!(
                                                "Successfully probed {} models from Anthropic API for instance '{}'",
                                                probed_models.len(),
                                                instance_id
                                            );
                                        for model_id in probed_models {
                                            instance.add_model(model_id.clone());
                                        }
                                    }
                                    Ok(_) => {
                                        tracing::warn!(
                                                "Anthropic API probe returned no models for instance '{}'",
                                                instance_id
                                            );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                                "Failed to probe Anthropic API for models (instance '{}'): {}. Continuing without API-discovered models.",
                                                instance_id,
                                                e
                                            );
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                tracing::debug!(
                    "Skipping model probing for provider '{}': plugin_registry={:?}, model_ids.len={}",
                    provider_name,
                    plugin_registry.is_some(),
                    model_ids.len()
                );
            }

            // Add temperature to metadata if present
            if let Some(temp) = temperature {
                metadata.insert("temperature".to_string(), temp.to_string());
            }

            // Set metadata if any was collected
            if !metadata.is_empty() {
                instance = instance.with_metadata(metadata);
                tracing::debug!(
                    "Added {} metadata entries to instance '{}'",
                    instance.metadata.len(),
                    instance_id
                );
            }

            // Validate the instance before adding
            if let Err(e) = instance.validate() {
                tracing::error!(
                    "Failed to validate provider instance '{}': {}",
                    instance_id,
                    e
                );
                return Err(Error::ConfigError(format!(
                    "Invalid provider instance '{instance_id}': {e}"
                )));
            }

            tracing::info!(
                "Successfully created provider instance '{}' with API key and {} models",
                instance_id,
                instance.model_count()
            );

            instances.push(instance);
        }

        Ok(instances)
    }

    /// Convenience method to build provider instances directly from discovered keys.
    ///
    /// This combines grouping and building into a single operation.
    ///
    /// # Arguments
    /// * `keys` - A slice of `DiscoveredKey` objects
    /// * `source_path` - The source file path where keys were discovered
    /// * `plugin_registry` - Optional plugin registry for API-based model discovery
    ///
    /// # Returns
    /// A Result containing a vector of `ProviderInstance` objects
    ///
    /// # Errors
    /// Returns an error if instance building fails
    fn build_instances_from_keys(
        &self,
        keys: &[DiscoveredKey],
        source_path: &str,
        plugin_registry: Option<&crate::plugins::PluginRegistry>,
    ) -> Result<Vec<ProviderInstance>> {
        tracing::info!(
            "Building provider instances from {} discovered keys in {}",
            keys.len(),
            source_path
        );

        let grouped = self.group_keys_by_provider(keys);
        self.build_provider_instances(grouped, source_path, plugin_registry)
    }
}

// Blanket implementation for all types that implement ScannerPlugin
impl<T: ScannerPlugin + ?Sized> ScannerPluginExt for T {}

/// Registers all built-in scanner plugins.
/// # Errors
/// Returns an error if a scanner fails to register.
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

    // Mock scanner for testing
    struct MockScanner;

    impl ScannerPlugin for MockScanner {
        fn name(&self) -> &'static str {
            "mock-scanner"
        }

        fn app_name(&self) -> &'static str {
            "Mock Scanner"
        }

        fn scan_paths(&self, _home_dir: &Path) -> Vec<PathBuf> {
            vec![]
        }

        fn parse_config(&self, _path: &Path, _content: &str) -> Result<ScanResult> {
            Ok(ScanResult::new())
        }

        fn can_handle_file(&self, _path: &Path) -> bool {
            true
        }
    }

    #[test]
    fn test_group_keys_by_provider() {
        let scanner = MockScanner;

        let keys = vec![
            DiscoveredKey::new(
                "OpenAI".to_string(),
                "/test/config".to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-test123".to_string(),
            ),
            DiscoveredKey::new(
                "OpenAI".to_string(),
                "/test/config".to_string(),
                ValueType::BaseUrl,
                Confidence::High,
                "https://api.openai.com".to_string(),
            ),
            DiscoveredKey::new(
                "Anthropic".to_string(),
                "/test/config".to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-ant-test456".to_string(),
            ),
        ];

        let grouped = scanner.group_keys_by_provider(&keys);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("OpenAI").unwrap().len(), 2);
        assert_eq!(grouped.get("Anthropic").unwrap().len(), 1);
    }

    #[test]
    fn test_build_provider_instances_basic() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![DiscoveredKey::new(
                "OpenAI".to_string(),
                "/test/config".to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-test123456789".to_string(),
            )],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        assert_eq!(instance.provider_type, "openai");
        assert!(instance.has_non_empty_api_key());
    }

    #[test]
    fn test_build_provider_instances_with_metadata() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ApiKey,
                    Confidence::High,
                    "sk-test123456789".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::BaseUrl,
                    Confidence::High,
                    "https://api.openai.com".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ModelId,
                    Confidence::High,
                    "gpt-4".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::Temperature,
                    Confidence::High,
                    "0.7".to_string(),
                ),
            ],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        assert_eq!(instance.base_url, "https://api.openai.com");
        assert_eq!(instance.model_count(), 1);
        assert_eq!(instance.models[0].model_id, "gpt-4");
        assert!(!instance.metadata.is_empty());
        assert_eq!(
            instance.metadata.get("temperature"),
            Some(&"0.7".to_string())
        );
    }

    #[test]
    fn test_build_provider_instances_multiple_keys() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ApiKey,
                    Confidence::High,
                    "sk-prod-key".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ApiKey,
                    Confidence::Medium,
                    "sk-dev-key".to_string(),
                ),
            ],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        // Verify API key is present (ProviderInstance only stores one key)
        assert!(instance.has_non_empty_api_key());

        // Note: ProviderInstance only stores one API key, not multiple keys
        // Metadata is only set if there are special value types (temperature, headers, etc.)
        // In this test, we only have API keys, so metadata will be None
    }

    #[test]
    fn test_build_provider_instances_no_api_keys() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::BaseUrl,
                    Confidence::High,
                    "https://api.openai.com".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ModelId,
                    Confidence::High,
                    "gpt-4".to_string(),
                ),
            ],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        // Should skip provider without API keys
        assert_eq!(instances.len(), 0);
    }

    #[test]
    fn test_build_provider_instances_custom_metadata() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ApiKey,
                    Confidence::High,
                    "sk-test123".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::Custom("organization_id".to_string()),
                    Confidence::High,
                    "org-123456".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ParallelToolCalls,
                    Confidence::High,
                    "true".to_string(),
                ),
            ],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        let metadata = instance.metadata.as_ref().unwrap();
        assert_eq!(
            metadata.get("organization_id"),
            Some(&"org-123456".to_string())
        );
        assert_eq!(
            metadata.get("parallel_tool_calls"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_build_instances_from_keys() {
        let scanner = MockScanner;

        let keys = vec![
            DiscoveredKey::new(
                "OpenAI".to_string(),
                "/test/config".to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-test123".to_string(),
            ),
            DiscoveredKey::new(
                "Anthropic".to_string(),
                "/test/config".to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-ant-test456".to_string(),
            ),
        ];

        let instances = scanner
            .build_instances_from_keys(&keys, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 2);
        assert!(instances.iter().any(|i| i.provider_type == "openai"));
        assert!(instances.iter().any(|i| i.provider_type == "anthropic"));
    }

    #[test]
    fn test_build_provider_instances_invalid_temperature() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        grouped.insert(
            "OpenAI".to_string(),
            vec![
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::ApiKey,
                    Confidence::High,
                    "sk-test123".to_string(),
                ),
                DiscoveredKey::new(
                    "OpenAI".to_string(),
                    "/test/config".to_string(),
                    ValueType::Temperature,
                    Confidence::High,
                    "invalid".to_string(),
                ),
            ],
        );

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        // Should still create instance, just skip invalid temperature
        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        assert!(
            instance.metadata.is_empty()
                || !instance.metadata.contains_key("temperature")
        );
    }

    #[test]
    fn test_build_provider_instances_with_line_numbers() {
        let scanner = MockScanner;

        let mut grouped = HashMap::new();
        let mut key = DiscoveredKey::new(
            "OpenAI".to_string(),
            "/test/config".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "sk-test123".to_string(),
        );
        key = key.with_position(42, 10);

        grouped.insert("OpenAI".to_string(), vec![key]);

        let instances = scanner
            .build_provider_instances(grouped, "/test/config", None)
            .unwrap();

        assert_eq!(instances.len(), 1);
        let instance = &instances[0];
        // Metadata is only set if there are special value types (temperature, headers, etc.)
        // Line numbers from DiscoveredKey are not automatically stored in instance metadata
        // unless the instance goes through ProviderConfig conversion
    }
}
