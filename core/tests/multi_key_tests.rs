//! Tests for the new multi-key provider configuration schema

// Allow clippy lints for multi-key tests
#![allow(unused_imports)]

use aicred_core::models::{Confidence, Environment, ProviderConfig, ProviderKey, ValidationStatus};
use chrono::{DateTime, Utc};

#[test]
fn test_provider_key_creation() {
    let key = ProviderKey {
        id: "test-key-1".to_string(),
        value: Some("sk-test123456789".to_string()),
        discovered_at: Utc::now(),
        source: "test.env".to_string(),
        line_number: Some(42),
        confidence: Confidence::High,
        environment: Environment::Development,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(key.id, "test-key-1");
    assert_eq!(key.value, Some("sk-test123456789".to_string()));
    assert_eq!(key.confidence, Confidence::High);
    assert_eq!(key.environment, Environment::Development);
    assert_eq!(key.validation_status, ValidationStatus::Unknown);
}

#[test]
fn test_provider_config_with_multiple_keys() {
    let mut config = ProviderConfig::new("1.0".to_string());

    let key1 = ProviderKey {
        id: "default".to_string(),
        value: Some("sk-default123".to_string()),
        discovered_at: Utc::now(),
        source: "config.json".to_string(),
        line_number: Some(10),
        confidence: Confidence::High,
        environment: Environment::Production,
        last_validated: None,
        validation_status: ValidationStatus::Valid,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let key2 = ProviderKey {
        id: "staging".to_string(),
        value: Some("sk-staging456".to_string()),
        discovered_at: Utc::now(),
        source: "config.json".to_string(),
        line_number: Some(15),
        confidence: Confidence::Medium,
        environment: Environment::Staging,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    config.add_key(key1.clone());
    config.add_key(key2.clone());

    assert_eq!(config.key_count(), 2);
    assert!(config
        .keys
        .iter()
        .any(|k| k.id == "default" && k.value == key1.value));
    assert!(config
        .keys
        .iter()
        .any(|k| k.id == "staging" && k.value == key2.value));
    assert!(!config.keys.iter().any(|k| k.id == "nonexistent"));
}

#[test]
fn test_provider_config_key_count_by_environment() {
    let mut config = ProviderConfig::new("1.0".to_string());

    // Add keys for different environments
    for (env, count) in &[
        (Environment::Production, 3),
        (Environment::Staging, 2),
        (Environment::Development, 1),
    ] {
        for i in 0..*count {
            let key = ProviderKey {
                id: format!("{:?}-key-{}", env, i),
                value: Some(format!("sk-key{}", i)),
                discovered_at: Utc::now(),
                source: "test.env".to_string(),
                line_number: None,
                confidence: Confidence::High,
                environment: env.clone(),
                last_validated: None,
                validation_status: ValidationStatus::Unknown,
                metadata: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            config.add_key(key);
        }
    }

    assert_eq!(config.key_count(), 6);

    // Test environment filtering
    let prod_keys: Vec<&ProviderKey> = config
        .keys
        .iter()
        .filter(|k| k.environment == Environment::Production)
        .collect();
    assert_eq!(prod_keys.len(), 3);

    let staging_keys: Vec<&ProviderKey> = config
        .keys
        .iter()
        .filter(|k| k.environment == Environment::Staging)
        .collect();
    assert_eq!(staging_keys.len(), 2);
}

#[test]
fn test_backward_compatibility_from_old_format() {
    // Test that we can create a ProviderConfig from old single-key format
    let old_config = ProviderConfig::from_old_format(
        Some("sk-oldformat123".to_string()),
        vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        None,
        "1.0".to_string(),
        Utc::now(),
        Utc::now(),
    );

    assert_eq!(old_config.key_count(), 1);
    assert_eq!(old_config.keys[0].id, "default");
    assert_eq!(
        old_config.keys[0].value,
        Some("sk-oldformat123".to_string())
    );
    assert_eq!(old_config.keys[0].environment, Environment::Production);
    assert_eq!(old_config.models.len(), 2);
}

#[test]
fn test_validation_status_transitions() {
    let mut key = ProviderKey {
        id: "test-key".to_string(),
        value: Some("sk-test123".to_string()),
        discovered_at: Utc::now(),
        source: "test.env".to_string(),
        line_number: None,
        confidence: Confidence::High,
        environment: Environment::Production,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test validation status update
    key.validation_status = ValidationStatus::Valid;
    key.last_validated = Some(Utc::now());

    assert_eq!(key.validation_status, ValidationStatus::Valid);
    assert!(key.last_validated.is_some());
}

#[test]
fn test_provider_key_serialization() {
    let key = ProviderKey {
        id: "serialize-test".to_string(),
        value: Some("sk-serialize123".to_string()),
        discovered_at: Utc::now(),
        source: "config.yaml".to_string(),
        line_number: Some(25),
        confidence: Confidence::VeryHigh,
        environment: Environment::Production,
        last_validated: Some(Utc::now()),
        validation_status: ValidationStatus::Valid,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test serialization
    let serialized = serde_json::to_string(&key).expect("Failed to serialize key");
    let deserialized: ProviderKey =
        serde_json::from_str(&serialized).expect("Failed to deserialize key");

    assert_eq!(deserialized.id, key.id);
    assert_eq!(deserialized.confidence, key.confidence);
    assert_eq!(deserialized.environment, key.environment);
    assert_eq!(deserialized.validation_status, key.validation_status);
}

#[test]
fn test_provider_config_serialization() {
    let mut config = ProviderConfig::new("1.0".to_string());

    let key = ProviderKey {
        id: "test-serialization".to_string(),
        value: Some("sk-serialize456".to_string()),
        discovered_at: Utc::now(),
        source: "test.yaml".to_string(),
        line_number: None,
        confidence: Confidence::Medium,
        environment: Environment::Development,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    config.add_key(key);
    config.models = vec!["model1".to_string(), "model2".to_string()];

    // Test YAML serialization
    let yaml_content = serde_yaml::to_string(&config).expect("Failed to serialize config");
    let deserialized: ProviderConfig =
        serde_yaml::from_str(&yaml_content).expect("Failed to deserialize config");

    assert_eq!(deserialized.key_count(), 1);
    assert_eq!(deserialized.models.len(), 2);
    assert_eq!(deserialized.schema_version, "3.0");
}
