//! Interactive setup wizard for first-time AICred configuration

use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;

mod scan;
mod review;
mod configure;
mod labels;
mod summary;
mod ui;

use scan::run_scan_phase;
use review::run_review_phase;
use configure::run_configure_phase;
use labels::run_labels_phase;
use summary::run_summary_phase;

/// Wizard configuration options
#[derive(Debug, Clone)]
pub struct WizardOptions {
    /// Skip prompts, use defaults (for --yes flag)
    pub auto_accept: bool,
    
    /// Skip model probing
    pub skip_probe: bool,
    
    /// Probe timeout in seconds
    pub probe_timeout: u64,
    
    /// Skip label setup
    pub skip_labels: bool,
    
    /// Output verbosity
    pub verbose: bool,
    
    /// Home directory override
    pub home: Option<PathBuf>,
}

impl Default for WizardOptions {
    fn default() -> Self {
        Self {
            auto_accept: false,
            skip_probe: false,
            probe_timeout: 30,
            skip_labels: false,
            verbose: false,
            home: None,
        }
    }
}

/// Result of a wizard run
#[derive(Debug)]
pub struct WizardResult {
    /// Number of instances created
    pub instances_created: usize,
    
    /// Number of labels created
    pub labels_created: usize,
    
    /// Path to config file
    pub config_path: PathBuf,
    
    /// Any warnings or notes
    pub warnings: Vec<String>,
}

/// Main wizard entry point
pub fn run_wizard(options: WizardOptions) -> Result<WizardResult> {
    // Show welcome screen
    ui::show_welcome()?;
    
    // Phase 1: Scan for existing credentials
    let scan_result = run_scan_phase(&options)?;
    
    // Phase 2: Review and select credentials to import
    let selected_credentials = run_review_phase(&scan_result, &options)?;
    
    // Phase 3: Configure provider instances
    let instances = run_configure_phase(&selected_credentials, &scan_result, &options)?;
    
    // Phase 4: Set up labels (optional)
    let labels = if options.skip_labels {
        std::collections::HashMap::new()
    } else {
        run_labels_phase(&instances, &options)?
    };
    
    // Phase 5: Show summary and confirm
    let (instances, labels, config_path) = run_summary_phase(instances, labels, &options)?;
    
    // Success!
    ui::show_success(&config_path, instances.len(), labels.len())?;
    
    Ok(WizardResult {
        instances_created: instances.len(),
        labels_created: labels.len(),
        config_path,
        warnings: Vec::new(),
    })
}

/// Check if configuration already exists
pub fn config_exists(home: Option<&PathBuf>) -> Result<bool> {
    let config_dir = if let Some(h) = home {
        h.join(".config").join("aicred")
    } else {
        dirs_next::config_dir()
            .context("Could not determine config directory")?
            .join("aicred")
    };
    
    let instances_file = config_dir.join("instances.yaml");
    Ok(instances_file.exists())
}

/// Handle existing configuration
pub fn handle_existing_config(home: Option<&PathBuf>) -> Result<ExistingConfigAction> {
    use inquire::Select;
    
    let message = format!(
        "{}\n\n{}",
        style("Configuration file already exists").yellow().bold(),
        "What would you like to do?"
    );
    
    let options = vec![
        "Merge (add new providers, keep existing)",
        "Replace (overwrite existing config)",
        "Cancel",
    ];
    
    let answer = Select::new(&message, options)
        .prompt()
        .context("Failed to get user input")?;
    
    match answer {
        "Merge (add new providers, keep existing)" => Ok(ExistingConfigAction::Merge),
        "Replace (overwrite existing config)" => Ok(ExistingConfigAction::Replace),
        _ => Ok(ExistingConfigAction::Cancel),
    }
}

/// Action to take when config exists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistingConfigAction {
    Merge,
    Replace,
    Cancel,
}
