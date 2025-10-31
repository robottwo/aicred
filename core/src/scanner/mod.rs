//! Scanner module for discovering configuration files and API keys.

use crate::error::{Error, Result};
use crate::models::ScanResult;
use crate::plugins::PluginRegistry;
use crate::scanners::ScannerRegistry;
use chrono::Utc;
use std::path::Path;
use tracing::info;

/// Default maximum file size to scan (1MB).
pub const DEFAULT_MAX_FILE_SIZE: usize = 1024 * 1024;

/// Scanner configuration.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Maximum file size to scan in bytes.
    pub max_file_size: usize,
    /// Whether to follow symbolic links.
    pub follow_symlinks: bool,
    /// File extensions to include.
    pub include_extensions: Option<Vec<String>>,
    /// File extensions to exclude.
    pub exclude_extensions: Option<Vec<String>>,
    /// Specific files to exclude.
    pub exclude_files: Option<Vec<String>>,
    /// Whether to scan hidden files/directories.
    pub scan_hidden: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            follow_symlinks: false,
            include_extensions: None,
            exclude_extensions: Some(vec![".log".to_string(), ".tmp".to_string()]),
            exclude_files: Some(vec![".DS_Store".to_string(), "Thumbs.db".to_string()]),
            scan_hidden: true,
        }
    }
}

/// File scanner that discovers configuration files and extracts API keys.
pub struct Scanner {
    /// Provider plugin registry for key validation.
    provider_registry: PluginRegistry,
    /// Scanner plugin registry for discovering keys and configs.
    scanner_registry: Option<ScannerRegistry>,
}

impl Scanner {
    /// Creates a new scanner with the given provider registry.
    #[must_use] pub const fn new(provider_registry: PluginRegistry) -> Self {
        Self {
            provider_registry,
            scanner_registry: None,
        }
    }

    /// Sets the scanner registry for discovering keys and configs.
    #[must_use] pub fn with_scanner_registry(mut self, scanner_registry: ScannerRegistry) -> Self {
        self.scanner_registry = Some(scanner_registry);
        self
    }

    /// Scans the home directory for API keys using scanner-specific paths.
    /// This method no longer performs broad directory traversal - only scanner-specific scanning.
    pub fn scan(&self, home_dir: &Path) -> Result<ScanResult> {
        if !home_dir.exists() {
            return Err(Error::NotFound(format!(
                "Home directory does not exist: {}",
                home_dir.display()
            )));
        }

        if !home_dir.is_dir() {
            return Err(Error::ValidationError(format!(
                "Path is not a directory: {}",
                home_dir.display()
            )));
        }

        info!(
            "Starting targeted scan of configured scanners: {}",
            home_dir.display()
        );

        let scan_started_at = Utc::now();
        let providers = self.provider_registry.list();

        let mut result = ScanResult::new(
            home_dir.display().to_string(),
            providers,
            scan_started_at,
        );

        // Scanner-specific scanning is now handled by scan_with_scanners in lib.rs
        // This method just initializes the result structure for compatibility

        result.set_stats(0, 0);
        result.set_completed();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        assert_eq!(config.max_file_size, DEFAULT_MAX_FILE_SIZE);
        assert!(!config.follow_symlinks);
        assert!(config.scan_hidden);
    }

    #[test]
    fn test_scanner_creation() {
        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        assert!(scanner.scanner_registry.is_none());
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        let result = scanner.scan(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_file_instead_of_directory() {
        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = scanner.scan(temp_file.path());
        assert!(result.is_err());
    }
}
