use crate::utils::provider_loader::load_provider_instances;
use aicred_core::models::{ProviderCollection, ProviderInstance};
use anyhow::Result;
use colored::*;
use std::path::PathBuf;

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
    format!("{}...", truncated)
}

/// Save provider instances to configuration directory
fn save_provider_instances(instances: &ProviderCollection) -> Result<()> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred")
        .join("inference_services");

    std::fs::create_dir_all(&config_dir)?;

    // Save each instance to its own file
    for instance in instances.all_instances() {
        // Use provider name and first 4 chars of instance ID (hash)
        let file_name = format!("{}-{}.yaml", instance.provider_type, &instance.id[..4]);
        let file_path = config_dir.join(&file_name);

        // Serialize into a ProviderInstance YAML
        let yaml_content = serde_yaml::to_string(instance)?;
        std::fs::write(&file_path, yaml_content)?;
    }

    Ok(())
}

/// Handle the list-instances command
pub fn handle_list_instances(
    home: Option<PathBuf>,
    verbose: bool,
    provider_type: Option<String>,
    active_only: bool,
    tag: Option<String>,
    label: Option<String>,
) -> Result<()> {
    let instances = load_provider_instances(home.as_deref())?;

    if instances.is_empty() {
        println!("{}", "No provider instances configured.".yellow());
        println!(
            "{}",
            "Use 'aicred instances add' to create a new instance.".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Configured Provider Instances:".green().bold());

    let all_instances = instances.all_instances();
    let filtered_instances: Vec<&ProviderInstance> = all_instances
        .into_iter()
        .filter(|instance| {
            let type_match = provider_type
                .as_ref()
                .is_none_or(|pt| instance.provider_type == *pt);
            let active_match = !active_only || instance.active;

            // Tag filtering
            let tag_match =
                tag.as_ref().is_none_or(
                    |tag_name| match crate::commands::tags::get_tags_for_target(
                        &instance.id,
                        None,
                        home.as_deref(),
                    ) {
                        Ok(tags) => tags.iter().any(|t| t.name == *tag_name),
                        Err(_) => false,
                    },
                );

            // Label filtering
            let label_match = label.as_ref().is_none_or(|label_name| {
                match crate::commands::labels::get_labels_for_target(
                    &instance.id,
                    None,
                    home.as_deref(),
                ) {
                    Ok(labels) => labels.iter().any(|l| l.name == *label_name),
                    Err(_) => false,
                }
            });

            type_match && active_match && tag_match && label_match
        })
        .collect();

    if filtered_instances.is_empty() {
        println!("{}", "No instances match the specified criteria.".yellow());
        return Ok(());
    }

    let total_count = filtered_instances.len();

    if verbose {
        // Verbose mode: show detailed information for each instance
        for instance in filtered_instances {
            println!("\n{}", instance.id.cyan().bold());
            println!("  Provider Type: {}", instance.provider_type);
            println!("  Base URL: {}", instance.base_url);
            println!(
                "  Status: {}",
                if instance.active {
                    "Active".green()
                } else {
                    "Inactive".red()
                }
            );
            println!(
                "  Keys: {} total, {} valid",
                if instance.has_api_key() { 1 } else { 0 },
                if instance.has_non_empty_api_key() {
                    1
                } else {
                    0
                }
            );
            println!("  Models: {} configured", instance.model_count());

            if !instance.models.is_empty() {
                let model_names: Vec<String> = instance.models.clone();
                println!("  Available Models: {}", model_names.join(", "));
            }

            if !instance.metadata.is_empty() {
                for (key, value) in &instance.metadata {
                    println!("  {}: {}", key, value);
                }
            }

            // Note: created_at and updated_at not available in new ProviderInstance
            println!("  Created: N/A");
            println!("  Updated: N/A");
        }
    } else {
        // Table mode: show instances in a nicely formatted table
        println!(
            "{:<20} {:<15} {:<15}",
            "ID".bold(),
            "Provider".bold(),
            "Num of Models".bold()
        );
        println!("{}", "-".repeat(55));

        for instance in filtered_instances {
            println!(
                "{:<20} {:<15} {:<15}",
                instance.id.cyan(),
                instance.provider_type.yellow(),
                instance.model_count()
            );
        }
    }

    println!("\n{}", format!("Total instances: {}", total_count).cyan());

    Ok(())
}

/// Handle the add-instance command
pub fn handle_add_instance(
    id: String,
    _name: String, // display_name not supported in new ProviderInstance
    provider_type: String,
    base_url: String,
    api_key: Option<String>,
    models: Option<String>,
    active: bool,
) -> Result<()> {
    let mut instances = load_provider_instances(None)?;

    // Check if instance with this ID already exists
    if instances.get_instance(&id).is_some() {
        return Err(anyhow::anyhow!(
            "Provider instance with ID '{}' already exists",
            id
        ));
    }

    let mut instance = ProviderInstance::new(
        id.clone(),
        provider_type,
        base_url,
        String::new(),
        Vec::new(),
    );
    instance.active = active;

    // Add API key if provided
    if let Some(key_value) = api_key {
        instance.set_api_key(key_value);
    }

    // Add models if provided
    if let Some(models_str) = models {
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                instance.add_model(model_id);
            }
        }
    }

    // Validate the instance
    if let Err(e) = instance.validate() {
        return Err(anyhow::anyhow!("Invalid instance configuration: {}", e));
    }

    // Add to collection
    instances
        .add_instance(instance.clone())
        .map_err(|e| anyhow::anyhow!(e))?;

    // Save to disk - create a copy to avoid borrow issues
    let instances_copy = instances.clone();
    save_provider_instances(&instances_copy)?;

    println!(
        "{} Provider instance '{}' added successfully.",
        "✓".green(),
        instance.id.cyan()
    );
    println!("  ID: {}", instance.id);
    println!("  Type: {}", instance.provider_type);
    println!(
        "  Status: {}",
        if instance.active {
            "Active"
        } else {
            "Inactive"
        }
    );

    Ok(())
}

/// Handle the remove-instance command
pub fn handle_remove_instance(id: String, force: bool) -> Result<()> {
    let mut instances = load_provider_instances(None)?;

    // Check if instance exists
    if instances.get_instance(&id).is_none() {
        return Err(anyhow::anyhow!(
            "Provider instance with ID '{}' not found",
            id
        ));
    }

    // Get instance for confirmation
    let instance = instances.get_instance(&id).unwrap();

    if !force {
        println!(
            "{}",
            "Warning: This will permanently remove the provider instance."
                .yellow()
                .bold()
        );
        println!("Instance: {}", instance.id.cyan());
        print!("Are you sure? (y/N): ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Removal cancelled.".dimmed());
            return Ok(());
        }
    }

    // Remove the instance
    instances.remove_instance(&id);

    // Save to disk - create a copy to avoid borrow issues
    let instances_copy = instances.clone();
    save_provider_instances(&instances_copy)?;

    println!(
        "{} Provider instance '{}' removed successfully.",
        "✓".green(),
        id.cyan()
    );

    Ok(())
}

/// Handle the update-instance command
pub fn handle_update_instance(
    id: String,
    _name: Option<String>, // display_name not supported in new ProviderInstance
    base_url: Option<String>,
    api_key: Option<String>,
    models: Option<String>,
    active: Option<bool>,
) -> Result<()> {
    let mut instances = load_provider_instances(None)?;

    // Get mutable reference to the instance
    let instance = instances
        .get_instance_mut(&id)
        .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", id))?;

    // Store original values for later use
    let instance_id = instance.id.clone();

    // Update fields if provided
    if let Some(new_base_url) = base_url {
        instance.base_url = new_base_url;
    }

    if let Some(new_active) = active {
        instance.active = new_active;
    }

    // Update API key if provided
    if let Some(new_key_value) = api_key {
        instance.set_api_key(new_key_value);
    }

    // Update models if provided
    if let Some(models_str) = models {
        instance.models.clear();
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                instance.add_model(model_id);
            }
        }
    }

    // Validate the updated instance
    if let Err(e) = instance.validate() {
        return Err(anyhow::anyhow!("Invalid instance configuration: {}", e));
    }

    // Get the final active status before saving
    let final_active_status = instance.active;

    // Save to disk
    save_provider_instances(&instances)?;

    println!(
        "{} Provider instance '{}' updated successfully.",
        "✓".green(),
        instance_id.cyan()
    );
    println!(
        "  Status: {}",
        if final_active_status {
            "Active"
        } else {
            "Inactive"
        }
    );

    Ok(())
}

/// Handle the get-instance command
pub fn handle_get_instance(home: Option<PathBuf>, id: String, include_values: bool) -> Result<()> {
    let instances = load_provider_instances(home.as_deref())?;

    // Add debug logging to validate the home parameter issue
    if home.is_some() {
        tracing::debug!(
            "handle_list_models called with custom home directory: {:?}",
            home
        );
    } else {
        tracing::debug!("handle_list_models called with default home directory (None)");
    }

    let instance = instances
        .get_instance(&id)
        .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", id))?;

    println!("\n{}", instance.id.cyan().bold());
    println!("{}", "─".repeat(50).dimmed());

    println!("Provider Type: {}", instance.provider_type.yellow());
    println!("Base URL: {}", instance.base_url);
    println!(
        "Status: {}",
        if instance.active {
            "Active".green()
        } else {
            "Inactive".red()
        }
    );
    // Note: created_at and updated_at not available in new ProviderInstance
    println!("Created: N/A");
    println!("Updated: N/A");

    // Show keys
    println!("\n{}", "API Keys:".green().bold());
    if let Some(api_key) = instance.get_api_key() {
        if include_values {
            println!("  Value: {}", api_key.red());
        } else {
            println!("  Value: {}", "********".dimmed());
        }
        println!("  Status: Unknown");
        println!(
            "  Discovered: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        println!("  {}", "No keys configured".dimmed());
    }

    // Show models
    println!("{}", "Models:".green().bold());
    if instance.models.is_empty() {
        println!("  {}", "No models configured".dimmed());
    } else {
        for model in &instance.models {
            println!("  {}", model.cyan());
        }
        println!();
    }

    // Show metadata
    if !instance.metadata.is_empty() {
        println!("{}", "Metadata:".green().bold());
        for (key, value) in &instance.metadata {
            println!("  {}: {}", key.cyan(), value);
        }
    }

    Ok(())
}

/// Handle the validate-instances command
pub fn handle_validate_instances(id: Option<String>, all_errors: bool) -> Result<()> {
    let instances = load_provider_instances(None)?;

    if instances.is_empty() {
        println!("{}", "No provider instances configured.".yellow());
        return Ok(());
    }

    if let Some(instance_id) = id {
        // Validate specific instance
        let instance = instances.get_instance(&instance_id).ok_or_else(|| {
            anyhow::anyhow!("Provider instance with ID '{}' not found", instance_id)
        })?;

        match instance.validate() {
            Ok(()) => {
                println!(
                    "{} Instance '{}' is valid.",
                    "✓".green(),
                    instance.id.cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Instance '{}' has validation errors:",
                    "✗".red(),
                    instance.id.cyan()
                );
                println!("  {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Validate all instances
        let mut all_valid = true;
        let mut errors = Vec::new();
        for instance in instances.list() {
            if let Err(e) = instance.validate() {
                all_valid = false;
                errors.push(format!("Instance '{}': {}", instance.id, e));
            }
        }

        if all_valid {
            println!(
                "{} All {} provider instances are valid.",
                "✓".green(),
                instances.len()
            );
        } else {
            println!("{} Validation errors found:", "✗".red());
            for error in errors {
                println!("  {}", error);
                if !all_errors {
                    break;
                }
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Original handle_providers function for backward compatibility
pub fn handle_providers(verbose: bool) -> Result<()> {
    println!("\n{}", "Available Providers:".green().bold());

    let providers = vec![
        ("openai", "OpenAI API keys"),
        ("anthropic", "Anthropic (Claude) API keys"),
        ("huggingface", "Hugging Face tokens"),
        ("ollama", "Ollama local configurations"),
        ("litellm", "LiteLLM configurations"),
        ("groq", "Groq API keys"),
    ];

    for (name, desc) in providers {
        if verbose {
            println!("  {} - {}", name.cyan(), desc);
        } else {
            println!("  {}", name.cyan());
        }
    }

    println!("\n{}", "Available Application Scanners:".green().bold());

    let scanners = vec![
        ("roo-code", "Roo Code VSCode extension"),
        ("claude-desktop", "Claude Desktop application"),
        ("ragit", "Ragit configurations"),
        ("langchain", "LangChain application configs"),
        ("gsh", "GSH configurations"),
    ];

    for (name, desc) in scanners {
        if verbose {
            println!("  {} - {}", name.cyan(), desc);
        } else {
            println!("  {}", name.cyan());
        }
    }

    println!("\n{}", "Provider Instance Management:".green().bold());
    println!("  Use 'aicred instances --help' for instance management commands");

    Ok(())
}

/// Handle the list-models command
pub fn handle_list_models(
    home: Option<PathBuf>,
    verbose: bool,
    provider_type: Option<String>,
    tag: Option<String>,
    label: Option<String>,
) -> Result<()> {
    let instances = load_provider_instances(home.as_deref())?;

    if instances.is_empty() {
        println!("{}", "No provider instances configured.".yellow());
        println!(
            "{}",
            "Use 'aicred instances add' to create a new instance.".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Configured Models:".green().bold());

    // Collect all models from all instances
    let mut all_models: Vec<(&ProviderInstance, &String)> = Vec::new();
    for instance in instances.all_instances() {
        for model in &instance.models {
            all_models.push((instance, model));
        }
    }

    if all_models.is_empty() {
        println!("{}", "No models configured.".yellow());
        return Ok(());
    }

    // Filter models
    let mut filtered_models: Vec<(&ProviderInstance, &String)> = all_models
        .into_iter()
        .filter(|(instance, model)| {
            let type_match = provider_type
                .as_ref()
                .is_none_or(|pt| instance.provider_type == *pt);

            // Tag filtering
            let tag_match =
                tag.as_ref().is_none_or(
                    |tag_name| match crate::commands::tags::get_tags_for_target(
                        &instance.id,
                        Some(model.as_str()),
                        home.as_deref(),
                    ) {
                        Ok(tags) => tags.iter().any(|t| t.name == *tag_name),
                        Err(_) => false,
                    },
                );

            // Label filtering
            let label_match = label.as_ref().is_none_or(|label_name| {
                match crate::commands::labels::get_labels_for_target(
                    &instance.id,
                    Some(model.as_str()),
                    home.as_deref(),
                ) {
                    Ok(labels) => labels.iter().any(|l| l.name == *label_name),
                    Err(_) => false,
                }
            });

            type_match && tag_match && label_match
        })
        .collect();

    if filtered_models.is_empty() {
        println!("{}", "No models match the specified criteria.".yellow());
        return Ok(());
    }

    let total_count = filtered_models.len();

    if verbose {
        println!("Found {} model(s):\n", total_count);

        for (instance, model_id) in filtered_models {
            println!("{} ({})", model_id.cyan(), instance.provider_type);
            println!("  Instance: {} ({})", instance.id, instance.id);

            // Show tags
            if let Ok(tags) = crate::commands::tags::get_tags_for_target(
                &instance.id,
                Some(model_id.as_str()),
                home.as_deref(),
            ) {
                if !tags.is_empty() {
                    println!("  Tags:");
                    for tag in tags {
                        println!("    - {}", tag.name);
                    }
                }
            }

            // Show labels
            if let Ok(labels) = crate::commands::labels::get_labels_for_target(
                &instance.id,
                Some(model_id.as_str()),
                home.as_deref(),
            ) {
                if !labels.is_empty() {
                    println!("  Labels:");
                    for label in labels {
                        println!("    - {}", label.name);
                    }
                }
            }

            println!();
        }
    } else {
        // Sort models by provider type, then by model id
        filtered_models.sort_by(|(inst_a, model_a), (inst_b, model_b)| {
            inst_a
                .provider_type
                .cmp(&inst_b.provider_type)
                .then_with(|| model_a.cmp(model_b))
        });

        // Table mode: show models in a nicely formatted table
        println!(
            "{:<25} {:<20} {:<35} {:<15} {:<15}",
            "Basename".bold(),
            "Provider".bold(),
            "Model".bold(),
            "Labels".bold(),
            "Tags".bold()
        );
        println!("{}", "-".repeat(105));

        for (instance, model_id) in filtered_models {
            // Extract basename from model_id (everything after the last slash)
            let basename = if let Some(last_slash_pos) = model_id.rfind('/') {
                &model_id[last_slash_pos + 1..]
            } else {
                model_id
            };

            // Get labels and tags for this model
            let labels = match crate::commands::labels::get_labels_for_target(
                &instance.id,
                Some(model_id.as_str()),
                home.as_deref(),
            ) {
                Ok(labels) => labels
                    .iter()
                    .map(|l| l.name.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                Err(_) => String::new(),
            };

            let tags = match crate::commands::tags::get_tags_for_target(
                &instance.id,
                Some(model_id.as_str()),
                home.as_deref(),
            ) {
                Ok(tags) => tags
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                Err(_) => String::new(),
            };

            println!(
                "{:<25} {:<20} {:<35} {:<15} {:<15}",
                basename.cyan(),
                format!("{} ({})", instance.provider_type, instance.id).yellow(),
                truncate_string(model_id, 35),
                if labels.is_empty() {
                    "-".dimmed()
                } else {
                    labels.dimmed()
                },
                if tags.is_empty() {
                    "-".dimmed()
                } else {
                    tags.dimmed()
                }
            );
        }
    }

    Ok(())
}
