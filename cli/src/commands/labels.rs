//! Label management commands for the aicred CLI.

use aicred_core::models::{Label, LabelAssignment, LabelAssignmentTarget};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load all labels from the configuration directory
pub fn load_labels() -> Result<Vec<Label>> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    let labels_file = config_dir.join("labels.yaml");

    if !labels_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&labels_file)?;
    let labels: Vec<Label> = serde_yaml::from_str(&content)?;
    Ok(labels)
}

/// Save labels to the configuration directory
pub fn save_labels(labels: &[Label]) -> Result<()> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    std::fs::create_dir_all(&config_dir)?;

    let labels_file = config_dir.join("labels.yaml");
    let content = serde_yaml::to_string(labels)?;
    std::fs::write(&labels_file, content)?;

    Ok(())
}

/// Load all label assignments from the configuration directory
pub fn load_label_assignments() -> Result<Vec<LabelAssignment>> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    let assignments_file = config_dir.join("label_assignments.yaml");

    if !assignments_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&assignments_file)?;
    let assignments: Vec<LabelAssignment> = serde_yaml::from_str(&content)?;
    Ok(assignments)
}

/// Save label assignments to the configuration directory
pub fn save_label_assignments(assignments: &[LabelAssignment]) -> Result<()> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    std::fs::create_dir_all(&config_dir)?;

    let assignments_file = config_dir.join("label_assignments.yaml");
    let content = serde_yaml::to_string(assignments)?;
    std::fs::write(&assignments_file, content)?;

    Ok(())
}

/// Generate a unique label ID
fn generate_label_id(name: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash_result = hasher.finalize();
    format!("label-{:x}", hash_result)
}

/// Handle the labels list command
pub fn handle_list_labels() -> Result<()> {
    let labels = load_labels()?;
    let assignments = load_label_assignments()?;

    if labels.is_empty() {
        println!("{}", "No labels configured.".yellow());
        println!(
            "{}",
            "Use 'aicred labels add' to create a new label.".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Configured Labels:".green().bold());

    for label in &labels {
        println!("  {} - {}", label.name.cyan().bold(), label.id.dimmed());

        if let Some(ref description) = label.description {
            println!("    Description: {}", description);
        }

        if let Some(ref color) = label.color {
            println!("    Color: {}", color);
        }

        // Check if label is assigned
        let is_assigned = assignments
            .iter()
            .any(|assignment| assignment.label_id == label.id);
        let assignment_status = if is_assigned {
            "Assigned".yellow()
        } else {
            "Unassigned".green()
        };
        println!("    Status: {}", assignment_status);

        println!(
            "    Created: {}",
            label.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    println!("{}", format!("Total labels: {}", labels.len()).cyan());

    Ok(())
}

/// Handle the labels add command
pub fn handle_add_label(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut labels = load_labels()?;
    let assignments = load_label_assignments()?;

    // Check if label with this name already exists
    if labels.iter().any(|label| label.name == name) {
        return Err(anyhow::anyhow!("Label with name '{}' already exists", name));
    }

    // Check if label is already assigned (uniqueness constraint)
    if assignments
        .iter()
        .any(|assignment| assignment.label_id == name)
    {
        return Err(anyhow::anyhow!(
            "Label '{}' is already assigned and cannot be recreated",
            name
        ));
    }

    let label_id = generate_label_id(&name);
    let mut label = Label::new(label_id, name.clone());

    if let Some(desc) = description {
        label = label.with_description(desc);
    }

    if let Some(col) = color {
        label = label.with_color(col);
    }

    // Validate the label
    if let Err(e) = label.validate() {
        return Err(anyhow::anyhow!("Invalid label configuration: {}", e));
    }

    labels.push(label);

    // Save to disk
    save_labels(&labels)?;

    println!("{} Label '{}' added successfully.", "✓".green(), name);

    Ok(())
}

/// Handle the labels remove command
pub fn handle_remove_label(name: String, force: bool) -> Result<()> {
    let mut labels = load_labels()?;
    let mut assignments = load_label_assignments()?;

    // Find the label
    let label_index = labels.iter().position(|label| label.name == name);
    if label_index.is_none() {
        return Err(anyhow::anyhow!("Label with name '{}' not found", name));
    }

    let label = labels[label_index.unwrap()].clone();
    let label_id = label.id.clone();

    // Check if label is assigned (labels can only be assigned to one target)
    let assigned_assignment = assignments
        .iter()
        .find(|assignment| assignment.label_id == label_id);
    let has_assignment = assigned_assignment.is_some();

    if let Some(assignment) = assigned_assignment {
        if !force {
            println!(
                "{}",
                "Warning: This label is currently assigned and cannot be removed."
                    .red()
                    .bold()
            );
            println!("Label: {}", label.name.cyan());
            println!("Assigned to: {}", assignment.target_description());
            println!("Labels are unique and must be unassigned before removal.");
            return Ok(());
        }

        // Remove the assignment if force is used
        assignments.retain(|a| a.label_id != label_id);
        save_label_assignments(&assignments)?;
    }

    // Remove the label
    labels.remove(label_index.unwrap());
    save_labels(&labels)?;

    println!("{} Label '{}' removed successfully.", "✓".green(), name);

    if has_assignment {
        println!("  Removed assignment");
    }

    Ok(())
}

/// Handle the labels update command
pub fn handle_update_label(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut labels = load_labels()?;

    // Find the label
    let label_index = labels.iter().position(|label| label.name == name);
    if label_index.is_none() {
        return Err(anyhow::anyhow!("Label with name '{}' not found", name));
    }

    let label = &mut labels[label_index.unwrap()];

    // Update fields if provided
    if let Some(desc) = description {
        label.set_description(Some(desc));
    }

    if let Some(col) = color {
        label.set_color(Some(col));
    }

    // Validate the updated label
    if let Err(e) = label.validate() {
        return Err(anyhow::anyhow!("Invalid label configuration: {}", e));
    }

    // Save to disk
    save_labels(&labels)?;

    println!(
        "{} Label '{}' updated successfully.",
        "✓".green(),
        name.cyan()
    );

    Ok(())
}

/// Handle the labels assign command
pub fn handle_assign_label(
    label_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<()> {
    let mut labels = load_labels()?;
    let mut assignments = load_label_assignments()?;

    // Find the label
    let label = labels
        .iter()
        .find(|label| label.name == label_name)
        .ok_or_else(|| anyhow::anyhow!("Label with name '{}' not found", label_name))?;

    // Validate target parameters
    let (target_instance_id, target_model_id) = match (instance_id, model_id) {
        (Some(instance), None) => (instance, None),
        (Some(instance), Some(model)) => (instance, Some(model)),
        (None, Some(_)) => {
            return Err(anyhow::anyhow!(
                "Instance ID is required when specifying a model"
            ));
        }
        (None, None) => {
            return Err(anyhow::anyhow!(
                "Either instance ID or model ID must be specified"
            ));
        }
    };

    // Check if label is already assigned (uniqueness constraint)
    if assignments
        .iter()
        .any(|assignment| assignment.label_id == label.id)
    {
        let existing_assignment = assignments
            .iter()
            .find(|assignment| assignment.label_id == label.id)
            .unwrap();
        return Err(anyhow::anyhow!(
            "Label '{}' is already assigned to {}",
            label_name,
            existing_assignment.target_description()
        ));
    }

    // Create assignment ID
    let assignment_id = format!(
        "assignment-{}-{}",
        label.id,
        if let Some(model) = &target_model_id {
            format!("{}-{}", target_instance_id, model)
        } else {
            target_instance_id.clone()
        }
    );

    // Create the assignment
    let assignment = if let Some(model) = target_model_id {
        LabelAssignment::new_to_model(assignment_id, label.id.clone(), target_instance_id, model)
    } else {
        LabelAssignment::new_to_instance(assignment_id, label.id.clone(), target_instance_id)
    };

    // Validate the assignment
    if let Err(e) = assignment.validate() {
        return Err(anyhow::anyhow!("Invalid assignment: {}", e));
    }

    assignments.push(assignment);
    save_label_assignments(&assignments)?;

    println!(
        "{} Label '{}' assigned successfully.",
        "✓".green(),
        label_name.cyan()
    );

    Ok(())
}

/// Handle the labels unassign command
pub fn handle_unassign_label(
    label_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<()> {
    let labels = load_labels()?;
    let mut assignments = load_label_assignments()?;

    // Find the label
    let label = labels
        .iter()
        .find(|label| label.name == label_name)
        .ok_or_else(|| anyhow::anyhow!("Label with name '{}' not found", label_name))?;

    // Validate target parameters
    let (target_instance_id, target_model_id) = match (instance_id, model_id) {
        (Some(instance), None) => (instance, None),
        (Some(instance), Some(model)) => (instance, Some(model)),
        (None, Some(_)) => {
            return Err(anyhow::anyhow!(
                "Instance ID is required when specifying a model"
            ));
        }
        (None, None) => {
            return Err(anyhow::anyhow!(
                "Either instance ID or model ID must be specified"
            ));
        }
    };

    // Find and remove the assignment
    let original_count = assignments.len();
    assignments.retain(|assignment| {
        !(assignment.label_id == label.id
            && assignment.targets_instance(&target_instance_id)
            && assignment.targets_model(
                &target_instance_id,
                &target_model_id.as_deref().unwrap_or(""),
            ))
    });

    if assignments.len() == original_count {
        return Err(anyhow::anyhow!(
            "Label '{}' is not assigned to the specified target",
            label_name
        ));
    }

    save_label_assignments(&assignments)?;

    println!(
        "{} Label '{}' unassigned successfully.",
        "✓".green(),
        label_name.cyan()
    );

    Ok(())
}

/// Get labels assigned to a specific instance or model
pub fn get_labels_for_target(instance_id: &str, model_id: Option<&str>) -> Result<Vec<Label>> {
    let labels = load_labels()?;
    let assignments = load_label_assignments()?;

    let mut result = Vec::new();

    for assignment in assignments {
        if assignment.targets_instance(instance_id)
            && assignment.targets_model(instance_id, model_id.unwrap_or(""))
        {
            if let Some(label) = labels.iter().find(|label| label.id == assignment.label_id) {
                result.push(label.clone());
            }
        }
    }

    Ok(result)
}
