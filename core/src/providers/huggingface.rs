//! Hugging Face provider plugin for scanning Hugging Face tokens and configuration.

use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, DiscoveredKey, ValueType};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning Hugging Face tokens and configuration files.
pub struct HuggingFacePlugin;

impl ProviderPlugin for HuggingFacePlugin {
    fn name(&self) -> &str {
        "huggingface"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Hugging Face tokens have very specific patterns
        if key.starts_with("hf_") {
            0.95 // Very distinctive Hugging Face prefix
        } else if key.len() >= 40 && key.contains('_') {
            0.70 // Might be a Hugging Face token without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }
}

impl HuggingFacePlugin {
    // ProviderPlugin no longer handles scanning or parsing
    // Only confidence scoring is used for key validation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;

    #[test]
    fn test_huggingface_plugin_name() {
        let plugin = HuggingFacePlugin;
        assert_eq!(plugin.name(), "huggingface");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = HuggingFacePlugin;

        assert_eq!(plugin.confidence_score("hf_1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("hf_1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random_key_with_underscores_123456789"),
            0.30
        );
    }
}
