//! Export command implementation - export discovered configurations as shell variables

use crate::utils::provider_loader::load_provider_instances;
use aicred_core::export::{
    default_template, export_vars, ExportConfig, ExportContext, ExportFormat, ExportTemplate,
};
use aicred_core::models::{ConfigInstance, Provider};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Handle the export command
pub fn handle_export(
    home_dir: Option<PathBuf>,
    template: Option<String>,
    format: Option<String>,
    output: Option<String>,
    include_secrets: bool,
    prefix: Option<String>,
    vars: Option<Vec<String>>,
    dry_run: bool,
) -> Result<()> {
    // Set home directory if provided
    if let Some(home) = &home_dir {
        std::env::set_var("HOME", home);
    }

    // Parse format
    let format = format
        .map(|f| ExportFormat::from_str(&f))
        .unwrap_or(Ok(ExportFormat::Bash))?;

    // Load provider instances
    let provider_instances_collection = load_provider_instances(home_dir.as_deref())?;
    let instances: Vec<_> = provider_instances_collection
        .all_instances()
        .iter()
        .map(|inst| (*inst).clone())
        .collect();

    // Build export context from discovered instances
    let mut ctx = ExportContext::new();
    for instance in &instances {
        let provider_name = instance.provider.to_string().to_lowercase();
        if let Some(api_key) = &instance.api_key {
            ctx.add_provider_var(&provider_name, "api_key", api_key);
        }
        if let Some(base_url) = &instance.base_url {
            ctx.add_provider_var(&provider_name, "base_url", base_url);
        }
        ctx.add_provider_var(&provider_name, "id", &instance.id);
    }

    // Add custom variables from --vars flag
    if let Some(var_list) = vars {
        for var_spec in var_list {
            let parts: Vec<&str> = var_spec.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(anyhow!(
                    "Invalid variable specification: '{}'. Expected format: NAME=VALUE",
                    var_spec
                ));
            }
            ctx.add_custom_var(parts[0].trim(), parts[1].trim());
        }
    }

    // Generate export statements
    let export_content = if let Some(template_path) = template {
        // Load custom template
        let template = ExportTemplate::from_file(template_path)?;
        template.render(&ctx, format)?
    } else {
        // Use default template
        let template = default_template();
        template.render(&ctx, format)?
    };

    // Apply prefix if specified
    let export_content = if let Some(pfx) = &prefix {
        apply_prefix(&export_content, pfx, format)?
    } else {
        export_content
    };

    // Handle dry run mode
    if dry_run {
        println!("{}", export_content);
        return Ok(());
    }

    // Handle output file or stdout
    if let Some(output_path) = output {
        std::fs::write(&output_path, export_content)?;
        eprintln!(
            "Exported {} variable(s) to {}",
            count_exported_vars(&export_content),
            output_path.display()
        );
    } else {
        println!("{}", export_content);
    }

    Ok(())
}

/// Count the number of exported variables in the content
fn count_exported_vars(content: &str) -> usize {
    content.lines().filter(|line| {
        line.starts_with("export ")
            || line.starts_with("set -gx ")
            || line.starts_with("$env:")
    }).count()
}

/// Apply a prefix to all exported variable names
fn apply_prefix(content: &str, prefix: &str, format: ExportFormat) -> Result<String> {
    let mut result = String::new();
    let prefix_upper = prefix.to_uppercase();

    for line in content.lines() {
        let prefixed_line = match format {
            ExportFormat::Bash => {
                if line.starts_with("export ") {
                    // Extract variable name and value
                    let rest = &line[7..]; // Skip "export "
                    if let Some(eq_pos) = rest.find('=') {
                        let var_name = &rest[..eq_pos];
                        let value = &rest[eq_pos..];
                        format!("export {}{}{}", prefix_upper, var_name, value)
                    } else {
                        line.to_string()
                    }
                } else if line.starts_with("#") {
                    // Comment line - might contain variable name references
                    line.to_string()
                } else {
                    line.to_string()
                }
            }
            ExportFormat::Fish => {
                if line.starts_with("set -gx ") {
                    // Extract variable name and value
                    let rest = &line[8..]; // Skip "set -gx "
                    if let Some(space_pos) = rest.find(' ') {
                        let var_name = &rest[..space_pos];
                        let value = &rest[space_pos..];
                        format!("set -gx {}{}{}", prefix_upper, var_name, value)
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            }
            ExportFormat::PowerShell => {
                if line.starts_with("$env:") {
                    // Extract variable name and value
                    let rest = &line[5..]; // Skip "$env:"
                    if let Some(space_pos) = rest.find(' ') {
                        let var_name = &rest[..space_pos];
                        let value = &rest[space_pos..];
                        format!("$env:{}{}{}", prefix_upper, var_name, value)
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            }
        };
        result.push_str(&prefixed_line);
        result.push('\n');
    }

    Ok(result)
}

/// Handle export of just variables (no template) for simple use cases
pub fn handle_export_simple(
    vars: HashMap<String, String>,
    format: Option<String>,
    include_secrets: bool,
) -> Result<()> {
    let format = format
        .map(|f| ExportFormat::from_str(&f))
        .unwrap_or(Ok(ExportFormat::Bash))?;

    let export_content = export_vars(&vars, format, include_secrets)?;
    println!("{}", export_content);

    Ok(())
}
