# Plugin Development Guide

Learn how to create custom provider plugins and application scanners.

## Architecture Overview

The AICred architecture separates concerns between **discovery** and **validation**:

- **ScannerPlugin**: Discovers API keys and configuration files across applications and providers
- **ProviderPlugin**: Validates and scores discovered keys, providing confidence metrics

## ScannerPlugin (Discovery)

ScannerPlugin implementations handle the discovery of API keys and configuration files across different applications and providers.

### Required Methods

```rust
use aicred_core::scanners::{ScannerPlugin, ScanResult};
use aicred_core::models::{DiscoveredKey, ConfigInstance};
use aicred_core::error::Result;
use std::path::{Path, PathBuf};

pub struct MyScannerPlugin;

impl ScannerPlugin for MyScannerPlugin {
    /// Returns the name of this scanner (e.g., "my-app").
    fn name(&self) -> &str {
        "my-app"
    }
    
    /// Returns the application name (e.g., "My Application").
    fn app_name(&self) -> &str {
        "My Application"
    }
    
    /// Returns the paths that this scanner should scan for configuration files.
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            home_dir.join(".my-app/config.json"),
            home_dir.join(".config/my-app/settings.yaml"),
        ]
    }
    
    /// Parses a configuration file and extracts discovered keys and config instances.
    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> {
        let mut result = ScanResult::new();
        
        // Parse the configuration content
        // Extract API keys and create DiscoveredKey instances
        // Extract configuration instances and create ConfigInstance instances
        
        // Example: Extract API keys from JSON config
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(api_key) = json.get("api_key").and_then(|v| v.as_str()) {
                let key = DiscoveredKey::new(
                    "my-provider".to_string(),
                    path.display().to_string(),
                    aicred_core::models::ValueType::ApiKey,
                    aicred_core::models::Confidence::High,
                    api_key.to_string(),
                );
                result.add_key(key);
            }
        }
        
        Ok(result)
    }
    
    /// Validates that this scanner can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool {
        // Check if this is a configuration file for your application
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name.ends_with("config.json") || file_name.ends_with("settings.yaml")
    }
}
```

### Optional Methods for Enhanced Functionality

```rust
impl ScannerPlugin for MyScannerPlugin {
    /// Returns true if this scanner can discover provider-specific configurations.
    /// This enables the scanner to look for provider keys in application configs.
    fn supports_provider_scanning(&self) -> bool {
        true
    }
    
    /// Returns a list of provider names that this scanner can discover.
    /// Only used when supports_provider_scanning() returns true.
    fn supported_providers(&self) -> Vec<String> {
        vec!["openai".to_string(), "anthropic".to_string(), "my-provider".to_string()]
    }
    
    /// Scans for provider-specific configuration files (e.g., .env files, provider configs).
    /// This method enables scanners to discover provider keys in standard locations.
    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        // Common provider configuration file patterns
        paths.push(home_dir.join(".env"));
        paths.push(home_dir.join(".env.local"));
        paths.push(home_dir.join("openai.json"));
        paths.push(home_dir.join("anthropic.json"));
        
        // Filter to only existing paths
        Ok(paths.into_iter().filter(|p| p.exists()).collect())
    }
    
    /// Scans for multiple instances of this application (e.g., multiple installations).
    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>> {
        let mut instances = Vec::new();
        
        // Scan application-specific paths
        let app_paths = self.scan_paths(home_dir);
        for path in app_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(result) = self.parse_config(&path, &content) {
                        if !result.keys.is_empty() || !result.instances.is_empty() {
                            instances.extend(result.instances);
                        }
                    }
                }
            }
        }
        
        // If this scanner supports provider scanning, also scan for provider configs
        if self.supports_provider_scanning() {
            if let Ok(provider_paths) = self.scan_provider_configs(home_dir) {
                for path in provider_paths {
                    if path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(result) = self.parse_config(&path, &content) {
                                if !result.keys.is_empty() || !result.instances.is_empty() {
                                    instances.extend(result.instances);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(instances)
    }
}
```

## ProviderPlugin (Validation)

ProviderPlugin implementations now focus on validating and scoring discovered keys, rather than discovering them.

### Basic Implementation

```rust
use aicred_core::plugins::ProviderPlugin;
use aicred_core::error::Result;
use std::path::Path;

pub struct MyProviderPlugin;

impl ProviderPlugin for MyProviderPlugin {
    /// Returns the name of this plugin.
    fn name(&self) -> &str {
        "my-provider"
    }
    
    /// Returns a confidence score for a potential key (0.0 to 1.0).
    fn confidence_score(&self, key: &str) -> f32 {
        // Analyze the key and return a confidence score
        if key.starts_with("mp-") && key.len() > 20 {
            0.95
        } else if key.len() > 15 && key.contains('-') {
            0.70
        } else {
            0.30
        }
    }
    
    /// Validates that this plugin can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool {
        // Check if this plugin should handle the file
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name.ends_with(".env") || file_name.ends_with("my-provider.json")
    }
    
    /// Gets the provider type this plugin handles.
    fn provider_type(&self) -> &str {
        "my-provider"
    }
}
```

### Advanced Implementation with Pattern Matching

```rust
impl ProviderPlugin for MyProviderPlugin {
    fn confidence_score(&self, key: &str) -> f32 {
        let mut score: f32 = 0.3; // Base score
        
        // Length-based scoring
        if key.len() >= 20 {
            score += 0.2;
        }
        if key.len() >= 40 {
            score += 0.1;
        }
        
        // Character diversity scoring
        let has_uppercase = key.chars().any(|c| c.is_uppercase());
        let has_lowercase = key.chars().any(|c| c.is_lowercase());
        let has_digits = key.chars().any(|c| c.is_ascii_digit());
        let has_special = key.chars().any(|c| !c.is_alphanumeric());
        
        if has_uppercase && has_lowercase {
            score += 0.1;
        }
        if has_digits {
            score += 0.05;
        }
        if has_special {
            score += 0.05;
        }
        
        // Common key prefixes
        if key.starts_with("mp-") || key.starts_with("ak-") {
            score += 0.1;
        }
        
        score.min(1.0) as f32
    }
}
```

## Built-in Helper Plugins

### CommonConfigPlugin

The `CommonConfigPlugin` provides basic confidence scoring for common configuration patterns:

```rust
use aicred_core::plugins::CommonConfigPlugin;

let plugin = CommonConfigPlugin;
let score = plugin.confidence_score("sk-EXAMPLE_FAKE_TOKEN_1234567890abcdef");
// Returns a score based on length, character diversity, and common patterns
```

## Registering Your Plugins

### Scanner Registration

```rust
use aicred_core::scanners::{ScannerRegistry, register_builtin_scanners};
use std::sync::Arc;

let scanner_registry = ScannerRegistry::new();

// Register built-in scanners
register_builtin_scanners(&scanner_registry)?;

// Register your custom scanner
scanner_registry.register(Arc::new(MyScannerPlugin))?;
```

### Provider Registration

```rust
use aicred_core::plugins::{PluginRegistry, register_builtin_plugins};
use std::sync::Arc;

let provider_registry = PluginRegistry::new();

// Register built-in providers
register_builtin_plugins(&provider_registry)?;

// Register your custom provider
provider_registry.register(Arc::new(MyProviderPlugin))?;
```

## Testing Your Plugin

### ScannerPlugin Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_my_scanner_detection() {
        let scanner = MyScannerPlugin;
        let content = r#"{
            "api_key": "mp-1234567890abcdef",
            "provider": "my-provider"
        }"#;
        
        let result = scanner.parse_config(Path::new("test.json"), content).unwrap();
        assert_eq!(result.keys.len(), 1);
        assert_eq!(result.keys[0].provider, "my-provider");
    }
    
    #[test]
    fn test_provider_scanning_support() {
        let scanner = MyScannerPlugin;
        assert!(scanner.supports_provider_scanning());
        assert!(scanner.supported_providers().contains(&"openai".to_string()));
    }
}
```

### ProviderPlugin Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_provider_confidence() {
        let plugin = MyProviderPlugin;
        
        // High confidence for valid key
        let score1 = plugin.confidence_score("mp-1234567890abcdef");
        assert!(score1 > 0.8);
        
        // Low confidence for simple key
        let score2 = plugin.confidence_score("simple-key");
        assert!(score2 < 0.5);
    }
}
```

## Integration Example

Here's how to use both plugins together in the main scanning process:

```rust
use aicred_core::{
    scan, ScanOptions, PluginRegistry, ScannerRegistry,
    register_builtin_plugins, register_builtin_scanners
};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registries
    let provider_registry = PluginRegistry::new();
    let scanner_registry = ScannerRegistry::new();
    
    // Register built-in plugins and scanners
    register_builtin_plugins(&provider_registry)?;
    register_builtin_scanners(&scanner_registry)?;
    
    // Register custom plugins
    provider_registry.register(Arc::new(MyProviderPlugin))?;
    scanner_registry.register(Arc::new(MyScannerPlugin))?;
    
    // Configure scan options
    let options = ScanOptions::default()
        .with_only_providers(vec!["my-provider".to_string()]);
    
    // Run the scan
    let result = scan(options)?;
    println!("Found {} keys", result.total_keys());
    
    Ok(())
}
```

## Migration from Old Architecture

If you're upgrading from the previous architecture where ProviderPlugin handled both discovery and validation:

### Before (Old Architecture)
```rust
impl ProviderPlugin for MyPlugin {
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> { /* ... */ }
    fn parse_config(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> { /* ... */ }
    fn confidence_score(&self, key: &str) -> f32 { /* ... */ }
}
```

### After (New Architecture)
```rust
// Split into two separate plugins
impl ScannerPlugin for MyScannerPlugin {
    fn name(&self) -> &str { "my-app" }
    fn app_name(&self) -> &str { "My Application" }
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> { /* ... */ }
    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult> { /* ... */ }
    fn can_handle_file(&self, path: &Path) -> bool { /* ... */ }
}

impl ProviderPlugin for MyProviderPlugin {
    fn name(&self) -> &str { "my-provider" }
    fn confidence_score(&self, key: &str) -> f32 { /* ... */ }
    fn can_handle_file(&self, path: &Path) -> bool { /* ... */ }
}
```

## Tips

- **ScannerPlugin**: Focus on file discovery and parsing logic
- **ProviderPlugin**: Focus on key validation and confidence scoring
- **Reuse helpers**: Use built-in helpers like `CommonConfigPlugin` for common patterns
- **Security**: Always respect security defaults - don't serialize full values
- **Confidence**: Provide conservative confidence scores unless you match strong provider-specific patterns
- **Testing**: Add integration tests using fixtures in a `tests/fixtures/` directory

## Debugging

- Run unit tests in your crate:
  ```bash
  cargo test -p aicred-core
  ```
- Log parsing paths and hits with `tracing` at debug level if you need deeper insight
- Use the `--dry-run` flag to see what would be scanned without actually reading files