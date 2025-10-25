use colored::*;
use genai_keyfinder_core::ScanResult;

pub fn output_summary(result: &ScanResult, verbose: bool) -> Result<(), anyhow::Error> {
    println!("\n{}", "Scan Summary".green().bold());
    println!("  Home Directory: {}", result.home_directory);
    println!("  Scan Time: {}", result.scan_completed_at);
    println!(
        "  Providers Scanned: {}",
        result.providers_scanned.join(", ")
    );
    println!("\n{}", "Results:".cyan().bold());
    println!("  Keys Found: {}", result.keys.len());
    println!("  Config Instances: {}", result.config_instances.len());

    // Group by provider
    let mut by_provider: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for key in &result.keys {
        *by_provider.entry(key.provider.clone()).or_insert(0) += 1;
    }

    if !by_provider.is_empty() {
        println!("\n{}", "By Provider:".cyan().bold());
        for (provider, count) in by_provider {
            println!("  {}: {}", provider, count);
        }
    }

    // Show detailed key information if verbose
    if verbose && !result.keys.is_empty() {
        println!("\n{}", "Discovered Keys:".cyan().bold());
        for key in &result.keys {
            println!(
                "  - {}: {} ({} - confidence: {})",
                key.provider, key.value_type, key.source, key.confidence
            );
            if let Some(full_value) = key.full_value() {
                println!("    Value: {}", full_value);
            } else {
                println!("    Value: {}", key.redacted_value());
            }
        }
    }

    // Show detailed config instances if verbose
    if verbose && !result.config_instances.is_empty() {
        println!("\n{}", "Config Instances:".cyan().bold());
        for instance in &result.config_instances {
            println!(
                "  - {}: {}",
                instance.app_name,
                instance.config_path.display()
            );
            if !instance.keys.is_empty() {
                println!("    Keys: {}", instance.keys.len());
            }
        }
    }

    Ok(())
}
