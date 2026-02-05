//! Provider key model for representing individual API keys with comprehensive metadata.

use crate::models::credentials::Confidence;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the validation status of a provider key.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Not yet validated.
    Unknown,
    /// Successfully validated.
    Valid,
    /// Failed validation.
    Invalid,
    /// Key has expired.
    Expired,
    /// Key has been revoked.
    Revoked,
    /// Key is rate limited.
    RateLimited,
}

impl fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Valid => write!(f, "Valid"),
            Self::Invalid => write!(f, "Invalid"),
            Self::Expired => write!(f, "Expired"),
            Self::Revoked => write!(f, "Revoked"),
            Self::RateLimited => write!(f, "Rate Limited"),
        }
    }
}

/// Environment context for API keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Environment {
    /// Development environment.
    Development,
    /// Staging environment.
    Staging,
    /// Production environment.
    Production,
    /// Testing environment.
    Testing,
    /// Custom environment.
    Custom(String),
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Staging => write!(f, "staging"),
            Self::Production => write!(f, "production"),
            Self::Testing => write!(f, "testing"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// Represents a single API key with comprehensive metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderKey {
    /// Unique identifier for this key (e.g., "default", "staging", "production").
    pub id: String,

    /// The actual API key value (None when stored in config, populated at runtime).
    #[serde(rename = "api_key", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// When this key was discovered.
    pub discovered_at: DateTime<Utc>,

    /// Source file path where the key was found.
    pub source: String,

    /// Line number in the source file.
    pub line_number: Option<u32>,

    /// Confidence level of key detection (from `DiscoveredCredential`).
    pub confidence: Confidence,

    /// Environment context (dev/staging/prod).
    pub environment: Environment,

    /// When this key was last validated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated: Option<DateTime<Utc>>,

    /// Current validation status.
    pub validation_status: ValidationStatus,

    /// Additional key-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// When this key configuration was created.
    pub created_at: DateTime<Utc>,

    /// When this key configuration was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ProviderKey {
    /// Creates a new provider key with default values.
    #[must_use]
    pub fn new(
        id: String,
        source: String,
        confidence: Confidence,
        environment: Environment,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            value: None,
            discovered_at: now,
            source,
            line_number: None,
            confidence,
            environment,
            last_validated: None,
            validation_status: ValidationStatus::Unknown,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Sets the actual key value (for runtime population).
    #[must_use]
    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    /// Sets the line number where the key was found.
    #[must_use]
    pub const fn with_line_number(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Sets additional metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Updates the validation status.
    pub fn set_validation_status(&mut self, status: ValidationStatus) {
        self.validation_status = status;
        self.last_validated = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Checks if this key is considered valid and active.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(self.validation_status, ValidationStatus::Valid)
    }

    /// Gets a redacted version of the key value for display.
    #[must_use]
    pub fn redacted_value(&self) -> String {
        match &self.value {
            Some(value) => {
                if value.chars().count() <= 8 {
                    let prefix: String = value.chars().take(2).collect();
                    format!("{prefix}****")
                } else {
                    let last_chars: String = value
                        .chars()
                        .rev()
                        .take(4)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    format!("****{last_chars}")
                }
            }
            None => "****".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_key_creation() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );

        assert_eq!(key.id, "test-key");
        assert_eq!(key.source, "/test/path");
        assert_eq!(key.confidence, Confidence::High);
        assert_eq!(key.environment, Environment::Production);
        assert_eq!(key.validation_status, ValidationStatus::Unknown);
        assert!(key.value.is_none());
    }

    #[test]
    fn test_provider_key_with_value() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("sk-test123456789".to_string());

        assert_eq!(key.value, Some("sk-test123456789".to_string()));
    }

    #[test]
    fn test_redacted_value_short_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("abc".to_string());

        assert_eq!(key.redacted_value(), "ab****");
    }

    #[test]
    fn test_redacted_value_long_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("sk-test123456789abcdef".to_string());

        assert_eq!(key.redacted_value(), "****cdef");
    }

    #[test]
    fn test_validation_status() {
        let mut key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        );

        assert!(!key.is_valid());

        key.set_validation_status(ValidationStatus::Valid);
        assert!(key.is_valid());
        assert_eq!(key.validation_status, ValidationStatus::Valid);
        assert!(key.last_validated.is_some());
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(
            Environment::Custom("test-env".to_string()).to_string(),
            "test-env"
        );
    }

    #[test]
    fn test_validation_status_display() {
        assert_eq!(ValidationStatus::Valid.to_string(), "Valid");
        assert_eq!(ValidationStatus::Invalid.to_string(), "Invalid");
        assert_eq!(ValidationStatus::Expired.to_string(), "Expired");
    }

    #[test]
    fn test_redacted_value_unicode_short_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("αβγ".to_string()); // Greek letters, 3 chars but 6 bytes

        assert_eq!(key.redacted_value(), "αβ****");
    }

    #[test]
    fn test_redacted_value_unicode_long_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("sk-test-αβγδεζηθ".to_string()); // Mixed ASCII and Greek

        assert_eq!(key.redacted_value(), "****εζηθ");
    }

    #[test]
    fn test_redacted_value_emoji_short_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("🔑🔐".to_string()); // Emojis, 2 chars but 8 bytes

        assert_eq!(key.redacted_value(), "🔑🔐****");
    }

    #[test]
    fn test_redacted_value_emoji_long_key() {
        let key = ProviderKey::new(
            "test-key".to_string(),
            "/test/path".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("sk-test-🔑🔐🔒🔓".to_string()); // Mixed ASCII and emojis

        // The actual result shows the last 4 characters correctly
        let result = key.redacted_value();
        assert!(result.starts_with("****"));
        // Each emoji is 4 bytes but 1 character, so total length is 4 asterisks + 4 emojis
        // But the string length in Rust counts Unicode scalar values, so it's 8
        assert_eq!(result.chars().count(), 8); // 4 asterisks + 4 emoji characters
    }
}
