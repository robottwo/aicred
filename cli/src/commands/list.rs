use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

/// Import the ProviderConfig from core library
use genai_keyfinder_core::models::ProviderConfig;

/// Provider information calculated from provider files
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider name (derived from file name)
    pub name: String,
    /// File name for this provider (without extension)
    pub file_name: String,
    /// Provider configuration
    pub config: ProviderConfig,
    /// Total number of keys for this provider
    pub key_count: u32,
    /// Number of active keys for this provider
    pub active_key_count: u32,
    /// Keys grouped by environment
    pub keys_by_environment: HashMap<String, u32>,
}

pub fn handle_list(verbose: bool) -> Result<()> {
    // Read from new configuration directory structure
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    let providers_dir = config_dir.join("providers");
    let old_config_path = config_dir.join("providers.yaml");

    // Check if we need to migrate from old format
    if old_config_path.exists() && !providers_dir.exists() {
        println!("{}", "Migrating from old single-file format to new multi-file format...".yellow());
        migrate_from_single_file(&old_config_path, &providers_dir)?;
    }

    if !providers_dir.exists() {
        println!("{}", "No configuration found. Run 'keyfinder scan --update' first.".yellow());
        return Ok(());
    }

    // Scan providers directory for all .yaml files
    let providers = scan_providers_directory(&providers_dir)?;

    if providers.is_empty() {
        println!("{}", "No providers configured.".yellow());
        return Ok(());
    }

    println!("\n{}", "Configured Providers:".green().bold());
    
    // Find the most recent update timestamp across all providers
    let last_updated = providers.iter()
        .map(|p| p.config.updated_at)
        .max()
        .unwrap_or_else(|| chrono::Utc::now());
    
    println!("{}", format!("Last updated: {}", last_updated.format("%Y-%m-%d %H:%M:%S UTC")).dimmed());
    
    for provider_info in &providers {
        if verbose {
            println!("\n{} {}", provider_info.name.cyan().bold(), format!("(v{})", provider_info.config.version).dimmed());
            
            // Show key information
            println!("  Keys: {} total, {} active", provider_info.key_count, provider_info.active_key_count);
            
            // Show environment breakdown
            if !provider_info.keys_by_environment.is_empty() {
                let env_display: Vec<String> = provider_info.keys_by_environment.iter()
                    .map(|(env, count)| format!("{}: {}", env, count))
                    .collect();
                println!("  Environments: {}", env_display.join(", "));
            }
            
            // Show models
            if !provider_info.config.models.is_empty() {
                println!("  Models: {}", provider_info.config.models.join(", "));
            }
            
            // Show metadata if available
            if let Some(metadata_map) = &provider_info.config.metadata {
                for (key, value) in metadata_map {
                    println!("  {}: {:?}", key, value);
                }
            }
            
            println!("  Schema Version: {}", provider_info.config.schema_version);
            println!("  Created: {}", provider_info.config.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("  Updated: {}", provider_info.config.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        } else {
            println!("  {}", provider_info.name.cyan());
            
            // Show key status and model count
            let model_count = if provider_info.config.models.is_empty() {
                "No models".to_string()
            } else {
                format!("{} models", provider_info.config.models.len())
            };
            println!("    {} keys, {}", provider_info.key_count, model_count);
        }
    }

    println!(
        "\n{}",
        format!("Total providers: {}", providers.len()).cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_provider_name_basic() {
        assert_eq!(sanitize_provider_name("OpenAI"), "openai");
        assert_eq!(sanitize_provider_name("Anthropic"), "anthropic");
        assert_eq!(sanitize_provider_name("Google Cloud"), "google_cloud");
    }

    #[test]
    fn test_sanitize_provider_name_special_chars() {
        assert_eq!(sanitize_provider_name("OpenAI/ChatGPT"), "openai_chatgpt");
        assert_eq!(sanitize_provider_name("Azure-OpenAI"), "azure-openai");
        assert_eq!(sanitize_provider_name("AWS Bedrock v2.0"), "aws_bedrock_v2_0");
        assert_eq!(sanitize_provider_name("Provider@Home"), "provider_home");
        assert_eq!(sanitize_provider_name("Test[Provider]"), "test_provider");
    }

    #[test]
    fn test_sanitize_provider_name_consecutive_underscores() {
        assert_eq!(sanitize_provider_name("OpenAI  ChatGPT"), "openai_chatgpt");
        assert_eq!(sanitize_provider_name("Test___Provider"), "test_provider");
        assert_eq!(sanitize_provider_name("A__B__C"), "a_b_c");
    }

    #[test]
    fn test_sanitize_provider_name_edge_cases() {
        assert_eq!(sanitize_provider_name(""), "provider");
        assert_eq!(sanitize_provider_name("___"), "provider");
        assert_eq!(sanitize_provider_name("123"), "123");
        assert_eq!(sanitize_provider_name("___test___"), "test");
    }

    #[test]
    fn test_sanitize_provider_name_path_traversal() {
        assert_eq!(sanitize_provider_name("../../../etc/passwd"), "etc_passwd");
        assert_eq!(sanitize_provider_name("..\\..\\windows\\system32"), "windows_system32");
        assert_eq!(sanitize_provider_name("provider/../evil"), "provider_evil");
    }

    #[test]
    fn test_sanitize_provider_name_unicode() {
        assert_eq!(sanitize_provider_name("Café"), "caf");
        assert_eq!(sanitize_provider_name("北京"), "provider");
        assert_eq!(sanitize_provider_name("тест"), "provider");
    }
}

/// Scans the providers directory and returns provider information
fn scan_providers_directory(providers_dir: &PathBuf) -> Result<Vec<ProviderInfo>> {
    let mut providers = Vec::new();
    
    if !providers_dir.exists() {
        return Ok(providers);
    }

    // Read all .yaml files in the providers directory
    let entries = std::fs::read_dir(providers_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .yaml and .yml files (case-insensitive)
        if path.extension().map_or(false, |ext| {
            ext.to_str().map_or(false, |s| {
                s.eq_ignore_ascii_case("yaml") || s.eq_ignore_ascii_case("yml")
            })
        }) {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        match ProviderConfig::from_yaml(&content) {
                            Ok(config) => {
                                // Calculate provider information
                                let key_count = config.keys.len() as u32;
                                let active_key_count = config.keys.iter()
                                    .filter(|k| k.validation_status == genai_keyfinder_core::models::ValidationStatus::Valid)
                                    .count() as u32;
                                
                                let keys_by_environment = config.keys.iter()
                                    .fold(HashMap::new(), |mut acc, key| {
                                        let env_str = format!("{:?}", key.environment);
                                        *acc.entry(env_str).or_insert(0) += 1;
                                        acc
                                    });
                                
                                // Convert file name to provider name (capitalize and replace underscores)
                                let provider_name = file_stem
                                    .split('_')
                                    .map(|word| {
                                        let mut chars = word.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(" ");
                                
                                providers.push(ProviderInfo {
                                    name: provider_name,
                                    file_name: file_stem.to_string(),
                                    config,
                                    key_count,
                                    active_key_count,
                                    keys_by_environment,
                                });
                            }
                            Err(e) => {
                                eprintln!("{} {}: {}", "Error parsing provider file:".red(), path.display(), e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}: {}", "Error reading provider file:".red(), path.display(), e);
                    }
                }
            }
        }
    }
    
    // Sort providers by name for consistent output
    providers.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(providers)
}

/// Sanitizes a provider name to prevent path traversal and OS issues
fn sanitize_provider_name(name: &str) -> String {
    // Convert to lowercase
    let mut sanitized = name.to_lowercase();
    
    // Replace any character not in [a-z0-9_-] with underscore
    sanitized = sanitized.chars().map(|c| {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
            c
        } else {
            '_'
        }
    }).collect();
    
    // Collapse consecutive underscores into single underscore
    let mut result = String::new();
    let mut prev_was_underscore = false;
    for c in sanitized.chars() {
        if c == '_' {
            if !prev_was_underscore {
                result.push(c);
                prev_was_underscore = true;
            }
        } else {
            result.push(c);
            prev_was_underscore = false;
        }
    }
    
    // Trim leading/trailing underscores
    let trimmed = result.trim_matches('_');
    
    // If result is empty, use safe default
    if trimmed.is_empty() {
        "provider".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Migrates from the old single-file format to the new multi-file format
fn migrate_from_single_file(old_path: &PathBuf, providers_dir: &PathBuf) -> Result<()> {
    println!("{}", "Backing up old configuration file...".yellow());
    
    // Create backup
    let backup_path = old_path.with_extension("yaml.backup");
    std::fs::copy(old_path, &backup_path)?;

    // Load old format
    let content = std::fs::read_to_string(old_path)?;
    let old_config: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Create providers directory
    std::fs::create_dir_all(providers_dir)?;

    // Extract providers from old format
    if let Some(providers) = old_config.get("providers").and_then(|p| p.as_mapping()) {
        let now = chrono::Utc::now();

        for (provider_name, provider_data) in providers {
            if let Some(name) = provider_name.as_str() {
                let provider_name = name.to_string();
                let sanitized_name = sanitize_provider_name(&provider_name);
                let provider_file_name = format!("{}.yaml", sanitized_name);
                let provider_file_path = providers_dir.join(&provider_file_name);

                // Convert provider data to new format
                let mut provider_config = ProviderConfig {
                    keys: Vec::new(),
                    models: Vec::new(),
                    metadata: None,
                    version: "1.0".to_string(),
                    schema_version: "3.0".to_string(),
                    created_at: now,
                    updated_at: now,
                };

                // Extract data from old format and convert to new key format
                if let Some(api_key) = provider_data.get("api_key").and_then(|v| v.as_str()) {
                    // Create a default key from the old api_key field
                    let mut default_key = genai_keyfinder_core::models::ProviderKey::new(
                        "default".to_string(),
                        "migration".to_string(),
                        genai_keyfinder_core::models::Confidence::High,
                        genai_keyfinder_core::models::Environment::Production,
                    );
                    default_key.value = Some(api_key.to_string());
                    default_key.discovered_at = chrono::Utc::now();
                    default_key.validation_status = genai_keyfinder_core::models::ValidationStatus::Unknown;
                    provider_config.keys.push(default_key);
                }
                if let Some(models) = provider_data.get("models").and_then(|v| v.as_sequence()) {
                    provider_config.models = models.iter()
                        .filter_map(|m| m.as_str().map(String::from))
                        .collect();
                }
                if let Some(version) = provider_data.get("version").and_then(|v| v.as_str()) {
                    provider_config.version = version.to_string();
                }

                // Write individual provider file with secure permissions
                write_secure_file(&provider_file_path, &serde_yaml::to_string(&provider_config)?)?;
            }
        }

        // Remove old file
        std::fs::remove_file(old_path)?;

        println!("{}", "Migration completed successfully!".green());
    }

    Ok(())
}

/// Write a file with secure permissions (0o600 on Unix, restrictive ACL on Windows)
fn write_secure_file(path: &std::path::Path, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600) // Set restrictive permissions during file creation
            .open(path)?;
        
        file.write_all(content.as_bytes())?;
        
        // Additional safety check to ensure permissions are set correctly
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(path, permissions)?;
    }
    
    #[cfg(windows)]
    {
        // On Windows, create the file with restrictive permissions
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        
        file.write_all(content.as_bytes())?;
        
        // On Windows, we should ideally use Windows ACL APIs to set restrictive permissions
        // For now, we'll create the file and then try to set restrictive permissions
        // This is a basic implementation - in production you might want to use
        // the `winapi` crate or `windows-rs` for more granular ACL control
        
        // Try to remove inherited permissions and grant access only to current user
        use std::process::Command;
        
        // Use icacls to set restrictive permissions (current user only)
        // This removes inheritance and grants full control only to the current user
        let output = Command::new("icacls")
            .arg(path)
            .arg("/inheritance:r") // Remove inherited permissions
            .arg("/grant:r")
            .arg(&format!("{}:F", std::env::var("USERNAME").unwrap_or_else(|_| "CURRENT_USER".to_string()))) // Grant full control to current user
            .output();
        
        if let Err(e) = output {
            eprintln!("Warning: Could not set restrictive file permissions on Windows: {}", e);
            // Continue execution - file is still created, just with default permissions
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, fall back to standard file creation
        std::fs::write(path, content)?;
    }
    
    Ok(())
}
