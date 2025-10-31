use anyhow::Result;
use colored::*;
use genai_keyfinder_core::models::{
    Confidence, Environment, Model, ProviderInstance, ProviderKey, ValidationStatus,
};
use genai_keyfinder_core::{scan, ScanOptions};
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

/// Sanitizes a provider name to prevent path traversal and OS issues
fn sanitize_provider_name(name: &str) -> String {
    // Convert to lowercase
    let mut sanitized = name.to_lowercase();

    // Replace any character not in [a-z0-9_-] with underscore
    sanitized = sanitized
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Collapse consecutive underscores into single underscore
    let mut result = String::new();
    let mut prev_was_underscore = false;
    for c in sanitized.chars() {
        if c == '_' {
            if !prev_was_underscore {
                result.push(c);
                prev_was_underscore = true;
            }
        } else {
            result.push(c);
            prev_was_underscore = false;
        }
    }

    // Trim leading/trailing underscores
    let trimmed = result.trim_matches('_');

    // If result is empty, use safe default
    if trimmed.is_empty() {
        "provider".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn handle_scan(
    home: Option<String>,
    format: String,
    include_values: bool,
    only: Option<String>,
    exclude: Option<String>,
    max_bytes_per_file: usize,
    dry_run: bool,
    audit_log: Option<String>,
    verbose: bool,
    update: bool,
) -> Result<()> {
    // Determine home directory
    let home_dir = match home {
        Some(h) => PathBuf::from(h),
        None => dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?,
    };

    // Parse provider filters
    let only_providers = only.map(|s| s.split(',').map(String::from).collect());
    let exclude_providers = exclude.map(|s| s.split(',').map(String::from).collect());

    // When --update is specified, we MUST include full values to write them to config files
    // Otherwise config files will contain redacted placeholders
    let include_full_values = include_values || update;

    // Create scan options
    let options = ScanOptions {
        home_dir: Some(home_dir.clone()),
        include_full_values,
        max_file_size: max_bytes_per_file,
        only_providers,
        exclude_providers,
    };

    if dry_run {
        println!("{}", "DRY RUN MODE - No files will be read".yellow().bold());
        // Show what would be scanned
        println!("Would scan directory: {}", home_dir.display());
        return Ok(());
    }

    // Perform scan
    println!("{}", "Scanning for GenAI credentials...".cyan().bold());
    let result = scan(options)?;

    // Output results based on format
    match format.as_str() {
        "json" => crate::output::json::output_json(&result, verbose)?,
        "ndjson" => crate::output::ndjson::output_ndjson(&result, verbose)?,
        "table" => crate::output::table::output_table(&result, verbose)?,
        "summary" => crate::output::summary::output_summary(&result, verbose)?,
        _ => anyhow::bail!("Unknown format: {}", format),
    }

    // Write audit log if requested
    if let Some(log_path) = audit_log {
        write_audit_log(&log_path, &result)?;
    }

    // Exit code: 0 if keys found, 1 if none found
    if result.keys.is_empty() && result.config_instances.is_empty() {
        std::process::exit(1);
    }

    // Update YAML configuration file if requested
    if update {
        update_yaml_config(&result, &home_dir)?;
    }

    Ok(())
}

/// Helper function to create a full Model struct with capabilities based on model ID
fn create_full_model(model_id: &str) -> Model {
    let model = Model::new(model_id.to_string(), model_id.to_string());

    // Set default capabilities based on common model patterns
    let mut capabilities = genai_keyfinder_core::models::Capabilities::default();

    // Enable text generation for most models
    capabilities.text_generation = true;

    // Set specific capabilities based on model name patterns
    let model_lower = model_id.to_lowercase();
    if model_lower.contains("gpt")
        || model_lower.contains("claude")
        || model_lower.contains("llama")
    {
        capabilities.code_generation = true;
        capabilities.function_calling = true;
        capabilities.streaming = true;
    }

    if model_lower.contains("vision") || model_lower.contains("multimodal") {
        capabilities.multimodal = true;
        capabilities.image_generation = true;
    }

    if model_lower.contains("dall")
        || model_lower.contains("stable-diffusion")
        || model_lower.contains("midjourney")
    {
        capabilities.image_generation = true;
    }

    if model_lower.contains("whisper") || model_lower.contains("audio") {
        capabilities.audio_processing = true;
    }

    model.with_capabilities(capabilities)
}

/// Helper function to save a model to a config file and return a reference
fn save_model_config(model: &Model, models_dir: &std::path::Path) -> Result<String> {
    let model_filename = format!("{}.yaml", sanitize_provider_name(&model.model_id));
    let model_path = models_dir.join(&model_filename);

    let yaml_content = serde_yaml::to_string(model)?;
    std::fs::write(&model_path, yaml_content)?;

    Ok(model_filename)
}

/// Helper function to load a model from a config file
fn load_model_config(model_path: &std::path::Path) -> Result<Model> {
    let content = std::fs::read_to_string(model_path)?;
    let model: Model = serde_yaml::from_str(&content)?;
    Ok(model)
}

/// Helper function to get default base URLs for providers
fn get_default_base_url(provider_name: &str) -> String {
    match provider_name.to_lowercase().as_str() {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
        "huggingface" => "https://huggingface.co/api".to_string(),
        "google" => "https://generativelanguage.googleapis.com".to_string(),
        _ => "https://api.example.com".to_string(),
    }
}

/// Helper function to hash a value using SHA-256 (for API key filename generation)
fn hash_value(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Updates or creates the YAML configuration files with discovered providers and keys
/// NEW APPROACH: One instance per API key, using API key as instance ID
fn update_yaml_config(
    result: &genai_keyfinder_core::ScanResult,
    home_dir: &std::path::Path,
) -> Result<()> {
    let config_dir = home_dir
        .join(".config")
        .join("aicred")
        .join("inference_services");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)?;

    // Debug: Print the actual directory being used
    tracing::info!("Using config directory: {}", config_dir.display());

    let old_config_path = config_dir.parent().unwrap().join("providers.yaml");

    // Create models directory
    let models_dir = config_dir.parent().unwrap().join("models");
    std::fs::create_dir_all(&models_dir)?;

    // Check if we need to migrate from old format
    if old_config_path.exists() && !config_dir.exists() {
        println!(
            "{}",
            "Migrating from old single-file format to new multi-file format...".yellow()
        );
        migrate_from_single_file(&old_config_path, &config_dir)?;
    }

    let now = chrono::Utc::now();

    // NEW APPROACH: Process each API key individually instead of grouping by provider
    // Create a map to track configuration context by source file
    let mut source_context: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<genai_keyfinder_core::models::DiscoveredKey>>,
    > = std::collections::HashMap::new();

    // Collect all keys by source file first
    for key in &result.keys {
        let source = key.source.clone();
        let provider = key.provider.clone();
        source_context
            .entry(source)
            .or_default()
            .entry(provider)
            .or_default()
            .push(key.clone());
    }

    // Also collect from config instances
    for instance in &result.config_instances {
        for key in &instance.keys {
            let source = key.source.clone();
            let provider = key.provider.clone();
            source_context
                .entry(source)
                .or_default()
                .entry(provider)
                .or_default()
                .push(key.clone());
        }
    }

    // Process each source file individually
    for (source_file, provider_keys) in source_context {
        tracing::debug!(
            "Processing source file: {} with {} providers",
            source_file,
            provider_keys.len()
        );

        // For each provider in this source file, find the API key and create an instance
        for (provider_name, keys) in provider_keys {
            tracing::debug!(
                "Processing provider {} from source {} with {} keys",
                provider_name,
                source_file,
                keys.len()
            );

            // Find API keys in this group
            let api_keys: Vec<&genai_keyfinder_core::models::DiscoveredKey> = keys
                .iter()
                .filter(|k| {
                    matches!(
                        k.value_type,
                        genai_keyfinder_core::models::discovered_key::ValueType::ApiKey
                    )
                })
                .collect();

            // Also check for other key types that can serve as primary keys
            let other_keys: Vec<&genai_keyfinder_core::models::DiscoveredKey> = keys
                .iter()
                .filter(|k| {
                    matches!(
                        k.value_type,
                        genai_keyfinder_core::models::discovered_key::ValueType::AccessToken
                            | genai_keyfinder_core::models::discovered_key::ValueType::SecretKey
                            | genai_keyfinder_core::models::discovered_key::ValueType::BearerToken
                    )
                })
                .collect();

            // Process each API key found in this source file
            let primary_keys = if !api_keys.is_empty() {
                api_keys
            } else {
                other_keys
            };

            for primary_key in primary_keys {
                // Use the hash as the instance ID since the actual API key is redacted
                let instance_id = &primary_key.hash;

                // Create a hash-based filename using first 16 chars of instance hash
                let filename = format!("{}.yaml", &instance_id[..16]);
                let instance_file_path = config_dir.join(&filename);

                tracing::debug!(
                    "Creating instance for API key from provider {}: filename={}",
                    provider_name,
                    filename
                );

                // Since we don't have the actual API key (it's redacted), we'll store a placeholder
                // The actual API key will need to be manually configured or the user will need to
                // run the scan with --include-values (which is not recommended for security)
                let api_key_placeholder = format!("REDACTED_{}", &instance_id[..16]);

                // Check if we have the API key value (it might be redacted)
                if let Some(api_key_value) = primary_key.full_value() {
                    // Use the actual API key if available
                    let api_key_to_store = api_key_value.to_string();

                    // Check if this instance already exists
                    let mut instance = if instance_file_path.exists() {
                        let content = std::fs::read_to_string(&instance_file_path)?;
                        match serde_yaml::from_str::<ProviderInstance>(&content) {
                            Ok(mut instance) => {
                                tracing::debug!(
                                    "Successfully loaded existing instance: {}",
                                    instance.id
                                );
                                instance.updated_at = now;
                                instance
                            }
                            Err(e) => {
                                tracing::warn!("Failed to deserialize existing instance at {}: {}. Creating new instance.", instance_file_path.display(), e);
                                // Create new instance if deserialization fails
                                let mut new_instance = ProviderInstance::new(
                                    instance_id.to_string(),
                                    format!("{} Instance", provider_name),
                                    provider_name.to_lowercase(),
                                    get_default_base_url(&provider_name),
                                );
                                new_instance.updated_at = now;
                                new_instance
                            }
                        }
                    } else {
                        // Create new instance
                        println!(
                            "{}",
                            format!("Creating new instance for {} API key", provider_name).green()
                        );
                        let mut new_instance = ProviderInstance::new(
                            instance_id.to_string(),
                            format!("{} Instance", provider_name),
                            provider_name.to_lowercase(),
                            get_default_base_url(&provider_name),
                        );
                        new_instance.updated_at = now;
                        new_instance
                    };

                    // Extract models, base_url, and metadata from the same source file
                    let mut _base_url: Option<String> = None;
                    let mut models_found = Vec::new();
                    let mut metadata_map = instance.metadata.clone().unwrap_or_default();

                    // Remove base_url and model_id from metadata if they exist (they should be at instance/models level)
                    metadata_map.remove("base_url");
                    metadata_map.remove("baseurl");
                    metadata_map.remove("model_id");
                    metadata_map.remove("modelid");

                    for key in &keys {
                        if let genai_keyfinder_core::models::discovered_key::ValueType::Custom(
                            ref custom_type,
                        ) = key.value_type
                        {
                            let custom_type_lower = custom_type.to_lowercase();

                            if custom_type_lower == "baseurl" {
                                if let Some(full_value) = key.full_value() {
                                    _base_url = Some(full_value.to_string());
                                    instance.base_url = full_value.to_string();
                                }
                            } else if custom_type_lower == "modelid" {
                                if let Some(full_value) = key.full_value() {
                                    models_found.push(full_value.to_string());
                                }
                            } else {
                                // Collect all other custom fields as metadata (merge with existing)
                                if let Some(full_value) = key.full_value() {
                                    metadata_map.insert(custom_type_lower, full_value.to_string());
                                }
                            }
                        }
                    }

                    // Always preserve metadata (even if empty from new scan, keep existing)
                    if !metadata_map.is_empty() || instance.metadata.is_some() {
                        instance.metadata = Some(metadata_map);
                    }

                    // Add found models to the instance - create full models and save to config files
                    // Only add models that don't already exist
                    for model_id in models_found {
                        let model_exists = instance.models.iter().any(|m| m.model_id == model_id);
                        if !model_exists {
                            let full_model = create_full_model(&model_id);
                            let _model_ref = save_model_config(&full_model, &models_dir)?;

                            // Create a lightweight model reference for the instance
                            let model_ref = Model::new(model_id.clone(), model_id.clone());
                            instance.add_model(model_ref);
                        }
                    }

                    // Check if this key already exists in the instance
                    let key_exists = instance.keys.iter().any(|k| {
                        k.value
                            .as_ref()
                            .map(|v| v == &api_key_to_store)
                            .unwrap_or(false)
                    });

                    // Only add the key if it doesn't already exist
                    if !key_exists {
                        let mut provider_key = ProviderKey::new(
                            instance_id.to_string(),
                            primary_key.source.clone(),
                            Confidence::High,
                            Environment::Production,
                        );
                        provider_key.value = Some(api_key_to_store.clone());
                        provider_key.discovered_at = primary_key.discovered_at;
                        provider_key.validation_status = ValidationStatus::Unknown;
                        instance.add_key(provider_key);
                    }

                    // Save the instance configuration
                    let yaml_content = serde_yaml::to_string(&instance)?;
                    std::fs::write(&instance_file_path, yaml_content)?;

                    tracing::debug!("Saved instance config to: {}", instance_file_path.display());
                } else {
                    // API key is redacted - still create instance but with placeholder
                    // Check if this instance already exists
                    let mut instance = if instance_file_path.exists() {
                        let content = std::fs::read_to_string(&instance_file_path)?;
                        match serde_yaml::from_str::<ProviderInstance>(&content) {
                            Ok(mut instance) => {
                                instance.updated_at = now;
                                instance
                            }
                            Err(_) => {
                                // Create new instance if deserialization fails
                                let mut new_instance = ProviderInstance::new(
                                    instance_id.to_string(),
                                    format!("{} Instance", provider_name),
                                    provider_name.to_lowercase(),
                                    get_default_base_url(&provider_name),
                                );
                                new_instance.updated_at = now;
                                new_instance
                            }
                        }
                    } else {
                        // Create new instance
                        println!(
                            "{}",
                            format!(
                                "Creating new instance for {} API key (redacted)",
                                provider_name
                            )
                            .green()
                        );
                        let mut new_instance = ProviderInstance::new(
                            instance_id.to_string(),
                            format!("{} Instance", provider_name),
                            provider_name.to_lowercase(),
                            get_default_base_url(&provider_name),
                        );
                        new_instance.updated_at = now;
                        new_instance
                    };

                    // Extract models, base_url, and metadata from the same source file
                    let mut _base_url: Option<String> = None;
                    let mut models_found = Vec::new();
                    let mut metadata_map = instance.metadata.clone().unwrap_or_default();

                    // Remove base_url and model_id from metadata if they exist (they should be at instance/models level)
                    metadata_map.remove("base_url");
                    metadata_map.remove("baseurl");
                    metadata_map.remove("model_id");
                    metadata_map.remove("modelid");

                    for key in &keys {
                        if let genai_keyfinder_core::models::discovered_key::ValueType::Custom(
                            ref custom_type,
                        ) = key.value_type
                        {
                            let custom_type_lower = custom_type.to_lowercase();

                            if custom_type_lower == "baseurl" {
                                if let Some(full_value) = key.full_value() {
                                    _base_url = Some(full_value.to_string());
                                    instance.base_url = full_value.to_string();
                                }
                            } else if custom_type_lower == "modelid" {
                                if let Some(full_value) = key.full_value() {
                                    models_found.push(full_value.to_string());
                                }
                            } else {
                                // Collect all other custom fields as metadata
                                if let Some(full_value) = key.full_value() {
                                    metadata_map.insert(custom_type_lower, full_value.to_string());
                                }
                            }
                        }
                    }

                    // Set metadata if any was collected (merge with existing)
                    if !metadata_map.is_empty() {
                        instance.metadata = Some(metadata_map);
                    }

                    // Add found models to the instance - create full models and save to config files
                    // Only add models that don't already exist
                    for model_id in models_found {
                        let model_exists = instance.models.iter().any(|m| m.model_id == model_id);
                        if !model_exists {
                            let full_model = create_full_model(&model_id);
                            let _model_ref = save_model_config(&full_model, &models_dir)?;

                            // Create a lightweight model reference for the instance
                            let model_ref = Model::new(model_id.clone(), model_id.clone());
                            instance.add_model(model_ref);
                        }
                    }

                    // Check if this key already exists in the instance (by placeholder or hash)
                    let key_exists = instance.keys.iter().any(|k| {
                        k.id == *instance_id
                            || k.value
                                .as_ref()
                                .map(|v| v == &api_key_placeholder)
                                .unwrap_or(false)
                    });

                    // Only add the key if it doesn't already exist
                    if !key_exists {
                        let mut provider_key = ProviderKey::new(
                            instance_id.to_string(),
                            primary_key.source.clone(),
                            Confidence::High,
                            Environment::Production,
                        );
                        provider_key.value = Some(api_key_placeholder.clone());
                        provider_key.discovered_at = primary_key.discovered_at;
                        provider_key.validation_status = ValidationStatus::Unknown;
                        instance.add_key(provider_key);
                    }

                    // Save the instance configuration
                    let yaml_content = serde_yaml::to_string(&instance)?;
                    std::fs::write(&instance_file_path, yaml_content)?;

                    tracing::debug!("Saved instance config to: {}", instance_file_path.display());
                }
            }
        }
    }

    println!(
        "{}",
        format!("Updated configuration files in: {}", config_dir.display())
            .green()
            .bold()
    );

    Ok(())
}

/// Migrates from the old single-file format to the new multi-file format
fn migrate_from_single_file(old_path: &PathBuf, config_dir: &PathBuf) -> Result<()> {
    println!("{}", "Backing up old configuration file...".yellow());

    // Create backup
    let backup_path = old_path.with_extension("yaml.backup");
    std::fs::copy(old_path, &backup_path)?;

    // Load old format
    let content = std::fs::read_to_string(old_path)?;
    let old_config: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Extract providers from old format
    if let Some(providers) = old_config.get("providers").and_then(|p| p.as_mapping()) {
        let now = chrono::Utc::now();

        for (provider_name, provider_data) in providers {
            if let Some(name) = provider_name.as_str() {
                let _provider_name = name.to_string();

                // Extract API key from old format
                if let Some(api_key) = provider_data.get("api_key").and_then(|v| v.as_str()) {
                    // Create instance file using hash of API key as filename
                    let instance_file_name = format!("{}.yaml", sanitize_provider_name(name));
                    let instance_file_path = config_dir.join(&instance_file_name);

                    // Convert provider data to new format
                    let mut instance = ProviderInstance::new(
                        api_key.to_string(),
                        name.to_string(),
                        name.to_lowercase(),
                        get_default_base_url(name),
                    );
                    instance.updated_at = now;

                    // Extract models from old format and add them as Model objects
                    if let Some(models) = provider_data.get("models").and_then(|v| v.as_sequence())
                    {
                        for model_str in models.iter().filter_map(|m| m.as_str()) {
                            instance.add_model(Model::new(
                                model_str.to_string(),
                                model_str.to_string(),
                            ));
                        }
                    }

                    // Add the API key as a ProviderKey
                    let mut provider_key = ProviderKey::new(
                        api_key.to_string(),
                        "migration".to_string(),
                        Confidence::High,
                        Environment::Production,
                    );
                    provider_key.value = Some(api_key.to_string());
                    provider_key.discovered_at = now;
                    provider_key.validation_status = ValidationStatus::Unknown;
                    instance.add_key(provider_key);

                    if let Some(version) = provider_data.get("version").and_then(|v| v.as_str()) {
                        // Version is not directly supported in ProviderInstance, so we'll skip it
                    }

                    // Write individual instance file
                    let yaml_content = serde_yaml::to_string(&instance)?;
                    std::fs::write(&instance_file_path, yaml_content)?;
                }
            }
        }

        // Remove old file
        std::fs::remove_file(old_path)?;

        println!("{}", "Migration completed successfully!".green());
    }

    Ok(())
}

fn write_audit_log(log_path: &str, result: &genai_keyfinder_core::ScanResult) -> Result<()> {
    #[cfg(unix)]
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(log_path)?;
    #[cfg(not(unix))]
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(log_path)?;
    writeln!(file, "GenAI Key Finder Audit Log")?;
    writeln!(file, "=========================")?;
    writeln!(file, "Scan Date: {}", result.scan_completed_at)?;
    writeln!(file, "Home Directory: {}", result.home_directory)?;
    writeln!(
        file,
        "Providers Scanned: {}",
        result.providers_scanned.join(", ")
    )?;
    writeln!(file, "Total Keys Found: {}", result.keys.len())?;
    writeln!(
        file,
        "Total Config Instances: {}",
        result.config_instances.len()
    )?;
    writeln!(file)?;

    if !result.keys.is_empty() {
        writeln!(file, "Discovered Keys:")?;
        for key in &result.keys {
            writeln!(
                file,
                "  - {}: {} ({} - confidence: {})",
                key.provider, key.value_type, key.source, key.confidence
            )?;
        }
    }

    if !result.config_instances.is_empty() {
        writeln!(file, "\nConfig Instances:")?;
        for instance in &result.config_instances {
            writeln!(
                file,
                "  - {}: {}",
                instance.app_name,
                instance.config_path.display()
            )?;
        }
    }

    Ok(())
}
