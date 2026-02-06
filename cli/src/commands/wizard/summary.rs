//! Summary phase - confirm and write configuration

use anyhow::{Context, Result};
use aicred_core::{Label, LabelAssignment, ProviderInstance};
use console::style;
use inquire::Confirm;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::WizardOptions;
use super::ui;

/// Run the summary phase
pub fn run_summary_phase(
    instances: Vec<ProviderInstance>,
    labels: HashMap<String, (String, String)>,
    options: &WizardOptions,
) -> Result<(Vec<ProviderInstance>, HashMap<String, (String, String)>, PathBuf)> {
    ui::section_header("Configuration Summary");
    
    // Determine config directory
    let config_dir = if let Some(home) = &options.home {
        home.join(".config").join("aicred")
    } else {
        dirs_next::config_dir()
            .context("Could not determine config directory")?
            .join("aicred")
    };
    
    let instances_file = config_dir.join("instances.yaml");
    let labels_file = config_dir.join("labels.yaml");
    
    println!("The following will be saved to:");
    println!("  {}", style(instances_file.display()).cyan());
    if !labels.is_empty() {
        println!("  {}", style(labels_file.display()).cyan());
    }
    println!();
    
    // Show provider instances summary
    if !instances.is_empty() {
        println!("{}", style("Provider Instances:").bold());
        for instance in &instances {
            let status = if instance.active {
                style("Active").green()
            } else {
                style("Inactive").dim()
            };
            
            let display_name = instance.metadata.get("display_name")
                .map(|s| s.as_str())
                .unwrap_or(&instance.id);
            
            println!(
                "  {} {} ({})",
                style("✓").green(),
                style(&instance.id).cyan().bold(),
                display_name
            );
            println!("    {} {} models enabled", style("→").dim(), instance.models.len());
            println!("    {} Status: {}", style("→").dim(), status);
        }
        println!();
    }
    
    // Show labels summary
    if !labels.is_empty() {
        println!("{}", style("Labels:").bold());
        for (label, (instance, model)) in &labels {
            println!(
                "  {} {} → {}:{}",
                style("✓").green(),
                style(label).cyan().bold(),
                instance,
                model
            );
        }
        println!();
    }
    
    // Confirm
    if !options.auto_accept {
        let should_save = Confirm::new("Save and finish?")
            .with_default(true)
            .prompt()
            .unwrap_or(false);
        
        if !should_save {
            println!("{}", style("Setup cancelled.").yellow());
            std::process::exit(0);
        }
    }
    
    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    
    // Write instances file
    write_instances_file(&instances_file, &instances)?;
    
    // Set restrictive permissions on instances file (contains API keys)
    set_secure_permissions(&instances_file)?;
    
    // Write labels file if we have labels
    if !labels.is_empty() {
        write_labels_file(&labels_file, &labels)?;
        set_secure_permissions(&labels_file)?;
    }
    
    Ok((instances, labels, config_dir))
}

/// Set secure file permissions (owner read/write only)
fn set_secure_permissions(path: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .context("Failed to read file metadata")?
            .permissions();
        perms.set_mode(0o600); // Owner read/write only
        fs::set_permissions(path, perms)
            .context("Failed to set file permissions")?;
    }
    
    // On Windows, the default ACLs are generally restrictive enough
    // but we could enhance this with Windows-specific ACL manipulation if needed
    
    Ok(())
}

/// Write instances.yaml file
fn write_instances_file(path: &PathBuf, instances: &[ProviderInstance]) -> Result<()> {
    use aicred_core::ProviderCollection;
    
    let mut collection = ProviderCollection::new();
    for instance in instances {
        collection.add_or_replace_instance(instance.clone());
    }
    
    let yaml = serde_yaml::to_string(&collection)
        .context("Failed to serialize instances to YAML")?;
    
    fs::write(path, yaml)
        .context("Failed to write instances.yaml")?;
    
    Ok(())
}

/// Structure for serializing labels file
#[derive(serde::Serialize)]
struct LabelsFile {
    labels: HashMap<String, Label>,
    assignments: Vec<LabelAssignment>,
}

/// Write labels.yaml file
fn write_labels_file(
    path: &PathBuf,
    labels: &HashMap<String, (String, String)>,
) -> Result<()> {
    use aicred_core::{Label, LabelAssignment, LabelTarget};
    use chrono::Utc;
    use std::collections::HashMap as StdHashMap;
    
    // Build labels structure
    let mut labels_map: StdHashMap<String, Label> = StdHashMap::new();
    let mut assignments: Vec<LabelAssignment> = Vec::new();
    
    for (label_name, (instance_id, model_id)) in labels {
        // Create label if it doesn't exist
        if !labels_map.contains_key(label_name) {
            labels_map.insert(
                label_name.clone(),
                Label {
                    name: label_name.clone(),
                    description: None,
                    created_at: Utc::now(),
                    metadata: StdHashMap::new(),
                },
            );
        }
        
        // Create assignment with LabelTarget
        assignments.push(LabelAssignment {
            label_name: label_name.clone(),
            target: LabelTarget::ProviderModel {
                instance_id: instance_id.clone(),
                model_id: model_id.clone(),
            },
            assigned_at: Utc::now(),
            assigned_by: None,
        });
    }
    
    // Serialize to YAML using serde_yaml for consistency
    let labels_file = LabelsFile {
        labels: labels_map,
        assignments,
    };
    
    let yaml = serde_yaml::to_string(&labels_file)
        .context("Failed to serialize labels to YAML")?;
    
    fs::write(path, yaml)
        .context("Failed to write labels.yaml")?;
    
    Ok(())
}
