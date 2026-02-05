//! Configuration validation functions for provider instances.
//!
//! This module provides validation functions for YAML configuration files.
//! It validates the structure and required fields of provider instance configurations.

use crate::models::{ProviderCollection, ProviderInstance};

/// Validates a single provider instance YAML configuration.
///
/// This function deserializes the YAML content and validates that it conforms
/// to the `ProviderInstance` structure with all required fields present and valid.
///
/// # Arguments
/// * `content` - The YAML content as a string slice
///
/// # Returns
/// * `Ok(())` if the YAML is valid and can be deserialized into a `ProviderInstance`
/// * `Err(String)` with a descriptive error message if validation fails
///
/// # Examples
/// ```
/// use aicred_core::models::config_validator::validate_provider_instance_yaml;
///
/// let yaml = r#"
/// id: openai-prod
/// display_name: OpenAI Production
/// provider_type: openai
/// base_url: https://api.openai.com
/// keys: []
/// models: []
/// active: true
/// created_at: 2024-01-01T00:00:00Z
/// updated_at: 2024-01-01T00:00:00Z
/// "#;
///
/// assert!(validate_provider_instance_yaml(yaml).is_ok());
/// ```
pub fn validate_provider_instance_yaml(content: &str) -> Result<(), String> {
    // Attempt to deserialize the YAML content
    let instance: ProviderInstance = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse YAML as ProviderInstance: {e}"))?;

    // Validate the deserialized instance
    instance.validate().map_err(|e| {
        format!(
            "ProviderInstance validation failed for '{}': {}",
            instance.id, e
        )
    })?;

    Ok(())
}

/// Validates a provider instances collection YAML configuration.
///
/// This function deserializes the YAML content and validates that it conforms
/// to the `ProviderCollection` structure, which is a collection of provider instances
/// stored as a flattened `HashMap`.
///
/// # Arguments
/// * `content` - The YAML content as a string slice
///
/// # Returns
/// * `Ok(())` if the YAML is valid and can be deserialized into a `ProviderCollection` collection
/// * `Err(String)` with a descriptive error message if validation fails
///
/// # Examples
/// ```
/// use aicred_core::models::config_validator::validate_provider_instances_yaml;
///
/// let yaml = r#"
/// openai-prod:
///   id: openai-prod
///   display_name: OpenAI Production
///   provider_type: openai
///   base_url: https://api.openai.com
///   keys: []
///   models: []
///   active: true
///   created_at: 2024-01-01T00:00:00Z
///   updated_at: 2024-01-01T00:00:00Z
/// anthropic-dev:
///   id: anthropic-dev
///   display_name: Anthropic Development
///   provider_type: anthropic
///   base_url: https://api.anthropic.com
///   keys: []
///   models: []
///   active: true
///   created_at: 2024-01-01T00:00:00Z
///   updated_at: 2024-01-01T00:00:00Z
/// "#;
///
/// assert!(validate_provider_instances_yaml(yaml).is_ok());
/// ```
pub fn validate_provider_instances_yaml(content: &str) -> Result<(), String> {
    // Attempt to deserialize the YAML content
    let instances: ProviderCollection = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse YAML as ProviderCollection: {e}"))?;

    // Validate each instance
    for instance in instances.list() {
        instance.validate()
            .map_err(|e| format!("Instance '{}' validation failed: {e}", instance.id))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_provider_instance_yaml_valid() {
        let yaml = r#"
id: openai-prod
provider_type: openai
base_url: https://api.openai.com
api_key: ""
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

        assert!(validate_provider_instance_yaml(yaml).is_ok());
    }

    #[test]
    fn test_validate_provider_instance_yaml_with_keys_and_models() {
        let yaml = r#"
id: openai-prod
provider_type: openai
base_url: https://api.openai.com
api_key: sk-test123
models:
  - gpt-4
  - gpt-3.5-turbo
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
        if let Err(e) = &result {
            eprintln!("Validation error: {e}");
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_provider_instance_yaml_missing_required_field() {
        let yaml = r"
display_name: OpenAI Production
provider_type: openai
base_url: https://api.openai.com
keys: []
models: []
active: true
created_at: 2024-01-01T00:00:00Z
updated_at: 2024-01-01T00:00:00Z
";

        let result = validate_provider_instance_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse YAML"));
    }

    #[test]
    fn test_validate_provider_instance_yaml_empty_id() {
        let yaml = r#"
id: ""
provider_type: openai
base_url: https://api.openai.com
api_key: ""
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
        assert!(result.is_err());
        let error = result.unwrap_err();
        // The error might be from YAML parsing or validation
        assert!(error.contains("empty") || error.contains("invalid"));
    }

    #[test]
    fn test_validate_provider_instance_yaml_invalid_yaml() {
        let yaml = r"
id: openai-prod
display_name: OpenAI Production
  invalid indentation
provider_type: openai
";

        let result = validate_provider_instance_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse YAML"));
    }

    #[test]
    fn test_validate_provider_instances_yaml_valid() {
        let yaml = r#"
openai-prod:
  id: openai-prod
  provider_type: openai
  base_url: https://api.openai.com
  api_key: ""
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
  api_key: ""
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

        assert!(validate_provider_instances_yaml(yaml).is_ok());
    }

    #[test]
    fn test_validate_provider_instances_yaml_empty_collection() {
        let yaml = "{}";
        assert!(validate_provider_instances_yaml(yaml).is_ok());
    }

    #[test]
    fn test_validate_provider_instances_yaml_invalid_instance() {
        let yaml = r#"
openai-prod:
  id: ""
  provider_type: openai
  base_url: https://api.openai.com
  api_key: ""
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

        let result = validate_provider_instances_yaml(yaml);
        assert!(result.is_err());
        let error = result.unwrap_err();
        // The error might be from YAML parsing or validation
        assert!(error.contains("empty") || error.contains("invalid"));
    }

    #[test]
    fn test_validate_provider_instances_yaml_multiple_invalid() {
        let yaml = r#"
openai-prod:
  id: ""
  provider_type: openai
  base_url: https://api.openai.com
  api_key: ""
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
  base_url: ""
  api_key: ""
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

        let result = validate_provider_instances_yaml(yaml);
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should contain errors for both instances
        assert!(error.contains("empty") || error.contains("invalid"));
    }

    #[test]
    fn test_validate_provider_instances_yaml_invalid_yaml() {
        let yaml = r"
openai-prod:
  id: openai-prod
    invalid indentation
  display_name: OpenAI Production
";

        let result = validate_provider_instances_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse YAML"));
    }
}
