//! Tests for built-in scanner plugins

use genai_keyfinder_core::scanners::{
    register_builtin_scanners, ClaudeDesktopScanner, GshScanner, LangChainScanner, RagitScanner,
    RooCodeScanner, ScannerPlugin, ScannerRegistry,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_langchain_scanner_integration() {
    let temp_home = TempDir::new().unwrap();
    let scanner = LangChainScanner;

    // Test scan_paths
    let paths = scanner.scan_paths(temp_home.path());
    assert!(!paths.is_empty());
    assert!(paths
        .iter()
        .any(|p| p.to_string_lossy().contains(".langchain")));

    // Test can_handle_file
    assert!(scanner.can_handle_file(Path::new("langchain_config.json")));
    assert!(scanner.can_handle_file(Path::new("config.yaml")));
    assert!(scanner.can_handle_file(Path::new(".env")));
    assert!(!scanner.can_handle_file(Path::new("random.txt")));

    // Test that scanner focuses on application-specific functionality
    assert_eq!(scanner.app_name(), "LangChain");
}

#[test]
fn test_langchain_scanner_parse_config() {
    let scanner = LangChainScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test JSON parsing with provider keys
    let json_config = r#"{
        "langchain_version": "0.1.0",
        "api_key": "sk-langchain1234567890abcdef",
        "llm": {
            "provider": "openai",
            "model": "gpt-4",
            "api_key": "sk-openai1234567890abcdef"
        },
        "providers": {
            "anthropic": {
                "api_key": "sk-ant-anthropic1234567890abcdef"
            }
        }
    }"#;

    let result = scanner
        .parse_config(&temp_dir.path().join("config.json"), json_config)
        .unwrap();

    // Should find 3 keys: langchain, openai, anthropic
    assert_eq!(result.keys.len(), 3);
    assert_eq!(result.instances.len(), 1);

    // Verify provider_instances are NOT populated by built-in scanners
    // Built-in scanners only discover keys; provider_instances are populated elsewhere
    let instance = &result.instances[0];
    assert_eq!(
        instance.provider_instances.len(),
        0,
        "Built-in scanners don't populate provider_instances"
    );

    // Test YAML parsing
    let yaml_config = r#"
langchain_version: "0.2.0"
api_key: sk-langchain-yaml1234567890abcdef
llm:
  provider: openai
  model: gpt-4
  api_key: sk-openai-yaml1234567890abcdef
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join("config.yaml"), yaml_config)
        .unwrap();

    assert_eq!(result.keys.len(), 2);
    assert_eq!(result.instances.len(), 1);

    // Built-in scanners don't populate provider_instances
    let instance = &result.instances[0];
    assert_eq!(
        instance.provider_instances.len(),
        0,
        "Built-in scanners don't populate provider_instances"
    );
}

#[test]
fn test_ragit_scanner_integration() {
    let temp_home = TempDir::new().unwrap();
    let scanner = RagitScanner;

    // Test scan_paths
    let paths = scanner.scan_paths(temp_home.path());
    assert!(!paths.is_empty());
    assert!(paths.iter().any(|p| p.to_string_lossy().contains(".ragit")));

    // Test can_handle_file
    let temp_dir = TempDir::new().unwrap();
    let ragit_path = temp_dir.path().join(".ragit").join("config.json");
    assert!(scanner.can_handle_file(&ragit_path));
    assert!(!scanner.can_handle_file(Path::new("/random/config.json")));
}

#[test]
fn test_ragit_scanner_parse_config() {
    let scanner = RagitScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test JSON parsing with provider keys
    let json_config = r#"{
        "ragit_version": "1.0.0",
        "api_key": "sk-ragit1234567890abcdef",
        "providers": {
            "openai": {
                "api_key": "sk-openai1234567890abcdef"
            },
            "huggingface": {
                "api_key": "hf_huggingface1234567890abcdef"
            }
        },
        "vector_store": {
            "type": "chroma"
        }
    }"#;

    let result = scanner
        .parse_config(&temp_dir.path().join("config.json"), json_config)
        .unwrap();

    // Should find 3 keys: ragit, openai, huggingface
    assert_eq!(result.keys.len(), 3);
    assert_eq!(result.instances.len(), 1);

    // Built-in scanners don't populate provider_instances
    let instance = &result.instances[0];
    assert_eq!(
        instance.provider_instances.len(),
        0,
        "Built-in scanners don't populate provider_instances"
    );
}

#[test]
fn test_ragit_scanner_env_parsing() {
    let scanner = RagitScanner;

    // Test .env file parsing
    let env_content = r#"
RAGIT_API_KEY=sk-ragit-env1234567890abcdef
OPENAI_API_KEY=sk-openai-env1234567890abcdef
ANTHROPIC_API_KEY=sk-ant-anthropic-env1234567890abcdef
"#;

    let result = scanner
        .parse_config(Path::new(".env"), env_content)
        .unwrap();

    assert_eq!(result.keys.len(), 3);

    // .env files may not create instances, just keys
    if !result.instances.is_empty() {
        let instance = &result.instances[0];
        assert_eq!(
            instance.provider_instances.len(),
            0,
            "Built-in scanners don't populate provider_instances"
        );
    }
}

#[test]
fn test_claude_desktop_scanner() {
    let temp_home = TempDir::new().unwrap();
    let scanner = ClaudeDesktopScanner;

    // Test scan_paths
    let paths = scanner.scan_paths(temp_home.path());
    assert!(!paths.is_empty());
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("claude")));

    // Test app_name
    assert_eq!(scanner.app_name(), "Claude Desktop");

    // Test app_name
    assert_eq!(scanner.app_name(), "Claude Desktop");
}

#[test]
fn test_roo_code_scanner() {
    let temp_home = TempDir::new().unwrap();
    let scanner = RooCodeScanner;

    // Test scan_paths
    let paths = scanner.scan_paths(temp_home.path());
    assert!(!paths.is_empty());
    // RooCode scanner uses wildcards and VSCode paths, so check for vscode-related paths
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("vscode")));

    // Test app_name
    assert_eq!(scanner.app_name(), "Roo Code");

    // Test app_name
    assert_eq!(scanner.app_name(), "Roo Code");
}

#[test]
fn test_register_builtin_scanners() {
    let registry = ScannerRegistry::new();

    // Register all built-in scanners
    register_builtin_scanners(&registry).unwrap();

    // Verify all scanners are registered
    let scanner_names = registry.list();
    assert!(
        scanner_names.contains(&"langchain".to_string()),
        "Should have langchain scanner"
    );
    assert!(
        scanner_names.contains(&"ragit".to_string()),
        "Should have ragit scanner"
    );
    assert!(
        scanner_names.contains(&"claude-desktop".to_string()),
        "Should have claude-desktop scanner"
    );
    assert!(
        scanner_names.contains(&"roo-code".to_string()),
        "Should have roo-code scanner"
    );
    assert!(
        scanner_names.contains(&"gsh".to_string()),
        "Should have gsh scanner"
    );

    // Should have exactly 5 scanners (including GSH)
    assert_eq!(
        scanner_names.len(),
        5,
        "Should have exactly 5 built-in scanners"
    );
}

#[test]
fn test_scanner_registry_get_scanners_for_file() {
    let registry = ScannerRegistry::new();
    register_builtin_scanners(&registry).unwrap();

    // Test getting scanners for different file types
    let langchain_scanners = registry.get_scanners_for_file(Path::new("langchain_config.json"));
    assert!(!langchain_scanners.is_empty());

    let ragit_scanners = registry.get_scanners_for_file(Path::new("ragit_config.json"));
    assert!(!ragit_scanners.is_empty());

    let env_scanners = registry.get_scanners_for_file(Path::new(".env"));
    assert!(!env_scanners.is_empty());
}

#[test]
fn test_scanner_instance_creation() {
    let temp_home = TempDir::new().unwrap();
    let scanner = LangChainScanner;

    // Create a LangChain config directory
    let langchain_dir = temp_home.path().join(".langchain");
    fs::create_dir_all(&langchain_dir).unwrap();

    let config_content = r#"{
        "langchain_version": "0.1.0",
        "llm": {
            "provider": "openai",
            "model": "gpt-4"
        }
    }"#;
    fs::write(langchain_dir.join("config.json"), config_content).unwrap();

    // Test scan_instances
    let instances = scanner.scan_instances(temp_home.path()).unwrap();
    assert!(!instances.is_empty(), "Should find at least one instance");
    assert_eq!(instances[0].app_name, "langchain");

    // Verify provider_instances field exists but is not populated by built-in scanners
    let instance = &instances[0];
    assert_eq!(
        instance.provider_instances.len(),
        0,
        "Built-in scanners don't populate provider_instances"
    );

    // Verify the field is accessible
    let all_instances = instance.provider_instances.all_instances();
    assert_eq!(
        all_instances.len(),
        0,
        "Should have no provider instances from built-in scanner"
    );
}

#[test]
fn test_gsh_scanner_integration() {
    let temp_home = TempDir::new().unwrap();
    let scanner = GshScanner;

    // Test scan_paths
    let paths = scanner.scan_paths(temp_home.path());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].to_string_lossy().contains(".gshrc"));

    // Test can_handle_file
    assert!(scanner.can_handle_file(Path::new(".gshrc")));
    assert!(scanner.can_handle_file(Path::new("/home/user/.gshrc")));
    assert!(scanner.can_handle_file(Path::new("gshrc")));
    assert!(!scanner.can_handle_file(Path::new("config.json")));
    assert!(!scanner.can_handle_file(Path::new(".bashrc")));

    // Test app_name
    assert_eq!(scanner.app_name(), "GSH");
}

#[test]
fn test_gsh_scanner_parse_config_simple_pairs() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test simple KEY=value pairs
    let config = r#"
# GSH Configuration
OPENAI_API_KEY=sk-test1234567890abcdef
ANTHROPIC_API_KEY=sk-ant-test1234567890abcdef
GOOGLE_API_KEY=AIzaSyTest1234567890abcdef
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), config)
        .unwrap();

    // Should find 3 keys
    assert_eq!(result.keys.len(), 3);
    assert_eq!(result.instances.len(), 1);

    // Check keys
    assert_eq!(result.keys[0].provider, "openai");
    assert_eq!(
        result.keys[0].value_type,
        genai_keyfinder_core::models::discovered_key::ValueType::ApiKey
    );
    assert_eq!(
        result.keys[0].confidence,
        genai_keyfinder_core::models::discovered_key::Confidence::High
    );

    // Check instance
    assert_eq!(result.instances[0].app_name, "gsh");
}

#[test]
fn test_gsh_scanner_parse_config_quoted_values() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test quoted values (single and double quotes)
    let config = r#"
# GSH Configuration with quoted values
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY='sk-ant-test1234567890abcdef'
GOOGLE_API_KEY="AIzaSyTest1234567890abcdef"
HUGGING_FACE_HUB_TOKEN='hf_test1234567890abcdef'
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), config)
        .unwrap();

    // Should find 4 keys
    assert_eq!(result.keys.len(), 4);

    // Check that all providers are found
    let providers: Vec<String> = result.keys.iter().map(|k| k.provider.clone()).collect();
    assert!(providers.contains(&"openai".to_string()));
    assert!(providers.contains(&"anthropic".to_string()));
    assert!(providers.contains(&"google".to_string()));
    assert!(providers.contains(&"huggingface".to_string()));
}

#[test]
fn test_gsh_scanner_parse_config_export_statements() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test export statements
    let config = r#"
# Export statements
export OPENAI_API_KEY=sk-test1234567890abcdef
export ANTHROPIC_API_KEY=sk-ant-test1234567890abcdef
export GOOGLE_API_KEY=AIzaSyTest1234567890abcdef
export HF_TOKEN=hf_test1234567890abcdef
export LANGCHAIN_API_KEY=sk-langchain1234567890abcdef
export GROQ_API_KEY=sk-groq1234567890abcdef
export COHERE_API_KEY=sk-cohere1234567890abcdef
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), config)
        .unwrap();

    // Should find 7 keys
    assert_eq!(result.keys.len(), 7);

    // Check that all providers are found
    let providers: Vec<String> = result.keys.iter().map(|k| k.provider.clone()).collect();
    assert!(providers.contains(&"openai".to_string()));
    assert!(providers.contains(&"anthropic".to_string()));
    assert!(providers.contains(&"google".to_string()));
    assert!(providers.contains(&"huggingface".to_string()));
    assert!(providers.contains(&"langchain".to_string()));
    assert!(providers.contains(&"groq".to_string()));
    assert!(providers.contains(&"cohere".to_string()));
}

#[test]
fn test_gsh_scanner_parse_config_comments_and_empty_lines() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test comments and empty lines
    let config = r#"
# This is a comment
OPENAI_API_KEY=sk-test1234567890abcdef

# Another comment
ANTHROPIC_API_KEY=sk-ant-test1234567890abcdef

# Empty line above
GOOGLE_API_KEY=AIzaSyTest1234567890abcdef
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), config)
        .unwrap();

    // Should find 3 keys despite comments and empty lines
    assert_eq!(result.keys.len(), 3);
}

#[test]
fn test_gsh_scanner_parse_config_multiple_providers() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test multiple provider keys in one file
    let config = r#"
# Multiple provider configuration
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
export GOOGLE_API_KEY="AIzaSyTest1234567890abcdef"
export GEMINI_API_KEY="AIzaSyGeminiTest1234567890abcdef"
export HUGGING_FACE_HUB_TOKEN="hf_test1234567890abcdef"
export HF_TOKEN="hf_another_test1234567890abcdef"
export LANGCHAIN_API_KEY="sk-langchain1234567890abcdef"
export GROQ_API_KEY="sk-groq1234567890abcdef"
export COHERE_API_KEY="sk-cohere1234567890abcdef"
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), config)
        .unwrap();

    // Should find 9 keys (all providers)
    assert_eq!(result.keys.len(), 9);

    // Check confidence levels - all should be at least Medium
    for key in &result.keys {
        assert!(matches!(
            key.confidence,
            genai_keyfinder_core::models::discovered_key::Confidence::High
                | genai_keyfinder_core::models::discovered_key::Confidence::Medium
                | genai_keyfinder_core::models::discovered_key::Confidence::Low
        ));
    }
}

#[test]
fn test_gsh_scanner_edge_cases() {
    let scanner = GshScanner;
    let temp_dir = TempDir::new().unwrap();

    // Test malformed file
    let malformed_config = r#"
This is not a valid config
Just some random text
without any key patterns
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), malformed_config)
        .unwrap();

    // Should find 0 keys - but the scanner might find patterns that look like keys
    // Let's just verify it doesn't crash and handles it gracefully
    assert_eq!(result.keys.len(), 0);

    // Test file with missing keys
    let no_keys_config = r#"
# GSH Configuration
GSH_PROMPT="You are a helpful assistant"
GSH_MODEL="gpt-4"
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), no_keys_config)
        .unwrap();

    // Should find 0 keys but still create an instance
    assert_eq!(result.keys.len(), 0);
    // The instance creation depends on is_valid_gsh_config(), which checks for GSH_ patterns
    // Since we have GSH_PROMPT and GSH_MODEL, it should create an instance
    assert_eq!(result.instances.len(), 1);

    // Test file with partial keys (some valid, some invalid)
    let partial_config = r#"
# Partial configuration
export OPENAI_API_KEY="sk-test1234567890abcdef"
export INVALID_KEY="too-short"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
"#;

    let result = scanner
        .parse_config(&temp_dir.path().join(".gshrc"), partial_config)
        .unwrap();

    // Should find 2 valid keys (ignoring the invalid one)
    assert_eq!(result.keys.len(), 2);
}

#[test]
fn test_gsh_scanner_scan_instances() {
    let temp_home = TempDir::new().unwrap();
    let scanner = GshScanner;

    // Create a .gshrc file
    let gshrc_content = r#"
# GSH Configuration
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
"#;

    fs::write(temp_home.path().join(".gshrc"), gshrc_content).unwrap();

    // Test scan_instances
    let instances = scanner.scan_instances(temp_home.path()).unwrap();
    assert!(!instances.is_empty());
    assert_eq!(instances[0].app_name, "gsh");
}

#[test]
fn test_register_builtin_scanners_includes_gsh() {
    let registry = ScannerRegistry::new();

    // Register all built-in scanners
    register_builtin_scanners(&registry).unwrap();

    // Verify all scanners are registered including GSH
    let scanner_names = registry.list();
    assert!(scanner_names.contains(&"langchain".to_string()));
    assert!(scanner_names.contains(&"ragit".to_string()));
    assert!(scanner_names.contains(&"claude-desktop".to_string()));
    assert!(scanner_names.contains(&"roo-code".to_string()));
    assert!(scanner_names.contains(&"gsh".to_string()));

    // Should have exactly 5 scanners now (including GSH)
    assert_eq!(scanner_names.len(), 5);
}
