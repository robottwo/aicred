//! Core library for genai-keyfinder - discovers AI API keys in configuration files.
//!
//! This library provides functionality to scan home directories and configuration
//! files for AI service API keys from various providers like OpenAI, Anthropic, Google, etc.
//!
//! # Example
//!
//! ```rust
//! use genai_keyfinder_core::{scan, ScanOptions, PluginRegistry};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a plugin registry (plugins will be added here)
//! let registry = PluginRegistry::new();
//!
//! // Configure scan options
//! let temp_dir = tempfile::tempdir()?;
//! let options = ScanOptions {
//!     home_dir: Some(temp_dir.path().to_path_buf()),
//!     include_full_values: false,
//!     max_file_size: 1024 * 1024, // 1MB
//!     only_providers: None,
//!     exclude_providers: None,
//! };
//!
//! // Run the scan
//! let result = scan(options)?;
//! println!("Found {} keys", result.total_keys());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod error;
pub mod models;
pub mod parser;
pub mod plugins;
pub mod providers;
pub mod scanner;
pub mod scanners;

pub use error::{Error, Result};
pub use models::{
    AuthMethod, Capabilities, Confidence, ConfigInstance, DiscoveredKey, Model, Provider,
    RateLimit, ScanResult, ScanSummary, ValueType,
};
pub use parser::{ConfigParser, FileFormat};
pub use plugins::{register_builtin_plugins, CommonConfigPlugin, PluginRegistry, ProviderPlugin};
pub use scanner::{Scanner, ScannerConfig, DEFAULT_MAX_FILE_SIZE};
pub use scanners::{register_builtin_scanners, ScannerPlugin, ScannerRegistry};

use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Options for configuring a scan operation.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Home directory to scan (defaults to user's home directory).
    pub home_dir: Option<PathBuf>,
    /// Whether to include full key values in results (default: false for security).
    pub include_full_values: bool,
    /// Maximum file size to scan in bytes (default: 1MB).
    pub max_file_size: usize,
    /// Only scan specific providers (optional).
    pub only_providers: Option<Vec<String>>,
    /// Exclude specific providers (optional).
    pub exclude_providers: Option<Vec<String>>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            home_dir: None,
            include_full_values: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            only_providers: None,
            exclude_providers: None,
        }
    }
}

impl ScanOptions {
    /// Creates a new ScanOptions with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the home directory to scan.
    pub fn with_home_dir(mut self, home_dir: PathBuf) -> Self {
        self.home_dir = Some(home_dir);
        self
    }

    /// Sets whether to include full key values.
    pub fn with_full_values(mut self, include: bool) -> Self {
        self.include_full_values = include;
        self
    }

    /// Sets the maximum file size to scan.
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Sets specific providers to scan.
    pub fn with_only_providers(mut self, providers: Vec<String>) -> Self {
        self.only_providers = Some(providers);
        self
    }

    /// Sets providers to exclude.
    pub fn with_exclude_providers(mut self, providers: Vec<String>) -> Self {
        self.exclude_providers = Some(providers);
        self
    }

    /// Gets the effective home directory (either provided or user's home).
    pub fn get_home_dir(&self) -> Result<PathBuf> {
        if let Some(ref home) = self.home_dir {
            Ok(home.clone())
        } else {
            dirs_next::home_dir()
                .ok_or_else(|| Error::ConfigError("Could not determine home directory".to_string()))
        }
    }
}

/// Main entry point for scanning for API keys.
///
/// This function orchestrates the entire scanning process:
/// 1. Creates plugin and scanner registries with available plugins/scanners
/// 2. Configures the scanner with the provided options
/// 3. Scans the home directory for configuration files
/// 4. Parses found files and extracts API keys using both providers and scanners
/// 5. Returns the results
///
/// # Arguments
///
/// * `options` - Configuration options for the scan operation
///
/// # Returns
///
/// A `ScanResult` containing all discovered keys and scan metadata
///
/// # Errors
///
/// Returns an error if the scan fails due to IO errors, invalid configuration, etc.
pub fn scan(options: ScanOptions) -> Result<ScanResult> {
    // Get the home directory to scan
    let home_dir = options.get_home_dir()?;

    // Create plugin registry for key validation (providers no longer handle scanning)
    let provider_registry = create_default_registry()?;

    // Create scanner registry and register available scanners (applications and providers)
    let scanner_registry = create_default_scanner_registry()?;

    // Filter plugins based on options (for key validation only)
    let filtered_provider_registry = filter_registry(provider_registry, &options)?;

    // Filter scanners based on options
    let filtered_scanner_registry = filter_scanner_registry(scanner_registry, &options)?;

    // Create scanner configuration
    let scanner_config = ScannerConfig {
        max_file_size: options.max_file_size,
        ..ScannerConfig::default()
    };

    // Create scanner with provider registry for key validation only
    // Create scanner with provider registry only for key validation
    let scanner = Scanner::with_config(filtered_provider_registry.clone(), scanner_config)
        .with_scanner_registry(filtered_scanner_registry.clone());

    // Initialize result without scanning entire directory
    let scan_started_at = chrono::Utc::now();
    let mut result = ScanResult::new(
        home_dir.display().to_string(),
        filtered_provider_registry.list(),
        scan_started_at,
    );

    // Run targeted scanner-specific scanning only
    let scanner_results = scan_with_scanners(&filtered_scanner_registry, &home_dir)?;

    // Process scanner results and validate keys with provider plugins
    for (_scanner_name, mut scan_result) in scanner_results {
        // Validate discovered keys using provider plugins for confidence scoring
        for key in &mut scan_result.keys {
            if let Some(plugin) = filtered_provider_registry.get(&key.provider) {
                // Use provider plugin to validate and score the key
                if let Some(full_value) = key.full_value() {
                    let confidence_score = plugin.confidence_score(full_value);
                    // For now, we validate but don't modify the key structure
                    // The scanner has already determined the confidence
                    debug!(
                        "Validated key from {} with confidence {}",
                        key.provider, confidence_score
                    );
                }
            }
        }

        result.add_keys(scan_result.keys);
        for instance in scan_result.instances {
            result.add_config_instance(instance);
        }
    }

    // Apply redaction if needed
    if !options.include_full_values {
        result.keys = result
            .keys
            .into_iter()
            .map(|key| key.with_full_value(false))
            .collect();
    }

    Ok(result)
}

/// Creates a default plugin registry with built-in plugins.
fn create_default_registry() -> Result<PluginRegistry> {
    let registry = PluginRegistry::new();

    // Register all built-in plugins
    register_builtin_plugins(&registry)?;

    Ok(registry)
}

/// Creates a default scanner registry with built-in scanners.
fn create_default_scanner_registry() -> Result<ScannerRegistry> {
    let registry = ScannerRegistry::new();

    // Register all built-in scanners
    register_builtin_scanners(&registry)?;

    Ok(registry)
}

/// Scans using application scanners to find config instances.
fn scan_with_scanners(
    scanner_registry: &ScannerRegistry,
    home_dir: &std::path::Path,
) -> Result<Vec<(String, scanners::ScanResult)>> {
    let mut results = Vec::new();

    for scanner_name in scanner_registry.list() {
        if let Some(scanner) = scanner_registry.get(&scanner_name) {
            let mut scan_result = scanners::ScanResult::new();

            // Scan for application instances
            if let Ok(instances) = scanner.scan_instances(home_dir) {
                for instance in instances {
                    scan_result.add_instance(instance);
                }
            }

            // If this scanner supports provider scanning, scan for provider configurations
            if scanner.supports_provider_scanning() {
                // Use a HashSet to track unique paths we've already scanned
                let mut scanned_paths = std::collections::HashSet::new();

                // Get all potential paths from both provider configs and app paths
                let provider_paths = scanner.scan_provider_configs(home_dir).unwrap_or_default();
                let app_paths = scanner.scan_paths(home_dir);

                // Combine all paths and scan each unique one only once
                let all_paths: Vec<_> = provider_paths
                    .into_iter()
                    .chain(app_paths.into_iter())
                    .filter(|path| path.exists() && scanned_paths.insert(path.clone()))
                    .collect();

                for path in all_paths {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(result) = scanner.parse_config(&path, &content) {
                            // Extract provider keys from application configs
                            for key in result.keys {
                                // Only include keys that match supported providers
                                if scanner.supported_providers().contains(&key.provider) {
                                    scan_result.add_key(key);
                                }
                            }

                            // Add instances
                            for instance in result.instances {
                                scan_result.add_instance(instance);
                            }
                        }
                    }
                }
            }

            // Only include results if we found something
            if !scan_result.keys.is_empty() || !scan_result.instances.is_empty() {
                results.push((scanner_name, scan_result));
            }
        }
    }

    Ok(results)
}

/// Filters the scanner registry based on scan options.
fn filter_scanner_registry(
    registry: ScannerRegistry,
    options: &ScanOptions,
) -> Result<ScannerRegistry> {
    let filtered_registry = ScannerRegistry::new();

    let all_scanners = registry.list();

    for scanner_name in all_scanners {
        // Check if we should include this scanner
        let should_include = if let Some(ref only_providers) = options.only_providers {
            only_providers.contains(&scanner_name)
        } else if let Some(ref exclude_providers) = options.exclude_providers {
            !exclude_providers.contains(&scanner_name)
        } else {
            true
        };

        if should_include {
            if let Some(scanner) = registry.get(&scanner_name) {
                filtered_registry.register(scanner)?;
            }
        }
    }

    Ok(filtered_registry)
}

/// Filters the plugin registry based on scan options.
fn filter_registry(registry: PluginRegistry, options: &ScanOptions) -> Result<PluginRegistry> {
    let filtered_registry = PluginRegistry::new();

    let all_plugins = registry.list();

    for plugin_name in all_plugins {
        // Check if we should include this plugin
        let should_include = if let Some(ref only_providers) = options.only_providers {
            only_providers.contains(&plugin_name)
        } else if let Some(ref exclude_providers) = options.exclude_providers {
            !exclude_providers.contains(&plugin_name)
        } else {
            true
        };

        if should_include {
            if let Some(plugin) = registry.get(&plugin_name) {
                filtered_registry.register(plugin)?;
            }
        }
    }

    if filtered_registry.is_empty() {
        return Err(Error::ConfigError(
            "No plugins available after filtering".to_string(),
        ));
    }

    Ok(filtered_registry)
}

/// Utility function to get the default home directory.
pub fn default_home_dir() -> Result<PathBuf> {
    dirs_next::home_dir()
        .ok_or_else(|| Error::ConfigError("Could not determine home directory".to_string()))
}

/// Utility function to check if a path is a configuration file.
pub fn is_config_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_str.as_str(),
            "json" | "yaml" | "yml" | "toml" | "ini" | "env" | "conf" | "config"
        )
    } else {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        matches!(
            file_name.as_ref(),
            ".env" | ".envrc" | "config" | "settings" | "preferences"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_scan_options_default() {
        let options = ScanOptions::default();
        assert!(!options.include_full_values);
        assert_eq!(options.max_file_size, DEFAULT_MAX_FILE_SIZE);
        assert!(options.only_providers.is_none());
        assert!(options.exclude_providers.is_none());
    }

    #[test]
    fn test_scan_options_builder() {
        let options = ScanOptions::new()
            .with_full_values(true)
            .with_max_file_size(2048);

        assert!(options.include_full_values);
        assert_eq!(options.max_file_size, 2048);
    }

    #[test]
    fn test_is_config_file() {
        assert!(is_config_file(std::path::Path::new("test.json")));
        assert!(is_config_file(std::path::Path::new("config.yaml")));
        assert!(is_config_file(Path::new(".env")));
        assert!(!is_config_file(std::path::Path::new("document.txt")));
        assert!(!is_config_file(std::path::Path::new("image.png")));
    }

    #[test]
    fn test_create_default_registry() {
        let registry = create_default_registry().unwrap();
        assert!(!registry.is_empty());
        assert!(registry.get("common-config").is_some());
    }

    #[test]
    fn test_filter_registry() {
        let registry = create_default_registry().unwrap();

        // Test with only_providers
        let options = ScanOptions::new().with_only_providers(vec!["common-config".to_string()]);
        let filtered = filter_registry(registry.clone(), &options).unwrap();
        assert!(!filtered.is_empty());

        // Test with exclude_providers
        let options = ScanOptions::new().with_exclude_providers(vec!["nonexistent".to_string()]);
        let filtered = filter_registry(registry, &options).unwrap();
        assert!(!filtered.is_empty());
    }
}
