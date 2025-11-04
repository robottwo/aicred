//! Tag management commands for the aicred CLI.

use aicred_core::models::{Tag, TagAssignment};
use anyhow::Result;
use colored::*;
use std::path::Path;

/// Load all tags from the configuration directory
pub fn load_tags_with_home(home: Option<&Path>) -> Result<Vec<Tag>> {
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
    let tags: Vec<Tag> = serde_yaml::from_str(&content)?;
    Ok(tags)
}

/// Load all tags from the configuration directory
pub fn load_tags() -> Result<Vec<Tag>> {
    load_tags_with_home(None)
}

/// Save tags to the configuration directory
pub fn save_tags_with_home(tags: &[Tag], home: Option<&Path>) -> Result<()> {
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

/// Save tags to the configuration directory
pub fn save_tags(tags: &[Tag]) -> Result<()> {
    save_tags_with_home(tags, None)
}

/// Load all tag assignments from the configuration directory
pub fn load_tag_assignments_with_home(home: Option<&Path>) -> Result<Vec<TagAssignment>> {
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
    let assignments: Vec<TagAssignment> = serde_yaml::from_str(&content)?;
    Ok(assignments)
}

/// Load all tag assignments from the configuration directory
pub fn load_tag_assignments() -> Result<Vec<TagAssignment>> {
    load_tag_assignments_with_home(None)
}

/// Save tag assignments to the configuration directory
pub fn save_tag_assignments_with_home(
    assignments: &[TagAssignment],
    home: Option<&Path>,
) -> Result<()> {
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

/// Save tag assignments to the configuration directory
pub fn save_tag_assignments(assignments: &[TagAssignment]) -> Result<()> {
    save_tag_assignments_with_home(assignments, None)
}

/// Generate a unique tag ID
fn generate_tag_id(name: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash_result = hasher.finalize();
    format!("tag-{:x}", hash_result)
}

/// Handle the tags list command
pub fn handle_list_tags() -> Result<()> {
    let tags = load_tags()?;

    if tags.is_empty() {
        println!("{}", "No tags configured.".yellow());
        println!("{}", "Use 'aicred tags add' to create a new tag.".dimmed());
        return Ok(());
    }

    println!("\n{}", "Configured Tags:".green().bold());

    for tag in &tags {
        println!("  {} - {}", tag.name.cyan().bold(), tag.id.dimmed());

        if let Some(ref description) = tag.description {
            println!("    Description: {}", description);
        }

        if let Some(ref color) = tag.color {
            println!("    Color: {}", color);
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
    color: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut tags = load_tags()?;

    // Check if tag with this name already exists
    if tags.iter().any(|tag| tag.name == name) {
        return Err(anyhow::anyhow!("Tag with name '{}' already exists", name));
    }

    let tag_id = generate_tag_id(&name);
    let mut tag = Tag::new(tag_id, name.clone());

    if let Some(desc) = description {
        tag = tag.with_description(desc);
    }

    if let Some(col) = color {
        tag = tag.with_color(col);
    }

    // Validate the tag
    if let Err(e) = tag.validate() {
        return Err(anyhow::anyhow!("Invalid tag configuration: {}", e));
    }

    tags.push(tag);

    // Save to disk
    save_tags(&tags)?;

    println!("{} Tag '{}' added successfully.", "✓".green(), name);

    Ok(())
}

/// Handle the tags remove command
pub fn handle_remove_tag(name: String, force: bool) -> Result<()> {
    let mut tags = load_tags()?;
    let mut assignments = load_tag_assignments()?;

    // Find the tag
    let tag_index = tags.iter().position(|tag| tag.name == name);
    if tag_index.is_none() {
        return Err(anyhow::anyhow!("Tag with name '{}' not found", name));
    }

    let tag = tags[tag_index.unwrap()].clone();

    // Check if tag is assigned to any instances/models
    let assigned_count = assignments
        .iter()
        .filter(|assignment| assignment.tag_id == tag.id)
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
        assignments.retain(|assignment| assignment.tag_id != tag.id);
        save_tag_assignments(&assignments)?;
    }

    // Remove the tag
    tags.remove(tag_index.unwrap());
    save_tags(&tags)?;

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
    color: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut tags = load_tags()?;

    // Find the tag
    let tag_index = tags.iter().position(|tag| tag.name == name);
    if tag_index.is_none() {
        return Err(anyhow::anyhow!("Tag with name '{}' not found", name));
    }

    let tag = &mut tags[tag_index.unwrap()];

    // Update fields if provided
    if let Some(desc) = description {
        tag.set_description(Some(desc));
    }

    if let Some(col) = color {
        tag.set_color(Some(col));
    }

    // Validate the updated tag
    if let Err(e) = tag.validate() {
        return Err(anyhow::anyhow!("Invalid tag configuration: {}", e));
    }

    // Save to disk
    save_tags(&tags)?;

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
) -> Result<()> {
    let tags = load_tags()?;
    let mut assignments = load_tag_assignments()?;

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

    // Create assignment ID
    let assignment_id = format!(
        "assignment-{}-{}",
        tag.id,
        if let Some(model) = &target_model_id {
            format!("{}-{}", target_instance_id, model)
        } else {
            target_instance_id.clone()
        }
    );

    // Check if assignment already exists
    let assignment_exists = assignments.iter().any(|assignment| {
        assignment.tag_id == tag.id
            && assignment.targets_instance(&target_instance_id)
            && assignment.targets_model(
                &target_instance_id,
                target_model_id.as_deref().unwrap_or(""),
            )
    });

    if assignment_exists {
        return Err(anyhow::anyhow!(
            "Tag '{}' is already assigned to the specified target",
            tag_name
        ));
    }

    // Create the assignment
    let assignment = if let Some(model) = target_model_id {
        TagAssignment::new_to_model(assignment_id, tag.id.clone(), target_instance_id, model)
    } else {
        TagAssignment::new_to_instance(assignment_id, tag.id.clone(), target_instance_id)
    };

    // Validate the assignment
    if let Err(e) = assignment.validate() {
        return Err(anyhow::anyhow!("Invalid assignment: {}", e));
    }

    assignments.push(assignment);
    save_tag_assignments(&assignments)?;

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
) -> Result<()> {
    let tags = load_tags()?;
    let mut assignments = load_tag_assignments()?;

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
    assignments.retain(|assignment| {
        !(assignment.tag_id == tag.id
            && assignment.targets_instance(&target_instance_id)
            && assignment.targets_model(
                &target_instance_id,
                target_model_id.as_deref().unwrap_or(""),
            ))
    });

    if assignments.len() == original_count {
        return Err(anyhow::anyhow!(
            "Tag '{}' is not assigned to the specified target",
            tag_name
        ));
    }

    save_tag_assignments(&assignments)?;

    println!(
        "{} Tag '{}' unassigned successfully.",
        "✓".green(),
        tag_name.cyan()
    );

    Ok(())
}

/// Get tags assigned to a specific instance or model
pub fn get_tags_for_target(instance_id: &str, model_id: Option<&str>) -> Result<Vec<Tag>> {
    let tags = load_tags()?;
    let assignments = load_tag_assignments()?;

    let mut result = Vec::new();

    for assignment in assignments {
        if assignment.targets_instance(instance_id)
            && assignment.targets_model(instance_id, model_id.unwrap_or(""))
        {
            if let Some(tag) = tags.iter().find(|tag| tag.id == assignment.tag_id) {
                result.push(tag.clone());
            }
        }
    }

    Ok(result)
}
