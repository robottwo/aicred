//! SetEnv command implementation - generates shell export statements for LLM environment variables

use crate::commands::labels::load_label_assignments_with_home;
use aicred_core::models::UnifiedLabel;
use aicred_core::scanners::ScannerRegistry;
use aicred_core::ProviderModelTuple;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
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

    // 5. Build environment variable map
    let env_vars = build_env_var_map(&env_schema, &label_mappings, &labels)?;
    tracing::info!(
        "Built environment variable map with {} variables",
        env_vars.len()
    );
    tracing::debug!("Environment variables: {:?}", env_vars);

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
                format!("{}...{}", &value[..4], &value[value.len() - 4..])
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

/// Build environment variable map from schema, mappings, and labels
fn build_env_var_map(
    schema: &[aicred_core::scanners::EnvVarDeclaration],
    mappings: &[aicred_core::scanners::LabelMapping],
    labels: &[UnifiedLabel],
) -> Result<HashMap<String, String>> {
    tracing::debug!(
        "Building env var map with {} schema entries, {} mappings, {} labels",
        schema.len(),
        mappings.len(),
        labels.len()
    );
    let mut env_vars = HashMap::new();

    // For each label mapping, find the corresponding label assignment
    for mapping in mappings {
        tracing::debug!("Processing mapping: {:?}", mapping);
        if let Some(label) = labels.iter().find(|l| l.label_name == mapping.label_name) {
            tracing::debug!("Found matching label: {:?}", label);
            // Use the target directly (it's already a ProviderModelTuple)
            let tuple = &label.target;

            // Find matching env vars for this group
            let prefix = &mapping.env_var_group;
            for var in schema.iter().filter(|v| v.name.starts_with(prefix)) {
                tracing::debug!("Processing env var: {:?}", var);
                // Resolve value based on value_type
                if let Some(value) = resolve_env_var_value(tuple, &var.value_type, &var.name)? {
                    tracing::debug!("Resolved value for {}: {}", var.name, value);
                    // DIAGNOSTIC: Show what we resolved vs what the schema default would be
                    if var.name.contains("MODEL_ID") {
                        tracing::warn!(
                            "⚠️  RESOLVED MODEL ID: {} = {} (from user's label: {}:{})",
                            var.name,
                            value,
                            tuple.provider,
                            tuple.model
                        );
                        if let Some(default_value) = &var.default_value {
                            tracing::warn!("⚠️  Schema default would have been: {}", default_value);
                        }
                    }
                    env_vars.insert(var.name.clone(), value);
                } else {
                    tracing::debug!("No value resolved for {}", var.name);
                    // Use default value if provided and no resolution
                    if let Some(default_value) = &var.default_value {
                        tracing::debug!("Using default value for {}: {}", var.name, default_value);
                        if var.name.contains("MODEL_ID") {
                            tracing::warn!(
                                "⚠️  USING DEFAULT MODEL ID: {} = {} (no label match)",
                                var.name,
                                default_value
                            );
                        }
                        env_vars.insert(var.name.clone(), default_value.clone());
                    }
                }
            }
        } else {
            tracing::debug!(
                "No matching label found for mapping: {}",
                mapping.label_name
            );
        }
    }

    // Handle direct provider mappings (for scanners without label mappings)
    if mappings.is_empty() && !schema.is_empty() {
        tracing::debug!("No label mappings found, using direct provider mappings");
        // For scanners like RooCode, ClaudeDesktop, etc., use direct provider mappings
        for var in schema {
            tracing::debug!("Processing direct env var: {:?}", var);
            if let Some(value) = resolve_direct_env_var(var, labels)? {
                tracing::debug!("Resolved direct value for {}: {}", var.name, value);
                env_vars.insert(var.name.clone(), value);
            } else {
                tracing::debug!("No direct value resolved for {}", var.name);
            }
        }
    }

    tracing::debug!("Final env_vars map contains {} entries", env_vars.len());
    Ok(env_vars)
}

/// Resolve environment variable value based on value type and provider:model tuple
fn resolve_env_var_value(
    tuple: &ProviderModelTuple,
    value_type: &str,
    var_name: &str,
) -> Result<Option<String>> {
    tracing::debug!(
        "Resolving env var value for tuple: {:?}, value_type: {}",
        tuple,
        value_type
    );

    // Handle both explicit semantic types and generic "string" type
    match value_type {
        "ApiKey" => {
            // Look up API key for this provider
            // This is a simplified version - in reality we'd need to load discovered keys
            let api_key = format!("api_key_for_{}", tuple.provider);
            tracing::debug!("Resolved API key for {}: {}", tuple.provider, api_key);
            Ok(Some(api_key))
        }
        "BaseUrl" => {
            // Default base URLs for known providers
            let base_url = match tuple.provider.as_str() {
                "openai" => "https://api.openai.com/v1",
                "anthropic" => "https://api.anthropic.com",
                "groq" => "https://api.groq.com/openai/v1",
                "openrouter" => "https://openrouter.ai/api/v1",
                _ => {
                    tracing::debug!("No base URL found for provider: {}", tuple.provider);
                    return Ok(None);
                }
            };
            tracing::debug!(
                "Resolved base URL: {} for provider: {}",
                base_url,
                tuple.provider
            );
            Ok(Some(base_url.to_string()))
        }
        "ModelId" => {
            // Use the model from the tuple
            let model_id = tuple.model.clone();
            tracing::debug!(
                "Resolved model ID: {} for provider: {}",
                model_id,
                tuple.provider
            );
            Ok(Some(model_id))
        }
        "string" => {
            // For generic "string" type, we need to infer the semantic meaning from the variable name
            // This is used by scanners like GSH that don't specify explicit semantic types
            tracing::debug!(
                "Handling generic 'string' value_type - inferring from variable name: {}",
                var_name
            );

            // Infer the semantic type from the variable name
            if var_name.ends_with("_API_KEY") {
                // This is an API key
                let api_key = format!("api_key_for_{}", tuple.provider);
                tracing::debug!("Inferred API key for {}: {}", tuple.provider, api_key);
                Ok(Some(api_key))
            } else if var_name.ends_with("_BASE_URL") {
                // This is a base URL
                let base_url = match tuple.provider.as_str() {
                    "openai" => "https://api.openai.com/v1",
                    "anthropic" => "https://api.anthropic.com",
                    "groq" => "https://api.groq.com/openai/v1",
                    "openrouter" => "https://openrouter.ai/api/v1",
                    _ => {
                        tracing::debug!("No base URL found for provider: {}", tuple.provider);
                        return Ok(None);
                    }
                };
                tracing::debug!(
                    "Inferred base URL: {} for provider: {}",
                    base_url,
                    tuple.provider
                );
                Ok(Some(base_url.to_string()))
            } else if var_name.ends_with("_ID") {
                // This is a model ID
                let model_id = tuple.model.clone();
                tracing::debug!(
                    "Inferred model ID: {} for provider: {}",
                    model_id,
                    tuple.provider
                );
                Ok(Some(model_id))
            } else {
                // For other variable types, use the model from the tuple as default
                tracing::debug!("Using model from tuple as default value: {}", tuple.model);
                Ok(Some(tuple.model.clone()))
            }
        }
        _ => {
            tracing::warn!("Unknown value_type '{}' - cannot resolve", value_type);
            tracing::debug!(
                "Available value_types we can handle: ApiKey, BaseUrl, ModelId, string"
            );
            Ok(None)
        }
    }
}

/// Resolve direct environment variables for scanners without label mappings
fn resolve_direct_env_var(
    var: &aicred_core::scanners::EnvVarDeclaration,
    labels: &[UnifiedLabel],
) -> Result<Option<String>> {
    tracing::debug!(
        "Resolving direct env var: {:?}, available labels: {:?}",
        var,
        labels.iter().map(|l| &l.label_name).collect::<Vec<_>>()
    );
    // For scanners like RooCode, we need to map their specific env vars
    // This is a simplified implementation
    if var.name == "ROO_CODE_API_KEY" {
        // Look for a label that maps to roo_code or similar
        if let Some(_label) = labels.iter().find(|l| l.label_name == "roo_code") {
            tracing::debug!("Found roo_code label, returning placeholder API key");
            return Ok(Some("roo_code_api_key".to_string()));
        }
    } else if var.name == "ANTHROPIC_API_KEY" {
        // Look for anthropic label
        if let Some(_label) = labels.iter().find(|l| l.label_name == "anthropic") {
            tracing::debug!("Found anthropic label, returning placeholder API key");
            return Ok(Some("anthropic_api_key".to_string()));
        }
    }
    // Add more mappings as needed

    tracing::debug!("No direct mapping found for {}", var.name);
    Ok(None)
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
    use aicred_core::scanners::{EnvVarDeclaration, LabelMapping};

    #[test]
    fn test_build_env_var_map() -> Result<()> {
        let schema = vec![
            EnvVarDeclaration::required(
                "GSH_FAST_MODEL_API_KEY".to_string(),
                "API key for fast model".to_string(),
                "ApiKey".to_string(),
            ),
            EnvVarDeclaration::required(
                "GSH_FAST_MODEL_ID".to_string(),
                "Model ID for fast model".to_string(),
                "ModelId".to_string(),
            ),
        ];

        let mappings = vec![LabelMapping::new(
            "fast".to_string(),
            "GSH_FAST_MODEL".to_string(),
            "Fast model configuration".to_string(),
        )];

        let labels = vec![UnifiedLabel::new(
            "fast".to_string(),
            ProviderModelTuple::parse("groq:llama3-70b-8192")
                .map_err(|e| anyhow::anyhow!("Failed to parse tuple: {}", e))?,
        )];

        let result = build_env_var_map(&schema, &mappings, &labels)?;

        assert!(result.contains_key("GSH_FAST_MODEL_API_KEY"));
        assert!(result.contains_key("GSH_FAST_MODEL_ID"));
        assert_eq!(result.get("GSH_FAST_MODEL_ID").unwrap(), "llama3-70b-8192");

        Ok(())
    }

    #[test]
    fn test_escape_shell_value() {
        assert_eq!(escape_shell_value("simple", "bash"), "simple");
        assert_eq!(escape_shell_value("it's", "bash"), "it'\\''s");
        assert_eq!(escape_shell_value("it's", "fish"), "it\\'s");
        assert_eq!(escape_shell_value("it's", "powershell"), "it''s");
    }
}
