//! Review phase - select credentials to import

use anyhow::{Context, Result};
use aicred_core::{DiscoveredCredential, ScanResult};
use console::style;
use inquire::MultiSelect;

use super::WizardOptions;
use super::ui;

/// Run the review phase
pub fn run_review_phase(
    scan_result: &ScanResult,
    options: &WizardOptions,
) -> Result<Vec<DiscoveredCredential>> {
    use aicred_core::Confidence;
    
    // If no credentials found, return empty vec
    if scan_result.keys.is_empty() {
        return Ok(Vec::new());
    }
    
    ui::section_header("Review Discovered Credentials");
    
    println!("Select credentials to import:");
    println!("{}", style("(Space to toggle, Enter to confirm)").dim());
    println!();
    
    // Build options for multi-select
    let credential_options: Vec<String> = scan_result
        .keys
        .iter()
        .map(|cred| format_credential_option(cred))
        .collect();
    
    // Pre-select all high-confidence credentials
    let defaults: Vec<usize> = scan_result
        .keys
        .iter()
        .enumerate()
        .filter(|(_, cred)| matches!(cred.confidence, Confidence::High | Confidence::VeryHigh))
        .map(|(i, _)| i)
        .collect();
    
    if options.auto_accept {
        // Auto-accept all high-confidence credentials
        let selected = scan_result
            .keys
            .iter()
            .filter(|cred| matches!(cred.confidence, Confidence::High | Confidence::VeryHigh))
            .cloned()
            .collect();
        
        println!(
            "{} Auto-selected {} high-confidence credentials",
            style("✓").green(),
            defaults.len()
        );
        println!();
        
        return Ok(selected);
    }
    
    // Let user select
    let selections = MultiSelect::new("", credential_options)
        .with_default(&defaults)
        .prompt()
        .context("Failed to get credential selection")?;
    
    // Map back to credentials
    let selected: Vec<DiscoveredCredential> = selections
        .iter()
        .filter_map(|s| {
            scan_result
                .keys
                .iter()
                .find(|c| format_credential_option(c) == *s)
                .cloned()
        })
        .collect();
    
    println!();
    println!(
        "{} Selected {} credentials to import",
        style("✓").green(),
        style(selected.len()).cyan().bold()
    );
    println!();
    
    Ok(selected)
}

/// Format a credential for display in the multi-select
fn format_credential_option(cred: &DiscoveredCredential) -> String {
    use aicred_core::Confidence;
    
    let provider_name = format!("{} API Key", cred.provider);
    let confidence_str = format!("{:?}", cred.confidence);
    let confidence = match cred.confidence {
        Confidence::High | Confidence::VeryHigh => style(&confidence_str).green(),
        Confidence::Medium => style(&confidence_str).yellow(),
        _ => style(&confidence_str).red(),
    };
    
    let redacted = cred.redacted_value();
    let short_hash = if redacted.len() >= 12 {
        &redacted[..12]
    } else {
        &redacted
    };
    
    format!(
        "{} (SHA-256: {}..., confidence: {})",
        provider_name,
        short_hash.trim_end_matches('.'),
        confidence
    )
}
