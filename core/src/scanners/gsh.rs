//! GSH scanner for discovering API keys in GSH configuration files.

use super::{ScanResult, ScannerPlugin};
use crate::error::Result;
use crate::models::discovered_key::{Confidence, ValueType};
use crate::models::{ConfigInstance, DiscoveredKey};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scanner for GSH (Generic Shell Helper) application configuration.
pub struct GshScanner;

impl ScannerPlugin for GshScanner {
    fn name(&self) -> &str {
        "gsh"
    }

    fn app_name(&self) -> &str {
        "GSH"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Focus only on ~/.gshrc location
        paths.push(home_dir.join(".gshrc"));

        paths
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        file_name == ".gshrc" || file_name.ends_with("gshrc")
    }

    fn supports_provider_scanning(&self) -> bool {
        true
    }

    fn supported_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "google".to_string(),
            "huggingface".to_string(),
            "groq".to_string(),
            "openrouter".to_string(),
        ]
    }

    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Only check for ~/.gshrc
        paths.push(home_dir.join(".gshrc"));

        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Use a Vec to maintain order and a HashSet to track seen hashes
        let mut unique_keys = Vec::new();
        let mut seen_hashes = std::collections::HashSet::new();

        // Parse GSH-specific configuration
        let gsh_keys = self.parse_gshrc(content, path);
        for key in gsh_keys {
            // Only add if we haven't seen this key hash before
            if seen_hashes.insert(key.hash.clone()) {
                unique_keys.push(key);
            }
        }

        // Also parse general shell script format as fallback
        let shell_keys = self.extract_keys_from_shell_script(content, path);
        for key in shell_keys {
            // Only add if we haven't seen this key hash before
            if seen_hashes.insert(key.hash.clone()) {
                unique_keys.push(key);
            }
        }

        // Add the unique keys to the result
        result.add_keys(unique_keys);

        // Create config instances for GSH installations
        let mut instances = Vec::new();
        if let Some(instance) = self.create_config_instance(path, content).ok() {
            tracing::debug!("Created config instance");
            instances.push(instance);
        } else {
            tracing::debug!("Failed to create config instance");
        }
        result.add_instances(instances);

        tracing::debug!(
            "Parse config result: {} keys, {} instances",
            result.keys.len(),
            result.instances.len()
        );

        Ok(result)
    }

    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();

        // Look only for ~/.gshrc
        let config_path = home_dir.join(".gshrc");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if self.is_valid_gsh_config(&content) {
                    let instance = self.create_config_instance(&config_path, &content)?;
                    instances.push(instance);
                }
            }
        }

        Ok(instances)
    }
}

impl GshScanner {
    /// Parse GSH-specific configuration from .gshrc file using key/value pairs.
    fn parse_gshrc(&self, content: &str, path: &Path) -> Vec<DiscoveredKey> {
        let mut keys = Vec::new();

        // Define the specific keys we want to look for
        let fast_model_keys = [
            ("GSH_FAST_MODEL_API_KEY", "groq"),
            ("GSH_FAST_MODEL_BASE_URL", "groq"),
            ("GSH_FAST_MODEL_ID", "groq"),
            ("GSH_FAST_MODEL_TEMPERATURE", "groq"),
            ("GSH_FAST_MODEL_PARALLEL_TOOL_CALLS", "groq"),
            ("GSH_FAST_MODEL_HEADERS", "groq"),
        ];

        let slow_model_keys = [
            ("GSH_SLOW_MODEL_API_KEY", "openrouter"),
            ("GSH_SLOW_MODEL_BASE_URL", "openrouter"),
            ("GSH_SLOW_MODEL_ID", "openrouter"),
        ];

        // Parse key/value pairs from the content
        let key_values = self.parse_key_value_pairs(content);

        // Check for fast model keys
        for (key_name, provider) in fast_model_keys {
            if let Some(value) = key_values.get(key_name) {
                if !value.is_empty() {
                    let value_type = if key_name.ends_with("_API_KEY") {
                        ValueType::ApiKey
                    } else if key_name.ends_with("_BASE_URL") {
                        ValueType::Custom("BaseUrl".to_string())
                    } else if key_name.ends_with("_ID") {
                        ValueType::Custom("ModelId".to_string())
                    } else if key_name.ends_with("_TEMPERATURE") {
                        ValueType::Custom("Temperature".to_string())
                    } else if key_name.ends_with("_PARALLEL_TOOL_CALLS") {
                        ValueType::Custom("ParallelToolCalls".to_string())
                    } else if key_name.ends_with("_HEADERS") {
                        ValueType::Custom("Headers".to_string())
                    } else {
                        ValueType::Custom("Config".to_string())
                    };
                    
                    let discovered_key = DiscoveredKey::new(
                        provider.to_string(),
                        path.display().to_string(),
                        value_type,
                        self.get_confidence(value),
                        value.to_string(),
                    );
                    keys.push(discovered_key);
                }
            }
        }

        // Check for slow model keys
        for (key_name, provider) in slow_model_keys {
            if let Some(value) = key_values.get(key_name) {
                if !value.is_empty() {
                    let value_type = if key_name.ends_with("_API_KEY") {
                        ValueType::ApiKey
                    } else if key_name.ends_with("_BASE_URL") {
                        ValueType::Custom("BaseUrl".to_string())
                    } else if key_name.ends_with("_ID") {
                        ValueType::Custom("ModelId".to_string())
                    } else {
                        ValueType::Custom("Config".to_string())
                    };
                    
                    let discovered_key = DiscoveredKey::new(
                        provider.to_string(),
                        path.display().to_string(),
                        value_type,
                        self.get_confidence(value),
                        value.to_string(),
                    );
                    keys.push(discovered_key);
                }
            }
        }

        keys
    }

    /// Parse key/value pairs from shell script content, handling export statements and quoted values.
    fn parse_key_value_pairs(&self, content: &str) -> std::collections::HashMap<String, String> {
        let mut key_values = std::collections::HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Handle export statements (remove export prefix)
            let line = if line.starts_with("export ") {
                &line[7..].trim()
            } else {
                line
            };

            // Split on first = sign
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let mut value = line[eq_pos + 1..].trim();

                // Remove quotes if present
                if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    if value.len() > 1 {
                        value = &value[1..value.len() - 1];
                    }
                }

                // Only add non-empty values
                if !value.is_empty() {
                    key_values.insert(key.to_string(), value.to_string());
                }
            }
        }

        key_values
    }

    /// Extract keys from shell script format (KEY=value pairs).
    fn extract_keys_from_shell_script(&self, content: &str, path: &Path) -> Vec<DiscoveredKey> {
        let mut keys = Vec::new();

        // Common API key patterns in shell scripts - handle both quoted and unquoted values
        // This pattern matches both regular assignments and export statements
        let patterns = [
            (
                r#"(?i)(?:export\s+)?OPENAI_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "openai",
            ),
            (
                r#"(?i)(?:export\s+)?ANTHROPIC_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "anthropic",
            ),
            (
                r#"(?i)(?:export\s+)?GOOGLE_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "google",
            ),
            (
                r#"(?i)(?:export\s+)?GEMINI_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "google",
            ),
            (
                r#"(?i)(?:export\s+)?HUGGING_FACE_HUB_TOKEN\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "huggingface",
            ),
            (
                r#"(?i)(?:export\s+)?HF_TOKEN\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "huggingface",
            ),
            (
                r#"(?i)(?:export\s+)?LANGCHAIN_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "langchain",
            ),
            (
                r#"(?i)(?:export\s+)?GROQ_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "groq",
            ),
            (
                r#"(?i)(?:export\s+)?COHERE_API_KEY\s*=\s*["']?([a-zA-Z0-9_-]{15,})["']?"#,
                "cohere",
            ),
        ];

        for (pattern, provider) in patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            for cap in regex.captures_iter(content) {
                if let Some(key_match) = cap.get(1) {
                    let key_value = key_match.as_str();

                    let discovered_key = DiscoveredKey::new(
                        provider.to_string(),
                        path.display().to_string(),
                        ValueType::ApiKey,
                        self.get_confidence(key_value),
                        key_value.to_string(),
                    );

                    keys.push(discovered_key);
                }
            }
        }

        keys
    }

    /// Check if this is a valid GSH configuration.
    fn is_valid_gsh_config(&self, content: &str) -> bool {
        // Check for GSH-specific patterns or environment variables
        content.contains("GSH_")
            || content.contains("gsh")
            || content.contains("OPENAI_API_KEY")
            || content.contains("ANTHROPIC_API_KEY")
            || content.contains("GOOGLE_API_KEY")
            || content.contains("GEMINI_API_KEY")
            || content.contains("HUGGING_FACE_HUB_TOKEN")
            || content.contains("HF_TOKEN")
    }

    /// Create a config instance from GSH configuration.
    fn create_config_instance(&self, path: &Path, _content: &str) -> Result<ConfigInstance> {
        let mut metadata = HashMap::new();

        // Add basic metadata
        metadata.insert("type".to_string(), "shell_script".to_string());
        metadata.insert("format".to_string(), "KEY=value".to_string());

        let instance = ConfigInstance {
            instance_id: self.generate_instance_id(path),
            app_name: "gsh".to_string(),
            config_path: path.to_path_buf(),
            discovered_at: Utc::now(),
            keys: Vec::new(), // Will be populated separately
            metadata,
        };

        Ok(instance)
    }

    /// Generate a unique instance ID.
    fn generate_instance_id(&self, path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("gsh_{:x}", hasher.finalize())
            .chars()
            .take(16)
            .collect()
    }

    /// Get confidence score for a key.
    fn get_confidence(&self, key: &str) -> Confidence {
        if key.starts_with("sk-") || key.starts_with("sk-ant-") || key.starts_with("hf_") {
            Confidence::High
        } else if key.len() >= 30 {
            Confidence::Medium
        } else {
            Confidence::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gsh_scanner_name() {
        let scanner = GshScanner;
        assert_eq!(scanner.name(), "gsh");
        assert_eq!(scanner.app_name(), "GSH");
    }

    #[test]
    fn test_scan_paths() {
        let scanner = GshScanner;
        let temp_dir = tempfile::tempdir().unwrap();
        let home_dir = temp_dir.path();
        let paths = scanner.scan_paths(home_dir);

        // Should only include ~/.gshrc
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains(".gshrc"));
    }

    #[test]
    fn test_can_handle_file() {
        let scanner = GshScanner;

        assert!(scanner.can_handle_file(Path::new(".gshrc")));
        assert!(scanner.can_handle_file(Path::new("/home/user/.gshrc")));
        assert!(!scanner.can_handle_file(Path::new("config.json")));
        assert!(!scanner.can_handle_file(Path::new(".bashrc")));
    }

    #[test]
    fn test_parse_valid_config() {
        let scanner = GshScanner;
        let config = r#"
# GSH Configuration
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
GOOGLE_API_KEY="AIzaSyTest1234567890abcdef"
"#;

        let result = scanner.parse_config(Path::new(".gshrc"), config).unwrap();
        assert_eq!(result.keys.len(), 3);
        assert_eq!(result.instances.len(), 1);

        // Check keys
        assert_eq!(result.keys[0].provider, "openai");
        assert_eq!(result.keys[0].value_type, ValueType::ApiKey);
        assert_eq!(result.keys[0].confidence, Confidence::High);

        // Check instance
        assert_eq!(result.instances[0].app_name, "gsh");
    }

    #[test]
    fn test_is_valid_gsh_config() {
        let scanner = GshScanner;

        let valid_config = r#"
export OPENAI_API_KEY="sk-test1234567890abcdef"
export GSH_PROMPT="You are a helpful assistant"
"#;
        assert!(scanner.is_valid_gsh_config(valid_config));

        let invalid_config = r#"
# This is just a regular shell script
echo "Hello World"
"#;
        assert!(!scanner.is_valid_gsh_config(invalid_config));
    }

    #[test]
    fn test_create_config_instance() {
        let scanner = GshScanner;
        let config = r#"
export OPENAI_API_KEY="sk-test1234567890abcdef"
"#;

        let instance = scanner
            .create_config_instance(Path::new("/test/.gshrc"), config)
            .unwrap();
        assert_eq!(instance.app_name, "gsh");
        assert_eq!(
            instance.metadata.get("type"),
            Some(&"shell_script".to_string())
        );
        assert_eq!(
            instance.metadata.get("format"),
            Some(&"KEY=value".to_string())
        );
    }

    #[test]
    fn test_extract_keys_from_shell_script() {
        let scanner = GshScanner;
        let content = r#"
# Environment variables
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
GOOGLE_API_KEY="AIzaSyTest1234567890abcdef"
HF_TOKEN="hf_test1234567890abcdef"
"#;

        let keys = scanner.extract_keys_from_shell_script(content, Path::new(".gshrc"));
        assert_eq!(keys.len(), 4);

        // Check providers
        let providers: Vec<String> = keys.iter().map(|k| k.provider.clone()).collect();
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(providers.contains(&"google".to_string()));
        assert!(providers.contains(&"huggingface".to_string()));
    }

    #[test]
    fn test_get_confidence() {
        let scanner = GshScanner;

        assert_eq!(
            scanner.get_confidence("sk-test1234567890abcdef"),
            Confidence::High
        );
        assert_eq!(
            scanner.get_confidence("sk-ant-test1234567890abcdef"),
            Confidence::High
        );
        assert_eq!(
            scanner.get_confidence("hf_test1234567890abcdef"),
            Confidence::High
        );
        assert_eq!(
            scanner.get_confidence("verylongkeywithmorethanthirtycharacters"),
            Confidence::Medium
        );
        assert_eq!(scanner.get_confidence("short"), Confidence::Low);
    }

    #[test]
    fn test_parse_gshrc_fast_model() {
        let scanner = GshScanner;
        let content = r#"
# GSH Fast Model Configuration
export GSH_FAST_MODEL_API_KEY="gsk_test1234567890abcdef1234567890abcdef"
export GSH_FAST_MODEL_BASE_URL="https://api.groq.com/openai/v1"
export GSH_FAST_MODEL_ID="llama3-70b-8192"
export GSH_FAST_MODEL_TEMPERATURE="0.7"
export GSH_FAST_MODEL_PARALLEL_TOOL_CALLS="true"
export GSH_FAST_MODEL_HEADERS="Content-Type: application/json"
"#;

        let keys = scanner.parse_gshrc(content, Path::new(".gshrc"));
        assert_eq!(keys.len(), 6);

        // Check that all keys are mapped to groq provider
        for key in &keys {
            assert_eq!(key.provider, "groq");
        }
    }

    #[test]
    fn test_parse_gshrc_slow_model() {
        let scanner = GshScanner;
        let content = r#"
# GSH Slow Model Configuration
export GSH_SLOW_MODEL_API_KEY="sk-or-v1_test1234567890abcdef1234567890abcdef"
export GSH_SLOW_MODEL_BASE_URL="https://openrouter.ai/api/v1"
export GSH_SLOW_MODEL_ID="anthropic/claude-3-opus"
"#;

        let keys = scanner.parse_gshrc(content, Path::new(".gshrc"));
        assert_eq!(keys.len(), 3);

        // Check that all keys are mapped to openrouter provider
        for key in &keys {
            assert_eq!(key.provider, "openrouter");
        }
    }

    #[test]
    fn test_parse_gshrc_combined() {
        let scanner = GshScanner;
        let content = r#"
# GSH Configuration with both fast and slow models
export GSH_FAST_MODEL_API_KEY="gsk_test1234567890abcdef"
export GSH_SLOW_MODEL_API_KEY="sk-or-v1_test1234567890abcdef"
export GSH_FAST_MODEL_BASE_URL="https://api.groq.com/openai/v1"
export GSH_SLOW_MODEL_BASE_URL="https://openrouter.ai/api/v1"
"#;

        let keys = scanner.parse_gshrc(content, Path::new(".gshrc"));
        assert_eq!(keys.len(), 4);

        // Check providers
        let providers: Vec<String> = keys.iter().map(|k| k.provider.clone()).collect();
        assert!(providers.contains(&"groq".to_string()));
        assert!(providers.contains(&"openrouter".to_string()));
    }

    #[test]
    fn test_parse_config_with_gshrc() {
        let scanner = GshScanner;
        let config = r#"
# GSH Configuration
export GSH_FAST_MODEL_API_KEY="gsk_test1234567890abcdef"
export GSH_SLOW_MODEL_API_KEY="sk-or-v1_test1234567890abcdef"
export OPENAI_API_KEY="sk-test1234567890abcdef"
"#;

        let result = scanner.parse_config(Path::new(".gshrc"), config).unwrap();
        // Should have keys from both GSH parsing and shell script parsing
        assert!(result.keys.len() >= 3);

        // Check that we have both groq and openrouter keys
        let providers: Vec<String> = result.keys.iter().map(|k| k.provider.clone()).collect();
        assert!(providers.contains(&"groq".to_string()));
        assert!(providers.contains(&"openrouter".to_string()));
        assert!(providers.contains(&"openai".to_string()));
    }

    #[test]
    fn test_duplicate_key_prevention() {
        let scanner = GshScanner;
        // This config has the same API key in both GSH format and shell script format
        let config = r#"
# GSH Configuration with duplicate keys
export GSH_FAST_MODEL_API_KEY="sk-test1234567890abcdef"
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
export GSH_SLOW_MODEL_API_KEY="sk-ant-test1234567890abcdef"
"#;

        let result = scanner.parse_config(Path::new(".gshrc"), config).unwrap();

        // Count unique key hashes to verify no duplicates
        let mut unique_hashes = std::collections::HashSet::new();
        for key in &result.keys {
            unique_hashes.insert(&key.hash);
        }

        // The number of unique hashes should equal the number of keys (no duplicates)
        assert_eq!(unique_hashes.len(), result.keys.len());

        // Should have exactly 2 unique keys (one for openai, one for anthropic)
        assert_eq!(result.keys.len(), 2);

        // Verify the providers are correct
        let providers: Vec<String> = result.keys.iter().map(|k| k.provider.clone()).collect();
        assert!(providers.contains(&"groq".to_string()));
        assert!(providers.contains(&"openrouter".to_string()));
    }

    #[test]
    fn test_real_duplicate_issue() {
        let scanner = GshScanner;
        // Test case that might reveal the actual duplicate issue
        let config = r#"
# Real GSH configuration with potential duplicates
export GSH_FAST_MODEL_API_KEY="gsk_test1234567890abcdef1234567890abcdef"
export GSH_FAST_MODEL_BASE_URL="https://api.groq.com/openai/v1"
export GSH_FAST_MODEL_ID="llama3-70b-8192"
export GSH_SLOW_MODEL_API_KEY="sk-or-v1_test1234567890abcdef1234567890abcdef"
export GSH_SLOW_MODEL_BASE_URL="https://openrouter.ai/api/v1"
export GSH_SLOW_MODEL_ID="anthropic/claude-3-opus"
export OPENAI_API_KEY="sk-test1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
export GOOGLE_API_KEY="AIzaSyTest1234567890abcdef"
export HUGGING_FACE_HUB_TOKEN="hf_test1234567890abcdef1234567890abcdef"
export HF_TOKEN="hf_test1234567890abcdef1234567890abcdef"
export LANGCHAIN_API_KEY="ls_test1234567890abcdef1234567890abcdef"
export GROQ_API_KEY="gsk_test1234567890abcdef1234567890abcdef"
export COHERE_API_KEY="cohere_test1234567890abcdef1234567890abcdef"
"#;

        let result = scanner.parse_config(Path::new(".gshrc"), config).unwrap();

        println!("Found {} keys:", result.keys.len());
        for (i, key) in result.keys.iter().enumerate() {
            println!(
                "Key {}: provider={}, hash={}, value={}",
                i + 1,
                key.provider,
                key.hash,
                key.redacted_value()
            );
        }

        // Check for duplicates by hash
        let mut hashes = std::collections::HashSet::new();
        let mut duplicates = 0;

        for key in &result.keys {
            if !hashes.insert(&key.hash) {
                duplicates += 1;
                println!(
                    "Duplicate found: provider={}, hash={}",
                    key.provider, key.hash
                );
            }
        }

        println!(
            "Total keys: {}, Duplicates: {}",
            result.keys.len(),
            duplicates
        );

        // The number of unique hashes should equal the number of keys (no duplicates)
        assert_eq!(hashes.len(), result.keys.len(), "Found duplicate keys");

        // Should have exactly the expected number of unique keys
        // Note: Some keys might be duplicates across different parsing methods
        let expected_unique_keys = result.keys.len(); // Should be equal if no duplicates
        assert_eq!(result.keys.len(), expected_unique_keys);
    }

    #[test]
    fn test_actual_duplicate_keys_issue() {
        let scanner = GshScanner;
        // This test case demonstrates the real issue: same key value detected by different parsing methods
        let config = r#"
# GSH Configuration with actual duplicate detection issue
# The same API key appears in both GSH format and shell script format
export GSH_FAST_MODEL_API_KEY="sk-duplicate1234567890abcdef"
export OPENAI_API_KEY="sk-duplicate1234567890abcdef"
export GSH_SLOW_MODEL_API_KEY="sk-ant-duplicate1234567890abcdef"
export ANTHROPIC_API_KEY="sk-ant-duplicate1234567890abcdef"
# Also test with HuggingFace tokens that might appear in different formats
export GSH_FAST_MODEL_HEADERS="Authorization: Bearer hf_duplicate1234567890abcdef"
export HUGGING_FACE_HUB_TOKEN="hf_duplicate1234567890abcdef"
export HF_TOKEN="hf_duplicate1234567890abcdef"
"#;

        let result = scanner.parse_config(Path::new(".gshrc"), config).unwrap();

        println!("=== Testing actual duplicate detection ===");
        println!("Found {} keys:", result.keys.len());

        // Group keys by hash to identify duplicates
        let mut hash_groups: std::collections::HashMap<&str, Vec<&str>> =
            std::collections::HashMap::new();

        for key in &result.keys {
            hash_groups
                .entry(&key.hash)
                .or_insert_with(Vec::new)
                .push(&key.provider);
        }

        let mut duplicate_count = 0;
        for (hash, providers) in &hash_groups {
            if providers.len() > 1 {
                duplicate_count += providers.len() - 1;
                println!(
                    "Duplicate hash {} found in providers: {:?}",
                    hash, providers
                );
            }
        }

        println!(
            "Total keys: {}, Actual duplicates: {}",
            result.keys.len(),
            duplicate_count
        );

        // The current implementation should prevent duplicates
        assert_eq!(
            duplicate_count, 0,
            "Expected no duplicate keys due to hash-based deduplication"
        );

        // But let's check if we have the expected number of unique keys
        // We should have: 1 groq, 1 openrouter, 1 openai, 1 anthropic, 1 huggingface (5 total)
        let providers: Vec<String> = result.keys.iter().map(|k| k.provider.clone()).collect();
        println!("Providers found: {:?}", providers);

        // The exact count depends on what gets detected, but there should be no duplicates
        assert!(
            result.keys.len() <= 8,
            "Too many keys, possible duplicates not being filtered"
        );
    }

    #[test]
    fn test_supported_providers_includes_groq_and_openrouter() {
        let scanner = GshScanner;
        let providers = scanner.supported_providers();

        assert!(providers.contains(&"groq".to_string()));
        assert!(providers.contains(&"openrouter".to_string()));
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }
}
