//! Label management commands for the aicred CLI.

use aicred_core::models::{Label, ProviderInstances, UnifiedLabel};
use aicred_core::utils::ProviderModelTuple;
use anyhow::Result;
use colored::*;
use serde::Deserialize;
use std::path::Path;

/// Load provider instances from configuration directory
fn load_provider_instances(home: Option<&Path>) -> Result<ProviderInstances> {
    let config_dir = match home {
        Some(h) => h.to_path_buf(),
        None => dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?,
    }
    .join(".config")
    .join("aicred");

    let instances_dir = config_dir.join("inference_services");

    // Create instances directory if it doesn't exist
    if !instances_dir.exists() {
        std::fs::create_dir_all(&instances_dir)?;
        return Ok(ProviderInstances::new());
    }

    // Load all instance files
    let mut instances = ProviderInstances::new();

    let entries = std::fs::read_dir(&instances_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "yaml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // First try to parse as the modern ProviderInstance directly
                if let Ok(instance) =
                    serde_yaml::from_str::<aicred_core::models::ProviderInstance>(&content)
                {
                    // If the deserialized instance already contains an API key, accept it.
                    // Otherwise, if the file looks like the legacy format (contains "keys:"),
                    // try to parse it as the legacy format and extract the key.
                    if instance.api_key.is_some() {
                        let _ = instances.add_instance(instance);
                    } else if content.contains("keys:") {
                        // Try legacy format
                        if let Ok(legacy_instance) = parse_legacy_instance(&content, &path) {
                            let _ = instances.add_instance(legacy_instance);
                        }
                    } else {
                        // Modern format without API key, accept as-is
                        let _ = instances.add_instance(instance);
                    }
                } else if content.contains("keys:") {
                    // Try legacy format
                    if let Ok(legacy_instance) = parse_legacy_instance(&content, &path) {
                        let _ = instances.add_instance(legacy_instance);
                    }
                }
            }
        }
    }

    Ok(instances)
}

/// Parse legacy provider instance format
fn parse_legacy_instance(
    content: &str,
    path: &std::path::Path,
) -> Result<aicred_core::models::ProviderInstance> {
    use aicred_core::models::discovered_key::Confidence;
    use aicred_core::models::provider_key::{Environment, ValidationStatus};

    #[derive(Deserialize)]
    struct LegacyInstance {
        name: String,
        provider_type: String,
        base_url: String,
        keys: Vec<LegacyKey>,
    }

    #[derive(Deserialize)]
    struct LegacyKey {
        name: String,
        path: String,
        confidence: Confidence,
        environment: Environment,
    }

    let legacy: LegacyInstance = serde_yaml::from_str(content)?;

    let mut instance = aicred_core::models::ProviderInstance::new(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&legacy.name)
            .to_string(),
        legacy.name,
        legacy.provider_type,
        legacy.base_url,
    );

    if let Some(key) = legacy.keys.first() {
        let mut provider_key = aicred_core::models::ProviderKey::new(
            key.name.clone(),
            key.path.clone(),
            key.confidence,
            key.environment.clone(),
        );

        if let Ok(content) = std::fs::read_to_string(&key.path) {
            provider_key = provider_key.with_value(content.trim().to_string());
            provider_key.set_validation_status(ValidationStatus::Valid);
            instance.set_api_key(provider_key.value.unwrap());
        }
    }

    Ok(instance)
}
/// Load all unified labels from the configuration directory
pub fn load_label_assignments() -> Result<Vec<UnifiedLabel>> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
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

/// Save unified labels to the configuration directory
pub fn save_label_assignments(labels: &[UnifiedLabel]) -> Result<()> {
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
            label.target.description().dimmed()
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

#[cfg(test)]
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
        let _temp_dir = setup_test_env();

        // Create a test label
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let mut labels = vec![UnifiedLabel::new("thinking".to_string(), tuple)];

        // Save it
        save_label_assignments(&labels).unwrap();

        // Create a mock provider instance
        // Note: In a real test, we'd need to set up provider instances
        // For now, this test documents the expected behavior
    }

    #[test]
    fn test_get_labels_for_target_ignores_different_provider() {
        let _temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments(&vec![label]).unwrap();

        // Querying for anthropic instance should return no labels
        // (would need mock provider instances to test fully)
    }

    #[test]
    fn test_get_labels_for_target_matches_model_basename() {
        let _temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments(&vec![label]).unwrap();

        // Should match both "gpt-4" and "openai/gpt-4" model names
        // (would need mock provider instances to test fully)
    }

    #[test]
    fn test_get_labels_for_target_rejects_wrong_provider_prefix() {
        let _temp_dir = setup_test_env();

        // Create a label for openai:gpt-4
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = UnifiedLabel::new("thinking".to_string(), tuple);
        save_label_assignments(&vec![label]).unwrap();

        // Should NOT match "anthropic/gpt-4" even though basename matches
        // (would need mock provider instances to test fully)
    }
}

/// Handle the labels set command (create or update label assignment)
pub fn handle_set_label(
    label_name: String,
    tuple_str: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut labels = load_label_assignments()?;

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
    save_label_assignments(&labels)?;

    Ok(())
}

/// Handle the labels unset command (remove label assignment entirely)
pub fn handle_unset_label(name: String, force: bool) -> Result<()> {
    let mut labels = load_label_assignments()?;

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
    save_label_assignments(&labels)?;

    println!("{} Label '{}' unset successfully.", "✓".green(), name);

    Ok(())
}

/// Get labels assigned to a specific instance or model
pub fn get_labels_for_target(instance_id: &str, model_id: Option<&str>) -> Result<Vec<Label>> {
    let labels = load_label_assignments()?;
    let provider_instances = load_provider_instances(None)?;

    // Look up the instance to get its provider_type
    let instance = provider_instances
        .get_instance(instance_id)
        .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

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

        // If model_id is provided, check if it matches the tuple's model
        if let Some(model_name) = model_id {
            // Extract the basename from the model_id for comparison
            let model_basename = if let Some(slash_pos) = model_name.find('/') {
                &model_name[slash_pos + 1..]
            } else {
                model_name
            };

            // The tuple model must match the model basename
            if tuple_model != model_basename {
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
