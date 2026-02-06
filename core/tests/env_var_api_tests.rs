#![allow(deprecated)]
#![allow(unused_must_use)]
//! Tests for the new environment variable API extension for ScannerPlugin

use aicred_core::scanners::{
    ClaudeDesktopScanner, GshScanner, LangChainScanner, RagitScanner, RooCodeScanner,
};
use aicred_core::scanners::{EnvVarDeclaration, LabelMapping, ScannerPlugin};

#[test]
fn test_gsh_scanner_env_var_schema() {
    let scanner = GshScanner;
    let env_vars = scanner.get_env_var_schema();

    // GshScanner should return exactly 6 environment variables
    assert_eq!(
        env_vars.len(),
        6,
        "GshScanner should return 6 environment variables"
    );

    // Check for specific required environment variables
    let api_key_names: Vec<String> = env_vars.iter().map(|v| v.name.clone()).collect();

    assert!(api_key_names.contains(&"GSH_FAST_MODEL_API_KEY".to_string()));
    assert!(api_key_names.contains(&"GSH_FAST_MODEL_BASE_URL".to_string()));
    assert!(api_key_names.contains(&"GSH_FAST_MODEL_ID".to_string()));
    assert!(api_key_names.contains(&"GSH_SLOW_MODEL_API_KEY".to_string()));
    assert!(api_key_names.contains(&"GSH_SLOW_MODEL_BASE_URL".to_string()));
    assert!(api_key_names.contains(&"GSH_SLOW_MODEL_ID".to_string()));

    // Check GSH_FAST_MODEL_API_KEY - required
    let fast_api_key = env_vars
        .iter()
        .find(|v| v.name == "GSH_FAST_MODEL_API_KEY")
        .unwrap();
    assert!(
        fast_api_key.required,
        "GSH_FAST_MODEL_API_KEY should be required"
    );
    assert_eq!(fast_api_key.value_type, "string");
    assert_eq!(fast_api_key.default_value, None);

    // Check GSH_FAST_MODEL_BASE_URL - optional with default
    let fast_base_url = env_vars
        .iter()
        .find(|v| v.name == "GSH_FAST_MODEL_BASE_URL")
        .unwrap();
    assert!(
        !fast_base_url.required,
        "GSH_FAST_MODEL_BASE_URL should be optional"
    );
    assert_eq!(fast_base_url.value_type, "string");
    assert_eq!(
        fast_base_url.default_value,
        Some("https://api.groq.com/openai/v1".to_string())
    );

    // Check GSH_FAST_MODEL_ID - optional with default
    let fast_model_id = env_vars
        .iter()
        .find(|v| v.name == "GSH_FAST_MODEL_ID")
        .unwrap();
    assert!(
        !fast_model_id.required,
        "GSH_FAST_MODEL_ID should be optional"
    );
    assert_eq!(fast_model_id.value_type, "string");
    assert_eq!(
        fast_model_id.default_value,
        Some("llama3-70b-8192".to_string())
    );

    // Check GSH_SLOW_MODEL_API_KEY - required
    let slow_api_key = env_vars
        .iter()
        .find(|v| v.name == "GSH_SLOW_MODEL_API_KEY")
        .unwrap();
    assert!(
        slow_api_key.required,
        "GSH_SLOW_MODEL_API_KEY should be required"
    );
    assert_eq!(slow_api_key.value_type, "string");
    assert_eq!(slow_api_key.default_value, None);

    // Check GSH_SLOW_MODEL_BASE_URL - optional with default
    let slow_base_url = env_vars
        .iter()
        .find(|v| v.name == "GSH_SLOW_MODEL_BASE_URL")
        .unwrap();
    assert!(
        !slow_base_url.required,
        "GSH_SLOW_MODEL_BASE_URL should be optional"
    );
    assert_eq!(slow_base_url.value_type, "string");
    assert_eq!(
        slow_base_url.default_value,
        Some("https://openrouter.ai/api/v1".to_string())
    );

    // Check GSH_SLOW_MODEL_ID - optional with default
    let slow_model_id = env_vars
        .iter()
        .find(|v| v.name == "GSH_SLOW_MODEL_ID")
        .unwrap();
    assert!(
        !slow_model_id.required,
        "GSH_SLOW_MODEL_ID should be optional"
    );
    assert_eq!(slow_model_id.value_type, "string");
    assert_eq!(
        slow_model_id.default_value,
        Some("anthropic/claude-3-opus".to_string())
    );
}

#[test]
fn test_gsh_scanner_label_mappings() {
    let scanner = GshScanner;
    let label_mappings = scanner.get_label_mappings();

    // GshScanner should return exactly 2 label mappings
    assert_eq!(
        label_mappings.len(),
        2,
        "GshScanner should return 2 label mappings"
    );

    // Check for specific label mappings
    let label_names: Vec<String> = label_mappings
        .iter()
        .map(|l| l.label_name.clone())
        .collect();
    assert!(
        label_names.contains(&"fast".to_string()),
        "Should have 'fast' label"
    );
    assert!(
        label_names.contains(&"smart".to_string()),
        "Should have 'smart' label"
    );

    // Check the "fast" mapping details
    let fast_mapping = label_mappings
        .iter()
        .find(|l| l.label_name == "fast")
        .unwrap();
    assert_eq!(
        fast_mapping.env_var_group, "GSH_FAST_MODEL",
        "Fast label should map to GSH_FAST_MODEL"
    );
    assert_eq!(
        fast_mapping.description,
        "Fast model configuration for quick responses"
    );

    // Check the "smart" mapping details
    let smart_mapping = label_mappings
        .iter()
        .find(|l| l.label_name == "smart")
        .unwrap();
    assert_eq!(
        smart_mapping.env_var_group, "GSH_SLOW_MODEL",
        "Smart label should map to GSH_SLOW_MODEL"
    );
    assert_eq!(
        smart_mapping.description,
        "Smart model configuration for complex reasoning"
    );
}

#[test]
fn test_roo_code_scanner_env_var_schema() {
    let scanner = RooCodeScanner;
    let env_vars = scanner.get_env_var_schema();

    // RooCodeScanner should return exactly 3 environment variables
    assert_eq!(
        env_vars.len(),
        3,
        "RooCodeScanner should return 3 environment variables"
    );

    let api_key_names: Vec<String> = env_vars.iter().map(|v| v.name.clone()).collect();

    // Check for ROOCODE_API_KEY (note: implementation uses ROOCODE not ROO_CODE)
    assert!(
        api_key_names.contains(&"ROOCODE_API_KEY".to_string()),
        "Should have ROOCODE_API_KEY"
    );
    assert!(
        api_key_names.contains(&"ROOCODE_BASE_URL".to_string()),
        "Should have ROOCODE_BASE_URL"
    );
    assert!(
        api_key_names.contains(&"ROOCODE_MODEL_ID".to_string()),
        "Should have ROOCODE_MODEL_ID"
    );

    // Check ROOCODE_API_KEY - required
    let api_key = env_vars
        .iter()
        .find(|v| v.name == "ROOCODE_API_KEY")
        .unwrap();
    assert!(api_key.required, "ROOCODE_API_KEY should be required");
    assert_eq!(api_key.value_type, "ApiKey");
    assert_eq!(api_key.default_value, None);

    // Check ROOCODE_BASE_URL - optional with default
    let base_url = env_vars
        .iter()
        .find(|v| v.name == "ROOCODE_BASE_URL")
        .unwrap();
    assert!(!base_url.required, "ROOCODE_BASE_URL should be optional");
    assert_eq!(base_url.value_type, "BaseUrl");
    assert_eq!(
        base_url.default_value,
        Some("https://api.roocode.com/v1".to_string())
    );

    // Check ROOCODE_MODEL_ID - optional with default
    let model_id = env_vars
        .iter()
        .find(|v| v.name == "ROOCODE_MODEL_ID")
        .unwrap();
    assert!(!model_id.required, "ROOCODE_MODEL_ID should be optional");
    assert_eq!(model_id.value_type, "ModelId");
    assert_eq!(model_id.default_value, Some("roocode-70b".to_string()));
}

#[test]
fn test_roo_code_scanner_label_mappings() {
    let scanner = RooCodeScanner;
    let label_mappings = scanner.get_label_mappings();

    // RooCodeScanner should return empty label mappings
    assert_eq!(
        label_mappings.len(),
        0,
        "RooCodeScanner should return empty label mappings"
    );
}

#[test]
fn test_claude_desktop_scanner_env_var_schema() {
    let scanner = ClaudeDesktopScanner;
    let env_vars = scanner.get_env_var_schema();

    // ClaudeDesktopScanner should return exactly 3 environment variables
    assert_eq!(
        env_vars.len(),
        3,
        "ClaudeDesktopScanner should return 3 environment variables"
    );

    let api_key_names: Vec<String> = env_vars.iter().map(|v| v.name.clone()).collect();
    assert!(
        api_key_names.contains(&"CLAUDE_DESKTOP_API_KEY".to_string()),
        "Should have CLAUDE_DESKTOP_API_KEY"
    );
    assert!(
        api_key_names.contains(&"CLAUDE_DESKTOP_BASE_URL".to_string()),
        "Should have CLAUDE_DESKTOP_BASE_URL"
    );
    assert!(
        api_key_names.contains(&"CLAUDE_DESKTOP_MODEL_ID".to_string()),
        "Should have CLAUDE_DESKTOP_MODEL_ID"
    );

    // Check CLAUDE_DESKTOP_API_KEY - required
    let api_key = env_vars
        .iter()
        .find(|v| v.name == "CLAUDE_DESKTOP_API_KEY")
        .unwrap();
    assert!(
        api_key.required,
        "CLAUDE_DESKTOP_API_KEY should be required"
    );
    assert_eq!(api_key.value_type, "ApiKey");
    assert_eq!(api_key.default_value, None);

    // Check CLAUDE_DESKTOP_BASE_URL - optional with default
    let base_url = env_vars
        .iter()
        .find(|v| v.name == "CLAUDE_DESKTOP_BASE_URL")
        .unwrap();
    assert!(
        !base_url.required,
        "CLAUDE_DESKTOP_BASE_URL should be optional"
    );
    assert_eq!(base_url.value_type, "BaseUrl");
    assert_eq!(
        base_url.default_value,
        Some("https://api.anthropic.com/v1".to_string())
    );

    // Check CLAUDE_DESKTOP_MODEL_ID - optional with default
    let model_id = env_vars
        .iter()
        .find(|v| v.name == "CLAUDE_DESKTOP_MODEL_ID")
        .unwrap();
    assert!(
        !model_id.required,
        "CLAUDE_DESKTOP_MODEL_ID should be optional"
    );
    assert_eq!(model_id.value_type, "ModelId");
    assert_eq!(
        model_id.default_value,
        Some("claude-3-opus-20240229".to_string())
    );
}

#[test]
fn test_claude_desktop_scanner_label_mappings() {
    let scanner = ClaudeDesktopScanner;
    let label_mappings = scanner.get_label_mappings();

    // ClaudeDesktopScanner should return empty label mappings
    assert_eq!(
        label_mappings.len(),
        0,
        "ClaudeDesktopScanner should return empty label mappings"
    );
}

#[test]
fn test_ragit_scanner_env_var_schema() {
    let scanner = RagitScanner;
    let env_vars = scanner.get_env_var_schema();

    // RagitScanner should return exactly 3 environment variables
    assert_eq!(
        env_vars.len(),
        3,
        "RagitScanner should return 3 environment variables"
    );

    let api_key_names: Vec<String> = env_vars.iter().map(|v| v.name.clone()).collect();
    assert!(
        api_key_names.contains(&"RAGIT_API_KEY".to_string()),
        "Should have RAGIT_API_KEY"
    );
    assert!(
        api_key_names.contains(&"RAGIT_BASE_URL".to_string()),
        "Should have RAGIT_BASE_URL"
    );
    assert!(
        api_key_names.contains(&"RAGIT_MODEL_ID".to_string()),
        "Should have RAGIT_MODEL_ID"
    );

    // Check RAGIT_API_KEY - required
    let api_key = env_vars.iter().find(|v| v.name == "RAGIT_API_KEY").unwrap();
    assert!(api_key.required, "RAGIT_API_KEY should be required");
    assert_eq!(api_key.value_type, "ApiKey");
    assert_eq!(api_key.default_value, None);

    // Check RAGIT_BASE_URL - optional with default
    let base_url = env_vars
        .iter()
        .find(|v| v.name == "RAGIT_BASE_URL")
        .unwrap();
    assert!(!base_url.required, "RAGIT_BASE_URL should be optional");
    assert_eq!(base_url.value_type, "BaseUrl");
    assert_eq!(
        base_url.default_value,
        Some("https://api.ragit.ai/v1".to_string())
    );

    // Check RAGIT_MODEL_ID - optional with default
    let model_id = env_vars
        .iter()
        .find(|v| v.name == "RAGIT_MODEL_ID")
        .unwrap();
    assert!(!model_id.required, "RAGIT_MODEL_ID should be optional");
    assert_eq!(model_id.value_type, "ModelId");
    assert_eq!(model_id.default_value, Some("ragit-70b".to_string()));
}

#[test]
fn test_ragit_scanner_label_mappings() {
    let scanner = RagitScanner;
    let label_mappings = scanner.get_label_mappings();

    // RagitScanner should return empty label mappings
    assert_eq!(
        label_mappings.len(),
        0,
        "RagitScanner should return empty label mappings"
    );
}

#[test]
fn test_langchain_scanner_env_var_schema() {
    let scanner = LangChainScanner;
    let env_vars = scanner.get_env_var_schema();

    // LangChainScanner should return exactly 3 environment variables
    assert_eq!(
        env_vars.len(),
        3,
        "LangChainScanner should return 3 environment variables"
    );

    let api_key_names: Vec<String> = env_vars.iter().map(|v| v.name.clone()).collect();
    assert!(
        api_key_names.contains(&"LANGCHAIN_API_KEY".to_string()),
        "Should have LANGCHAIN_API_KEY"
    );
    assert!(
        api_key_names.contains(&"LANGCHAIN_BASE_URL".to_string()),
        "Should have LANGCHAIN_BASE_URL"
    );
    assert!(
        api_key_names.contains(&"LANGCHAIN_MODEL_ID".to_string()),
        "Should have LANGCHAIN_MODEL_ID"
    );

    // Check LANGCHAIN_API_KEY - required
    let api_key = env_vars
        .iter()
        .find(|v| v.name == "LANGCHAIN_API_KEY")
        .unwrap();
    assert!(api_key.required, "LANGCHAIN_API_KEY should be required");
    assert_eq!(api_key.value_type, "ApiKey");
    assert_eq!(api_key.default_value, None);

    // Check LANGCHAIN_BASE_URL - optional with default
    let base_url = env_vars
        .iter()
        .find(|v| v.name == "LANGCHAIN_BASE_URL")
        .unwrap();
    assert!(!base_url.required, "LANGCHAIN_BASE_URL should be optional");
    assert_eq!(base_url.value_type, "BaseUrl");
    assert_eq!(
        base_url.default_value,
        Some("https://api.langchain.com/v1".to_string())
    );

    // Check LANGCHAIN_MODEL_ID - optional with default
    let model_id = env_vars
        .iter()
        .find(|v| v.name == "LANGCHAIN_MODEL_ID")
        .unwrap();
    assert!(!model_id.required, "LANGCHAIN_MODEL_ID should be optional");
    assert_eq!(model_id.value_type, "ModelId");
    assert_eq!(model_id.default_value, Some("langchain-70b".to_string()));
}

#[test]
fn test_langchain_scanner_label_mappings() {
    let scanner = LangChainScanner;
    let label_mappings = scanner.get_label_mappings();

    // LangChainScanner should return empty label mappings
    assert_eq!(
        label_mappings.len(),
        0,
        "LangChainScanner should return empty label mappings"
    );
}

#[test]
fn test_env_var_declaration_construction() {
    // Test required constructor
    let required_var = EnvVarDeclaration::required(
        "TEST_API_KEY".to_string(),
        "Test API key".to_string(),
        "string".to_string(),
    );

    assert_eq!(required_var.name, "TEST_API_KEY");
    assert_eq!(required_var.description, "Test API key");
    assert_eq!(required_var.value_type, "string");
    assert!(required_var.required);
    assert_eq!(required_var.default_value, None);

    // Test optional constructor
    let optional_var = EnvVarDeclaration::optional(
        "TEST_OPTIONAL".to_string(),
        "Optional test".to_string(),
        "number".to_string(),
        Some("42".to_string()),
    );

    assert_eq!(optional_var.name, "TEST_OPTIONAL");
    assert!(!optional_var.required);
    assert_eq!(optional_var.default_value, Some("42".to_string()));
}

#[test]
fn test_label_mapping_construction() {
    let mapping = LabelMapping::new(
        "test_label".to_string(),
        "TEST_GROUP".to_string(),
        "Test description".to_string(),
    );

    assert_eq!(mapping.label_name, "test_label");
    assert_eq!(mapping.env_var_group, "TEST_GROUP");
    assert_eq!(mapping.description, "Test description");
}

#[test]
fn test_backward_compatibility() {
    // Test that ScannerPlugin trait has default implementations
    struct TestScanner;

    impl ScannerPlugin for TestScanner {
        fn name(&self) -> &'static str {
            "test"
        }

        fn app_name(&self) -> &'static str {
            "Test"
        }

        fn scan_paths(&self, _home_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
            Vec::new()
        }

        fn can_handle_file(&self, _path: &std::path::Path) -> bool {
            false
        }

        fn parse_config(
            &self,
            _path: &std::path::Path,
            _content: &str,
        ) -> aicred_core::error::Result<aicred_core::scanners::ScanResult> {
            let result = aicred_core::scanners::ScanResult::new();
            Ok(result)
        }
    }

    let scanner = TestScanner;

    // These should return empty vectors by default (backward compatibility)
    let env_vars = scanner.get_env_var_schema();
    assert!(env_vars.is_empty());

    let label_mappings = scanner.get_label_mappings();
    assert!(label_mappings.is_empty());
}

#[test]
fn test_env_var_types() {
    let scanner = GshScanner;
    let env_vars = scanner.get_env_var_schema();

    // Check that all environment variables have the correct type
    for env_var in env_vars {
        assert!(!env_var.name.is_empty());
        assert!(!env_var.description.is_empty());
        assert!(!env_var.value_type.is_empty());

        // All API keys should be strings
        if env_var.name.contains("API_KEY") {
            assert_eq!(env_var.value_type, "string");
        }
    }
}

#[test]
fn test_label_mapping_consistency() {
    let scanner = GshScanner;
    let label_mappings = scanner.get_label_mappings();

    // Check that all label mappings have consistent naming
    for mapping in label_mappings {
        assert!(!mapping.label_name.is_empty());
        assert!(!mapping.env_var_group.is_empty());
        assert!(!mapping.description.is_empty());

        // Label names should be lowercase
        assert_eq!(mapping.label_name.to_lowercase(), mapping.label_name);

        // Env var groups should be uppercase
        assert_eq!(mapping.env_var_group.to_uppercase(), mapping.env_var_group);
    }
}
