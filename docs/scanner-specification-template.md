# Scanner Specification Template

This template demonstrates the format for scanner specifications that the 🔍 Scanner Researcher mode creates.

## Application Information

**Application Name**: [App Name]
**Description**: [Brief description of the application]
**GitHub Repository**: [Repository URL]
**Primary Language**: [Language/framework]
**License**: [License type]

## Configuration Analysis

### Configuration File Formats
- **Primary Format**: [JSON/YAML/TOML/etc.]
- **Alternative Formats**: [List any alternative formats supported]
- **Environment Variables**: [Yes/No - list key patterns]

### Configuration File Locations

#### macOS
```
~/Library/Application Support/[AppName]/
~/.config/[appname]/
~/.[appname]/
```

#### Linux
```
~/.config/[appname]/
~/.[appname]/
/etc/[appname]/
```

#### Windows
```
%APPDATA%/[AppName]/
%USERPROFILE%/.[appname]/
```

### Key Configuration Files
1. **config.json** - Main configuration file
2. **settings.yaml** - User preferences
3. **.env** - Environment variables
4. **[appname].json** - Application-specific config

## API Key Patterns

### Supported Providers
- OpenAI
- Anthropic
- Google/Gemini
- Hugging Face
- [Other providers]

### Key Validation Rules
- Minimum length: [X] characters
- Required prefixes: [List prefixes like "sk-", "sk-ant-", "hf_"]
- Pattern validation: [Regex patterns if applicable]

### Configuration Examples

#### Example 1: Basic Configuration
```json
{
  "api_key": "sk-test1234567890abcdef",
  "model": "gpt-4",
  "temperature": 0.7
}
```

#### Example 2: Provider-Specific Keys
```json
{
  "providers": {
    "openai": {
      "api_key": "sk-openai1234567890abcdef"
    },
    "anthropic": {
      "api_key": "sk-ant-anthropic1234567890abcdef"
    }
  }
}
```

#### Example 3: Environment Variables
```bash
export APPNAME_OPENAI_API_KEY="sk-test1234567890abcdef"
export APPNAME_ANTHROPIC_API_KEY="sk-ant-test1234567890abcdef"
```

## Scanner Implementation

### File Detection Logic
```rust
fn can_handle_file(&self, path: &Path) -> bool {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let path_str = path.to_string_lossy();
    
    // Check for app-specific files
    file_name == "config.json" && path_str.contains("[appname]") ||
    file_name == "[appname].json" ||
    file_name == ".env" && path_str.contains("[appname]")
}
```

### Key Extraction Logic
```rust
fn extract_keys_from_json(&self, json_value: &serde_json::Value, path: &Path) -> Option<Vec<DiscoveredKey>> {
    let mut keys = Vec::new();
    
    // Look for API keys in common locations
    if let Some(api_key) = json_value.get("api_key").and_then(|v| v.as_str()) {
        if self.is_valid_key(api_key) {
            keys.push(DiscoveredKey::new(
                "[provider]".to_string(),
                path.display().to_string(),
                ValueType::ApiKey,
                self.get_confidence(api_key),
                api_key.to_string(),
            ));
        }
    }
    
    // Look for provider-specific keys
    if let Some(providers) = json_value.get("providers").and_then(|v| v.as_object()) {
        for (provider_name, provider_config) in providers {
            if let Some(key) = provider_config.get("api_key").and_then(|v| v.as_str()) {
                if self.is_valid_key(key) {
                    keys.push(DiscoveredKey::new(
                        provider_name.clone(),
                        path.display().to_string(),
                        ValueType::ApiKey,
                        self.get_confidence(key),
                        key.to_string(),
                    ));
                }
            }
        }
    }
    
    if keys.is_empty() { None } else { Some(keys) }
}
```

## Test Cases

### Test Configuration 1: Valid Keys
```json
{
  "api_key": "sk-test1234567890abcdef",
  "model": "gpt-4"
}
```
Expected: Should find 1 key with high confidence

### Test Configuration 2: Multiple Providers
```json
{
  "providers": {
    "openai": { "api_key": "sk-openai1234567890abcdef" },
    "anthropic": { "api_key": "sk-ant-anthropic1234567890abcdef" }
  }
}
```
Expected: Should find 2 keys with high confidence

### Test Configuration 3: Invalid Keys
```json
{
  "api_key": "short-key",
  "model": "gpt-4"
}
```
Expected: Should find 0 keys (invalid key format)

## Implementation Notes

### Scanner Name
`[app-name]` (e.g., "claude-desktop", "langchain")

### Supported Providers
List all providers that this scanner can detect

### Platform Considerations
- macOS-specific paths
- Linux-specific paths  
- Windows-specific paths
- Cross-platform compatibility

### Security Considerations
- Key validation to avoid false positives
- Confidence scoring based on key patterns
- Safe handling of sensitive data

## References
- [Application Documentation URL]
- [GitHub Repository](repository-url)
- [Configuration Examples URL]
- [Related Issues/Discussions]