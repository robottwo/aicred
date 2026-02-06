//! Summary phase - confirm and write configuration

use anyhow::{Context, Result};
use aicred_core::ProviderInstance;
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
    
    // Write labels file if we have labels
    if !labels.is_empty() {
        write_labels_file(&labels_file, &labels)?;
    }
    
    Ok((instances, labels, config_dir))
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
    
    // Serialize to YAML
    let mut output = String::new();
    
    // Write labels section
    if !labels_map.is_empty() {
        output.push_str("labels:\n");
        for (name, label) in labels_map {
            output.push_str(&format!("  {}:\n", name));
            output.push_str(&format!("    name: {}\n", label.name));
            output.push_str(&format!("    created_at: {}\n", label.created_at.to_rfc3339()));
        }
        output.push('\n');
    }
    
    // Write assignments section
    if !assignments.is_empty() {
        output.push_str("assignments:\n");
        for assignment in assignments {
            output.push_str(&format!("  - label_name: {}\n", assignment.label_name));
            output.push_str("    target:\n");
            if let LabelTarget::ProviderModel { instance_id, model_id } = &assignment.target {
                output.push_str("      type: provider_model\n");
                output.push_str(&format!("      instance_id: {}\n", instance_id));
                output.push_str(&format!("      model_id: {}\n", model_id));
            }
            output.push_str(&format!("    assigned_at: {}\n", assignment.assigned_at.to_rfc3339()));
        }
    }
    
    fs::write(path, output)
        .context("Failed to write labels.yaml")?;
    
    Ok(())
}
