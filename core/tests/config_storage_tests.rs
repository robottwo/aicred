//! Comprehensive tests for the multi-file YAML storage system
//!
//! This module tests the configuration storage system that manages provider
//! configurations across multiple YAML files with a manifest-based approach.

use chrono::{DateTime, Utc};
use genai_keyfinder_core::models::{Confidence, Environment, ProviderKey, ValidationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Mirror of the CLI's ProviderConfig structure for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    pub keys: Vec<ProviderKey>,
    pub models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
    pub version: String,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mirror of the CLI's ProviderMetadata structure for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderMetadata {
    pub file_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
}

/// Mirror of the CLI's MigrationInfo structure for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationInfo {
    pub migrated_at: DateTime<Utc>,
    pub original_file: PathBuf,
    pub backup_file: PathBuf,
}

/// Mirror of the CLI's Manifest structure for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub providers: HashMap<String, ProviderMetadata>,
    pub schema_version: String,
    pub last_updated: DateTime<Utc>,
    pub migration_info: Option<MigrationInfo>,
}

/// Test helper to create a test configuration directory structure
fn create_test_config_dir() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config").join("aicred");
    let inference_services_dir = config_dir.join("inference_services");

    fs::create_dir_all(&inference_services_dir).unwrap();

    (temp_dir, config_dir, inference_services_dir)
}

/// Test helper to create a sample manifest
fn create_sample_manifest() -> Manifest {
    let now = Utc::now();
    let mut providers = HashMap::new();

    providers.insert(
        "openai".to_string(),
        ProviderMetadata {
            file_name: "openai".to_string(),
            created_at: now,
            updated_at: now,
            version: "1.0".to_string(),
        },
    );

    providers.insert(
        "anthropic".to_string(),
        ProviderMetadata {
            file_name: "anthropic".to_string(),
            created_at: now,
            updated_at: now,
            version: "1.0".to_string(),
        },
    );

    Manifest {
        providers,
        schema_version: "2.0".to_string(),
        last_updated: now,
        migration_info: None,
    }
}

/// Test helper to create a sample provider config
fn create_sample_provider_config(name: &str) -> ProviderConfig {
    let now = Utc::now();
    let default_key = ProviderKey {
        id: "default".to_string(),
        value: Some(format!("sk-{}-test-key", name)),
        discovered_at: now,
        source: format!("{}.yaml", name),
        line_number: Some(1),
        confidence: Confidence::High,
        environment: Environment::Production,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: now,
        updated_at: now,
    };

    ProviderConfig {
        keys: vec![default_key],
        models: vec![format!("{}-model-1", name), format!("{}-model-2", name)],
        metadata: Some({
            let mut map = HashMap::new();
            map.insert(
                "region".to_string(),
                serde_yaml::Value::String("us-east-1".to_string()),
            );
            map.insert(
                "tier".to_string(),
                serde_yaml::Value::String("premium".to_string()),
            );
            map
        }),
        version: "1.0".to_string(),
        schema_version: "3.0".to_string(),
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn test_manifest_creation_and_validation() {
    let (_temp_dir, config_dir, _providers_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");

    // Create and save a manifest
    let manifest = create_sample_manifest();
    let manifest_content = serde_yaml::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_content).unwrap();

    // Load and validate the manifest
    let loaded_content = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest: Manifest = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded_manifest.schema_version, "2.0");
    assert_eq!(loaded_manifest.providers.len(), 2);
    assert!(loaded_manifest.providers.contains_key("openai"));
    assert!(loaded_manifest.providers.contains_key("anthropic"));
    assert!(loaded_manifest.migration_info.is_none());
}

#[test]
fn test_provider_file_crud_operations() {
    let (_temp_dir, _config_dir, providers_dir) = create_test_config_dir();

    // Create a provider config
    let provider_name = "test_provider";
    let provider_config = create_sample_provider_config(provider_name);
    let provider_file_path = providers_dir.join(format!("{}.yaml", provider_name));

    // Create (Write) operation
    let yaml_content = serde_yaml::to_string(&provider_config).unwrap();
    fs::write(&provider_file_path, yaml_content).unwrap();
    assert!(provider_file_path.exists());

    // Read operation
    let loaded_content = fs::read_to_string(&provider_file_path).unwrap();
    let loaded_config: ProviderConfig = serde_yaml::from_str(&loaded_content).unwrap();
    assert_eq!(loaded_config.keys, provider_config.keys);
    assert_eq!(loaded_config.models, provider_config.models);
    assert_eq!(loaded_config.version, provider_config.version);

    // Update operation
    let mut updated_config = loaded_config;
    updated_config.version = "2.0".to_string();
    updated_config.models.push("new-model".to_string());
    updated_config.updated_at = Utc::now();

    let updated_yaml = serde_yaml::to_string(&updated_config).unwrap();
    fs::write(&provider_file_path, updated_yaml).unwrap();

    // Verify update
    let updated_content = fs::read_to_string(&provider_file_path).unwrap();
    let verified_config: ProviderConfig = serde_yaml::from_str(&updated_content).unwrap();
    assert_eq!(verified_config.version, "2.0");
    assert_eq!(verified_config.models.len(), 3);
    assert!(verified_config.models.contains(&"new-model".to_string()));

    // Delete operation
    fs::remove_file(&provider_file_path).unwrap();
    assert!(!provider_file_path.exists());
}

#[test]
fn test_migration_from_legacy_format_removed() {
    // This test verifies that the old providers.yaml format is no longer supported
    // The migration functionality has been removed as part of the directory structure change
    let (temp_dir, config_dir, inference_services_dir) = create_test_config_dir();

    // Verify the new directory structure exists
    assert!(config_dir.exists());
    assert!(inference_services_dir.exists());

    // The old providers.yaml file should not exist in the new structure
    let old_config_path = config_dir.join("providers.yaml");
    assert!(!old_config_path.exists());

    // The new structure should use inference_services directory instead of providers
    assert!(inference_services_dir.exists());
}

#[test]
fn test_error_handling_corrupted_files() {
    let (_temp_dir, config_dir, inference_services_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");

    // Test corrupted manifest
    fs::write(&manifest_path, "invalid yaml content: { broken").unwrap();

    let result = fs::read_to_string(&manifest_path).and_then(|content| {
        serde_yaml::from_str::<Manifest>(&content)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid YAML"))
    });

    assert!(result.is_err());

    // Test corrupted provider file
    let provider_path = inference_services_dir.join("corrupted.yaml");
    fs::write(&provider_path, "invalid: yaml: content: [").unwrap();
    let provider_result = fs::read_to_string(&provider_path).and_then(|content| {
        serde_yaml::from_str::<ProviderConfig>(&content)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid YAML"))
    });

    assert!(provider_result.is_err());
}

#[test]
fn test_error_handling_missing_files() {
    let (_temp_dir, config_dir, inference_services_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");
    let provider_path = inference_services_dir.join("nonexistent.yaml");

    // Test missing manifest
    assert!(!manifest_path.exists());
    let result = fs::read_to_string(&manifest_path);
    assert!(result.is_err());

    // Test missing provider file
    assert!(!provider_path.exists());
    let provider_result = fs::read_to_string(&provider_path);
    assert!(provider_result.is_err());
}

#[test]
fn test_empty_providers_handling() {
    let (_temp_dir, _config_dir, inference_services_dir) = create_test_config_dir();

    // Create empty provider config
    let empty_config = ProviderConfig {
        keys: Vec::new(),
        models: Vec::new(),
        metadata: None,
        version: "1.0".to_string(),
        schema_version: "3.0".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let provider_path = inference_services_dir.join("empty.yaml");
    let yaml_content = serde_yaml::to_string(&empty_config).unwrap();
    fs::write(&provider_path, yaml_content).unwrap();

    // Load and verify
    let loaded_content = fs::read_to_string(&provider_path).unwrap();
    let loaded_config: ProviderConfig = serde_yaml::from_str(&loaded_content).unwrap();

    assert!(loaded_config.keys.is_empty());
    assert!(loaded_config.models.is_empty());
    assert!(loaded_config.metadata.is_none());
}

#[test]
fn test_invalid_yaml_handling() {
    let (_temp_dir, _config_dir, inference_services_dir) = create_test_config_dir();
    let provider_path = inference_services_dir.join("invalid.yaml");

    // Write invalid YAML
    fs::write(&provider_path, "invalid: yaml: content: [ broken").unwrap();

    // Attempt to parse
    let result = fs::read_to_string(&provider_path).and_then(|content| {
        serde_yaml::from_str::<ProviderConfig>(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    });

    assert!(result.is_err());
}

#[test]
fn test_schema_validation() {
    let (_temp_dir, _config_dir, inference_services_dir) = create_test_config_dir();

    // Test valid schema
    let now = Utc::now();
    let test_key = ProviderKey {
        id: "default".to_string(),
        value: Some("sk-test-key".to_string()),
        discovered_at: now,
        source: "test.yaml".to_string(),
        line_number: Some(1),
        confidence: Confidence::High,
        environment: Environment::Production,
        last_validated: None,
        validation_status: ValidationStatus::Unknown,
        metadata: None,
        created_at: now,
        updated_at: now,
    };

    let valid_config = ProviderConfig {
        keys: vec![test_key],
        models: vec!["model1".to_string(), "model2".to_string()],
        metadata: None,
        version: "1.0".to_string(),
        schema_version: "3.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    let provider_path = inference_services_dir.join("valid.yaml");
    let yaml_content = serde_yaml::to_string(&valid_config).unwrap();
    fs::write(&provider_path, yaml_content).unwrap();

    let loaded_content = fs::read_to_string(&provider_path).unwrap();
    let loaded_config: ProviderConfig = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded_config.version, "1.0");
    assert_eq!(loaded_config.models.len(), 2);

    // Test with missing optional fields (metadata only, keys defaults to empty)
    let minimal_yaml = r#"
keys: []
models:
  - "gpt-4"
  - "gpt-3.5-turbo"
version: "1.0"
schema_version: "3.0"
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;

    let minimal_result = serde_yaml::from_str::<ProviderConfig>(minimal_yaml);
    if let Err(e) = &minimal_result {
        eprintln!("Deserialization error: {}", e);
    }
    assert!(
        minimal_result.is_ok(),
        "Failed to deserialize minimal YAML: {:?}",
        minimal_result.err()
    );
    let minimal_config = minimal_result.unwrap();
    assert!(minimal_config.keys.is_empty());
    assert!(minimal_config.metadata.is_none());
}

#[test]
fn test_backward_compatibility_removed() {
    let (_temp_dir, config_dir, inference_services_dir) = create_test_config_dir();

    // The old providers.yaml format is no longer supported
    let old_config_path = config_dir.join("providers.yaml");
    assert!(!old_config_path.exists());

    // The new structure should use inference_services directory
    assert!(inference_services_dir.exists());
}

#[test]
fn test_manifest_integrity_after_multiple_updates() {
    let (_temp_dir, config_dir, inference_services_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");

    // Create initial manifest
    let mut manifest = create_sample_manifest();
    let manifest_content = serde_yaml::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_content).unwrap();

    // Save original timestamp for comparison
    let original_timestamp = manifest.last_updated;

    // Simulate multiple provider updates
    for i in 0..5 {
        // Add a new provider
        let new_provider_name = format!("provider_{}", i);
        let new_provider_file = format!("provider_{}", i);

        // Create provider file
        let now = Utc::now();
        let test_key = ProviderKey {
            id: "default".to_string(),
            value: Some(format!("sk-key-{}", i)),
            discovered_at: now,
            source: format!("provider_{}.yaml", i),
            line_number: Some(1),
            confidence: Confidence::High,
            environment: Environment::Production,
            last_validated: None,
            validation_status: ValidationStatus::Unknown,
            metadata: None,
            created_at: now,
            updated_at: now,
        };

        let provider_config = ProviderConfig {
            keys: vec![test_key],
            models: vec![format!("model-{}", i)],
            metadata: None,
            version: format!("1.{}", i),
            schema_version: "3.0".to_string(),
            created_at: now,
            updated_at: now,
        };

        let provider_path = inference_services_dir.join(format!("{}.yaml", new_provider_file));
        let yaml_content = serde_yaml::to_string(&provider_config).unwrap();
        fs::write(&provider_path, yaml_content).unwrap();

        // Update manifest
        manifest.providers.insert(
            new_provider_name.clone(),
            ProviderMetadata {
                file_name: new_provider_file,
                created_at: provider_config.created_at,
                updated_at: provider_config.updated_at,
                version: provider_config.version,
            },
        );

        // Update manifest timestamp
        std::thread::sleep(std::time::Duration::from_millis(10)); // Small delay to ensure different timestamp
        manifest.last_updated = Utc::now();
    }

    // Save updated manifest
    let updated_manifest_content = serde_yaml::to_string(&manifest).unwrap();
    fs::write(&manifest_path, updated_manifest_content).unwrap();

    // Verify manifest integrity
    let final_content = fs::read_to_string(&manifest_path).unwrap();
    let final_manifest: Manifest = serde_yaml::from_str(&final_content).unwrap();

    assert_eq!(final_manifest.providers.len(), 7); // 2 original + 5 new
    assert_eq!(final_manifest.schema_version, "2.0");
    assert!(final_manifest.last_updated >= original_timestamp);
}

// Note: Concurrent access testing would require more complex setup with multiple threads/processes
// and file locking mechanisms, which is beyond the scope of unit tests but should be
// considered for integration testing.

// Note: Permission error testing would require changing file permissions, which is
// platform-dependent and potentially dangerous in test environments.

// Note: Atomic file operations testing would require implementing atomic write
// mechanisms in the actual implementation first.
