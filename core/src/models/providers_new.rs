//! Provider metadata and instance configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about an AI provider (e.g., OpenAI, Anthropic).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        username: String
    },
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Capabilities of a provider instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
}
