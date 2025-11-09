//! Error types for the aicred core library.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the core library.
#[derive(Error, Debug)]
pub enum Error {
    /// IO-related errors during file operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Errors when parsing configuration files.
    #[error("Parse error in {path}: {message}")]
    ParseError {
        /// The path of the file that failed to parse
        path: PathBuf,
        /// The error message describing what went wrong
        message: String,
    },

    /// Errors related to plugin operations.
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Security-related errors (e.g., invalid key formats).
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Errors when a required directory or file is not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Errors when validation fails.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Errors when serialization/deserialization fails.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// General configuration errors.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// API-related errors (e.g., authentication failures, rate limits).
    #[error("API error: {0}")]
    ApiError(String),

    /// HTTP request errors.
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Result type alias for the core library.
pub type Result<T> = std::result::Result<T, Error>;
