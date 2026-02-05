//! Label management commands for the aicred CLI.

use aicred_core::models::{Label, LabelAssignment, LabelTarget};
use anyhow::Result;
use colored::*;
use std::path::Path;

/// Load all labels from the configuration directory
pub fn load_tags(home: Option<&Path>) -> Result<Vec<Label>> {
    let config_dir = match home {
        Some(h) => h.to_path_buf(),
        None => {
            // Check HOME environment variable first (for test compatibility)
            if let Ok(home_env) = std::env::var("HOME") {
                std::path::PathBuf::from(home_env)
            } else {
                dirs_next::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            }
        }
    }
    .join(".config")
    .join("aicred");

    let tags_file = config_dir.join("tags.yaml");

    if !tags_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&tags_file)?;
    let tags: Vec<Label> = serde_yaml::from_str(&content)?;
    Ok(tags)
}

/// Save tags to the configuration directory
pub fn save_tags(tags: &[Label], home: Option<&Path>) -> Result<()> {
    let config_dir = match home {
        Some(h) => h.to_path_buf(),
        None => {
            // Check HOME environment variable first (for test compatibility)
            if let Ok(home_env) = std::env::var("HOME") {
                std::path::PathBuf::from(home_env)
            } else {
                dirs_next::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            }
        }
    }
    .join(".config")
    .join("aicred");

    std::fs::create_dir_all(&config_dir)?;

    let tags_file = config_dir.join("tags.yaml");
    let content = serde_yaml::to_string(tags)?;
    std::fs::write(&tags_file, content)?;

    Ok(())
}

/// Load all label assignments from the configuration directory
pub fn load_tag_assignments(home: Option<&Path>) -> Result<Vec<LabelAssignment>> {
    let config_dir = match home {
        Some(h) => h.to_path_buf(),
        None => {
            // Check HOME environment variable first (for test compatibility)
            if let Ok(home_env) = std::env::var("HOME") {
                std::path::PathBuf::from(home_env)
            } else {
                dirs_next::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            }
        }
    }
    .join(".config")
    .join("aicred");

    let assignments_file = config_dir.join("tag_assignments.yaml");

    if !assignments_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&assignments_file)?;
    let assignments: Vec<LabelAssignment> = serde_yaml::from_str(&content)?;
    Ok(assignments)
}

/// Save tag assignments to the configuration directory
pub fn save_tag_assignments(assignments: &[LabelAssignment], home: Option<&Path>) -> Result<()> {
    let config_dir = match home {
        Some(h) => h.to_path_buf(),
        None => {
            // Check HOME environment variable first (for test compatibility)
            if let Ok(home_env) = std::env::var("HOME") {
                std::path::PathBuf::from(home_env)
            } else {
                dirs_next::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            }
        }
    }
    .join(".config")
    .join("aicred");

    std::fs::create_dir_all(&config_dir)?;

    let assignments_file = config_dir.join("tag_assignments.yaml");
    let content = serde_yaml::to_string(assignments)?;
    std::fs::write(&assignments_file, content)?;

    Ok(())
}

/// Handle the tags list command
pub fn handle_list_tags(home: Option<&Path>) -> Result<()> {
    let tags = load_tags(home)?;

    if tags.is_empty() {
        println!("{}", "No tags configured.".yellow());
        println!("{}", "Use 'aicred tags add' to create a new tag.".dimmed());
        return Ok(());
    }

    println!("\n{}", "Configured Tags:".green().bold());

    for tag in &tags {
        println!("  {} - {}", tag.name.cyan().bold(), tag.name.dimmed());

        if let Some(ref description) = tag.description {
            println!("    Description: {}", description);
        }

        println!(
            "    Created: {}",
            tag.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    println!("{}", format!("Total tags: {}", tags.len()).cyan());

    Ok(())
}

/// Handle the tags add command
pub fn handle_add_tag(
    name: String,
    _color: Option<String>,  // Color not supported in Label
    description: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    let mut tags = load_tags(home)?;

    // Check if tag with this name already exists
    if tags.iter().any(|tag| tag.name == name) {
        return Err(anyhow::anyhow!("Tag with name '{}' already exists", name));
    }

    let now = chrono::Utc::now();
    let mut tag = Label {
        name: name.clone(),
        description,
        created_at: now,
        metadata: std::collections::HashMap::new(),
    };

    tags.push(tag);

    // Save to disk
    save_tags(&tags, home)?;

    println!("{} Tag '{}' added successfully.", "✓".green(), name);

    Ok(())
}

/// Handle the tags remove command
pub fn handle_remove_tag(name: String, force: bool, home: Option<&Path>) -> Result<()> {
    let mut tags = load_tags(home)?;
    let mut assignments = load_tag_assignments(home)?;

    // Find the tag
    let tag_index = tags.iter().position(|tag| tag.name == name);
    if tag_index.is_none() {
        return Err(anyhow::anyhow!("Tag with name '{}' not found", name));
    }

    let tag = tags[tag_index.unwrap()].clone();

    // Check if tag is assigned to any instances/models
    let assigned_count = assignments
        .iter()
        .filter(|assignment| assignment.label_name == tag.name)
        .count();

    if assigned_count > 0 && !force {
        println!(
            "{}",
            "Warning: This tag is currently assigned to instances/models."
                .yellow()
                .bold()
        );
        println!("Tag: {} ({} assignments)", tag.name.cyan(), assigned_count);
        print!("Are you sure you want to remove it? (y/N): ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Removal cancelled.".dimmed());
            return Ok(());
        }
    }

    // Remove tag assignments if force is used or user confirmed
    if assigned_count > 0 {
        assignments.retain(|assignment| assignment.label_name != tag.name);
        save_tag_assignments(&assignments, home)?;
    }

    // Remove the tag
    tags.remove(tag_index.unwrap());
    save_tags(&tags, home)?;

    println!(
        "{} Tag '{}' removed successfully.",
        "✓".green(),
        name.cyan()
    );

    if assigned_count > 0 {
        println!("  Removed {} assignment(s)", assigned_count);
    }

    Ok(())
}

/// Handle the tags update command
pub fn handle_update_tag(
    name: String,
    _color: Option<String>,  // Color not supported in Label
    description: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    let mut tags = load_tags(home)?;

    // Find the tag
    let tag_index = tags.iter().position(|tag| tag.name == name);
    if tag_index.is_none() {
        return Err(anyhow::anyhow!("Tag with name '{}' not found", name));
    }

    let tag = &mut tags[tag_index.unwrap()];

    // Update fields if provided
    if description.is_some() {
        tag.description = description;
    }

    // Save to disk
    save_tags(&tags, home)?;

    println!(
        "{} Tag '{}' updated successfully.",
        "✓".green(),
        name.cyan()
    );

    Ok(())
}

/// Handle the tags assign command
pub fn handle_assign_tag(
    tag_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    let tags = load_tags(home)?;
    let mut assignments = load_tag_assignments(home)?;

    // Find the tag
    let tag = tags
        .iter()
        .find(|tag| tag.name == tag_name)
        .ok_or_else(|| anyhow::anyhow!("Tag with name '{}' not found", tag_name))?;

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

    // Create assignment
    let assignment = if let Some(model) = target_model_id {
        LabelAssignment {
            label_name: tag.name.clone(),
            target: LabelTarget::ProviderModel {
                instance_id: target_instance_id,
                model_id: model,
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        }
    } else {
        LabelAssignment {
            label_name: tag.name.clone(),
            target: LabelTarget::ProviderInstance {
                instance_id: target_instance_id,
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        }
    };

    // Check if assignment already exists
    let assignment_exists = assignments.iter().any(|existing| {
        existing.label_name == assignment.label_name && match (&existing.target, &assignment.target) {
            (LabelTarget::ProviderInstance { instance_id: e_i }, LabelTarget::ProviderInstance { instance_id: a_i }) => e_i == a_i,
            (LabelTarget::ProviderModel { instance_id: e_i, model_id: e_m }, LabelTarget::ProviderModel { instance_id: a_i, model_id: a_m }) => e_i == a_i && e_m == a_m,
            _ => false,
        }
    });

    if assignment_exists {
        return Err(anyhow::anyhow!(
            "Tag '{}' is already assigned to the specified target",
            tag_name
        ));
    }

    assignments.push(assignment);
    save_tag_assignments(&assignments, home)?;

    println!(
        "{} Tag '{}' assigned successfully.",
        "✓".green(),
        tag_name.cyan()
    );

    Ok(())
}

/// Handle the tags unassign command
pub fn handle_unassign_tag(
    tag_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    let tags = load_tags(home)?;
    let mut assignments = load_tag_assignments(home)?;

    // Find the tag
    let tag = tags
        .iter()
        .find(|tag| tag.name == tag_name)
        .ok_or_else(|| anyhow::anyhow!("Tag with name '{}' not found", tag_name))?;

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

    // Filter assignments - keep those that don't match the target to remove
    let mut filtered_assignments = Vec::new();
    for assignment in assignments {
        if assignment.label_name != tag.name {
            filtered_assignments.push(assignment);
            continue;
        }

        let matches_target = match &assignment.target {
            LabelTarget::ProviderInstance { instance_id } => {
                // Match if instance IDs match and model_id is None
                if let Some(model_id) = &target_model_id {
                    false  // Should not match if model_id is specified
                } else {
                    instance_id == &target_instance_id
                }
            }
            LabelTarget::ProviderModel { instance_id, model_id } => {
                // Match if both instance and model match
                if let (Some(inst), Some(mod)) = (&target_instance_id.as_str(), target_model_id.as_ref()) {
                    instance_id == inst && model_id == mod
                } else {
                    false
                }
            }
        };

        if !matches_target {
            filtered_assignments.push(assignment);
        }
    }

    *assignments = filtered_assignments;

    if assignments.len() == original_count {
        return Err(anyhow::anyhow!(
            "Tag '{}' is not assigned to the specified target",
            tag_name
        ));
    }

    save_tag_assignments(&assignments, home)?;

    println!(
        "{} Tag '{}' unassigned successfully.",
        "✓".green(),
        tag_name.cyan()
    );

    Ok(())
}

/// Get tags assigned to a specific instance or model
pub fn get_tags_for_target(
    instance_id: &str,
    model_id: Option<&str>,
    home: Option<&Path>,
) -> Result<Vec<Label>> {
    let tags = load_tags(home)?;
    let assignments = load_tag_assignments(home)?;

    let mut result = Vec::new();

    for assignment in assignments {
        let matches_target = match (&assignment.target, model_id) {
            (LabelTarget::ProviderInstance { instance_id: inst_id }, None) => inst_id == instance_id,
            (LabelTarget::ProviderModel { instance_id: inst_id, model_id: mod_id }, Some(model)) => {
                inst_id == instance_id && mod_id == model
            }
            _ => false,
        };

        if matches_target {
            if let Some(tag) = tags.iter().find(|tag| tag.name == assignment.label_name) {
                result.push(tag.clone());
            }
        }
    }

    Ok(result)
}
