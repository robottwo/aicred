# Migration Guide: Upgrading to the New Plugin Architecture

This guide helps you migrate from the old ProviderPlugin architecture to the new separated ScannerPlugin/ProviderPlugin architecture.

## Overview of Changes

### Old Architecture (Before)
- **ProviderPlugin**: Handled both key discovery AND validation
- Single plugin type responsible for finding keys, parsing configs, and scoring
- Limited flexibility for application-specific scanning

### New Architecture (After)
- **ScannerPlugin**: Handles key discovery and configuration parsing
- **ProviderPlugin**: Handles key validation and confidence scoring
- Clear separation of concerns between discovery and validation

## Breaking Changes

### ProviderPlugin Trait Changes

**Removed Methods:**
- `scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>`
- `parse_config(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>>`
- `version(&self) -> &str`
- `provider_metadata(&self) -> Provider`

**Remaining Methods:**
- `name(&self) -> &str`
- `confidence_score(&self, key: &str) -> f32`
- `can_handle_file(&self, path: &Path) -> bool` (now required)

**New Method:**
- `provider_type(&self) -> &str` (default implementation available)

### New ScannerPlugin Trait

**Brand new trait for discovery:**
- `name(&self) -> &str`
- `app_name(&self) -> &str`
- `scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>`
- `parse_config(&self, path: &Path, content: &str) -> Result<ScanResult>`
- `can_handle_file(&self, path: &Path) -> bool`

**Optional methods for enhanced functionality:**
- `supports_provider_scanning(&self) -> bool`
- `supported_providers(&self) -> Vec<String>`
- `scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>>`
- `scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>>`

## Migration Steps

### Step 1: Split Your Plugin into Two

#### Before (Single Plugin)
```rust
pub struct MyPlugin;

impl ProviderPlugin for MyPlugin {
    fn name(&self) -> &str {
        "my-provider"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            home_dir.join(".my-provider/config.json"),
            home_dir.join(".config/my-provider/api_key"),
        ]
    }
    
    fn parse_config(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> {
        // Parse configuration and extract keys
        // Return discovered keys with confidence scores
        Ok(vec![])
    }
    
    fn confidence_score(&self, key: &str) -> f32 {
        if key.starts_with("mp-") {
            0.95
        } else {
            0.70
        }
    }
}
```

#### After (Two Separate Plugins)
```rust
// 1. Create a ScannerPlugin for discovery
pub struct MyAppScanner;

impl ScannerPlugin for MyAppScanner {
    fn name(&self) -> &str {
        "my-app"  // Different from provider name
    }
    
    fn app_name(&self) -> &str {
        "My Application"
    }
    
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![
            home_dir.join(".my-provider/config.json"),
            home_dir.join(".config/my-provider/api_key"),
        ]
    }
    
    fn parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult> {
        let mut result = scanners::ScanResult::new();
        
        // Parse configuration and extract keys
        // Use DiscoveredKey::new() to create keys
        // Return ScanResult with keys and instances
        
        Ok(result)
    }
    
    fn can_handle_file(&self, path: &Path) -> bool {
        // Check if this scanner should handle the file
        path.extension().map_or(false, |ext| ext == "json")
    }
}

// 2. Create a ProviderPlugin for validation
pub struct MyProviderPlugin;

impl ProviderPlugin for MyProviderPlugin {
    fn name(&self) -> &str {
        "my-provider"  // Same as original provider name
    }
    
    fn confidence_score(&self, key: &str) -> f32 {
        if key.starts_with("mp-") {
            0.95
        } else {
            0.70
        }
    }
    
    fn can_handle_file(&self, path: &Path) -> bool {
        // Same logic as before
        path.extension().map_or(false, |ext| ext == "json")
    }
}
```

### Step 2: Update Plugin Registration

#### Before
```rust
use genai_keyfinder_core::plugins::{PluginRegistry, ProviderPlugin};
use std::sync::Arc;

let registry = PluginRegistry::new();
registry.register(Arc::new(MyPlugin))?;
```

#### After
```rust
use genai_keyfinder_core::plugins::PluginRegistry;
use genai_keyfinder_core::scanners::ScannerRegistry;
use std::sync::Arc;

// Register scanner plugin for discovery
let scanner_registry = ScannerRegistry::new();
scanner_registry.register(Arc::new(MyAppScanner))?;

// Register provider plugin for validation
let provider_registry = PluginRegistry::new();
provider_registry.register(Arc::new(MyProviderPlugin))?;
```

### Step 3: Update Parse Method Return Type

#### Before
```rust
fn parse_config(&self, path: &Path, content: &str) -> Result<Vec<DiscoveredKey>> {
    let mut keys = Vec::new();
    
    // Parse and extract keys
    if let Some(api_key) = extract_key(content) {
        keys.push(DiscoveredKey::new(
            "my-provider".to_string(),
            path.display().to_string(),
            ValueType::ApiKey,
            Confidence::High,
            api_key.to_string(),
        ));
    }
    
    Ok(keys)
}
```

#### After
```rust
fn parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult> {
    let mut result = scanners::ScanResult::new();
    
    // Parse and extract keys
    if let Some(api_key) = extract_key(content) {
        let key = DiscoveredKey::new(
            "my-provider".to_string(),
            path.display().to_string(),
            ValueType::ApiKey,
            Confidence::High,
            api_key.to_string(),
        );
        result.add_key(key);
    }
    
    // Can also add config instances
    if let Some(instance) = create_config_instance(path, content) {
        result.add_instance(instance);
    }
    
    Ok(result)
}
```

### Step 4: Handle Provider-Specific Scanning (Optional)

If your scanner can discover keys for multiple providers, implement the optional methods:

```rust
impl ScannerPlugin for MyAppScanner {
    // ... required methods ...
    
    fn supports_provider_scanning(&self) -> bool {
        true
    }
    
    fn supported_providers(&self) -> Vec<String> {
        vec!["openai".to_string(), "anthropic".to_string(), "my-provider".to_string()]
    }
    
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
}
```

## Common Migration Patterns

### Pattern 1: Simple Provider Migration

**Before**: Single provider plugin that scans and validates
**After**: Separate scanner for discovery, provider for validation

### Pattern 2: Application Scanner Migration

**Before**: No application-specific scanning
**After**: Create ScannerPlugin for your application

```rust
pub struct MyAppScanner;

impl ScannerPlugin for MyAppScanner {
    fn name(&self) -> &str { "my-app" }
    fn app_name(&self) -> &str { "My Application" }
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![home_dir.join(".my-app").join("config.json")]
    }
    fn parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult> {
        // Parse your app's config format
        Ok(scanners::ScanResult::new())
    }
    fn can_handle_file(&self, path: &Path) -> bool {
        path.file_name().map_or(false, |name| name == "config.json")
    }
}
```

### Pattern 3: Multi-Provider Scanner

**Before**: Multiple provider plugins with duplicate scanning logic
**After**: Single scanner that discovers keys for multiple providers

```rust
pub struct MultiProviderScanner;

impl ScannerPlugin for MultiProviderScanner {
    fn name(&self) -> &str { "multi-provider" }
    fn app_name(&self) -> &str { "Multi Provider Scanner" }
    
    fn supports_provider_scanning(&self) -> bool { true }
    fn supported_providers(&self) -> Vec<String> {
        vec!["openai".to_string(), "anthropic".to_string(), "google".to_string()]
    }
    
    fn parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult> {
        let mut result = scanners::ScanResult::new();
        
        // Parse content and extract keys for multiple providers
        // Create DiscoveredKey instances with appropriate provider names
        
        Ok(result)
    }
}
```

## Benefits of the New Architecture

1. **Clear Separation of Concerns**: Discovery vs validation logic is separated
2. **Better Code Organization**: Each plugin has a single responsibility
3. **Enhanced Flexibility**: Mix and match scanners and providers
4. **Improved Performance**: Parallel scanning with specialized plugins
5. **Better Testing**: Test discovery and validation logic independently
6. **Extensibility**: Easy to add new applications or providers

## Backward Compatibility

The new architecture is **not backward compatible**. You must:

1. Split existing ProviderPlugin implementations into ScannerPlugin + ProviderPlugin
2. Update plugin registration code
3. Update any custom scanning logic
4. Test thoroughly with the new architecture

## Migration Checklist

- [ ] Identify all existing ProviderPlugin implementations
- [ ] Split each into ScannerPlugin (discovery) + ProviderPlugin (validation)
- [ ] Update `parse_config()` return type from `Vec<DiscoveredKey>` to `ScanResult`
- [ ] Update plugin registration code to use both registries
- [ ] Test scanner plugins independently
- [ ] Test provider plugins independently
- [ ] Test integrated scanning workflow
- [ ] Update documentation and examples
- [ ] Verify confidence scoring works correctly
- [ ] Test with different provider combinations

## Getting Help

If you need help with migration:

1. Review the [Plugin Development Guide](plugin-development.md)
2. Check the [Architecture Documentation](architecture.md)
3. Look at the built-in plugin implementations in the codebase
4. Test your plugins with the new architecture incrementally
5. Reach out to the community for support

Remember: The new architecture provides better separation of concerns and more flexibility, making your plugins more maintainable and extensible in the long run.