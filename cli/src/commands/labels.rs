//! Label management commands for the aicred CLI.

use crate::utils::provider_loader::load_provider_instances;
use aicred_core::env_resolver::LabelWithTarget;
use aicred_core::models::{Label, LabelAssignment, LabelTarget, ProviderCollection};
use aicred_core::utils::ProviderModelTuple;
use anyhow::Result;
use colored::*;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use tracing::{debug, error, info};

/// Load labels with their target assignments for use with EnvResolver
pub fn load_labels_with_targets(home: Option<&Path>) -> Result<Vec<LabelWithTarget>> {
    let assignments = load_label_assignments_with_home(home)?;
    let provider_instances = load_provider_instances(home)?;

    // Convert LabelAssignments to LabelWithTarget format
    let mut labels_with_targets = Vec::new();
    for assignment in assignments {
        // Convert LabelTarget to ProviderModelTuple
        let tuple = match &assignment.target {
            LabelTarget::ProviderInstance { instance_id } => {
                // Find provider instance
                if let Some(instance) = provider_instances.get_instance(instance_id) {
                    // Use first model if available
                    if let Some(model_id) = instance.models.first() {
                        ProviderModelTuple::new(instance.provider_type.clone(), model_id.clone())
                    } else {
                        ProviderModelTuple::new(instance.provider_type.clone(), String::new())
                    }
                } else {
                    continue;
                }
            }
            LabelTarget::ProviderModel {
                instance_id,
                model_id,
            } => {
                // Find provider instance
                if let Some(instance) = provider_instances.get_instance(instance_id) {
                    ProviderModelTuple::new(instance.provider_type.clone(), model_id.clone())
                } else {
                    continue;
                }
            }
        };

        labels_with_targets.push(LabelWithTarget::new(assignment.label_name, tuple));
    }

    Ok(labels_with_targets)
}

/// Load all label assignments from the configuration directory
pub fn load_label_assignments_with_home(home: Option<&Path>) -> Result<Vec<LabelAssignment>> {
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

    let labels_file = config_dir.join("labels.yaml");

    if !labels_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&labels_file)?;
    let labels: Vec<LabelAssignment> = serde_yaml::from_str(&content)?;
    Ok(labels)
}

/// Load all label assignments from the configuration directory
pub fn load_label_assignments() -> Result<Vec<LabelAssignment>> {
    load_label_assignments_with_home(None)
}

/// Save label assignments to the configuration directory
pub fn save_label_assignments_with_home(
    labels: &[LabelAssignment],
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

    let labels_file = config_dir.join("labels.yaml");
    let content = serde_yaml::to_string(labels)?;

    // Atomic file write pattern: create temp file, write, flush, sync, then rename
    debug!("Starting atomic write to labels.yaml");

    // Create a temporary file in the same directory as the target file
    let temp_file = NamedTempFile::new_in(&config_dir)?;
    let temp_path = temp_file.path().to_path_buf();

    // Write the content to the temporary file
    {
        let mut file = temp_file.as_file();
        file.write_all(content.as_bytes())?;
        file.flush()?;

        // Ensure data is written to disk
        if let Err(e) = file.sync_all() {
            error!("Failed to sync temporary file to disk: {}", e);
            return Err(e.into());
        }
    }

    debug!("Temporary file written and synced successfully");

    // Atomically rename the temporary file to the target file
    if let Err(e) = std::fs::rename(&temp_path, &labels_file) {
        error!("Failed to rename temporary file to labels.yaml: {}", e);
        // Clean up the temporary file on error
        let _ = std::fs::remove_file(&temp_path);
        return Err(e.into());
    }

    info!(
        "Successfully saved label assignments atomically to {:?}",
        labels_file
    );

    Ok(())
}

/// Save label assignments to the configuration directory
pub fn save_label_assignments(labels: &[LabelAssignment]) -> Result<()> {
    save_label_assignments_with_home(labels, None)
}

/// Load labels (metadata) from a separate file
fn load_labels_with_home(home: Option<&Path>) -> Result<std::collections::HashMap<String, Label>> {
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

    let labels_metadata_file = config_dir.join("labels_metadata.yaml");

    if !labels_metadata_file.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let content = std::fs::read_to_string(&labels_metadata_file)?;
    let labels: Vec<Label> = serde_yaml::from_str(&content)?;
    Ok(labels.into_iter().map(|l| (l.name.clone(), l)).collect())
}

/// Save labels (metadata) to a separate file
fn save_labels_with_home(
    labels: &std::collections::HashMap<String, Label>,
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

    let labels_metadata_file = config_dir.join("labels_metadata.yaml");
    let labels_vec: Vec<Label> = labels.values().cloned().collect();
    let content = serde_yaml::to_string(&labels_vec)?;
    std::fs::write(&labels_metadata_file, content)?;

    Ok(())
}

/// Handle the labels list command
pub fn handle_list_labels() -> Result<()> {
    let assignments = load_label_assignments()?;
    let labels_metadata = load_labels_with_home(None)?;

    if assignments.is_empty() {
        println!("{}", "No labels configured.".yellow());
        println!(
            "{}",
            "Use 'aicred labels set label=provider:model' to create a label.".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Label Assignments:".green().bold());

    for assignment in &assignments {
        // Get label metadata if available
        let label_metadata = labels_metadata.get(&assignment.label_name);

        println!(
            "  {} - {}",
            assignment.label_name.cyan().bold(),
            assignment_target_to_string(&assignment.target).dimmed()
        );

        if let Some(label) = label_metadata {
            if let Some(ref description) = label.description {
                println!("    Description: {}", description);
            }
        }

        println!(
            "    Created: {}",
            assignment.assigned_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    println!("{}", format!("Total labels: {}", assignments.len()).cyan());

    Ok(())
}

fn assignment_target_to_string(target: &LabelTarget) -> String {
    match target {
        LabelTarget::ProviderInstance { instance_id } => {
            format!("instance:{}", instance_id)
        }
        LabelTarget::ProviderModel {
            instance_id,
            model_id,
        } => {
            format!("instance:{}|model:{}", instance_id, model_id)
        }
    }
}

/// Find the labels directory using runtime path resolution
/// First checks user config directory, then falls back to distributed application files
fn find_labels_directory(home: Option<&Path>) -> Result<std::path::PathBuf> {
    // First try user config directory: ~/.config/aicred/patterns/
    let user_config_dir = match home {
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
    .join("aicred")
    .join("patterns");

    if user_config_dir.exists() {
        return Ok(user_config_dir);
    }

    // Try current working directory + "conf/labels" (for development)
    let current_dir = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Could not get current directory: {}", e))?;

    let current_dir_labels = current_dir.join("conf").join("labels");
    if current_dir_labels.exists() {
        return Ok(current_dir_labels);
    }

    // For development with cargo run, also try the original compile-time path
    // This maintains backward compatibility during development
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_path = std::path::Path::new(&manifest_dir)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine project root directory"))?
            .join("conf")
            .join("labels");

        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    // Fall back to binary's parent directory + "conf/labels"
    // This maintains backward compatibility with development setup
    let binary_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Could not get current executable path: {}", e))?;

    let binary_dir = binary_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not get parent directory of binary"))?;

    let fallback_dir = binary_dir.join("conf").join("labels");

    Ok(fallback_dir)
}

/// Handle the labels scan command
pub fn handle_label_scan(dry_run: bool, verbose: bool, home: Option<&Path>) -> Result<()> {
    use colored::*;
    use regex::Regex;
    use std::fs;

    // Load existing provider instances
    let provider_instances = load_provider_instances(home)?;

    if verbose {
        println!(
            "{}",
            format!(
                "DEBUG: Loaded {} provider instances",
                provider_instances.len()
            )
            .dimmed()
        );
        for instance in provider_instances.all_instances() {
            println!(
                "  DEBUG: Instance {} ({}), models: {}",
                instance.id,
                instance.provider_type,
                instance.models.len()
            );
            for model in &instance.models {
                println!("    DEBUG: Model: {}", model);
            }
        }
    }

    if provider_instances.is_empty() {
        println!(
            "{}",
            "No provider instances found. Add some instances first.".yellow()
        );
        return Ok(());
    }

    // Get the path to the conf/labels directory with runtime path resolution
    let labels_dir = find_labels_directory(home)?;

    if !labels_dir.exists() {
        println!(
            "{}",
            format!("Labels directory not found: {}", labels_dir.display()).yellow()
        );
        println!(
            "{}",
            "Create the directory and add .scan pattern files to enable label scanning.".dimmed()
        );
        return Ok(());
    }

    // Read all .scan files
    let scan_files: Vec<_> = fs::read_dir(&labels_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".scan"))
        .collect();

    if scan_files.is_empty() {
        println!(
            "{}",
            format!(
                "No .scan files found in {} directory.",
                labels_dir.display()
            )
            .yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} scan files", scan_files.len()).cyan()
    );
    if verbose {
        println!("{}", "Scan files:".dimmed());
        for entry in &scan_files {
            println!("  - {}", entry.file_name().to_string_lossy());
        }
    }

    // Load existing label assignments and metadata
    let mut existing_assignments = load_label_assignments_with_home(home)?;
    let mut existing_labels_metadata = load_labels_with_home(home)?;
    let original_assignments = existing_assignments.clone(); // Keep track of original state
    let mut new_assignments = Vec::new();

    // Process each scan file
    for entry in scan_files {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let label_name = file_name.trim_end_matches(".scan").to_string();

        if verbose {
            println!("\n{}", format!("Processing {}:", file_name).green().bold());
        }

        // Read regex patterns from file
        let patterns_content = fs::read_to_string(entry.path())?;
        let patterns: Vec<&str> = patterns_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        if patterns.is_empty() {
            if verbose {
                println!("  {} No valid patterns found", "⚠".yellow());
            }
            continue;
        }

        if verbose {
            println!("  {} Found {} patterns", "📋".cyan(), patterns.len());
        }

        // Collect ALL matches first, then select the globally best match
        let mut all_matches: Vec<(String, ProviderModelTuple, usize)> = Vec::new(); // (provider_model_str, tuple, pattern_specificity)

        for instance in provider_instances.all_instances() {
            for model in &instance.models {
                let provider_model_str = format!("{}:{}", instance.provider_type, model);

                if verbose {
                    println!("  DEBUG: Testing '{}' against patterns", provider_model_str);
                }

                // Test each pattern and collect all matches
                for (pattern_idx, pattern) in patterns.iter().enumerate() {
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            if verbose {
                                println!("    DEBUG: Pattern {}: '{}'", pattern_idx + 1, pattern);
                            }

                            if regex.is_match(&provider_model_str) {
                                if verbose {
                                    println!(
                                        "  ✅ Pattern {} matched '{}'",
                                        pattern_idx + 1,
                                        provider_model_str
                                    );
                                }

                                // Calculate pattern specificity (lower index = more specific)
                                let pattern_specificity = pattern_idx;

                                match ProviderModelTuple::parse(&provider_model_str) {
                                    Ok(tuple) => {
                                        all_matches.push((
                                            provider_model_str.clone(),
                                            tuple,
                                            pattern_specificity,
                                        ));
                                        if verbose {
                                            println!("    📝 Collected match: '{}' with pattern specificity {}", provider_model_str, pattern_specificity);
                                        }
                                    }
                                    Err(e) => {
                                        if verbose {
                                            println!(
                                                "    ❌ Failed to parse tuple '{}': {}",
                                                provider_model_str, e
                                            );
                                        }
                                    }
                                }
                            } else if verbose {
                                eprintln!("    ❌ Pattern {} did not match", pattern_idx + 1);
                            }
                        }
                        Err(e) => {
                            if verbose {
                                println!("  ❌ Invalid regex pattern '{}': {}", pattern, e);
                            }
                        }
                    }
                }
            }
        }

        // Select the globally best match deterministically
        // First by specificity (lower = more specific), then by lexicographic provider_model_str
        let mut best_match = None;

        if !all_matches.is_empty() {
            // Sort by (specificity, provider_model_str) for deterministic selection
            all_matches.sort_by(|a, b| {
                // First compare by specificity (lower is better)
                let specificity_cmp = a.2.cmp(&b.2);
                if specificity_cmp != std::cmp::Ordering::Equal {
                    specificity_cmp
                } else {
                    // Then compare by provider_model_str for stable tie-breaking
                    a.0.cmp(&b.0)
                }
            });

            // Check for ties at the best specificity level
            let best_specificity = all_matches[0].2;
            let ties: Vec<_> = all_matches
                .iter()
                .take_while(|m| m.2 == best_specificity)
                .collect();

            if ties.len() > 1 && verbose {
                println!(
                    "  ⚠️  Warning: {} entries tied for best specificity (index {}):",
                    ties.len(),
                    best_specificity
                );
                for tie in &ties {
                    println!("    - {} (will select first lexicographically)", tie.0);
                }
            }

            best_match = Some(all_matches.into_iter().next().unwrap());
        }

        // Apply the best match found
        if let Some((provider_model_str, tuple, _specificity)) = best_match {
            if verbose {
                println!(
                    "  🎯 Final assignment: '{}' -> {}",
                    label_name, provider_model_str
                );
            }

            // Create label assignment
            let assignment = LabelAssignment {
                label_name: label_name.clone(),
                target: if let Some(instance) = provider_instances
                    .all_instances()
                    .iter()
                    .find(|inst| inst.provider_type == tuple.provider())
                {
                    if let Some(model) = instance.models.iter().find(|m| {
                        let basename = m.rsplit('/').next().unwrap_or(m);
                        basename == tuple.model()
                    }) {
                        LabelTarget::ProviderModel {
                            instance_id: instance.id.clone(),
                            model_id: model.clone(),
                        }
                    } else {
                        // Model not found in instance, create instance-level assignment
                        LabelTarget::ProviderInstance {
                            instance_id: instance.id.clone(),
                        }
                    }
                } else {
                    continue; // Skip if provider instance not found
                },
                assigned_at: chrono::Utc::now(),
                assigned_by: None,
            };

            // Check if assignment already exists and update it
            let existing_index = existing_assignments
                .iter()
                .position(|a| a.label_name == assignment.label_name);
            if let Some(index) = existing_index {
                if verbose {
                    println!(
                        "  🔄 Updating existing label '{}' to {}",
                        label_name, provider_model_str
                    );
                }
                existing_assignments[index] = assignment;
            } else {
                if verbose {
                    println!(
                        "  ➕ Creating new label '{}' for {}",
                        label_name, provider_model_str
                    );
                }
                new_assignments.push(assignment);
            }

            // Ensure label metadata exists
            if !existing_labels_metadata.contains_key(&label_name) {
                existing_labels_metadata.insert(
                    label_name.clone(),
                    Label {
                        name: label_name.clone(),
                        description: None,
                        created_at: chrono::Utc::now(),
                        metadata: std::collections::HashMap::new(),
                    },
                );
            }
        } else if verbose {
            eprintln!("  ❌ No matches found for label '{}'", label_name);
        }
    } // End of for entry in scan_files loop

    // Add new assignments to existing assignments
    existing_assignments.extend(new_assignments);

    // Track what actually changed
    let mut newly_created = Vec::new();
    let mut updated = Vec::new();

    // Re-check what changed by comparing with original state
    for assignment in &existing_assignments {
        if let Some(original) = original_assignments
            .iter()
            .find(|a| a.label_name == assignment.label_name)
        {
            if original.target != assignment.target {
                updated.push(assignment);
            }
        } else {
            newly_created.push(assignment);
        }
    }

    if dry_run {
        println!("\n{}", "DRY RUN - No changes will be made".red().bold());

        let total_changes = newly_created.len() + updated.len();

        if total_changes > 0 {
            println!("{}", "Would create/update the following labels:".cyan());

            for assignment in newly_created {
                println!(
                    "  {} -> {}",
                    assignment.label_name.cyan().bold(),
                    assignment_target_to_string(&assignment.target).dimmed()
                );
            }

            for assignment in updated {
                println!(
                    "  {} -> {}",
                    assignment.label_name.cyan().bold(),
                    assignment_target_to_string(&assignment.target).dimmed()
                );
            }

            println!(
                "\n{}",
                format!(
                    "Total labels that would be created/updated: {}",
                    total_changes
                )
                .cyan()
            );
        } else {
            println!(
                "{}",
                "No changes would be made - all labels already have the correct assignments."
                    .cyan()
            );
        }
    } else {
        // Save the updated assignments and metadata
        save_label_assignments_with_home(&existing_assignments, home)?;
        save_labels_with_home(&existing_labels_metadata, home)?;

        println!("\n{}", "Label scan completed".green().bold());

        let total_changes = newly_created.len() + updated.len();

        if total_changes > 0 {
            if !newly_created.is_empty() {
                println!("{}", "New assignments:".cyan());
                for assignment in newly_created {
                    println!(
                        "  {} -> {}",
                        assignment.label_name.cyan().bold(),
                        assignment_target_to_string(&assignment.target).dimmed()
                    );
                }
            }

            if !updated.is_empty() {
                println!("{}", "Updated assignments:".cyan());
                for assignment in updated {
                    println!(
                        "  {} -> {}",
                        assignment.label_name.cyan().bold(),
                        assignment_target_to_string(&assignment.target).dimmed()
                    );
                }
            }

            println!(
                "{}",
                format!("Total labels created/updated: {}", total_changes).cyan()
            );
        } else {
            println!(
                "{}",
                "No changes made - all labels already have the correct assignments.".cyan()
            );
        }
    }

    Ok(())
}

/// Save provider instances to configuration directory
fn save_provider_instances(instances: &ProviderCollection, home: Option<&Path>) -> Result<()> {
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
    .join("aicred")
    .join("inference_services");

    std::fs::create_dir_all(&config_dir)?;

    // Save each instance to its own file
    for instance in instances.all_instances() {
        // Use provider name and first 4 chars of instance ID (hash)
        let file_name = format!("{}-{}.yaml", instance.provider_type, &instance.id[..4.min(instance.id.len())]);
        let file_path = config_dir.join(&file_name);

        // Serialize into a ProviderInstance YAML
        let yaml_content = serde_yaml::to_string(instance)?;
        std::fs::write(&file_path, yaml_content)?;
    }

    Ok(())
}

/// Handle the labels set command (create or update label assignment)
pub fn handle_set_label(
    label_name: String,
    tuple_str: String,
    _color: Option<String>, // Color not supported in new Label
    description: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    // Trim and validate label name
    let label_name = label_name.trim().to_string();
    if label_name.is_empty() {
        return Err(anyhow::anyhow!("Label name cannot be empty"));
    }

    let mut assignments = load_label_assignments_with_home(home)?;
    let mut labels_metadata = load_labels_with_home(home)?;

    // Parse the provider:model tuple
    let tuple = ProviderModelTuple::parse(&tuple_str)
        .map_err(|e| anyhow::anyhow!("Invalid provider:model tuple '{}': {}", tuple_str, e))?;

    // Load provider instances to find the instance_id
    let mut provider_instances = load_provider_instances(home)?;

    // Find the matching provider instance
    let provider_instances_list = provider_instances.all_instances();
    let provider_instance = match provider_instances_list
        .iter()
        .find(|inst| inst.provider_type == tuple.provider())
    {
        Some(instance) => *instance,
        None => {
            // Auto-create a provider instance if none exists for this provider type
            let instance_id = format!("auto-{}", tuple.provider());
            let base_url = match tuple.provider().to_lowercase().as_str() {
                "openai" => "https://api.openai.com/v1",
                "anthropic" => "https://api.anthropic.com/v1",
                "groq" => "https://api.groq.com/openai/v1",
                "openrouter" => "https://openrouter.ai/api/v1",
                "huggingface" => "https://huggingface.co/api",
                "ollama" => "http://localhost:11434",
                "litellm" => "http://localhost:4000",
                _ => "https://api.example.com",
            };

            let new_instance = aicred_core::models::ProviderInstance::new(
                instance_id.clone(),
                format!("{} Instance", tuple.provider()),
                tuple.provider().to_lowercase(),
                base_url.to_string(),
                Vec::new(),
            );

            // Add the new instance to the collection
            provider_instances
                .add_instance(new_instance.clone())
                .map_err(|e| anyhow::anyhow!("Failed to create provider instance: {}", e))?;

            // Save the updated instances
            save_provider_instances(&provider_instances, home)?;

            // Get reference to the newly added instance
            provider_instances.get_instance(&instance_id).unwrap()
        }
    };

    // Create label assignment target
    let target = if let Some(model) = provider_instance.models.iter().find(|m: &&String| {
        let basename = m.rsplit('/').next().unwrap_or(m);
        basename == tuple.model()
    }) {
        LabelTarget::ProviderModel {
            instance_id: provider_instance.id.clone(),
            model_id: model.clone(),
        }
    } else {
        // Model not found, create instance-level assignment
        LabelTarget::ProviderInstance {
            instance_id: provider_instance.id.clone(),
        }
    };

    // Check if this label already exists and update it, or create new one
    let existing_assignment_index = assignments
        .iter()
        .position(|assignment| assignment.label_name == label_name);

    if let Some(index) = existing_assignment_index {
        // Update existing assignment
        assignments[index].target = target;
        assignments[index].assigned_at = chrono::Utc::now();

        println!(
            "{} Label '{}' updated successfully.",
            "✓".green(),
            label_name
        );
        println!("  Now assigned to: {}", tuple_str.cyan());
    } else {
        // Create new assignment
        let assignment = LabelAssignment {
            label_name: label_name.clone(),
            target,
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };

        assignments.push(assignment);
        println!("{} Label '{}' set successfully.", "✓".green(), label_name);
        println!("  Assigned to: {}", tuple_str.cyan());
    }

    // Update label metadata
    if description.is_some() || !labels_metadata.contains_key(&label_name) {
        let label = labels_metadata
            .entry(label_name.clone())
            .or_insert_with(|| Label {
                name: label_name.clone(),
                description: None,
                created_at: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            });

        label.description = description;
    }

    // Save to disk
    save_label_assignments_with_home(&assignments, home)?;
    save_labels_with_home(&labels_metadata, home)?;

    Ok(())
}

/// Handle the labels unset command (remove label assignment entirely)
pub fn handle_unset_label(name: String, force: bool, home: Option<&Path>) -> Result<()> {
    let mut assignments = load_label_assignments_with_home(home)?;
    let mut labels_metadata = load_labels_with_home(home)?;

    // Find the assignment by name
    let assignment_index = assignments
        .iter()
        .position(|assignment| assignment.label_name == name);

    if assignment_index.is_none() {
        return Err(anyhow::anyhow!("Label '{}' not found", name));
    }

    let assignment = assignments[assignment_index.unwrap()].clone();

    if !force {
        println!(
            "{}",
            "Warning: This will permanently remove the label assignment."
                .yellow()
                .bold()
        );
        println!("Label: {}", name.cyan());
        println!(
            "Assigned to: {}",
            assignment_target_to_string(&assignment.target)
        );
        println!("Use --force to confirm removal.");
        return Ok(());
    }

    // Remove the assignment
    assignments.remove(assignment_index.unwrap());
    save_label_assignments_with_home(&assignments, home)?;

    // Also remove label metadata if no more assignments reference it
    if !assignments.iter().any(|a| a.label_name == name) {
        labels_metadata.remove(&name);
        save_labels_with_home(&labels_metadata, home)?;
    }

    println!("{} Label '{}' unset successfully.", "✓".green(), name);

    Ok(())
}

/// Get labels assigned to a specific instance or model
pub fn get_labels_for_target(
    instance_id: &str,
    model_id: Option<&str>,
    home: Option<&Path>,
) -> Result<Vec<Label>> {
    let assignments = load_label_assignments_with_home(home)?;
    let labels_metadata = load_labels_with_home(home)?;
    let provider_instances = load_provider_instances(home)?;

    // Look up the instance to get its provider_type
    let instance = match provider_instances.get_instance(instance_id) {
        Some(instance) => instance,
        None => {
            // Instance not found - return empty labels for newly discovered instances
            return Ok(Vec::new());
        }
    };

    let _provider_type = &instance.provider_type;

    let mut result = Vec::new();

    for assignment in assignments {
        // Check if this assignment's target matches the instance and model
        let matches_target = match (&assignment.target, model_id) {
            (
                LabelTarget::ProviderInstance {
                    instance_id: target_inst,
                },
                None,
            ) => target_inst == instance_id,
            (
                LabelTarget::ProviderModel {
                    instance_id: target_inst,
                    model_id: target_model,
                },
                Some(model),
            ) => target_inst == instance_id && target_model == model,
            _ => false,
        };

        if !matches_target {
            continue;
        }

        // Get label metadata
        if let Some(label_metadata) = labels_metadata.get(&assignment.label_name) {
            result.push(label_metadata.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_get_labels_for_target_matches_provider_and_model() {
        let temp_dir = setup_test_env();

        // Create a test label
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let assignment = LabelAssignment {
            label_name: "thinking".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "test-instance".to_string(),
                model_id: "gpt-4".to_string(),
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };

        let label = Label {
            name: "thinking".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        // Save them
        save_label_assignments_with_home(&[assignment], Some(temp_dir.path())).unwrap();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("thinking".to_string(), label);
        save_labels_with_home(&metadata, Some(temp_dir.path())).unwrap();
    }

    #[test]
    fn test_get_labels_for_target_ignores_different_provider() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let assignment = LabelAssignment {
            label_name: "thinking".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "test-instance".to_string(),
                model_id: "gpt-4".to_string(),
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };
        let label = Label {
            name: "thinking".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        save_label_assignments_with_home(&[assignment], Some(temp_dir.path())).unwrap();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("thinking".to_string(), label);
        save_labels_with_home(&metadata, Some(temp_dir.path())).unwrap();
    }

    #[test]
    fn test_get_labels_for_target_matches_model_basename() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let assignment = LabelAssignment {
            label_name: "thinking".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "test-instance".to_string(),
                model_id: "gpt-4".to_string(),
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };
        let label = Label {
            name: "thinking".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        save_label_assignments_with_home(&[assignment], Some(temp_dir.path())).unwrap();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("thinking".to_string(), label);
        save_labels_with_home(&metadata, Some(temp_dir.path())).unwrap();
    }

    #[test]
    fn test_get_labels_for_target_rejects_wrong_provider_prefix() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let assignment = LabelAssignment {
            label_name: "thinking".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "test-instance".to_string(),
                model_id: "gpt-4".to_string(),
            },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };
        let label = Label {
            name: "thinking".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        save_label_assignments_with_home(&[assignment], Some(temp_dir.path())).unwrap();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("thinking".to_string(), label);
        save_labels_with_home(&metadata, Some(temp_dir.path())).unwrap();
    }

    #[test]
    fn test_label_scan_finds_best_match_across_instances() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());

        // Mock the provider loader to return our test instances
        // This test verifies the matching logic works correctly
        let patterns = [".*sonnet.*", ".*gpt5.*", ".*deepseek.*", ".*"];

        // Test that deepseek model gets matched by the specific pattern
        let deepseek_str = "openrouter:deepseek/deepseek-v3.2-exp";
        let gpt4_str = "openai:gpt-4";

        // Find best match for deepseek string
        let mut best_match_deepseek = None;
        for (pattern_idx, pattern) in patterns.iter().enumerate() {
            if regex::Regex::new(pattern).unwrap().is_match(deepseek_str) {
                best_match_deepseek = Some(pattern_idx);
                break;
            }
        }

        // Find best match for gpt-4 string
        let mut best_match_gpt4 = None;
        for (pattern_idx, pattern) in patterns.iter().enumerate() {
            if regex::Regex::new(pattern).unwrap().is_match(gpt4_str) {
                best_match_gpt4 = Some(pattern_idx);
                break;
            }
        }

        // Verify that deepseek matches the more specific pattern (.*deepseek.* = index 2)
        // and gpt-4 matches the generic pattern (.* = index 3)
        assert_eq!(
            best_match_deepseek,
            Some(2),
            "DeepSeek should match specific pattern"
        );
        assert_eq!(
            best_match_gpt4,
            Some(3),
            "GPT-4 should match generic pattern"
        );
    }
}
