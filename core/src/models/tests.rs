#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::models::discovered_key::{Confidence, ValueType};
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
        let model = Model::new(
            "gpt-4".to_string(),
            "openai-prod".to_string(),
            "GPT-4".to_string(),
        )
        .with_context_window(8192);

        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.provider_instance_id, "openai-prod");
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
}
