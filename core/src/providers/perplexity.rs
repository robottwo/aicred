//! Perplexity AI provider plugin for scanning Perplexity API keys.

use crate::error::{Error, Result};
use crate::models::ProviderInstance;
use crate::plugins::ProviderPlugin;

/// Plugin for scanning Perplexity AI API keys and configuration files.
pub struct PerplexityPlugin;

impl ProviderPlugin for PerplexityPlugin {
    fn name(&self) -> &str {
        "perplexity"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Perplexity keys start with "pplx-"
        if key.starts_with("pplx-") && key.len() >= 30 {
            0.95
        } else if key.starts_with("pplx") && key.len() >= 20 {
            0.75
        } else {
            0.35
        }
    }

    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
        if instance.base_url.is_empty() {
            return Err(Error::PluginError(
                "Perplexity base URL cannot be empty".to_string(),
            ));
        }

        // Check for valid Perplexity base URL patterns
        let is_valid_url = instance.base_url.contains("perplexity.ai")
            || instance.base_url.contains("api.perplexity");

        if !is_valid_url {
            return Err(Error::PluginError(
                "Invalid Perplexity base URL".to_string(),
            ));
        }

        Ok(())
    }

    fn get_instance_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        if !instance.models.is_empty() {
            return Ok(instance.models.iter().map(|m| m.model_id.clone()).collect());
        }

        // Default Perplexity models
        Ok(vec![
            "llama-3.1-sonar-small-128k-online".to_string(),
            "llama-3.1-sonar-large-128k-online".to_string(),
            "llama-3.1-sonar-huge-128k-online".to_string(),
        ])
    }

    fn is_instance_configured(&self, instance: &ProviderInstance) -> Result<bool> {
        Ok(instance.has_non_empty_api_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perplexity_plugin_name() {
        let plugin = PerplexityPlugin;
        assert_eq!(plugin.name(), "perplexity");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = PerplexityPlugin;
        assert_eq!(plugin.confidence_score("pplx-1234567890abcdef1234567890abcdef"), 0.95);
        assert_eq!(plugin.confidence_score("pplx1234567890abcdef123456"), 0.75);
        assert_eq!(plugin.confidence_score("short"), 0.35);
    }
}
