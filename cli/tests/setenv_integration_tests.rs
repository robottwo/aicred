use aicred_core::{models::ProviderInstance, EnvResolverBuilder, ProviderModelTuple, UnifiedLabel};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_setenv_command_bash_format() {
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

    let env_resolver = EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .build();

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Verify bash format output
    let mut bash_output = String::new();
    for (key, value) in &resolution_result.variables {
        bash_output.push_str(&format!("export {}=\"{}\"\n", key, value));
    }

    // Should contain export statements
    assert!(bash_output.contains("export GSH_FAST_MODEL=\"openai:gpt-4\""));
    assert!(bash_output.contains("export GSH_FAST_API_KEY=\"test-openai-key\""));
    assert!(bash_output.contains("export GSH_FAST_BASE_URL=\"https://api.openai.com/v1\""));
}

#[test]
fn test_setenv_command_fish_format() {
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

    let result = env_resolver.resolve(false);

    assert!(result.is_ok());
    let resolution_result = result.unwrap();

    // Verify fish format output
    let mut fish_output = String::new();
    for (key, value) in &resolution_result.variables {
        fish_output.push_str(&format!("set -x {} \"{}\"\n", key, value));
    }

    // Should contain set statements
    assert!(fish_output.contains("set -x ROO_CODE_SMART_MODEL \"anthropic:claude-3-opus\""));
    assert!(fish_output.contains("set -x ROO_CODE_SMART_API_KEY \"test-anthropic-key\""));
    assert!(fish_output.contains("set -x ROO_CODE_SMART_BASE_URL \"https://api.anthropic.com\""));
}

#[test]
fn test_setenv_command_powershell_format() {
    let mut instance = ProviderInstance::new(
        "test-instance-3".to_string(),
        "Test Instance 3".to_string(),
        "groq".to_string(),
        "https://api.groq.com/openai/v1".to_string(),
    );
    instance.set_api_key("test-groq-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new(
        "llama3-70b-8192".to_string(),
        "llama3-70b-8192".to_string(),
    );
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("groq:llama3-70b-8192").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "local".to_string(),
        target: target.clone(),
        description: Some("Local model label".to_string()),
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

    // Verify PowerShell format output
    let mut ps_output = String::new();
    for (key, value) in &resolution_result.variables {
        ps_output.push_str(&format!("$env:{} = \"{}\"\n", key, value));
    }

    // Should contain $env: statements
    assert!(ps_output.contains("$env:GSH_LOCAL_MODEL = \"groq:llama3-70b-8192\""));
    assert!(ps_output.contains("$env:GSH_LOCAL_API_KEY = \"test-groq-key\""));
    assert!(ps_output.contains("$env:GSH_LOCAL_BASE_URL = \"https://api.groq.com/openai/v1\""));
}

#[test]
fn test_setenv_command_variable_resolution_accuracy() {
    let mut instance = ProviderInstance::new(
        "test-instance-accuracy".to_string(),
        "Test Instance Accuracy".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("sk-test-accuracy-key".to_string());
    // Add a model to the instance so it can match the target
    let model =
        aicred_core::models::Model::new("gpt-4-turbo".to_string(), "gpt-4-turbo".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4-turbo").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "accurate".to_string(),
        target: target.clone(),
        description: Some("Accurate model label".to_string()),
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

    // Verify variable resolution accuracy
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_ACCURATE_MODEL")
            .unwrap(),
        "openai:gpt-4-turbo"
    );
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_ACCURATE_API_KEY")
            .unwrap(),
        "sk-test-accuracy-key"
    );
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_ACCURATE_BASE_URL")
            .unwrap(),
        "https://api.openai.com/v1"
    );
}

#[test]
fn test_setenv_command_dry_run_mode() {
    let mut instance = ProviderInstance::new(
        "test-instance-dry".to_string(),
        "Test Instance Dry".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("sk-test-dry-key".to_string());
    // Add a model to the instance so it can match the target
    let model = aicred_core::models::Model::new("gpt-4".to_string(), "gpt-4".to_string());
    instance.add_model(model);
    let provider_instances = vec![instance];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "secret".to_string(),
        target: target.clone(),
        description: Some("Secret model label".to_string()),
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

    // In dry run mode, API keys should be masked
    if let Some(api_key) = resolution_result.variables.get("GSH_SECRET_API_KEY") {
        assert!(api_key.contains("***") || api_key == "<masked>");
    }

    // But model names should still be visible
    assert_eq!(
        resolution_result.variables.get("GSH_SECRET_MODEL").unwrap(),
        "openai:gpt-4"
    );
}

#[test]
fn test_setenv_command_multiple_scanners() {
    let mut instance1 = ProviderInstance::new(
        "test-instance-gsh".to_string(),
        "Test Instance GSH".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance1.set_api_key("test-gsh-key".to_string());
    // Add a model to the instance so it can match the target
    let model1 =
        aicred_core::models::Model::new("gpt-3.5-turbo".to_string(), "gpt-3.5-turbo".to_string());
    instance1.add_model(model1);

    let mut instance2 = ProviderInstance::new(
        "test-instance-roo".to_string(),
        "Test Instance Roo".to_string(),
        "anthropic".to_string(),
        "https://api.anthropic.com".to_string(),
    );
    instance2.set_api_key("test-roo-key".to_string());
    // Add a model to the instance so it can match the target (even though it's a different provider, we'll use the same target)
    let model2 =
        aicred_core::models::Model::new("gpt-3.5-turbo".to_string(), "gpt-3.5-turbo".to_string());
    instance2.add_model(model2);

    let provider_instances = vec![instance1, instance2];

    let target = ProviderModelTuple::parse("openai:gpt-3.5-turbo").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "turbo".to_string(),
        target: target.clone(),
        description: Some("Turbo model label".to_string()),
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

    // Should generate variables for both scanners
    assert!(resolution_result.variables.contains_key("GSH_TURBO_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_TURBO_API_KEY"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_TURBO_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("ROO_CODE_TURBO_API_KEY"));

    // Both scanners should use the same API key since they resolve to the same instance
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_TURBO_API_KEY")
            .unwrap(),
        "test-gsh-key"
    );
    assert_eq!(
        resolution_result
            .variables
            .get("ROO_CODE_TURBO_API_KEY")
            .unwrap(),
        "test-gsh-key"
    );
}

#[test]
fn test_setenv_command_custom_metadata_variables() {
    let mut metadata = HashMap::new();
    metadata.insert("temperature".to_string(), "0.8".to_string());
    metadata.insert("max_tokens".to_string(), "2000".to_string());
    metadata.insert("top_p".to_string(), "0.9".to_string());

    let mut instance = ProviderInstance::new(
        "test-instance-meta".to_string(),
        "Test Instance Meta".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-meta-key".to_string());
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

    // Should include metadata as environment variables
    assert!(resolution_result
        .variables
        .contains_key("GSH_CUSTOM_TEMPERATURE"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_CUSTOM_MAX_TOKENS"));

    // Verify values
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_CUSTOM_TEMPERATURE")
            .unwrap(),
        "0.8"
    );
    assert_eq!(
        resolution_result
            .variables
            .get("GSH_CUSTOM_MAX_TOKENS")
            .unwrap(),
        "2000"
    );
}

#[test]
fn test_setenv_command_error_handling() {
    // Test with missing provider instances
    let provider_instances = vec![];

    let target = ProviderModelTuple::parse("openai:gpt-4").unwrap();
    let labels = vec![UnifiedLabel {
        label_name: "missing".to_string(),
        target: target.clone(),
        description: Some("Missing model label".to_string()),
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

    // Should handle missing provider instances gracefully
    assert!(result.is_err() || !result.unwrap().missing_required.is_empty());
}

#[test]
fn test_setenv_command_empty_labels() {
    let mut instance = ProviderInstance::new(
        "test-instance-empty".to_string(),
        "Test Instance Empty".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-empty-key".to_string());
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
fn test_setenv_command_special_characters_in_labels() {
    // Test labels with special characters
    let mut instance = ProviderInstance::new(
        "test-instance-special".to_string(),
        "Test Instance Special".to_string(),
        "openai".to_string(),
        "https://api.openai.com/v1".to_string(),
    );
    instance.set_api_key("test-special-key".to_string());
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

    // Should handle special characters in label names
    assert!(resolution_result
        .variables
        .contains_key("GSH_FAST_MODEL_2024_MODEL"));
    assert!(resolution_result
        .variables
        .contains_key("GSH_SMART_AI_MODEL"));
}
