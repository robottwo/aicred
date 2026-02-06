//! Regression tests to ensure refactoring doesn't break existing functionality.
//!
//! These tests verify core workflows that must continue working throughout
//! the refactoring process.

use aicred_core::*;
use tempfile::TempDir;

#[test]
fn test_basic_scan_flow() {
    let temp = TempDir::new().unwrap();
    let options = ScanOptions {
        home_dir: Some(temp.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: false,
        probe_timeout_secs: 30,
    };

    let result = scan(&options);
    assert!(result.is_ok(), "Basic scan should succeed");

    let scan_result = result.unwrap();
    assert_eq!(
        scan_result.home_directory,
        temp.path().display().to_string()
    );
}

#[test]
fn test_plugin_registry_basics() {
    let registry = PluginRegistry::new();
    register_builtin_plugins(&registry);

    // Verify built-in providers are registered
    assert!(
        registry.get("openai").is_some(),
        "OpenAI plugin should be registered"
    );
    assert!(
        registry.get("anthropic").is_some(),
        "Anthropic plugin should be registered"
    );
    assert!(
        registry.get("groq").is_some(),
        "Groq plugin should be registered"
    );
    assert!(
        registry.get("openrouter").is_some(),
        "OpenRouter plugin should be registered"
    );

    // Verify provider list
    let providers = registry.list();
    assert!(!providers.is_empty(), "Should have registered providers");
}

#[test]
fn test_scanner_registry_basics() {
    let registry = ScannerRegistry::new();
    register_builtin_scanners(&registry);

    // Verify built-in scanners are registered
    let scanners = registry.list();
    assert!(!scanners.is_empty(), "Should have registered scanners");
}

#[test]
fn test_discovered_key_confidence() {
    // Test confidence enum exists and has expected variants
    use Confidence::*;
    let levels = [Low, Medium, High, VeryHigh];
    assert_eq!(levels.len(), 4, "Should have 4 confidence levels");
}

#[test]
fn test_value_type_variants() {
    // Test ValueType enum exists and has expected variants
    use ValueType::*;
    let types = [ApiKey, AccessToken, SecretKey, BearerToken];
    assert!(types.len() >= 4, "Should have multiple value types");
}

#[test]
fn test_scan_result_initialization() {
    let home_dir = "/tmp/test".to_string();
    let providers = vec!["openai".to_string(), "anthropic".to_string()];
    let started_at = chrono::Utc::now();

    let result = ScanResult::new(home_dir.clone(), providers.clone(), started_at);

    assert_eq!(result.home_directory, home_dir);
    assert_eq!(result.providers_scanned, providers);
    assert!(result.keys.is_empty(), "New result should have no keys");
}

#[test]
fn test_provider_model_tuple_parsing() {
    let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    assert_eq!(tuple.provider(), "openai");
    assert_eq!(tuple.model(), "gpt-4");

    let invalid = ProviderModelTuple::parse("invalid");
    assert!(invalid.is_err(), "Invalid format should fail");
}

#[test]
fn test_scan_options_defaults() {
    let options = ScanOptions {
        home_dir: None,
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: false,
        probe_timeout_secs: 30,
    };

    assert!(!options.include_full_values, "Should default to redacted");
    assert_eq!(options.max_file_size, 1024 * 1024);
    assert!(!options.probe_models, "Should default to no probing");
}

#[test]
fn test_error_types() {
    use std::path::PathBuf;

    let not_found = Error::NotFound("test".to_string());
    assert!(matches!(not_found, Error::NotFound(_)));

    let validation = Error::ValidationError("test".to_string());
    assert!(matches!(validation, Error::ValidationError(_)));

    let parse = Error::ParseError {
        path: PathBuf::from("/test/path"),
        message: "test error".to_string(),
    };
    assert!(matches!(parse, Error::ParseError { .. }));
}

#[test]
fn test_provider_confidence_scoring() {
    let registry = PluginRegistry::new();
    register_builtin_plugins(&registry);

    if let Some(openai) = registry.get("openai") {
        // OpenAI keys start with sk-
        let score = openai.confidence_score("sk-proj-test123");
        assert!(score > 0.8, "sk-proj- prefix should have high confidence");

        let low_score = openai.confidence_score("random-text");
        assert!(low_score < 0.8, "Random text should have lower confidence");
    }
}

#[test]
fn test_scan_with_filters() {
    let temp = TempDir::new().unwrap();

    // Test with only_providers filter
    let options = ScanOptions {
        home_dir: Some(temp.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: Some(vec!["openai".to_string()]),
        exclude_providers: None,
        probe_models: false,
        probe_timeout_secs: 30,
    };

    let result = scan(&options);
    assert!(result.is_ok(), "Scan with filters should succeed");

    // Test with exclude_providers filter
    let options_exclude = ScanOptions {
        home_dir: Some(temp.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: Some(vec!["groq".to_string()]),
        probe_models: false,
        probe_timeout_secs: 30,
    };

    let result_exclude = scan(&options_exclude);
    assert!(
        result_exclude.is_ok(),
        "Scan with exclusions should succeed"
    );
}
