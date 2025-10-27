//! Property-based tests for core library invariants.

// Allow clippy lints for property tests
#![allow(clippy::manual_range_contains)]

//! Property-based tests for core library invariants.

use genai_keyfinder_core::{
    models::{discovered_key::Confidence, DiscoveredKey, ValueType},
    providers::openai::OpenAIPlugin,
    ProviderPlugin,
};

use proptest::prelude::*;

proptest! {
    // Confidence scores must always be within [0.0, 1.0]
    #[test]
    fn confidence_score_within_bounds(key in ".*") {
        let plugin = OpenAIPlugin;
        let score = plugin.confidence_score(&key);
        prop_assert!(score >= 0.0 && score <= 1.0, "score {} out of bounds for key {}", score, key);
    }

    // Hash is deterministic with respect to the full_value content
    #[test]
    fn discovered_key_hash_consistency(val in ".*") {
        let k1 = DiscoveredKey::new(
            "test-provider".to_string(),
            "/tmp/source".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            val.clone(),
        );
        let k2 = DiscoveredKey::new(
            "test-provider".to_string(),
            "/tmp/source".to_string(),
            ValueType::ApiKey,
            Confidence::High,
            val,
        );
        prop_assert_eq!(k1.hash, k2.hash, "hash should be stable for identical values");
    }

    // Redaction should never expose full secrets; when value is long enough, it never returns the full string
    #[test]
    fn redaction_never_exposes_full_value(secret in ".{9,128}") {
        let key = DiscoveredKey::new(
            "test".to_string(),
            "/path".to_string(),
            ValueType::ApiKey,
            Confidence::Medium,
            secret.clone(),
        );
        let red = key.redacted_value();
        // For values >= 9, current impl returns either ab**** or ****last4
        // Ensure we never return the exact secret back
        prop_assert_ne!(red, secret, "redaction leaked full value");
    }
}
