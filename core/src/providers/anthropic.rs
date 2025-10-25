//! Anthropic provider plugin for scanning Anthropic API keys and configuration.

use crate::error::{Error, Result};
use crate::models::discovered_key::{Confidence, DiscoveredKey, ValueType};
use crate::plugins::ProviderPlugin;
use std::path::{Path, PathBuf};

/// Plugin for scanning Anthropic API keys and configuration files.
pub struct AnthropicPlugin;

impl ProviderPlugin for AnthropicPlugin {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        // Anthropic keys have very specific patterns
        if key.starts_with("sk-ant-") {
            0.95 // Very distinctive Anthropic prefix
        } else if key.len() >= 40 && key.contains('-') {
            0.70 // Might be an Anthropic key without the prefix
        } else {
            0.30 // Lower confidence for other patterns
        }
    }
}

impl AnthropicPlugin {
    // ProviderPlugin no longer handles scanning or parsing
    // Only confidence scoring is used for key validation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;

    #[test]
    fn test_anthropic_plugin_name() {
        let plugin = AnthropicPlugin;
        assert_eq!(plugin.name(), "anthropic");
    }

    #[test]
    fn test_confidence_scoring() {
        let plugin = AnthropicPlugin;

        assert_eq!(plugin.confidence_score("sk-ant-1234567890abcdef"), 0.95);
        assert_eq!(
            plugin.confidence_score("sk-ant-1234567890abcdef1234567890abcdef"),
            0.95
        );
        assert_eq!(
            plugin.confidence_score("random-key-with-dashes-123456789"),
            0.30
        );
    }
}
