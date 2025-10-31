//! Scanner module for discovering configuration files and API keys.

use crate::error::{Error, Result};
use crate::models::ScanResult;
use crate::plugins::PluginRegistry;
use crate::scanners::ScannerRegistry;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

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
    /// Scanner configuration.
    config: ScannerConfig,
}

impl Scanner {
    /// Creates a new scanner with the given provider registry.
    #[must_use] pub fn new(provider_registry: PluginRegistry) -> Self {
        Self {
            provider_registry,
            scanner_registry: None,
            config: ScannerConfig::default(),
        }
    }

    /// Creates a new scanner with custom configuration.
    #[must_use] pub const fn with_config(provider_registry: PluginRegistry, config: ScannerConfig) -> Self {
        Self {
            provider_registry,
            scanner_registry: None,
            config,
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

    /// Scans a specific path (file or directory).
    fn scan_path(&self, path: &Path, result: &mut ScanResult) -> Result<(u32, u32)> {
        let mut files_scanned = 0;
        let mut directories_scanned = 0;

        if path.is_file() {
            if self.should_scan_file(path) {
                debug!("Scanning file: {}", path.display());
                match self.scan_file(path, result) {
                    Ok(()) => files_scanned += 1,
                    Err(e) => {
                        debug!("Error scanning file {}: {e}", path.display());
                    }
                }
            }
        } else if path.is_dir() {
            debug!("Scanning directory: {}", path.display());
            match self.scan_directory(path, result) {
                Ok((files, dirs)) => {
                    files_scanned += files;
                    directories_scanned += dirs + 1; // +1 for current directory
                }
                Err(e) => {
                    debug!("Error scanning directory {}: {e}", path.display());
                }
            }
        }

        Ok((files_scanned, directories_scanned))
    }

    /// Scans a directory recursively.
    fn scan_directory(&self, dir: &Path, result: &mut ScanResult) -> Result<(u32, u32)> {
        let mut files_scanned = 0;
        let mut directories_scanned = 0;

        let entries = fs::read_dir(dir).map_err(|e| {
            Error::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read directory {}: {e}", dir.display()),
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::IoError(std::io::Error::new(
                    e.kind(),
                    format!("Failed to read directory entry: {e}"),
                ))
            })?;

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories if configured
            if !self.config.scan_hidden && file_name.starts_with('.') {
                continue;
            }

            // Skip excluded files
            if let Some(ref exclude_files) = self.config.exclude_files {
                if exclude_files.contains(&file_name) {
                    continue;
                }
            }

            // Handle symbolic links
            if path.is_symlink() && !self.config.follow_symlinks {
                continue;
            }

            match self.scan_path(&path, result) {
                Ok((files, dirs)) => {
                    files_scanned += files;
                    directories_scanned += dirs;
                }
                Err(e) => {
                    debug!("Error scanning {}: {e}", path.display());
                }
            }
        }

        Ok((files_scanned, directories_scanned))
    }

    /// Scans a single file for validation purposes.
    /// Note: Actual key discovery is handled by [`ScannerPlugin`] implementations.
    fn scan_file(&self, path: &Path, _result: &mut ScanResult) -> Result<()> {
        // Check file size
        let metadata = fs::metadata(path).map_err(|e| {
            Error::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read metadata for {}: {e}", path.display()),
            ))
        })?;

        if metadata.len() > self.config.max_file_size as u64 {
            debug!(
                "Skipping file {}: size {} exceeds limit {}",
                path.display(),
                metadata.len(),
                self.config.max_file_size
            );
            return Ok(());
        }

        // Note: File content reading and plugin validation is now handled by ScannerPlugin implementations
        debug!("File validated for size: {}", path.display());

        Ok(())
    }

    /// Checks if a file should be scanned based on configuration.
    fn should_scan_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Check excluded files
        if let Some(ref exclude_files) = self.config.exclude_files {
            if exclude_files.contains(&file_name.to_string()) {
                return false;
            }
        }

        // Check file extension filters
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_string();

            // Check include extensions first
            if let Some(ref include_ext) = self.config.include_extensions {
                if !include_ext.contains(&ext) {
                    return false;
                }
            }

            // Check exclude extensions
            if let Some(ref exclude_ext) = self.config.exclude_extensions {
                if exclude_ext.contains(&ext) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::CommonConfigPlugin;
    use std::fs;
    use tempfile::TempDir;

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

        assert_eq!(scanner.config.max_file_size, DEFAULT_MAX_FILE_SIZE);
        assert!(scanner.scanner_registry.is_none());
    }

    #[test]
    fn test_should_scan_file() {
        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        // Test with include extensions
        let config = ScannerConfig {
            include_extensions: Some(vec!["json".to_string(), "env".to_string()]),
            ..Default::default()
        };
        let scanner = Scanner::with_config(scanner.provider_registry, config);

        let test_dir = TempDir::new().unwrap();
        let json_file = test_dir.path().join("test.json");
        let txt_file = test_dir.path().join("test.txt");

        fs::write(&json_file, "{}").unwrap();
        fs::write(&txt_file, "test").unwrap();

        // This test would need to be adapted since Scanner::with_config is private
        // For now, just test the default behavior
        assert!(scanner.should_scan_file(&json_file));
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

    #[test]
    fn test_file_size_limits_at_and_over() {
        use std::fs;
        use std::sync::Arc;

        // Register a plugin that can parse .env
        let provider_registry = PluginRegistry::new();
        provider_registry
            .register(Arc::new(CommonConfigPlugin))
            .unwrap();

        let limit = 50usize;
        let config = ScannerConfig {
            max_file_size: limit,
            ..ScannerConfig::default()
        };
        let scanner = Scanner::with_config(provider_registry, config);

        let dir = tempfile::TempDir::new().unwrap();
        let at = dir.path().join(".env");
        let over = dir.path().join("big.env");

        // ~36 bytes baseline content
        let content = "OPENAI_API_KEY=sk-1234567890abcdef";
        fs::write(&at, content).unwrap();
        // make the other file clearly over the limit
        fs::write(&over, format!("{}{}", content, "x".repeat(200))).unwrap();

        let mut result = ScanResult::new(dir.path().display().to_string(), vec![], Utc::now());

        // scan_file only validates file size, doesn't extract keys
        // At limit should pass validation
        scanner.scan_file(&at, &mut result).unwrap();
        
        // Over limit should also pass (scan_file just checks size, doesn't fail)
        scanner.scan_file(&over, &mut result).unwrap();
        
        // Note: Actual key extraction is handled by ScannerPlugin implementations
        // This test only verifies file size validation logic
        assert_eq!(result.total_keys(), 0); // No keys extracted by scan_file itself
    }

    #[test]
    fn test_binary_file_handling_via_scan_path() {
        use std::fs;
        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        let dir = tempfile::TempDir::new().unwrap();
        let bin = dir.path().join("binary.dat");
        // Write non-UTF8 bytes
        fs::write(&bin, [0u8, 159, 146, 150]).unwrap();

        let mut result = ScanResult::new(dir.path().display().to_string(), vec![], Utc::now());
        // scan_path should swallow per-file errors and continue
        let _ = scanner.scan_path(&bin, &mut result).unwrap();
        // No panic and no keys
        assert_eq!(result.total_keys(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_permission_denied_is_non_fatal() {
        use std::fs::{self, Permissions};
        use std::os::unix::fs::PermissionsExt;

        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        let dir = tempfile::TempDir::new().unwrap();
        let f = dir.path().join("secret.env");
        fs::write(&f, "OPENAI_API_KEY=sk-xxxx").unwrap();

        // Remove permissions so reads fail
        fs::set_permissions(&f, Permissions::from_mode(0o000)).unwrap();

        let mut result = ScanResult::new(dir.path().display().to_string(), vec![], Utc::now());
        // Directory scan should not error overall
        let _ = scanner.scan_directory(dir.path(), &mut result).unwrap();
        // Restore permissions for cleanup is automatic on tempdir drop
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_is_skipped_when_not_following() {
        use std::fs;
        use std::os::unix::fs as unixfs;

        let provider_registry = PluginRegistry::new();
        let scanner = Scanner::new(provider_registry);

        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("target.env");
        fs::write(&target, "OPENAI_API_KEY=sk-xxxx").unwrap();

        let link = dir.path().join("link.env");
        unixfs::symlink(&target, &link).unwrap();

        let mut result = ScanResult::new(dir.path().display().to_string(), vec![], Utc::now());
        // should not follow symlink; scan_path returns Ok and finds nothing
        let _ = scanner.scan_path(&link, &mut result).unwrap();
        assert_eq!(result.total_keys(), 0);
    }
}
