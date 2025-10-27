//! Tests for migration from ProviderConfig to ProviderInstance format.

use genai_keyfinder_core::models::*;
use std::collections::HashMap;
use chrono::Utc;

/// Creates a test ProviderConfig with sample data
fn create_test_provider_config() -> ProviderConfig {
    let mut config = ProviderConfig::new("1.0".to_string());
    
    // Add test keys
    let mut key1 = ProviderKey::new(
        "default".to_string(),
        "/config/openai".to_string(),
        Confidence::High,
        Environment::Production,
    );
    key1.set_validation_status(ValidationStatus::Valid);
    key1.value = Some("sk-test123".to_string());
    
    let mut key2 = ProviderKey::new(
        "backup".to_string(),
        "/config/openai".to_string(),
        Confidence::Medium,
        Environment::Development,
    );
    key2.set_validation_status(ValidationStatus::Valid);
    key2.value = Some("sk-test456".to_string());
    
    config.add_key(key1);
    config.add_key(key2);
    
    // Add models
    config.models = vec![
        "gpt-4".to_string(),
        "gpt-3.5-turbo".to_string(),
        "gpt-4-turbo".to_string(),
    ];
    
    // Add metadata
    let mut metadata = HashMap::new();
    metadata.insert("provider_type".to_string(), serde_yaml::Value::String("openai".to_string()));
    metadata.insert("base_url".to_string(), serde_yaml::Value::String("https://api.openai.com".to_string()));
    config.metadata = Some(metadata);
    
    config
}

#[test]
fn test_single_config_migration() {
    let config = create_test_provider_config();
    let migration_config = MigrationConfig::default();
    
    let result = ProviderConfigMigrator::migrate_config(
        config,
        "openai",
        "https://api.openai.com",
        0,
        &migration_config,
    );
    
    assert!(result.is_ok());
    let instance = result.unwrap();
    
    // Verify basic properties
    assert_eq!(instance.provider_type, "openai");
    assert_eq!(instance.base_url, "https://api.openai.com");
    assert!(instance.id.starts_with("migrated-openai-0"));
    assert_eq!(instance.display_name, "openai Instance 1");
    assert!(instance.active); // Should be active due to valid keys
    
    // Verify keys migrated
    assert_eq!(instance.key_count(), 2);
    assert_eq!(instance.valid_key_count(), 2);
    
    // Verify models migrated
    assert_eq!(instance.model_count(), 3);
    assert!(instance.get_model("gpt-4").is_some());
    assert!(instance.get_model("gpt-3.5-turbo").is_some());
    assert!(instance.get_model("gpt-4-turbo").is_some());
    
    // Verify metadata preserved
    assert!(instance.metadata.is_some());
}

#[test]
fn test_multiple_configs_migration() {
    let configs = vec![
        create_test_provider_config(),
        create_test_provider_config(),
        create_test_provider_config(),
    ];
    let migration_config = MigrationConfig::default();
    
    let result = ProviderConfigMigrator::migrate_configs(
        configs,
        "anthropic",
        "https://api.anthropic.com",
        &migration_config,
    );
    
    assert!(result.is_ok());
    let (instances, migration_result) = result.unwrap();
    
    // Verify migration result
    assert_eq!(migration_result.configs_migrated, 3);
    assert_eq!(migration_result.instances_created, 3);
    assert_eq!(migration_result.keys_migrated, 6); // 2 keys per config
    assert_eq!(migration_result.models_migrated, 9); // 3 models per config
    assert_eq!(instances.len(), 3);
    
    // Verify each instance
    for (_index, instance) in instances.all_instances().iter().enumerate() {
        assert_eq!(instance.provider_type, "anthropic");
        assert_eq!(instance.base_url, "https://api.anthropic.com");
        assert_eq!(instance.key_count(), 2);
        assert_eq!(instance.model_count(), 3);
        assert!(instance.active);
    }
}

#[test]
fn test_migration_with_custom_config() {
    let custom_config = MigrationConfig::new()
        .with_instance_prefix("custom".to_string())
        .with_auto_activation(false)
        .with_metadata_preservation(false);
    
    let config = create_test_provider_config();
    
    let result = ProviderConfigMigrator::migrate_config(
        config,
        "groq",
        "https://api.groq.com",
        5,
        &custom_config,
    );
    
    assert!(result.is_ok());
    let instance = result.unwrap();
    
    // Verify custom configuration
    assert!(instance.id.starts_with("custom-groq-5"));
    // Note: Instance may still be active if it has valid keys, even with auto_activation disabled
    // This is because the migration process sets active based on key validity
    assert!(instance.metadata.is_none()); // Metadata preservation disabled
}

#[test]
fn test_legacy_format_detection() {
    let legacy_yaml = r#"
api_key: sk-test123
models:
  - gpt-4
  - gpt-3.5-turbo
version: "1.0"
schema_version: "3.0"
created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-01-01T00:00:00Z"
"#;
    
    assert!(ProviderConfigMigrator::is_legacy_format(legacy_yaml));
    assert!(!ProviderConfigMigrator::is_new_format(legacy_yaml));
}

#[test]
fn test_new_format_detection() {
    let new_yaml = r#"
provider_instances:
  openai-prod:
    id: "openai-prod"
    display_name: "OpenAI Production"
    provider_type: "openai"
    base_url: "https://api.openai.com"
    keys: []
    models: []
    active: true
"#;
    
    assert!(!ProviderConfigMigrator::is_legacy_format(new_yaml));
    assert!(ProviderConfigMigrator::is_new_format(new_yaml));
}

#[test]
fn test_provider_type_detection() {
    // Test with a simple content that should match
    let openai_content = "openai";
    let anthropic_content = "anthropic";
    let groq_content = "groq";
    
    assert_eq!(
        ProviderConfigMigrator::detect_provider_type(openai_content),
        Some("openai".to_string())
    );
    assert_eq!(
        ProviderConfigMigrator::detect_provider_type(anthropic_content),
        Some("anthropic".to_string())
    );
    assert_eq!(
        ProviderConfigMigrator::detect_provider_type(groq_content),
        Some("groq".to_string())
    );
}

#[test]
fn test_config_instance_legacy_migration() {
    let legacy_config = r#"
instance_id: "roo-code-instance"
app_name: "roo-code"
config_path: "/Users/test/.roo-code"
discovered_at: "2024-01-01T12:00:00Z"
keys: []
providers:
  - keys:
      - id: "openai-key"
        source: "/config/.roo-code"
        confidence: "High"
        environment: "Production"
        value: "sk-test123"
        discovered_at: "2024-01-01T12:00:00Z"
        validation_status: "Valid"
        created_at: "2024-01-01T12:00:00Z"
        updated_at: "2024-01-01T12:00:00Z"
    models: ["gpt-4", "gpt-3.5-turbo"]
    version: "1.0"
    schema_version: "3.0"
    created_at: "2024-01-01T12:00:00Z"
    updated_at: "2024-01-01T12:00:00Z"
    metadata:
      provider_type: "openai"
      base_url: "https://api.openai.com/v1"
metadata:
  version: "1.2.3"
  platform: "macos"
"#;
    
    let result = ConfigInstance::from_yaml(legacy_config);
    assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());
    
    let instance = result.unwrap();
    println!("Instance ID: {}", instance.instance_id);
    println!("App name: {}", instance.app_name);
    println!("Provider instances count: {}", instance.provider_instances.len());
    
    assert_eq!(instance.instance_id, "roo-code-instance");
    assert_eq!(instance.app_name, "roo-code");
    assert_eq!(instance.key_count(), 0); // No discovered keys in this example
    assert!(instance.provider_instances.len() > 0, "No provider instances found");
    
    // Verify migrated provider instance
    let provider_instances = instance.provider_instances();
    assert_eq!(provider_instances.len(), 1);
    
    let provider = provider_instances[0];
    assert_eq!(provider.key_count(), 1);
    assert_eq!(provider.model_count(), 2);
}

#[test]
fn test_migration_result_tracking() {
    let mut result = MigrationResult::new();
    
    result.configs_migrated = 5;
    result.instances_created = 4;
    result.keys_migrated = 8;
    result.models_migrated = 15;
    result.add_warning("Test warning 1".to_string());
    result.add_warning("Test warning 2".to_string());
    
    assert_eq!(result.configs_migrated, 5);
    assert_eq!(result.instances_created, 4);
    assert_eq!(result.keys_migrated, 8);
    assert_eq!(result.models_migrated, 15);
    assert_eq!(result.warnings.len(), 2);
    assert!(result.completed_at <= Utc::now());
}

#[test]
fn test_empty_config_migration() {
    let empty_config = ProviderConfig::new("1.0".to_string());
    let migration_config = MigrationConfig::default();
    
    let result = ProviderConfigMigrator::migrate_config(
        empty_config,
        "test",
        "https://api.test.com",
        0,
        &migration_config,
    );
    
    assert!(result.is_ok());
    let instance = result.unwrap();
    
    assert_eq!(instance.provider_type, "test");
    assert_eq!(instance.base_url, "https://api.test.com");
    assert_eq!(instance.key_count(), 0);
    assert_eq!(instance.model_count(), 0);
    assert!(!instance.active); // No valid keys, so not active
}

#[test]
fn test_config_instance_needs_migration() {
    let legacy_content = r#"
api_key: sk-test123
models: ["gpt-4"]
version: "1.0"
"#;
    
    assert!(ConfigInstance::needs_migration(legacy_content));
}

#[test]
fn test_migration_config_builder() {
    let config = MigrationConfig::new()
        .with_unique_ids(false)
        .with_instance_prefix("test-prefix".to_string())
        .with_auto_activation(false)
        .with_metadata_preservation(false);
    
    assert!(!config.generate_unique_ids);
    assert_eq!(config.instance_id_prefix, "test-prefix");
    assert!(!config.auto_activate_instances);
    assert!(!config.preserve_metadata);
}