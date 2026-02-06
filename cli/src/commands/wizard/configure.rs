//! Configure phase - set up provider instances

use anyhow::{Context, Result};
use aicred_core::{
    CredentialValue, DiscoveredCredential, ProviderInstance, ScanResult,
};
use console::style;
use inquire::{Confirm, Text};
use inquire::validator::Validation;
use std::collections::HashMap;

use super::WizardOptions;
use super::ui;

// Provider base URLs
const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const HUGGINGFACE_BASE_URL: &str = "https://api-inference.huggingface.co";
const OLLAMA_BASE_URL: &str = "http://localhost:11434";
const LITELLM_BASE_URL: &str = "http://localhost:4000";

/// Run the configure phase
pub fn run_configure_phase(
    selected_credentials: &[DiscoveredCredential],
    scan_result: &ScanResult,
    options: &WizardOptions,
) -> Result<Vec<ProviderInstance>> {
    // If no credentials selected, offer manual setup
    if selected_credentials.is_empty() {
        return run_manual_setup(options);
    }
    
    ui::section_header("Configure Provider Instances");
    
    let mut instances = Vec::new();
    
    // Group credentials by provider
    let mut by_provider: HashMap<String, Vec<&DiscoveredCredential>> = HashMap::new();
    for cred in selected_credentials {
        by_provider
            .entry(cred.provider.clone())
            .or_default()
            .push(cred);
    }
    
    let total_providers = by_provider.len();
    
    for (index, (provider_type, creds)) in by_provider.iter().enumerate() {
        println!(
            "\n{} Configuring {} ({}/{})",
            style("→").cyan(),
            style(provider_type).cyan().bold(),
            index + 1,
            total_providers
        );
        println!("{}", style("─".repeat(60)).dim());
        println!();
        
        // Configure each credential for this provider
        for (cred_index, cred) in creds.iter().enumerate() {
            let instance_id_default = if creds.len() == 1 {
                format!("my-{}", provider_type.to_lowercase())
            } else {
                format!("{}-{}", provider_type.to_lowercase(), cred_index + 1)
            };
            
            let instance = configure_instance(
                cred,
                &instance_id_default,
                provider_type,
                scan_result,
                options,
            )?;
            
            instances.push(instance);
        }
    }
    
    println!();
    println!(
        "{} Configured {} provider instances",
        style("✓").green(),
        style(instances.len()).cyan().bold()
    );
    println!();
    
    Ok(instances)
}

/// Configure a single provider instance
fn configure_instance(
    cred: &DiscoveredCredential,
    default_id: &str,
    provider_type: &str,
    _scan_result: &ScanResult,
    options: &WizardOptions,
) -> Result<ProviderInstance> {
    // Get instance ID
    let instance_id = if options.auto_accept {
        default_id.to_string()
    } else {
        Text::new("Instance ID:")
            .with_default(default_id)
            .with_help_message("Used to reference this provider in commands")
            .with_validator(validate_instance_id)
            .prompt()
            .context("Failed to get instance ID")?
    };
    
    // Get display name
    let display_name = if options.auto_accept {
        format!("{} (Personal)", provider_type)
    } else {
        Text::new("Display name:")
            .with_default(&format!("{} (Personal)", provider_type))
            .prompt()
            .context("Failed to get display name")?
    };
    
    // Get base URL - use provider defaults
    let base_url = get_default_base_url(provider_type);
    
    if !options.auto_accept {
        println!("  {} Base URL: {}", style("→").dim(), style(&base_url).cyan());
    }
    
    // Get models - use provider defaults
    // TODO: Implement actual model probing when API clients are available
    let models = get_default_models(provider_type);
    
    if !options.auto_accept && !models.is_empty() {
        println!(
            "  {} Using default models: {}",
            style("→").dim(),
            models.join(", ")
        );
    }
    
    // Mark as active
    let is_active = if options.auto_accept {
        true
    } else {
        Confirm::new("Mark as active?")
            .with_default(true)
            .prompt()
            .unwrap_or(true)
    };
    
    // Get the actual credential value
    let api_key_value = match &cred.value {
        CredentialValue::Full(v) => v.clone(),
        CredentialValue::Redacted { .. } => {
            // This shouldn't happen in wizard mode since we set include_full_values to true
            return Err(anyhow::anyhow!("Expected full credential value, got redacted"));
        }
    };
    
    // Create the instance
    let mut instance = ProviderInstance::new(
        instance_id.clone(),
        provider_type.to_string(),
        base_url,
        api_key_value,
        models,
    );
    
    instance.active = is_active;
    
    // Set metadata
    instance.metadata.insert("display_name".to_string(), display_name);
    
    println!("  {} Created instance: {}", style("✓").green(), style(&instance_id).cyan());
    
    Ok(instance)
}

/// Run manual setup if no credentials were found/selected
fn run_manual_setup(options: &WizardOptions) -> Result<Vec<ProviderInstance>> {
    if options.auto_accept {
        // In auto-accept mode, just return empty
        return Ok(Vec::new());
    }
    
    println!("{}", style("No credentials selected for import.").yellow());
    println!();
    
    let should_add = Confirm::new("Would you like to manually add a provider?")
        .with_default(false)
        .prompt()
        .unwrap_or(false);
    
    if !should_add {
        return Ok(Vec::new());
    }
    
    let mut instances = Vec::new();
    
    loop {
        let instance = manual_add_provider()?;
        instances.push(instance);
        
        let add_another = Confirm::new("Add another provider?")
            .with_default(false)
            .prompt()
            .unwrap_or(false);
        
        if !add_another {
            break;
        }
    }
    
    Ok(instances)
}

/// Manually add a single provider
fn manual_add_provider() -> Result<ProviderInstance> {
    println!();
    
    let provider_types = vec![
        "openai",
        "anthropic",
        "groq",
        "openrouter",
        "huggingface",
        "ollama",
        "litellm",
    ];
    
    let provider_type = inquire::Select::new("Provider type:", provider_types)
        .prompt()
        .context("Failed to get provider type")?
        .to_string();
    
    let instance_id = Text::new("Instance ID:")
        .with_default(&format!("my-{}", provider_type))
        .with_validator(validate_instance_id)
        .prompt()
        .context("Failed to get instance ID")?;
    
    let display_name = Text::new("Display name:")
        .with_default(&format!("{} (Personal)", provider_type))
        .prompt()
        .context("Failed to get display name")?;
    
    let base_url = get_default_base_url(&provider_type);
    
    let api_key = inquire::Password::new("API Key:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .context("Failed to get API key")?;
    
    let models = get_default_models(&provider_type);
    
    let mut instance = ProviderInstance::new(
        instance_id.clone(),
        provider_type,
        base_url,
        api_key,
        models,
    );
    
    instance.active = true;
    instance.metadata.insert("display_name".to_string(), display_name);
    
    println!("  {} Created instance: {}", style("✓").green(), style(&instance_id).cyan());
    
    Ok(instance)
}

/// Validate instance ID format
fn validate_instance_id(input: &str) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if input.is_empty() {
        return Ok(Validation::Invalid("Instance ID cannot be empty".into()));
    }
    
    if input.contains(' ') {
        return Ok(Validation::Invalid("Instance IDs cannot contain spaces. Use hyphens or underscores.".into()));
    }
    
    if !input.chars().next().unwrap().is_alphabetic() {
        return Ok(Validation::Invalid("Instance IDs must start with a letter.".into()));
    }
    
    Ok(Validation::Valid)
}

/// Get default base URL for a provider
fn get_default_base_url(provider_type: &str) -> String {
    match provider_type.to_lowercase().as_str() {
        "openai" => OPENAI_BASE_URL.to_string(),
        "anthropic" => ANTHROPIC_BASE_URL.to_string(),
        "groq" => GROQ_BASE_URL.to_string(),
        "openrouter" => OPENROUTER_BASE_URL.to_string(),
        "huggingface" => HUGGINGFACE_BASE_URL.to_string(),
        "ollama" => OLLAMA_BASE_URL.to_string(),
        "litellm" => LITELLM_BASE_URL.to_string(),
        _ => String::new(),
    }
}

/// Get default/common models for a provider
fn get_default_models(provider_type: &str) -> Vec<String> {
    match provider_type.to_lowercase().as_str() {
        "openai" => vec![
            "gpt-4o".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ],
        "anthropic" => vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ],
        "groq" => vec![
            "llama3-70b-8192".to_string(),
            "mixtral-8x7b-32768".to_string(),
        ],
        "openrouter" => vec![
            "openai/gpt-4".to_string(),
            "anthropic/claude-3-opus".to_string(),
        ],
        "huggingface" => Vec::new(),
        "ollama" => vec!["llama2".to_string()],
        "litellm" => Vec::new(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_instance_id_valid() {
        let result = validate_instance_id("my-openai");
        assert!(matches!(result, Ok(Validation::Valid)));
    }

    #[test]
    fn test_validate_instance_id_with_spaces() {
        let result = validate_instance_id("my openai");
        assert!(matches!(result, Ok(Validation::Invalid(_))));
    }

    #[test]
    fn test_validate_instance_id_empty() {
        let result = validate_instance_id("");
        assert!(matches!(result, Ok(Validation::Invalid(_))));
    }

    #[test]
    fn test_validate_instance_id_starts_with_number() {
        let result = validate_instance_id("123-openai");
        assert!(matches!(result, Ok(Validation::Invalid(_))));
    }

    #[test]
    fn test_validate_instance_id_valid_with_underscore() {
        let result = validate_instance_id("my_openai");
        assert!(matches!(result, Ok(Validation::Valid)));
    }

    #[test]
    fn test_get_default_base_url_openai() {
        assert_eq!(get_default_base_url("openai"), OPENAI_BASE_URL);
    }

    #[test]
    fn test_get_default_base_url_anthropic() {
        assert_eq!(get_default_base_url("anthropic"), ANTHROPIC_BASE_URL);
    }

    #[test]
    fn test_get_default_base_url_case_insensitive() {
        assert_eq!(get_default_base_url("OpenAI"), OPENAI_BASE_URL);
        assert_eq!(get_default_base_url("GROQ"), GROQ_BASE_URL);
    }

    #[test]
    fn test_get_default_base_url_unknown() {
        assert_eq!(get_default_base_url("unknown-provider"), "");
    }

    #[test]
    fn test_get_default_models_openai() {
        let models = get_default_models("openai");
        assert!(models.contains(&"gpt-4o".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
    }

    #[test]
    fn test_get_default_models_anthropic() {
        let models = get_default_models("anthropic");
        assert!(models.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(models.len() == 3);
    }

    #[test]
    fn test_get_default_models_groq() {
        let models = get_default_models("groq");
        assert!(models.contains(&"llama3-70b-8192".to_string()));
    }

    #[test]
    fn test_get_default_models_unknown() {
        let models = get_default_models("unknown-provider");
        assert!(models.is_empty());
    }

    #[test]
    fn test_get_default_models_case_insensitive() {
        let models = get_default_models("OpenAI");
        assert!(!models.is_empty());
    }
}

