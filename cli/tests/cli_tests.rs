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
        .stdout(predicate::str::contains("Available Providers"));
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
        .stdout(predicate::str::contains("OpenAI API keys"));
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
        .stdout(predicate::str::contains("openai"));
}

#[test]
fn test_list_contains_scanners() {
    let mut cmd = Command::cargo_bin("keyfinder").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("roo-code"));
}
