//! Credential discovery and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A credential discovered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCredential {
    /// Provider this credential belongs to
    pub provider: String,
    /// The credential value (redacted or full)
    pub value: CredentialValue,
    /// Confidence level for this discovery
    pub confidence: Confidence,
    /// SHA-256 hash of the credential value
    pub hash: String,
    /// Source file where credential was found
    pub source_file: String,
    /// Line number in source file (if applicable)
    pub source_line: Option<usize>,
    /// Column number in the source file (if applicable)
    pub column_number: Option<u32>,
    /// Environment where credential was discovered
    pub environment: Environment,
    /// When this credential was discovered
    pub discovered_at: DateTime<Utc>,
    /// Type of credential
    pub value_type: ValueType,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl DiscoveredCredential {
    /// Creates a new discovered credential with a full value
    #[must_use]
    pub fn new(
        provider: String,
        source_file: String,
        value_type: ValueType,
        confidence: Confidence,
        full_value: String,
    ) -> Self {
        let hash = Self::hash_value(&full_value);
        let discovered_at = Utc::now();

        Self {
            provider,
            value: CredentialValue::full(full_value),
            confidence,
            hash,
            source_file,
            source_line: None,
            column_number: None,
            environment: Environment::UserConfig,
            discovered_at,
            value_type,
            metadata: None,
        }
    }

    /// Creates a new discovered credential with a redacted value
    #[must_use]
    pub fn new_redacted(
        provider: String,
        source_file: String,
        value_type: ValueType,
        confidence: Confidence,
        full_value: &str,
    ) -> Self {
        let hash = Self::hash_value(full_value);
        let discovered_at = Utc::now();

        Self {
            provider,
            value: CredentialValue::redact(full_value),
            confidence,
            hash,
            source_file,
            source_line: None,
            column_number: None,
            environment: Environment::UserConfig,
            discovered_at,
            value_type,
            metadata: None,
        }
    }

    /// Returns the redacted version of the credential value
    #[must_use]
    pub fn redacted_value(&self) -> String {
        match &self.value {
            CredentialValue::Full(value) => {
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
            CredentialValue::Redacted { prefix, .. } => {
                format!("{prefix}****")
            }
        }
    }

    /// Gets the full value if available
    #[must_use]
    pub fn full_value(&self) -> Option<&str> {
        match &self.value {
            CredentialValue::Full(s) => Some(s),
            CredentialValue::Redacted { .. } => None,
        }
    }

    /// Sets whether to include the full value (converts between Full and Redacted)
    #[must_use]
    pub fn with_full_value(mut self, include: bool) -> Self {
        let current_value = self.full_value();
        match (include, current_value) {
            (true, Some(full)) => {
                // Already has full value
                self.value = CredentialValue::Full(full.to_string());
            }
            (true, None) => {
                // Don't have full value, can't include it - keep as redacted
                // This is a limitation of the redacted approach
            }
            (false, Some(full)) => {
                // Convert to redacted
                self.value = CredentialValue::redact(full);
            }
            (false, None) => {
                // Already redacted
            }
        }
        self
    }

    /// Returns whether this credential has a full value stored
    #[must_use]
    pub const fn has_full_value(&self) -> bool {
        matches!(self.value, CredentialValue::Full(_))
    }

    /// Sets the line and column numbers where the credential was found
    #[must_use]
    pub const fn with_position(mut self, line: usize, column: u32) -> Self {
        self.source_line = Some(line);
        self.column_number = Some(column);
        self
    }

    /// Sets the environment where the credential was discovered
    #[must_use]
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets additional metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Calculates SHA-256 hash of a value
    #[must_use]
    pub fn hash_value(value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Checks if this credential matches a hash
    #[must_use]
    pub fn matches_hash(&self, other_hash: &str) -> bool {
        self.hash == other_hash
    }

    /// Gets a short description of the credential
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "{} for {} (confidence: {})",
            self.value_type, self.provider, self.confidence
        )
    }

    /// Gets the source field name for backward compatibility with DiscoveredKey
    #[must_use]
    pub const fn source(&self) -> &String {
        &self.source_file
    }
}

impl std::fmt::Display for DiscoveredCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} for {} ({} confidence, discovered at {})",
            self.value_type,
            self.provider,
            self.confidence,
            self.discovered_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

/// Credential value (full or redacted for security).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialValue {
    /// Full credential value (use with caution)
    Full(String),
    /// Redacted credential with hash and prefix
    Redacted {
        /// SHA-256 hash of the full value
        sha256: String,
        /// First few characters (for identification)
        prefix: String
    },
}

impl CredentialValue {
    /// Creates a redacted credential value
    #[must_use]
    pub fn redact(key: &str) -> Self {
        let hash = Sha256::digest(key.as_bytes());
        let prefix = if key.len() >= 8 {
            key[..8].to_string()
        } else {
            key.to_string()
        };
        
        Self::Redacted {
            sha256: hex::encode(hash),
            prefix,
        }
    }
    
    /// Creates a full credential value
    #[must_use]
    pub const fn full(key: String) -> Self {
        Self::Full(key)
    }
    
    /// Checks if this is a redacted value
    #[must_use]
    pub const fn is_redacted(&self) -> bool {
        matches!(self, Self::Redacted { .. })
    }
    
    /// Gets the full value if available
    #[must_use]
    pub const fn as_full(&self) -> Option<&String> {
        match self {
            Self::Full(s) => Some(s),
            Self::Redacted { .. } => None,
        }
    }
}

/// Confidence level for discovered credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Confidence {
    /// Low confidence (<0.5)
    Low = 0,
    /// Medium confidence (0.5-0.7)
    Medium = 1,
    /// High confidence (0.7-0.9)
    High = 2,
    /// Very high confidence (>0.9)
    VeryHigh = 3,
}

impl From<f32> for Confidence {
    fn from(score: f32) -> Self {
        if score < 0.5 { Self::Low }
        else if score < 0.7 { Self::Medium }
        else if score < 0.9 { Self::High }
        else { Self::VeryHigh }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// Type of discovered value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// API key
    ApiKey,
    /// Access token
    AccessToken,
    /// Secret key
    SecretKey,
    /// Bearer token
    BearerToken,
    /// Model identifier
    ModelId,
    /// Base URL for API endpoint
    BaseUrl,
    /// Temperature parameter
    Temperature,
    /// Parallel tool calls setting
    ParallelToolCalls,
    /// HTTP headers
    Headers,
    /// Custom type
    Custom(String),
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey => write!(f, "API Key"),
            Self::AccessToken => write!(f, "Access Token"),
            Self::SecretKey => write!(f, "Secret Key"),
            Self::BearerToken => write!(f, "Bearer Token"),
            Self::ModelId => write!(f, "Model ID"),
            Self::BaseUrl => write!(f, "Base URL"),
            Self::Temperature => write!(f, "Temperature"),
            Self::ParallelToolCalls => write!(f, "Parallel Tool Calls"),
            Self::Headers => write!(f, "Headers"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// Environment where credential was discovered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Environment {
    /// System-wide configuration
    SystemConfig,
    /// User-specific configuration
    UserConfig,
    /// Project-specific configuration
    ProjectConfig {
        /// Path to the project
        project_path: String
    },
    /// Environment variable
    EnvironmentVariable,
    /// Production environment (backward compatibility)
    Production,
}

/// Validation status for a credential.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Not yet validated
    NotValidated,
    /// Validated and confirmed working
    Valid,
    /// Validated but invalid/expired
    Invalid {
        /// Reason for invalidity
        reason: String
    },
    /// Rate limited during validation
    RateLimited,
    /// Network error during validation
    NetworkError,
}
