//! Plugin system for extensible provider support.

// Allow clippy lints for the plugins module
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::significant_drop_tightening)]

// Suppress deprecated warnings - these are intentional during transition from plugins to providers
#![allow(deprecated)]

use crate::error::{Error, Result};
use crate::models::{ModelMetadata, ProviderInstance};
use crate::providers::{
    anthropic::AnthropicPlugin, groq::GroqPlugin, huggingface::HuggingFacePlugin,
    litellm::LiteLLMPlugin, ollama::OllamaPlugin, openai::OpenAIPlugin,
    openrouter::OpenRouterPlugin,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Trait that all provider plugins must implement.
#[async_trait]
pub trait ProviderPlugin: Send + Sync {
    /// Returns the name of this plugin.
    fn name(&self) -> &str;

    /// Returns a confidence score for a potential key (0.0 to 1.0).
    fn confidence_score(&self, key: &str) -> f32;

    /// Validates that this plugin can handle the given file.
    fn can_handle_file(&self, _path: &Path) -> bool {
        // Default implementation - can be overridden
        true
    }

    /// Gets the provider type this plugin handles.
    fn provider_type(&self) -> &str {
        self.name()
    }

    /// Initializes the provider with instance-specific configuration.
    /// This method is called when a provider instance is created or updated.
    fn initialize_instance(&self, _instance: &ProviderInstance) -> Result<()> {
        // Default implementation - can be overridden by providers that need initialization
        Ok(())
    }

    /// Validates the provider instance configuration.
    /// Returns Ok(()) if valid, or an error message if invalid.
    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        // Default implementation - can be overridden for provider-specific validation
        if instance.base_url.is_empty() {
            return Err(Error::PluginError("Base URL cannot be empty".to_string()));
        }
        if !instance.base_url.starts_with("http://") && !instance.base_url.starts_with("https://") {
            return Err(Error::PluginError(
                "Base URL must start with http:// or https://".to_string(),
            ));
        }
        Ok(())
    }

    /// Gets the list of models available for this provider instance.
    /// Returns a vector of model IDs that this instance supports.
    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // Default implementation - returns the models configured in the instance
        Ok(instance.models.clone())
    }

    /// Gets the full model configuration with provider-specific overrides applied.
    /// This loads the base model from the models directory and merges it with
    /// provider-specific overrides from the instance metadata.
    fn get_model_with_overrides(
        &self,
        _instance: &ProviderInstance,
        model_id: &str,
        home_dir: &std::path::Path,
    ) -> Result<Option<crate::models::Model>> {
        use crate::models::Model;

        // Try to load the base model from the models directory
        let config_dir = home_dir.join(".config").join("aicred").join("models");

        let model_file_name = format!("{}.yaml", model_id.replace(['/', ':'], "-"));
        let model_file_path = config_dir.join(&model_file_name);

        if !model_file_path.exists() {
            return Ok(None);
        }

        // Load the base model
        let model_content = std::fs::read_to_string(&model_file_path).map_err(|e| {
            crate::error::Error::PluginError(format!("Failed to read model file: {e}"))
        })?;

        let model: Model = serde_yaml::from_str(&model_content).map_err(|e| {
            crate::error::Error::PluginError(format!("Failed to parse model file: {e}"))
        })?;

        // TODO: Re-implement provider-specific overrides for v0.2.0 metadata structure
        // The old model.metadata field was Option<HashMap>, but the new Model.metadata
        // is a ModelMetadata struct. Override logic needs to be updated.

        Ok(Some(model))
    }

    /// Checks if the provider instance has valid configuration for operation.
    /// This includes checking API keys, base URL accessibility, etc.
    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Default implementation - checks if instance has valid keys
        Ok(instance.has_non_empty_api_key())
    }

    /// Probes the provider API to fetch available models using the provided API key.
    /// This is called during scanning when no models are explicitly configured.
    ///
    /// # Arguments
    /// * `api_key` - The API key to use for authentication
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of model IDs available from the API
    /// * `Err(_)` - If the API call fails or is not supported
    ///
    /// # Default Implementation
    /// Returns an empty vector, indicating no API-based model discovery.
    /// Providers that support API-based model discovery should override this method.
    fn probe_models(&self, _api_key: &str) -> Result<Vec<String>> {
        // Default implementation - no API probing
        Ok(Vec::new())
    }

    /// Asynchronously probes the provider API to fetch detailed model metadata.
    ///
    /// This method queries the provider's API to retrieve comprehensive information
    /// about available models, including their capabilities, pricing, and architecture.
    /// It is designed for use in async contexts and provides richer information than
    /// the synchronous `probe_models` method.
    ///
    /// # Arguments
    /// * `api_key` - The API key to use for authentication with the provider
    /// * `base_url` - Optional custom base URL for the API endpoint. If None, uses the provider's default
    ///
    /// # Returns
    /// * `Ok(Vec<ModelMetadata>)` - List of models with detailed metadata
    /// * `Err(Error::ApiError)` - If authentication fails or the API returns an error
    /// * `Err(Error::HttpError)` - If the network request fails
    /// * `Err(Error::SerializationError)` - If the API response cannot be parsed
    ///
    /// # Default Implementation
    /// Returns an empty vector, indicating no async API-based model discovery.
    /// Providers that support async model probing should override this method.
    ///
    /// # Example
    /// ```ignore
    /// use aicred_core::plugins::ProviderPlugin;
    /// use aicred_core::models::ModelMetadata;
    ///
    /// async fn probe_provider(plugin: &dyn ProviderPlugin, api_key: &str) {
    ///     match plugin.probe_models_async(api_key, None).await {
    ///         Ok(models) => {
    ///             for model in models {
    ///                 println!("Found model: {} ({})", model.name, model.id);
    ///             }
    ///         }
    ///         Err(e) => eprintln!("Failed to probe models: {}", e),
    ///     }
    /// }
    /// ```
    async fn probe_models_async(
        &self,
        _api_key: &str,
        _base_url: Option<&str>,
    ) -> Result<Vec<ModelMetadata>> {
        // Default implementation - no async API probing
        Ok(Vec::new())
    }
}

/// Type alias for provider plugin registry (v0.2.0+ simplified API).
///
/// This is a simple `HashMap` that maps provider name to plugin.
/// For backward compatibility, the old `PluginRegistry` wrapper is still available.
pub type ProviderRegistry = HashMap<String, Arc<dyn ProviderPlugin>>;

/// Legacy registry wrapper (deprecated in v0.2.0, use `ProviderRegistry` `HashMap` directly).
///
/// This wrapper adds Arc<`RwLock`<>> around a `HashMap` for thread-safety,
/// but most use cases don't need the complexity. Prefer using `ProviderRegistry` directly.
#[deprecated(
    since = "0.2.0",
    note = "Use ProviderRegistry (HashMap) with helper functions instead"
)]
#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn ProviderPlugin>>>>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field(
                "plugins",
                &format!("<{} plugins>", self.plugins.read().unwrap().len()),
            )
            .finish()
    }
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new plugin.
    ///
    /// # Errors
    /// Returns an error if the plugin cannot be registered (e.g., already exists or lock acquisition fails).
    pub fn register(&self, plugin: Arc<dyn ProviderPlugin>) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on plugins".to_string())
        })?;

        let name = plugin.name().to_string();
        if plugins.contains_key(&name) {
            return Err(Error::PluginError(format!(
                "Plugin '{name}' is already registered"
            )));
        }

        plugins.insert(name, plugin);
        Ok(())
    }

    /// Gets a plugin by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn ProviderPlugin>> {
        self.plugins
            .read()
            .ok()
            .and_then(|plugins| plugins.get(name).cloned())
    }

    /// Lists all registered plugin names.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.plugins
            .read()
            .ok()
            .map(|plugins| plugins.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Removes a plugin by name.
    ///
    /// # Errors
    /// Returns an error if the write lock on plugins cannot be acquired.
    pub fn remove(&self, name: &str) -> Result<Option<Arc<dyn ProviderPlugin>>> {
        Ok(self
            .plugins
            .write()
            .map_err(|_| Error::PluginError("Failed to acquire write lock on plugins".to_string()))?
            .remove(name))
    }

    /// Clears all plugins.
    ///
    /// # Errors
    /// Returns an error if the write lock on plugins cannot be acquired.
    pub fn clear(&self) -> Result<()> {
        self.plugins
            .write()
            .map_err(|_| Error::PluginError("Failed to acquire write lock on plugins".to_string()))?
            .clear();
        Ok(())
    }

    /// Gets the number of registered plugins.
    #[must_use]
    pub fn len(&self) -> usize {
        self.plugins.read().ok().map_or(0, |plugins| plugins.len())
    }

    /// Checks if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets all plugins that can handle a specific file.
    #[must_use]
    pub fn get_plugins_for_file(&self, path: &Path) -> Vec<Arc<dyn ProviderPlugin>> {
        self.plugins
            .read()
            .ok()
            .map(|plugins| {
                plugins
                    .values()
                    .filter(|plugin| plugin.can_handle_file(path))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in plugin for common configuration file patterns.
pub struct CommonConfigPlugin;

impl ProviderPlugin for CommonConfigPlugin {
    fn name(&self) -> &'static str {
        "common-config"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Simple confidence scoring based on key characteristics
        let mut score: f32 = 0.3; // Base score (lowered to make simple keys score lower)

        // Length-based scoring
        if key.len() >= 20 {
            score += 0.2;
        }
        if key.len() >= 40 {
            score += 0.1;
        }

        // Character diversity scoring
        let has_uppercase = key.chars().any(char::is_uppercase);
        let has_lowercase = key.chars().any(char::is_lowercase);
        let has_digits = key.chars().any(|c| c.is_ascii_digit());
        let has_special = key.chars().any(|c| !c.is_alphanumeric());

        if has_uppercase && has_lowercase {
            score += 0.1;
        }
        if has_digits {
            score += 0.05;
        }
        if has_special {
            score += 0.05;
        }

        // Common key prefixes
        if key.starts_with("sk-") || key.starts_with("ak-") {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        // Check if this plugin should handle the file
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        file_name.ends_with(".env")
            || file_name.ends_with(".env.local")
            || file_name.ends_with(".json")
            || file_name.ends_with(".yaml")
            || file_name.ends_with(".yml")
            || file_name.ends_with(".toml")
            || file_name.ends_with(".ini")
    }
}

/// Registers all built-in provider plugins (legacy wrapper-based API).
#[deprecated(since = "0.2.0", note = "Use register_builtin_providers() instead")]
pub fn register_builtin_plugins(registry: &PluginRegistry) -> Result<()> {
    // Core AI provider plugins
    registry.register(Arc::new(OpenAIPlugin))?;
    registry.register(Arc::new(AnthropicPlugin))?;
    registry.register(Arc::new(GroqPlugin))?;
    registry.register(Arc::new(HuggingFacePlugin))?;
    registry.register(Arc::new(OllamaPlugin))?;
    registry.register(Arc::new(OpenRouterPlugin))?;

    // Framework and tool plugins
    registry.register(Arc::new(LiteLLMPlugin))?;

    // Common config plugin (should be registered last as fallback)
    registry.register(Arc::new(CommonConfigPlugin))?;

    Ok(())
}

// ============================================================================
// Simplified Plugin API (v0.2.0+) - Preferred
// ============================================================================

/// Register all built-in provider plugins (v0.2.0+ simplified API).
///
/// Returns a ready-to-use `HashMap` of provider plugins.
/// This is the preferred way to get a provider registry.
///
/// # Example
/// ```
/// use aicred_core::plugins::{register_builtin_providers, get_provider};
///
/// let registry = register_builtin_providers();
/// if let Some(plugin) = get_provider(&registry, "openai") {
///     println!("Found OpenAI plugin");
/// }
/// ```
#[must_use]
pub fn register_builtin_providers() -> ProviderRegistry {
    let mut registry = HashMap::new();

    // Helper macro to register plugins
    macro_rules! register {
        ($plugin:expr) => {{
            let plugin = Arc::new($plugin) as Arc<dyn ProviderPlugin>;
            let name = plugin.name().to_string();
            registry.insert(name, plugin);
        }};
    }

    // Core AI provider plugins
    register!(OpenAIPlugin);
    register!(AnthropicPlugin);
    register!(GroqPlugin);
    register!(HuggingFacePlugin);
    register!(OllamaPlugin);
    register!(OpenRouterPlugin);

    // Framework and tool plugins
    register!(LiteLLMPlugin);

    // Note: CommonConfigPlugin removed in v0.2.0 - not needed

    registry
}

/// Get a plugin from the registry by name (v0.2.0+ API).
#[inline]
#[must_use]
pub fn get_provider<'a>(
    registry: &'a ProviderRegistry,
    name: &str,
) -> Option<&'a dyn ProviderPlugin> {
    registry.get(name).map(|arc| &**arc)
}

/// List all provider names in the registry (v0.2.0+ API).
#[inline]
pub fn list_providers(registry: &ProviderRegistry) -> Vec<&str> {
    registry.keys().map(String::as_str).collect()
}

/// Get all plugins that can handle a specific file (v0.2.0+ API).
#[must_use]
pub fn get_providers_for_file(
    registry: &ProviderRegistry,
    path: &Path,
) -> Vec<Arc<dyn ProviderPlugin>> {
    registry
        .values()
        .filter(|plugin| plugin.can_handle_file(path))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(CommonConfigPlugin);

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(plugin).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let retrieved = registry.get("common-config");
        assert!(retrieved.is_some());

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"common-config".to_string()));

        registry.remove("common-config").unwrap();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(CommonConfigPlugin);

        registry.register(plugin.clone()).unwrap();
        let result = registry.register(plugin);
        assert!(result.is_err());
    }

    #[test]
    fn test_common_config_plugin() {
        let plugin = CommonConfigPlugin;

        assert_eq!(plugin.name(), "common-config");
        assert_eq!(plugin.provider_type(), "common-config");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = CommonConfigPlugin;

        // Test various key formats
        let score1 = plugin.confidence_score("sk-1234567890abcdef");
        assert!(score1 > 0.5);

        let score2 = plugin.confidence_score("simple-key");
        assert!(score2 < 0.5);

        let score3 = plugin.confidence_score("sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890");
        assert!(score3 > 0.8);
    }

    #[tokio::test]
    async fn test_default_probe_models_async() {
        let plugin = CommonConfigPlugin;
        let result = plugin.probe_models_async("test-key", None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
