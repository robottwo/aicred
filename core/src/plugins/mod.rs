//! Plugin system for extensible provider support.

use crate::error::{Error, Result};
use crate::models::{DiscoveredKey, ProviderInstance};
use crate::providers::{
    anthropic::AnthropicPlugin, groq::GroqPlugin, huggingface::HuggingFacePlugin,
    litellm::LiteLLMPlugin, ollama::OllamaPlugin, openai::OpenAIPlugin,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Trait that all provider plugins must implement.
pub trait ProviderPlugin: Send + Sync {
    /// Returns the name of this plugin.
    fn name(&self) -> &str;

    /// Returns a confidence score for a potential key (0.0 to 1.0).
    fn confidence_score(&self, key: &str) -> f32;

    /// Validates that this plugin can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool {
        // Default implementation - can be overridden
        true
    }

    /// Gets the provider type this plugin handles.
    fn provider_type(&self) -> &str {
        self.name()
    }

    /// Initializes the provider with instance-specific configuration.
    /// This method is called when a provider instance is created or updated.
    fn initialize_instance(&self, instance: &ProviderInstance) -> Result<()> {
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
            return Err(Error::PluginError("Base URL must start with http:// or https://".to_string()));
        }
        Ok(())
    }

    /// Gets the list of models available for this provider instance.
    /// Returns a vector of model IDs that this instance supports.
    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        // Default implementation - returns the models configured in the instance
        Ok(instance.models.iter().map(|m| m.model_id.clone()).collect())
    }

    /// Checks if the provider instance has valid configuration for operation.
    /// This includes checking API keys, base URL accessibility, etc.
    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // Default implementation - checks if instance has valid keys
        Ok(instance.has_valid_keys())
    }
}

/// Registry for managing provider plugins.
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
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new plugin.
    pub fn register(&self, plugin: Arc<dyn ProviderPlugin>) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on plugins".to_string())
        })?;

        let name = plugin.name().to_string();
        if plugins.contains_key(&name) {
            return Err(Error::PluginError(format!(
                "Plugin '{}' is already registered",
                name
            )));
        }

        plugins.insert(name, plugin);
        Ok(())
    }

    /// Gets a plugin by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ProviderPlugin>> {
        self.plugins
            .read()
            .ok()
            .and_then(|plugins| plugins.get(name).cloned())
    }

    /// Lists all registered plugin names.
    pub fn list(&self) -> Vec<String> {
        self.plugins
            .read()
            .ok()
            .map(|plugins| plugins.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Removes a plugin by name.
    pub fn remove(&self, name: &str) -> Result<Option<Arc<dyn ProviderPlugin>>> {
        let mut plugins = self.plugins.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on plugins".to_string())
        })?;

        Ok(plugins.remove(name))
    }

    /// Clears all plugins.
    pub fn clear(&self) -> Result<()> {
        let mut plugins = self.plugins.write().map_err(|_| {
            Error::PluginError("Failed to acquire write lock on plugins".to_string())
        })?;

        plugins.clear();
        Ok(())
    }

    /// Gets the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins
            .read()
            .ok()
            .map(|plugins| plugins.len())
            .unwrap_or(0)
    }

    /// Checks if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets all plugins that can handle a specific file.
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
    fn name(&self) -> &str {
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
        let has_uppercase = key.chars().any(|c| c.is_uppercase());
        let has_lowercase = key.chars().any(|c| c.is_lowercase());
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

        score.min(1.0) as f32
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

/// Registers all built-in provider plugins.
pub fn register_builtin_plugins(registry: &PluginRegistry) -> Result<()> {
    // Core AI provider plugins
    registry.register(Arc::new(OpenAIPlugin))?;
    registry.register(Arc::new(AnthropicPlugin))?;
    registry.register(Arc::new(GroqPlugin))?;
    registry.register(Arc::new(HuggingFacePlugin))?;
    registry.register(Arc::new(OllamaPlugin))?;

    // Framework and tool plugins
    registry.register(Arc::new(LiteLLMPlugin))?;

    // Common config plugin (should be registered last as fallback)
    registry.register(Arc::new(CommonConfigPlugin))?;

    Ok(())
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

        registry.register(plugin.clone()).unwrap();
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
}
