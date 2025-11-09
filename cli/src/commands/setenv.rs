//! SetEnv command implementation - generates shell export statements for LLM environment variables

use crate::commands::labels::load_label_assignments_with_home;
use crate::utils::provider_loader::load_provider_instances;
use aicred_core::scanners::ScannerRegistry;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Handle the setenv command - generate shell export statements
pub fn handle_setenv(
    scanner_names: Option<Vec<String>>,
    format: Option<String>,
    dry_run: bool,
    home_dir: Option<PathBuf>,
) -> Result<()> {
    tracing::info!(
        "Starting setenv command with scanner_names: {:?}, format: {:?}, dry_run: {}",
        scanner_names,
        format,
        dry_run
    );

    // Set home directory if provided
    if let Some(home) = &home_dir {
        std::env::set_var("HOME", home);
        tracing::debug!("Set HOME environment variable to: {}", home.display());
    }

    // 1. Load configuration and scanner registry
    let scanner_registry = ScannerRegistry::new();
    aicred_core::register_builtin_scanners(&scanner_registry)?;
    tracing::debug!("Registered builtin scanners: {:?}", scanner_registry.list());

    // 2. Determine which scanner to use
    let scanner = if let Some(names) = scanner_names {
        tracing::info!("Using specified scanner names: {:?}", names);
        // Use the first scanner name provided
        let name = names
            .first()
            .ok_or_else(|| anyhow!("Scanner names list is empty"))?;
        scanner_registry.get(name).ok_or_else(|| {
            anyhow!(
                "Scanner '{}' not found. Available scanners: {}",
                name,
                scanner_registry.list().join(", ")
            )
        })?
    } else {
        // Default to first available scanner or error
        let scanners = scanner_registry.list();
        tracing::debug!("Available scanners: {:?}", scanners);
        if scanners.is_empty() {
            return Err(anyhow!("No scanners available"));
        }
        // For now, default to gsh if available, otherwise first scanner
        let selected_scanner = if let Some(scanner) = scanner_registry.get("gsh") {
            tracing::info!("Defaulting to gsh scanner");
            scanner
        } else {
            let first_scanner = &scanners[0];
            tracing::info!("Defaulting to first available scanner: {}", first_scanner);
            scanner_registry
                .get(first_scanner)
                .ok_or_else(|| anyhow!("Failed to get default scanner"))?
        };
        selected_scanner
    };

    tracing::info!("Selected scanner: {}", scanner.name());

    // 3. Get environment variable schema from scanner
    let env_schema = scanner.get_env_var_schema();
    let label_mappings = scanner.get_label_mappings();

    tracing::debug!("Environment variable schema: {:?}", env_schema);
    tracing::debug!("Label mappings: {:?}", label_mappings);

    // DIAGNOSTIC: Check if schema has default values
    tracing::warn!("⚠️  DIAGNOSTIC: Checking scanner schema for default values:");
    for var in &env_schema {
        if let Some(default) = &var.default_value {
            tracing::warn!("⚠️    {} has default value: {}", var.name, default);
        }
    }

    // 4. Load labels from actual configuration
    let labels = load_label_assignments_with_home(home_dir.as_deref())?;
    tracing::info!("Loaded {} labels from configuration", labels.len());
    tracing::debug!("Labels: {:?}", labels);

    // DIAGNOSTIC: Show what labels were loaded
    if labels.is_empty() {
        tracing::warn!(
            "⚠️  No labels found in configuration. Run 'aicred labels assign' to create labels."
        );
    } else {
        tracing::info!("Labels loaded from configuration:");
        for label in &labels {
            tracing::info!(
                "  {} -> {}:{}",
                label.label_name,
                label.target.provider,
                label.target.model
            );
        }
    }

    // 5. Load provider instances from configuration
    let provider_instances_collection = load_provider_instances(home_dir.as_deref())?;
    let provider_instances = provider_instances_collection
        .all_instances()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    tracing::info!("Loaded {} provider instances", provider_instances.len());

    // 6. Use EnvResolver to properly resolve environment variables
    let env_resolver = aicred_core::EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let resolution_result = env_resolver.resolve(dry_run)?;
    let env_vars = resolution_result.variables;
    tracing::info!(
        "Built environment variable map with {} variables",
        env_vars.len()
    );
    // Log variable names only (never log values that may contain secrets)
    tracing::debug!(
        "Environment variable names: {:?}",
        env_vars.keys().collect::<Vec<_>>()
    );

    // DIAGNOSTIC: Show final env vars with model IDs
    tracing::warn!("⚠️  DIAGNOSTIC: Final environment variables generated:");
    for (key, value) in &env_vars {
        if key.contains("MODEL_ID") {
            tracing::warn!("⚠️    {} = {} (THIS IS THE OUTPUT VALUE)", key, value);
        }
    }

    // 6. Handle dry run
    if dry_run {
        println!("Environment variables that would be exported:");
        for (key, value) in &env_vars {
            // Mask sensitive values (API keys)
            let display_value = if key.contains("API_KEY") && !value.is_empty() {
                // Only show first 4 and last 4 chars if value is long enough
                if value.len() >= 8 {
                    format!("{}...{}", &value[..4], &value[value.len() - 4..])
                } else {
                    // For short values, mask completely for security
                    "****".to_string()
                }
            } else {
                value.clone()
            };
            println!("  {}={}", key, display_value);
        }
        return Ok(());
    }

    // 7. Generate export statements based on format
    let format_str = format.as_deref().unwrap_or("bash");
    match format_str {
        "bash" | "zsh" => {
            for (key, value) in env_vars {
                println!("export {}='{}'", key, escape_shell_value(&value, "bash"));
            }
        }
        "fish" => {
            for (key, value) in env_vars {
                println!("set -gx {} '{}'", key, escape_shell_value(&value, "fish"));
            }
        }
        "powershell" => {
            for (key, value) in env_vars {
                println!(
                    "$env:{} = '{}'",
                    key,
                    escape_shell_value(&value, "powershell")
                );
            }
        }
        _ => {
            return Err(anyhow!(
                "Unsupported format: {}. Supported formats: bash, fish, powershell",
                format_str
            ))
        }
    }

    Ok(())
}

/// Escape shell value based on the shell format
fn escape_shell_value(value: &str, shell_type: &str) -> String {
    match shell_type {
        "bash" | "zsh" => {
            // Escape single quotes in bash/zsh
            value.replace("'", "'\\''")
        }
        "fish" => {
            // Escape single quotes in fish
            value.replace("'", "\\'")
        }
        "powershell" => {
            // Escape single quotes in PowerShell
            value.replace("'", "''")
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_shell_value() {
        assert_eq!(escape_shell_value("simple", "bash"), "simple");
        assert_eq!(escape_shell_value("it's", "bash"), "it'\\''s");
        assert_eq!(escape_shell_value("it's", "fish"), "it\\'s");
        assert_eq!(escape_shell_value("it's", "powershell"), "it''s");
    }

    #[test]
    fn test_api_key_masking_various_lengths() {
        // Test masking logic that would be used in dry run mode
        let test_cases = vec![
            ("short", "****"),                           // len=5, < 8, fully masked
            ("1234567", "****"),                         // len=7, < 8, fully masked
            ("12345678", "1234...5678"),                 // len=8, = 8, show first 4 and last 4
            ("sk-proj-abcdefghijklmnop", "sk-p...mnop"), // len=23, > 8, show first 4 and last 4
        ];

        for (input, expected) in test_cases {
            let result = if !input.is_empty() {
                if input.len() >= 8 {
                    format!("{}...{}", &input[..4], &input[input.len() - 4..])
                } else {
                    "****".to_string()
                }
            } else {
                input.to_string()
            };
            assert_eq!(
                result,
                expected,
                "Failed for input '{}' (len={})",
                input,
                input.len()
            );
        }
    }
}
