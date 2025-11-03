// Example: implement and register a custom provider plugin

use aicred_core::{
    error::Result,
    models::{DiscoveredKey, ValueType, Confidence},
    plugins::{PluginRegistry, ProviderPlugin},
    scanner::{Scanner, ScannerConfig},
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct MyProviderPlugin;

impl ProviderPlugin for MyProviderPlugin {
    fn name(&self) -> &str {
        "my-provider"
    }

    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            home_dir.join(".my-provider").join("config.json"),
            home_dir.join(".config").join("my-provider").join("api_key"),
        ]
    }

    fn parse_config(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> {
        // Minimal example parser:
        // If content looks like it contains a key starting with "mp-", emit a discovered key.
        let mut keys = Vec::new();
        if content.contains("mp-") {
            // In a real implementation, properly extract the key string.
            let full_value = "mp-EXAMPLE-KEY-123456".to_string();
            keys.push(DiscoveredKey::new(
                self.name().to_string(),
                path.display().to_string(),
                ValueType::ApiKey,
                Confidence::High,
                full_value,
            ));
        }
        Ok(keys)
    }

    fn confidence_score(&self, key: &str) -> f32 {
        if key.starts_with("mp-") { 0.95 } else { 0.70 }
    }
}

fn main() -> Result<()> {
    // Create a registry and register built-ins plus our custom plugin
    let registry = PluginRegistry::new();
    aicred_core::register_builtin_plugins(&registry)?;
    registry.register(Arc::new(MyProviderPlugin))?;

    // Create a scanner with default config
    let scanner = Scanner::with_config(registry, ScannerConfig::default());

    // Scan the user's home directory
    let home = dirs_next::home_dir().expect("Failed to resolve home directory");
    let result = scanner.scan(&home)?;

    println!("Found {} keys", result.total_keys());
    for key in &result.keys {
        println!("{}: {} (confidence: {:?}) from {}",
            key.provider,
            "****", // full values are redacted by default
            key.confidence,
            key.source
        );
    }

    Ok(())
}