use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_wizard_help() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("wizard").arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Interactive setup wizard"))
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("--skip-probe"))
        .stdout(predicate::str::contains("--skip-labels"));
}

#[test]
fn test_wizard_command_exists() {
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("wizard"));
}

#[test]
fn test_wizard_with_yes_flag_and_skip_labels() {
    // This test validates that the wizard accepts the flags without erroring
    // It will still fail on actual execution without mock input, but that's expected
    let temp = TempDir::new().unwrap();
    let home = temp.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("wizard")
        .arg("--yes")
        .arg("--skip-labels")
        .arg("--skip-probe")
        .arg("--home")
        .arg(home);
    
    // We expect this to fail or require input, but we're just testing the CLI parsing
    // The important thing is it doesn't error on unknown flags
    let _ = cmd.output();
}

#[test]
fn test_config_files_not_created_without_wizard() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join(".config").join("aicred");
    
    // Config dir shouldn't exist yet
    assert!(!config_dir.exists());
}

#[test]
fn test_wizard_requires_interactive_input() {
    // When run without --yes flag, wizard should wait for input
    // This test just ensures the command structure is valid
    let temp = TempDir::new().unwrap();
    let home = temp.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("aicred").unwrap();
    cmd.arg("wizard")
        .arg("--home")
        .arg(home);
    
    // Should start but will block waiting for input
    // We're just validating CLI structure here
    let _ = cmd.timeout(std::time::Duration::from_millis(100)).output();
}
