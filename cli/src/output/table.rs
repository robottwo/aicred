use aicred_core::{models::Label, ScanResult};
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
            // Verbose mode: show settings and tags/labels columns
            println!(
                "{:<15} {:<40} {:<25} {:<20} {:<15} {:<20}",
                "Provider".bold(),
                "Source".bold(),
                "Models".bold(),
                "Tags".bold(),
                "Labels".bold(),
                "Settings".bold()
            );
            println!("{}", "-".repeat(140));

            for instance in &result.config_instances {
                for provider_instance in instance.provider_instances() {
                    let models_display = if provider_instance.models.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        truncate_string(&provider_instance.models.join(", "), 25)
                    };

                    // Get tags and labels for this instance
                    let tags = get_tags_for_instance(&provider_instance.id)?;
                    let labels = get_labels_for_instance(&provider_instance.id)?;

                    let tags_display = if tags.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
                        truncate_string(&tag_names.join(", "), 20)
                    };

                    let labels_display = if labels.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let label_names: Vec<String> =
                            labels.iter().map(|l| l.name.clone()).collect();
                        truncate_string(&label_names.join(", "), 15)
                    };

                    let settings_display = if provider_instance.metadata.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let settings_str = provider_instance
                            .metadata
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ");
                        truncate_string(&settings_str, 20)
                    };

                    println!(
                        "{:<15} {:<40} {:<25} {:<20} {:<15} {:<20}",
                        provider_instance.provider_type.cyan(),
                        truncate_path(&instance.config_path.display().to_string(), 40),
                        models_display,
                        tags_display,
                        labels_display,
                        settings_display
                    );

                    // Show API key if verbose and available
                    if let Some(api_key) = provider_instance.get_api_key() {
                        if !api_key.is_empty() {
                            println!("  API Key: {}", "********".yellow());
                        }
                    }

                    // Show tags and labels details if verbose
                    if !tags.is_empty() {
                        println!("  Tags:");
                        for tag in &tags {
                            println!("    {}", tag.name);
                        }
                    }

                    if !labels.is_empty() {
                        println!("  Labels:");
                        for label in &labels {
                            println!("    {}", label.name);
                        }
                    }

                    if !provider_instance.metadata.is_empty() {
                        println!("  Settings:");
                        for (key, value) in &provider_instance.metadata {
                            println!("    {}: {}", key.dimmed(), value);
                        }
                    }
                }
            }
        } else {
            // Normal mode: show tags and labels columns
            println!(
                "{:<15} {:<40} {:<25} {:<20} {:<15}",
                "Provider".bold(),
                "Source".bold(),
                "Models".bold(),
                "Tags".bold(),
                "Labels".bold()
            );
            println!("{}", "-".repeat(120));

            for instance in &result.config_instances {
                for provider_instance in instance.provider_instances() {
                    let models_display = if provider_instance.models.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        truncate_string(&provider_instance.models.join(", "), 25)
                    };

                    // Get tags and labels for this instance
                    let tags = get_tags_for_instance(&provider_instance.id)?;
                    let labels = get_labels_for_instance(&provider_instance.id)?;

                    let tags_display = if tags.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
                        truncate_string(&tag_names.join(", "), 20)
                    };

                    let labels_display = if labels.is_empty() {
                        "-".dimmed().to_string()
                    } else {
                        let label_names: Vec<String> =
                            labels.iter().map(|l| l.name.clone()).collect();
                        truncate_string(&label_names.join(", "), 15)
                    };

                    println!(
                        "{:<15} {:<40} {:<25} {:<20} {:<15}",
                        provider_instance.provider_type.cyan(),
                        truncate_path(&instance.config_path.display().to_string(), 40),
                        models_display,
                        tags_display,
                        labels_display
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
            // Count unique providers and models from the provider instances
            let mut providers = std::collections::HashSet::new();
            let mut models = std::collections::HashSet::new();

            // Count from actual provider instances
            for provider_instance in instance.provider_instances() {
                providers.insert(provider_instance.provider_type.clone());

                // Count all models configured in this provider instance
                for model in &provider_instance.models {
                    models.insert(model.clone());
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
                        provider_instance.id, provider_instance.provider_type
                    );
                    if !provider_instance.models.is_empty() {
                        println!("      Models: {}", provider_instance.models.join(", "));
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

/// Get tags for a specific instance
fn get_tags_for_instance(instance_id: &str) -> Result<Vec<Label>, anyhow::Error> {
    use crate::commands::tags::get_tags_for_target;
    get_tags_for_target(instance_id, None, None)
}

/// Get labels for a specific instance
fn get_labels_for_instance(
    instance_id: &str,
) -> Result<Vec<aicred_core::models::Label>, anyhow::Error> {
    use crate::commands::labels::get_labels_for_target;
    get_labels_for_target(instance_id, None, None)
}
