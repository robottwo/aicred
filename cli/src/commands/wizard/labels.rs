//! Labels phase - set up semantic model labels

use anyhow::{Context, Result};
use aicred_core::ProviderInstance;
use console::style;
use inquire::{Confirm, Select};
use std::collections::HashMap;

use super::WizardOptions;
use super::ui;

/// Run the labels phase
pub fn run_labels_phase(
    instances: &[ProviderInstance],
    options: &WizardOptions,
) -> Result<HashMap<String, (String, String)>> {
    // Skip if no instances
    if instances.is_empty() {
        return Ok(HashMap::new());
    }
    
    ui::section_header("Set Default Models (Optional)");
    
    println!("AICred's label system lets you assign semantic names to");
    println!("provider:model combinations. This makes it easy to switch");
    println!("between 'fast' and 'smart' models across your tools.");
    println!();
    println!("{}", style("Examples:").bold());
    println!("  {} 'fast' → groq:llama3-70b-8192 (cheap, quick responses)", style("•").dim());
    println!("  {} 'smart' → openai:gpt-4 (high quality, reasoning)", style("•").dim());
    println!();
    
    if options.auto_accept {
        println!("{} Skipping label setup in auto-accept mode", style("⊘").yellow());
        return Ok(HashMap::new());
    }
    
    let should_setup = Confirm::new("Would you like to set up default labels?")
        .with_default(true)
        .prompt()
        .unwrap_or(false);
    
    if !should_setup {
        return Ok(HashMap::new());
    }
    
    let mut labels = HashMap::new();
    
    // Set up "fast" label
    if let Some((instance_id, model_id)) = setup_label("fast", "for quick, cheap tasks", instances)? {
        labels.insert("fast".to_string(), (instance_id, model_id));
    }
    
    // Set up "smart" label
    if let Some((instance_id, model_id)) = setup_label("smart", "for high-quality tasks", instances)? {
        labels.insert("smart".to_string(), (instance_id, model_id));
    }
    
    println!();
    if labels.is_empty() {
        println!("{} No labels configured", style("⊘").yellow());
    } else {
        println!("{} Labels configured:", style("✓").green());
        for (label, (instance, model)) in &labels {
            println!("  {} {} → {}:{}", 
                style("•").cyan(),
                style(label).cyan().bold(),
                instance,
                model
            );
        }
    }
    println!();
    
    Ok(labels)
}

/// Set up a single label
fn setup_label(
    label_name: &str,
    description: &str,
    instances: &[ProviderInstance],
) -> Result<Option<(String, String)>> {
    println!();
    println!("{}", style("─".repeat(60)).dim());
    println!();
    println!("{} Label: {} ({})", 
        style("→").cyan(),
        style(label_name).cyan().bold(),
        description
    );
    println!();
    
    // Build list of available models
    let mut model_options = Vec::new();
    let mut option_map = Vec::new();
    
    for instance in instances {
        for model in &instance.models {
            let display = format!("{}:{}", instance.id, model);
            let provider_note = match instance.provider_type.as_str() {
                "groq" => " (fast, free)",
                "openai" => " (high quality)",
                "anthropic" => " (high quality)",
                _ => "",
            };
            
            model_options.push(format!("{}{}", display, style(provider_note).dim()));
            option_map.push((instance.id.clone(), model.clone()));
        }
    }
    
    if model_options.is_empty() {
        ui::show_warning("No models available to assign");
        return Ok(None);
    }
    
    let selection = Select::new("Select:", model_options.clone())
        .prompt()
        .context("Failed to get model selection")?;
    
    // Find the corresponding instance and model
    let selected_index = model_options.iter().position(|o| o == &selection).unwrap();
    let (instance_id, model_id) = option_map[selected_index].clone();
    
    Ok(Some((instance_id, model_id)))
}
