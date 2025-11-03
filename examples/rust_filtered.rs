// Example: filter by providers and include options

use aicred_core::{scan, ScanOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure provider filters
    let options = ScanOptions::default()
        .with_only_providers(vec!["openai".into(), "anthropic".into()])
        .with_full_values(false) // keep secrets redacted
        .with_max_file_size(1_048_576);

    let result = scan(options)?;

    println!("Providers scanned: {:?}", result.providers_scanned);
    println!("Found {} keys ({} config instances)", result.keys.len(), result.config_instances.len());

    for key in &result.keys {
        // When full value is omitted, redaction preview will be "****"
        println!(
            "{}: {} (type={:?}, confidence={:?}) in {}",
            key.provider,
            "****",
            key.value_type,
            key.confidence,
            key.source
        );
    }

    Ok(())
}