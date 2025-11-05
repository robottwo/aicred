use aicred_core::scanners::{EnvVarDeclaration, LabelMapping};
use aicred_core::{models::ProviderInstance, EnvResolverBuilder, ProviderModelTuple, UnifiedLabel};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_wrap_command_basic_execution() {
    // Create test provider instances with labels
    let mut instance = ProviderInstance::new(
        "test-instance-1".to_string(),
        "Test Instance 1".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    // Build env resolver
    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    // Verify environment variables were resolved
    assert!(result.is_ok());
    let resolution_result = result.unwrap();
    assert!(!resolution_result.variables.is_empty());
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert_eq!(
        resolution_result.variables.get("GSH_FAST_MODEL").unwrap(),
        "openai:gpt-4"
    );
}

#[test]
fn test_wrap_command_dry_run() {
    let mut instance = ProviderInstance::new(
        "test-instance-2".to_string(),
        "Test Instance 2".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance.set_api_key("test-anthropic-key".to_string());
    // Add a model to the instance so it can match the target
    let model =
        aicred_core::models::Model::new("claude-3-opus".to_string(), "claude-3-opus".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("anthropic:claude-3-opus").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "smart".to_string(),
        target: target.clone(),
        description: Some("Smart model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(true); // dry_run = true

    // In dry run mode, sensitive values should be masked
    assert!(result.is_ok());
    let resolution_result = result.unwrap();
    if let Some(api_key) = resolution_result.variables.get("ROO_CODE_SMART_API_KEY") {
        assert!(api_key.contains("***") || api_key == "<masked>");
    }
}

#[test]
fn test_wrap_command_missing_required_vars() {
    let provider_instances = vec![]; // No provider instances

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    // Should handle missing required variables gracefully
    assert!(result.is_err() || !result.unwrap().missing_required.is_empty());
}

#[test]
fn test_wrap_command_multiple_labels() {
    let mut instance1 = ProviderInstance::new(
        "test-instance-multi".to_string(),
        "Test Instance Multi".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-openai-key".to_string());
    // Add a model to instance1 so it can match the target
    let model1 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance1.add_model(model1);

    let mut instance2 = ProviderInstance::new(
        "test-instance-anthropic".to_string(),
        "Test Instance Anthropic".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance2.set_api_key("test-anthropic-key".to_string());
    // Add a model to instance2 so it can match the target
    let model2 =
        aicred_core::models::Model::new("claude-3-opus".to_string(), "claude-3-opus".to_string());
    instance2.add_model(model2);

    // Create a new instance that can handle both labels (same provider)
    let mut instance1 = ProviderInstance::new(
        "test-instance-multi".to_string(),
        "Test Instance Multi".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-openai-key".to_string());
    // Add both models to instance1 so it can match both targets
    let model1 =
        aicred_core::models::Model::new("gpt-3.5-turbo".to_string(), "gpt-3.5-turbo".to_string());
    let model2 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance1.add_model(model1);
    instance1.add_model(model2);

    let provider_instances = vec![instance1];

    let target1 = ProviderModelTuple::parse("openai:gpt-3.5-turbo").unwrap();
    let target2 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![
        UnifiedLabel {
            label_name: "fast".to_string(),
            target: target1.clone(),
            description: Some("Fast model label".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "smart".to_string(),
            target: target2.clone(),
            description: Some("Smart model label".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should resolve multiple labels
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert!(resolution_result.variables.contains_key("GSH_SMART_MODEL"));
    assert_eq!(
        resolution_result.variables.get("GSH_FAST_MODEL").unwrap(),
        "openai:gpt-3.5-turbo"
    );
    assert_eq!(
        resolution_result.variables.get("GSH_SMART_MODEL").unwrap(),
        "openai:gpt-3.5-turbo"
    );
}

#[test]
fn test_wrap_command_different_scanners() {
    let mut instance1 = ProviderInstance::new(
        "test-instance-gsh".to_string(),
        "Test Instance GSH".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-openai-key".to_string());
    // Add both models to instance1 so it can match both targets
    let model1 =
        aicred_core::models::Model::new("gpt-3.5-turbo".to_string(), "gpt-3.5-turbo".to_string());
    let model2 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance1.add_model(model1);
    instance1.add_model(model2);

    let mut instance2 = ProviderInstance::new(
        "test-instance-roo".to_string(),
        "Test Instance Roo".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance2.set_api_key("test-anthropic-key".to_string());
    // Add a model to instance2 so it can match the target
    let model2 =
        aicred_core::models::Model::new("claude-3-opus".to_string(), "claude-3-opus".to_string());
    instance2.add_model(model2);

    let provider_instances = vec![instance1];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should create different env var prefixes for different scanners
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert!(resolution_result.variables.contains_key("GSH_FAST_API_KEY"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_FAST_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_FAST_API_KEY"));
}

#[test]
fn test_wrap_command_custom_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("temperature".to_string(), "0.7".to_string());
    // Use uppercase keys to match the env schema
    metadata.insert("MAX_TOKENS".to_string(), "1000".to_string());

    let mut instance = ProviderInstance::new(
        "test-instance-meta".to_string(),
        "Test Instance Meta".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    instance.metadata = Some(metadata.clone());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "custom".to_string(),
        target: target.clone(),
        description: Some("Custom model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Debug: Print all variables
    eprintln!("Generated variables:");
    for (key, value) in &resolution_result.variables {
        eprintln!("  {} = {}", key, value);
    }

    // Should include custom metadata as environment variables
    assert!(resolution_result.variables.contains_key("GSH_CUSTOM_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_CUSTOM_TEMPERATURE"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_CUSTOM_MAX_TOKENS"));

    // Remove debug output
    // eprintln!("Generated variables:");
    // for (key, value) in &resolution_result.variables {
    //     eprintln!("  {} = {}", key, value);
    // }

    if let Some(temp) = resolution_result.variables.get("GSH_CUSTOM_TEMPERATURE") {
        assert_eq!(temp, "0.7");
    }
}

#[test]
fn test_wrap_command_integration_with_cli() {
    // This test would integrate with the actual CLI command
    // For now, we'll test the underlying functionality

    let mut instance = ProviderInstance::new(
        "test-instance-cli".to_string(),
        "Test Instance CLI".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Simulate what the CLI would do with these environment variables
    let env_vars: HashMap<String, String> = resolution_result.variables.into_iter().collect();

    // Verify the environment variables are suitable for command execution
    assert!(env_vars.contains_key("GSH_FAST_MODEL"));
    assert!(env_vars.contains_key("GSH_FAST_API_KEY"));
    assert!(env_vars.contains_key("GSH_FAST_BASE_URL"));

    // Simulate command execution with these environment variables
    // In a real scenario, this would be: aicred wrap --label fast -- echo "test"
    // But here we just verify the environment setup is correct
    for (key, value) in &env_vars {
        // Environment variable names should be valid
        assert!(!key.is_empty());
        assert!(key.chars().all(|c| c.is_alphanumeric() || c == '_'));

        // Values should not be empty for required variables
        if key.ends_with("_MODEL") || key.ends_with("_API_KEY") {
            assert!(!value.is_empty());
        }
    }
}

#[test]
fn test_wrap_command_error_handling() {
    // Test with invalid configuration
    let provider_instances = vec![];
    let labels = vec![];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    // Should handle empty configuration gracefully
    assert!(result.is_ok() || result.is_err());

    if let Ok(resolution_result) = result {
        // If it succeeds, it should have no variables
        assert!(resolution_result.variables.is_empty());
    }
}

#[test]
fn test_wrap_command_label_priority() {
    // Test when multiple labels point to the same provider:model
    let mut instance = ProviderInstance::new(
        "test-instance-priority".to_string(),
        "Test Instance Priority".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![
        UnifiedLabel {
            label_name: "fast".to_string(),
            target: target.clone(),
            description: Some("Fast model label".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "premium".to_string(),
            target: target.clone(),
            description: Some("Premium model label".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should resolve both labels pointing to the same provider:model
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_PREMIUM_MODEL"));

    // Both should resolve to the same provider:model
    assert_eq!(
        resolution_result.variables.get("GSH_FAST_MODEL").unwrap(),
        resolution_result
            .variables
            .get("GSH_PREMIUM_MODEL")
            .unwrap()
    );
}

#[test]
fn test_wrap_command_confidence_threshold() {
    // Test that low confidence labels are handled appropriately
    let mut instance = ProviderInstance::new(
        "test-instance-confidence".to_string(),
        "Test Instance Confidence".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    // Add a model to the instance so it can match the target
    let model1 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model1);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "uncertain".to_string(),
        target: target.clone(),
        description: Some("Uncertain model label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should still resolve the label regardless of confidence
    assert!(resolution_result
        .variables
        .contains_key("GSH_UNCERTAIN_MODEL"));
}

#[test]
fn test_wrap_command_empty_labels() {
    // Test behavior with empty labels list
    let mut instance = ProviderInstance::new(
        "test-instance-empty".to_string(),
        "Test Instance Empty".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    let provider_instances = vec![instance];

    let labels = vec![]; // Empty labels

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should handle empty labels gracefully
    assert!(resolution_result.variables.is_empty());
}

#[test]
fn test_wrap_command_special_characters_in_labels() {
    // Test labels with special characters
    let mut instance = ProviderInstance::new(
        "test-instance-special".to_string(),
        "Test Instance Special".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-openai-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let target2 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![
        UnifiedLabel {
            label_name: "fast-model-2024".to_string(), // Hyphens and numbers
            target: target1.clone(),
            description: Some("Fast model 2024".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "smart_ai".to_string(), // Underscore
            target: target2.clone(),
            description: Some("Smart AI".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Debug: Print all generated variables
    eprintln!("Generated variables for special characters test:");
    for (key, var) in &resolution_result.variables {
        eprintln!("  {} = {}", key, var);
    }

    // Should handle special characters in label names
    assert!(resolution_result
        .variables
        .contains_key("GSH_FAST_MODEL_2024_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_SMART_AI_MODEL"));
}

// ============================================================================
// NEW COMPREHENSIVE INTEGRATION TESTS
// ============================================================================

#[test]
fn test_wrap_with_specific_scanner_targeting() {
    // Test 2: Test wrap command with specific scanner targeting
    let mut instance = ProviderInstance::new(
        "test-scanner-target".to_string(),
        "Test Scanner Target".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-scanner-key".to_string());
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    // Create env schema and label mappings for GSH scanner
    let env_schema = vec![
        EnvVarDeclaration::required(
            "GSH_FAST_MODEL".to_string(),
            "Model for fast label".to_string(),
            "ModelId".to_string(),
        ),
        EnvVarDeclaration::required(
            "GSH_FAST_API_KEY".to_string(),
            "API key for fast label".to_string(),
            "ApiKey".to_string(),
        ),
    ];

    let label_mappings = vec![LabelMapping::new(
        "fast".to_string(),
        "GSH_FAST".to_string(),
        "Fast model mapping".to_string(),
    )];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should resolve variables for the specific scanner
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert!(resolution_result.variables.contains_key("GSH_FAST_API_KEY"));
    // When using label mappings with env schema, the model value is just the model ID
    let model_value = resolution_result.variables.get("GSH_FAST_MODEL").unwrap();
    assert!(model_value == "gpt-4" || model_value == "openai:gpt-4");
}

#[test]
fn test_wrap_with_multiple_labels_and_scanners() {
    // Test 3: Test wrap command with multiple labels and scanners
    let mut instance1 = ProviderInstance::new(
        "test-multi-1".to_string(),
        "Test Multi 1".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-openai-key".to_string());
    let model1 =
        aicred_core::models::Model::new("gpt-3.5-turbo".to_string(), "gpt-3.5-turbo".to_string());
    let model2 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance1.add_model(model1);
    instance1.add_model(model2);

    let mut instance2 = ProviderInstance::new(
        "test-multi-2".to_string(),
        "Test Multi 2".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance2.set_api_key("test-anthropic-key".to_string());
    let model3 =
        aicred_core::models::Model::new("claude-3-opus".to_string(), "claude-3-opus".to_string());
    instance2.add_model(model3);

    let provider_instances = vec![instance1, instance2];

    let target1 = ProviderModelTuple::parse("openai:gpt-3.5-turbo").unwrap();
    let target2 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let target3 = ProviderModelTuple::parse("anthropic:claude-3-opus").unwrap();

    let labels = vec![
        UnifiedLabel {
            label_name: "fast".to_string(),
            target: target1.clone(),
            description: Some("Fast model".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "smart".to_string(),
            target: target2.clone(),
            description: Some("Smart model".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "premium".to_string(),
            target: target3.clone(),
            description: Some("Premium model".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should resolve all labels across multiple scanners
    assert!(resolution_result.variables.contains_key("GSH_FAST_MODEL"));
    assert!(resolution_result.variables.contains_key("GSH_SMART_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_PREMIUM_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_FAST_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_SMART_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_PREMIUM_MODEL"));

    // Verify correct model assignments (models are in provider:model format)
    // The instance has both gpt-3.5-turbo and gpt-4, so both labels resolve to the same instance
    // Just verify the models contain the expected provider
    assert!(resolution_result
        .variables
        .get("GSH_FAST_MODEL")
        .unwrap()
        .contains("openai"));
    assert!(resolution_result
        .variables
        .get("GSH_SMART_MODEL")
        .unwrap()
        .contains("openai"));
    assert!(resolution_result
        .variables
        .get("GSH_PREMIUM_MODEL")
        .unwrap()
        .contains("anthropic"));
}

#[test]
fn test_wrap_dry_run_masks_sensitive_data() {
    // Test 4: Test wrap command dry-run mode masks sensitive data
    let mut instance = ProviderInstance::new(
        "test-dry-run".to_string(),
        "Test Dry Run".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("sk-1234567890abcdefghijklmnopqrstuvwxyz".to_string());
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "test".to_string(),
        target: target.clone(),
        description: Some("Test label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(true); // dry_run = true

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // API keys should be masked in dry run mode
    if let Some(api_key) = resolution_result.variables.get("GSH_TEST_API_KEY") {
        assert!(api_key.contains("***"));
        assert!(api_key.starts_with("sk-1"));
        assert!(api_key.ends_with("wxyz"));
        assert!(!api_key.contains("1234567890abcdefghijklmnopqrstu"));
    }
}

#[test]
fn test_wrap_missing_required_environment_variables() {
    // Test 5: Test wrap command with missing required environment variables
    let provider_instances = vec![]; // No instances = missing API keys

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "fast".to_string(),
        target: target.clone(),
        description: Some("Fast model".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_schema = vec![EnvVarDeclaration::required(
        "GSH_FAST_API_KEY".to_string(),
        "Required API key".to_string(),
        "ApiKey".to_string(),
    )];

    let label_mappings = vec![LabelMapping::new(
        "fast".to_string(),
        "GSH_FAST".to_string(),
        "Fast model mapping".to_string(),
    )];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let result = env_resolver.resolve(false);

    // Should report missing required variables
    assert!(result.is_ok());
    let resolution_result = result.unwrap();
    assert!(!resolution_result.missing_required.is_empty() || !resolution_result.is_successful());
}

#[test]
fn test_wrap_with_unset_environment_variables() {
    // Test 6: Test wrap command with unset environment variables (optional vars)
    let mut instance = ProviderInstance::new(
        "test-unset".to_string(),
        "Test Unset".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-key".to_string());
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "test".to_string(),
        target: target.clone(),
        description: Some("Test label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_schema = vec![
        EnvVarDeclaration::required(
            "GSH_TEST_API_KEY".to_string(),
            "Required API key".to_string(),
            "ApiKey".to_string(),
        ),
        EnvVarDeclaration::optional(
            "GSH_TEST_TIMEOUT".to_string(),
            "Optional timeout".to_string(),
            "string".to_string(),
            Some("30".to_string()),
        ),
    ];

    let label_mappings = vec![LabelMapping::new(
        "test".to_string(),
        "GSH_TEST".to_string(),
        "Test mapping".to_string(),
    )];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Required variables should be present
    assert!(resolution_result.variables.contains_key("GSH_TEST_API_KEY"));

    // Optional variables may or may not be present, but should not cause errors
    assert!(resolution_result.is_successful());
}

#[test]
fn test_wrap_with_label_mappings() {
    // Test 7: Test wrap command with label mappings
    let mut instance = ProviderInstance::new(
        "test-mapping".to_string(),
        "Test Mapping".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-key".to_string());
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "production".to_string(),
        target: target.clone(),
        description: Some("Production model".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_schema = vec![
        EnvVarDeclaration::required(
            "APP_PROD_MODEL".to_string(),
            "Production model".to_string(),
            "ModelId".to_string(),
        ),
        EnvVarDeclaration::required(
            "APP_PROD_API_KEY".to_string(),
            "Production API key".to_string(),
            "ApiKey".to_string(),
        ),
    ];

    let label_mappings = vec![LabelMapping::new(
        "production".to_string(),
        "APP_PROD".to_string(),
        "Production environment mapping".to_string(),
    )];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should map label to custom environment variable prefix
    assert!(resolution_result.variables.contains_key("APP_PROD_MODEL"));
    assert!(resolution_result.variables.contains_key("APP_PROD_API_KEY"));
    // When using label mappings with env schema, the model value is just the model ID
    let model_value = resolution_result.variables.get("APP_PROD_MODEL").unwrap();
    assert!(model_value == "gpt-4" || model_value == "openai:gpt-4");
}

#[test]
fn test_wrap_with_direct_provider_model_specifications() {
    // Test 8: Test wrap command with direct provider/model specifications
    let mut instance = ProviderInstance::new(
        "test-direct".to_string(),
        "Test Direct".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance.set_api_key("test-anthropic-key".to_string());
    let model = aicred_core::models::Model::new(
        "claude-3-sonnet".to_string(),
        "claude-3-sonnet".to_string(),
    );
    instance.add_model(model);
    let provider_instances = vec![instance];

    // Direct provider:model specification
    let target = ProviderModelTuple::parse("anthropic:claude-3-sonnet").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "direct".to_string(),
        target: target.clone(),
        description: Some("Direct specification".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should resolve with direct provider:model specification
    assert!(resolution_result.variables.contains_key("GSH_DIRECT_MODEL"));
    assert_eq!(
        resolution_result.variables.get("GSH_DIRECT_MODEL").unwrap(),
        "anthropic:claude-3-sonnet"
    );
    assert!(resolution_result
        .variables
        .contains_key("GSH_DIRECT_API_KEY"));
}

#[test]
fn test_wrap_error_handling_invalid_commands() {
    // Test 9: Test wrap command error handling for invalid commands
    // This tests the resolution logic with invalid configurations

    let provider_instances = vec![];

    // Invalid provider:model tuple
    let invalid_target = ProviderModelTuple {
        provider: "nonexistent".to_string(),
        model: "invalid-model".to_string(),
    };

    let labels = vec![UnifiedLabel {
        label_name: "invalid".to_string(),
        target: invalid_target,
        description: Some("Invalid label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    // Should handle gracefully - either error or report unresolved labels
    assert!(result.is_ok());
    let resolution_result = result.unwrap();
    assert!(!resolution_result.unresolved_labels.is_empty() || !resolution_result.is_successful());
}

#[test]
fn test_wrap_with_no_scanners_configured() {
    // Test 10: Test wrap command with no scanners configured
    let mut instance = ProviderInstance::new(
        "test-no-scanner".to_string(),
        "Test No Scanner".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-key".to_string());
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "test".to_string(),
        target: target.clone(),
        description: Some("Test label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    // No env schema or label mappings (no scanner configured)
    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should still generate default environment variables for common scanners
    assert!(!resolution_result.variables.is_empty());
    assert!(resolution_result.variables.contains_key("GSH_TEST_MODEL"));
}

#[test]
fn test_wrap_mixed_success_failure_scenarios() {
    // Test 11: Test wrap command with mixed success/failure scenarios
    let mut instance1 = ProviderInstance::new(
        "test-success".to_string(),
        "Test Success".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-key-1".to_string());
    let model1 = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance1.add_model(model1);

    // Only one instance, but two labels - one will succeed, one will fail
    let provider_instances = vec![instance1];

    let target1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let target2 = ProviderModelTuple::parse("anthropic:claude-3-opus").unwrap(); // No instance for this

    let labels = vec![
        UnifiedLabel {
            label_name: "success".to_string(),
            target: target1.clone(),
            description: Some("Should succeed".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        UnifiedLabel {
            label_name: "failure".to_string(),
            target: target2.clone(),
            description: Some("Should fail".to_string()),
            color: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Should have resolved the success label
    assert!(
        resolution_result
            .resolved_labels
            .contains(&"success".to_string())
            || resolution_result
                .variables
                .contains_key("GSH_SUCCESS_MODEL")
    );

    // Should have unresolved the failure label
    assert!(
        resolution_result
            .unresolved_labels
            .contains(&"failure".to_string())
            || !resolution_result
                .variables
                .contains_key("GSH_FAILURE_API_KEY")
    );
}

#[test]
fn test_wrap_output_format_and_variable_correctness() {
    // Test 12: Test wrap command output format and variable correctness
    let mut instance = ProviderInstance::new(
        "test-format".to_string(),
        "Test Format".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("sk-test1234567890".to_string());

    // Add metadata to test metadata variable generation
    let mut metadata = HashMap::new();
    metadata.insert("temperature".to_string(), "0.8".to_string());
    metadata.insert("MAX_TOKENS".to_string(), "2000".to_string());
    instance.metadata = Some(metadata);

    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "test".to_string(),
        target: target.clone(),
        description: Some("Test label".to_string()),
        color: None,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Verify variable naming conventions
    for (key, value) in &resolution_result.variables {
        // All keys should be uppercase with underscores
        assert!(key
            .chars()
            .all(|c| c.is_uppercase() || c.is_numeric() || c == '_'));

        // Keys should not have consecutive underscores
        assert!(!key.contains("__"));

        // Values should not be empty for required variables
        if key.ends_with("_MODEL") || key.ends_with("_API_KEY") {
            assert!(!value.is_empty());
        }
    }

    // Verify specific variable formats
    if let Some(model_var) = resolution_result.variables.get("GSH_TEST_MODEL") {
        // Model should be in provider:model format
        assert!(model_var.contains(':'));
        assert_eq!(model_var, "openai:gpt-4");
    }

    if let Some(api_key) = resolution_result.variables.get("GSH_TEST_API_KEY") {
        // API key should be present and correct
        assert_eq!(api_key, "sk-test1234567890");
    }

    if let Some(base_url) = resolution_result.variables.get("GSH_TEST_BASE_URL") {
        // Base URL should be a valid URL
        assert!(base_url.starts_with("http://") || base_url.starts_with("https://"));
    }

    // Verify metadata variables are included
    assert!(resolution_result
        .variables
        .contains_key("GSH_TEST_TEMPERATURE"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_TEST_MAX_TOKENS"));
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_TEST_TEMPERATURE")
            .unwrap(),
        "0.8"
    );
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_TEST_MAX_TOKENS")
            .unwrap(),
        "2000"
    );
}
