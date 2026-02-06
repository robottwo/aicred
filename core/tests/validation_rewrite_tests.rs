//! Comprehensive tests for configuration validation and rewrite logic.
//!
//! This module tests the validation functions in config_validator.rs and the
//! automatic rewrite behavior in ConfigInstance::load_or_create().

use aicred_core::models::{
    config_validator::{validate_provider_instance_yaml, validate_provider_instances_yaml},
    ConfigInstance,
};
use std::fs;
use tempfile::TempDir;

/// Creates a valid provider instance YAML for testing
fn create_valid_provider_yaml() -> &'static str {
    r#"
id: openai-prod
provider_type: openai
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
capabilities:
  chat: true
  completion: true
  embedding: false
  image_generation: false
  function_calling: true
  streaming: true
active: true
"#
}

/// Creates a valid provider instances collection YAML for testing
fn create_valid_providers_yaml() -> &'static str {
    r#"
openai-prod:
  id: openai-prod
  provider_type: openai
  base_url: https://api.openai.com
  api_key: sk-test1234567890abcdef
  models: []
  capabilities:
    chat: true
    completion: true
    embedding: false
    image_generation: false
    function_calling: true
    streaming: true
  active: true
anthropic-dev:
  id: anthropic-dev
  provider_type: anthropic
  base_url: https://api.anthropic.com
  api_key: sk-ant-test1234567890abcdef
  models: []
  capabilities:
    chat: true
    completion: true
    embedding: false
    image_generation: false
    function_calling: true
    streaming: true
  active: true
"#
}

#[test]
fn test_validate_provider_instance_yaml_valid() {
    let yaml = create_valid_provider_yaml();
    let result = validate_provider_instance_yaml(yaml);
    assert!(
        result.is_ok(),
        "Valid YAML should pass validation: {:?}",
        result
    );
}

#[test]
fn test_validate_provider_instance_yaml_missing_required_field() {
    let yaml = r#"
provider_type: openai
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    assert!(result.is_err(), "Missing id field should fail validation");
    let error = result.unwrap_err();
    assert!(
        error.contains("Failed to parse YAML") || error.contains("missing field"),
        "Error should indicate parsing issue: {}",
        error
    );
}

#[test]
fn test_validate_provider_instance_yaml_empty_id() {
    let yaml = r#"
id: ""
provider_type: openai
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
capabilities:
  chat: true
  completion: true
  embedding: false
  image_generation: false
  function_calling: true
  streaming: true
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    // Note: Empty ID validation was removed in v0.2.0 refactoring
    // The YAML should deserialize and validate successfully even with empty ID
    assert!(result.is_ok(), "Validation should succeed (empty ID not validated in v0.2.0)");
}

#[test]
fn test_validate_provider_instance_yaml_empty_display_name() {
    let yaml = r#"
id: openai-prod
provider_type: openai
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
capabilities:
  chat: true
  completion: true
  embedding: false
  image_generation: false
  function_calling: true
  streaming: true
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    // Should succeed - display_name field was removed in v0.2.0
    assert!(result.is_ok(), "Validation should succeed without display_name field");
}

#[test]
fn test_validate_provider_instance_yaml_empty_provider_type() {
    let yaml = r#"
id: openai-prod
provider_type: ""
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
capabilities:
  chat: true
  completion: true
  embedding: false
  image_generation: false
  function_calling: true
  streaming: true
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    assert!(
        result.is_err(),
        "Empty provider type should fail validation"
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("Provider type cannot be empty"),
        "Error should indicate empty provider type: {}",
        error
    );
}

#[test]
fn test_validate_provider_instance_yaml_empty_base_url() {
    let yaml = r#"
id: openai-prod
provider_type: openai
base_url: ""
api_key: sk-test1234567890abcdef
models: []
capabilities:
  chat: true
  completion: true
  embedding: false
  image_generation: false
  function_calling: true
  streaming: true
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    assert!(result.is_err(), "Empty base URL should fail validation");
    let error = result.unwrap_err();
    assert!(
        error.contains("Base URL cannot be empty"),
        "Error should indicate empty base URL: {}",
        error
    );
}

#[test]
fn test_validate_provider_instance_yaml_malformed_yaml() {
    let yaml = r#"
id: openai-prod
provider_type: openai
  invalid: indentation
base_url: https://api.openai.com
api_key: sk-test1234567890abcdef
models: []
active: true
"#;

    let result = validate_provider_instance_yaml(yaml);
    assert!(result.is_err(), "Malformed YAML should fail validation");
    let error = result.unwrap_err();
    assert!(
        error.contains("Failed to parse YAML") || error.contains("unknown field"),
        "Error should indicate YAML parsing failure: {}",
        error
    );
}

#[test]
fn test_validate_provider_instances_yaml_valid() {
    let yaml = create_valid_providers_yaml();
    let result = validate_provider_instances_yaml(yaml);
    assert!(
        result.is_ok(),
        "Valid providers YAML should pass validation: {:?}",
        result
    );
}

#[test]
fn test_validate_provider_instances_yaml_empty_collection() {
    let yaml = "{}";
    let result = validate_provider_instances_yaml(yaml);
    assert!(
        result.is_ok(),
        "Empty collection should pass validation: {:?}",
        result
    );
}

#[test]
fn test_config_instance_load_or_create_with_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Write invalid YAML to file
    fs::write(&config_path, "invalid yaml content: [").unwrap();

    let result = ConfigInstance::load_or_create(
        &config_path,
        "test-instance".to_string(),
        "test-app".to_string(),
    );

    assert!(
        result.is_ok(),
        "Should create new instance when YAML is invalid"
    );
    let instance = result.unwrap();
    assert_eq!(instance.instance_id, "test-instance");
    assert_eq!(instance.app_name, "test-app");

    // Verify invalid file was deleted
    assert!(!config_path.exists(), "Invalid file should be deleted");
}

#[test]
fn test_config_instance_load_or_create_with_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Write empty file
    fs::write(&config_path, "").unwrap();

    let result = ConfigInstance::load_or_create(
        &config_path,
        "test-instance".to_string(),
        "test-app".to_string(),
    );

    assert!(
        result.is_ok(),
        "Should create new instance when file is empty"
    );
    let instance = result.unwrap();
    assert_eq!(instance.instance_id, "test-instance");
}

#[test]
fn test_config_instance_load_or_create_with_whitespace_only() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Write whitespace-only file
    fs::write(&config_path, "   \n\t  \n  ").unwrap();

    let result = ConfigInstance::load_or_create(
        &config_path,
        "test-instance".to_string(),
        "test-app".to_string(),
    );

    assert!(
        result.is_ok(),
        "Should create new instance when file contains only whitespace"
    );
    let instance = result.unwrap();
    assert_eq!(instance.instance_id, "test-instance");
}

#[test]
fn test_config_instance_load_or_create_with_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.yaml");

    let result = ConfigInstance::load_or_create(
        &config_path,
        "test-instance".to_string(),
        "test-app".to_string(),
    );

    assert!(
        result.is_ok(),
        "Should create new instance when file does not exist"
    );
    let instance = result.unwrap();
    assert_eq!(instance.instance_id, "test-instance");
    assert_eq!(instance.app_name, "test-app");
}

#[test]
#[cfg(unix)]
fn test_config_instance_load_or_create_file_deletion_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("readonly.yaml");

    // Write invalid content
    fs::write(&config_path, "invalid content").unwrap();

    // Make parent directory read-only to prevent file deletion
    let parent_dir = config_path.parent().unwrap();
    let mut perms = fs::metadata(parent_dir).unwrap().permissions();
    let original_mode = perms.mode();
    perms.set_mode(0o555); // Read and execute only, no write
    fs::set_permissions(parent_dir, perms).unwrap();

    let result = ConfigInstance::load_or_create(
        &config_path,
        "test-instance".to_string(),
        "test-app".to_string(),
    );

    // Restore permissions before asserting to ensure cleanup
    let mut perms = fs::metadata(parent_dir).unwrap().permissions();
    perms.set_mode(original_mode);
    fs::set_permissions(parent_dir, perms).unwrap();

    // Should fail because we cannot delete the invalid file
    assert!(
        result.is_err(),
        "Should fail when unable to delete invalid file"
    );
}
