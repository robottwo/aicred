//! Wrap command implementation - executes commands with LLM environment variables

use aicred_core::models::UnifiedLabel;
use aicred_core::scanners::ScannerRegistry;
use aicred_core::ProviderModelTuple;
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

    // 4. Load labels and provider instances
    let labels = load_labels()?;

    // 5. Create provider instances from scanned configurations
    let provider_instances = create_provider_instances()?;

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

/// Simple function to load labels (placeholder implementation)
fn load_labels() -> Result<Vec<UnifiedLabel>> {
    // For now, return hardcoded test labels
    // In a real implementation, this would load from configuration
    let labels = vec![
        UnifiedLabel::new(
            "fast".to_string(),
            ProviderModelTuple::parse("groq:llama3-70b-8192")
                .map_err(|e| anyhow::anyhow!("Failed to parse tuple: {}", e))?,
        ),
        UnifiedLabel::new(
            "smart".to_string(),
            ProviderModelTuple::parse("openrouter:anthropic/claude-3-opus")
                .map_err(|e| anyhow::anyhow!("Failed to parse tuple: {}", e))?,
        ),
    ];
    Ok(labels)
}

/// Create provider instances from scanned configurations
fn create_provider_instances() -> Result<Vec<aicred_core::models::ProviderInstance>> {
    // For now, return hardcoded test instances
    // In a real implementation, this would scan for actual configurations
    let mut instances = Vec::new();

    // Create a Groq instance for the "fast" label
    let mut groq_instance = aicred_core::models::ProviderInstance::new(
        "groq-test-instance".to_string(),
        "Groq Test Instance".to_string(),
        "groq".to_string(),
        "https://api.groq.com/openai/v1".to_string(),
    );
    groq_instance.set_api_key("gsk_test1234567890abcdef1234567890abcdef".to_string());
    let groq_model = aicred_core::models::Model::new(
        "llama3-70b-8192".to_string(),
        "llama3-70b-8192".to_string(),
    );
    groq_instance.add_model(groq_model);
    instances.push(groq_instance);

    // Create an OpenRouter instance for the "smart" label
    let mut openrouter_instance = aicred_core::models::ProviderInstance::new(
        "openrouter-test-instance".to_string(),
        "OpenRouter Test Instance".to_string(),
        "openrouter".to_string(),
        "https://openrouter.ai/api/v1".to_string(),
    );
    openrouter_instance.set_api_key("sk-or-v1_test1234567890abcdef1234567890abcdef".to_string());
    let openrouter_model = aicred_core::models::Model::new(
        "anthropic/claude-3-opus".to_string(),
        "anthropic/claude-3-opus".to_string(),
    );
    openrouter_instance.add_model(openrouter_model);
    instances.push(openrouter_instance);

    Ok(instances)
}
