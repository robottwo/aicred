use aicred_core::models::{Model, ProviderInstance};
use aicred_core::{scan, DiscoveredCredential, ScanOptions};
use anyhow::Result;
use colored::*;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::Arc;

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

#[allow(clippy::too_many_arguments)]
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
    probe_models: bool,
    probe_timeout: Option<u64>,
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
        probe_models,
        probe_timeout_secs: probe_timeout.unwrap_or(30),
    };

    if dry_run {
        println!("{}", "DRY RUN MODE - No files will be read".yellow().bold());
        // Show what would be scanned
        println!("Would scan directory: {}", home_dir.display());
        return Ok(());
    }

    // Perform scan
    println!("{}", "Scanning for GenAI credentials...".cyan().bold());
    let result = scan(&options)?;

    // Output results based on format
    match format.as_str() {
        "json" => crate::output::json::output_json(&result, verbose, None)?,
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
    let capabilities = aicred_core::models::ModelCapabilities {
        chat: true,
        completion: true,
        ..Default::default()
    };

    // Set specific capabilities based on model name patterns
    let model_lower = model_id.to_lowercase();
    let mut capabilities = capabilities;

    if model_lower.contains("gpt")
        || model_lower.contains("claude")
        || model_lower.contains("llama")
    {
        capabilities.function_calling = true;
    }

    if model_lower.contains("vision") || model_lower.contains("multimodal") {
        capabilities.vision = true;
    }

    Model {
        id: model_id.to_string(),
        provider: "unknown".to_string(),
        name: model_id.to_string(),
        capabilities,
        context_window: None,
        pricing: None,
        metadata: Default::default(),
    }
}

/// Helper function to save a model to a config file and return a reference
fn save_model_config(model: &Model, models_dir: &std::path::Path) -> Result<String> {
    let model_filename = format!("{}.yaml", sanitize_provider_name(&model.id));
    let model_path = models_dir.join(&model_filename);

    let yaml_content = serde_yaml::to_string(model)?;
    std::fs::write(&model_path, yaml_content)?;

    Ok(model_filename)
}

/// Helper function to load a model from a config file
#[allow(dead_code)]
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
#[allow(dead_code)]
fn hash_value(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Updates or creates the YAML configuration files with discovered providers and keys
/// NEW APPROACH: One instance per API key, using API key as instance ID
fn update_yaml_config(result: &aicred_core::ScanResult, home_dir: &std::path::Path) -> Result<()> {
    let config_dir = home_dir
        .join(".config")
        .join("aicred")
        .join("inference_services");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_dir)?;

    // Debug: Print the actual directory being used
    tracing::info!("Using config directory: {}", config_dir.display());

    // Create models directory
    let models_dir = config_dir.parent().unwrap().join("models");
    std::fs::create_dir_all(&models_dir)?;

    let _now = chrono::Utc::now();

    // NEW APPROACH: Process each API key individually instead of grouping by provider
    // Create a map to track configuration context by source file
    let mut source_context: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<DiscoveredCredential>>,
    > = std::collections::HashMap::new();

    // Step 1: We're no longer tracking existing instances to ensure consistent SHA-256 based IDs

    // Process config_instances first to extract ProviderInstance objects with API-probed models
    // Build a map: (provider_type, source_path) -> models
    // This matches how instance IDs are generated during scan: hash("{provider}:{source_path}")
    let mut probed_models_by_source: std::collections::HashMap<
        (String, String), // (provider_type, source_path)
        Vec<String>,      // Model IDs (new API uses Vec<String> not Vec<Model>)
    > = std::collections::HashMap::new();

    tracing::debug!(
        "Processing {} config instances to extract existing ProviderInstances",
        result.config_instances.len()
    );

    // Extract probed models from each config instance
    for config_instance in &result.config_instances {
        let source_path = config_instance.config_path.to_string_lossy().to_string();

        for provider_instance in config_instance.provider_instances.all_instances() {
            if !provider_instance.models.is_empty() {
                let key = (provider_instance.provider_type.clone(), source_path.clone());
                probed_models_by_source.insert(key.clone(), provider_instance.models.clone());

                tracing::debug!(
                    "Stored {} probed models for provider {} from source '{}' (instance {})",
                    provider_instance.models.len(),
                    provider_instance.provider_type,
                    source_path,
                    provider_instance.id
                );
                tracing::debug!(
                    "  Map key: ({}, '{}')",
                    provider_instance.provider_type,
                    source_path
                );
            }
        }
    }

    tracing::debug!(
        "Extracted probed models for {} (provider, source) combinations",
        probed_models_by_source.len()
    );

    // Step 2: Collect all keys by source file first
    for key in &result.keys {
        let source = key.source_file.clone();
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
            let source = key.source_file.clone();
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
            let api_keys: Vec<&DiscoveredCredential> = keys
                .iter()
                .filter(|k| matches!(k.value_type, aicred_core::ValueType::ApiKey))
                .collect();

            // Also check for other key types that can serve as primary keys
            let other_keys: Vec<&DiscoveredCredential> = keys
                .iter()
                .filter(|k| {
                    matches!(
                        k.value_type,
                        aicred_core::ValueType::AccessToken
                            | aicred_core::ValueType::SecretKey
                            | aicred_core::ValueType::BearerToken
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
                // Generate SHA-256 hash of the primary key content for consistent instance ID
                let mut hasher = Sha256::new();
                hasher.update(primary_key.hash.as_bytes());
                let hash_result = hasher.finalize();
                let full_hash = format!("{:x}", hash_result);
                let instance_id = full_hash[..4].to_string();

                // Create a filename using provider name and first 4 chars of hash
                let sanitized_provider = sanitize_provider_name(&provider_name);
                let filename = format!("{}-{}.yaml", sanitized_provider, &instance_id[..4]);
                let instance_file_path = config_dir.join(&filename);

                tracing::debug!(
                    "Creating instance for API key from provider {}: filename={}",
                    provider_name,
                    filename
                );

                // Since we don't have the actual API key (it's redacted), we'll store a placeholder
                // The actual API key will need to be manually configured or the user will need to
                // run the scan with --include-values (which is not recommended for security)
                let api_key_placeholder = format!("REDACTED_{}", &instance_id);

                // Always create new instance with SHA-256 based ID

                // Always create new instance with SHA-256 based ID for consistency
                let mut instance = {
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
                        Vec::new(), // Empty models initially
                    );

                    // Check if we have probed models for this source
                    let source_key = (provider_name.clone(), primary_key.source_file.clone());
                    tracing::debug!(
                        "  Looking up key: ({}, '{}')",
                        provider_name,
                        primary_key.source_file
                    );
                    if let Some(probed_models) = probed_models_by_source.get(&source_key) {
                        tracing::info!(
                            "Found {} probed models for {} (key hash {}), adding to new instance",
                            probed_models.len(),
                            provider_name,
                            &primary_key.hash[..8]
                        );
                        new_instance.models = probed_models.clone();
                    } else {
                        tracing::debug!(
                            "No probed models found for {} (instance {})",
                            provider_name,
                            instance_id
                        );
                    }

                    new_instance
                };

                // Check if we have the API key value (it might be redacted)
                if let Some(api_key_value) = primary_key.full_value() {
                    // Use the actual API key if available
                    let api_key_to_store = api_key_value.to_string();

                    // Extract models, base_url, and metadata from the same source file
                    let mut _base_url: Option<String> = None;
                    let mut models_found = Vec::new();
                    let mut metadata_map = instance.metadata.clone();

                    // Remove base_url and model_id from metadata if they exist (they should be at instance/models level)
                    metadata_map.remove("base_url");
                    metadata_map.remove("baseurl");
                    metadata_map.remove("model_id");
                    metadata_map.remove("modelid");

                    tracing::debug!(
                        "Processing {} keys for model extraction from {}",
                        keys.len(),
                        source_file
                    );

                    // Debug: Show what keys we have
                    for (i, key) in keys.iter().enumerate() {
                        tracing::debug!(
                            "Key {}: value_type={:?}, has_full_value={}",
                            i,
                            key.value_type,
                            key.full_value().is_some()
                        );
                    }

                    for key in &keys {
                        match &key.value_type {
                            aicred_core::ValueType::ModelId => {
                                if let Some(full_value) = key.full_value() {
                                    tracing::debug!("Found ModelId: {}", full_value);
                                    models_found.push(full_value.to_string());
                                } else {
                                    tracing::warn!(
                                        "ModelId key has no full value (redacted?): {}",
                                        key.redacted_value()
                                    );
                                }
                            }
                            aicred_core::ValueType::BaseUrl => {
                                if let Some(full_value) = key.full_value() {
                                    _base_url = Some(full_value.to_string());
                                    instance.base_url = full_value.to_string();
                                    tracing::debug!("Found base_url: {}", full_value);
                                }
                            }
                            aicred_core::ValueType::Temperature => {
                                if let Some(full_value) = key.full_value() {
                                    if let Ok(temp) = full_value.parse::<f32>() {
                                        metadata_map
                                            .insert("temperature".to_string(), temp.to_string());
                                        tracing::debug!("Found temperature: {}", temp);
                                    }
                                }
                            }
                            aicred_core::ValueType::Custom(ref custom_type) => {
                                let custom_type_lower = custom_type.to_lowercase();

                                if custom_type_lower == "baseurl" {
                                    if let Some(full_value) = key.full_value() {
                                        _base_url = Some(full_value.to_string());
                                        instance.base_url = full_value.to_string();
                                        tracing::debug!("Found base_url: {}", full_value);
                                    }
                                } else if custom_type_lower == "modelid" {
                                    if let Some(full_value) = key.full_value() {
                                        tracing::debug!("Found custom modelid: {}", full_value);
                                        models_found.push(full_value.to_string());
                                    }
                                } else {
                                    // Collect all other custom fields as metadata (merge with existing)
                                    if let Some(full_value) = key.full_value() {
                                        tracing::debug!(
                                            "Found custom metadata {}: {}",
                                            custom_type_lower,
                                            full_value
                                        );
                                        metadata_map
                                            .insert(custom_type_lower, full_value.to_string());
                                    }
                                }
                            }
                            _ => {
                                tracing::trace!("Skipping key type: {:?}", key.value_type);
                            }
                        }
                    }

                    tracing::info!(
                        "Extracted {} models from {} keys for provider {}",
                        models_found.len(),
                        keys.len(),
                        provider_name
                    );

                    // Debug: Check if this provider instance already has models from scanning
                    let existing_models = instance.models.len();
                    tracing::info!(
                        "Provider {} already has {} models from scanning, extracted {} more from keys",
                        provider_name,
                        existing_models,
                        models_found.len()
                    );

                    // Debug: Show the actual models that are being probed and found
                    tracing::debug!(
                        "Instance {} ({}): {} models from scanning, {} models from keys",
                        instance_id,
                        provider_name,
                        existing_models,
                        models_found.len()
                    );

                    // Always include existing models from the ProviderInstance
                    // This handles the case where API-probed models exist but weren't converted to DiscoveredCredential objects
                    for existing_model in &instance.models {
                        if !models_found.contains(existing_model) {
                            tracing::debug!(
                                "Adding existing model {} that wasn't found via keys",
                                existing_model
                            );
                            models_found.push(existing_model.clone());
                        }
                    }

                    // Always preserve metadata (even if empty from new scan, keep existing)
                    if !metadata_map.is_empty() {
                        instance.metadata = metadata_map;
                    }

                    tracing::debug!(
                        "About to process {} models for persistence",
                        models_found.len()
                    );

                    // Add found models to the instance - create full models and save to config files
                    // Only add models that don't already exist
                    for model_id in models_found {
                        let model_exists = instance.models.iter().any(|m| m == &model_id);
                        tracing::debug!("Processing model {}: exists={}", model_id, model_exists);

                        if !model_exists {
                            let full_model = create_full_model(&model_id);
                            let _model_ref = save_model_config(&full_model, &models_dir)?;

                            // Add model ID to instance (new API uses Vec<String>)
                            instance.add_model(model_id.clone());
                            tracing::debug!("Added new model {} to instance", model_id);
                        } else {
                            // Model already exists in instance (e.g., from API probing), but we still need to create the config file
                            let full_model = create_full_model(&model_id);
                            let _model_ref = save_model_config(&full_model, &models_dir)?;
                            tracing::debug!(
                                "Model {} already exists in instance, created config file",
                                model_id
                            );
                        }
                    }

                    // Check if this key already exists in the instance
                    let key_exists = instance.get_api_key() == Some(&api_key_to_store);

                    // Only add the key if it doesn't already exist
                    if !key_exists {
                        instance.set_api_key(api_key_to_store.clone());
                    }

                    tracing::debug!(
                        "About to save instance: {} with {} models",
                        instance.id,
                        instance.models.len()
                    );
                    tracing::debug!("Instance models before save: {:?}", instance.models);

                    // Save the instance configuration
                    let yaml_content = serde_yaml::to_string(&instance)?;
                    tracing::debug!(
                        "Generated YAML content length: {} chars",
                        yaml_content.len()
                    );
                    std::fs::write(&instance_file_path, yaml_content)?;

                    tracing::debug!("Saved instance config to: {}", instance_file_path.display());
                } else {
                    // API key is redacted - still create instance but with placeholder
                    // Always create new instance with SHA-256 based ID for consistency
                    let mut instance = {
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
                            Vec::new(), // Empty models initially
                        );

                        // Check if we have probed models for this provider and instance
                        let source_key = (provider_name.clone(), primary_key.source_file.clone());
                        if let Some(probed_models) = probed_models_by_source.get(&source_key) {
                            tracing::info!(
                                "Found {} probed models for {} (instance {}), adding to new instance (redacted key)",
                                probed_models.len(),
                                provider_name,
                                instance_id
                            );
                            new_instance.models = probed_models.clone();
                        } else {
                            tracing::debug!(
                                "No probed models found for {} (instance {}) (redacted key)",
                                provider_name,
                                instance_id
                            );
                        }

                        new_instance
                    };

                    // Extract models, base_url, and metadata from the same source file
                    let mut _base_url: Option<String> = None;
                    let mut models_found = Vec::new();
                    let mut metadata_map = instance.metadata.clone();

                    // Remove base_url and model_id from metadata if they exist (they should be at instance/models level)
                    metadata_map.remove("base_url");
                    metadata_map.remove("baseurl");
                    metadata_map.remove("model_id");
                    metadata_map.remove("modelid");

                    for key in &keys {
                        if let aicred_core::ValueType::Custom(ref custom_type) = key.value_type {
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
                        instance.metadata = metadata_map;
                    }

                    // Add found models to the instance - create full models and save to config files
                    // Only add models that don't already exist
                    for model_id in models_found {
                        let model_exists = instance.models.iter().any(|m| m == &model_id);
                        if !model_exists {
                            let full_model = create_full_model(&model_id);
                            let _model_ref = save_model_config(&full_model, &models_dir)?;

                            // Add model ID to instance (new API uses Vec<String>)
                            instance.add_model(model_id.clone());
                        }
                    }

                    // Check if this key already exists in the instance
                    let key_exists = instance.get_api_key() == Some(&api_key_placeholder);

                    // Only add the key if it doesn't already exist
                    if !key_exists {
                        instance.set_api_key(api_key_placeholder.clone());
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

fn write_audit_log(log_path: &str, result: &aicred_core::ScanResult) -> Result<()> {
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
    writeln!(file, "AICred Audit Log")?;
    writeln!(file, "==================")?;
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
                key.provider, key.value_type, key.source_file, key.confidence
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
