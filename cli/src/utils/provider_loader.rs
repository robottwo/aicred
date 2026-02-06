//! Provider instance loading utilities.

use aicred_core::models::{ProviderCollection, ProviderInstance};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Load provider instances from configuration directory
pub fn load_provider_instances(home: Option<&Path>) -> Result<ProviderCollection> {
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
        return Ok(ProviderCollection::new());
    }

    // Load all instance files
    let mut instances = ProviderCollection::new();

    let entries = std::fs::read_dir(&instances_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "yaml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Parse as ProviderInstance (modern format)
                if let Ok(new_instance) = serde_yaml::from_str::<ProviderInstance>(&content) {
                    let id = new_instance.id.clone();
                    instances.add(id, new_instance);
                    continue;
                }

                // Fallback: try a permissive parse for ad-hoc YAML fixtures
                #[allow(clippy::collapsible_match)]
                if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    if let serde_yaml::Value::Mapping(map) = value {
                        // Helper to extract string fields
                        let get_str = |k: &str| -> Option<String> {
                            map.get(serde_yaml::Value::String(k.to_string()))
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                        };

                        let id = get_str("id").unwrap_or_else(|| {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string()
                        });

                        let provider_type =
                            get_str("provider_type").unwrap_or_else(|| "unknown".to_string());
                        let base_url = get_str("base_url")
                            .unwrap_or_else(|| "https://api.example.com".to_string());

                        let mut instance = ProviderInstance {
                            id: id.clone(),
                            provider_type: provider_type.clone(),
                            base_url,
                            api_key: String::new(),
                            models: Vec::new(),
                            capabilities: Default::default(),
                            active: true,
                            metadata: std::collections::HashMap::new(),
                        };

                        // Active flag
                        if let Some(active_val) =
                            map.get(serde_yaml::Value::String("active".to_string()))
                        {
                            if let Some(b) = active_val.as_bool() {
                                instance.active = b;
                            }
                        }

                        // Extract API key from legacy `keys` sequence if present
                        if let Some(keys_val) =
                            map.get(serde_yaml::Value::String("keys".to_string()))
                        {
                            if let Some(seq) = keys_val.as_sequence() {
                                if !seq.is_empty() {
                                    if let Some(first_key) = seq[0].as_mapping() {
                                        let api_key = first_key
                                            .get(serde_yaml::Value::String("api_key".to_string()))
                                            .or_else(|| {
                                                first_key.get(serde_yaml::Value::String(
                                                    "value".to_string(),
                                                ))
                                            })
                                            .and_then(|v| v.as_str().map(|s| s.to_string()));
                                        if let Some(k) = api_key {
                                            instance.api_key = k;
                                        }
                                    } else if let Some(s) = seq[0].as_str() {
                                        instance.api_key = s.to_string();
                                    }
                                }
                            }
                        }

                        // Extract models: either sequence of strings or sequence of maps with model_id
                        if let Some(models_val) =
                            map.get(serde_yaml::Value::String("models".to_string()))
                        {
                            if let Some(seq) = models_val.as_sequence() {
                                for item in seq {
                                    if let Some(s) = item.as_str() {
                                        instance.add_model(s.to_string());
                                    } else if let Some(m) = item.as_mapping() {
                                        if let Some(model_id_val) =
                                            m.get(serde_yaml::Value::String("model_id".to_string()))
                                        {
                                            if let Some(model_id) = model_id_val.as_str() {
                                                instance.add_model(model_id.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        instances.add(id.clone(), instance);
                        continue;
                    }
                }

                eprintln!(
                    "{} {}: failed to parse as ProviderInstance",
                    "Error parsing instance file:".red(),
                    path.display()
                );
            }
        }
    }

    Ok(instances)
}
