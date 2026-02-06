#![allow(deprecated)]
#![allow(unused_must_use)]
//! Integration tests for probe functionality in scan workflow.

use aicred_core::{scan, ScanOptions};

#[test]
fn test_scan_with_probe_models_disabled() {
    // Create test home directory
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create scan options with probing disabled
    let options = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: true,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: false,
        probe_timeout_secs: 30,
    };

    // Run scan
    let result = scan(&options).expect("Scan should succeed");

    // Verify no probe metadata exists when probing is disabled
    assert!(
        result.metadata.is_none()
            || !result
                .metadata
                .as_ref()
                .unwrap()
                .contains_key("probe_total_instances"),
        "Probe metadata should not exist when probing is disabled"
    );
}

#[test]
fn test_scan_with_probe_models_enabled() {
    // Create test home directory
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create scan options with probing enabled
    let options = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: true,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: true,
        probe_timeout_secs: 5,
    };

    // Run scan
    let result = scan(&options).expect("Scan should succeed even with no instances to probe");

    // Verify probe metadata exists when probing is enabled
    let metadata = result
        .metadata
        .as_ref()
        .expect("Metadata should exist when probing is enabled");
    assert!(
        metadata.contains_key("probe_total_instances"),
        "Should have probe_total_instances"
    );
    assert!(
        metadata.contains_key("probe_successful"),
        "Should have probe_successful"
    );
    assert!(
        metadata.contains_key("probe_failures"),
        "Should have probe_failures"
    );
    assert!(
        metadata.contains_key("probe_models_discovered"),
        "Should have probe_models_discovered"
    );
}

#[test]
fn test_scan_graceful_error_handling_with_probing() {
    // Create test home directory
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create scan options with probing enabled
    let options = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: true,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: true,
        probe_timeout_secs: 5,
    };

    // Run scan - should succeed even if no instances are found
    let result = scan(&options).expect("Scan should succeed even with no instances");

    // Verify probe metadata exists
    let metadata = result.metadata.as_ref().expect("Metadata should exist");
    assert!(metadata.contains_key("probe_total_instances"));
    assert!(metadata.contains_key("probe_failures"));
}

#[test]
fn test_probe_statistics_in_metadata() {
    // Create test home directory
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create scan options with probing enabled
    let options = ScanOptions {
        home_dir: Some(temp_dir.path().to_path_buf()),
        include_full_values: true,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: true,
        probe_timeout_secs: 5,
    };

    // Run scan
    let result = scan(&options).expect("Scan should succeed");

    // Verify all expected probe statistics are present
    let metadata = result.metadata.as_ref().expect("Metadata should exist");
    assert!(
        metadata.contains_key("probe_total_instances"),
        "Should have probe_total_instances"
    );
    assert!(
        metadata.contains_key("probe_successful"),
        "Should have probe_successful"
    );
    assert!(
        metadata.contains_key("probe_failures"),
        "Should have probe_failures"
    );
    assert!(
        metadata.contains_key("probe_models_discovered"),
        "Should have probe_models_discovered"
    );

    // Verify the values are valid numbers
    let total = metadata["probe_total_instances"]
        .as_u64()
        .expect("Should be a number");
    let successful = metadata["probe_successful"]
        .as_u64()
        .expect("Should be a number");
    let failures = metadata["probe_failures"]
        .as_u64()
        .expect("Should be a number");
    let _models = metadata["probe_models_discovered"]
        .as_u64()
        .expect("Should be a number");

    // Basic sanity checks
    assert_eq!(
        total,
        successful + failures,
        "Total should equal successful + failures"
    );
}
