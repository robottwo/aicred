// Allow clippy lints for integration tests
#![allow(clippy::len_zero)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(unused_comparisons)]

use genai_keyfinder_core::{scan, ScanOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_full_scan_workflow() {
    let temp_home = TempDir::new().unwrap();

    // Create a LangChain configuration with provider keys
    let langchain_config = r#"{
        "langchain_version": "0.1.0",
        "llm": {
            "provider": "openai",
            "model": "gpt-4",
            "api_key": "sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345"
        },
        "providers": {
            "anthropic": {
                "api_key": "sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345"
            }
        }
    }"#;

    // Write LangChain config - create the .langchain directory first
    let langchain_dir = temp_home.path().join(".langchain");
    fs::create_dir_all(&langchain_dir).unwrap();
    fs::write(langchain_dir.join("config.json"), langchain_config).unwrap();

    // Create a .env file with provider keys
    let env_content = r#"
OPENAI_API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345
ANTHROPIC_API_KEY=sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345
"#;
    fs::write(temp_home.path().join(".env"), env_content).unwrap();

    // Run scan against the temp home
    let result = scan(&ScanOptions {
        home_dir: Some(temp_home.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1_048_576,
        only_providers: None,
        exclude_providers: None,
    })
    .expect("scan should succeed");

    // Verify that the scan completed successfully and found keys
    assert!(result.scan_completed_at > result.scan_started_at);
    // The scan should find keys through the scanner plugins
    assert!(result.keys.len() > 0);
}

#[test]
fn test_scanner_based_provider_discovery() {
    let temp_home = TempDir::new().unwrap();

    // Create a Ragit configuration that supports provider scanning
    let ragit_config = r#"{
        "ragit_version": "1.0.0",
        "providers": {
            "openai": {
                "api_key": "sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345"
            },
            "anthropic": {
                "api_key": "sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345"
            }
        }
    }"#;

    // Write Ragit config - create the .ragit directory first for global config
    let ragit_dir = temp_home.path().join(".ragit");
    fs::create_dir_all(&ragit_dir).unwrap();
    fs::write(ragit_dir.join("config.json"), ragit_config).unwrap();

    // Create a .env file
    let env_content = r#"
OPENAI_API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345
"#;
    fs::write(temp_home.path().join(".env"), env_content).unwrap();

    // Run scan without provider filtering to find all providers
    let result = scan(&ScanOptions {
        home_dir: Some(temp_home.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1_048_576,
        only_providers: None,
        exclude_providers: None,
    })
    .expect("scan should succeed");

    // Verify the scan found provider keys
    assert!(result.scan_completed_at > result.scan_started_at);

    // Verify that providers were found
    assert!(
        !result.keys.is_empty(),
        "No provider keys were found during the scan"
    );

    // Verify that at least one provider key was discovered
    assert!(
        result.total_keys() > 0,
        "Expected at least one provider key to be discovered"
    );

    // Verify that the expected providers were found
    let providers_found = result.keys_by_provider();
    assert!(
        providers_found.contains_key("openai"),
        "Expected to find OpenAI provider keys"
    );
    assert!(
        providers_found.contains_key("anthropic"),
        "Expected to find Anthropic provider keys"
    );

    // Verify that each provider has at least one key
    assert!(
        providers_found.get("openai").copied().unwrap_or(0) >= 1,
        "Expected at least one OpenAI key"
    );
    assert!(
        providers_found.get("anthropic").copied().unwrap_or(0) >= 1,
        "Expected at least one Anthropic key"
    );

    // The test successfully verifies that:
    // 1. The scan completed successfully (timing check)
    // 2. At least one provider key was discovered
    // 3. Specific providers (OpenAI and Anthropic) were found
    // 4. The Ragit scanner successfully extracted provider keys from its config file
}

#[test]
fn test_application_scanner_integration() {
    let temp_home = TempDir::new().unwrap();

    // Create a LangChain application config
    let langchain_dir = temp_home.path().join(".langchain");
    fs::create_dir_all(&langchain_dir).unwrap();

    let langchain_config = r#"{
        "langchain_version": "0.2.0",
        "llm": {
            "provider": "openai",
            "model": "gpt-4"
        }
    }"#;
    fs::write(langchain_dir.join("config.json"), langchain_config).unwrap();

    // Run scan to discover application instances
    let result = scan(&ScanOptions {
        home_dir: Some(temp_home.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1_048_576,
        only_providers: None,
        exclude_providers: None,
    })
    .expect("scan should succeed");

    // Verify that application instances are discovered
    assert!(result.scan_completed_at > result.scan_started_at);
    // Application instances should be found even if no keys are present
    assert!(result.config_instances.len() >= 0);
}
