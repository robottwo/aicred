#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_excessive_bools)]
//! `ScanResult` model for collecting and querying discovered credentials.

use crate::models::config_instance::ConfigInstance;
use crate::models::credentials::{Confidence, DiscoveredCredential, ValueType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results from scanning for API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Discovered credentials.
    pub keys: Vec<DiscoveredCredential>,
    /// Configuration instances discovered.
    pub config_instances: Vec<ConfigInstance>,
    /// When the scan started.
    pub scan_started_at: DateTime<Utc>,
    /// When the scan completed.
    pub scan_completed_at: DateTime<Utc>,
    /// Home directory that was scanned.
    pub home_directory: String,
    /// Providers that were scanned.
    pub providers_scanned: Vec<String>,
    /// Total files scanned.
    pub files_scanned: u32,
    /// Total directories scanned.
    pub directories_scanned: u32,
    /// Scan metadata.
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ScanResult {
    /// Creates a new scan result.
    #[must_use]
    pub fn new(
        home_directory: String,
        providers_scanned: Vec<String>,
        scan_started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            keys: Vec::new(),
            config_instances: Vec::new(),
            scan_started_at,
            scan_completed_at: Utc::now(),
            home_directory,
            providers_scanned,
            files_scanned: 0,
            directories_scanned: 0,
            metadata: None,
        }
    }

    /// Adds a discovered key to the result.
    /// Only adds the key if no key with the same hash already exists.
    pub fn add_key(&mut self, key: DiscoveredCredential) {
        // Check if a key with the same hash already exists
        if self
            .keys
            .iter()
            .any(|existing_key| existing_key.hash == key.hash)
        {
            tracing::debug!(
                "Skipping duplicate key for provider: {} (hash: {})",
                key.provider,
                &key.hash[..8]
            );
        } else {
            tracing::debug!(
                "Adding key for provider: {} (hash: {})",
                key.provider,
                &key.hash[..8]
            );
            self.keys.push(key);
        }
    }

    /// Adds multiple discovered keys to the result.
    /// Only adds keys that don't have duplicate hashes.
    pub fn add_keys(&mut self, keys: Vec<DiscoveredCredential>) {
        for key in keys {
            self.add_key(key);
        }
    }

    /// Adds a configuration instance to the result.
    pub fn add_config_instance(&mut self, instance: ConfigInstance) {
        self.config_instances.push(instance);
    }

    /// Adds multiple configuration instances to the result.
    pub fn add_config_instances(&mut self, instances: Vec<ConfigInstance>) {
        self.config_instances.extend(instances);
    }

    /// Sets the scan completion time.
    pub fn set_completed(&mut self) {
        self.scan_completed_at = Utc::now();
    }

    /// Sets scan statistics.
    pub const fn set_stats(&mut self, files: u32, directories: u32) {
        self.files_scanned = files;
        self.directories_scanned = directories;
    }

    /// Sets additional metadata.
    pub fn set_metadata(&mut self, metadata: HashMap<String, serde_json::Value>) {
        self.metadata = Some(metadata);
    }

    /// Gets the total number of discovered keys.
    #[must_use]
    pub const fn total_keys(&self) -> usize {
        self.keys.len()
    }

    /// Gets the total number of configuration instances.
    #[must_use]
    pub const fn total_config_instances(&self) -> usize {
        self.config_instances.len()
    }

    /// Gets the number of keys by provider.
    #[must_use]
    pub fn keys_by_provider(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for key in &self.keys {
            *counts.entry(key.provider.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Gets the number of keys by type.
    #[must_use]
    pub fn keys_by_type(&self) -> HashMap<ValueType, usize> {
        let mut counts = HashMap::new();
        for key in &self.keys {
            *counts.entry(key.value_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Gets the number of keys by confidence level.
    #[must_use]
    pub fn keys_by_confidence(&self) -> HashMap<Confidence, usize> {
        let mut counts = HashMap::new();
        for key in &self.keys {
            *counts.entry(key.confidence).or_insert(0) += 1;
        }
        counts
    }

    /// Filters keys by provider.
    #[must_use]
    pub fn filter_by_provider(&self, provider: &str) -> Vec<&DiscoveredCredential> {
        self.keys
            .iter()
            .filter(|key| key.provider == provider)
            .collect()
    }

    /// Filters keys by confidence level (minimum confidence).
    #[must_use]
    pub fn filter_by_confidence(&self, min_confidence: Confidence) -> Vec<&DiscoveredCredential> {
        self.keys
            .iter()
            .filter(|key| key.confidence >= min_confidence)
            .collect()
    }

    /// Filters keys by type.
    #[must_use]
    pub fn filter_by_type(&self, value_type: &ValueType) -> Vec<&DiscoveredCredential> {
        self.keys
            .iter()
            .filter(|key| &key.value_type == value_type)
            .collect()
    }

    /// Gets high confidence keys (High and `VeryHigh`).
    #[must_use]
    pub fn high_confidence_credentials(&self) -> Vec<&DiscoveredCredential> {
        self.filter_by_confidence(Confidence::High)
    }

    /// Checks if any keys were found.
    #[must_use]
    pub const fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// Gets the scan duration in seconds.
    #[must_use]
    pub fn scan_duration(&self) -> f64 {
        let duration = self.scan_completed_at - self.scan_started_at;
        duration.num_milliseconds() as f64 / 1000.0
    }

    /// Gets a summary of the scan results.
    #[must_use]
    pub fn summary(&self) -> ScanSummary {
        ScanSummary {
            total_keys: self.total_keys(),
            total_config_instances: self.total_config_instances(),
            providers_found: self.keys_by_provider(),
            types_found: self.keys_by_type(),
            confidence_distribution: self.keys_by_confidence(),
            files_scanned: self.files_scanned,
            directories_scanned: self.directories_scanned,
            scan_duration: self.scan_duration(),
        }
    }
}

/// Summary statistics for a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    /// Total number of keys found.
    pub total_keys: usize,
    /// Total number of configuration instances found.
    pub total_config_instances: usize,
    /// Number of keys by provider.
    pub providers_found: HashMap<String, usize>,
    /// Number of keys by type.
    pub types_found: HashMap<ValueType, usize>,
    /// Distribution of confidence levels.
    pub confidence_distribution: HashMap<Confidence, usize>,
    /// Number of files scanned.
    pub files_scanned: u32,
    /// Number of directories scanned.
    pub directories_scanned: u32,
    /// Duration of the scan in seconds.
    pub scan_duration: f64,
}

impl ScanSummary {
    /// Gets the number of high confidence keys.
    #[must_use]
    pub fn high_confidence_count(&self) -> usize {
        self.confidence_distribution
            .iter()
            .filter(|(confidence, _)| **confidence >= Confidence::High)
            .map(|(_, count)| count)
            .sum()
    }

    /// Gets the number of medium confidence keys.
    #[must_use]
    pub fn medium_confidence_count(&self) -> usize {
        self.confidence_distribution
            .get(&Confidence::Medium)
            .copied()
            .unwrap_or(0)
    }

    /// Gets the number of low confidence keys.
    #[must_use]
    pub fn low_confidence_count(&self) -> usize {
        self.confidence_distribution
            .get(&Confidence::Low)
            .copied()
            .unwrap_or(0)
    }

    /// Checks if any keys were found.
    #[must_use]
    pub const fn has_keys(&self) -> bool {
        self.total_keys > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::credentials::DiscoveredCredential;

    fn create_test_credential(
        provider: &str,
        value_type: ValueType,
        confidence: Confidence,
    ) -> DiscoveredCredential {
        // Create different credential values based on provider and confidence to avoid deduplication in tests
        let credential_value = format!("test-credential-{provider}-{confidence:?}");
        DiscoveredCredential::new_redacted(
            provider.to_string(),
            "/test".to_string(),
            value_type,
            confidence,
            &credential_value,
        )
    }

    #[test]
    fn test_scan_result_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        let result = ScanResult::new(
            home_path.clone(),
            vec!["openai".to_string(), "anthropic".to_string()],
            Utc::now(),
        );

        assert_eq!(result.home_directory, home_path);
        assert_eq!(result.providers_scanned.len(), 2);
        assert_eq!(result.total_keys(), 0);
        assert_eq!(result.total_config_instances(), 0);
        assert!(!result.has_keys());
    }

    #[test]
    fn test_adding_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        let mut result = ScanResult::new(home_path, vec!["openai".to_string()], Utc::now());

        let key1 = create_test_credential("openai", ValueType::ApiKey, Confidence::High);
        let key2 = create_test_credential("anthropic", ValueType::ApiKey, Confidence::Medium);

        result.add_key(key1);
        result.add_key(key2);

        assert_eq!(result.total_keys(), 2); // 2 keys (different hashes due to different providers)
        assert!(result.has_keys());
    }

    #[test]
    fn test_filtering_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home_path = temp_dir.path().to_string_lossy().to_string();
        let mut result = ScanResult::new(home_path, vec![], Utc::now());

        result.add_key(create_test_credential(
            "openai",
            ValueType::ApiKey,
            Confidence::High,
        ));
        result.add_key(create_test_credential(
            "anthropic",
            ValueType::ApiKey,
            Confidence::Medium,
        ));
        result.add_key(create_test_credential(
            "google",
            ValueType::SecretKey,
            Confidence::Low,
        ));

        let openai_keys = result.filter_by_provider("openai");
        assert_eq!(openai_keys.len(), 1);

        let high_confidence = result.filter_by_confidence(Confidence::High);
        assert_eq!(high_confidence.len(), 1);

        let api_keys = result.filter_by_type(&ValueType::ApiKey);
        assert_eq!(api_keys.len(), 2); // 2 API keys (different hashes due to different providers)
    }

    #[test]
    fn test_key_statistics() {
        let mut result = ScanResult::new("/home/user".to_string(), vec![], Utc::now());

        result.add_key(create_test_credential(
            "openai",
            ValueType::ApiKey,
            Confidence::High,
        ));
        result.add_key(create_test_credential(
            "openai",
            ValueType::ApiKey,
            Confidence::Medium,
        ));
        result.add_key(create_test_credential(
            "anthropic",
            ValueType::SecretKey,
            Confidence::High,
        ));

        let by_provider = result.keys_by_provider();
        assert_eq!(by_provider.get("openai"), Some(&2)); // 2 OpenAI keys (different hashes)
        assert_eq!(by_provider.get("anthropic"), Some(&1));

        let by_type = result.keys_by_type();
        assert_eq!(by_type.get(&ValueType::ApiKey), Some(&2)); // 2 API keys (different hashes)
        assert_eq!(by_type.get(&ValueType::SecretKey), Some(&1));
    }

    #[test]
    fn test_scan_summary() {
        let mut result = ScanResult::new("/home/user".to_string(), vec![], Utc::now());
        result.set_stats(100, 20);

        result.add_key(create_test_credential(
            "openai",
            ValueType::ApiKey,
            Confidence::High,
        ));
        result.add_key(create_test_credential(
            "anthropic",
            ValueType::ApiKey,
            Confidence::Medium,
        ));

        let summary = result.summary();
        assert_eq!(summary.total_keys, 2); // 2 keys (different hashes due to different providers)
        assert_eq!(summary.total_config_instances, 0);
        assert_eq!(summary.files_scanned, 100);
        assert_eq!(summary.directories_scanned, 20);
        assert_eq!(summary.high_confidence_count(), 1);
        assert_eq!(summary.medium_confidence_count(), 1);
        assert_eq!(summary.low_confidence_count(), 0);
    }

    #[test]
    fn test_config_instances() {
        let mut result = ScanResult::new("/home/user".to_string(), vec![], Utc::now());

        let mut instance1 = ConfigInstance::new(
            "instance-1".to_string(),
            "test-app".to_string(),
            std::path::PathBuf::from("/path/1"),
        );
        instance1.add_key(create_test_credential(
            "openai",
            ValueType::ApiKey,
            Confidence::High,
        ));

        let mut instance2 = ConfigInstance::new(
            "instance-2".to_string(),
            "test-app".to_string(),
            std::path::PathBuf::from("/path/2"),
        );
        instance2.add_key(create_test_credential(
            "anthropic",
            ValueType::ApiKey,
            Confidence::Medium,
        ));

        result.add_config_instance(instance1);
        result.add_config_instance(instance2);

        assert_eq!(result.total_config_instances(), 2);
        assert_eq!(result.total_keys(), 0); // Keys are in instances, not directly in result
    }
}
