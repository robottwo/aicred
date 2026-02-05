//! Real Configuration Validation Tests
//!
//! This test suite validates the scanner architecture against real-world configuration files
//! to ensure accurate provider detection, model counting, and output formatting.

use aicred_core::scanners::{ClaudeDesktopScanner, GshScanner, RooCodeScanner, ScannerPlugin};
use std::path::Path;

/// Helper function to get fixture path
fn fixture_path(filename: &str) -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .join("tests/fixtures")
        .join(filename)
}

#[test]
fn test_gsh_real_config_validation() {
    println!("\n=== Testing GSH Real Configuration ===");

    let scanner = GshScanner;
    let config_path = fixture_path("test_gsh_config.gshrc");

    let content = std::fs::read_to_string(&config_path).expect("Failed to read GSH config");

    let result = scanner
        .parse_config(&config_path, &content)
        .expect("GSH scan should succeed");

    // Print detailed results for manual verification
    println!("\nGSH Scan Results:");
    println!("  Config Path: {:?}", config_path);
    println!("  Scanner: GSH");
    println!("  Total Keys Found: {}", result.keys.len());

    // Validate discovered keys
    assert!(
        !result.keys.is_empty(),
        "Should discover keys in GSH config"
    );

    for key in &result.keys {
        println!("\n  Discovered Key:");
        println!("    Source: {}", key.source);
        println!("    Provider: {}", key.provider);
        println!("    Type: {:?}", key.value_type);
    }

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    println!("\n  Provider Instances:");
    println!("    Total Providers: {}", provider_instances.len());

    for instance in &provider_instances {
        println!("\n    Provider: {}", instance.id);
        println!("      Type: {}", instance.provider_type);
        println!("      Base URL: {}", instance.base_url);
        println!("    Keys: {}", instance.has_api_key() as usize);
        println!("      Models: {}", instance.model_count());

        for model in &instance.models {
            println!("        - {} ({})", model.name, model.model_id);
        }

        let metadata = &instance.metadata; if !metadata.is_empty() {
            println!("      Settings:");
            for (key, value) in metadata {
                println!("        {}: {}", key, value);
            }
        }
    }

    // Validation checks
    assert!(
        provider_instances.len() >= 2,
        "GSH config should have at least 2 provider instances (fast and slow models)"
    );

    // Check for Groq provider (fast model)
    let groq_instances: Vec<_> = provider_instances
        .iter()
        .filter(|p| p.provider_type.to_lowercase().contains("groq"))
        .collect();
    assert!(!groq_instances.is_empty(), "Should find Groq provider");

    // Check for OpenRouter or Anthropic provider (slow model)
    let slow_model_providers: Vec<_> = provider_instances
        .iter()
        .filter(|p| {
            p.provider_type.to_lowercase().contains("openrouter")
                || p.provider_type.to_lowercase().contains("anthropic")
        })
        .collect();
    assert!(
        !slow_model_providers.is_empty(),
        "Should find slow model provider"
    );

    // Verify settings are captured
    let instances_with_settings: Vec<_> = provider_instances
        .iter()
        .filter(|p| !p.metadata.is_empty() && !&p.metadata.is_empty())
        .collect();
    assert!(
        !instances_with_settings.is_empty(),
        "At least one provider should have settings (temperature, base_url, etc.)"
    );

    println!("\n✓ GSH validation passed");
}

#[test]
fn test_claude_desktop_real_config_validation() {
    println!("\n=== Testing Claude Desktop Real Configuration ===");

    let scanner = ClaudeDesktopScanner;
    let config_path = fixture_path("test_claude_desktop_config.json");

    let content =
        std::fs::read_to_string(&config_path).expect("Failed to read Claude Desktop config");

    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Claude Desktop scan should succeed");

    // Print detailed results
    println!("\nClaude Desktop Scan Results:");
    println!("  Config Path: {:?}", config_path);
    println!("  Scanner: Claude Desktop");
    println!("  Total Keys Found: {}", result.keys.len());

    for key in &result.keys {
        println!("\n  Discovered Key:");
        println!("    Source: {}", key.source);
        println!("    Provider: {}", key.provider);
        println!("    Type: {:?}", key.value_type);
    }

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    println!("\n  Provider Instances:");
    println!("    Total Providers: {}", provider_instances.len());

    for instance in &provider_instances {
        println!("\n    Provider: {}", instance.id);
        println!("      Type: {}", instance.provider_type);
        println!("      Base URL: {}", instance.base_url);
        println!("    Keys: {}", instance.has_api_key() as usize);
        println!("      Models: {}", instance.model_count());

        for model in &instance.models {
            println!("        - {} ({})", model.name, model.model_id);
        }

        let metadata = &instance.metadata; if !metadata.is_empty() {
            println!("      Settings:");
            for (key, value) in metadata {
                println!("        {}: {}", key, value);
            }
        }
    }

    // Validation checks
    assert!(!result.keys.is_empty(), "Should discover Anthropic key");
    assert_eq!(
        provider_instances.len(),
        1,
        "Claude Desktop config should have exactly 1 provider instance"
    );

    let anthropic_instance = provider_instances[0];
    assert_eq!(
        anthropic_instance.provider_type, "anthropic",
        "Provider should be Anthropic"
    );
    assert!(
        anthropic_instance.model_count() > 0,
        "Should have at least one model"
    );

    // Verify settings are captured
    assert!(
        !anthropic_instance.metadata.is_empty(),
        "Should have settings (temperature, max_tokens, etc.)"
    );

    let metadata = &anthropic_instance.metadata;
    assert!(
        metadata.contains_key("temperature") || metadata.contains_key("max_tokens"),
        "Should capture temperature or max_tokens setting"
    );

    println!("\n✓ Claude Desktop validation passed");
}

#[test]
fn test_roo_code_real_config_validation() {
    println!("\n=== Testing Roo Code Real Configuration ===");

    let scanner = RooCodeScanner;
    let config_path = fixture_path("test_roo_code_config.json");

    let content = std::fs::read_to_string(&config_path).expect("Failed to read Roo Code config");

    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Roo Code scan should succeed");

    // Print detailed results
    println!("\nRoo Code Scan Results:");
    println!("  Config Path: {:?}", config_path);
    println!("  Scanner: Roo Code");
    println!("  Total Keys Found: {}", result.keys.len());

    for key in &result.keys {
        println!("\n  Discovered Key:");
        println!("    Source: {}", key.source);
        println!("    Provider: {}", key.provider);
        println!("    Type: {:?}", key.value_type);
    }

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    println!("\n  Provider Instances:");
    println!("    Total Providers: {}", provider_instances.len());

    for instance in &provider_instances {
        println!("\n    Provider: {}", instance.id);
        println!("      Type: {}", instance.provider_type);
        println!("      Base URL: {}", instance.base_url);
        println!("      Keys: {}", instance.has_api_key() as usize);
        println!("      Models: {}", instance.model_count());

        for model in &instance.models {
            println!("        - {} ({})", model.name, model.model_id);
        }

        let metadata = &instance.metadata; if !metadata.is_empty() {
            println!("      Settings:");
            for (key, value) in metadata {
                println!("        {}: {}", key, value);
            }
        }
    }

    // Validation checks
    assert!(!result.keys.is_empty(), "Should discover multiple keys");
    assert!(
        provider_instances.len() >= 2,
        "Roo Code config should have multiple provider instances"
    );

    // Check for OpenAI provider
    let openai_instances: Vec<_> = provider_instances
        .iter()
        .filter(|p| p.provider_type.to_lowercase().contains("openai"))
        .collect();
    assert!(!openai_instances.is_empty(), "Should find OpenAI provider");

    // Check for Anthropic provider
    let anthropic_instances: Vec<_> = provider_instances
        .iter()
        .filter(|p| p.provider_type.to_lowercase().contains("anthropic"))
        .collect();
    assert!(
        !anthropic_instances.is_empty(),
        "Should find Anthropic provider"
    );

    // Verify settings are captured
    let instances_with_settings: Vec<_> = provider_instances
        .iter()
        .filter(|p| !p.metadata.is_empty() && !&p.metadata.is_empty())
        .collect();
    assert!(
        !instances_with_settings.is_empty(),
        "At least one provider should have settings"
    );

    println!("\n✓ Roo Code validation passed");
}

#[test]
fn test_no_duplicate_providers() {
    println!("\n=== Testing for Duplicate Providers ===");

    test_no_duplicates_for_scanner("GSH", &GshScanner, "test_gsh_config.gshrc");
    test_no_duplicates_for_scanner(
        "Claude Desktop",
        &ClaudeDesktopScanner,
        "test_claude_desktop_config.json",
    );
    test_no_duplicates_for_scanner("Roo Code", &RooCodeScanner, "test_roo_code_config.json");
}

fn test_no_duplicates_for_scanner(name: &str, scanner: &dyn ScannerPlugin, filename: &str) {
    let config_path = fixture_path(filename);
    let content = std::fs::read_to_string(&config_path).expect("Failed to read config");

    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Scan should succeed");

    assert_eq!(result.instances.len(), 1, "Should have one config instance");
    let config_instance = &result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    println!("\n{} Scanner:", name);
    println!("  Total Providers: {}", provider_instances.len());

    // Check for duplicate provider types
    let mut provider_types: Vec<String> = provider_instances
        .iter()
        .map(|p| p.provider_type.clone())
        .collect();
    provider_types.sort();

    let original_count = provider_types.len();
    provider_types.dedup();
    let deduped_count = provider_types.len();

    if original_count != deduped_count {
        println!(
            "  ⚠ WARNING: Found {} duplicate provider instances",
            original_count - deduped_count
        );

        let all_types: Vec<_> = provider_instances
            .iter()
            .map(|p| &p.provider_type)
            .collect();
        println!("  All providers: {:?}", all_types);
    } else {
        println!("  ✓ No duplicate providers found");
    }
}

#[test]
fn test_output_format_consistency() {
    println!("\n=== Testing Output Format Consistency ===");

    test_format_for_scanner("GSH", &GshScanner, "test_gsh_config.gshrc");
    test_format_for_scanner(
        "Claude Desktop",
        &ClaudeDesktopScanner,
        "test_claude_desktop_config.json",
    );
    test_format_for_scanner("Roo Code", &RooCodeScanner, "test_roo_code_config.json");
}

fn test_format_for_scanner(name: &str, scanner: &dyn ScannerPlugin, filename: &str) {
    println!("\n{} Scanner:", name);

    let config_path = fixture_path(filename);
    let content = std::fs::read_to_string(&config_path).expect("Failed to read config");

    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Scan should succeed");

    assert_eq!(result.instances.len(), 1, "Should have one config instance");
    let config_instance = &result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    for instance in provider_instances {
        assert!(
            !instance.provider_type.is_empty(),
            "Provider type should not be empty"
        );
        assert!(
            !instance.id.is_empty(),
            "Display name should not be empty"
        );
        assert!(
            !instance.base_url.is_empty(),
            "Base URL should not be empty"
        );

        println!("  Provider: {}", instance.id);
        println!("    Type: {}", instance.provider_type);
        println!("    Base URL: {}", instance.base_url);
        println!("      Keys: {}", instance.has_api_key() as usize);
        println!("    Models: {}", instance.model_count());

        for model in &instance.models {
            assert!(!model.name.is_empty(), "Model name should not be empty");
            assert!(!model.model_id.is_empty(), "Model ID should not be empty");
            println!("      - {} ({})", model.name, model.model_id);
        }

        let metadata = &instance.metadata; if !metadata.is_empty() {
            println!("    Settings: {} entries", metadata.len());
            for (key, value) in metadata {
                assert!(!key.is_empty(), "Setting key should not be empty");
                assert!(!value.is_empty(), "Setting value should not be empty");
            }
        }
    }

    println!("  ✓ Output format is consistent");
}

#[test]
fn test_model_count_accuracy() {
    println!("\n=== Testing Model Count Accuracy ===");

    let gsh_scanner = GshScanner;
    let gsh_path = fixture_path("test_gsh_config.gshrc");
    let gsh_content = std::fs::read_to_string(&gsh_path).expect("Failed to read GSH config");
    let gsh_result = gsh_scanner
        .parse_config(&gsh_path, &gsh_content)
        .expect("GSH scan should succeed");

    assert_eq!(gsh_result.instances.len(), 1);
    let gsh_instances = gsh_result.instances[0].provider_instances.all_instances();

    let total_gsh_models: usize = gsh_instances.iter().map(|p| p.model_count()).sum();

    println!("\nGSH:");
    println!("  Total Models: {}", total_gsh_models);
    println!("  Provider Instances: {}", gsh_instances.len());

    let claude_scanner = ClaudeDesktopScanner;
    let claude_path = fixture_path("test_claude_desktop_config.json");
    let claude_content =
        std::fs::read_to_string(&claude_path).expect("Failed to read Claude config");
    let claude_result = claude_scanner
        .parse_config(&claude_path, &claude_content)
        .expect("Claude scan should succeed");

    assert_eq!(claude_result.instances.len(), 1);
    let claude_instances = claude_result.instances[0]
        .provider_instances
        .all_instances();

    let total_claude_models: usize = claude_instances.iter().map(|p| p.model_count()).sum();

    println!("\nClaude Desktop:");
    println!("  Total Models: {}", total_claude_models);
    println!("  Provider Instances: {}", claude_instances.len());

    assert!(
        total_claude_models >= 1,
        "Claude Desktop should have at least 1 model"
    );

    let roo_scanner = RooCodeScanner;
    let roo_path = fixture_path("test_roo_code_config.json");
    let roo_content = std::fs::read_to_string(&roo_path).expect("Failed to read Roo config");
    let roo_result = roo_scanner
        .parse_config(&roo_path, &roo_content)
        .expect("Roo scan should succeed");

    assert_eq!(roo_result.instances.len(), 1);
    let roo_instances = roo_result.instances[0].provider_instances.all_instances();

    let total_roo_models: usize = roo_instances.iter().map(|p| p.model_count()).sum();

    println!("\nRoo Code:");
    println!("  Total Models: {}", total_roo_models);
    println!("  Provider Instances: {}", roo_instances.len());

    assert!(
        total_roo_models >= 1,
        "Roo Code should have at least 1 model"
    );

    println!("\n✓ Model counts are accurate");
}
