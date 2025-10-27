use anyhow::Result;
use colored::*;
use std::path::PathBuf;
use genai_keyfinder_core::models::{ProviderInstance, ProviderInstances, ProviderKey, Model, Environment, Confidence, ValidationStatus};

/// Load provider instances from configuration directory
fn load_provider_instances() -> Result<ProviderInstances> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred");

    let instances_dir = config_dir.join("instances");
    let providers_dir = config_dir.join("providers");
    let old_config_path = config_dir.join("providers.yaml");

    // Check if we need to migrate from old format
    if old_config_path.exists() && !instances_dir.exists() {
        println!("{}", "Migrating from old single-file format to new instance-based format...".yellow());
        migrate_from_old_format(&old_config_path, &instances_dir)?;
    }

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
                match serde_yaml::from_str::<ProviderInstance>(&content) {
                    Ok(instance) => {
                        let _ = instances.add_instance(instance);
                    }
                    Err(e) => {
                        eprintln!("{} {}: {}", "Error parsing instance file:".red(), path.display(), e);
                    }
                }
            }
        }
    }

    // If no instances found, try to load from providers directory as fallback
    if instances.is_empty() && providers_dir.exists() {
        println!("{}", "No instances found, loading from legacy provider configurations...".yellow());
        load_instances_from_providers(&providers_dir, &mut instances)?;
    }

    Ok(instances)
}

/// Migrate from old format to new instance-based format
fn migrate_from_old_format(old_path: &PathBuf, instances_dir: &PathBuf) -> Result<()> {
    println!("{}", "Backing up old configuration file...".yellow());
    
    // Create backup
    let backup_path = old_path.with_extension("yaml.backup");
    std::fs::copy(old_path, &backup_path)?;

    // Create instances directory
    std::fs::create_dir_all(instances_dir)?;

    // Load old format
    let content = std::fs::read_to_string(old_path)?;
    let old_config: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Extract providers from old format and convert to instances
    if let Some(providers) = old_config.get("providers").and_then(|p| p.as_mapping()) {
        for (provider_name, provider_data) in providers {
            if let Some(name) = provider_name.as_str() {
                let instance_id = name.to_lowercase().replace(' ', "_");
                let display_name = name.to_string();
                
                // Determine provider type and base URL from known patterns
                let (provider_type, base_url) = match name.to_lowercase().as_str() {
                    "openai" => ("openai", "https://api.openai.com"),
                    "anthropic" => ("anthropic", "https://api.anthropic.com"),
                    "huggingface" => ("huggingface", "https://huggingface.co"),
                    "ollama" => ("ollama", "http://localhost:11434"),
                    "groq" => ("groq", "https://api.groq.com"),
                    _ => ("unknown", "https://api.example.com"),
                };

                let mut instance = ProviderInstance::new(
                    instance_id.clone(),
                    display_name,
                    provider_type.to_string(),
                    base_url.to_string(),
                );

                // Extract API key if available
                if let Some(api_key) = provider_data.get("api_key").and_then(|v| v.as_str()) {
                    let mut key = ProviderKey::new(
                        "default".to_string(),
                        "migration".to_string(),
                        Confidence::High,
                        Environment::Production,
                    );
                    key.value = Some(api_key.to_string());
                    key.discovered_at = chrono::Utc::now();
                    key.validation_status = ValidationStatus::Unknown;
                    instance.add_key(key);
                }

                // Extract models if available
                if let Some(models) = provider_data.get("models").and_then(|v| v.as_sequence()) {
                    for model_str in models.iter().filter_map(|m| m.as_str()) {
                        let model = Model::new(
                            model_str.to_string(),
                            instance_id.clone(),
                            model_str.to_string(),
                        );
                        instance.add_model(model);
                    }
                }

                // Save instance to file
                let instance_file_name = format!("{}.yaml", instance_id);
                let instance_file_path = instances_dir.join(&instance_file_name);
                let yaml_content = serde_yaml::to_string(&instance)?;
                std::fs::write(&instance_file_path, yaml_content)?;
            }
        }

        // Remove old file
        std::fs::remove_file(old_path)?;

        println!("{}", "Migration completed successfully!".green());
    }

    Ok(())
}

/// Load instances from legacy provider configurations
fn load_instances_from_providers(providers_dir: &PathBuf, instances: &mut ProviderInstances) -> Result<()> {
    use genai_keyfinder_core::models::ProviderConfig;
    
    if !providers_dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(providers_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "yaml") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = ProviderConfig::from_yaml(&content) {
                        // Convert ProviderConfig to ProviderInstance
                        let instance_id = file_stem.to_string();
                        let display_name = file_stem
                            .split('_')
                            .map(|word| {
                                let mut chars = word.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(" ");

                        let (provider_type, base_url) = match file_stem.to_lowercase().as_str() {
                            s if s.contains("openai") => ("openai", "https://api.openai.com"),
                            s if s.contains("anthropic") => ("anthropic", "https://api.anthropic.com"),
                            s if s.contains("huggingface") => ("huggingface", "https://huggingface.co"),
                            s if s.contains("ollama") => ("ollama", "http://localhost:11434"),
                            s if s.contains("groq") => ("groq", "https://api.groq.com"),
                            _ => ("unknown", "https://api.example.com"),
                        };

                        let mut instance = ProviderInstance::new(
                            instance_id,
                            display_name,
                            provider_type.to_string(),
                            base_url.to_string(),
                        );

                        // Migrate keys
                        instance.keys = config.keys;

                        // Convert model strings to Model objects
                        for model_id in &config.models {
                            let model = Model::new(
                                model_id.clone(),
                                instance.id.clone(),
                                model_id.clone(),
                            );
                            instance.add_model(model);
                        }

                        let _ = instances.add_instance(instance);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Save provider instances to configuration directory
fn save_provider_instances(instances: &ProviderInstances) -> Result<()> {
    let config_dir = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".config")
        .join("aicred")
        .join("instances");

    std::fs::create_dir_all(&config_dir)?;

    // Save each instance to its own file
    for instance in instances.all_instances() {
        let file_name = format!("{}.yaml", instance.id);
        let file_path = config_dir.join(&file_name);
        let yaml_content = serde_yaml::to_string(instance)?;
        std::fs::write(&file_path, yaml_content)?;
    }

    Ok(())
}

/// Handle the list-instances command
pub fn handle_list_instances(verbose: bool, provider_type: Option<String>, active_only: bool) -> Result<()> {
    let instances = load_provider_instances()?;

    if instances.is_empty() {
        println!("{}", "No provider instances configured.".yellow());
        println!("{}", "Use 'keyfinder instances add' to create a new instance.".dimmed());
        return Ok(());
    }

    println!("\n{}", "Configured Provider Instances:".green().bold());

    let all_instances = instances.all_instances();
    let filtered_instances: Vec<&ProviderInstance> = all_instances
        .into_iter()
        .filter(|instance| {
            let type_match = provider_type.as_ref()
                .map_or(true, |pt| instance.provider_type == *pt);
            let active_match = !active_only || instance.active;
            type_match && active_match
        })
        .collect();

    if filtered_instances.is_empty() {
        println!("{}", "No instances match the specified criteria.".yellow());
        return Ok(());
    }

    let total_count = filtered_instances.len();
    
    for instance in filtered_instances {
        if verbose {
            println!("\n{} {}", instance.display_name.cyan().bold(), format!("({})", instance.id).dimmed());
            println!("  Provider Type: {}", instance.provider_type);
            println!("  Base URL: {}", instance.base_url);
            println!("  Status: {}", if instance.active { "Active".green() } else { "Inactive".red() });
            println!("  Keys: {} total, {} valid", instance.key_count(), instance.valid_key_count());
            println!("  Models: {} configured", instance.model_count());
            
            if !instance.models.is_empty() {
                let model_names: Vec<String> = instance.models.iter()
                    .map(|m| m.model_id.clone())
                    .collect();
                println!("  Available Models: {}", model_names.join(", "));
            }

            if let Some(metadata) = &instance.metadata {
                for (key, value) in metadata {
                    println!("  {}: {}", key, value);
                }
            }

            println!("  Created: {}", instance.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("  Updated: {}", instance.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        } else {
            let key_status = format!("{} keys ({} valid)", instance.key_count(), instance.valid_key_count());
            let model_status = format!("{} models", instance.model_count());
            println!("  {} - {} - {} - {}", 
                instance.display_name.cyan(), 
                instance.provider_type.yellow(),
                key_status.dimmed(),
                model_status.dimmed()
            );
        }
    }

    println!("\n{}", format!("Total instances: {}", total_count).cyan());

    Ok(())
}

/// Handle the add-instance command
pub fn handle_add_instance(
    id: String,
    name: String,
    provider_type: String,
    base_url: String,
    api_key: Option<String>,
    models: Option<String>,
    active: bool,
) -> Result<()> {
    let mut instances = load_provider_instances()?;

    // Check if instance with this ID already exists
    if instances.get_instance(&id).is_some() {
        return Err(anyhow::anyhow!("Provider instance with ID '{}' already exists", id));
    }

    let mut instance = ProviderInstance::new(id.clone(), name, provider_type, base_url);
    instance.active = active;

    // Add API key if provided
    if let Some(key_value) = api_key {
        let mut key = ProviderKey::new(
            "default".to_string(),
            "cli".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some(key_value);
        key.discovered_at = chrono::Utc::now();
        key.validation_status = ValidationStatus::Unknown;
        instance.add_key(key);
    }

    // Add models if provided
    if let Some(models_str) = models {
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                let model = Model::new(
                    model_id.clone(),
                    instance.id.clone(),
                    model_id,
                );
                instance.add_model(model);
            }
        }
    }

    // Validate the instance
    if let Err(e) = instance.validate() {
        return Err(anyhow::anyhow!("Invalid instance configuration: {}", e));
    }

    // Add to collection
    instances.add_instance(instance.clone()).map_err(|e| anyhow::anyhow!(e))?;

    // Save to disk - create a copy to avoid borrow issues
    let instances_copy = instances.clone();
    save_provider_instances(&instances_copy)?;

    println!("{} Provider instance '{}' added successfully.", "✓".green(), instance.display_name.cyan());
    println!("  ID: {}", instance.id);
    println!("  Type: {}", instance.provider_type);
    println!("  Status: {}", if instance.active { "Active" } else { "Inactive" });

    Ok(())
}

/// Handle the remove-instance command
pub fn handle_remove_instance(id: String, force: bool) -> Result<()> {
    let mut instances = load_provider_instances()?;

    // Check if instance exists
    if instances.get_instance(&id).is_none() {
        return Err(anyhow::anyhow!("Provider instance with ID '{}' not found", id));
    }

    // Get instance for confirmation
    let instance = instances.get_instance(&id).unwrap();
    
    if !force {
        println!("{}", "Warning: This will permanently remove the provider instance.".yellow().bold());
        println!("Instance: {} ({})", instance.display_name.cyan(), instance.id);
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

    println!("{} Provider instance '{}' removed successfully.", "✓".green(), id.cyan());

    Ok(())
}

/// Handle the update-instance command
pub fn handle_update_instance(
    id: String,
    name: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    models: Option<String>,
    active: Option<bool>,
) -> Result<()> {
    let mut instances = load_provider_instances()?;

    // Get mutable reference to the instance
    let instance = instances.get_instance_mut(&id)
        .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", id))?;

    // Store original values for later use
    let instance_name = instance.display_name.clone();
    let instance_id = instance.id.clone();

    // Update fields if provided
    if let Some(new_name) = name {
        instance.display_name = new_name;
    }

    if let Some(new_base_url) = base_url {
        instance.base_url = new_base_url;
    }

    if let Some(new_active) = active {
        instance.active = new_active;
    }

    // Update API key if provided
    if let Some(new_key_value) = api_key {
        // Remove existing default key if it exists
        instance.keys.retain(|key| key.id != "default");
        
        let mut key = ProviderKey::new(
            "default".to_string(),
            "cli".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some(new_key_value);
        key.discovered_at = chrono::Utc::now();
        key.validation_status = ValidationStatus::Unknown;
        instance.add_key(key);
    }

    // Update models if provided
    if let Some(models_str) = models {
        instance.models.clear();
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                let model = Model::new(
                    model_id.clone(),
                    instance.id.clone(),
                    model_id,
                );
                instance.add_model(model);
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

    println!("{} Provider instance '{}' updated successfully.", "✓".green(), instance_name.cyan());
    println!("  ID: {}", instance_id);
    println!("  Status: {}", if final_active_status { "Active" } else { "Inactive" });

    Ok(())
}

/// Handle the get-instance command
pub fn handle_get_instance(id: String, include_values: bool) -> Result<()> {
    let instances = load_provider_instances()?;

    let instance = instances.get_instance(&id)
        .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", id))?;

    println!("\n{} {}", instance.display_name.cyan().bold(), format!("({})", instance.id).dimmed());
    println!("{}", "─".repeat(50).dimmed());
    
    println!("Provider Type: {}", instance.provider_type.yellow());
    println!("Base URL: {}", instance.base_url);
    println!("Status: {}", if instance.active { "Active".green() } else { "Inactive".red() });
    println!("Created: {}", instance.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Updated: {}", instance.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));

    // Show keys
    println!("\n{}", "API Keys:".green().bold());
    if instance.keys.is_empty() {
        println!("  {}", "No keys configured".dimmed());
    } else {
        for key in &instance.keys {
            println!("  ID: {}", key.id.cyan());
            println!("    Environment: {:?}", key.environment);
            println!("    Confidence: {:?}", key.confidence);
            println!("    Status: {:?}", key.validation_status);
            println!("    Discovered: {}", key.discovered_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if include_values {
                if let Some(value) = &key.value {
                    println!("    Value: {}", value.red());
                } else {
                    println!("    Value: {}", "Not available".dimmed());
                }
            } else {
                println!("    Value: {}", if key.value.is_some() { "********" } else { "Not available" }.dimmed());
            }
            println!();
        }
    }

    // Show models
    println!("{}", "Models:".green().bold());
    if instance.models.is_empty() {
        println!("  {}", "No models configured".dimmed());
    } else {
        for model in &instance.models {
            println!("  {} - {}", model.model_id.cyan(), model.name);
            if let Some(capabilities) = &model.capabilities {
                let mut caps = Vec::new();
                if capabilities.text_generation { caps.push("text_generation"); }
                if capabilities.image_generation { caps.push("image_generation"); }
                if capabilities.audio_processing { caps.push("audio_processing"); }
                if capabilities.video_processing { caps.push("video_processing"); }
                if capabilities.code_generation { caps.push("code_generation"); }
                if capabilities.function_calling { caps.push("function_calling"); }
                if capabilities.fine_tuning { caps.push("fine_tuning"); }
                if capabilities.streaming { caps.push("streaming"); }
                if capabilities.multimodal { caps.push("multimodal"); }
                if capabilities.tool_use { caps.push("tool_use"); }
                println!("    Capabilities: {}", caps.join(", "));
            }
            if let Some(cost) = &model.cost {
                if let Some(input_cost) = cost.input_cost_per_million {
                    println!("    Input cost: ${} per 1M tokens", input_cost);
                }
                if let Some(output_cost) = cost.output_cost_per_million {
                    println!("    Output cost: ${} per 1M tokens", output_cost);
                }
            }
            println!();
        }
    }

    // Show metadata
    if let Some(metadata) = &instance.metadata {
        println!("{}", "Metadata:".green().bold());
        for (key, value) in metadata {
            println!("  {}: {}", key.cyan(), value);
        }
    }

    Ok(())
}

/// Handle the validate-instances command
pub fn handle_validate_instances(id: Option<String>, all_errors: bool) -> Result<()> {
    let instances = load_provider_instances()?;

    if instances.is_empty() {
        println!("{}", "No provider instances configured.".yellow());
        return Ok(());
    }

    if let Some(instance_id) = id {
        // Validate specific instance
        let instance = instances.get_instance(&instance_id)
            .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", instance_id))?;

        match instance.validate() {
            Ok(()) => {
                println!("{} Instance '{}' is valid.", "✓".green(), instance.display_name.cyan());
            }
            Err(e) => {
                println!("{} Instance '{}' has validation errors:", "✗".red(), instance.display_name.cyan());
                println!("  {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Validate all instances
        match instances.validate() {
            Ok(()) => {
                println!("{} All {} provider instances are valid.", "✓".green(), instances.len());
            }
            Err(e) => {
                println!("{} Validation errors found:", "✗".red());
                for error in e.split(';') {
                    println!("  {}", error.trim());
                    if !all_errors {
                        break;
                    }
                }
                std::process::exit(1);
            }
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
    println!("  Use 'keyfinder instances --help' for instance management commands");

    Ok(())
}