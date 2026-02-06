//! Scan phase - discover existing AI credentials

use anyhow::{Context, Result};
use aicred_core::{scan, ScanOptions, ScanResult};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use super::WizardOptions;
use super::ui;

/// Run the scan phase
pub fn run_scan_phase(options: &WizardOptions) -> Result<ScanResult> {
    ui::section_header("Scanning for AI Credentials");
    
    println!("Scanning your system for existing API keys and configurations...");
    println!();
    
    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message("Scanning configuration files...");
    
    // Build scan options
    let scan_opts = ScanOptions {
        home_dir: options.home.clone(),
        include_full_values: true, // We need full values to create instances
        only_providers: None,
        exclude_providers: None,
        max_file_size: 1024 * 1024, // 1MB
        probe_models: false,
        probe_timeout_secs: 30,
    };
    
    // Run the scan
    let result = scan(&scan_opts)
        .context("Failed to scan for credentials")?;
    
    pb.finish_and_clear();
    
    // Show results
    let total_keys = result.keys.len();
    let total_instances = result.config_instances.len();
    
    if total_keys == 0 && total_instances == 0 {
        println!("{}", style("No existing credentials found.").yellow());
        println!();
        println!("That's okay! We'll help you add providers manually.");
    } else {
        println!(
            "{} Found {} credentials across {} applications",
            style("✓").green(),
            style(total_keys).cyan().bold(),
            style(total_instances).cyan().bold()
        );
        
        if options.verbose {
            println!();
            println!("Details:");
            for cred in &result.keys {
                let confidence_str = format!("{:?}", cred.confidence);
                println!(
                    "  {} {} (confidence: {})",
                    style("•").dim(),
                    cred.provider,
                    style(confidence_str).dim()
                );
            }
        }
    }
    
    println!();
    Ok(result)
}
