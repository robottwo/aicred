use crate::commands::{get_labels_for_target, get_tags_for_target};
use aicred_core::ScanResult;
use colored::*;
use tracing::debug;

pub fn output_summary(result: &ScanResult, verbose: bool) -> Result<(), anyhow::Error> {
    debug!(
        "Starting summary output with {} config instances",
        result.config_instances.len()
    );

    println!("\n{}", "Scan Summary".green().bold());
    println!("  Home Directory: {}", result.home_directory);
    println!("  Scan Time: {}", result.scan_completed_at);
    println!(
        "  Providers Scanned: {}",
        result.providers_scanned.join(", ")
    );

    let total_provider_instances: usize = result
        .config_instances
        .iter()
        .map(|instance| instance.provider_instances.len())
        .sum();

    println!("\n{}", "Results:".cyan().bold());
    println!("  Configurations Found: {}", total_provider_instances);
    println!("  Application Instances: {}", result.config_instances.len());

    // Group provider instances by type
    let mut by_provider: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for instance in &result.config_instances {
        for provider_instance in instance.provider_instances() {
            *by_provider
                .entry(provider_instance.provider_type.clone())
                .or_insert(0) += 1;
        }
    }

    if !by_provider.is_empty() {
        println!("\n{}", "By Provider:".cyan().bold());
        let mut providers: Vec<_> = by_provider.iter().collect();
        providers.sort_by_key(|(name, _)| *name);
        for (provider, count) in providers {
            println!("  {}: {} configuration(s)", provider, count);
        }
    }

    // Show detailed configuration information if verbose
    if verbose && !result.config_instances.is_empty() {
        println!("\n{}", "Discovered Configurations:".cyan().bold());
        for instance in &result.config_instances {
            for provider_instance in instance.provider_instances() {
                println!(
                    "  - {} ({})",
                    provider_instance.provider_type.cyan(),
                    instance.config_path.display()
                );

                if provider_instance.has_non_empty_api_key() {
                    println!("    API Key: {}", "configured".green());
                }
                if !provider_instance.models.is_empty() {
                    let model_names: Vec<String> = provider_instance
                        .models
                        .iter()
                        .map(|m| m.name.clone())
                        .collect();
                    println!("        Models: {}", model_names.join(", "));

                    // Show tags and labels for each model
                    for model in &provider_instance.models {
                        if let Ok(tags) =
                            get_tags_for_target(&instance.instance_id, Some(&model.name))
                        {
                            if !tags.is_empty() {
                                println!("          {} tags:", model.name);
                                for tag in tags {
                                    let tag_display = if let Some(ref color) = tag.color {
                                        format!("{} ({})", tag.name, color)
                                    } else {
                                        tag.name.clone()
                                    };
                                    println!("            - {}", tag_display);
                                }
                            }
                        }

                        if let Ok(labels) =
                            get_labels_for_target(&instance.instance_id, Some(&model.name))
                        {
                            if !labels.is_empty() {
                                println!("          {} labels:", model.name);
                                for label in labels {
                                    let label_display = if let Some(ref color) = label.color {
                                        format!("{} ({})", label.name, color)
                                    } else {
                                        label.name.clone()
                                    };
                                    println!("            - {}", label_display);
                                }
                            }
                        }
                    }
                }

                // Show tags for this provider instance
                if let Ok(tags) = get_tags_for_target(&instance.instance_id, None) {
                    if !tags.is_empty() {
                        println!("    Tags:");
                        for tag in tags {
                            let tag_display = if let Some(ref color) = tag.color {
                                format!("{} ({})", tag.name, color)
                            } else {
                                tag.name.clone()
                            };
                            println!("      - {}", tag_display);
                        }
                    }
                }

                // Show labels for this provider instance
                if let Ok(labels) = get_labels_for_target(&instance.instance_id, None) {
                    if !labels.is_empty() {
                        println!("    Labels:");
                        for label in labels {
                            let label_display = if let Some(ref color) = label.color {
                                format!("{} ({})", label.name, color)
                            } else {
                                label.name.clone()
                            };
                            println!("      - {}", label_display);
                        }
                    }
                }

                if let Some(metadata) = &provider_instance.metadata {
                    if !metadata.is_empty() {
                        println!("    Settings:");
                        for (key, value) in metadata {
                            println!("      {}: {}", key, value);
                        }
                    }
                }
            }
        }
    }

    // Show detailed application instances if verbose
    if verbose && !result.config_instances.is_empty() {
        println!("\n{}", "Application Instances:".cyan().bold());
        for instance in &result.config_instances {
            println!(
                "  - {}: {}",
                instance.app_name.cyan(),
                instance.config_path.display()
            );

            // Show provider instances
            let provider_instances = instance.provider_instances();
            if !provider_instances.is_empty() {
                println!("    Configured Providers:");
                for provider_instance in provider_instances {
                    println!(
                        "      - {} ({})",
                        provider_instance.display_name, provider_instance.provider_type
                    );
                    if !provider_instance.models.is_empty() {
                        let model_names: Vec<String> = provider_instance
                            .models
                            .iter()
                            .map(|m| m.name.clone())
                            .collect();
                        println!("        Models: {}", model_names.join(", "));
                    }
                }
            }
        }
    }

    Ok(())
}
