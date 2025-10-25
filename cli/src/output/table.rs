use colored::*;
use genai_keyfinder_core::ScanResult;

pub fn output_table(result: &ScanResult) -> Result<(), anyhow::Error> {
    println!("\n{}", "=== Discovered Keys ===".green().bold());
    println!(
        "{:<20} {:<40} {:<15} {:<10}",
        "Provider".bold(),
        "Source".bold(),
        "Type".bold(),
        "Confidence".bold()
    );
    println!("{}", "-".repeat(90));

    for key in &result.keys {
        let confidence_color = match key.confidence {
            genai_keyfinder_core::models::discovered_key::Confidence::VeryHigh => {
                key.confidence.to_string().green()
            }
            genai_keyfinder_core::models::discovered_key::Confidence::High => {
                key.confidence.to_string().green()
            }
            genai_keyfinder_core::models::discovered_key::Confidence::Medium => {
                key.confidence.to_string().yellow()
            }
            genai_keyfinder_core::models::discovered_key::Confidence::Low => {
                key.confidence.to_string().red()
            }
        };

        println!(
            "{:<20} {:<40} {:<15} {:<10}",
            key.provider,
            truncate_path(&key.source, 40),
            key.value_type,
            confidence_color
        );
    }

    if !result.config_instances.is_empty() {
        println!("\n{}", "=== Config Instances ===".green().bold());
        println!(
            "{:<20} {:<50} {:<10}",
            "Application".bold(),
            "Path".bold(),
            "Keys".bold()
        );
        println!("{}", "-".repeat(85));

        for instance in &result.config_instances {
            println!(
                "{:<20} {:<50} {:<10}",
                instance.app_name,
                truncate_path(&instance.config_path.display().to_string(), 50),
                instance.keys.len()
            );
        }
    }

    println!(
        "\n{}",
        format!(
            "Total: {} keys, {} config instances",
            result.keys.len(),
            result.config_instances.len()
        )
        .cyan()
    );

    Ok(())
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    }
}
