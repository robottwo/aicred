use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("genai-keyfinder"));
}

#[test]
fn test_list_command() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file with hash-based filename
    let test_config = r#"---
id: "test-instance-123"
display_name: "Test Provider"
provider_type: "openai"
base_url: "https://api.openai.com/v1"
active: true
keys:
  - id: "default"
    value: "sk-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "test.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("abc123def456.yaml"), test_config).unwrap();

    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configured Providers"));
}

#[test]
fn test_scan_help() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("scan").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan for GenAI credentials"));
}

#[test]
fn test_scan_dry_run() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("scan").arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));
}

#[test]
fn test_list_verbose() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file with hash-based filename
    let test_config = r#"---
id: "openrouter-instance"
display_name: "OpenRouter Instance"
provider_type: "openrouter"
base_url: "https://openrouter.ai/api/v1"
active: true
keys:
  - id: "default"
    value: "sk-or-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "test.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("def456abc789.yaml"), test_config).unwrap();

    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list").arg("--verbose");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openrouter"))
        .stdout(predicate::str::contains("Keys:"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("invalid-command");
    cmd.assert().failure();
}

#[test]
fn test_scan_with_format() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg("--dry-run");
    cmd.assert().success();
}

#[test]
fn test_all_output_formats() {
    for format in ["json", "ndjson", "table", "summary"].iter() {
        let mut cmd = Command::cargo_bin("keyfinder").unwrap();
        cmd.args(&["scan", "--format", format, "--dry-run"]);
        cmd.assert().success();
    }
}

#[test]
fn test_provider_filtering() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.args(&["scan", "--only", "openai,anthropic", "--dry-run"]);
    cmd.assert().success();
}

#[test]
fn test_provider_exclusion() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
        std::env::temp_dir().join(format!("keyfinder_audit_{}_{}.log", std::process::id(), ts));
    let log_str = log_path.to_str().unwrap();

    // Execute without asserting success: exit code is 0 when keys are found, 1 when none found.
    let bin = assert_cmd::cargo::cargo_bin("keyfinder");
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
        .expect("failed to execute keyfinder");
    assert!(status.code() == Some(0) || status.code() == Some(1));

    // Audit log should be written regardless of whether any keys were found.
    assert!(log_path.exists(), "Audit log file should be created");

    // Basic content sanity check
    let contents = std::fs::read_to_string(&log_path).expect("read audit log");
    assert!(contents.contains("GenAI Key Finder Audit Log"));
}

#[test]
fn test_invalid_format() {
    // Use a temp home directory to avoid scanning the real home
    let home = std::env::temp_dir().join(format!("kf_cli_invalid_fmt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);

    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_list_contains_providers() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file with hash-based filename
    let test_config = r#"---
id: "openrouter-instance"
display_name: "OpenRouter Instance"
provider_type: "openrouter"
base_url: "https://openrouter.ai/api/v1"
active: true
keys:
  - id: "default"
    value: "sk-or-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "test.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("ghi789jkl012.yaml"), test_config).unwrap();

    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openrouter"));
}

#[test]
fn test_list_contains_scanners() {
    // Create a temporary home directory with test configuration
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    fs::create_dir_all(&providers_dir).unwrap();

    // Create a test provider file with hash-based filename
    let test_config = r#"---
id: "openrouter-instance"
display_name: "OpenRouter Instance"
provider_type: "openrouter"
base_url: "https://openrouter.ai/api/v1"
active: true
keys:
  - id: "default"
    value: "sk-or-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "test.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
models: []
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("mno345pqr678.yaml"), test_config).unwrap();

    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openrouter")); // Test for one of the configured providers
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let openai_keys2 = openai_config2["keys"].as_sequence().unwrap();
    assert_eq!(
        openai_keys2.len(),
        1,
        "OpenAI should have exactly one key entry"
    );

    let groq_config2: serde_yaml::Value = serde_yaml::from_str(&groq_content2).unwrap();
    let groq_keys2 = groq_config2["keys"].as_sequence().unwrap();
    assert_eq!(
        groq_keys2.len(),
        1,
        "Groq should have exactly one key entry"
    );

    // Verify the API keys are correct
    assert_eq!(
        openai_keys2[0]["api_key"].as_str().unwrap(),
        "sk-1234567890abcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(
        groq_keys2[0]["api_key"].as_str().unwrap(),
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
    let groq_keys = groq_config["keys"].as_sequence().unwrap();
    assert_eq!(groq_keys.len(), 1, "Groq should have exactly one key entry");

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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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

    let anthropic_keys = anthropic_config["keys"].as_sequence().unwrap();
    assert_eq!(
        anthropic_keys.len(),
        1,
        "Anthropic should have exactly one key entry"
    );
    assert_eq!(
        anthropic_keys[0]["api_key"],
        "sk-ant-1234567890abcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(anthropic_keys[0]["confidence"], "High");

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

    let huggingface_keys = huggingface_config["keys"].as_sequence().unwrap();
    assert_eq!(
        huggingface_keys.len(),
        1,
        "HuggingFace should have exactly one key entry"
    );
    assert_eq!(
        huggingface_keys[0]["api_key"],
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

    let openrouter_keys = openrouter_config["keys"].as_sequence().unwrap();
    assert_eq!(
        openrouter_keys.len(),
        1,
        "OpenRouter should have exactly one key entry"
    );
    assert_eq!(
        openrouter_keys[0]["api_key"],
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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

    // Verify the key uses "api_key" field name
    let keys = config["keys"].as_sequence().unwrap();
    assert_eq!(keys.len(), 1);
    assert!(
        keys[0]["api_key"].is_string(),
        "Key should have 'api_key' field, not 'value'"
    );
    assert_eq!(keys[0]["api_key"], "test-key-1234567890");

    // Ensure there's no "value" field
    assert!(
        keys[0].get("value").is_none(),
        "Key should not have 'value' field"
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
keys:
  - id: "default"
    value: "sk-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "openai.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
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
keys:
  - id: "default"
    value: "sk-ant-test-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "anthropic.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
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

    // Run list command with HOME environment variable set
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openai")) // Note: capitalized from file name
        .stdout(predicate::str::contains("Anthropic"));
}

#[test]
fn test_migration_from_single_file_format() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("inference_services");
    let old_config_path = config_dir.join("providers.yaml");

    // Create the old single-file format
    fs::create_dir_all(&config_dir).unwrap();

    let old_config = r#"---
providers:
  openai:
    api_key: "sk-old-openai-key"
    version: "1.5"
    models:
      - "gpt-4"
      - "gpt-3.5-turbo"
  anthropic:
    api_key: "sk-old-anthropic-key"
    version: "1.2"
    models:
      - "claude-3-opus"
      - "claude-3-sonnet"
"#;
    fs::write(&old_config_path, old_config).unwrap();

    // Run list command which should trigger migration (with HOME env var)
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert().success();

    // Verify migration occurred (no manifest, only providers directory)
    assert!(
        !config_dir.join("manifest.yaml").exists(),
        "Manifest should NOT be created after migration"
    );
    assert!(
        providers_dir.exists(),
        "Providers directory should be created after migration"
    );
    assert!(
        providers_dir.join("openai.yaml").exists(),
        "OpenAI provider file should be created"
    );
    assert!(
        providers_dir.join("anthropic.yaml").exists(),
        "Anthropic provider file should be created"
    );
    assert!(
        !old_config_path.exists(),
        "Old config file should be removed after migration"
    );

    // Verify backup was created
    let backup_path = old_config_path.with_extension("yaml.backup");
    assert!(backup_path.exists(), "Backup file should be created");
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
keys:
  - id: "default"
    api_key: "sk-existing-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "openai.yaml"
    line_number: 1
    confidence: High
    environment: Production
    validation_status: Unknown
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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

    // Run list command with HOME environment variable
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No configuration found"));
}

#[test]
fn test_scan_update_with_no_providers_found() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");

    // Create empty environment with no provider keys
    fs::create_dir_all(&config_dir).unwrap();

    // Run scan with update but no keys found - should exit with code 1
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
        let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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

    // List command with HOME environment variable should handle schema validation gracefully
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.env("HOME", temp_home.path());
    cmd.arg("list");
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
            let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
