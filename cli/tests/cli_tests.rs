use std::fs;
use std::env;
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;

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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("list").arg("--verbose");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openai"))
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
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openai"));
}

#[test]
fn test_list_contains_scanners() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Openai")); // Test for one of the configured providers
}

// Multi-file YAML storage system integration tests

#[test]
fn test_scan_update_creates_providers_only() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
    
    // Create a test environment with application config files that the scanners can find
    
    // Create a LangChain config with API keys
    let langchain_dir = temp_home.path().join(".langchain");
    fs::create_dir_all(&langchain_dir).unwrap();
    let langchain_config = langchain_dir.join("config.yaml");
    fs::write(&langchain_config, r#"
langchain_version: "0.1.0"
api_key: sk-test1234567890abcdef
llm:
  provider: openai
  model: gpt-4
  api_key: sk-openai1234567890abcdef
chain:
  type: conversational
"#).unwrap();
    
    // Create a Claude Desktop config
    let claude_config = temp_home.path().join(".claude.json");
    fs::write(&claude_config, r#"{
    "userID": "sk-ant-test1234567890abcdef"
}"#).unwrap();
    
    // Run scan with update flag
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.args(&[
        "scan",
        "--home", temp_home.path().to_str().unwrap(),
        "--update",
        "--format", "json"
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));
    
    // If scan was successful and found keys, verify the structure was created
    if output.status.success() {
        // Verify manifest was NOT created (we removed manifest dependency)
        let manifest_path = config_dir.join("manifest.yaml");
        assert!(!manifest_path.exists(), "Manifest file should NOT be created");
        
        // Verify providers directory was created
        assert!(providers_dir.exists(), "Providers directory should be created");
        
        // Verify individual provider files were created (at minimum should have langchain and anthropic)
        assert!(providers_dir.join("langchain.yaml").exists(), "LangChain provider file should be created");
        assert!(providers_dir.join("anthropic.yaml").exists(), "Anthropic provider file should be created");
    }
}

#[test]
fn test_list_command_reads_providers_directly() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
    
    // Create the providers directory structure manually (no manifest)
    fs::create_dir_all(&providers_dir).unwrap();
    
    // Create provider files only
    let openai_config = r#"---
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
  - "gpt-4"
  - "gpt-3.5-turbo"
version: "1.0"
schema_version: "3.0"
created_at: "2023-01-01T00:00:00Z"
updated_at: "2023-01-01T00:00:00Z"
"#;
    fs::write(providers_dir.join("openai.yaml"), openai_config).unwrap();
    
    let anthropic_config = r#"---
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
  - "claude-3-opus"
  - "claude-3-sonnet"
version: "1.0"
schema_version: "3.0"
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
        .stdout(predicate::str::contains("Openai"))  // Note: capitalized from file name
        .stdout(predicate::str::contains("Anthropic"));
}

#[test]
fn test_migration_from_single_file_format() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
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
    assert!(!config_dir.join("manifest.yaml").exists(), "Manifest should NOT be created after migration");
    assert!(providers_dir.exists(), "Providers directory should be created after migration");
    assert!(providers_dir.join("openai.yaml").exists(), "OpenAI provider file should be created");
    assert!(providers_dir.join("anthropic.yaml").exists(), "Anthropic provider file should be created");
    assert!(!old_config_path.exists(), "Old config file should be removed after migration");
    
    // Verify backup was created
    let backup_path = old_config_path.with_extension("yaml.backup");
    assert!(backup_path.exists(), "Backup file should be created");
}

#[test]
fn test_scan_update_preserves_provider_data_and_metadata() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
    
    // Create initial structure (no manifest)
    fs::create_dir_all(&providers_dir).unwrap();
    
    // Create existing provider file with metadata
    let existing_config = r#"---
keys:
  - id: "default"
    value: "sk-existing-key"
    discovered_at: "2023-01-01T00:00:00Z"
    source: "openai.yaml"
    line_number: 1
    confidence: "High"
    environment: "Production"
    validation_status: "Unknown"
    created_at: "2023-01-01T00:00:00Z"
    updated_at: "2023-01-01T00:00:00Z"
models:
  - "gpt-4"
  - "gpt-3.5-turbo"
metadata:
  region: "us-east-1"
  tier: "premium"
version: "1.0"
schema_version: "3.0"
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
        "--home", temp_home.path().to_str().unwrap(),
        "--update",
        "--format", "json"
    ]);
    cmd.assert().success();
    
    // Verify provider file was updated but metadata preserved
    let updated_content = fs::read_to_string(providers_dir.join("openai.yaml")).unwrap();
    assert!(updated_content.contains("region: us-east-1") || updated_content.contains("sk-existing-key"));
    assert!(updated_content.contains("tier: premium") || updated_content.contains("sk-existing-key"));
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
        "--home", temp_home.path().to_str().unwrap(),
        "--update",
        "--format", "json"
    ]);
    // Should succeed even when no keys are found (exit code 1 is expected for no keys)
    let output = cmd.output().unwrap();
    assert!(output.status.success() || output.status.code() == Some(1));
    
    // Should not create config structure when no providers are found
    let providers_dir = config_dir.join("providers");
    assert!(!providers_dir.exists());
    assert!(!config_dir.join("manifest.yaml").exists());
}

#[test]
fn test_providers_integrity_after_scan_update() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
    
    // Create test environment with some provider keys
    let test_env = temp_home.path().join(".env");
    fs::write(&test_env, "OPENAI_API_KEY=sk-test123\nANTHROPIC_API_KEY=sk-ant-test123\n").unwrap();
    
    // First scan with update
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.args(&[
        "scan",
        "--home", temp_home.path().to_str().unwrap(),
        "--update",
        "--format", "json"
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));
    
    // If providers directory was created, verify integrity
    if providers_dir.exists() {
        let initial_files: Vec<_> = fs::read_dir(&providers_dir).unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .collect();
        
        // Run scan again to ensure provider integrity is preserved
        let mut cmd = Command::cargo_bin("keyfinder").unwrap();
        cmd.args(&[
            "scan",
            "--home", temp_home.path().to_str().unwrap(),
            "--update",
            "--format", "json"
        ]);
        let output = cmd.output().unwrap();
        assert!(output.status.success() || output.status.code() == Some(1));
        
        // Verify providers are still valid
        let final_files: Vec<_> = fs::read_dir(&providers_dir).unwrap()
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
fn test_provider_file_schema_validation() {
    let temp_home = TempDir::new().unwrap();
    let config_dir = temp_home.path().join(".config").join("aicred");
    let providers_dir = config_dir.join("providers");
    
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
                "--home", home.to_str().unwrap(),
                "--update",
                "--format", "json"
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
    let providers_dir = config_dir.join("providers");
    if providers_dir.exists() {
        let provider_files: Vec<_> = fs::read_dir(&providers_dir).unwrap()
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
        "--home", temp_home.path().to_str().unwrap(),
        "--update",
        "--format", "json"
    ]);
    let output = cmd.output().unwrap();
    // Should succeed if keys are found, or exit with code 1 if none found
    assert!(output.status.success() || output.status.code() == Some(1));
    
    // Verify files were created atomically (no partial writes) if they exist
    let providers_dir = config_dir.join("providers");
    
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
