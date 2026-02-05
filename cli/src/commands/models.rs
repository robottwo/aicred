//! Model management commands for the AICred CLI.
//!
//! Provides commands to list, search, and display information about
//! available AI models from the model registry.

use crate::output::utils::format_capabilities;
use aicred_core::models::model_registry::{CapabilityFilter, ModelEntry, ModelRegistry};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// List all models from the registry.
pub fn handle_list_registry_models(
    provider: Option<String>,
    capability: Option<String>,
    search: Option<String>,
    verbose: bool,
) -> Result<()> {
    let registry = ModelRegistry::new();

    let mut models: Vec<&ModelEntry> = registry.all();

    // Apply filters
    if let Some(provider_name) = provider {
        models = models
            .into_iter()
            .filter(|m| m.provider == provider_name)
            .collect();
    }

    if let Some(cap_str) = capability {
        let cap_filter = match cap_str.to_lowercase().as_str() {
            "text" => CapabilityFilter::Text,
            "image" => CapabilityFilter::Image,
            "vision" => CapabilityFilter::Vision,
            "code" => CapabilityFilter::Code,
            "function" | "function-calling" => CapabilityFilter::FunctionCalling,
            "streaming" => CapabilityFilter::Streaming,
            "json" | "json-mode" => CapabilityFilter::JsonMode,
            _ => {
                eprintln!("{}", "Unknown capability. Valid options: text, image, vision, code, function, streaming, json".red());
                return Ok(());
            }
        };
        models = registry.by_capability(cap_filter);
    }

    if let Some(query) = search {
        models = models
            .into_iter()
            .filter(|m| {
                m.id.to_lowercase().contains(&query.to_lowercase())
                    || m.name.to_lowercase().contains(&query.to_lowercase())
            })
            .collect();
    }

    if models.is_empty() {
        println!("{}", "No models found matching your criteria.".yellow());
        return Ok(());
    }

    // Group by provider for display
    let mut by_provider: HashMap<&str, Vec<&ModelEntry>> = HashMap::new();
    for model in &models {
        by_provider.entry(&model.provider).or_default().push(model);
    }

    println!("\n{}", "Available Models".green().bold());
    println!("{}", "════════════════".green());

    for (provider, provider_models) in by_provider.iter().collect::<Vec<_>>() {
        println!("\n{} {}", "Provider:".cyan(), provider.cyan().bold());

        for model in provider_models {
            if verbose {
                println_model_detailed(model);
            } else {
                println_model_compact(model);
            }
        }
    }

    println!(
        "\n{} {} models total",
        "Total:".green(),
        models.len().to_string().white()
    );

    Ok(())
}

/// Show detailed information about a specific model.
pub fn handle_get_model(model_id: &str) -> Result<()> {
    let registry = ModelRegistry::new();

    match registry.get(model_id) {
        Some(model) => {
            println_model_detailed(model);
            Ok(())
        }
        None => {
            eprintln!("{} {}", "Model not found:".red(), model_id.red().bold());
            eprintln!(
                "{}",
                "Use 'aicred models list' to see all available models.".dimmed()
            );
            Err(anyhow::anyhow!("Model {} not found", model_id))
        }
    }
}

/// Compare pricing between models.
pub fn handle_compare_models(model_ids: Vec<String>) -> Result<()> {
    let registry = ModelRegistry::new();

    if model_ids.is_empty() {
        eprintln!("{}", "No models specified for comparison.".red());
        eprintln!(
            "{}",
            "Usage: aicred models compare <model-id-1> <model-id-2> ...".dimmed()
        );
        return Ok(());
    }

    println!("\n{}", "Pricing Comparison".green().bold());
    println!("{}", "════════════════════".green());

    // Print header
    println!(
        "{:<40} {:>15} {:>15} {:>20}",
        "Model", "Input ($/1K)", "Output ($/1K)", "Context (tokens)"
    );
    println!("{:-<40} {:->15} {:->15} {:->20}", "", "", "", "");

    for model_id in &model_ids {
        if let Some(model) = registry.get(model_id) {
            let input_cost = (model.pricing.input * 1000.0).to_string();
            let output_cost = (model.pricing.output * 1000.0).to_string();
            let context = model.context_length.to_string();

            println!(
                "{:<40} {:>15} {:>15} {:>20}",
                model.name.white(),
                input_cost.yellow(),
                output_cost.yellow(),
                context.cyan()
            );
        } else {
            println!(
                "{:<40} {:>15} {:>15} {:>20}",
                format!("{} (not found)", model_id).red(),
                "-".dimmed(),
                "-".dimmed(),
                "-".dimmed()
            );
        }
    }

    Ok(())
}

/// List models with specific capabilities.
pub fn handle_list_by_capability(capability: &str) -> Result<()> {
    let cap_filter = match capability.to_lowercase().as_str() {
        "text" => CapabilityFilter::Text,
        "image" => CapabilityFilter::Image,
        "vision" => CapabilityFilter::Vision,
        "code" => CapabilityFilter::Code,
        "function" | "function-calling" => CapabilityFilter::FunctionCalling,
        "streaming" => CapabilityFilter::Streaming,
        "json" | "json-mode" => CapabilityFilter::JsonMode,
        _ => {
            eprintln!("{}", "Unknown capability. Valid options: text, image, vision, code, function, streaming, json".red());
            return Ok(());
        }
    };

    let registry = ModelRegistry::new();
    let models = registry.by_capability(cap_filter);

    if models.is_empty() {
        println!("{}", "No models found with this capability.".yellow());
        return Ok(());
    }

    println!(
        "\n{} models with {} capability",
        models.len().to_string().white(),
        capability.cyan()
    );
    println!("{}", "════════════════════════════════════════".green());

    for model in models {
        println_model_compact(model);
    }

    Ok(())
}

/// Get model statistics and summary.
pub fn handle_model_stats() -> Result<()> {
    let registry = ModelRegistry::new();
    let all_models = registry.all();

    println!("\n{}", "Model Registry Statistics".green().bold());
    println!("{}", "═════════════════════════════".green());

    println!(
        "\n{} {}",
        "Total models:".white(),
        all_models.len().to_string().cyan()
    );

    // Count by provider
    let mut provider_counts: HashMap<&str, usize> = HashMap::new();
    for model in &all_models {
        *provider_counts.entry(&model.provider).or_insert(0) += 1;
    }

    println!("\n{}", "By Provider:".cyan().bold());
    let mut providers: Vec<_> = provider_counts.into_iter().collect();
    providers.sort_by(|a, b| b.1.cmp(&a.1));

    for (provider, count) in providers {
        println!(
            "  {:<20} {} models",
            provider.white(),
            count.to_string().yellow()
        );
    }

    // Count by capability
    let mut cap_counts: HashMap<&str, usize> = HashMap::new();
    cap_counts.insert("text", registry.by_capability(CapabilityFilter::Text).len());
    cap_counts.insert("code", registry.by_capability(CapabilityFilter::Code).len());
    cap_counts.insert(
        "vision",
        registry.by_capability(CapabilityFilter::Vision).len(),
    );
    cap_counts.insert(
        "function",
        registry
            .by_capability(CapabilityFilter::FunctionCalling)
            .len(),
    );
    cap_counts.insert(
        "streaming",
        registry.by_capability(CapabilityFilter::Streaming).len(),
    );

    println!("\n{}", "By Capability:".cyan().bold());
    let mut caps: Vec<_> = cap_counts.into_iter().collect();
    caps.sort_by(|a, b| b.1.cmp(&a.1));

    for (cap, count) in caps {
        println!(
            "  {:<20} {} models",
            cap.white(),
            count.to_string().yellow()
        );
    }

    // Context length ranges
    let contexts: Vec<_> = all_models.iter().map(|m| m.context_length).collect();
    let max_context = contexts.iter().max();
    let avg_context = contexts.iter().sum::<u32>() as f64 / contexts.len() as f64;

    println!("\n{}", "Context Window:".cyan().bold());
    println!(
        "  {:<20} {} tokens",
        "Maximum:".white(),
        max_context.unwrap_or(&0).to_string().yellow()
    );
    println!("  {:<20} {:.0} tokens", "Average:".white(), avg_context);

    Ok(())
}

/// Print model in compact format.
fn println_model_compact(model: &ModelEntry) {
    let status = match model.status {
        aicred_core::models::model_registry::ModelStatus::Active => "✓".green(),
        aicred_core::models::model_registry::ModelStatus::Beta => "β".yellow(),
        _ => "○".dimmed(),
    };

    let caps = format_capabilities(&model.capabilities, false);
    println!(
        "  {} {:<30} {:<40} {}",
        status,
        model.id.white(),
        model.name.dimmed(),
        caps.dimmed()
    );
}

/// Print model in detailed format.
fn println_model_detailed(model: &ModelEntry) {
    let status = match model.status {
        aicred_core::models::model_registry::ModelStatus::Active => "Active".green(),
        aicred_core::models::model_registry::ModelStatus::Beta => "Beta".yellow(),
        aicred_core::models::model_registry::ModelStatus::Deprecated => "Deprecated".red(),
        aicred_core::models::model_registry::ModelStatus::Archived => "Archived".dimmed(),
    };

    println!("\n{}", "═".repeat(70).cyan());
    println!(
        "{} {}",
        model.name.white().bold(),
        format!("({})", model.id).dimmed()
    );
    println!("{}", "═".repeat(70).cyan());

    if let Some(desc) = &model.description {
        println!("\n{}\n{}", "Description:".cyan(), desc.white());
    }

    println!("\n{}", "Basic Information:".cyan().bold());
    println!("  {:<20} {}", "ID:", model.id.white());
    println!("  {:<20} {}", "Provider:", model.provider.cyan());
    println!(
        "  {:<20} {}",
        "Family:",
        model.family.as_deref().unwrap_or("-").white()
    );
    println!("  {:<20} {}", "Status:", status);
    println!(
        "  {:<20} {}",
        "Released:",
        model.released.as_deref().unwrap_or("-").white()
    );

    println!("\n{}", "Capabilities:".cyan().bold());
    println!(
        "  {:<20} {}",
        "Text:",
        bool_indicator(model.capabilities.text)
    );
    println!(
        "  {:<20} {}",
        "Vision:",
        bool_indicator(model.capabilities.vision)
    );
    println!(
        "  {:<20} {}",
        "Code:",
        bool_indicator(model.capabilities.code)
    );
    println!(
        "  {:<20} {}",
        "Function Calling:",
        bool_indicator(model.capabilities.function_calling)
    );
    println!(
        "  {:<20} {}",
        "Streaming:",
        bool_indicator(model.capabilities.streaming)
    );
    println!(
        "  {:<20} {}",
        "JSON Mode:",
        bool_indicator(model.capabilities.json_mode)
    );
    println!(
        "  {:<20} {}",
        "Audio In:",
        bool_indicator(model.capabilities.audio_in)
    );
    println!(
        "  {:<20} {}",
        "Audio Out:",
        bool_indicator(model.capabilities.audio_out)
    );

    println!("\n{}", "Architecture:".cyan().bold());
    println!("  {:<20} {}", "Modality:", model.architecture.modality.white());
    println!(
        "  {:<20} {}",
        "Parameters:",
        model
            .architecture
            .parameters
            .as_deref()
            .unwrap_or("-")
            .white()
    );
    println!(
        "  {:<20} {}",
        "Tokenizer:",
        model.architecture.tokenizer.cyan()
    );
    println!(
        "  {:<20} {}",
        "Instruct Format:",
        model
            .architecture
            .instruct_type
            .as_deref()
            .unwrap_or("-")
            .white()
    );

    println!("\n{}", "Pricing:".cyan().bold());
    println!(
        "  {:<20} ${}/token",
        "Input:",
        model.pricing.input.to_string().yellow()
    );
    println!(
        "  {:<20} ${}/token",
        "Output:",
        model.pricing.output.to_string().yellow()
    );
    if let Some(cached) = model.pricing.cached_input {
        println!(
            "  {:<20} {:.0}% discount",
            "Cached Input:",
            (cached * 100.0).yellow()
        );
    }
    println!(
        "  {:<20} {} tokens",
        "Context Window:",
        model.context_length.to_string().cyan()
    );

    // Example costs
    println!("\n{}", "Example Costs:".cyan().bold());
    let prompt_tokens = 1000u32;
    let completion_tokens = 500u32;
    let input_cost = prompt_tokens as f64 * model.pricing.input;
    let output_cost = completion_tokens as f64 * model.pricing.output;
    let total = input_cost + output_cost;
    println!(
        "  1K prompt + 500 completion tokens: ${:.6}",
        total.yellow()
    );
}

fn bool_indicator(value: bool) -> colored::ColoredString {
    if value {
        "✓".green()
    } else {
        "-".dimmed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_capabilities_empty() {
        use aicred_core::models::model_registry::ModelCapabilities;
        let caps = ModelCapabilities {
            text: false,
            image: false,
            vision: false,
            code: false,
            function_calling: false,
            streaming: false,
            json_mode: false,
            system_prompt: false,
            audio_in: false,
            audio_out: false,
        };
        let result = format_capabilities(&caps, true);
        assert!(result.is_empty() || result.contains("none"));
    }

    #[test]
    fn test_format_capabilities_full() {
        use aicred_core::models::model_registry::ModelCapabilities;
        let caps = ModelCapabilities {
            text: true,
            image: true,
            vision: true,
            code: true,
            function_calling: true,
            streaming: true,
            json_mode: true,
            system_prompt: true,
            audio_in: true,
            audio_out: true,
        };
        let result = format_capabilities(&caps, true);
        assert!(result.contains("text") || result.contains("code"));
    }
}
