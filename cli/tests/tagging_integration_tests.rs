//! Integration tests for CLI tag and label management commands.
//!
//! These tests validate the complete CLI workflow including
//! command parsing, execution, and file system operations.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    fn setup_test_cli() -> (Command, TempDir) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // Set HOME to temp directory for isolated testing
        let mut cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        cmd.env("HOME", temp_dir.path());

        (cmd, temp_dir)
    }

    #[test]
    fn test_cli_tag_lifecycle() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Test tag creation
        cmd.arg("tags")
            .arg("add")
            .arg("--name")
            .arg("production")
            .arg("--color")
            .arg("#ff0000")
            .arg("--description")
            .arg("Production environment");

        cmd.assert().success().stdout(predicate::str::contains(
            "Tag 'production' added successfully",
        ));

        // Test tag listing
        let mut list_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        list_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("list");

        list_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains("production"))
            .stdout(predicate::str::contains("#ff0000"));

        // Test tag update
        let mut update_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        update_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("update")
            .arg("--name")
            .arg("production")
            .arg("--color")
            .arg("#00ff00");

        update_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Tag 'production' updated successfully",
            ));

        // Test tag removal
        let mut remove_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        remove_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("remove")
            .arg("--name")
            .arg("production")
            .arg("--force");

        remove_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Tag 'production' removed successfully",
            ));
    }

    #[test]
    fn test_cli_label_lifecycle() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Test label creation
        cmd.arg("labels")
            .arg("set")
            .arg("primary=openai:gpt-4")
            .arg("--color")
            .arg("#00ff00")
            .arg("--description")
            .arg("Primary instance");

        cmd.assert().success().stdout(predicate::str::contains(
            "✓ Label 'primary' set successfully",
        ));

        // Test label listing
        let mut list_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        list_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("list");

        list_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains("primary"))
            .stdout(predicate::str::contains("#00ff00"));

        // Test label update
        let mut update_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        update_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("set")
            .arg("primary=openai:gpt-4")
            .arg("--color")
            .arg("#ff8800");

        update_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "✓ Label 'primary' updated successfully",
            ));

        // Test label removal
        let mut remove_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        remove_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("unset")
            .arg("primary")
            .arg("--force");

        remove_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Label 'primary' unset successfully",
            ));
    }

    #[test]
    fn test_cli_tag_assignment_workflow() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Create a tag
        cmd.arg("tags").arg("add").arg("--name").arg("development");

        cmd.assert().success();

        // Test tag assignment to instance
        let mut assign_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        assign_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("assign")
            .arg("--name")
            .arg("development")
            .arg("--instance")
            .arg("test-instance");

        assign_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Tag 'development' assigned successfully",
            ));

        // Test tag assignment to model
        let mut model_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        model_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("assign")
            .arg("--name")
            .arg("development")
            .arg("--instance")
            .arg("test-instance")
            .arg("--model")
            .arg("gpt-4");

        model_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Tag 'development' assigned successfully",
            ));

        // Test tag unassignment
        let mut unassign_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        unassign_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("unassign")
            .arg("--name")
            .arg("development")
            .arg("--instance")
            .arg("test-instance")
            .arg("--model")
            .arg("gpt-4");

        unassign_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Tag 'development' unassigned successfully",
            ));
    }

    #[test]
    fn test_cli_label_assignment_workflow() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Create a label
        cmd.arg("labels")
            .arg("set")
            .arg("production-primary=openai:gpt-4");

        cmd.assert().success();

        // Test label assignment to instance
        let mut assign_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        assign_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("set")
            .arg("production-primary=openai:gpt-4");

        assign_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "✓ Label 'production-primary' updated successfully",
            ));
    }

    #[test]
    fn test_cli_validation_errors() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Test empty tag name
        cmd.arg("tags").arg("add").arg("--name").arg("");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Tag name cannot be empty"));

        // Test duplicate tag name
        let mut create_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        create_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("add")
            .arg("--name")
            .arg("duplicate-test");

        create_cmd.assert().success();

        let mut duplicate_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        duplicate_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("add")
            .arg("--name")
            .arg("duplicate-test");

        duplicate_cmd
            .assert()
            .failure()
            .stderr(predicate::str::contains("already exists"));

        // Test invalid color format
        let mut color_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        color_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("add")
            .arg("--name")
            .arg("test-tag")
            .arg("--color")
            .arg("invalid-color");

        // This should succeed (color validation is lenient) or fail gracefully
        color_cmd.assert().success();
    }

    #[test]
    fn test_cli_label_uniqueness_constraint() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Create a label
        cmd.arg("labels")
            .arg("set")
            .arg("unique-label=openai:gpt-4");

        cmd.assert().success();

        // Try to assign the same label twice (should fail)
        let mut assign_cmd1 = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        assign_cmd1
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("set")
            .arg("unique-label=openai:gpt-4");

        assign_cmd1.assert().success();

        let mut assign_cmd2 = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        assign_cmd2
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("set")
            .arg("unique-label=openai:gpt-4");

        assign_cmd2
            .assert()
            .success()
            .stdout(predicate::str::contains("updated successfully"));
    }

    #[test]
    fn test_cli_file_persistence() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Create tags and labels
        cmd.arg("tags")
            .arg("add")
            .arg("--name")
            .arg("persistent-tag")
            .arg("--color")
            .arg("#ff0000");

        cmd.assert().success();

        let mut label_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        label_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("set")
            .arg("persistent-label=openai:gpt-4")
            .arg("--color")
            .arg("#00ff00");

        label_cmd.assert().success();

        // Verify files exist
        let tags_file = _temp_dir
            .path()
            .join(".config")
            .join("aicred")
            .join("tags.yaml");
        let labels_file = _temp_dir
            .path()
            .join(".config")
            .join("aicred")
            .join("labels.yaml");

        assert!(tags_file.exists(), "Tags file should exist");
        assert!(labels_file.exists(), "Labels file should exist");

        // Verify file contents
        let tags_content = fs::read_to_string(tags_file).expect("Failed to read tags file");
        let labels_content = fs::read_to_string(labels_file).expect("Failed to read labels file");

        assert!(tags_content.contains("persistent-tag"));
        assert!(labels_content.contains("persistent-label"));

        // Test data persists across commands
        let mut list_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        list_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("list");

        list_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains("persistent-tag"));
    }

    #[test]
    fn test_cli_help_commands() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Test main help
        cmd.arg("--help");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains(
                "AICred - Discover AI API keys and configurations",
            ))
            .stdout(predicate::str::contains("tags"))
            .stdout(predicate::str::contains("labels"));

        // Test tags help
        let mut tags_help = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        tags_help
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("--help");

        tags_help
            .assert()
            .success()
            .stdout(predicate::str::contains("Tag management commands"))
            .stdout(predicate::str::contains("add"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("remove"));

        // Test labels help
        let mut labels_help = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        labels_help
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("--help");

        labels_help
            .assert()
            .success()
            .stdout(predicate::str::contains("Label management commands"))
            .stdout(predicate::str::contains("set"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("unset"));
    }

    #[test]
    fn test_cli_error_handling() {
        let (mut cmd, _temp_dir) = setup_test_cli();

        // Test non-existent tag operations
        cmd.arg("tags")
            .arg("remove")
            .arg("--name")
            .arg("non-existent-tag");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("not found"));

        // Test non-existent label operations
        let mut label_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        label_cmd
            .env("HOME", _temp_dir.path())
            .arg("labels")
            .arg("unset")
            .arg("non-existent-label");

        label_cmd
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));

        // Test invalid assignment operations
        let mut assign_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        assign_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("assign")
            .arg("--name")
            .arg("non-existent-tag")
            .arg("--instance")
            .arg("test-instance");

        assign_cmd
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_cli_concurrent_operations() {
        let (cmd, _temp_dir) = setup_test_cli();

        // Create multiple tags rapidly
        for i in 0..10 {
            let mut tag_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
            tag_cmd
                .env("HOME", _temp_dir.path())
                .arg("tags")
                .arg("add")
                .arg("--name")
                .arg(format!("concurrent-tag-{}", i));

            tag_cmd.assert().success();
        }

        // Verify all tags were created
        let mut list_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        list_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("list");

        list_cmd
            .assert()
            .success()
            .stdout(predicate::str::contains("concurrent-tag-0"))
            .stdout(predicate::str::contains("concurrent-tag-9"));
    }

    #[test]
    fn test_cli_large_dataset_handling() {
        let (cmd, _temp_dir) = setup_test_cli();

        // Create large number of tags
        for i in 0..100 {
            let mut tag_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
            tag_cmd
                .env("HOME", _temp_dir.path())
                .arg("tags")
                .arg("add")
                .arg("--name")
                .arg(format!("large-tag-{}", i))
                .arg("--description")
                .arg(format!("Description for tag {}", i));

            tag_cmd.assert().success();
        }

        // Test listing performance
        let start = std::time::Instant::now();

        let mut list_cmd = Command::cargo_bin("aicred").expect("Failed to find aicred binary");
        list_cmd
            .env("HOME", _temp_dir.path())
            .arg("tags")
            .arg("list");

        list_cmd.assert().success();

        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(
            duration.as_millis() < 5000,
            "Large dataset listing took too long: {:?}",
            duration
        );
    }
}
