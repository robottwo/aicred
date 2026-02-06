// Example: implement and register a custom provider plugin
//
// This example shows the NEW v0.2.0+ API (simplified plugin registry).
// For the legacy API, see rust_custom_plugin_legacy.rs

use aicred_core::{
    error::Result,
    models::{DiscoveredKey, ValueType, Confidence},
    plugins::{ProviderPlugin, ProviderRegistry, register_builtin_providers},
    ScanOptions, scan,
};
use std::path::Path;
use std::sync::Arc;

/// Custom provider plugin example
struct MyProviderPlugin;

impl ProviderPlugin for MyProviderPlugin {
    fn name(&self) -> &str {
        "my-provider"
    }

    fn confidence_score(&self, key: &str) -> f32 {
        if key.starts_with("mp-") { 0.95 } else { 0.70 }
    }

    fn can_handle_file(&self, path: &Path) -> bool {
        // This plugin can handle files in .my-provider directory
        path.to_string_lossy().contains(".my-provider")
    }
}

fn main() -> Result<()> {
    // Create a registry with built-in providers (v0.2.0+ API - returns HashMap directly)
    let mut registry: ProviderRegistry = register_builtin_providers();
    
    // Add our custom plugin
    let plugin = Arc::new(MyProviderPlugin) as Arc<dyn ProviderPlugin>;
    registry.insert(plugin.name().to_string(), plugin);

    println!("Registered {} provider plugins", registry.len());
    
    // Note: The scan() function uses register_builtin_providers() internally,
    // so this example demonstrates the registry API but doesn't actually use
    // the custom plugin during scanning.
    //
    // To use a custom plugin during scanning, you would need to:
    // 1. Create a custom scanner that uses your plugin
    // 2. Or extend the built-in discovery system
    
    let options = ScanOptions::default();
    let result = scan(&options)?;

    println!("Found {} keys", result.keys.len());
    for key in &result.keys {
        println!("{}: {} (confidence: {:?}) from {}",
            key.provider,
            key.redacted_value(),
            key.confidence,
            key.source
        );
    }

    Ok(())
}
