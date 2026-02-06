//! Architecture validation tests for the new scanner architecture.
//!
//! These tests verify that the refactored scanner architecture correctly:
//! - Populates provider_instances from discovered keys
//! - Maintains accurate provider and model counts
//! - Prevents duplicate entries
//! - Correctly maps settings (temperature, base_url, etc.)
//! - Handles edge cases (missing keys, invalid configs, mixed providers)

use aicred_core::scanners::{ClaudeDesktopScanner, GshScanner, RooCodeScanner, ScannerPlugin};
use std::path::Path;

#[test]
fn test_gsh_scanner_architecture() {
    let scanner = GshScanner;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = Path::new(manifest_dir).join("tests/fixtures/test_gsh_config.gshrc");

    // Read the test configuration
    let content = std::fs::read_to_string(&config_path).expect("Failed to read test GSH config");

    // Parse the configuration
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse GSH config");

    println!("\n=== GSH Scanner Architecture Test ===");
    println!("Discovered keys: {}", result.keys.len());
    println!("Config instances: {}", result.instances.len());

    // Verify keys were discovered
    assert!(!result.keys.is_empty(), "Should discover API keys");

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    println!(
        "Provider instances: {}",
        config_instance.provider_instances.len()
    );

    // Verify provider_instances are populated
    assert!(
        !config_instance.provider_instances.is_empty(),
        "Config instance should have provider instances populated from discovered keys"
    );

    // Get all provider instances
    let provider_instances = config_instance.provider_instances.all_instances();
    println!("Total provider instances: {}", provider_instances.len());

    // Verify each provider instance has proper structure
    for (i, instance) in provider_instances.iter().enumerate() {
        println!("\nProvider Instance {}:", i + 1);
        println!("  Type: {}", instance.provider_type);
        println!("  Display Name: {}", instance.id);
        println!("  Key Count: {}", instance.has_api_key() as usize);
        println!("  Has Valid Keys: {}", instance.has_api_key());
        println!("  Model Count: {}", instance.model_count());
        println!("  Base URL: {}", instance.base_url);

        // Verify basic properties
        assert!(
            !instance.provider_type.is_empty(),
            "Provider type should not be empty"
        );
        assert!(!instance.id.is_empty(), "Display name should not be empty");
        assert!(
            !instance.base_url.is_empty(),
            "Base URL should not be empty"
        );
        assert!(
            instance.has_api_key(),
            "Provider instance should have at least one key"
        );
        // Note: has_api_key() may be false for test fixtures with placeholder keys
    }

    // Verify no duplicate provider instances and collect provider types
    let mut provider_types = std::collections::HashSet::new();
    let mut provider_type_list = Vec::new();
    for instance in provider_instances {
        provider_type_list.push(instance.provider_type.as_str());
        assert!(
            provider_types.insert(&instance.provider_type),
            "Found duplicate provider instance: {}",
            instance.provider_type
        );
    }

    println!("\nProvider types found: {:?}", provider_type_list);

    // Should have groq, openrouter, openai, anthropic, google, huggingface
    assert!(
        provider_type_list.contains(&"groq"),
        "Should have groq provider"
    );
    assert!(
        provider_type_list.contains(&"openrouter"),
        "Should have openrouter provider"
    );
    assert!(
        provider_type_list.contains(&"openai"),
        "Should have openai provider"
    );
    assert!(
        provider_type_list.contains(&"anthropic"),
        "Should have anthropic provider"
    );

    println!("\n✓ GSH Scanner architecture validation passed");
}

#[test]
fn test_claude_desktop_scanner_architecture() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let scanner = ClaudeDesktopScanner;
    let config_path =
        Path::new(manifest_dir).join("tests/fixtures/test_claude_desktop_config.json");

    // Read the test configuration
    let content =
        std::fs::read_to_string(&config_path).expect("Failed to read test Claude Desktop config");

    // Parse the configuration
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse Claude Desktop config");

    println!("\n=== Claude Desktop Scanner Architecture Test ===");
    println!("Discovered keys: {}", result.keys.len());
    println!("Config instances: {}", result.instances.len());

    // Verify keys were discovered (API key + model + temperature + max_tokens)
    assert_eq!(
        result.keys.len(),
        4,
        "Should discover 4 configuration values"
    );

    // Verify we have an API key
    let api_keys: Vec<_> = result
        .keys
        .iter()
        .filter(|k| matches!(k.value_type, aicred_core::models::ValueType::ApiKey))
        .collect();
    assert_eq!(api_keys.len(), 1, "Should have exactly one API key");
    assert_eq!(
        api_keys[0].provider, "anthropic",
        "API key should be for anthropic provider"
    );

    // Verify we have a model
    let models: Vec<_> = result
        .keys
        .iter()
        .filter(|k| matches!(k.value_type, aicred_core::models::ValueType::ModelId))
        .collect();
    assert_eq!(models.len(), 1, "Should have exactly one model");

    // Verify we have temperature
    let temps: Vec<_> = result
        .keys
        .iter()
        .filter(|k| matches!(k.value_type, aicred_core::models::ValueType::Temperature))
        .collect();
    assert_eq!(
        temps.len(),
        1,
        "Should have exactly one temperature setting"
    );

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    println!(
        "Provider instances: {}",
        config_instance.provider_instances.len()
    );

    // Verify provider_instances are populated
    assert_eq!(
        config_instance.provider_instances.len(),
        1,
        "Should have exactly one provider instance"
    );

    // Get the provider instance
    let provider_instances = config_instance.provider_instances.all_instances();
    assert_eq!(
        provider_instances.len(),
        1,
        "Should have one provider instance"
    );

    let provider_instance = provider_instances[0];
    println!("\nProvider Instance:");
    println!("  Type: {}", provider_instance.provider_type);
    println!("  Display Name: {}", provider_instance.id);
    println!("  Key Count: {}", provider_instance.has_api_key() as usize);
    println!("  Has Valid Keys: {}", provider_instance.has_api_key());

    // Verify provider instance properties
    assert_eq!(
        provider_instance.provider_type, "anthropic",
        "Should be anthropic provider"
    );
    assert_eq!(
        provider_instance.id, "anthropic",
        "Display name should match provider type"
    );
    assert_eq!(
        provider_instance.has_api_key() as usize,
        1,
        "Should have exactly one API key"
    );
    assert!(provider_instance.has_api_key(), "Should have valid keys");
    assert!(
        !provider_instance.base_url.is_empty(),
        "Should have a base URL"
    );

    // Verify model was added to provider instance
    assert_eq!(
        provider_instance.model_count(),
        1,
        "Should have exactly one model"
    );
    assert_eq!(
        provider_instance.models[0], "claude-3-opus-20240229",
        "Model ID should match the config"
    );

    // Verify temperature was added to provider instance metadata
    let metadata = &provider_instance.metadata;
    if !metadata.is_empty() {
        assert_eq!(
            metadata.get("temperature"),
            Some(&"0.7".to_string()),
            "Temperature should be in provider instance metadata"
        );
        assert_eq!(
            metadata.get("max_tokens"),
            Some(&"4096".to_string()),
            "Max tokens should be in provider instance metadata"
        );
    } else {
        panic!("Provider instance should have metadata");
    }

    // Verify metadata is preserved
    assert!(
        config_instance.metadata.contains_key("model"),
        "Should have model metadata"
    );
    assert!(
        config_instance.metadata.contains_key("temperature"),
        "Should have temperature metadata"
    );
    assert!(
        config_instance.metadata.contains_key("max_tokens"),
        "Should have max_tokens metadata"
    );

    println!("\nMetadata:");
    for (key, value) in &config_instance.metadata {
        println!("  {}: {}", key, value);
    }

    println!("\n✓ Claude Desktop Scanner architecture validation passed");
}

#[test]
fn test_roo_code_scanner_architecture() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let scanner = RooCodeScanner;
    let config_path = Path::new(manifest_dir).join("tests/fixtures/test_roo_code_config.json");

    // Read the test configuration
    let content =
        std::fs::read_to_string(&config_path).expect("Failed to read test Roo Code config");

    // Parse the configuration
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse Roo Code config");

    println!("\n=== Roo Code Scanner Architecture Test ===");
    println!("Discovered keys: {}", result.keys.len());
    println!("Config instances: {}", result.instances.len());

    // Verify keys were discovered
    assert!(!result.keys.is_empty(), "Should discover API keys");

    // Verify config instance was created
    assert_eq!(
        result.instances.len(),
        1,
        "Should create exactly one config instance"
    );

    let config_instance = &result.instances[0];
    println!(
        "Provider instances: {}",
        config_instance.provider_instances.len()
    );

    // Verify provider_instances are populated
    assert!(
        !config_instance.provider_instances.is_empty(),
        "Config instance should have provider instances"
    );

    // Get all provider instances
    let provider_instances = config_instance.provider_instances.all_instances();
    println!("Total provider instances: {}", provider_instances.len());

    // Verify each provider instance
    for (i, instance) in provider_instances.iter().enumerate() {
        println!("\nProvider Instance {}:", i + 1);
        println!("  Type: {}", instance.provider_type);
        println!("  Display Name: {}", instance.id);
        println!("  Key Count: {}", instance.has_api_key() as usize);
        println!("  Model Count: {}", instance.model_count());
        println!("  Has Valid Keys: {}", instance.has_api_key());
        println!("  Base URL: {}", instance.base_url);

        // Verify basic properties
        assert!(
            !instance.provider_type.is_empty(),
            "Provider type should not be empty"
        );
        assert!(!instance.id.is_empty(), "Display name should not be empty");
        assert!(
            !instance.base_url.is_empty(),
            "Base URL should not be empty"
        );
        assert!(
            instance.has_api_key(),
            "Provider instance should have at least one key"
        );
        // Note: has_api_key() may be false for test fixtures with placeholder keys
    }

    // Verify no duplicate provider instances
    let mut provider_types = std::collections::HashSet::new();
    for instance in provider_instances {
        assert!(
            provider_types.insert(&instance.provider_type),
            "Found duplicate provider instance: {}",
            instance.provider_type
        );
    }

    println!("\n✓ Roo Code Scanner architecture validation passed");
}

#[test]
fn test_gsh_missing_keys_edge_case() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let scanner = GshScanner;
    let config_path = Path::new(manifest_dir).join("tests/fixtures/test_gsh_missing_keys.gshrc");

    // Read the test configuration
    let content = std::fs::read_to_string(&config_path)
        .expect("Failed to read test GSH config with missing keys");

    // Parse the configuration
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse GSH config with missing keys");

    println!("\n=== GSH Missing Keys Edge Case Test ===");
    println!("Discovered keys: {}", result.keys.len());
    println!("Config instances: {}", result.instances.len());

    // Should still create a config instance even with missing keys
    assert_eq!(
        result.instances.len(),
        1,
        "Should create config instance even with missing keys"
    );

    let config_instance = &result.instances[0];
    println!(
        "Provider instances: {}",
        config_instance.provider_instances.len()
    );

    // May have some provider instances from non-empty keys
    let provider_instances = config_instance.provider_instances.all_instances();
    println!("Total provider instances: {}", provider_instances.len());

    // Verify that invalid/empty keys are not included
    for instance in provider_instances {
        println!("\nProvider Instance:");
        println!("  Type: {}", instance.provider_type);
        println!("  Key Count: {}", instance.has_api_key() as usize);
        println!("  Has Valid Keys: {}", instance.has_api_key());

        // All included instances should have at least one key
        assert!(
            instance.has_api_key(),
            "Provider instance should have at least one key"
        );
        assert!(
            !instance.base_url.is_empty(),
            "Provider instance should have a base URL"
        );
        // Note: has_valid_keys() may be false for test fixtures with placeholder keys
    }

    println!("\n✓ GSH missing keys edge case validation passed");
}

#[test]
fn test_invalid_config_edge_case() {
    let scanner = ClaudeDesktopScanner;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = Path::new(manifest_dir).join("tests/fixtures/test_invalid_config.json");

    // Read the test configuration
    let content = std::fs::read_to_string(&config_path).expect("Failed to read invalid config");

    // Parse the configuration
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse invalid config");

    println!("\n=== Invalid Config Edge Case Test ===");
    println!("Discovered keys: {}", result.keys.len());
    println!("Config instances: {}", result.instances.len());

    // Should not discover any keys
    assert_eq!(
        result.keys.len(),
        0,
        "Should not discover keys in invalid config"
    );

    // Should still create a config instance
    assert_eq!(
        result.instances.len(),
        1,
        "Should create config instance even for invalid config"
    );

    let config_instance = &result.instances[0];
    println!(
        "Provider instances: {}",
        config_instance.provider_instances.len()
    );

    // Should not have any provider instances (no valid keys)
    assert_eq!(
        config_instance.provider_instances.len(),
        0,
        "Should not have provider instances without valid keys"
    );

    println!("\n✓ Invalid config edge case validation passed");
}

#[test]
fn test_no_duplicate_keys_across_scanners() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("\n=== No Duplicate Keys Test ===");

    // Test GSH scanner
    let gsh_scanner = GshScanner;
    let gsh_config_path = Path::new(manifest_dir).join("tests/fixtures/test_gsh_config.gshrc");
    let gsh_content = std::fs::read_to_string(&gsh_config_path).expect("Failed to read GSH config");
    let gsh_result = gsh_scanner
        .parse_config(&gsh_config_path, &gsh_content)
        .expect("Failed to parse GSH config");

    // Collect all key hashes
    let mut all_hashes = std::collections::HashSet::new();
    let mut duplicate_count = 0;

    for key in &gsh_result.keys {
        if !all_hashes.insert(&key.hash) {
            duplicate_count += 1;
            println!(
                "Duplicate key found: provider={}, hash={}",
                key.provider, key.hash
            );
        }
    }

    println!("GSH Scanner:");
    println!("  Total keys: {}", gsh_result.keys.len());
    println!("  Unique keys: {}", all_hashes.len());
    println!("  Duplicates: {}", duplicate_count);

    assert_eq!(
        duplicate_count, 0,
        "GSH scanner should not produce duplicate keys"
    );

    println!("\n✓ No duplicate keys validation passed");
}

#[test]
fn test_provider_instance_counts() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("\n=== Provider Instance Counts Test ===");

    // Test GSH scanner
    let gsh_scanner = GshScanner;
    let gsh_config_path = Path::new(manifest_dir).join("tests/fixtures/test_gsh_config.gshrc");
    let gsh_content = std::fs::read_to_string(&gsh_config_path).expect("Failed to read GSH config");
    let gsh_result = gsh_scanner
        .parse_config(&gsh_config_path, &gsh_content)
        .expect("Failed to parse GSH config");

    let config_instance = &gsh_result.instances[0];
    let provider_instances = config_instance.provider_instances.all_instances();

    println!("GSH Config:");
    println!("  Total discovered keys: {}", gsh_result.keys.len());
    println!("  Total provider instances: {}", provider_instances.len());

    // Count API keys per provider (not all discovered keys, just API keys)
    let mut provider_api_key_counts = std::collections::HashMap::new();
    for key in &gsh_result.keys {
        if matches!(key.value_type, aicred_core::models::ValueType::ApiKey) {
            *provider_api_key_counts.entry(&key.provider).or_insert(0) += 1;
        }
    }

    println!("\nAPI keys per provider:");
    for (provider, count) in &provider_api_key_counts {
        println!("  {}: {}", provider, count);
    }

    println!("\nProvider instances:");
    for instance in provider_instances {
        println!(
            "  {}: {} keys, {} models",
            instance.provider_type,
            instance.has_api_key() as usize,
            instance.model_count()
        );

        // Verify key count matches API keys only (not all discovered keys)
        let expected_count = provider_api_key_counts
            .get(&instance.provider_type)
            .copied()
            .unwrap_or(0);
        assert_eq!(
            instance.has_api_key() as usize,
            expected_count,
            "Provider instance key count should match API key count for {}",
            instance.provider_type
        );

        // Verify provider instance has valid structure
        assert!(
            !instance.base_url.is_empty(),
            "Provider instance should have a base URL"
        );
        // Note: has_valid_keys() may be false for test fixtures with placeholder keys
    }

    println!("\n✓ Provider instance counts validation passed");
}

#[test]
fn test_settings_mapping() {
    println!("\n=== Settings Mapping Test ===");

    // Test Claude Desktop scanner with metadata
    let scanner = ClaudeDesktopScanner;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path =
        Path::new(manifest_dir).join("tests/fixtures/test_claude_desktop_config.json");
    let content =
        std::fs::read_to_string(&config_path).expect("Failed to read Claude Desktop config");
    let result = scanner
        .parse_config(&config_path, &content)
        .expect("Failed to parse Claude Desktop config");

    let config_instance = &result.instances[0];

    println!("Claude Desktop Config Metadata:");
    for (key, value) in &config_instance.metadata {
        println!("  {}: {}", key, value);
    }

    // Verify specific settings are mapped
    assert_eq!(
        config_instance.metadata.get("model"),
        Some(&"claude-3-opus-20240229".to_string()),
        "Model should be correctly mapped"
    );
    assert_eq!(
        config_instance.metadata.get("temperature"),
        Some(&"0.7".to_string()),
        "Temperature should be correctly mapped"
    );
    assert_eq!(
        config_instance.metadata.get("max_tokens"),
        Some(&"4096".to_string()),
        "Max tokens should be correctly mapped"
    );

    // Test GSH scanner with provider instance metadata
    println!("\n--- GSH Scanner Settings Mapping ---");
    let gsh_scanner = GshScanner;
    let gsh_config_path = Path::new(manifest_dir).join("tests/fixtures/test_gsh_config.gshrc");
    let gsh_content = std::fs::read_to_string(&gsh_config_path).expect("Failed to read GSH config");
    let gsh_result = gsh_scanner
        .parse_config(&gsh_config_path, &gsh_content)
        .expect("Failed to parse GSH config");

    let gsh_config_instance = &gsh_result.instances[0];
    let provider_instances = gsh_config_instance.provider_instances.all_instances();

    // Find the groq provider instance (fast model)
    let groq_instance = provider_instances
        .iter()
        .find(|i| i.provider_type == "groq")
        .expect("Should have groq provider instance");

    println!("\nGroq Provider Instance Metadata:");
    let metadata = &groq_instance.metadata;
    if !metadata.is_empty() {
        for (key, value) in metadata {
            println!("  {}: {}", key, value);
        }

        // Verify GSH-specific settings are mapped to provider instance metadata
        assert_eq!(
            metadata.get("temperature"),
            Some(&"0.7".to_string()),
            "Temperature should be mapped to groq provider instance metadata"
        );
        assert_eq!(
            metadata.get("parallel_tool_calls"),
            Some(&"true".to_string()),
            "Parallel tool calls should be mapped to groq provider instance metadata"
        );

        println!("\n✓ GSH settings correctly mapped to provider instance metadata");
    } else {
        panic!("Groq provider instance should have metadata");
    }

    // Verify the groq instance has the correct model
    assert_eq!(
        groq_instance.model_count(),
        1,
        "Groq instance should have 1 model"
    );
    assert_eq!(
        groq_instance.models[0], "llama3-70b-8192",
        "Model ID should be correctly mapped"
    );

    println!("\n✓ Settings mapping validation passed");
}
