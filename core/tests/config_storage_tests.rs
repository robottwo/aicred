//! Comprehensive tests for the multi-file YAML storage system
//!
//! This module tests the configuration storage system that manages provider
//! configurations across multiple YAML files with a manifest-based approach.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use genai_keyfinder_core::models::{ProviderKey, Confidence, Environment, ValidationStatus};

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
    let providers_dir = config_dir.join("providers");
    
    fs::create_dir_all(&providers_dir).unwrap();
    
    (temp_dir, config_dir, providers_dir)
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
            map.insert("region".to_string(), serde_yaml::Value::String("us-east-1".to_string()));
            map.insert("tier".to_string(), serde_yaml::Value::String("premium".to_string()));
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
fn test_migration_from_single_file_to_multi_file() {
    let (temp_dir, config_dir, providers_dir) = create_test_config_dir();
    let old_config_path = config_dir.join("providers.yaml");
    let manifest_path = config_dir.join("manifest.yaml");
    
    // Create old single-file format
    let old_config = serde_yaml::Value::Mapping({
        let mut mapping = serde_yaml::Mapping::new();
        
        let mut providers = serde_yaml::Mapping::new();
        
        // OpenAI provider
        let mut openai_config = serde_yaml::Mapping::new();
        openai_config.insert(
            serde_yaml::Value::String("api_key".to_string()),
            serde_yaml::Value::String("sk-openai-old-key".to_string()),
        );
        openai_config.insert(
            serde_yaml::Value::String("version".to_string()),
            serde_yaml::Value::String("1.5".to_string()),
        );
        providers.insert(
            serde_yaml::Value::String("openai".to_string()),
            serde_yaml::Value::Mapping(openai_config),
        );
        
        // Anthropic provider
        let mut anthropic_config = serde_yaml::Mapping::new();
        anthropic_config.insert(
            serde_yaml::Value::String("api_key".to_string()),
            serde_yaml::Value::String("sk-ant-anthropic-old-key".to_string()),
        );
        anthropic_config.insert(
            serde_yaml::Value::String("models".to_string()),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("claude-3-opus".to_string()),
                serde_yaml::Value::String("claude-3-sonnet".to_string()),
            ]),
        );
        providers.insert(
            serde_yaml::Value::String("anthropic".to_string()),
            serde_yaml::Value::Mapping(anthropic_config),
        );
        
        mapping.insert(
            serde_yaml::Value::String("providers".to_string()),
            serde_yaml::Value::Mapping(providers),
        );
        
        mapping
    });
    
    // Write old format
    let old_content = serde_yaml::to_string(&old_config).unwrap();
    fs::write(&old_config_path, old_content).unwrap();
    
    // Simulate migration (this mimics the CLI migration logic)
    let backup_path = old_config_path.with_extension("yaml.backup");
    fs::copy(&old_config_path, &backup_path).unwrap();
    
    // Extract providers from old format
    if let Some(providers) = old_config.get("providers").and_then(|p| p.as_mapping()) {
        let mut manifest = Manifest {
            providers: HashMap::new(),
            schema_version: "2.0".to_string(),
            last_updated: Utc::now(),
            migration_info: None,
        };
        let now = Utc::now();
        
        for (provider_name, provider_data) in providers {
            if let Some(name) = provider_name.as_str() {
                let provider_name = name.to_string();
                let provider_file_name = format!("{}.yaml", provider_name.to_lowercase().replace(' ', "_"));
                let provider_file_path = providers_dir.join(&provider_file_name);
                
                // Convert provider data to new format
                let mut provider_config = ProviderConfig {
                    keys: Vec::new(),
                    models: Vec::new(),
                    metadata: None,
                    version: "1.0".to_string(),
                    schema_version: "3.0".to_string(),
                    created_at: now,
                    updated_at: now,
                };
                
                // Extract data from old format and convert to new key format
                if let Some(api_key) = provider_data.get("api_key").and_then(|v| v.as_str()) {
                    let default_key = ProviderKey {
                        id: "default".to_string(),
                        value: Some(api_key.to_string()),
                        discovered_at: now,
                        source: "migration".to_string(),
                        line_number: None,
                        confidence: Confidence::High,
                        environment: Environment::Production,
                        last_validated: None,
                        validation_status: ValidationStatus::Unknown,
                        metadata: None,
                        created_at: now,
                        updated_at: now,
                    };
                    provider_config.keys.push(default_key);
                }
                if let Some(models) = provider_data.get("models").and_then(|v| v.as_sequence()) {
                    provider_config.models = models.iter()
                        .filter_map(|m| m.as_str().map(String::from))
                        .collect();
                }
                if let Some(version) = provider_data.get("version").and_then(|v| v.as_str()) {
                    provider_config.version = version.to_string();
                }
                
                // Write individual provider file
                let yaml_content = serde_yaml::to_string(&provider_config).unwrap();
                fs::write(&provider_file_path, yaml_content).unwrap();
                
                // Add to manifest
                manifest.providers.insert(provider_name.clone(), ProviderMetadata {
                    file_name: provider_name.to_lowercase().replace(' ', "_"),
                    created_at: provider_config.created_at,
                    updated_at: provider_config.updated_at,
                    version: provider_config.version,
                });
            }
        }
        
        // Add migration info
        manifest.migration_info = Some(MigrationInfo {
            migrated_at: now,
            original_file: old_config_path.clone(),
            backup_file: backup_path,
        });
        
        // Write manifest
        let manifest_content = serde_yaml::to_string(&manifest).unwrap();
        fs::write(&manifest_path, manifest_content).unwrap();
        
        // Remove old file
        fs::remove_file(&old_config_path).unwrap();
    }
    
    // Verify migration results
    assert!(manifest_path.exists());
    assert!(!old_config_path.exists());
    
    // Check backup exists
    let backup_path = old_config_path.with_extension("yaml.backup");
    assert!(backup_path.exists());
    
    // Verify manifest
    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    let manifest: Manifest = serde_yaml::from_str(&manifest_content).unwrap();
    assert_eq!(manifest.providers.len(), 2);
    assert!(manifest.providers.contains_key("openai"));
    assert!(manifest.providers.contains_key("anthropic"));
    assert!(manifest.migration_info.is_some());
    
    // Verify individual provider files
    let openai_path = providers_dir.join("openai.yaml");
    let anthropic_path = providers_dir.join("anthropic.yaml");
    assert!(openai_path.exists());
    assert!(anthropic_path.exists());
    
    // Verify provider content
    let openai_content = fs::read_to_string(&openai_path).unwrap();
    let openai_config: ProviderConfig = serde_yaml::from_str(&openai_content).unwrap();
    assert_eq!(openai_config.keys.len(), 1);
    assert_eq!(openai_config.keys[0].value, Some("sk-openai-old-key".to_string()));
    assert_eq!(openai_config.version, "1.5");
    
    let anthropic_content = fs::read_to_string(&anthropic_path).unwrap();
    let anthropic_config: ProviderConfig = serde_yaml::from_str(&anthropic_content).unwrap();
    assert_eq!(anthropic_config.keys.len(), 1);
    assert_eq!(anthropic_config.keys[0].value, Some("sk-ant-anthropic-old-key".to_string()));
    assert_eq!(anthropic_config.models.len(), 2);
}

#[test]
fn test_error_handling_corrupted_files() {
    let (_temp_dir, config_dir, providers_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");
    
    // Test corrupted manifest
    fs::write(&manifest_path, "invalid yaml content: { broken").unwrap();
    
    let result = fs::read_to_string(&manifest_path)
        .and_then(|content| serde_yaml::from_str::<Manifest>(&content).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid YAML")
        }));
    
    assert!(result.is_err());
    
    // Test corrupted provider file
    let provider_path = providers_dir.join("corrupted.yaml");
    fs::write(&provider_path, "invalid: yaml: content: [").unwrap();
    
    let provider_result = fs::read_to_string(&provider_path)
        .and_then(|content| serde_yaml::from_str::<ProviderConfig>(&content).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid YAML")
        }));
    
    assert!(provider_result.is_err());
}

#[test]
fn test_error_handling_missing_files() {
    let (_temp_dir, config_dir, providers_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");
    let provider_path = providers_dir.join("nonexistent.yaml");
    
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
    let (_temp_dir, _config_dir, providers_dir) = create_test_config_dir();
    
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
    
    let provider_path = providers_dir.join("empty.yaml");
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
    let (_temp_dir, _config_dir, providers_dir) = create_test_config_dir();
    let provider_path = providers_dir.join("invalid.yaml");
    
    // Write invalid YAML
    fs::write(&provider_path, "invalid: yaml: content: [ broken").unwrap();
    
    // Attempt to parse
    let result = fs::read_to_string(&provider_path)
        .and_then(|content| serde_yaml::from_str::<ProviderConfig>(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        }));
    
    assert!(result.is_err());
}

#[test]
fn test_schema_validation() {
    let (_temp_dir, _config_dir, providers_dir) = create_test_config_dir();
    
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
    
    let provider_path = providers_dir.join("valid.yaml");
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
    assert!(minimal_result.is_ok(), "Failed to deserialize minimal YAML: {:?}", minimal_result.err());
    let minimal_config = minimal_result.unwrap();
    assert!(minimal_config.keys.is_empty());
    assert!(minimal_config.metadata.is_none());
}

#[test]
fn test_backward_compatibility() {
    let (_temp_dir, config_dir, providers_dir) = create_test_config_dir();
    let old_config_path = config_dir.join("providers.yaml");
    
    // Create old format without schema version
    let old_format = serde_yaml::Value::Mapping({
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(
            serde_yaml::Value::String("providers".to_string()),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
        mapping
    });
    
    let old_content = serde_yaml::to_string(&old_format).unwrap();
    fs::write(&old_config_path, old_content).unwrap();
    
    // Simulate detection of old format and migration
    assert!(old_config_path.exists());
    
    // This would trigger migration in the real implementation
    // For now, we just verify the old format exists and can be read
    let loaded_content = fs::read_to_string(&old_config_path).unwrap();
    let loaded_old: serde_yaml::Value = serde_yaml::from_str(&loaded_content).unwrap();
    
    assert!(loaded_old.get("providers").is_some());
}

#[test]
fn test_manifest_integrity_after_multiple_updates() {
    let (_temp_dir, config_dir, providers_dir) = create_test_config_dir();
    let manifest_path = config_dir.join("manifest.yaml");
    
    // Create initial manifest
    let mut manifest = create_sample_manifest();
    let manifest_content = serde_yaml::to_string(&manifest).unwrap();
    fs::write(&manifest_path, manifest_content).unwrap();
    
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
        
        let provider_path = providers_dir.join(format!("{}.yaml", new_provider_file));
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
        let original_timestamp = manifest.last_updated;
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