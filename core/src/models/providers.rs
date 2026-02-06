#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_excessive_bools)]
//! Provider metadata and instance configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about an AI provider (e.g., `OpenAI`, Anthropic).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provider {
    /// Provider name (e.g., "openai", "anthropic")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Supported authentication methods
    pub auth_methods: Vec<AuthMethod>,
    /// Rate limiting information (if known)
    pub rate_limits: Option<RateLimit>,
    /// Default base URL for API endpoints
    pub base_url_default: String,
}

/// Authentication method for a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuthMethod {
    /// API key in header or query parameter
    ApiKey,
    /// Bearer token authentication
    BearerToken,
    /// OAuth 2.0 authentication
    OAuth,
    /// HTTP Basic authentication
    Basic {
        /// Username for basic auth
        username: String,
    },
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimit {
    /// Maximum requests per minute (if known)
    pub requests_per_minute: Option<u32>,
    /// Maximum requests per day (if known)
    pub requests_per_day: Option<u32>,
    /// Maximum tokens per minute (if known)
    pub tokens_per_minute: Option<u64>,
}

/// A configured instance of a provider with credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInstance {
    /// Unique identifier for this instance
    pub id: String,
    /// Provider type (matches Provider.name)
    pub provider_type: String,
    /// Base URL for API endpoint
    pub base_url: String,
    /// API key or authentication token
    pub api_key: String,
    /// List of available model IDs
    pub models: Vec<String>,
    /// Provider capabilities
    pub capabilities: Capabilities,
    /// Whether this instance is active (backward compatibility)
    #[serde(default = "default_active")]
    pub active: bool,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

const fn default_active() -> bool {
    true
}

impl ProviderInstance {
    /// Gets the API key for this instance (for backward compatibility).
    ///
    /// Returns `Some(&api_key)` if the key is non-empty, None otherwise.
    #[must_use]
    pub const fn get_api_key(&self) -> Option<&String> {
        if self.api_key.is_empty() {
            None
        } else {
            Some(&self.api_key)
        }
    }

    /// Checks if this instance has a non-empty API key.
    #[must_use]
    pub const fn has_non_empty_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Checks if this instance has an API key (backward compatibility).
    #[must_use]
    pub const fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Sets the API key for this instance (backward compatibility).
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    /// Adds a model ID to this instance (backward compatibility).
    pub fn add_model(&mut self, model_id: String) {
        if !self.models.contains(&model_id) {
            self.models.push(model_id);
        }
    }

    /// Gets the number of models (backward compatibility).
    #[must_use]
    pub const fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Creates a new `ProviderInstance` with all fields.
    #[must_use]
    pub fn new(
        id: String,
        provider_type: String,
        base_url: String,
        api_key: String,
        models: Vec<String>,
    ) -> Self {
        Self {
            id,
            provider_type,
            base_url,
            api_key,
            models,
            capabilities: Capabilities::default(),
            active: true,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new `ProviderInstance` without models (backward compatibility).
    ///
    /// This is for old tests that don't provide a models vec.
    #[must_use]
    pub fn new_without_models(
        id: String,
        provider_type: String,
        base_url: String,
        api_key: String,
    ) -> Self {
        Self::new(id, provider_type, base_url, api_key, Vec::new())
    }

    /// Validates the instance configuration (stub for backward compatibility).
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.provider_type.is_empty() {
            return Err(crate::error::Error::ValidationError(
                "Provider type cannot be empty".to_string(),
            ));
        }
        if self.base_url.is_empty() {
            return Err(crate::error::Error::ValidationError(
                "Base URL cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Builder: sets metadata (backward compatibility).
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Checks if a model exists in this instance (backward compatibility).
    ///
    /// Returns true if the model ID is in the models list.
    #[must_use]
    pub fn get_model(&self, model_id: &str) -> Option<&String> {
        self.models.iter().find(|&m| m == model_id)
    }
}

/// Capabilities of a provider instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    /// Supports chat/conversation endpoints
    pub chat: bool,
    /// Supports text completion endpoints
    pub completion: bool,
    /// Supports embedding generation
    pub embedding: bool,
    /// Supports image generation
    pub image_generation: bool,
    /// Supports function/tool calling
    pub function_calling: bool,
    /// Supports streaming responses
    pub streaming: bool,
}

/// Collection of provider instances (instances.yaml representation).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCollection {
    /// Map of instance ID to instance configuration
    #[serde(flatten)]
    pub instances: HashMap<String, ProviderInstance>,
}

impl ProviderCollection {
    /// Creates a new empty collection
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Adds an instance to the collection
    pub fn add(&mut self, id: String, instance: ProviderInstance) {
        self.instances.insert(id, instance);
    }

    /// Gets an instance by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ProviderInstance> {
        self.instances.get(id)
    }

    /// Removes an instance by ID
    pub fn remove(&mut self, id: &str) -> Option<ProviderInstance> {
        self.instances.remove(id)
    }

    /// Lists all instances
    #[must_use]
    pub fn list(&self) -> Vec<&ProviderInstance> {
        self.instances.values().collect()
    }

    /// Gets the number of instances
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Checks if the collection is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    // ==== Backward Compatibility Methods ====

    /// Gets an instance by ID (backward compat alias for `get`)
    #[must_use]
    pub fn get_instance(&self, id: &str) -> Option<&ProviderInstance> {
        self.get(id)
    }

    /// Gets a mutable instance by ID (backward compat)
    #[must_use]
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut ProviderInstance> {
        self.instances.get_mut(id)
    }

    /// Adds an instance (backward compat - returns Result for consistency with old API)
    ///
    /// # Errors
    /// Never returns an error (kept for API compatibility).
    pub fn add_instance(&mut self, instance: ProviderInstance) -> Result<(), String> {
        let id = instance.id.clone();
        self.add(id, instance);
        Ok(())
    }

    /// Gets all instances (backward compat alias for `list`)
    #[must_use]
    pub fn all_instances(&self) -> Vec<&ProviderInstance> {
        self.list()
    }

    /// Adds or replaces an instance (backward compat)
    pub fn add_or_replace_instance(&mut self, instance: ProviderInstance) {
        let id = instance.id.clone();
        self.add(id, instance);
    }

    /// Removes an instance by ID (backward compat)
    pub fn remove_instance(&mut self, id: &str) -> Option<ProviderInstance> {
        self.remove(id)
    }

    /// Gets all active instances (backward compat)
    #[must_use]
    pub fn active_instances(&self) -> Vec<&ProviderInstance> {
        self.instances.values().filter(|i| i.active).collect()
    }

    /// Gets instances by provider type (backward compat)
    #[must_use]
    pub fn instances_by_type(&self, provider_type: &str) -> Vec<&ProviderInstance> {
        self.instances
            .values()
            .filter(|i| i.provider_type == provider_type)
            .collect()
    }
}
