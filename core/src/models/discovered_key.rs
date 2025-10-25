//! DiscoveredKey model for representing found API keys with security features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Type of discovered key value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// API key.
    ApiKey,
    /// Access token.
    AccessToken,
    /// Secret key.
    SecretKey,
    /// Bearer token.
    BearerToken,
    /// Custom type.
    Custom(String),
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::ApiKey => write!(f, "API Key"),
            ValueType::AccessToken => write!(f, "Access Token"),
            ValueType::SecretKey => write!(f, "Secret Key"),
            ValueType::BearerToken => write!(f, "Bearer Token"),
            ValueType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Confidence level for a discovered key.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd)]
pub enum Confidence {
    /// Low confidence - might be a false positive.
    Low = 0,
    /// Medium confidence - likely a valid key.
    Medium = 1,
    /// High confidence - very likely a valid key.
    High = 2,
    /// Very high confidence - almost certainly a valid key.
    VeryHigh = 3,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::Low => write!(f, "Low"),
            Confidence::Medium => write!(f, "Medium"),
            Confidence::High => write!(f, "High"),
            Confidence::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// Represents a discovered API key or credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredKey {
    /// Provider this key belongs to.
    pub provider: String,
    /// Source file where the key was found.
    pub source: String,
    /// Type of key that was discovered.
    pub value_type: ValueType,
    /// Confidence level that this is a valid key.
    pub confidence: Confidence,
    /// SHA-256 hash of the actual key value.
    pub hash: String,
    /// When this key was discovered.
    pub discovered_at: DateTime<Utc>,
    /// Line number in the source file (if applicable).
    pub line_number: Option<u32>,
    /// Column number in the source file (if applicable).
    pub column_number: Option<u32>,
    /// Additional metadata.
    pub metadata: Option<serde_json::Value>,

    /// The actual key value (redacted by default).
    #[serde(skip_serializing)]
    full_value: Option<String>,
}

impl DiscoveredKey {
    /// Creates a new discovered key with security features.
    pub fn new(
        provider: String,
        source: String,
        value_type: ValueType,
        confidence: Confidence,
        full_value: String,
    ) -> Self {
        let hash = Self::hash_value(&full_value);
        let discovered_at = Utc::now();

        Self {
            provider,
            source,
            value_type,
            confidence,
            hash,
            discovered_at,
            line_number: None,
            column_number: None,
            metadata: None,
            full_value: Some(full_value),
        }
    }

    /// Creates a new discovered key without storing the full value.
    pub fn new_redacted(
        provider: String,
        source: String,
        value_type: ValueType,
        confidence: Confidence,
        full_value: &str,
    ) -> Self {
        let hash = Self::hash_value(full_value);
        let discovered_at = Utc::now();

        Self {
            provider,
            source,
            value_type,
            confidence,
            hash,
            discovered_at,
            line_number: None,
            column_number: None,
            metadata: None,
            full_value: None,
        }
    }

    /// Returns the redacted version of the key (last 4 characters visible).
    pub fn redacted_value(&self) -> String {
        if let Some(ref value) = self.full_value {
            if value.len() <= 8 {
                format!("{}****", &value[..value.len().min(2)])
            } else {
                // Use chars() to safely handle Unicode characters
                let last_chars: String = value
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                format!("****{}", last_chars)
            }
        } else {
            // For redacted keys, we can't show the last 4 chars since we don't store them
            // This is a limitation of the redacted approach for security
            "****".to_string()
        }
    }

    /// Returns the full value if available and explicitly requested.
    pub fn with_full_value(mut self, include: bool) -> Self {
        if !include {
            self.full_value = None;
        }
        self
    }

    /// Gets the full value if available.
    pub fn full_value(&self) -> Option<&str> {
        self.full_value.as_deref()
    }

    /// Sets the line and column numbers where the key was found.
    pub fn with_position(mut self, line: u32, column: u32) -> Self {
        self.line_number = Some(line);
        self.column_number = Some(column);
        self
    }

    /// Sets additional metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Calculates SHA-256 hash of a value.
    fn hash_value(value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Checks if this key matches another key by hash.
    pub fn matches_hash(&self, other_hash: &str) -> bool {
        self.hash == other_hash
    }

    /// Gets a short description of the key.
    pub fn description(&self) -> String {
        format!(
            "{} key for {} (confidence: {})",
            self.value_type, self.provider, self.confidence
        )
    }
}

impl fmt::Display for DiscoveredKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} key for {} ({} confidence, discovered at {})",
            self.value_type,
            self.provider,
            self.confidence,
            self.discovered_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_key_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let env_path = temp_dir.path().join(".env").to_string_lossy().to_string();
        let key = DiscoveredKey::new(
            "OpenAI".to_string(),
            env_path.clone(),
            ValueType::ApiKey,
            Confidence::High,
            "sk-1234567890abcdef".to_string(),
        );

        assert_eq!(key.provider, "OpenAI");
        assert_eq!(key.source, env_path);
        assert_eq!(key.value_type, ValueType::ApiKey);
        assert_eq!(key.confidence, Confidence::High);
        assert!(!key.hash.is_empty());
        assert!(key.full_value.is_some());
    }

    #[test]
    fn test_redacted_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir
            .path()
            .join("config.json")
            .to_string_lossy()
            .to_string();
        let key = DiscoveredKey::new_redacted(
            "Anthropic".to_string(),
            config_path,
            ValueType::ApiKey,
            Confidence::Medium,
            "sk-ant-1234567890abcdef",
        );

        assert_eq!(key.redacted_value(), "****");
        assert!(key.full_value.is_none());
    }

    #[test]
    fn test_short_key_redaction() {
        let key = DiscoveredKey::new(
            "Test".to_string(),
            "/test".to_string(),
            ValueType::ApiKey,
            Confidence::Low,
            "abc".to_string(),
        );

        assert_eq!(key.redacted_value(), "ab****");
    }

    #[test]
    fn test_hash_consistency() {
        let value = "test-key-123";
        let hash1 = DiscoveredKey::hash_value(value);
        let hash2 = DiscoveredKey::hash_value(value);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_with_full_value() {
        let key = DiscoveredKey::new(
            "Test".to_string(),
            "/test".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "secret-key".to_string(),
        );

        let redacted_key = key.with_full_value(false);
        assert!(redacted_key.full_value.is_none());

        let key2 = DiscoveredKey::new(
            "Test2".to_string(),
            "/test2".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "another-key".to_string(),
        );

        let full_key = key2.with_full_value(true);
        assert!(full_key.full_value.is_some());
    }

    #[test]
    fn test_position_setting() {
        let key = DiscoveredKey::new(
            "Test".to_string(),
            "/test".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "key".to_string(),
        )
        .with_position(42, 10);

        assert_eq!(key.line_number, Some(42));
        assert_eq!(key.column_number, Some(10));
    }

    #[test]
    fn test_description() {
        let key = DiscoveredKey::new(
            "OpenAI".to_string(),
            "/env".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "key".to_string(),
        );

        let desc = key.description();
        assert!(desc.contains("API Key"));
        assert!(desc.contains("OpenAI"));
        assert!(desc.contains("High"));
    }
}
