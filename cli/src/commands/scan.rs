use anyhow::Result;
use colored::*;
use genai_keyfinder_core::{scan, ScanOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// YAML configuration structure for individual provider files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// List of API keys for the provider
    pub keys: Vec<genai_keyfinder_core::models::ProviderKey>,
    /// List of models available for this provider
    pub models: Vec<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
    /// Version information
    pub version: String,
    /// Schema version for migration tracking
    pub schema_version: String,
    /// When this configuration was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When this configuration was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}


pub fn handle_scan(
    home: Option<String>,
    format: String,
    include_values: bool,
    only: Option<String>,
    exclude: Option<String>,
    max_bytes_per_file: usize,
    dry_run: bool,
    audit_log: Option<String>,
    verbose: bool,
    update: bool,
) -> Result<()> {
    // Determine home directory
    let home_dir = match home {
        Some(h) => PathBuf::from(h),
        None => dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?,
    };

    // Parse provider filters
    let only_providers = only.map(|s| s.split(',').map(String::from).collect());
    let exclude_providers = exclude.map(|s| s.split(',').map(String::from).collect());

    // Create scan options
    let options = ScanOptions {
        home_dir: Some(home_dir.clone()),
        include_full_values: include_values,
        max_file_size: max_bytes_per_file,
        only_providers,
        exclude_providers,
    };

    if dry_run {
        println!("{}", "DRY RUN MODE - No files will be read".yellow().bold());
        // Show what would be scanned
        println!("Would scan directory: {}", home_dir.display());
        return Ok(());
    }

    // Perform scan
    println!("{}", "Scanning for GenAI credentials...".cyan().bold());
    let result = scan(options)?;

    // Output results based on format
    match format.as_str() {
        "json" => crate::output::json::output_json(&result, verbose)?,
        "ndjson" => crate::output::ndjson::output_ndjson(&result, verbose)?,
        "table" => crate::output::table::output_table(&result, verbose)?,
        "summary" => crate::output::summary::output_summary(&result, verbose)?,
        _ => anyhow::bail!("Unknown format: {}", format),
    }

    // Write audit log if requested
    if let Some(log_path) = audit_log {
        write_audit_log(&log_path, &result)?;
    }

    // Exit code: 0 if keys found, 1 if none found
    if result.keys.is_empty() && result.config_instances.is_empty() {
        std::process::exit(1);
    }

    // Update YAML configuration file if requested
    if update {
        update_yaml_config(&result, &home_dir)?;
    }

    Ok(())
}

/// Updates or creates the YAML configuration files with discovered providers and keys
fn update_yaml_config(result: &genai_keyfinder_core::ScanResult, home_dir: &std::path::Path) -> Result<()> {
    let config_dir = home_dir
        .join(".config")
        .join("aicred")
        .join("providers");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)?;

    let old_config_path = config_dir.parent().unwrap().join("providers.yaml");

    // Check if we need to migrate from old format
    if old_config_path.exists() && !config_dir.exists() {
        println!("{}", "Migrating from old single-file format to new multi-file format...".yellow());
        migrate_from_single_file(&old_config_path, &config_dir)?;
    }

    // Extract unique providers from scan results
    let mut discovered_providers: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    // Collect providers from discovered keys
    for key in &result.keys {
        discovered_providers.entry(key.provider.clone()).or_default();
    }
    
    // Collect providers from config instances
    for instance in &result.config_instances {
        for key in &instance.keys {
            discovered_providers.entry(key.provider.clone()).or_default();
        }
    }

    let now = chrono::Utc::now();

    // Update providers configuration
    for (provider_name, _models) in discovered_providers {
        let provider_file_name = format!("{}.yaml", provider_name.to_lowercase().replace(' ', "_"));
        let provider_file_path = config_dir.join(&provider_file_name);

        // Check if provider already exists
        let mut provider_config = if provider_file_path.exists() {
            let content = std::fs::read_to_string(&provider_file_path)?;
            // Try to deserialize, but if it fails (old format), create new config
            match serde_yaml::from_str::<ProviderConfig>(&content) {
                Ok(mut config) => {
                    // Update timestamp for existing config
                    config.updated_at = now;
                    config
                },
                Err(_) => {
                    // Old format detected, create new config
                    println!("{}", format!("Migrating provider {} to new format", provider_name).yellow());
                    ProviderConfig {
                        keys: Vec::new(),
                        models: Vec::new(),
                        metadata: None,
                        version: "1.0".to_string(),
                        schema_version: "3.0".to_string(),
                        created_at: now,
                        updated_at: now,
                    }
                }
            }
        } else {
            println!("{}", format!("Adding new provider: {}", provider_name).green());
            ProviderConfig {
                keys: Vec::new(),
                models: Vec::new(),
                metadata: None,
                version: "1.0".to_string(),
                schema_version: "3.0".to_string(),
                created_at: now,
                updated_at: now,
            }
        };

        // Write provider config to individual file
        let yaml_content = serde_yaml::to_string(&provider_config)?;
        std::fs::write(&provider_file_path, yaml_content)?;
    }

    println!(
        "{}",
        format!(
            "Updated configuration files in: {}",
            config_dir.display()
        )
        .green()
        .bold()
    );

    Ok(())
}

/// Migrates from the old single-file format to the new multi-file format
fn migrate_from_single_file(old_path: &PathBuf, config_dir: &PathBuf) -> Result<()> {
    println!("{}", "Backing up old configuration file...".yellow());
    
    // Create backup
    let backup_path = old_path.with_extension("yaml.backup");
    std::fs::copy(old_path, &backup_path)?;

    // Load old format
    let content = std::fs::read_to_string(old_path)?;
    let old_config: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Extract providers from old format
    if let Some(providers) = old_config.get("providers").and_then(|p| p.as_mapping()) {
        let now = chrono::Utc::now();

        for (provider_name, provider_data) in providers {
            if let Some(name) = provider_name.as_str() {
                let provider_name = name.to_string();
                let provider_file_name = format!("{}.yaml", provider_name.to_lowercase().replace(' ', "_"));
                let provider_file_path = config_dir.join(&provider_file_name);

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

                // Write individual provider file
                let yaml_content = serde_yaml::to_string(&provider_config)?;
                std::fs::write(&provider_file_path, yaml_content)?;
            }
        }

        // Remove old file
        std::fs::remove_file(old_path)?;

        println!("{}", "Migration completed successfully!".green());
    }

    Ok(())
}

fn write_audit_log(log_path: &str, result: &genai_keyfinder_core::ScanResult) -> Result<()> {
    #[cfg(unix)]
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(log_path)?;
    #[cfg(not(unix))]
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(log_path)?;
    writeln!(file, "GenAI Key Finder Audit Log")?;
    writeln!(file, "=========================")?;
    writeln!(file, "Scan Date: {}", result.scan_completed_at)?;
    writeln!(file, "Home Directory: {}", result.home_directory)?;
    writeln!(
        file,
        "Providers Scanned: {}",
        result.providers_scanned.join(", ")
    )?;
    writeln!(file, "Total Keys Found: {}", result.keys.len())?;
    writeln!(
        file,
        "Total Config Instances: {}",
        result.config_instances.len()
    )?;
    writeln!(file)?;

    if !result.keys.is_empty() {
        writeln!(file, "Discovered Keys:")?;
        for key in &result.keys {
            writeln!(
                file,
                "  - {}: {} ({} - confidence: {})",
                key.provider, key.value_type, key.source, key.confidence
            )?;
        }
    }

    if !result.config_instances.is_empty() {
        writeln!(file, "\nConfig Instances:")?;
        for instance in &result.config_instances {
            writeln!(
                file,
                "  - {}: {}",
                instance.app_name,
                instance.config_path.display()
            )?;
        }
    }

    Ok(())
}
