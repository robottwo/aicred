#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;
    use crate::models::discovered_key::{Confidence, ValueType};
    use crate::models::provider_key::{Environment, ValidationStatus};
    use chrono::Utc;

    #[test]
    fn test_provider_creation() {
        let provider = Provider::new(
            "OpenAI".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        )
        .with_description("OpenAI API provider".to_string());

        assert_eq!(provider.name, "OpenAI");
        assert_eq!(provider.provider_type, "openai");
        assert_eq!(provider.base_url, "https://api.openai.com");
        assert_eq!(
            provider.description,
            Some("OpenAI API provider".to_string())
        );
    }

    #[test]
    fn test_model_creation() {
        let model = Model::new("gpt-4".to_string(), "GPT-4".to_string()).with_context_window(8192);

        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
        assert_eq!(model.context_window, Some(8192));
    }

    #[test]
    fn test_discovered_key_redaction() {
        let key = DiscoveredKey::new(
            "OpenAI".to_string(),
            "/path/to/config".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "sk-1234567890abcdef1234567890abcdef".to_string(),
        );

        assert_eq!(key.redacted_value(), "****cdef");
    }

    #[test]
    fn test_scan_result_collection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        let mut result = ScanResult::new(
            home_path,
            vec!["openai".to_string(), "anthropic".to_string()],
            Utc::now(),
        );

        let key1 = DiscoveredKey::new(
            "OpenAI".to_string(),
            "/path/to/config1".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            "sk-123".to_string(),
        );

        let key2 = DiscoveredKey::new(
            "Anthropic".to_string(),
            "/path/to/config2".to_string(),
            ValueType::ApiKey,
            Confidence::Medium,
            "sk-ant-123".to_string(),
        );

        result.add_key(key1);
        result.add_key(key2);

        assert_eq!(result.keys.len(), 2);
        assert_eq!(result.filter_by_provider("OpenAI").len(), 1);
        assert_eq!(result.filter_by_confidence(Confidence::Medium).len(), 2);
    }

    // ========================================================================
    // ProviderConfig to ProviderInstance Conversion Tests
    // ========================================================================

    #[test]
    fn test_provider_config_to_instance_with_full_metadata() {
        // Create a ProviderConfig with a key containing all metadata fields
        let mut config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "test-key".to_string(),
            "/path/to/config.json".to_string(),
            Confidence::VeryHigh,
            Environment::Staging,
        )
        .with_value("sk-test-key-12345".to_string())
        .with_line_number(42)
        .with_metadata(serde_json::json!({
            "custom_field": "custom_value",
            "another_field": 123
        }));

        config.add_key(key);
        config.models = vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()];

        // Convert to ProviderInstance
        let instance: ProviderInstance = config.clone().into();

        // Verify API key is preserved
        assert_eq!(instance.api_key, Some("sk-test-key-12345".to_string()));
        assert!(instance.has_api_key());
        assert!(instance.has_non_empty_api_key());

        // Verify metadata fields are captured
        let metadata = &instance.metadata;
        assert_eq!(metadata.get("environment"), Some(&"staging".to_string()));
        assert_eq!(metadata.get("confidence"), Some(&"Very High".to_string()));
        assert_eq!(
            metadata.get("validation_status"),
            Some(&"Unknown".to_string())
        );
        assert_eq!(
            metadata.get("source"),
            Some(&"/path/to/config.json".to_string())
        );
        assert_eq!(metadata.get("line_number"), Some(&"42".to_string()));
        assert!(metadata.contains_key("discovered_at"));
        assert!(metadata.contains_key("key_metadata"));

        // Verify models are converted
        assert_eq!(instance.models.len(), 2);
        assert_eq!(instance.models[0], "gpt-4");
        assert_eq!(instance.models[1], "gpt-3.5-turbo");
    }

    #[test]
    fn test_provider_config_to_instance_with_missing_api_key() {
        // Create a ProviderConfig without any keys
        let mut config = ProviderConfig::new("1.0".to_string());
        config.models = vec!["claude-3-opus".to_string()];

        // Convert to ProviderInstance
        let instance: ProviderInstance = config.into();

        // Verify no API key is present
        assert!(instance.api_key.is_empty());
        assert!(!instance.has_api_key());
        assert!(!instance.has_non_empty_api_key());

        // Verify metadata is empty when no keys exist
        assert!(instance.metadata.is_empty());

        // Verify models are still converted
        assert_eq!(instance.models.len(), 1);
        assert_eq!(instance.models[0], "claude-3-opus");
    }

    #[test]
    fn test_provider_config_to_instance_with_empty_api_key() {
        // Create a ProviderConfig with an empty key value
        let mut config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "empty-key".to_string(),
            "/path/to/config.json".to_string(),
            Confidence::Low,
            Environment::Development,
        )
        .with_value(String::new());

        config.add_key(key);

        // Convert to ProviderInstance
        let instance: ProviderInstance = config.into();

        // Verify empty API key is preserved
        assert_eq!(instance.api_key, Some(String::new()));
        assert!(instance.has_api_key()); // has_api_key() only checks presence
        assert!(!instance.has_non_empty_api_key()); // has_non_empty_api_key() checks for non-empty
    }

    #[test]
    fn test_provider_config_to_instance_with_minimal_metadata() {
        // Create a ProviderConfig with a key that has no optional metadata
        let mut config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "minimal-key".to_string(),
            "/minimal/path".to_string(),
            Confidence::Medium,
            Environment::Production,
        )
        .with_value("sk-minimal-key".to_string());
        // Note: no line_number, no additional metadata

        config.add_key(key);

        // Convert to ProviderInstance
        let instance: ProviderInstance = config.into();

        // Verify API key is preserved
        assert_eq!(instance.api_key, Some("sk-minimal-key".to_string()));

        // Verify metadata is present with required fields
        let metadata = instance
            .metadata
            .as_ref()
            .expect("Metadata should be present");
        assert_eq!(metadata.get("environment"), Some(&"production".to_string()));
        assert_eq!(metadata.get("confidence"), Some(&"Medium".to_string()));
        assert_eq!(metadata.get("source"), Some(&"/minimal/path".to_string()));

        // Verify optional fields are not present
        assert!(!metadata.contains_key("line_number"));
        assert!(!metadata.contains_key("key_metadata"));
    }

    #[test]
    fn test_provider_config_to_instance_all_environments() {
        // Test conversion preserves all environment types
        let environments = vec![
            (Environment::Development, "development"),
            (Environment::Staging, "staging"),
            (Environment::Production, "production"),
            (Environment::Testing, "testing"),
            (Environment::Custom("custom-env".to_string()), "custom-env"),
        ];

        for (env, expected_str) in environments {
            let mut config = ProviderConfig::new("1.0".to_string());
            let key = ProviderKey::new(
                "test-key".to_string(),
                "/path".to_string(),
                Confidence::High,
                env,
            )
            .with_value("sk-test".to_string());

            config.add_key(key);
            let instance: ProviderInstance = config.into();

            let metadata = &instance.metadata;
            assert_eq!(metadata.get("environment"), Some(&expected_str.to_string()));
        }
    }

    #[test]
    fn test_provider_config_to_instance_all_confidence_levels() {
        // Test conversion preserves all confidence levels
        let confidence_levels = vec![
            (Confidence::Low, "Low"),
            (Confidence::Medium, "Medium"),
            (Confidence::High, "High"),
            (Confidence::VeryHigh, "Very High"),
        ];

        for (conf, expected_str) in confidence_levels {
            let mut config = ProviderConfig::new("1.0".to_string());
            let key = ProviderKey::new(
                "test-key".to_string(),
                "/path".to_string(),
                conf,
                Environment::Production,
            )
            .with_value("sk-test".to_string());

            config.add_key(key);
            let instance: ProviderInstance = config.into();

            let metadata = &instance.metadata;
            assert_eq!(metadata.get("confidence"), Some(&expected_str.to_string()));
        }
    }

    #[test]
    fn test_provider_config_to_instance_all_validation_statuses() {
        // Test conversion preserves all validation statuses
        let statuses = vec![
            (ValidationStatus::Unknown, "Unknown"),
            (ValidationStatus::Valid, "Valid"),
            (ValidationStatus::Invalid, "Invalid"),
            (ValidationStatus::Expired, "Expired"),
            (ValidationStatus::Revoked, "Revoked"),
            (ValidationStatus::RateLimited, "Rate Limited"),
        ];

        for (status, expected_str) in statuses {
            let mut config = ProviderConfig::new("1.0".to_string());
            let mut key = ProviderKey::new(
                "test-key".to_string(),
                "/path".to_string(),
                Confidence::High,
                Environment::Production,
            )
            .with_value("sk-test".to_string());

            key.validation_status = status;
            config.add_key(key);
            let instance: ProviderInstance = config.into();

            let metadata = &instance.metadata;
            assert_eq!(
                metadata.get("validation_status"),
                Some(&expected_str.to_string())
            );
        }
    }

    // ========================================================================
    // ProviderInstance to ProviderConfig Conversion Tests
    // ========================================================================

    #[test]
    fn test_provider_instance_to_config_with_full_metadata() {
        // Create a ProviderInstance with full metadata
        let mut instance = ProviderInstance::new_without_models(
            "test-instance".to_string(),
            "Test Instance".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        instance.set_api_key("sk-test-key-67890".to_string());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("environment".to_string(), "staging".to_string());
        metadata.insert("confidence".to_string(), "High".to_string());
        metadata.insert("validation_status".to_string(), "Valid".to_string());
        metadata.insert("source".to_string(), "/config/path.json".to_string());
        metadata.insert("line_number".to_string(), "100".to_string());
        metadata.insert("discovered_at".to_string(), Utc::now().to_rfc3339());
        metadata.insert(
            "key_metadata".to_string(),
            r#"{"test":"value"}"#.to_string(),
        );
        instance.metadata = Some(metadata);

        instance.add_model(Model::new("gpt-4".to_string(), "GPT-4".to_string()));

        // Convert to ProviderConfig
        let config: ProviderConfig = instance.into();

        // Verify key is restored
        assert_eq!(config.keys.len(), 1);
        let key = &config.keys[0];
        assert_eq!(key.value, Some("sk-test-key-67890".to_string()));
        assert_eq!(key.environment, Environment::Staging);
        assert_eq!(key.confidence, Confidence::High);
        assert_eq!(key.validation_status, ValidationStatus::Valid);
        assert_eq!(key.source, "/config/path.json");
        assert_eq!(key.line_number, Some(100));
        assert!(key.metadata.is_some());

        // Verify models are converted back
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0], "gpt-4");
    }

    #[test]
    fn test_provider_instance_to_config_without_api_key() {
        // Create a ProviderInstance without an API key
        let instance = ProviderInstance::new_without_models(
            "no-key-instance".to_string(),
            "No Key Instance".to_string(),
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        // Convert to ProviderConfig
        let config: ProviderConfig = instance.into();

        // Verify no keys are created
        assert_eq!(config.keys.len(), 0);
    }

    #[test]
    fn test_provider_instance_to_config_without_metadata() {
        // Create a ProviderInstance with API key but no metadata
        let mut instance = ProviderInstance::new_without_models(
            "simple-instance".to_string(),
            "Simple Instance".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        instance.set_api_key("sk-simple-key".to_string());

        // Convert to ProviderConfig
        let config: ProviderConfig = instance.into();

        // Verify key is created with defaults
        assert_eq!(config.keys.len(), 1);
        let key = &config.keys[0];
        assert_eq!(key.value, Some("sk-simple-key".to_string()));
        // Default values should be used
        assert_eq!(key.environment, Environment::Production);
        assert_eq!(key.confidence, Confidence::Medium);
        assert_eq!(key.source, "converted");
    }

    // ========================================================================
    // Round-Trip Conversion Tests
    // ========================================================================

    #[test]
    fn test_round_trip_conversion_preserves_metadata() {
        // Create a ProviderConfig with comprehensive metadata
        let mut original_config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "round-trip-key".to_string(),
            "/original/path.json".to_string(),
            Confidence::VeryHigh,
            Environment::Staging,
        )
        .with_value("sk-round-trip-key".to_string())
        .with_line_number(55)
        .with_metadata(serde_json::json!({
            "custom": "data",
            "number": 42
        }));

        original_config.add_key(key);
        original_config.models = vec!["model-1".to_string(), "model-2".to_string()];

        // Convert to ProviderInstance and back
        let instance: ProviderInstance = original_config.clone().into();
        let round_trip_config: ProviderConfig = instance.into();

        // Verify key metadata survived the round trip
        assert_eq!(round_trip_config.keys.len(), 1);
        let round_trip_key = &round_trip_config.keys[0];

        assert_eq!(round_trip_key.value, Some("sk-round-trip-key".to_string()));
        assert_eq!(round_trip_key.environment, Environment::Staging);
        assert_eq!(round_trip_key.confidence, Confidence::VeryHigh);
        assert_eq!(round_trip_key.source, "/original/path.json");
        assert_eq!(round_trip_key.line_number, Some(55));
        assert!(round_trip_key.metadata.is_some());

        // Verify models survived
        assert_eq!(round_trip_config.models.len(), 2);
        assert_eq!(round_trip_config.models[0], "model-1");
        assert_eq!(round_trip_config.models[1], "model-2");
    }

    #[test]
    fn test_round_trip_conversion_with_empty_key() {
        // Test that empty keys are handled correctly in round-trip
        let mut original_config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "empty-key".to_string(),
            "/path".to_string(),
            Confidence::Low,
            Environment::Development,
        )
        .with_value(String::new());

        original_config.add_key(key);

        // Convert to ProviderInstance and back
        let instance: ProviderInstance = original_config.into();
        let round_trip_config: ProviderConfig = instance.into();

        // Verify empty key is preserved
        assert_eq!(round_trip_config.keys.len(), 1);
        assert_eq!(round_trip_config.keys[0].value, Some(String::new()));
    }

    #[test]
    fn test_round_trip_conversion_with_custom_environment() {
        // Test that custom environments survive round-trip
        let mut original_config = ProviderConfig::new("1.0".to_string());

        let key = ProviderKey::new(
            "custom-env-key".to_string(),
            "/path".to_string(),
            Confidence::High,
            Environment::Custom("my-custom-env".to_string()),
        )
        .with_value("sk-custom".to_string());

        original_config.add_key(key);

        // Convert to ProviderInstance and back
        let instance: ProviderInstance = original_config.into();
        let round_trip_config: ProviderConfig = instance.into();

        // Verify custom environment is preserved
        assert_eq!(round_trip_config.keys.len(), 1);
        assert_eq!(
            round_trip_config.keys[0].environment,
            Environment::Custom("my-custom-env".to_string())
        );
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_provider_config_with_multiple_keys_uses_first() {
        // When ProviderConfig has multiple keys, only the first should be used
        let mut config = ProviderConfig::new("1.0".to_string());

        let key1 = ProviderKey::new(
            "first-key".to_string(),
            "/path1".to_string(),
            Confidence::High,
            Environment::Production,
        )
        .with_value("sk-first-key".to_string());

        let key2 = ProviderKey::new(
            "second-key".to_string(),
            "/path2".to_string(),
            Confidence::Medium,
            Environment::Development,
        )
        .with_value("sk-second-key".to_string());

        config.add_key(key1);
        config.add_key(key2);

        // Convert to ProviderInstance
        let instance: ProviderInstance = config.into();

        // Verify only the first key is used
        assert_eq!(instance.api_key, Some("sk-first-key".to_string()));

        let metadata = &instance.metadata;
        assert_eq!(metadata.get("environment"), Some(&"production".to_string()));
        assert_eq!(metadata.get("source"), Some(&"/path1".to_string()));
    }

    #[test]
    fn test_malformed_metadata_handling() {
        // Test that malformed metadata in instance doesn't crash conversion
        let mut instance = ProviderInstance::new_without_models(
            "malformed-instance".to_string(),
            "Malformed Instance".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        instance.set_api_key("sk-test".to_string());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("environment".to_string(), "invalid-env".to_string());
        metadata.insert("confidence".to_string(), "InvalidConfidence".to_string());
        metadata.insert("validation_status".to_string(), "InvalidStatus".to_string());
        metadata.insert("line_number".to_string(), "not-a-number".to_string());
        metadata.insert("discovered_at".to_string(), "invalid-date".to_string());
        metadata.insert("key_metadata".to_string(), "not-valid-json".to_string());
        instance.metadata = Some(metadata);

        // Convert to ProviderConfig - should not panic
        let config: ProviderConfig = instance.into();

        // Verify key is created with defaults for invalid values
        assert_eq!(config.keys.len(), 1);
        let key = &config.keys[0];
        assert_eq!(key.value, Some("sk-test".to_string()));
        // Invalid environment should be treated as custom
        assert_eq!(
            key.environment,
            Environment::Custom("invalid-env".to_string())
        );
        // Invalid confidence should default to Medium
        assert_eq!(key.confidence, Confidence::Medium);
        // Invalid validation status should default to Unknown
        assert_eq!(key.validation_status, ValidationStatus::Unknown);
        // Invalid line number should be None
        assert_eq!(key.line_number, None);
        // Invalid metadata should be None
        assert_eq!(key.metadata, None);
    }

    #[test]
    fn test_has_api_key_vs_has_non_empty_api_key() {
        // Test the distinction between has_api_key() and has_non_empty_api_key()
        let mut instance = ProviderInstance::new_without_models(
            "test".to_string(),
            "Test".to_string(),
            "openai".to_string(),
            "https://api.openai.com".to_string(),
        );

        // No API key
        assert!(!instance.has_api_key());
        assert!(!instance.has_non_empty_api_key());

        // Empty API key
        instance.set_api_key(String::new());
        assert!(instance.has_api_key()); // Present but empty
        assert!(!instance.has_non_empty_api_key()); // Empty

        // Non-empty API key
        instance.set_api_key("sk-test".to_string());
        assert!(instance.has_api_key());
        assert!(instance.has_non_empty_api_key());
    }
}
