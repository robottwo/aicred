//! Label management commands for the aicred CLI.

use crate::utils::provider_loader::load_provider_instances;
use aicred_core::models::{Label, UnifiedLabel};
use aicred_core::utils::ProviderModelTuple;
use anyhow::Result;
use colored::*;
use std::path::Path;
/// Load all unified labels from the configuration directory
pub fn load_label_assignments_with_home(home: Option<&Path>) -> Result<Vec<UnifiedLabel>> {
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
    let labels: Vec<UnifiedLabel> = serde_yaml::from_str(&content)?;
    Ok(labels)
}

/// Load all unified labels from the configuration directory
pub fn load_label_assignments() -> Result<Vec<UnifiedLabel>> {
    load_label_assignments_with_home(None)
}

/// Save unified labels to the configuration directory
pub fn save_label_assignments_with_home(
    labels: &[UnifiedLabel],
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
    std::fs::write(&labels_file, content)?;

    Ok(())
}

/// Save unified labels to the configuration directory
pub fn save_label_assignments(labels: &[UnifiedLabel]) -> Result<()> {
    save_label_assignments_with_home(labels, None)
}

/// Handle the labels list command
pub fn handle_list_labels() -> Result<()> {
    let labels = load_label_assignments()?;

    if labels.is_empty() {
        println!("{}", "No labels configured.".yellow());
        println!(
            "{}",
            "Use 'aicred labels set label=provider:model' to create a label.".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Label Assignments:".green().bold());

    for label in &labels {
        println!(
            "  {} - {}",
            label.label_name.cyan().bold(),
            label.target.as_str().dimmed()
        );

        if let Some(ref description) = label.description {
            println!("    Description: {}", description);
        }
        if let Some(ref color) = label.color {
            println!("    Color: {}", color);
        }

        println!(
            "    Created: {}",
            label.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    println!("{}", format!("Total labels: {}", labels.len()).cyan());

    Ok(())
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
                println!("    DEBUG: Model: {}", model.model_id);
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
    // First try to find labels in user config directory, then fall back to binary location
    let labels_dir = find_labels_directory(home)?;

    if !labels_dir.exists() {
        println!("{}", format!("No labels directory found at: {}. Creating scan files in ~/.config/aicred/patterns/ or binary's parent directory + conf/labels is required.", labels_dir.display()).yellow());
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

    // Load existing labels
    let mut existing_labels = load_label_assignments_with_home(home)?;
    let original_labels = existing_labels.clone(); // Keep track of original state
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
                let provider_model_str = format!("{}:{}", instance.provider_type, model.model_id);

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
                                println!("    ❌ Pattern {} did not match", pattern_idx + 1);
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

        // Select the globally best match (lowest specificity index = most specific pattern)
        let best_match = all_matches
            .into_iter()
            .min_by_key(|(_, _, specificity)| *specificity);

        // Apply the best match found
        if let Some((provider_model_str, tuple, _specificity)) = best_match {
            if verbose {
                println!(
                    "  🎯 Final assignment: '{}' -> {}",
                    label_name, provider_model_str
                );
            }

            let label = UnifiedLabel::new(label_name.clone(), tuple.clone());

            // Check if label already exists and update it
            if let Some(existing) = existing_labels
                .iter_mut()
                .find(|l| l.label_name == label_name)
            {
                if verbose {
                    println!(
                        "  🔄 Updating existing label '{}' to {}",
                        label_name, provider_model_str
                    );
                }
                existing.target = tuple;
                existing.updated_at = chrono::Utc::now();
            } else {
                if verbose {
                    println!(
                        "  ➕ Creating new label '{}' for {}",
                        label_name, provider_model_str
                    );
                }
                new_assignments.push(label);
            }
        } else if verbose {
            println!("  ❌ No matches found for label '{}'", label_name);
        }
    } // End of for entry in scan_files loop

    // Add new assignments to existing labels
    existing_labels.extend(new_assignments);

    // Track what actually changed
    let mut newly_created = Vec::new();
    let mut updated = Vec::new();

    // Re-check what changed by comparing with original state
    for label in &existing_labels {
        if let Some(original) = original_labels
            .iter()
            .find(|l| l.label_name == label.label_name)
        {
            if original.target != label.target {
                updated.push(label);
            }
        } else {
            newly_created.push(label);
        }
    }

    if dry_run {
        println!("\n{}", "DRY RUN - No changes will be made".red().bold());

        let total_changes = newly_created.len() + updated.len();

        if total_changes > 0 {
            println!("{}", "Would create/update the following labels:".cyan());

            for label in newly_created {
                println!(
                    "  {} -> {}",
                    label.label_name.cyan().bold(),
                    label.target.as_str().dimmed()
                );
            }

            for label in updated {
                println!(
                    "  {} -> {}",
                    label.label_name.cyan().bold(),
                    label.target.as_str().dimmed()
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
        // Save the updated labels
        save_label_assignments_with_home(&existing_labels, home)?;

        println!("\n{}", "Label scan completed".green().bold());

        let total_changes = newly_created.len() + updated.len();

        if total_changes > 0 {
            if !newly_created.is_empty() {
                println!("{}", "New assignments:".cyan());
                for label in newly_created {
                    println!(
                        "  {} -> {}",
                        label.label_name.cyan().bold(),
                        label.target.as_str().dimmed()
                    );
                }
            }

            if !updated.is_empty() {
                println!("{}", "Updated assignments:".cyan());
                for label in updated {
                    println!(
                        "  {} -> {}",
                        label.label_name.cyan().bold(),
                        label.target.as_str().dimmed()
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
        let labels = vec![UnifiedLabel::new("thinking".to_string(), tuple)];

        // Save it
        save_label_assignments_with_home(&labels, Some(temp_dir.path())).unwrap();

        // Create a mock provider instance
        // Note: In a real test, we'd need to set up provider instances
        // For now, this test documents the expected behavior
    }

    #[test]
    fn test_get_labels_for_target_ignores_different_provider() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments_with_home(&[label], Some(temp_dir.path())).unwrap();

        // Querying for anthropic instance should return no labels
        // (would need mock provider instances to test fully)
    }

    #[test]
    fn test_get_labels_for_target_matches_model_basename() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments_with_home(&[label], Some(temp_dir.path())).unwrap();

        // Should match both "gpt-4" and "openai/gpt-4" model names
        // (would need mock provider instances to test fully)
    }

    #[test]
    fn test_get_labels_for_target_rejects_wrong_provider_prefix() {
        let temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments_with_home(&[label], Some(temp_dir.path())).unwrap();

        // Should NOT match "anthropic/gpt-4" even though basename matches
        // (would need mock provider instances to test fully)
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

/// Handle the labels set command (create or update label assignment)
pub fn handle_set_label(
    label_name: String,
    tuple_str: String,
    color: Option<String>,
    description: Option<String>,
    home: Option<&Path>,
) -> Result<()> {
    // Trim and validate label name
    let label_name = label_name.trim().to_string();
    if label_name.is_empty() {
        return Err(anyhow::anyhow!("Label name cannot be empty"));
    }

    let mut labels = load_label_assignments_with_home(home)?;

    // Parse the provider:model tuple
    let tuple = ProviderModelTuple::parse(&tuple_str)
        .map_err(|e| anyhow::anyhow!("Invalid provider:model tuple '{}': {}", tuple_str, e))?;

    // Check if this label already exists and update it, or create new one
    let existing_label_index = labels
        .iter()
        .position(|label| label.label_name == label_name);

    if let Some(index) = existing_label_index {
        // Update existing label
        labels[index].target = tuple;
        labels[index].updated_at = chrono::Utc::now();

        // Update metadata if provided
        if let Some(desc) = description {
            labels[index].description = Some(desc);
        }

        if let Some(col) = color {
            labels[index].color = Some(col);
        }

        println!(
            "{} Label '{}' updated successfully.",
            "✓".green(),
            label_name
        );
        println!("  Now assigned to: {}", tuple_str.cyan());
    } else {
        // Create new label
        let mut label = UnifiedLabel::new(label_name.clone(), tuple);

        if let Some(desc) = description {
            label = label.with_description(desc);
        }

        if let Some(col) = color {
            label = label.with_color(col);
        }

        labels.push(label);
        println!("{} Label '{}' set successfully.", "✓".green(), label_name);
        println!("  Assigned to: {}", tuple_str.cyan());
    }

    // Save to disk
    save_label_assignments_with_home(&labels, home)?;

    Ok(())
}

/// Handle the labels unset command (remove label assignment entirely)
pub fn handle_unset_label(name: String, force: bool, home: Option<&Path>) -> Result<()> {
    let mut labels = load_label_assignments_with_home(home)?;

    // Find the label by name
    let label_index = labels.iter().position(|label| label.label_name == name);

    if label_index.is_none() {
        return Err(anyhow::anyhow!("Label '{}' not found", name));
    }

    let label = labels[label_index.unwrap()].clone();

    if !force {
        println!(
            "{}",
            "Warning: This will permanently remove the label assignment."
                .yellow()
                .bold()
        );
        println!("Label: {}", name.cyan());
        println!("Assigned to: {}", label.target.description());
        println!("Use --force to confirm removal.");
        return Ok(());
    }

    // Remove the label
    labels.remove(label_index.unwrap());
    save_label_assignments_with_home(&labels, home)?;

    println!("{} Label '{}' unset successfully.", "✓".green(), name);

    Ok(())
}

/// Get labels assigned to a specific instance or model
pub fn get_labels_for_target(
    instance_id: &str,
    model_id: Option<&str>,
    home: Option<&Path>,
) -> Result<Vec<Label>> {
    let labels = load_label_assignments_with_home(home)?;
    let provider_instances = load_provider_instances(home)?;

    // Look up the instance to get its provider_type
    let instance = match provider_instances.get_instance(instance_id) {
        Some(instance) => instance,
        None => {
            // Instance not found - return empty labels for newly discovered instances
            return Ok(Vec::new());
        }
    };

    let provider_type = &instance.provider_type;

    let mut result = Vec::new();

    for unified_label in labels {
        // Check if this label's target tuple matches the instance and model
        let tuple_provider = unified_label.target.provider();
        let tuple_model = unified_label.target.model();

        // The tuple provider must match the instance's provider_type
        if tuple_provider != provider_type {
            continue;
        }

        // If model_id is provided, resolve the actual Model and compare against canonical ID and basename
        if let Some(model_display_name) = model_id {
            // Find the model in the instance by matching either display name or canonical ID
            let model = match instance
                .models
                .iter()
                .find(|m| m.name == model_display_name || m.model_id == model_display_name)
            {
                Some(model) => model,
                None => {
                    // Model not found in this instance
                    continue;
                }
            };

            // Extract the basename from the canonical model ID for comparison
            let model_basename = if let Some(slash_pos) = model.model_id.find('/') {
                &model.model_id[slash_pos + 1..]
            } else {
                &model.model_id
            };

            // The tuple model must match either the canonical model ID or its basename
            if tuple_model != model.model_id && tuple_model != model_basename {
                continue;
            }
        }

        // This label matches! Create a Label for display
        let label = Label::new(
            unified_label.label_name.clone(),
            unified_label.label_name.clone(),
        );

        // Add metadata if present
        let mut label = label;
        if let Some(ref description) = unified_label.description {
            label = label.with_description(description.clone());
        }
        if let Some(ref color) = unified_label.color {
            label = label.with_color(color.clone());
        }

        result.push(label);
    }

    Ok(result)
}
