use anyhow::Result;
use colored::*;
use genai_keyfinder_core::{scan, ScanOptions};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn handle_scan(
    home: Option<String>,
    format: String,
    include_values: bool,
    only: Option<String>,
    exclude: Option<String>,
    max_bytes_per_file: usize,
    dry_run: bool,
    audit_log: Option<String>,
) -> Result<()> {
    // Determine home directory
    let home_dir = match home {
        Some(h) => PathBuf::from(h),
        None => dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?,
    };

    // Parse provider filters
    let only_providers = only.map(|s| s.split(',').map(String::from).collect());
    let exclude_providers = exclude.map(|s| s.split(',').map(String::from).collect());

    // Create scan options
    let options = ScanOptions {
        home_dir: Some(home_dir.clone()),
        include_full_values: include_values,
        max_file_size: max_bytes_per_file,
        only_providers,
        exclude_providers,
    };

    if dry_run {
        println!("{}", "DRY RUN MODE - No files will be read".yellow().bold());
        // Show what would be scanned
        println!("Would scan directory: {}", home_dir.display());
        return Ok(());
    }

    // Perform scan
    println!("{}", "Scanning for GenAI credentials...".cyan().bold());
    let result = scan(options)?;

    // Output results based on format
    match format.as_str() {
        "json" => crate::output::json::output_json(&result)?,
        "ndjson" => crate::output::ndjson::output_ndjson(&result)?,
        "table" => crate::output::table::output_table(&result)?,
        "summary" => crate::output::summary::output_summary(&result)?,
        _ => anyhow::bail!("Unknown format: {}", format),
    }

    // Write audit log if requested
    if let Some(log_path) = audit_log {
        write_audit_log(&log_path, &result)?;
    }

    // Exit code: 0 if keys found, 1 if none found
    if result.keys.is_empty() && result.config_instances.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

fn write_audit_log(log_path: &str, result: &genai_keyfinder_core::ScanResult) -> Result<()> {
    let mut file = File::create(log_path)?;
    writeln!(file, "GenAI Key Finder Audit Log")?;
    writeln!(file, "=========================")?;
    writeln!(file, "Scan Date: {}", result.scan_completed_at)?;
    writeln!(file, "Home Directory: {}", result.home_directory)?;
    writeln!(
        file,
        "Providers Scanned: {}",
        result.providers_scanned.join(", ")
    )?;
    writeln!(file, "Total Keys Found: {}", result.keys.len())?;
    writeln!(
        file,
        "Total Config Instances: {}",
        result.config_instances.len()
    )?;
    writeln!(file)?;

    if !result.keys.is_empty() {
        writeln!(file, "Discovered Keys:")?;
        for key in &result.keys {
            writeln!(
                file,
                "  - {}: {} ({} - confidence: {})",
                key.provider, key.value_type, key.source, key.confidence
            )?;
        }
    }

    if !result.config_instances.is_empty() {
        writeln!(file, "\nConfig Instances:")?;
        for instance in &result.config_instances {
            writeln!(
                file,
                "  - {}: {}",
                instance.app_name,
                instance.config_path.display()
            )?;
        }
    }

    Ok(())
}
