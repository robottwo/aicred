//! Wrap command implementation - executes commands with LLM environment variables

use crate::commands::labels::load_label_assignments_with_home;
use crate::utils::provider_loader::load_provider_instances;
use aicred_core::scanners::ScannerRegistry;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

/// Handle the wrap command - execute a command with LLM environment variables
pub fn handle_wrap(
    scanner_names: Option<Vec<String>>,
    dry_run: bool,
    command_args: Vec<String>,
    home_dir: Option<PathBuf>,
) -> Result<()> {
    if command_args.is_empty() && !dry_run {
        return Err(anyhow!("No command specified to wrap"));
    }

    // Set home directory if provided
    if let Some(home) = &home_dir {
        std::env::set_var("HOME", home);
    }

    // 1. Load configuration and scanner registry
    let scanner_registry = ScannerRegistry::new();
    aicred_core::register_builtin_scanners(&scanner_registry)?;

    // 2. Determine which scanner to use
    let scanner = if let Some(names) = scanner_names {
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
        if scanners.is_empty() {
            return Err(anyhow!("No scanners available"));
        }
        // For now, default to gsh if available, otherwise first scanner
        if let Some(scanner) = scanner_registry.get("gsh") {
            scanner
        } else {
            scanner_registry
                .get(&scanners[0])
                .ok_or_else(|| anyhow!("Failed to get default scanner"))?
        }
    };

    // 3. Get environment variable schema and label mappings from scanner
    let env_schema = scanner.get_env_var_schema();
    let label_mappings = scanner.get_label_mappings();

    // 4. Load labels and provider instances from user configuration
    let labels = load_label_assignments_with_home(home_dir.as_deref())?;

    // 5. Load provider instances from disk/config
    let provider_instances_collection = load_provider_instances(home_dir.as_deref())?;
    let provider_instances: Vec<_> = provider_instances_collection
        .all_instances()
        .iter()
        .map(|inst| (*inst).clone())
        .collect();

    // 6. Use EnvResolver to properly resolve environment variables
    let env_resolver = aicred_core::EnvResolverBuilder::new()
        .with_provider_instances(provider_instances)
        .with_labels(labels)
        .with_env_schema(env_schema)
        .with_label_mappings(label_mappings)
        .build();

    let resolution_result = env_resolver.resolve(dry_run)?;

    // 7. Handle dry run
    if dry_run {
        println!("Environment variables that would be set:");
        for (key, value) in &resolution_result.variables {
            // Mask sensitive values (API keys)
            let display_value = if key.contains("API_KEY") && !value.is_empty() {
                if value.len() > 8 {
                    format!("{}***{}", &value[..4], &value[value.len() - 4..])
                } else {
                    "****".to_string()
                }
            } else {
                value.clone()
            };
            println!("  {}={}", key, display_value);
        }

        if !resolution_result.missing_required.is_empty() {
            println!("\nMissing required variables:");
            for var in &resolution_result.missing_required {
                println!("  {}", var);
            }
        }
        return Ok(());
    }

    // 8. Check for missing required variables in normal mode
    if !resolution_result.is_successful() {
        return Err(anyhow!(
            "Environment variable resolution failed. Missing required variables: {}",
            resolution_result.missing_required.join(", ")
        ));
    }

    // 9. Execute command with resolved environment variables
    let (cmd, args) = command_args.split_first().unwrap();

    let status = Command::new(cmd)
        .args(args)
        .envs(resolution_result.variables)
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}
