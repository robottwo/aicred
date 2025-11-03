# AICred Architecture

## Overview and Design Philosophy

The AICred is a cross-platform library designed to discover and extract GenAI API keys and provider configurations from user home directories. The architecture emphasizes extensibility, security, and cross-platform compatibility through a plugin-based design.

### Core Design Principles

1. **Plugin Architecture**: Extensible provider support through a trait-based plugin system
2. **Security First**: Built-in key redaction, hashing, and safe handling practices
3. **Cross-Platform**: Unified API across Windows, macOS, and Linux
4. **Language Bindings**: Native support for multiple programming languages
5. **Zero Dependencies Core**: Minimal external dependencies in the core library

## Core Data Models

### Provider Model

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Provider {
    /// Unique identifier for the provider
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Provider type (e.g., "openai", "anthropic", "google")
    pub provider_type: String,
    
    /// Base URL for API calls
    pub base_url: String,
    
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Documentation URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    
    /// Rate limits configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimits>,
    
    /// Supported authentication methods
    pub authentication_methods: Vec<AuthenticationMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub tokens_per_minute: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod {
    ApiKey,
    BearerToken,
    OAuth2,
    CustomHeader(String),
}
```

### Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique identifier for the model
    pub id: String,
    
    /// Reference to parent provider
    pub provider_id: String,
    
    /// Model identifier used in API calls
    pub model_id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Quantization information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    
    /// Cost per token (input)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_input_token: Option<f64>,
    
    /// Cost per token (output)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_per_output_token: Option<f64>,
    
    /// Context window size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    
    /// Model capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ModelCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_images: bool,
    pub supports_audio: bool,
    pub supports_video: bool,
    pub supports_function_calling: bool,
    pub supports_system_prompts: bool,
    pub supports_streaming: bool,
    pub supports_batch_processing: bool,
}
```

### DiscoveredKey Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredKey {
    /// Provider this key belongs to
    pub provider: String,
    
    /// Source file path where key was found
    pub source: PathBuf,
    
    /// Type of value discovered
    pub value_type: ValueType,
    
    /// Redacted value for display
    pub redacted_preview: String,
    
    /// SHA256 hash of the original value
    pub hash: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    
    /// Whether the key appears to be locked/encrypted
    pub locked: bool,
    
    /// Line number where found (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
    
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    ApiKey,
    BearerToken,
    OAuthToken,
    CustomHeader(String),
    ConfigurationValue,
}
```

### ScanResult Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Discovered keys
    pub keys: Vec<DiscoveredKey>,
    
    /// Scan metadata
    pub metadata: ScanMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    /// When the scan was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Home directory that was scanned
    pub home_dir: PathBuf,
    
    /// List of providers that were scanned
    pub providers_scanned: Vec<String>,
    
    /// Total files scanned
    pub files_scanned: usize,
    
    /// Total directories scanned
    pub directories_scanned: usize,
    
    /// Scan duration in milliseconds
    pub scan_duration_ms: u64,
    
    /// Any errors encountered during scan
    pub errors: Vec<String>,
}
```

## Plugin Architecture

The architecture now separates concerns between **discovery** and **validation**:

### ScannerPlugin Trait (Discovery)
ScannerPlugin implementations handle the discovery of API keys and configuration files across different applications and providers.

```rust
pub trait ScannerPlugin: Send + Sync {
    /// Returns the name of this scanner (e.g., "ragit", "claude-desktop").
    fn name(&self) -> &str;
    
    /// Returns the application name (e.g., "Ragit", "Claude Desktop").
    fn app_name(&self) -> &str;
    
    /// Returns the paths that this scanner should scan for configuration files.
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>;
    
    /// Parses a configuration file and extracts discovered keys and config instances.
    fn parse_config(&self, path: &Path, content: &str) -> Result<ScanResult>;
    
    /// Validates that this scanner can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool;
    
    /// Returns true if this scanner can discover provider-specific configurations.
    fn supports_provider_scanning(&self) -> bool;
    
    /// Returns a list of provider names that this scanner can discover.
    fn supported_providers(&self) -> Vec<String>;
    
    /// Scans for provider-specific configuration files.
    fn scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>>;
    
    /// Scans for multiple instances of this application.
    fn scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>>;
}
```

### ProviderPlugin Trait (Validation)
ProviderPlugin implementations now focus on validating and scoring discovered keys, rather than discovering them.

```rust
pub trait ProviderPlugin: Send + Sync {
    /// Returns the name of this plugin.
    fn name(&self) -> &str;
    
    /// Returns a confidence score for a potential key (0.0 to 1.0).
    fn confidence_score(&self, key: &str) -> f32;
    
    /// Validates that this plugin can handle the given file.
    fn can_handle_file(&self, path: &Path) -> bool;
    
    /// Gets the provider type this plugin handles.
    fn provider_type(&self) -> &str;
}
```

## Configuration Validation and Rewrite System

The architecture uses a validation-and-rewrite approach for handling configuration files. Instead of complex migration logic, invalid configurations are automatically replaced with default settings.

### Validation Components

#### ConfigValidator
Handles validation of configuration files and determines if they should be replaced with defaults:

```rust
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validates configuration content
    pub fn validate_config(content: &str, format: ConfigFormat) -> Result<bool, ValidationError>
    
    /// Returns default configuration for a given format
    pub fn get_default_config(format: ConfigFormat) -> String
}
```

#### ConfigFormat
Supported configuration formats:

```rust
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Env,
}
```

### Automatic Configuration Handling
The system automatically validates and handles configurations when loading:

```rust
impl ConfigInstance {
    /// Loads from JSON with validation
    pub fn from_json(content: &str) -> Result<Self, ConfigError>
    
    /// Loads from YAML with validation
    pub fn from_yaml(content: &str) -> Result<Self, ConfigError>
    
    /// Checks if content is valid
    pub fn is_valid(content: &str) -> bool
}
```

### Validation Benefits
- **Simplicity**: No complex migration logic to maintain
- **Reliability**: Invalid configs are replaced with known-good defaults
- **Consistency**: All configurations follow current format requirements
- **Security**: No risk of malformed configurations causing issues

### Plugin Registry

```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn ProviderPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn register(&mut self, plugin: Box<dyn ProviderPlugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn get_plugin(&self, name: &str) -> Option<&dyn ProviderPlugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }
    
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }
}
```

### Plugin Discovery

Plugins are discovered through:

1. **Built-in Plugins**: Compiled into the library
2. **Dynamic Plugins**: Loaded from plugin directories
3. **Registry-based**: Plugins registered at runtime

```rust
pub struct PluginDiscovery {
    builtin_plugins: Vec<Box<dyn ProviderPlugin>>,
    plugin_dirs: Vec<PathBuf>,
}

impl PluginDiscovery {
    pub fn discover_plugins(&self) -> Result<Vec<Box<dyn ProviderPlugin>>, PluginError> {
        let mut plugins = Vec::new();
        
        // Add built-in plugins
        plugins.extend(self.load_builtin_plugins()?);
        
        // Load from plugin directories
        for dir in &self.plugin_dirs {
            plugins.extend(self.load_from_directory(dir)?);
        }
        
        Ok(plugins)
    }
}
```

## Module Structure

```
aicred/
├── core/                    # Core library (Rust)
│   ├── src/
│   │   ├── lib.rs          # Main library entry point
│   │   ├── error.rs        # Error types and handling
│   │   ├── models/         # Data models
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs
│   │   │   ├── model.rs
│   │   │   ├── key.rs
│   │   │   └── scan.rs
│   │   ├── plugins/      # Plugin system
│   │   │   ├── mod.rs
│   │   │   ├── trait.rs
│   │   │   ├── registry.rs
│   │   │   └── discovery.rs
│   │   ├── scanner/        # File scanning logic
│   │   │   ├── mod.rs
│   │   │   ├── filesystem.rs
│   │   │   └── worker.rs
│   │   ├── parser/       # Configuration parsing
│   │   │   ├── mod.rs
│   │   │   ├── json.rs
│   │   │   ├── yaml.rs
│   │   │   ├── toml.rs
│   │   │   └── env.rs
│   │   ├── providers/    # Built-in providers
│   │   │   ├── mod.rs
│   │   │   ├── openai.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── google.rs
│   │   │   └── aws.rs
│   │   ├── ffi/         # FFI utilities
│   │   │   ├── mod.rs
│   │   │   ├── types.rs
│   │   │   └── memory.rs
│   │   └── utils/       # Utility functions
│   │       ├── mod.rs
│   │       ├── crypto.rs
│   │       └── paths.rs
│   ├── Cargo.toml
│   └── build.rs
├── ffi/                   # C-API layer
│   ├── include/
│   │   └── aicred.h
│   ├── src/
│   │   ├── lib.rs
│   │   └── c_api.rs
│   └── Cargo.toml
├── cli/                   # Command-line tool
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs
│   │   └── output.rs
│   └── Cargo.toml
├── gui/                   # Tauri application
│   ├── src/
│   │   ├── main.rs
│   │   └── app.rs
│   ├── src-tauri/
│   │   ├── main.rs
│   │   └── commands.rs
│   └── Cargo.toml
└── bindings/              # Language bindings
    ├── python/
    │   ├── src/
    │   │   ├── lib.rs
    │   │   └── python.rs
    │   └── Cargo.toml
    └── go/
        ├── src/
        │   ├── lib.rs
        │   └── go.rs
        └── Cargo.toml
```

## Key Design Decisions

### Error Handling Strategy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AICredError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin load error: {0}")]
    LoadError(String),
    
    #[error("Plugin parse error: {0}")]
    ParseError(String),
}
```

### Async vs Sync API

**Decision**: Synchronous core API with optional async wrapper

**Rationale**:
- File system operations are primarily I/O bound
- Simpler API surface for majority of use cases
- Async wrapper can be provided as separate crate or feature flag

```rust
// Core synchronous API
pub fn scan_keys(config: &ScanConfig) -> Result<ScanResult, AICredError> {
    // Implementation
}

// Optional async wrapper
#[cfg(feature = "async")]
pub async fn scan_keys_async(config: &ScanConfig) -> Result<ScanResult, AICredError> {
    tokio::task::spawn_blocking(|| scan_keys(config)).await?
}
```

### Security Considerations

1. **Key Redaction**: All keys are redacted by default
2. **Hashing**: SHA256 hashes of original values are stored
3. **File Size Limits**: Maximum file size limits to prevent DoS
4. **Path Validation**: Strict path validation to prevent directory traversal
5. **Memory Safety**: Secure memory handling for sensitive data

```rust
pub struct SecurityConfig {
    pub max_file_size: usize,
    pub redact_keys: bool,
    pub store_hashes: bool,
    pub allowed_extensions: Vec<String>,
    pub blocked_paths: Vec<PathBuf>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            redact_keys: true,
            store_hashes: true,
            allowed_extensions: vec![
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "env".to_string(),
                "conf".to_string(),
                "config".to_string(),
            ],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/usr"),
                PathBuf::from("/var"),
            ],
        }
    }
}
```

### Cross-Platform Path Handling

```rust
pub struct PlatformConfig {
    pub config_paths: Vec<PathBuf>,
    pub env_var_names: Vec<String>,
    pub file_extensions: Vec<String>,
}

impl PlatformConfig {
    pub fn for_current_platform() -> Self {
        match std::env::consts::OS {
            "windows" => Self::windows_config(),
            "macos" => Self::macos_config(),
            _ => Self::unix_config(),
        }
    }
    
    fn windows_config() -> Self {
        Self {
            config_paths: vec![
                PathBuf::from("AppData/Roaming"),
                PathBuf::from("AppData/Local"),
                PathBuf::from("AppData/LocalLow"),
            ],
            env_var_names: vec![
                "APPDATA".to_string(),
                "LOCALAPPDATA".to_string(),
                "USERPROFILE".to_string(),
            ],
            file_extensions: vec![
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "ini".to_string(),
                "cfg".to_string(),
            ],
        }
    }
}
```

### File Permission Handling

```rust
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub owner: String,
    pub group: String,
}

impl FilePermissions {
    pub fn from_path(path: &Path) -> Result<Self, AICredError> {
        let metadata = fs::metadata(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let mode = metadata.mode();
            Ok(Self {
                readable: mode & 0o400 != 0,
                writable: mode & 0o200 != 0,
                executable: mode & 0o100 != 0,
                owner: metadata.uid().to_string(),
                group: metadata.gid().to_string(),
            })
        }
        #[cfg(windows)]
        {
            Ok(Self {
                readable: true, // Simplified for Windows
                writable: true,
                executable: false,
                owner: "unknown".to_string(),
                group: "unknown".to_string(),
            })
        }
    }
}
```

## Security and Privacy Design

### Key Redaction Strategy

```rust
pub struct KeyRedactor {
    pub preserve_length: bool,
    pub redaction_char: char,
}

impl KeyRedactor {
    pub fn redact(&self, key: &str) -> String {
        if key.len() <= 8 {
            self.redaction_char.to_string().repeat(key.len())
        } else {
            let prefix = &key[..4];
            let suffix = &key[key.len() - 4..];
            format!("{}{}{}", prefix, self.redaction_char.to_string().repeat(key.len() - 8), suffix)
        }
    }
    
    pub fn redact_with_preview(&self, key: &str) -> String {
        let hash = self.hash_key(key);
        let preview = self.redact(key);
        format!("{} [{}]", preview, &hash[..8])
    }
    
    fn hash_key(&self, key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

### Privacy Controls

```rust
pub struct PrivacyConfig {
    pub collect_metadata: bool,
    pub store_file_paths: bool,
    pub store_line_numbers: bool,
    pub anonymize_paths: bool,
    pub retention_days: Option<u32>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            collect_metadata: true,
            store_file_paths: false, // Don't store full paths by default
            store_line_numbers: true,
            anonymize_paths: true,
            retention_days: Some(30),
        }
    }
}
```

## Testing Strategy

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_redaction() {
        let redactor = KeyRedactor {
            preserve_length: true,
            redaction_char: '*',
        };
        
        let key = "sk-EXAMPLE_FAKE_TOKEN_1234567890abcdef";
        let redacted = redactor.redact(key);
        assert_eq!(redacted, "sk-**************def");
    }
    
    #[test]
    fn test_provider_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(OpenAIPlugin::new());
        registry.register(plugin);
        
        assert_eq!(registry.list_plugins(), vec!["openai"]);
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_full_scan_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(temp_dir.path());
        
        let finder = AICred::new(config).unwrap();
        let result = finder.scan().unwrap();
        
        assert!(!result.keys.is_empty());
        assert!(result.metadata.scan_duration_ms > 0);
    }
}
```

### Security Testing

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_prevention() {
        let malicious_path = "../../../etc/passwd";
        let sanitized = sanitize_path(malicious_path);
        assert_eq!(sanitized, "etc/passwd");
    }
    
    #[test]
    fn test_file_size_limits() {
        let large_content = "x".repeat(20 * 1024 * 1024); // 20MB
        let result = parse_with_size_limit(&large_content, 10 * 1024 * 1024);
        assert!(result.is_err());
    }
}
```

## Performance Considerations

### Scanning Optimizations

```rust
pub struct ScanOptimizer {
    pub max_concurrent_files: usize,
    pub cache_results: bool,
    pub use_memory_map: bool,
}

impl Default for ScanOptimizer {
    fn default() -> Self {
        Self {
            max_concurrent_files: num_cpus::get(),
            cache_results: true,
            use_memory_map: true,
        }
    }
}
```

### Caching Strategy

```rust
pub struct ScanCache {
    pub cache_dir: PathBuf,
    pub ttl_seconds: u64,
    pub max_entries: usize,
}

impl ScanCache {
    pub fn get_or_scan<F>(
        &self,
        key: &str,
        scan_fn: F,
    ) -> Result<ScanResult, AICredError>
    where
        F: FnOnce() -> Result<ScanResult, AICredError>,
    {
        if let Some(cached) = self.get_cached(key)? {
            if !self.is_expired(&cached) {
                return Ok(cached);
            }
        }
        
        let result = scan_fn()?;
        self.cache_result(key, &result)?;
        Ok(result)
    }
}
```

## Future Considerations

### Extensibility Points

1. **Custom Plugins**: Plugin API for third-party providers
2. **Configuration Formats**: Support for new configuration formats
3. **Authentication Methods**: New authentication schemes
4. **Output Formats**: Additional output serialization formats
5. **Integration Points**: IDE plugins, CI/CD integration

### Scalability Considerations

1. **Distributed Scanning**: Support for scanning multiple machines
2. **Cloud Integration**: Cloud provider configuration scanning
3. **Enterprise Features**: LDAP integration, role-based access
4. **Monitoring**: Metrics and monitoring integration
5. **Updates**: Automatic provider definition updates

## Conclusion

This architecture provides a solid foundation for a secure, extensible, and cross-platform GenAI key discovery library. The plugin-based design ensures easy extensibility while maintaining security and performance standards. The modular structure supports multiple deployment scenarios from embedded libraries to standalone applications.

### Key Architectural Changes

**New Separation of Concerns:**
- **ScannerPlugin**: Handles discovery of API keys and configuration files across applications and providers
- **ProviderPlugin**: Validates and scores discovered keys, providing confidence metrics

**Benefits of the New Architecture:**
- Clear separation between discovery and validation logic
- Scanner plugins can discover keys for multiple providers
- Provider plugins focus on key validation and scoring
- More flexible and extensible plugin system
- Better support for application-specific configuration scanning