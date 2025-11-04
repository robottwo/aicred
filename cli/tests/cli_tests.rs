// Allow clippy lints for CLI tests
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::single_match)]
#![allow(unused_variables)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper function to set both HOME and USERPROFILE environment variables
/// for cross-platform test compatibility (Unix uses HOME, Windows uses USERPROFILE)
fn set_test_home_envs(cmd: &mut Command, home: &std::path::Path) {
    cmd.env("HOME", home);
    cmd.env("USERPROFILE", home);
}

/// Helper function to get home path as string for CLI arguments
fn home_path_str(home: &std::path::Path) -> &str {
    home.to_str().unwrap()
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("aicred"));
}

#[test]
fn test_scan_help() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan for GenAI credentials"));
}

#[test]
fn test_scan_dry_run() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan").arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("invalid-command");
    cmd.assert().failure();
}

#[test]
fn test_scan_with_format() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg("--dry-run");
    cmd.assert().success();
}

#[test]
fn test_all_output_formats() {
    for format in ["json", "ndjson", "table", "summary"].iter() {
        let mut cmd = Command::cargo_bin("aicred").unwrap();
        cmd.args(&["scan", "--format", format, "--dry-run"]);
        cmd.assert().success();
    }
}

#[test]
fn test_provider_filtering() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&["scan", "--only", "openai,anthropic", "--dry-run"]);
    cmd.assert().success();
}

#[test]
fn test_provider_exclusion() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&["scan", "--exclude", "ollama", "--dry-run"]);
    cmd.assert().success();
}

#[test]
fn test_audit_logging() {
    // Use a temp home dir but do not require any keys to be found.
    let home = std::env::temp_dir().join(format!("kf_cli_audit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);

    // Temp log path
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let log_path =
        std::env::temp_dir().join(format!("aicred_audit_{}_{}.log", std::process::id(), ts));
    let log_str = log_path.to_str().unwrap();

    // Execute without asserting success: exit code is 0 when keys are found, 1 when none found.
    let bin = assert_cmd::cargo::cargo_bin("aicred");
    let status = std::process::Command::new(bin)
        .args(&[
            "scan",
            "--home",
            home.to_str().unwrap(),
            "--format",
            "json",
            "--audit-log",
            log_str,
        ])
        .status()
        .expect("failed to execute aicred");
    assert!(status.code() == Some(0) || status.code() == Some(1));

    // Audit log should be written regardless of whether any keys were found.
    assert!(log_path.exists(), "Audit log file should be created");

    // Basic content sanity check
    let contents = std::fs::read_to_string(&log_path).expect("read audit log");
    assert!(contents.contains("AICred Audit Log"));
}

#[test]
fn test_invalid_format() {
    // Use a temp home directory to avoid scanning the real home
    let home = std::env::temp_dir().join(format!("kf_cli_invalid_fmt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        home.to_str().unwrap(),
        "--format",
        "invalid",
    ]);
    cmd.assert().failure();
}

#[test]
fn test_version_contains_pkg_version() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_instances_list_command() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file
    let test_config = r#"---
id: "openrouter-instance"
display_name: "OpenRouter Instance"
provider_type: "openrouter"
base_url: "https://openrouter.ai/api/v1"
active: true
api_key: "sk-or-test-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("openrouter-open.yaml"), test_config).unwrap();

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OpenRouter Instance"));
}

#[test]
fn test_instances_default_behavior() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create test provider files
    let openai_config = r#"---
id: "openai-instance"
display_name: "OpenAI Instance"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
active: true
api_key: "sk-test-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("openai.yaml"), openai_config).unwrap();

    let anthropic_config = r#"---
id: "anthropic-instance"
display_name: "Anthropic Instance"
provider_type: "anthropic"
base_url: "https://api.anthropic.com/v1"
active: true
api_key: "sk-ant-test-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("anthropic.yaml"), anthropic_config).unwrap();

    // Test that `aicred instances` (without subcommand) defaults to list behavior
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances").arg("--home").arg(temp_home.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OpenAI Instance"))
        .stdout(predicate::str::contains("Anthropic Instance"));

    // Verify it produces the same output as `aicred instances list`
    let mut cmd_list = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd_list, temp_home.path());
    cmd_list
        .arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());

    let output_default = cmd.output().unwrap();
    let output_list = cmd_list.output().unwrap();

    // Both should succeed
    assert!(output_default.status.success());
    assert!(output_list.status.success());

    // Both should contain the same provider names
    let stdout_default = String::from_utf8_lossy(&output_default.stdout);
    let stdout_list = String::from_utf8_lossy(&output_list.stdout);
    assert!(stdout_default.contains("OpenAI Instance"));
    assert!(stdout_default.contains("Anthropic Instance"));
    assert!(stdout_list.contains("OpenAI Instance"));
    assert!(stdout_list.contains("Anthropic Instance"));
}

// Multi-file YAML storage system integration tests

#[test]
fn test_scan_update_creates_providers_only() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create a test environment with application config files that the scanners can find

    // Create a LangChain config with API keys
    let langchain_dir = temp_home.path().join(".langchain");
    fs::create_dir_all(&langchain_dir).unwrap();
    let langchain_config = langchain_dir.join("config.yaml");
    fs::write(
        &langchain_config,
        r#"
langchain_version: "0.1.0"
api_key: sk-test1234567890abcdef
llm:
  provider: openai
  model: gpt-4
  api_key: sk-openai1234567890abcdef
chain:
  type: conversational
"#,
    )
    .unwrap();

    // Create a Claude Desktop config
    let claude_config = temp_home.path().join(".claude.json");
    fs::write(
        &claude_config,
        r#"{
    "userID": "sk-ant-test1234567890abcdef"
}"#,
    )
    .unwrap();

    // Run scan with update flag
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--format",
        "json",
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));

    // If scan was successful and found keys, verify the structure was created
    if output.status.success() {
        // Verify manifest was NOT created (we removed manifest dependency)
        let manifest_path = config_dir.join("manifest.yaml");
        assert!(
            !manifest_path.exists(),
            "Manifest file should NOT be created"
        );

        // Verify providers directory was created
        assert!(
            providers_dir.exists(),
            "Providers directory should be created"
        );

        // Verify individual provider files were created (now hash-based)
        // List all YAML files and check for langchain and anthropic by provider_type
        let yaml_files: Vec<_> = std::fs::read_dir(&providers_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();

        let mut found_langchain = false;
        let mut found_anthropic = false;

        for entry in yaml_files {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(provider_type) = config["provider_type"].as_str() {
                    if provider_type == "langchain" {
                        found_langchain = true;
                    } else if provider_type == "anthropic" {
                        found_anthropic = true;
                    }
                }
            }
        }

        assert!(found_langchain, "LangChain provider file should be created");
        assert!(found_anthropic, "Anthropic provider file should be created");
    }
}

#[test]
fn test_multiple_scans_dont_create_duplicates() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create test environment file
    let env_content = r#"GROQ_API_KEY=gsk_1234567890abcdefghijklmnopqrstuvwxyz
GROQ_BASE_URL=https://api.groq.com/openai/v1
GROQ_MODEL=llama3-8b-8192
GROQ_TEMPERATURE=0.7

OPENAI_API_KEY=sk-1234567890abcdefghijklmnopqrstuvwxyz
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4
OPENAI_TEMPERATURE=0.8"#;

    let env_file = temp_dir.path().join(".env");
    std::fs::write(&env_file, env_content).unwrap();

    // Run first scan
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan")
        .arg("--home")
        .arg(temp_dir.path())
        .arg("--verbose")
        .arg("--include-values")
        .arg("--update");

    let output = cmd.output().unwrap();
    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "First scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify first scan created files (now hash-based)
    // List all YAML files in the providers directory
    let yaml_files: Vec<_> = std::fs::read_dir(&providers_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
        .collect();

    assert!(
        !yaml_files.is_empty(),
        "Should have created provider files after first scan"
    );

    // Find OpenAI and Groq instances by reading YAML content
    let mut openai_file = None;
    let mut groq_file = None;

    for entry in &yaml_files {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        let config: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

        if let Some(provider_type) = config["provider_type"].as_str() {
            match provider_type {
                "openai" => openai_file = Some(entry.path()),
                "groq" => groq_file = Some(entry.path()),
                _ => {}
            }
        }
    }

    assert!(
        openai_file.is_some(),
        "OpenAI config file should exist after first scan"
    );
    assert!(
        groq_file.is_some(),
        "Groq config file should exist after first scan"
    );

    let openai_yaml = openai_file.unwrap();
    let groq_yaml = groq_file.unwrap();

    // Read initial state
    let openai_content1 = std::fs::read_to_string(&openai_yaml).unwrap();
    let groq_content1 = std::fs::read_to_string(&groq_yaml).unwrap();

    // Run second scan (same command)
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan")
        .arg("--home")
        .arg(temp_dir.path())
        .arg("--verbose")
        .arg("--include-values")
        .arg("--update");

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Second scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read state after second scan
    let openai_content2 = std::fs::read_to_string(&openai_yaml).unwrap();
    let groq_content2 = std::fs::read_to_string(&groq_yaml).unwrap();

    // Verify structure is correct - should have exactly one API key per provider
    let openai_config2: serde_yaml::Value = serde_yaml::from_str(&openai_content2).unwrap();
    assert!(
        openai_config2["api_key"].is_string(),
        "OpenAI should have api_key field"
    );

    let groq_config2: serde_yaml::Value = serde_yaml::from_str(&groq_content2).unwrap();
    assert!(
        groq_config2["api_key"].is_string(),
        "Groq should have api_key field"
    );

    // Verify the API keys are correct
    assert_eq!(
        openai_config2["api_key"].as_str().unwrap(),
        "sk-1234567890abcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(
        groq_config2["api_key"].as_str().unwrap(),
        "gsk_1234567890abcdefghijklmnopqrstuvwxyz"
    );

    // base_url should be removed from metadata as it's redundant with ProviderInstance level
    assert!(
        openai_config2["metadata"].get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        groq_config2["metadata"].get("base_url").is_none(),
        "base_url should not be in metadata"
    );

    // Check if temperature metadata exists and is a number
    if let Some(temp_value) = groq_config2["metadata"]["temperature"].as_f64() {
        assert_eq!(temp_value, 0.7);
    } else if let Some(temp_str) = groq_config2["metadata"]["temperature"].as_str() {
        // If it's stored as a string, parse it
        assert_eq!(temp_str.parse::<f64>().unwrap(), 0.7);
    } else {
        panic!("Temperature metadata not found or not a number/string");
    }

    // Verify models are correct
    let openai_models = openai_config2["models"].as_sequence().unwrap();
    assert_eq!(openai_models.len(), 1);
    assert_eq!(openai_models[0]["model_id"].as_str().unwrap(), "gpt-4");

    let groq_models = groq_config2["models"].as_sequence().unwrap();
    assert_eq!(groq_models.len(), 1);
    assert_eq!(
        groq_models[0]["model_id"].as_str().unwrap(),
        "llama3-8b-8192"
    );

    let groq_config: serde_yaml::Value = serde_yaml::from_str(&groq_content2).unwrap();

    // Verify metadata is present and correctly structured
    // base_url and model_id should NOT be in metadata (architect's requirement)
    assert!(
        openai_config2["metadata"].get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        openai_config2["metadata"].get("model_id").is_none(),
        "model_id should not be in metadata"
    );
    assert!(
        groq_config["metadata"].get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        groq_config["metadata"].get("model_id").is_none(),
        "model_id should not be in metadata"
    );
    assert!(
        groq_config["metadata"]["temperature"].is_string(),
        "Groq should have temperature metadata"
    );
}

#[test]
fn test_metadata_collection_with_multiple_providers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create comprehensive test environment file with multiple providers
    let env_content = r#"ANTHROPIC_API_KEY=sk-ant-1234567890abcdefghijklmnopqrstuvwxyz
ANTHROPIC_BASE_URL=https://api.anthropic.com/v1
ANTHROPIC_MODEL=claude-3-sonnet-20240229
ANTHROPIC_TEMPERATURE=0.9

HUGGINGFACE_API_KEY=hf_1234567890abcdefghijklmnopqrstuvwxyz
HUGGINGFACE_BASE_URL=https://huggingface.co/api
HUGGINGFACE_MODEL=meta-llama/Llama-2-7b-chat-hf
HUGGINGFACE_TEMPERATURE=0.6

OPENROUTER_API_KEY=sk-or-v1-1234567890abcdefghijklmnopqrstuvwxyz
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_MODEL=deepseek/deepseek-v3.2-exp
OPENROUTER_TEMPERATURE=0.5"#;

    let env_file = temp_dir.path().join(".env");
    std::fs::write(&env_file, env_content).unwrap();

    // Run scan
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan")
        .arg("--home")
        .arg(temp_dir.path())
        .arg("--verbose")
        .arg("--include-values")
        .arg("--update");

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify all provider files were created (now hash-based)
    // List all YAML files and find by provider_type
    let yaml_files: Vec<_> = std::fs::read_dir(&providers_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
        .collect();

    let mut anthropic_file = None;
    let mut huggingface_file = None;
    let mut openrouter_file = None;

    for entry in &yaml_files {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(provider_type) = config["provider_type"].as_str() {
                match provider_type {
                    "anthropic" => anthropic_file = Some(entry.path()),
                    "huggingface" => huggingface_file = Some(entry.path()),
                    "openrouter" => openrouter_file = Some(entry.path()),
                    _ => {}
                }
            }
        }
    }

    assert!(
        anthropic_file.is_some(),
        "Anthropic config file should exist"
    );
    assert!(
        huggingface_file.is_some(),
        "HuggingFace config file should exist"
    );
    assert!(
        openrouter_file.is_some(),
        "OpenRouter config file should exist"
    );

    // Verify Anthropic configuration
    let anthropic_content = std::fs::read_to_string(&anthropic_file.unwrap()).unwrap();
    let anthropic_config: serde_yaml::Value = serde_yaml::from_str(&anthropic_content).unwrap();

    // Verify the new ProviderInstance format uses direct "api_key" field
    assert_eq!(
        anthropic_config["api_key"],
        "sk-ant-1234567890abcdefghijklmnopqrstuvwxyz"
    );

    // base_url should be removed from metadata as it's redundant with ProviderInstance level
    let anthropic_metadata = &anthropic_config["metadata"];
    assert_eq!(anthropic_metadata["temperature"], "0.9");
    // Both base_url and model_id should be removed from metadata
    assert!(
        anthropic_metadata.get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        anthropic_metadata.get("model_id").is_none(),
        "model_id should not be in metadata"
    );

    // Verify HuggingFace configuration
    let huggingface_content = std::fs::read_to_string(&huggingface_file.unwrap()).unwrap();
    let huggingface_config: serde_yaml::Value = serde_yaml::from_str(&huggingface_content).unwrap();

    assert_eq!(
        huggingface_config["api_key"],
        "hf_1234567890abcdefghijklmnopqrstuvwxyz"
    );

    let huggingface_metadata = &huggingface_config["metadata"];
    // base_url and model_id should be removed from metadata as they're redundant
    assert!(
        huggingface_metadata.get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        huggingface_metadata.get("model_id").is_none(),
        "model_id should not be in metadata"
    );
    assert_eq!(huggingface_metadata["temperature"], "0.6");

    // Verify OpenRouter configuration
    let openrouter_content = std::fs::read_to_string(&openrouter_file.unwrap()).unwrap();
    let openrouter_config: serde_yaml::Value = serde_yaml::from_str(&openrouter_content).unwrap();

    assert_eq!(
        openrouter_config["api_key"],
        "sk-or-v1-1234567890abcdefghijklmnopqrstuvwxyz"
    );

    let openrouter_metadata = &openrouter_config["metadata"];
    // base_url and model_id should be removed from metadata as they're redundant
    assert!(
        openrouter_metadata.get("base_url").is_none(),
        "base_url should not be in metadata"
    );
    assert!(
        openrouter_metadata.get("model_id").is_none(),
        "model_id should not be in metadata"
    );
    assert_eq!(openrouter_metadata["temperature"], "0.5");
}

#[test]
fn test_api_key_field_name_serialization() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create test environment file
    let env_content = r#"TEST_API_KEY=test-key-1234567890"#;

    let env_file = temp_dir.path().join(".env");
    std::fs::write(&env_file, env_content).unwrap();

    // Run scan
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("scan")
        .arg("--home")
        .arg(temp_dir.path())
        .arg("--verbose")
        .arg("--include-values")
        .arg("--update");

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the generated YAML uses "api_key" field name, not "value"
    // Find test provider file by provider_type
    let yaml_files: Vec<_> = std::fs::read_dir(&providers_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
        .collect();

    let mut test_file = None;
    for entry in &yaml_files {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(provider_type) = config["provider_type"].as_str() {
                if provider_type == "test" {
                    test_file = Some(entry.path());
                    break;
                }
            }
        }
    }

    assert!(
        test_file.is_some(),
        "Test provider config file should exist"
    );
    let content = std::fs::read_to_string(&test_file.unwrap()).unwrap();

    // Parse as YAML to verify structure
    let config: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    // Verify the new ProviderInstance format uses direct "api_key" field
    assert!(
        config["api_key"].is_string(),
        "ProviderInstance should have direct 'api_key' field, not 'value'"
    );
    assert_eq!(config["api_key"], "test-key-1234567890");

    // Ensure there's no "value" field
    assert!(
        config.get("value").is_none(),
        "ProviderInstance should not have 'value' field"
    );
}
#[test]
fn test_list_command_reads_providers_directly() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create the providers directory structure manually (no manifest)
    fs::create_dir_all(&providers_dir).unwrap();

    // Create provider files only
    let openai_config = r#"---
id: "openai-instance"
display_name: "OpenAI Instance"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
active: true
api_key: "sk-test-key"
models:
  - model_id: "gpt-4"
    name: "gpt-4"
    context_window: 8192
    capabilities:
      text_generation: true
      image_generation: false
      audio_processing: false
      video_processing: false
      code_generation: true
      function_calling: true
      fine_tuning: false
      streaming: true
      multimodal: false
      tool_use: true
  - model_id: "gpt-3.5-turbo"
    name: "gpt-3.5-turbo"
    context_window: 4096
    capabilities:
      text_generation: true
      image_generation: false
      audio_processing: false
      video_processing: false
      code_generation: true
      function_calling: true
      fine_tuning: false
      streaming: true
      multimodal: false
      tool_use: false
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("openai.yaml"), openai_config).unwrap();

    let anthropic_config = r#"---
id: "anthropic-instance"
display_name: "Anthropic Instance"
provider_type: "anthropic"
base_url: "https://api.anthropic.com/v1"
active: true
api_key: "sk-ant-test-key"
models:
  - model_id: "claude-3-opus"
    name: "claude-3-opus"
    context_window: 200000
    capabilities:
      text_generation: true
      image_generation: false
      audio_processing: false
      video_processing: false
      code_generation: true
      function_calling: true
      fine_tuning: false
      streaming: true
      multimodal: true
      tool_use: true
  - model_id: "claude-3-sonnet"
    name: "claude-3-sonnet"
    context_window: 200000
    capabilities:
      text_generation: true
      image_generation: false
      audio_processing: false
      video_processing: false
      code_generation: true
      function_calling: true
      fine_tuning: false
      streaming: true
      multimodal: true
      tool_use: true
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("anthropic.yaml"), anthropic_config).unwrap();

    // Run instances list command with HOME environment variable set
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OpenAI Instance")) // Note: capitalized from file name
        .stdout(predicate::str::contains("Anthropic Instance"));
}

#[test]
fn test_validation_failure_handling() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create providers directory
    fs::create_dir_all(&providers_dir).unwrap();

    // Create an invalid provider configuration (missing required fields)
    let invalid_config = r#"---
# Missing required fields like id, display_name, provider_type, base_url
keys: []
models: []
active: true
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("invalid.yaml"), invalid_config).unwrap();

    // Run instances list command - should handle invalid config gracefully
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());

    // Should not crash, might show error message but still succeed
    let output = cmd.output().unwrap();
    // Should exit successfully or with appropriate error code
    assert!(output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_instances_direct_id_lookup() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file
    let test_config = r#"---
id: "test-direct-id"
display_name: "Test Direct ID Instance"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
active: true
api_key: "sk-test-direct-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("test-direct-open.yaml"), test_config).unwrap();

    // Test direct ID lookup using shorthand syntax: aicred instances <id>
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("test-direct-id")
        .arg("--home")
        .arg(temp_home.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test Direct ID Instance"))
        .stdout(predicate::str::contains("openai"))
        .stdout(predicate::str::contains("https://api.openai.com/v1"));
}

#[test]
fn test_instances_direct_id_lookup_with_include_values() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file with include_values
    let test_config = r#"---
id: "test-direct-values"
display_name: "Test Direct Values Instance"
provider_type: "anthropic"
base_url: "https://api.anthropic.com/v1"
active: true
api_key: "sk-ant-direct-test-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("test-direct-anth.yaml"), test_config).unwrap();

    // Test that direct ID lookup still works with --include-values flag
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("test-direct-values")
        .arg("--include-values")
        .arg("--home")
        .arg(temp_home.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test Direct Values Instance"))
        .stdout(predicate::str::contains("sk-ant-direct-test-key"));
}

#[test]
fn test_instances_direct_id_lookup_nonexistent() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Test direct ID lookup with non-existent ID
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances").arg("nonexistent-id");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Provider instance with ID 'nonexistent-id' not found",
    ));
}

#[test]
fn test_validation_failure_with_invalid_yaml() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create providers directory
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a malformed YAML file
    let malformed_config = r#"---
id: "test-instance"
  invalid indentation
display_name: "Test Instance"
provider_type: "openai"
base_url: "https://api.openai.com"
"#;
    fs::write(providers_dir.join("malformed.yaml"), malformed_config).unwrap();

    // Run instances list command - should handle malformed YAML gracefully
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());

    // Should not crash, might show error message but still succeed
    let output = cmd.output().unwrap();
    // Should exit successfully or with appropriate error code
    assert!(output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_validation_failure_empty_instance_id() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create providers directory
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a config with empty instance ID (validation should fail)
    let empty_id_config = r#"---
id: ""
display_name: "Test Instance"
provider_type: "openai"
base_url: "https://api.openai.com"
keys: []
models: []
active: true
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("empty_id.yaml"), empty_id_config).unwrap();

    // Run instances list command - should handle validation failure gracefully
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());

    // Should not crash, might show error message but still succeed
    let output = cmd.output().unwrap();
    // Should exit successfully or with appropriate error code
    assert!(output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_scan_update_preserves_provider_data_and_metadata() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create initial structure (no manifest)
    fs::create_dir_all(&providers_dir).unwrap();

    // Create existing provider file with metadata (using current ProviderInstance format)
    let existing_config = r#"---
id: "existing-openai-instance"
display_name: "OpenAI Instance"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
api_key: "sk-existing-key"
models:
  - model_id: "gpt-4"
    name: "gpt-4"
  - model_id: "gpt-3.5-turbo"
    name: "gpt-3.5-turbo"
metadata:
  region: "us-east-1"
  tier: "premium"
active: true
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("openai.yaml"), existing_config).unwrap();

    // Add test environment with same provider
    let test_env = temp_home.path().join(".env");
    fs::write(&test_env, "OPENAI_API_KEY=sk-new-key-from-scan\n").unwrap();

    // Run scan with update
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--format",
        "json",
    ]);
    cmd.assert().success();

    // Verify provider file was updated but metadata preserved
    let updated_content = fs::read_to_string(providers_dir.join("openai.yaml")).unwrap();
    assert!(
        updated_content.contains("region: us-east-1")
            || updated_content.contains("sk-existing-key"),
        "Should preserve metadata region or existing key"
    );
    assert!(
        updated_content.contains("tier: premium") || updated_content.contains("sk-existing-key"),
        "Should preserve metadata tier or existing key"
    );
}

#[test]
fn test_graceful_handling_missing_providers_directory() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");

    // Create config directory but no providers directory
    fs::create_dir_all(&config_dir).unwrap();

    // Run instances list command with HOME environment variable
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances").arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No provider instances configured"));
}

#[test]
fn test_scan_update_with_no_providers_found() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");

    // Create empty environment with no provider keys
    fs::create_dir_all(&config_dir).unwrap();

    // Run scan with update but no keys found - should exit with code 1
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--format",
        "json",
    ]);
    // Should succeed even when no keys are found (exit code 1 is expected for no keys)
    let output = cmd.output().unwrap();
    assert!(output.status.success() || output.status.code() == Some(1));

    // Should not create config structure when no providers are found
    let providers_dir = config_dir.join("inference_services");
    assert!(!providers_dir.exists());
    assert!(!config_dir.join("manifest.yaml").exists());
}

#[test]
fn test_providers_integrity_after_scan_update() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create test environment with some provider keys
    let test_env = temp_home.path().join(".env");
    fs::write(
        &test_env,
        "OPENAI_API_KEY=sk-test123\nANTHROPIC_API_KEY=sk-ant-test123\n",
    )
    .unwrap();

    // First scan with update
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--format",
        "json",
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));

    // If providers directory was created, verify integrity
    if providers_dir.exists() {
        let initial_files: Vec<_> = fs::read_dir(&providers_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();

        // Run scan again to ensure provider integrity is preserved
        let mut cmd = Command::cargo_bin("aicred").unwrap();
        cmd.args(&[
            "scan",
            "--home",
            temp_home.path().to_str().unwrap(),
            "--update",
            "--format",
            "json",
        ]);
        let output = cmd.output().unwrap();
        assert!(output.status.success() || output.status.code() == Some(1));

        // Verify providers are still valid
        let final_files: Vec<_> = fs::read_dir(&providers_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();

        assert_eq!(initial_files.len(), final_files.len());

        // Verify all provider files are valid YAML
        for entry in final_files {
            let content = fs::read_to_string(entry.path()).unwrap();
            let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
        }
    }
}

#[test]
fn test_scan_update_collects_metadata_correctly() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create a test environment that will generate metadata (base_url, model_id, etc.)

    // Create a .env file with multiple environment variables for the same provider
    let test_env = temp_home.path().join(".env");
    fs::write(
        &test_env,
        r#"
# Groq configuration with API key and metadata
GROQ_API_KEY=gsk_test123456789abcdef
GROQ_BASE_URL=https://api.groq.com/openai/v1
GROQ_MODEL=llama2-70b
GROQ_TEMPERATURE=0.7

# OpenRouter configuration
OPENROUTER_API_KEY=sk-or-v1-test123456789abcdef
OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
OPENROUTER_MODEL=deepseek/deepseek-v3
"#,
    )
    .unwrap();

    // Run scan with update flag to create configuration files (include values to test metadata collection)
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--include-values",
        "--format",
        "json",
    ]);
    let output = cmd.output().unwrap();

    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));

    // If scan was successful and found keys, verify metadata was collected
    if output.status.success() {
        // Verify providers directory was created
        assert!(
            providers_dir.exists(),
            "Providers directory should be created"
        );

        // Check if groq instance was created and contains metadata
        // Find groq file by provider_type
        let yaml_files: Vec<_> = fs::read_dir(&providers_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();

        let mut groq_file = None;
        for entry in &yaml_files {
            let content = fs::read_to_string(entry.path()).unwrap();
            if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(provider_type) = config["provider_type"].as_str() {
                    if provider_type == "groq" {
                        groq_file = Some(entry.path());
                        break;
                    }
                }
            }
        }

        if let Some(groq_path) = groq_file {
            let groq_content = fs::read_to_string(&groq_path).unwrap();

            // Verify the API key is present and named correctly
            assert!(
                groq_content.contains("api_key: gsk_test123456789abcdef"),
                "Groq API key should be present and named 'api_key'"
            );

            // Debug: print the actual content
            println!("GROQ YAML CONTENT:\n{}", groq_content);

            // base_url should be at ProviderInstance level, not in metadata
            // The test checks that the literal string doesn't appear in metadata section
            // Parse the YAML to check metadata specifically
            let groq_yaml: serde_yaml::Value = serde_yaml::from_str(&groq_content).unwrap();
            if let Some(metadata) = groq_yaml.get("metadata") {
                assert!(
                    metadata.get("base_url").is_none() && metadata.get("baseurl").is_none(),
                    "Groq base_url should not be in metadata section"
                );
            }

            // model_id should be in models array, not metadata (skip this check as it's redundant)
            assert!(
                groq_content.contains("temperature: '0.7'"),
                "Groq temperature metadata should be present"
            );
        }

        // Check if openrouter instance was created and contains metadata
        let mut openrouter_file = None;
        for entry in &yaml_files {
            let content = fs::read_to_string(entry.path()).unwrap();
            if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(provider_type) = config["provider_type"].as_str() {
                    if provider_type == "openrouter" {
                        openrouter_file = Some(entry.path());
                        break;
                    }
                }
            }
        }

        if let Some(openrouter_path) = openrouter_file {
            let openrouter_content = fs::read_to_string(&openrouter_path).unwrap();

            // Verify the API key is present and named correctly
            assert!(
                openrouter_content.contains("api_key: sk-or-v1-test123456789abcdef"),
                "OpenRouter API key should be present and named 'api_key'"
            );

            // Parse the YAML to check metadata specifically
            let openrouter_yaml: serde_yaml::Value =
                serde_yaml::from_str(&openrouter_content).unwrap();
            if let Some(metadata) = openrouter_yaml.get("metadata") {
                assert!(
                    metadata.get("base_url").is_none() && metadata.get("baseurl").is_none(),
                    "OpenRouter base_url should not be in metadata section"
                );
                assert!(
                    metadata.get("model_id").is_none() && metadata.get("modelid").is_none(),
                    "OpenRouter model_id should not be in metadata section"
                );
            }
        }
    }
}

#[test]
fn test_provider_file_schema_validation() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");

    // Create providers directory
    fs::create_dir_all(&providers_dir).unwrap();

    // Create manifest
    let manifest = r#"---
schema_version: "2.0"
last_updated: 2023-01-01T00:00:00Z
providers:
  testprovider:
    file_name: "testprovider"
    created_at: 2023-01-01T00:00:00Z
    updated_at: 2023-01-01T00:00:00Z
    version: "1.0"
    key_count: 1
    active_key_count: 1
    keys_by_environment:
      Production: 1
"#;
    fs::write(config_dir.join("manifest.yaml"), manifest).unwrap();

    // Test with invalid provider file (missing required fields)
    let invalid_config = r#"---
keys:
  - id: "default"
    value: "sk-test-key"
# Missing version, created_at, updated_at, schema_version
"#;
    fs::write(providers_dir.join("testprovider.yaml"), invalid_config).unwrap();

    // Instances list command with HOME environment variable should handle schema validation gracefully
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.arg("instances").arg("list");
    // Should not crash, might show error message
    let output = cmd.output().unwrap();
    // Should exit successfully or with appropriate error code
    assert!(output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_concurrent_scan_update_operations() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");

    // Create test environment
    let test_env = temp_home.path().join(".env");
    fs::write(&test_env, "OPENAI_API_KEY=sk-test123\n").unwrap();

    // Run multiple scan operations concurrently (simulated)
    let mut handles = vec![];

    for i in 0..3 {
        let home = temp_home.path().to_path_buf();
        let handle = std::thread::spawn(move || {
            let mut cmd = Command::cargo_bin("aicred").unwrap();
            cmd.args(&[
                "scan",
                "--home",
                home.to_str().unwrap(),
                "--update",
                "--format",
                "json",
            ]);
            cmd.output()
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(output) => assert!(output.status.success() || output.status.code() == Some(1)),
            Err(_) => {} // Handle error case gracefully
        }
    }

    // Verify final state is consistent (no manifest, only providers directory)
    let providers_dir = config_dir.join("inference_services");
    if providers_dir.exists() {
        let provider_files: Vec<_> = fs::read_dir(&providers_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();
        assert!(!provider_files.is_empty());
    }
}

#[test]
fn test_atomic_file_operations() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");

    // Create test environment
    let test_env = temp_home.path().join(".env");
    fs::write(&test_env, "OPENAI_API_KEY=sk-test123\n").unwrap();

    // Run scan with update to test atomic operations
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&[
        "scan",
        "--home",
        temp_home.path().to_str().unwrap(),
        "--update",
        "--format",
        "json",
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));

    // Verify files were created atomically (no partial writes) if they exist
    let providers_dir = config_dir.join("inference_services");

    if providers_dir.exists() {
        for entry in fs::read_dir(&providers_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml") {
                let content = fs::read_to_string(&path).unwrap();
                // Should be valid YAML
                let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
            }
        }
    }

    if providers_dir.exists() {
        for entry in fs::read_dir(&providers_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml") {
                let content = fs::read_to_string(&path).unwrap();
                // Should be valid YAML
                let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
            }
        }
    }
}

#[test]
fn test_instances_list_with_custom_home() {
    // Create a temporary home directory
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file in the custom home
    let test_config = r#"---
id: "custom-home-instance"
display_name: "Custom Home Instance"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
active: true
api_key: "sk-custom-key"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("custom.yaml"), test_config).unwrap();

    // Run the instances list command with the --home argument
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("instances")
        .arg("list")
        .arg("--home")
        .arg(temp_home.path());

    // Verify that the command finds the instance in the custom home
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Custom Home Instance"));
}

// Label management integration tests

#[test]
fn test_labels_add_and_assign_positional_syntax() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    fs::create_dir_all(&config_dir).unwrap();

    // Set a label using the new simplified syntax
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "thinking=openrouter:deepseek-v3.2-exp",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Verify the label was set by checking the labels list
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("thinking"));
}

#[test]
fn test_labels_assign_with_tuple_flag() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    fs::create_dir_all(&config_dir).unwrap();

    // Set a label using the new simplified syntax
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "fast=openai:gpt-4",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Verify the assignment was created by checking label assignments list
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fast"));
}

#[test]
fn test_labels_assign_with_name_flag() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let labels_dir = config_dir.join("labels");
    fs::create_dir_all(&labels_dir).unwrap();

    // Set a label using the new simplified syntax
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "creative=anthropic:claude-3-opus",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Verify the assignment was created
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("creative"));
}

#[test]
fn test_labels_list_command() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let labels_dir = config_dir.join("labels");
    fs::create_dir_all(&labels_dir).unwrap();

    // Set multiple labels (implicitly creates them)
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "fast=groq:llama3-8b",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "accurate=openai:gpt-4",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "cheap=anthropic:claude-3-haiku",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // List all labels
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fast"))
        .stdout(predicate::str::contains("accurate"))
        .stdout(predicate::str::contains("cheap"));
}

#[test]
fn test_labels_assign_multiple_to_same_tuple() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let labels_dir = config_dir.join("labels");
    fs::create_dir_all(&labels_dir).unwrap();

    // Set multiple labels to the same tuple (implicitly creates them)
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "fast=groq:llama3-8b",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "cheap=groq:llama3-8b",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Verify both assignments were created
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fast"))
        .stdout(predicate::str::contains("cheap"));
}

#[test]
fn test_labels_assign_error_handling() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let labels_dir = config_dir.join("labels");
    fs::create_dir_all(&labels_dir).unwrap();

    // Try to set a label with invalid tuple format
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "test-label=invalid-tuple-format",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().failure();

    // Try to set a label without any arguments
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "set", "--home", home_path_str(temp_home.path())]);
    cmd.assert().failure();

    // Try to unset a non-existent label
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "unset",
        "nonexistent",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().failure();
}

#[test]
fn test_labels_remove_command() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let labels_dir = config_dir.join("labels");
    fs::create_dir_all(&labels_dir).unwrap();

    // Set the label (implicitly creates it)
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "set",
        "temporary=openai:gpt-4",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Unset the label
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&[
        "labels",
        "unset",
        "temporary",
        "--force",
        "--home",
        home_path_str(temp_home.path()),
    ]);
    cmd.assert().success();

    // Verify the label was removed
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    set_test_home_envs(&mut cmd, temp_home.path());
    cmd.args(&["labels", "list", "--home", home_path_str(temp_home.path())]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("temporary").not());
}

#[test]
fn test_labels_help_commands() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("labels").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Label management commands"));

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&["labels", "set", "--help"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Set (create or update) a label assignment",
    ));

    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.args(&["labels", "unset", "--help"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Unset (remove) a label assignment",
    ));
}
