//! AWS Bedrock provider plugin for scanning AWS Bedrock credentials.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning AWS Bedrock credentials and configuration files.
pub struct AwsBedrockPlugin;

#[async_trait::async_trait]
impl ProviderPlugin for AwsBedrockPlugin {
    fn name(&self) -> &'static str {
        "aws-bedrock"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // AWS access keys have specific patterns
        if key.starts_with("AKIA") && key.len() == 20 {
            0.95 // Very distinctive AWS access key pattern
        } else if key.len() == 40 && key.chars().all(|c| c.is_ascii_hexdigit()) {
            0.80 // Could be AWS secret access key
        } else if key.len() == 16 {
            0.60 // Could be session token
        } else {
            0.30 // Lower confidence for other patterns
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "AWS Bedrock base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid AWS Bedrock base URL patterns
        let is_valid_bedrock_url = instance.base_url.contains("bedrock.")
            || instance.base_url.contains("amazonaws.com");

        if !is_valid_bedrock_url {
            return Err(Error::PluginError(
                "Invalid AWS Bedrock base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        // AWS Bedrock requires credentials
        Ok(instance.has_non_empty_api_key())
    }

    async fn probe_models_async(
        &self,
        _api_key: &str,
        _base_url: Option<&str>,
    ) -> Result<Vec<crate::models::ModelMetadata>> {
        // Return known Bedrock models
        Ok(vec![
            crate::models::ModelMetadata {
                id: "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
                name: "Claude 3.5 Sonnet v2".to_string(),
                description: Some("Most capable Sonnet model on AWS Bedrock".to_string()),
                context_length: Some(200000),
                pricing: None,
                architecture: None,
                metadata: None,
            },
            crate::models::ModelMetadata {
                id: "anthropic.claude-3-5-haiku-20241022-v1:0".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                description: Some("Fast and cost-effective model".to_string()),
                context_length: Some(200000),
                pricing: None,
                architecture: None,
                metadata: None,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_bedrock_plugin_name() {
        let plugin = AwsBedrockPlugin;
        assert_eq!(plugin.name(), "aws-bedrock");
    }

    #[test]
    fn test_aws_bedrock_confidence_score() {
        let plugin = AwsBedrockPlugin;

        // Test AWS access key format
        let score1 = plugin.confidence_score("AKIAIOSFODNN7EXAMPLE");
        assert!(score1 > 0.9, "Expected score > 0.9, got {score1}");

        // Test AWS secret key format
        let score2 = plugin.confidence_score("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
        assert!(score2 > 0.7, "Expected score > 0.7, got {score2}");

        // Test generic key
        let score3 = plugin.confidence_score("sk-1234567890");
        assert!(score3 < 0.5, "Expected score < 0.5, got {score3}");
    }

    #[test]
    fn test_validate_instance_empty_url() {
        let plugin = AwsBedrockPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "aws-bedrock".to_string(),
            base_url: "".to_string(),
            api_key: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_instance_invalid_url() {
        let plugin = AwsBedrockPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "aws-bedrock".to_string(),
            base_url: "https://example.com".to_string(),
            api_key: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_validate_instance_valid() {
        let plugin = AwsBedrockPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "aws-bedrock".to_string(),
            base_url: "https://bedrock.us-east-1.amazonaws.com".to_string(),
            api_key: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.validate_instance(&instance);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_instance_configured_with_key() {
        let plugin = AwsBedrockPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "aws-bedrock".to_string(),
            base_url: "https://bedrock.us-east-1.amazonaws.com".to_string(),
            api_key: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_instance_configured_without_key() {
        let plugin = AwsBedrockPlugin;
        let instance = ProviderInstance {
            id: "test".to_string(),
            provider_type: "aws-bedrock".to_string(),
            base_url: "https://bedrock.us-east-1.amazonaws.com".to_string(),
            api_key: None,
            models: vec![],
            metadata: None,
        };

        let result = plugin.is_instance_configured(&instance);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
