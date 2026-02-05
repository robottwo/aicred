//! Provider metadata system for managing provider information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Unique identifier for the provider (e.g., "openai", "anthropic")
    pub id: String,
    /// Human-readable name for the provider
    pub name: String,
    /// Brief description of the provider
    pub description: String,
    /// Base URL for the provider's API
    pub base_url: String,
    /// Whether this provider requires authentication
    pub requires_auth: bool,
    /// Key prefix pattern for this provider's API keys (if any)
    pub key_prefix: Option<String>,
    /// Expected key length for this provider's API keys (if applicable)
    pub key_length: Option<usize>,
    /// Supported model types by this provider
    pub model_types: Vec<String>,
    /// Example API key for documentation purposes (redacted)
    pub example_key: Option<String>,
    /// Default models for this provider
    pub default_models: Vec<String>,
}

impl ProviderMetadata {
    /// Creates a new provider metadata instance.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            base_url: base_url.into(),
            requires_auth: true,
            key_prefix: None,
            key_length: None,
            model_types: Vec::new(),
            example_key: None,
            default_models: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata_creation() {
        let metadata = ProviderMetadata::new(
            "test",
            "Test Provider",
            "A test provider",
            "https://api.test.com",
        );

        assert_eq!(metadata.id, "test");
        assert_eq!(metadata.name, "Test Provider");
        assert_eq!(metadata.description, "A test provider");
        assert_eq!(metadata.base_url, "https://api.test.com");
        assert!(metadata.requires_auth);
    }
}
