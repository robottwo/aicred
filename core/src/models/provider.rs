//! Provider model representing AI service providers.

use serde::{Deserialize, Serialize};

/// Authentication method supported by a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    /// API key authentication.
    ApiKey,
    /// OAuth 2.0 authentication.
    OAuth,
    /// Bearer token authentication.
    BearerToken,
    /// Custom authentication method.
    Custom(String),
}

/// Rate limit configuration for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per minute.
    pub requests_per_minute: Option<u32>,
    /// Requests per hour.
    pub requests_per_hour: Option<u32>,
    /// Requests per day.
    pub requests_per_day: Option<u32>,
    /// Custom rate limit description.
    pub description: Option<String>,
}

/// AI service provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Unique identifier for the provider.
    pub name: String,
    /// Type of provider (e.g., "openai", "anthropic", "google").
    pub provider_type: String,
    /// Base URL for the provider's API.
    pub base_url: String,
    /// Optional description of the provider.
    pub description: Option<String>,
    /// Optional documentation URL.
    pub documentation_url: Option<String>,
    /// Rate limit information.
    pub rate_limits: Option<RateLimit>,
    /// Supported authentication methods.
    pub authentication_methods: Option<Vec<AuthMethod>>,
}

impl Provider {
    /// Creates a new provider with required fields.
    pub fn new(name: String, provider_type: String, base_url: String) -> Self {
        Self {
            name,
            provider_type,
            base_url,
            description: None,
            documentation_url: None,
            rate_limits: None,
            authentication_methods: None,
        }
    }

    /// Sets the description for the provider.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the documentation URL for the provider.
    pub fn with_documentation_url(mut self, url: String) -> Self {
        self.documentation_url = Some(url);
        self
    }

    /// Sets the rate limits for the provider.
    pub fn with_rate_limits(mut self, rate_limits: RateLimit) -> Self {
        self.rate_limits = Some(rate_limits);
        self
    }

    /// Sets the authentication methods for the provider.
    pub fn with_auth_methods(mut self, methods: Vec<AuthMethod>) -> Self {
        self.authentication_methods = Some(methods);
        self
    }

    /// Validates the provider configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }
        if self.provider_type.is_empty() {
            return Err("Provider type cannot be empty".to_string());
        }
        if self.base_url.is_empty() {
            return Err("Base URL cannot be empty".to_string());
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err("Base URL must start with http:// or https://".to_string());
        }
        Ok(())
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider_type: String::new(),
            base_url: "https://api.example.com".to_string(),
            description: None,
            documentation_url: None,
            rate_limits: None,
            authentication_methods: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = Provider::new(
            "OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        assert_eq!(provider.name, "OpenAI");
        assert_eq!(provider.provider_type, "openai");
        assert_eq!(provider.base_url, "https://api.openai.com");
        assert!(provider.description.is_none());
    }

    #[test]
    fn test_provider_builder() {
        let provider = Provider::new(
            "Anthropic".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        )
        .with_description("Anthropic AI".to_string())
        .with_documentation_url("https://docs.anthropic.com".to_string());

        assert_eq!(provider.description, Some("Anthropic AI".to_string()));
        assert_eq!(
            provider.documentation_url,
            Some("https://docs.anthropic.com".to_string())
        );
    }

    #[test]
    fn test_provider_validation() {
        let valid_provider = Provider::new(
            "Valid".to_string(),
            "valid".to_string(),
            "https://api.valid.com".to_string(),
        );
        assert!(valid_provider.validate().is_ok());

        let invalid_provider = Provider::new(
            "".to_string(),
            "invalid".to_string(),
            "https://api.invalid.com".to_string(),
        );
        assert!(invalid_provider.validate().is_err());

        let invalid_url_provider = Provider::new(
            "Invalid URL".to_string(),
            "invalid".to_string(),
            "not-a-url".to_string(),
        );
        assert!(invalid_url_provider.validate().is_err());
    }
}
