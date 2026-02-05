// Allow specific clippy lints that are too pedantic for this codebase
// Phase 4 cleanup: Removing allows one by one
// Removed - let's fix these now:
// #![allow(clippy::needless_borrow)]
// #![allow(clippy::module_inception)]
// #![allow(clippy::float_cmp)]
// #![allow(clippy::len_zero)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]

//! Core library for aicred - discovers AI API keys in configuration files.
//!
//! This library provides functionality to scan home directories and configuration
//! files for AI service API keys from various providers like `OpenAI`, `Anthropic`, Google, etc.
//!
//! # API Versions
//!
//! As of v0.2.0, two APIs are available:
//!
//! - **Legacy API** (v0.1.x): `DiscoveredKey`, `Provider`, `Model`, etc. - Still works, deprecated.
//! - **New API** (v0.2.0+): `DiscoveredCredential`, `ProviderNew`, `ModelNew`, `LabelNew`, etc. - Recommended.
//!
//! The new API provides cleaner naming and better structure. Both APIs work in v0.2.x.
//! Legacy types will be removed in v0.3.0.
//!
//! # Example (Legacy API)
//!
//! ```rust
//! use aicred_core::{scan, ScanOptions, PluginRegistry};
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
//!     probe_models: false,
//!     probe_timeout_secs: 30,
//! };
//!
//! // Run the scan
//! let result = scan(&options)?;
//! println!("Found {} keys", result.total_keys());
//! # Ok(())
//! # }
//! ```
//!
//! # Example (New API v0.2.0+)
//!
//! ```rust
//! use aicred_core::{scan, ScanOptions};
//! use aicred_core::{DiscoveredCredential, LabelNew, ProviderNew};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let temp_dir = tempfile::tempdir()?;
//! let options = ScanOptions {
//!     home_dir: Some(temp_dir.path().to_path_buf()),
//!     include_full_values: false,
//!     max_file_size: 1024 * 1024,
//!     only_providers: None,
//!     exclude_providers: None,
//!     probe_models: false,
//!     probe_timeout_secs: 30,
//! };
//!
//! let result = scan(&options)?;
//! println!("Found {} credentials", result.total_keys());
//! # Ok(())
//! # }
//! ```
//!
//! See `MIGRATION_0.1_to_0.2.md` for migration guide.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod env_resolver;
pub mod error;
pub mod models;
pub mod parser;
pub mod plugins;
pub mod providers;
pub mod discovery;
pub mod scanners; // Backward compatibility re-export
pub mod utils;

pub use env_resolver::{EnvResolutionResult, EnvResolver, EnvResolverBuilder, EnvVarMapping};
pub use error::{Error, Result};

// Primary API exports (v0.2.0 - canonical types)
pub use models::{
    // Credentials & Discovery
    DiscoveredCredential,
    CredentialValue,
    Confidence,
    ValueType,
    Environment,
    ValidationStatus,
    // Labels
    Label,
    LabelAssignment,
    LabelTarget,
    LabelWithAssignments,
    // Models
    Model,
    ModelMetadata,
    ModelCapabilities,
    ModelPricing,
    TokenCost,
    // Providers
    Provider,
    ProviderInstance,
    ProviderCollection,
    AuthMethod,
    RateLimit,
    Capabilities,
    // Scan
    ScanResult,
    ScanSummary,
    // Config
    ConfigInstance,
};

// Internal export only (needed for ScanResult and backward compatibility)
// TODO: Remove when core library migrates to DiscoveredCredential
pub use models::discovered_key::DiscoveredKey;
pub use models::unified_label::UnifiedLabel;  // Temporary for wrap.rs compatibility

pub use parser::{ConfigParser, FileFormat};

// Plugin API exports
pub use plugins::{
    // Provider registry (v0.2.0)
    ProviderRegistry, register_builtin_providers,
    get_provider, list_providers, get_providers_for_file,
    // Legacy (still used internally)
    PluginRegistry, register_builtin_plugins,
    // Core traits
    ProviderPlugin, CommonConfigPlugin,
};

// Discovery system (application-specific credential scanners)
pub use crate::discovery::{
    register_builtin_scanners,
    ScannerConfig,
    ScannerPlugin,
    ScannerRegistry,
    DEFAULT_MAX_FILE_SIZE,
};
pub use utils::provider_model_tuple::ProviderModelTuple;

use std::path::PathBuf;
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
    /// Whether to probe provider instances for available models (default: false).
    pub probe_models: bool,
    /// Timeout for model probing in seconds (default: 30).
    pub probe_timeout_secs: u64,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            home_dir: None,
            include_full_values: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            only_providers: None,
            exclude_providers: None,
            probe_models: false,
            probe_timeout_secs: 30,
        }
    }
}

impl ScanOptions {
    /// Creates a new `ScanOptions` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the home directory to scan.
    #[must_use]
    pub fn with_home_dir(mut self, home_dir: PathBuf) -> Self {
        self.home_dir = Some(home_dir);
        self
    }

    /// Sets whether to include full key values.
    #[must_use]
    pub const fn with_full_values(mut self, include: bool) -> Self {
        self.include_full_values = include;
        self
    }

    /// Sets the maximum file size to scan.
    #[must_use]
    pub const fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Sets specific providers to scan.
    #[must_use]
    pub fn with_only_providers(mut self, providers: Vec<String>) -> Self {
        self.only_providers = Some(providers);
        self
    }

    /// Sets providers to exclude.
    #[must_use]
    pub fn with_exclude_providers(mut self, providers: Vec<String>) -> Self {
        self.exclude_providers = Some(providers);
        self
    }

    /// Gets the effective home directory (either provided or user's home).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined from the system.
    pub fn get_home_dir(&self) -> Result<PathBuf> {
        self.home_dir.as_ref().map_or_else(
            || {
                dirs_next::home_dir().ok_or_else(|| {
                    Error::ConfigError("Could not determine home directory".to_string())
                })
            },
            |home| Ok(home.clone()),
        )
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
#[allow(clippy::too_many_lines)]
pub fn scan(options: &ScanOptions) -> Result<ScanResult> {
    // Get the home directory to scan
    let home_dir = options.get_home_dir()?;

    // Create plugin registry for key validation (providers no longer handle scanning)
    let provider_registry = create_default_registry()?;

    // Create scanner registry and register available scanners (applications and providers)
    let scanner_registry = create_default_scanner_registry()?;

    // Filter plugins based on options (for key validation only)
    let filtered_provider_registry = filter_registry(&provider_registry, options)?;

    // Filter scanners based on options
    let filtered_scanner_registry = filter_scanner_registry(&scanner_registry, options)?;

    // Initialize result without scanning entire directory
    let scan_started_at = chrono::Utc::now();
    let mut result = ScanResult::new(
        home_dir.display().to_string(),
        filtered_provider_registry.list(),
        scan_started_at,
    );

    // Run targeted scanner-specific scanning only
    let scanner_results = scan_with_scanners(
        &filtered_scanner_registry,
        &filtered_provider_registry,
        &home_dir,
    );

    // Process scanner results and validate keys with provider plugins
    // Use a HashSet to track unique config instances by instance_id
    let mut seen_instances = std::collections::HashSet::new();

    for (scanner_name, mut scan_result) in scanner_results {
        debug!(
            "Processing {} keys from scanner: {}",
            scan_result.keys.len(),
            scanner_name
        );

        // Validate discovered keys using provider plugins for confidence scoring
        for key in &mut scan_result.keys {
            if let Some(plugin) = filtered_provider_registry.get(&key.provider) {
                // Use provider plugin to validate and score the key
                if let Some(full_value) = key.full_value() {
                    let confidence_score = plugin.confidence_score(full_value);
                    // For now, we validate but don't modify the key structure
                    // The scanner has already determined the confidence
                    debug!(
                        "Validated key from {} with confidence {} (hash: {})",
                        key.provider,
                        confidence_score,
                        &key.hash[..8]
                    );
                }
            }
        }

        debug!(
            "Adding {} keys from scanner {} to result",
            scan_result.keys.len(),
            scanner_name
        );
        let keys_before = result.keys.len();
        result.add_keys(scan_result.keys);
        debug!(
            "Result now has {} keys (added {})",
            result.keys.len(),
            result.keys.len() - keys_before
        );

        // Add config instances with deduplication
        for instance in scan_result.instances {
            if seen_instances.insert(instance.instance_id.clone()) {
                debug!(
                    "Adding config instance: {} ({})",
                    instance.app_name, instance.instance_id
                );
                result.add_config_instance(instance);
            } else {
                debug!(
                    "Skipping duplicate config instance: {} ({})",
                    instance.app_name, instance.instance_id
                );
            }
        }
    }

    // Probe provider instances for available models if requested
    if options.probe_models {
        debug!("Probing provider instances for available models...");
        let probe_stats = probe_provider_instances_async(
            &mut result.config_instances,
            &filtered_provider_registry,
            options.probe_timeout_secs,
        );

        debug!(
            "Probe complete: {}/{} instances probed successfully, {} models discovered",
            probe_stats.probed_successfully,
            probe_stats.total_instances,
            probe_stats.total_models_discovered
        );

        // Add probe statistics to result metadata
        let metadata = result
            .metadata
            .get_or_insert_with(std::collections::HashMap::new);
        metadata.insert(
            "probe_total_instances".to_string(),
            serde_json::json!(probe_stats.total_instances),
        );
        metadata.insert(
            "probe_successful".to_string(),
            serde_json::json!(probe_stats.probed_successfully),
        );
        metadata.insert(
            "probe_failures".to_string(),
            serde_json::json!(probe_stats.probe_failures),
        );
        metadata.insert(
            "probe_models_discovered".to_string(),
            serde_json::json!(probe_stats.total_models_discovered),
        );
    }

    // Apply selective redaction if needed
    // Always keep full values for non-sensitive fields like ModelId, but redact API keys
    if !options.include_full_values {
        let keys_before_redaction = result.keys.len();
        result.keys = result
            .keys
            .into_iter()
            .map(|key| {
                // Keep full values for non-sensitive value types
                use crate::models::discovered_key::ValueType as OldValueType;
                let should_preserve = match &key.value_type {
                    OldValueType::ModelId => {
                        tracing::debug!("Preserving ModelId key: {}", key.redacted_value());
                        true
                    }
                    OldValueType::Custom(name) if name == "ModelId" || name.contains("Model") => {
                        tracing::debug!("Preserving custom Model key: {}", name);
                        true
                    }
                    OldValueType::Custom(name) if name == "Temperature" || name == "BaseUrl" => true,
                    // Redact sensitive values like API keys
                    _ => false,
                };

                if should_preserve {
                    key
                } else {
                    tracing::trace!("Redacting key of type: {:?}", key.value_type);
                    key.with_full_value(false)
                }
            })
            .collect();

        tracing::info!(
            "Redaction complete: {} keys before, {} keys after ({} ModelId keys preserved)",
            keys_before_redaction,
            result.keys.len(),
            result
                .keys
                .iter()
                .filter(|k| matches!(k.value_type, crate::models::discovered_key::ValueType::ModelId))
                .count()
        );
    }

    // Set completion timestamp before returning
    result.set_completed();

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
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
fn scan_with_scanners(
    scanner_registry: &ScannerRegistry,
    plugin_registry: &PluginRegistry,
    home_dir: &std::path::Path,
) -> Vec<(String, scanners::ScanResult)> {
    let mut results = Vec::new();

    for scanner_name in scanner_registry.list() {
        debug!("Running scanner: {}", scanner_name);

        // Create scanner-specific instances to call _with_registry methods
        let mut scan_result = scanners::ScanResult::new();

        match scanner_name.as_str() {
            "claude-desktop" => {
                let scanner = scanners::ClaudeDesktopScanner;
                if let Ok(instances) =
                    scanner.scan_instances_with_registry(home_dir, Some(plugin_registry))
                {
                    debug!(
                        "Scanner {} found {} instances",
                        scanner_name,
                        instances.len()
                    );
                    for instance in instances {
                        scan_result.add_instance(instance);
                    }
                }

                let app_paths = scanner.scan_paths(home_dir);
                debug!(
                    "Scanner {} found {} app paths",
                    scanner_name,
                    app_paths.len()
                );

                let mut scanned_paths = std::collections::HashSet::new();
                for path in app_paths {
                    if path.exists() && scanned_paths.insert(path.clone()) {
                        debug!("Scanner {} scanning path: {}", scanner_name, path.display());
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(result) = scanner.parse_config_with_registry(
                                &path,
                                &content,
                                Some(plugin_registry),
                            ) {
                                debug!(
                                    "Scanner {} found {} keys and {} instances in {}",
                                    scanner_name,
                                    result.keys.len(),
                                    result.instances.len(),
                                    path.display()
                                );

                                for key in result.keys {
                                    debug!(
                                        "Scanner {} adding key for provider: {} (hash: {})",
                                        scanner_name,
                                        key.provider,
                                        &key.hash[..8]
                                    );
                                    scan_result.add_key(key);
                                }

                                for instance in result.instances {
                                    scan_result.add_instance(instance);
                                }
                            }
                        }
                    }
                }
            }
            "gsh" => {
                let scanner = scanners::GshScanner;
                if let Ok(instances) =
                    scanner.scan_instances_with_registry(home_dir, Some(plugin_registry))
                {
                    debug!(
                        "Scanner {} found {} instances",
                        scanner_name,
                        instances.len()
                    );
                    for instance in instances {
                        scan_result.add_instance(instance);
                    }
                }

                let app_paths = scanner.scan_paths(home_dir);
                debug!(
                    "Scanner {} found {} app paths",
                    scanner_name,
                    app_paths.len()
                );

                let mut scanned_paths = std::collections::HashSet::new();
                for path in app_paths {
                    if path.exists() && scanned_paths.insert(path.clone()) {
                        debug!("Scanner {} scanning path: {}", scanner_name, path.display());
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(result) = scanner.parse_config_with_registry(
                                &path,
                                &content,
                                Some(plugin_registry),
                            ) {
                                debug!(
                                    "Scanner {} found {} keys and {} instances in {}",
                                    scanner_name,
                                    result.keys.len(),
                                    result.instances.len(),
                                    path.display()
                                );

                                for key in result.keys {
                                    debug!(
                                        "Scanner {} adding key for provider: {} (hash: {})",
                                        scanner_name,
                                        key.provider,
                                        &key.hash[..8]
                                    );
                                    scan_result.add_key(key);
                                }

                                for instance in result.instances {
                                    scan_result.add_instance(instance);
                                }
                            }
                        }
                    }
                }
            }
            "roo-code" => {
                let scanner = scanners::RooCodeScanner;
                if let Ok(instances) = scanner.scan_instances(home_dir) {
                    debug!(
                        "Scanner {} found {} instances",
                        scanner_name,
                        instances.len()
                    );
                    for instance in instances {
                        scan_result.add_instance(instance);
                    }
                }

                let app_paths = scanner.scan_paths(home_dir);
                debug!(
                    "Scanner {} found {} app paths",
                    scanner_name,
                    app_paths.len()
                );

                let mut scanned_paths = std::collections::HashSet::new();
                for path in app_paths {
                    if path.exists() && scanned_paths.insert(path.clone()) {
                        debug!("Scanner {} scanning path: {}", scanner_name, path.display());
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(result) = scanner.parse_config_with_registry(
                                &path,
                                &content,
                                Some(plugin_registry),
                            ) {
                                debug!(
                                    "Scanner {} found {} keys and {} instances in {}",
                                    scanner_name,
                                    result.keys.len(),
                                    result.instances.len(),
                                    path.display()
                                );

                                for key in result.keys {
                                    debug!(
                                        "Scanner {} adding key for provider: {} (hash: {})",
                                        scanner_name,
                                        key.provider,
                                        &key.hash[..8]
                                    );
                                    scan_result.add_key(key);
                                }

                                for instance in result.instances {
                                    scan_result.add_instance(instance);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // For other scanners, use the default trait methods
                if let Some(scanner) = scanner_registry.get(&scanner_name) {
                    if let Ok(instances) = scanner.scan_instances(home_dir) {
                        debug!(
                            "Scanner {} found {} instances",
                            scanner_name,
                            instances.len()
                        );
                        for instance in instances {
                            scan_result.add_instance(instance);
                        }
                    }

                    let app_paths = scanner.scan_paths(home_dir);
                    debug!(
                        "Scanner {} found {} app paths",
                        scanner_name,
                        app_paths.len()
                    );

                    let mut scanned_paths = std::collections::HashSet::new();
                    for path in app_paths {
                        if path.exists() && scanned_paths.insert(path.clone()) {
                            debug!("Scanner {} scanning path: {}", scanner_name, path.display());
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(result) = scanner.parse_config(&path, &content) {
                                    debug!(
                                        "Scanner {} found {} keys and {} instances in {}",
                                        scanner_name,
                                        result.keys.len(),
                                        result.instances.len(),
                                        path.display()
                                    );

                                    for key in result.keys {
                                        debug!(
                                            "Scanner {} adding key for provider: {} (hash: {})",
                                            scanner_name,
                                            key.provider,
                                            &key.hash[..8]
                                        );
                                        scan_result.add_key(key);
                                    }

                                    for instance in result.instances {
                                        scan_result.add_instance(instance);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Only include results if we found something
        if !scan_result.keys.is_empty() || !scan_result.instances.is_empty() {
            debug!(
                "Scanner {} found {} keys and {} instances total",
                scanner_name,
                scan_result.keys.len(),
                scan_result.instances.len()
            );
            results.push((scanner_name, scan_result));
        }
    }

    results
}
/// Statistics from probing provider instances.
#[derive(Debug, Clone)]
pub struct ProbeStatistics {
    /// Total number of instances that were attempted to be probed.
    pub total_instances: usize,
    /// Number of instances successfully probed.
    pub probed_successfully: usize,
    /// Number of instances that failed to probe.
    pub probe_failures: usize,
    /// Total number of models discovered across all instances.
    pub total_models_discovered: usize,
}

/// Probes provider instances asynchronously to discover available models.
///
/// This function takes a mutable slice of `ConfigInstance`s and attempts to probe
/// each provider instance for available models using the provider's plugin.
/// Probing is done concurrently with a timeout to ensure responsiveness.
///
/// # Arguments
///
/// * `instances` - Mutable slice of config instances to probe
/// * `plugin_registry` - Registry containing provider plugins
/// * `timeout_secs` - Timeout in seconds for each probe operation
///
/// # Returns
///
/// Returns `ProbeStatistics` containing information about the probing operation.
///
/// # Errors
///
/// This function handles all errors gracefully and logs them. It never returns an error.
#[allow(clippy::too_many_lines)]
fn probe_provider_instances_async(
    instances: &mut [ConfigInstance],
    plugin_registry: &PluginRegistry,
    timeout_secs: u64,
) -> ProbeStatistics {
    use std::collections::HashMap;
    use tokio::time::{timeout, Duration};

    let mut stats = ProbeStatistics {
        total_instances: 0,
        probed_successfully: 0,
        probe_failures: 0,
        total_models_discovered: 0,
    };

    // Create a tokio runtime for async operations
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!("Failed to create tokio runtime for probing: {}", e);
            return stats;
        }
    };

    runtime.block_on(async {
        // Collect all probe tasks with their instance IDs for later lookup
        let mut probe_tasks = Vec::new();

        for instance in instances.iter() {
            for provider_instance in instance.provider_instances.all_instances() {
                stats.total_instances += 1;

                // Get the plugin for this provider
                let Some(plugin) = plugin_registry.get(&provider_instance.provider_type) else {
                    tracing::debug!(
                        "No plugin found for provider: {}",
                        provider_instance.provider_type
                    );
                    continue;
                };

                // Get API key if available
                if provider_instance.api_key.is_empty() {
                    tracing::debug!(
                        "No API key available for provider instance: {}",
                        provider_instance.id
                    );
                    continue;
                }
                let api_key = provider_instance.api_key.clone();

                // Get base URL
                let base_url = Some(provider_instance.base_url.as_str());

                // Clone what we need for the async task
                let plugin_clone = plugin.clone();
                let api_key_clone = api_key.clone();
                let base_url_clone = base_url.map(String::from);
                let provider_name = provider_instance.provider_type.clone();
                let instance_id = provider_instance.id.clone();
                let provider_instance_id = provider_instance.id.clone();

                // Spawn probe task with timeout
                let task = tokio::spawn(async move {
                    let probe_result = timeout(
                        Duration::from_secs(timeout_secs),
                        plugin_clone.probe_models_async(&api_key_clone, base_url_clone.as_deref()),
                    )
                    .await;

                    (
                        provider_instance_id,
                        instance_id,
                        provider_name,
                        probe_result,
                    )
                });

                probe_tasks.push(task);
            }
        }

        // Wait for all probe tasks to complete and collect results
        let mut probe_results = Vec::new();
        for task in probe_tasks {
            match task.await {
                Ok(result) => probe_results.push(result),
                Err(e) => {
                    tracing::error!("Probe task panicked: {}", e);
                    stats.probe_failures += 1;
                }
            }
        }

        // Now update the instances with the probe results
        for (provider_instance_id, instance_id, provider_name, probe_result) in probe_results {
            // Find the instance and provider instance to update
            for instance in instances.iter_mut() {
                if let Some(provider_instance) = instance
                    .provider_instances
                    .get_instance_mut(&provider_instance_id)
                {
                    // Update metadata
                    provider_instance.metadata.insert("probe_attempted".to_string(), "true".to_string());
                    provider_instance.metadata.insert(
                        "probe_timestamp".to_string(),
                        chrono::Utc::now().to_rfc3339(),
                    );

                    match probe_result {
                        Ok(Ok(models)) => {
                            tracing::info!(
                                "Successfully probed {} models from provider {} (instance: {})",
                                models.len(),
                                provider_name,
                                instance_id
                            );

                            // Extract model IDs from ModelMetadata
                            provider_instance.models =
                                models.into_iter()
                                    .filter_map(|m| m.id)
                                    .collect();

                            stats.probed_successfully += 1;
                            stats.total_models_discovered += provider_instance.models.len();

                            provider_instance.metadata.insert("probe_success".to_string(), "true".to_string());
                            provider_instance.metadata.insert(
                                "models_count".to_string(),
                                provider_instance.models.len().to_string(),
                            );
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(
                                "Failed to probe provider {} (instance: {}): {}",
                                provider_name,
                                instance_id,
                                e
                            );
                            stats.probe_failures += 1;
                            provider_instance.metadata.insert("probe_success".to_string(), "false".to_string());
                            provider_instance.metadata.insert("probe_error".to_string(), e.to_string());
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Probe timeout for provider {} (instance: {})",
                                provider_name,
                                instance_id
                            );
                            stats.probe_failures += 1;
                            provider_instance.metadata.insert("probe_success".to_string(), "false".to_string());
                            provider_instance.metadata.insert("probe_error".to_string(), "timeout".to_string());
                        }
                    }
                    break;
                }
            }
        }
    });

    stats
}

/// Filters the scanner registry based on scan options.
fn filter_scanner_registry(
    registry: &ScannerRegistry,
    _options: &ScanOptions,
) -> Result<ScannerRegistry> {
    let filtered_registry = ScannerRegistry::new();

    let all_scanners = registry.list();

    // Always include all scanners - provider filtering should only apply to providers/plugins,
    // not to scanner selection. Scanners are responsible for finding keys across all sources
    // regardless of which providers are configured.
    for scanner_name in all_scanners {
        if let Some(scanner) = registry.get(&scanner_name) {
            filtered_registry.register(scanner)?;
        }
    }

    Ok(filtered_registry)
}

/// Filters the plugin registry based on scan options.
fn filter_registry(registry: &PluginRegistry, options: &ScanOptions) -> Result<PluginRegistry> {
    let filtered_registry = PluginRegistry::new();

    let all_plugins = registry.list();

    for plugin_name in all_plugins {
        // Check if we should include this plugin
        let should_include = options.only_providers.as_ref().map_or_else(
            || {
                options
                    .exclude_providers
                    .as_ref()
                    .is_none_or(|exclude_providers| !exclude_providers.contains(&plugin_name))
            },
            |only_providers| only_providers.contains(&plugin_name),
        );

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
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined from the system.
pub fn default_home_dir() -> Result<PathBuf> {
    dirs_next::home_dir()
        .ok_or_else(|| Error::ConfigError("Could not determine home directory".to_string()))
}

/// Utility function to check if a path is a configuration file.
#[must_use]
pub fn is_config_file(path: &std::path::Path) -> bool {
    path.extension().map_or_else(
        || {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            matches!(
                file_name.as_ref(),
                ".env" | ".envrc" | "config" | "settings" | "preferences"
            )
        },
        |ext| {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "json" | "yaml" | "yml" | "toml" | "ini" | "env" | "conf" | "config"
            )
        },
    )
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
        let filtered = filter_registry(&registry, &options).unwrap();
        assert!(!filtered.is_empty());

        // Test with exclude_providers
        let options = ScanOptions::new().with_exclude_providers(vec!["nonexistent".to_string()]);
        let filtered = filter_registry(&registry, &options).unwrap();
        assert!(!filtered.is_empty());
    }
}
