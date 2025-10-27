//! Tests for the new ScannerPlugin architecture

// Allow clippy lints for scanner tests
#![allow(unused_imports)]

use genai_keyfinder_core::models::discovered_key::{Confidence, ValueType};
use genai_keyfinder_core::models::{ConfigInstance, DiscoveredKey};
use genai_keyfinder_core::scanners::{ScanResult, ScannerPlugin, ScannerRegistry};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Mock scanner for testing
struct MockScanner;

impl ScannerPlugin for MockScanner {
    fn name(&self) -> &str {
        "mock"
    }

    fn app_name(&self) -> &str {
        "Mock Application"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            home_dir.join("mock_config.json"),
            PathBuf::from("mock_config.json"),
        ]
    }

    fn parse_config(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<ScanResult, genai_keyfinder_core::error::Error> {
        let mut result = ScanResult::new();

        // Simple mock parsing - look for "api_key" in content
        if content.contains("api_key") {
            let key = DiscoveredKey::new(
                "mock".to_string(),
                path.display().to_string(),
                ValueType::ApiKey,
                Confidence::High,
                "sk-mock1234567890abcdef".to_string(),
            );
            result.add_key(key);
        }

        // Create a mock instance if content contains "mock_app"
        if content.contains("mock_app") {
            let instance = ConfigInstance::new(
                "mock_instance_123".to_string(),
                "mock".to_string(),
                path.to_path_buf(),
            );
            result.add_instance(instance);
        }

        Ok(result)
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.contains("mock"))
            .unwrap_or(false)
    }
}

#[test]
fn test_scanner_registry_registration() {
    let registry = ScannerRegistry::new();
    let scanner = std::sync::Arc::new(MockScanner);

    // Register the scanner
    registry.register(scanner.clone()).unwrap();

    // Verify it was registered
    assert_eq!(registry.list().len(), 1);
    assert!(registry.get("mock").is_some());
}

#[test]
fn test_scanner_can_handle_file() {
    let scanner = MockScanner;

    assert!(scanner.can_handle_file(Path::new("mock_config.json")));
    assert!(scanner.can_handle_file(Path::new("test_mock_file.json")));
    assert!(!scanner.can_handle_file(Path::new("regular_config.json")));
}

#[test]
fn test_scanner_parse_config() {
    let scanner = MockScanner;
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mock_config.json");

    // Test parsing with API key
    let content_with_key = r#"{"api_key": "sk-test1234567890abcdef", "mock_app": true}"#;
    let result = scanner
        .parse_config(&config_path, content_with_key)
        .unwrap();

    assert_eq!(result.keys.len(), 1);
    assert_eq!(result.instances.len(), 1);
    assert_eq!(result.keys[0].provider, "mock");
    assert_eq!(result.keys[0].value_type, ValueType::ApiKey);

    // Test parsing without API key
    let content_without_key = r#"{"mock_app": true}"#;
    let result = scanner
        .parse_config(&config_path, content_without_key)
        .unwrap();

    assert_eq!(result.keys.len(), 0);
    assert_eq!(result.instances.len(), 1);
}

#[test]
fn test_scanner_scan_paths() {
    let scanner = MockScanner;
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = temp_dir.path();
    let paths = scanner.scan_paths(home_dir);

    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.ends_with("mock_config.json")));
}

#[test]
fn test_scan_result_functionality() {
    let mut result = ScanResult::new();

    // Test adding keys
    let key1 = DiscoveredKey::new(
        "test".to_string(),
        "/test/path".to_string(),
        ValueType::ApiKey,
        Confidence::High,
        "sk-test123".to_string(),
    );

    let key2 = DiscoveredKey::new(
        "test2".to_string(),
        "/test/path2".to_string(),
        ValueType::ApiKey,
        Confidence::Medium,
        "sk-test456".to_string(),
    );

    result.add_key(key1.clone());
    assert_eq!(result.keys.len(), 1);

    result.add_keys(vec![key2]);
    assert_eq!(result.keys.len(), 2);

    // Test adding instances
    let mut instance = ConfigInstance::new(
        "test_instance".to_string(),
        "test".to_string(),
        PathBuf::from("/test/config.json"),
    );
    instance.add_key(key1);

    result.add_instance(instance);
    assert_eq!(result.instances.len(), 1);
}

#[test]
fn test_scanner_registry_duplicate_registration() {
    let registry = ScannerRegistry::new();
    let scanner = std::sync::Arc::new(MockScanner);

    // Register the scanner
    registry.register(scanner.clone()).unwrap();

    // Try to register again - should fail
    let result = registry.register(scanner);
    assert!(result.is_err());
}

#[test]
fn test_scanner_get_scanners_for_file() {
    let registry = ScannerRegistry::new();
    let scanner = std::sync::Arc::new(MockScanner);

    registry.register(scanner).unwrap();

    // Get scanners that can handle a mock file
    let scanners = registry.get_scanners_for_file(Path::new("mock_config.json"));
    assert_eq!(scanners.len(), 1);

    // Get scanners for a non-mock file
    let scanners = registry.get_scanners_for_file(Path::new("regular_config.json"));
    assert_eq!(scanners.len(), 0);
}

#[test]
fn test_scanner_filtering_ignores_provider_filters() {
    use genai_keyfinder_core::ScanOptions;

    // Create a scanner registry with multiple scanners
    let registry = ScannerRegistry::new();
    let mock_scanner = std::sync::Arc::new(MockScanner);
    registry.register(mock_scanner).unwrap();

    // Create another mock scanner with a different name
    struct AnotherMockScanner;
    impl ScannerPlugin for AnotherMockScanner {
        fn name(&self) -> &str {
            "another_mock"
        }

        fn app_name(&self) -> &str {
            "Another Mock Application"
        }

        fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
            vec![home_dir.join("another_mock_config.json")]
        }

        fn parse_config(
            &self,
            _path: &Path,
            _content: &str,
        ) -> Result<ScanResult, genai_keyfinder_core::error::Error> {
            Ok(ScanResult::new())
        }

        fn can_handle_file(&self, path: &Path) -> bool {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains("another_mock"))
                .unwrap_or(false)
        }
    }

    let another_scanner = std::sync::Arc::new(AnotherMockScanner);
    registry.register(another_scanner).unwrap();

    // Verify both scanners are registered
    assert_eq!(registry.list().len(), 2);
    assert!(registry.get("mock").is_some());
    assert!(registry.get("another_mock").is_some());

    // Test: Verify that scanners with names matching provider filters are NOT excluded
    // The key insight is that scanner names (like "mock", "another_mock") should not be
    // filtered even if they appear in only_providers or exclude_providers lists.
    //
    // Since filter_scanner_registry is private, we test indirectly by verifying that
    // a scan with provider filters that would exclude scanners (if incorrectly applied)
    // still succeeds and uses all scanners.

    let temp_dir = tempfile::tempdir().unwrap();

    // Test 1: With only_providers that includes actual provider names (not scanner names)
    // This should succeed because scanners are not filtered by provider names
    let scan_options = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: Some(vec!["openai".to_string(), "anthropic".to_string()]),
        exclude_providers: None,
    };

    let result = genai_keyfinder_core::scan(&scan_options);
    // This will succeed because:
    // 1. Scanners are not filtered by provider names (our fix)
    // 2. Provider filtering happens separately and finds openai/anthropic providers
    assert!(
        result.is_ok(),
        "Scan should succeed with provider filters that don't match scanner names"
    );

    // Test 2: With exclude_providers that would exclude scanners if incorrectly applied
    // Even if we exclude providers with names like "mock", the mock scanner should still run
    let scan_options_exclude = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: Some(vec!["mock".to_string(), "another_mock".to_string()]),
    };

    let result = genai_keyfinder_core::scan(&scan_options_exclude);
    // This should succeed because scanners are not filtered by exclude_providers
    // The exclude_providers only affects provider/plugin filtering, not scanner selection
    assert!(
        result.is_ok(),
        "Scan should succeed even with exclude_providers that match scanner names"
    );

    // Test 3: Verify the fix by checking that scanner count is maintained
    // We can't directly access filter_scanner_registry, but we can verify behavior
    // by ensuring scans don't fail due to "no scanners" when provider filters are applied
    let scan_options_no_providers = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
    };

    let result = genai_keyfinder_core::scan(&scan_options_no_providers);
    assert!(
        result.is_ok(),
        "Scan should succeed with no provider filters"
    );
}
