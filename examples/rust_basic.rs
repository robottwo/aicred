// Basic example: scan with default options and print results

use genai_keyfinder_core::{scan, ScanOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ScanOptions::default();
    let result = scan(options)?;

    println!("Found {} keys", result.keys.len());
    for key in &result.keys {
        // Full values are not serialized by default; show a generic redaction preview
        println!(
            "{}: {} (confidence: {:?}) from {}",
            key.provider,
            key.redacted_value(), // helper computes a redaction preview when full value is present; otherwise "****"
            key.confidence,
            key.source
        );
    }

    if !result.config_instances.is_empty() {
        println!("Discovered {} configuration instances:", result.config_instances.len());
        for inst in &result.config_instances {
            println!("  - {} at {}", inst.app_name, inst.config_path.display());
        }
    }

    Ok(())
}