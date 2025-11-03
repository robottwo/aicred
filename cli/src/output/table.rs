use aicred_core::ScanResult;
use colored::*;
use tracing::debug;

pub fn output_table(result: &ScanResult, verbose: bool) -> Result<(), anyhow::Error> {
    debug!(
        "Starting table output with {} config instances",
        result.config_instances.len()
    );

    if !result.config_instances.is_empty() {
        println!(
            "\n{}",
            "=== Discovered AI Configurations ===".green().bold()
        );

        if verbose {
            // Verbose mode: show settings column
            println!(
                "{:<15} {:<40} {:<25} {:<30}",
                "Provider".bold(),
                "Source".bold(),
                "Models".bold(),
                "Settings".bold()
            );
            println!("{}", "-".repeat(115));

            for instance in &result.config_instances {
                for provider_instance in instance.provider_instances() {
                    let models_display = if provider_instance.models.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let model_names: Vec<String> = provider_instance
                            .models
                            .iter()
                            .map(|m| m.name.clone())
                            .collect();
                        truncate_string(&model_names.join(", "), 25)
                    };

                    let settings_display = if let Some(metadata) = &provider_instance.metadata {
                        if metadata.is_empty() {
                            "-".dimmed().to_string()
                        } else {
                            let settings_str = metadata
                                .iter()
                                .map(|(k, v)| format!("{}={}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ");
                            truncate_string(&settings_str, 30)
                        }
                    } else {
                        "-".dimmed().to_string()
                    };

                    println!(
                        "{:<15} {:<40} {:<25} {:<30}",
                        provider_instance.provider_type.cyan(),
                        truncate_path(&instance.config_path.display().to_string(), 40),
                        models_display,
                        settings_display
                    );

                    // Show API key if verbose and available
                    if let Some(api_key) = provider_instance.get_api_key() {
                        if !api_key.is_empty() {
                            println!("  API Key: {}", "********".yellow());
                        }
                    }
                    if let Some(metadata) = &provider_instance.metadata {
                        if !metadata.is_empty() {
                            println!("  Settings:");
                            for (key, value) in metadata {
                                println!("    {}: {}", key.dimmed(), value);
                            }
                        }
                    }
                }
            }
        } else {
            // Normal mode: hide settings column
            println!(
                "{:<15} {:<40} {:<35}",
                "Provider".bold(),
                "Source".bold(),
                "Models".bold()
            );
            println!("{}", "-".repeat(95));

            for instance in &result.config_instances {
                for provider_instance in instance.provider_instances() {
                    let models_display = if provider_instance.models.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let model_names: Vec<String> = provider_instance
                            .models
                            .iter()
                            .map(|m| m.name.clone())
                            .collect();
                        truncate_string(&model_names.join(", "), 35)
                    };

                    println!(
                        "{:<15} {:<40} {:<35}",
                        provider_instance.provider_type.cyan(),
                        truncate_path(&instance.config_path.display().to_string(), 40),
                        models_display
                    );
                }
            }
        }
    }

    // Show config instances summary
    if !result.config_instances.is_empty() {
        println!("\n{}", "=== Application Instances ===".green().bold());
        println!(
            "{:<20} {:<10} {:<12} {:<48}",
            "Application".bold(),
            "Providers".bold(),
            "Models".bold(),
            "Path".bold()
        );
        println!("{}", "-".repeat(95));

        for instance in &result.config_instances {
            // Count unique providers and models from the keys associated with this instance
            let mut providers = std::collections::HashSet::new();
            let mut models = std::collections::HashSet::new();

            // Get all keys from the main result that match this instance's source
            let instance_path = instance.config_path.display().to_string();
            for key in &result.keys {
                if key.source == instance_path {
                    providers.insert(key.provider.clone());

                    // Count models (ModelId value type)
                    if matches!(key.value_type, aicred_core::ValueType::ModelId) {
                        if let Some(model_id) = key.full_value() {
                            models.insert(model_id.to_string());
                        }
                    }
                }
            }

            let provider_count = providers.len();
            let model_count = models.len();

            println!(
                "{:<20} {:<10} {:<12} {:<48}",
                instance.app_name.cyan(),
                provider_count,
                model_count,
                truncate_path(&instance.config_path.display().to_string(), 48)
            );

            // Show provider instances if verbose
            if verbose && !instance.provider_instances.is_empty() {
                println!("  Providers configured:");
                for provider_instance in instance.provider_instances() {
                    println!(
                        "    - {} ({})",
                        provider_instance.display_name, provider_instance.provider_type
                    );
                    if !provider_instance.models.is_empty() {
                        let model_names: Vec<String> = provider_instance
                            .models
                            .iter()
                            .map(|m| m.name.clone())
                            .collect();
                        println!("      Models: {}", model_names.join(", "));
                    }
                }
            }
        }
    }

    let total_provider_instances: usize = result
        .config_instances
        .iter()
        .map(|instance| instance.provider_instances.len())
        .sum();

    println!(
        "\n{}",
        format!(
            "Total: {} configurations, {} application instances",
            total_provider_instances,
            result.config_instances.len()
        )
        .cyan()
    );

    Ok(())
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.chars().count() <= max_len {
        return path.to_string();
    }
    let tail: String = path.chars().rev().take(max_len.saturating_sub(3)).collect();
    let tail = tail.chars().rev().collect::<String>();
    format!("...{}", tail)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
    format!("{}...", truncated)
}
