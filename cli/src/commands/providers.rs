use aicred_core::models::{
    Confidence, Environment, Model, ProviderInstance, ProviderInstances, ProviderKey,
    ValidationStatus,
};
use anyhow::Result;
use colored::*;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

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
                if let Ok(instance) = serde_yaml::from_str::<ProviderInstance>(&content) {
                    // If the deserialized instance already contains an API key, accept it.
                    // Otherwise, if the file looks like the legacy format (contains "keys:"),
                    // try parsing as ProviderConfig and convert so we don't lose nested keys.
                    if instance.get_api_key().is_some() {
                        let _ = instances.add_instance(instance);
                        continue;
                    } else if content.contains("keys:") {
                        match aicred_core::models::ProviderConfig::from_yaml(&content) {
                            Ok(config) => {
                                let instance: ProviderInstance = config.into();
                                let _ = instances.add_instance(instance);
                                continue;
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}: {}",
                                    "Error parsing instance file:".red(),
                                    path.display(),
                                    e
                                );
                                // fall through to next handling (will log below)
                            }
                        }
                    } else {
                        // No API key and not legacy-shaped; still add the instance as-is.
                        let _ = instances.add_instance(instance);
                        continue;
                    }
                }

                // If direct deserialization failed (or the file uses legacy 'keys' shape),
                // attempt to parse as a ProviderConfig (legacy multi-key format) and convert.
                match aicred_core::models::ProviderConfig::from_yaml(&content) {
                    Ok(config) => {
                        let instance: ProviderInstance = config.into();
                        let _ = instances.add_instance(instance);
                    }
                    Err(_e) => {
                        // Fallback: try a permissive parse for ad-hoc YAML fixtures produced by tests
                        // which may omit fields like `version` or use model objects instead of strings.
                        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            if let serde_yaml::Value::Mapping(map) = value {
                                use chrono::DateTime;
                                use chrono::Utc;

                                // Helper to extract string fields
                                let get_str = |k: &str| -> Option<String> {
                                    map.get(&serde_yaml::Value::String(k.to_string()))
                                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                                };

                                let id = get_str("id").unwrap_or_else(|| {
                                    // fallback to filename-like id
                                    path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("unknown")
                                        .to_string()
                                });
                                let display_name =
                                    get_str("display_name").unwrap_or_else(|| id.clone());
                                let provider_type = get_str("provider_type")
                                    .unwrap_or_else(|| "unknown".to_string());
                                let base_url = get_str("base_url")
                                    .unwrap_or_else(|| "https://api.example.com".to_string());

                                // Parse timestamps if present
                                let created_at = get_str("created_at")
                                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                                    .map(|dt| dt.with_timezone(&Utc))
                                    .unwrap_or_else(|| Utc::now());
                                let updated_at = get_str("updated_at")
                                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                                    .map(|dt| dt.with_timezone(&Utc))
                                    .unwrap_or_else(|| created_at);

                                let mut instance = ProviderInstance::new(
                                    id.clone(),
                                    display_name,
                                    provider_type.clone(),
                                    base_url,
                                );

                                // Active flag
                                if let Some(active_val) =
                                    map.get(&serde_yaml::Value::String("active".to_string()))
                                {
                                    if let Some(b) = active_val.as_bool() {
                                        instance.active = b;
                                    }
                                }

                                // Extract API key from legacy `keys` sequence if present
                                if let Some(keys_val) =
                                    map.get(&serde_yaml::Value::String("keys".to_string()))
                                {
                                    if let Some(seq) = keys_val.as_sequence() {
                                        if !seq.is_empty() {
                                            if let Some(first_key) = seq[0].as_mapping() {
                                                // try api_key then value as fallbacks
                                                let api_key = first_key
                                                    .get(&serde_yaml::Value::String(
                                                        "api_key".to_string(),
                                                    ))
                                                    .or_else(|| {
                                                        first_key.get(&serde_yaml::Value::String(
                                                            "value".to_string(),
                                                        ))
                                                    })
                                                    .and_then(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    });
                                                if let Some(k) = api_key {
                                                    instance.set_api_key(k);
                                                }
                                            } else if let Some(s) = seq[0].as_str() {
                                                // key represented as string (rare) - treat as api_key
                                                instance.set_api_key(s.to_string());
                                            }
                                        }
                                    }
                                }

                                // Extract models: either sequence of strings or sequence of maps with model_id
                                if let Some(models_val) =
                                    map.get(&serde_yaml::Value::String("models".to_string()))
                                {
                                    if let Some(seq) = models_val.as_sequence() {
                                        for item in seq {
                                            if let Some(s) = item.as_str() {
                                                let model =
                                                    Model::new(s.to_string(), s.to_string());
                                                instance.add_model(model);
                                            } else if let Some(m) = item.as_mapping() {
                                                if let Some(model_id_val) =
                                                    m.get(&serde_yaml::Value::String(
                                                        "model_id".to_string(),
                                                    ))
                                                {
                                                    if let Some(model_id) = model_id_val.as_str() {
                                                        let model = Model::new(
                                                            model_id.to_string(),
                                                            model_id.to_string(),
                                                        );
                                                        instance.add_model(model);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Preserve parsed timestamps on the instance if possible
                                instance.created_at = created_at;
                                instance.updated_at = updated_at;

                                let _ = instances.add_instance(instance);
                                continue;
                            }
                        }

                        // If all parsing attempts fail, log error so tests see the diagnostic
                        eprintln!(
                            "{} {}: failed to parse as ProviderConfig or permissive YAML",
                            "Error parsing instance file:".red(),
                            path.display()
                        );
                    }
                }
            }
        }
    }

    Ok(instances)
}

/// Load instances from legacy provider configurations
fn load_instances_from_providers(
    providers_dir: &PathBuf,
    instances: &mut ProviderInstances,
) -> Result<()> {
    use aicred_core::models::ProviderConfig;

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
                        // Generate instance ID as first 4 characters of SHA-256 hash of the file content
                        let mut hasher = Sha256::new();
                        hasher.update(content.as_bytes());
                        let hash_result = hasher.finalize();
                        let full_hash = format!("{:x}", hash_result);
                        let instance_id = full_hash[..4].to_string();
                        let display_name = file_stem
                            .split('_')
                            .map(|word| {
                                let mut chars = word.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(first) => {
                                        first.to_uppercase().collect::<String>() + chars.as_str()
                                    }
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(" ");

                        let (provider_type, base_url) = match file_stem.to_lowercase().as_str() {
                            s if s.contains("openai") => ("openai", "https://api.openai.com"),
                            s if s.contains("anthropic") => {
                                ("anthropic", "https://api.anthropic.com")
                            }
                            s if s.contains("huggingface") => {
                                ("huggingface", "https://huggingface.co")
                            }
                            s if s.contains("ollama") => ("ollama", "http://localhost:11434"),
                            s if s.contains("groq") => ("groq", "https://api.groq.com"),
                            s if s.contains("test") => ("test", "https://api.example.com"),
                            _ => ("unknown", "https://api.example.com"),
                        };

                        let mut instance = ProviderInstance::new(
                            instance_id,
                            display_name,
                            provider_type.to_string(),
                            base_url.to_string(),
                        );

                        // Copy API key - use the first key if available
                        if let Some(first_key) = config.keys.first() {
                            if let Some(key_value) = &first_key.value {
                                instance.set_api_key(key_value.clone());
                            }
                        }

                        // Convert model strings to Model objects
                        for model_id in &config.models {
                            let model = Model::new(model_id.clone(), model_id.clone());
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
        .join("inference_services");

    std::fs::create_dir_all(&config_dir)?;

    // Save each instance to its own file
    for instance in instances.all_instances() {
        // Use provider name and first 4 chars of instance ID (hash)
        let file_name = format!("{}-{}.yaml", instance.provider_type, &instance.id[..4]);
        let file_path = config_dir.join(&file_name);

        // Serialize into a legacy-compatible ProviderConfig-ish YAML so tests and users
        // that expect a `keys` sequence continue to work. We keep minimal fields:
        // id, display_name, provider_type, base_url, active, keys (with api_key),
        // models (list of model objects), created_at, updated_at.
        use serde_yaml::Value;

        let mut top = serde_yaml::Mapping::new();
        top.insert(
            Value::String("id".into()),
            Value::String(instance.id.clone()),
        );
        top.insert(
            Value::String("display_name".into()),
            Value::String(instance.display_name.clone()),
        );
        top.insert(
            Value::String("provider_type".into()),
            Value::String(instance.provider_type.clone()),
        );
        top.insert(
            Value::String("base_url".into()),
            Value::String(instance.base_url.clone()),
        );
        top.insert(Value::String("active".into()), Value::Bool(instance.active));

        // Keys: represent the single api_key (if present) as a sequence with one mapping
        let mut keys_seq = serde_yaml::Sequence::new();
        if let Some(api_key) = instance.get_api_key() {
            let mut key_map = serde_yaml::Mapping::new();
            key_map.insert(Value::String("id".into()), Value::String("default".into()));
            key_map.insert(
                Value::String("api_key".into()),
                Value::String(api_key.clone()),
            );
            // Include minimal discovered_at/created_at placeholders so older consumers are happy
            key_map.insert(
                Value::String("discovered_at".into()),
                Value::String(instance.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            );
            keys_seq.push(Value::Mapping(key_map));
        }
        top.insert(Value::String("keys".into()), Value::Sequence(keys_seq));

        // Models: convert to simple mapping objects with model_id and name
        let mut models_seq = serde_yaml::Sequence::new();
        for model in &instance.models {
            let mut m = serde_yaml::Mapping::new();
            m.insert(
                Value::String("model_id".into()),
                Value::String(model.model_id.clone()),
            );
            m.insert(
                Value::String("name".into()),
                Value::String(model.name.clone()),
            );
            models_seq.push(Value::Mapping(m));
        }
        top.insert(Value::String("models".into()), Value::Sequence(models_seq));

        // Timestamps
        top.insert(
            Value::String("created_at".into()),
            Value::String(instance.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );
        top.insert(
            Value::String("updated_at".into()),
            Value::String(instance.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );

        let yaml_value = Value::Mapping(top);
        let yaml_content = serde_yaml::to_string(&yaml_value)?;
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
            println!(
                "\n{} {}",
                instance.key_name().cyan().bold(),
                format!("({})", instance.display_name).dimmed()
            );
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
                let model_names: Vec<String> =
                    instance.models.iter().map(|m| m.model_id.clone()).collect();
                println!("  Available Models: {}", model_names.join(", "));
            }

            if let Some(metadata) = &instance.metadata {
                for (key, value) in metadata {
                    println!("  {}: {}", key, value);
                }
            }

            println!(
                "  Created: {}",
                instance.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!(
                "  Updated: {}",
                instance.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        } else {
            let key_status = format!(
                "{} keys ({} valid)",
                if instance.has_api_key() { 1 } else { 0 },
                if instance.has_non_empty_api_key() {
                    1
                } else {
                    0
                }
            );
            let model_status = format!("{} models", instance.model_count());
            println!(
                "  {} - {} - {} - {}",
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
    let mut instances = load_provider_instances(None)?;

    // Check if instance with this ID already exists
    if instances.get_instance(&id).is_some() {
        return Err(anyhow::anyhow!(
            "Provider instance with ID '{}' already exists",
            id
        ));
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
        instance.set_api_key(key.value.unwrap_or_default());
    }

    // Add models if provided
    if let Some(models_str) = models {
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                let model = Model::new(model_id.clone(), model_id);
                instance.add_model(model);
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
        instance.display_name.cyan()
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
        println!(
            "Instance: {} ({})",
            instance.display_name.cyan(),
            instance.id
        );
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
    name: Option<String>,
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
        // For single API key, we can't retain specific keys, so we clear it if it matches
        if let Some(current_key) = instance.get_api_key() {
            if current_key.is_empty() {
                instance.set_api_key(String::new());
            }
        }

        let mut key = ProviderKey::new(
            "default".to_string(),
            "cli".to_string(),
            Confidence::High,
            Environment::Production,
        );
        key.value = Some(new_key_value);
        key.discovered_at = chrono::Utc::now();
        key.validation_status = ValidationStatus::Unknown;
        instance.set_api_key(key.value.unwrap_or_default());
    }

    // Update models if provided
    if let Some(models_str) = models {
        instance.models.clear();
        for model_id in models_str.split(',') {
            let model_id = model_id.trim().to_string();
            if !model_id.is_empty() {
                let model = Model::new(model_id.clone(), model_id);
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

    println!(
        "{} Provider instance '{}' updated successfully.",
        "✓".green(),
        instance_name.cyan()
    );
    println!("  ID: {}", instance_id);
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

    let instance = instances
        .get_instance(&id)
        .ok_or_else(|| anyhow::anyhow!("Provider instance with ID '{}' not found", id))?;

    println!(
        "\n{} {}",
        instance.key_name().cyan().bold(),
        format!("({})", instance.display_name).dimmed()
    );
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
    println!(
        "Created: {}",
        instance.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "Updated: {}",
        instance.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Show keys
    println!("\n{}", "API Keys:".green().bold());
    if let Some(api_key) = instance.get_api_key() {
        if include_values {
            println!("  Value: {}", api_key.red());
        } else {
            println!("  Value: {}", "********".dimmed());
        }
        println!("  Status: {:?}", ValidationStatus::Unknown);
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
            println!("  {} - {}", model.model_id.cyan(), model.name);
            if let Some(capabilities) = &model.capabilities {
                let mut caps = Vec::new();
                if capabilities.text_generation {
                    caps.push("text_generation");
                }
                if capabilities.image_generation {
                    caps.push("image_generation");
                }
                if capabilities.audio_processing {
                    caps.push("audio_processing");
                }
                if capabilities.video_processing {
                    caps.push("video_processing");
                }
                if capabilities.code_generation {
                    caps.push("code_generation");
                }
                if capabilities.function_calling {
                    caps.push("function_calling");
                }
                if capabilities.fine_tuning {
                    caps.push("fine_tuning");
                }
                if capabilities.streaming {
                    caps.push("streaming");
                }
                if capabilities.multimodal {
                    caps.push("multimodal");
                }
                if capabilities.tool_use {
                    caps.push("tool_use");
                }
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
                    instance.display_name.cyan()
                );
            }
            Err(e) => {
                println!(
                    "{} Instance '{}' has validation errors:",
                    "✗".red(),
                    instance.display_name.cyan()
                );
                println!("  {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Validate all instances
        match instances.validate() {
            Ok(()) => {
                println!(
                    "{} All {} provider instances are valid.",
                    "✓".green(),
                    instances.len()
                );
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
    println!("  Use 'aicred instances --help' for instance management commands");

    Ok(())
}
