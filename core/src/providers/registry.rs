//! Provider registry for managing provider plugins and metadata.

use crate::error::Result;
use crate::plugins::{PluginRegistry, ProviderPlugin};
use crate::providers::metadata::ProviderMetadata;
use std::sync::Arc;

/// Main provider registry combining plugins and metadata.
pub struct ProviderRegistry {
    plugins: PluginRegistry,
    metadata: std::collections::HashMap<String, ProviderMetadata>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    /// Creates a new provider registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: PluginRegistry::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Registers a provider plugin and its metadata.
    pub fn register(
        &mut self,
        plugin: Arc<dyn ProviderPlugin>,
        metadata: ProviderMetadata,
    ) -> Result<()> {
        // Register the plugin
        self.plugins.register(plugin.clone())?;

        // Register the metadata
        let id = metadata.id.clone();
        self.metadata.insert(id, metadata);

        Ok(())
    }

    /// Gets provider metadata by ID.
    #[must_use]
    pub fn get_metadata(&self, id: &str) -> Option<&ProviderMetadata> {
        self.metadata.get(id)
    }

    /// Lists all registered provider IDs.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
    }

    /// Returns the number of registered providers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Checks if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
