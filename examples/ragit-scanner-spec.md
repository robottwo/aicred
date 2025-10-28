# Ragit Scanner Specification

## Application Information

**Application Name**: Ragit  
**Description**: A git-like RAG (Retrieval-Augmented Generation) pipeline that turns local files into a knowledge-base with AI-powered querying capabilities  
**GitHub Repository**: https://github.com/baehyunsol/ragit  
**Primary Language**: Rust  
**License**: MIT  

## Configuration Analysis

### Configuration File Formats
- **Primary Format**: JSON
- **Alternative Formats**: Environment variables (.env files)
- **Configuration Structure**: Modular with separate files for different aspects:
  - `api.json` - API configuration
  - `build.json` - Build configuration  
  - `query.json` - Query configuration

### Configuration File Locations

#### macOS
```
~/.config/ragit/
~/.ragit/
.ragit/
```

#### Linux
```
~/.config/ragit/
~/.ragit/
.ragit/
/etc/ragit/
```

#### Windows
```
%APPDATA%/ragit/
%USERPROFILE%/.ragit/
.ragit/
```

### Key Configuration Files
1. **config.json** - Main configuration file (in `.ragit/` directory)
2. **api.json** - API configuration (global defaults in `~/.config/ragit/`)
3. **build.json** - Build configuration (global defaults in `~/.config/ragit/`)
4. **query.json** - Query configuration (global defaults in `~/.config/ragit/`)
5. **.env** - Environment variables
6. **ragit_config.json** - Alternative project-level configuration

## API Key Patterns

### Supported Providers
- OpenAI
- Anthropic
- Google/Gemini
- Hugging Face
- Groq (mentioned in documentation examples)

### Key Validation Rules
- **Minimum length**: 15 characters
- **Required prefixes**: 
  - `sk-` (OpenAI)
  - `sk-ant-` (Anthropic)
  - `hf_` (Hugging Face)
- **Pattern validation**: Keys must contain alphanumeric characters
- **Confidence scoring**:
  - High: Keys starting with `sk-`, `sk-ant-`, or `hf_`
  - Medium: Keys with 30+ characters
  - Low: Other valid keys

### Configuration Examples

#### Example 1: Basic Configuration with API Key
```json
{
  "ragit_version": "1.0.0",
  "api_key": "sk-test1234567890abcdef",
  "model": "gpt-4",
  "chunk_size": 4000,
  "slide_len": 1000
}
```

#### Example 2: Provider-Specific Keys
```json
{
  "ragit_version": "1.0.0",
  "providers": {
    "openai": {
      "api_key": "sk-openai1234567890abcdef",
      "model": "gpt-4"
    },
    "anthropic": {
      "api_key": "sk-ant-anthropic1234567890abcdef",
      "model": "claude-3-sonnet"
    },
    "huggingface": {
      "api_key": "hf_huggingface1234567890abcdef"
    }
  },
  "vector_store": {
    "type": "chroma"
  }
}
```

#### Example 3: Environment Variables
```bash
# .env file or environment exports
RAGIT_API_KEY=sk-test1234567890abcdef
OPENAI_API_KEY=sk-openai1234567890abcdef
ANTHROPIC_API_KEY=sk-ant-anthropic1234567890abcdef
HUGGING_FACE_HUB_TOKEN=hf_huggingface1234567890abcdef
GROQ_API_KEY=groq_test1234567890abcdef
```

#### Example 4: Global Configuration Files
```json
// ~/.config/ragit/api.json
{
  "model": "gpt-4o",
  "dump_log": true,
  "max_retry": 5,
  "timeout": 120000
}
```

#### Example 5: Project-Level Configuration
```json
// .ragit/config.json
{
  "ragit_version": "0.4.5",
  "chunk_size": 4000,
  "slide_len": 1000,
  "image_size": 2000,
  "min_summary_len": 200,
  "max_summary_len": 1000,
  "model": "llama3-70b-8192",
  "max_retry": 5,
  "timeout": 120000,
  "dump_api_usage": true
}
```

## Scanner Implementation

### File Detection Logic
```rust
fn can_handle_file(&self, path: &Path) -> bool {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    
    // Check for Ragit-specific files
    file_name == "config.json" && (
        path.to_string_lossy().contains("ragit") ||
        path.parent()
            .map(|p| p.ends_with(".ragit"))
            .unwrap_or(false)
    ) ||
    file_name == "ragit_config.json"
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
                "ragit".to_string(),
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
    
    // Look for environment variables in config
    if let Some(env_vars) = json_value.get("env").and_then(|v| v.as_object()) {
        for (env_name, env_value) in env_vars {
            if env_name.contains("key") || env_name.contains("token") {
                if let Some(value) = env_value.as_str() {
                    if self.is_valid_key(value) {
                        let provider = self.infer_provider_from_env_name(env_name);
                        keys.push(DiscoveredKey::new(
                            provider,
                            path.display().to_string(),
                            ValueType::ApiKey,
                            self.get_confidence(value),
                            value.to_string(),
                        ));
                    }
                }
            }
        }
    }
    
    if keys.is_empty() { None } else { Some(keys) }
}
```

### Configuration Validation
```rust
fn is_valid_ragit_config(&self, json_value: &serde_json::Value) -> bool {
    // Check for Ragit-specific configuration keys
    json_value.get("ragit_version").is_some()
        || json_value.get("ragit").is_some()
        || json_value.get("vector_store").is_some()
        || json_value.get("chunking").is_some()
        || json_value.get("chunk_size").is_some()
        || json_value.get("model").is_some()
}
```

## Test Cases

### Test Configuration 1: Valid Keys with Version
```json
{
  "ragit_version": "1.0.0",
  "api_key": "sk-test1234567890abcdef",
  "model": "gpt-4",
  "chunk_size": 4000
}
```
Expected: Should find 1 key with high confidence and create 1 config instance

### Test Configuration 2: Multiple Providers with Global Config
```json
{
  "ragit_version": "0.4.5",
  "providers": {
    "openai": { "api_key": "sk-openai1234567890abcdef" },
    "anthropic": { "api_key": "sk-ant-anthropic1234567890abcdef" },
    "huggingface": { "api_key": "hf_huggingface1234567890abcdef" }
  },
  "vector_store": { "type": "chroma" }
}
```
Expected: Should find 3 keys with high confidence and create 1 config instance

### Test Configuration 3: Environment Variables in Config
```json
{
  "ragit_version": "1.0.0",
  "env": {
    "OPENAI_API_KEY": "sk-env-openai1234567890abcdef",
    "ANTHROPIC_API_KEY": "sk-env-anthropic1234567890abcdef"
  }
}
```
Expected: Should find 2 keys with medium/high confidence

### Test Configuration 4: Invalid Keys
```json
{
  "ragit_version": "1.0.0",
  "api_key": "short-key",
  "model": "gpt-4"
}
```
Expected: Should find 0 keys (invalid key format - too short)

### Test Configuration 5: Non-Ragit Config
```json
{
  "random_key": "sk-test1234567890abcdef",
  "other_setting": "value"
}
```
Expected: Should find 0 keys and 0 instances (not a valid Ragit config)

## Implementation Notes

### Scanner Name
`ragit`

### Supported Providers
- ragit (generic)
- openai
- anthropic
- huggingface
- google

### Platform Considerations
- **Cross-platform compatibility**: Works on Linux, macOS, and Windows
- **Global configs**: Stored in `~/.config/ragit/` (XDG Base Directory Specification)
- **Project configs**: Stored in `.ragit/` directory within projects
- **Environment variables**: Can be stored in `.env` files or system environment

### Security Considerations
- **Key validation**: Minimum 15 characters with alphanumeric content
- **Confidence scoring**: Based on key prefixes and length
- **Safe handling**: Keys are processed in memory and not logged
- **False positive reduction**: Validates against Ragit-specific configuration patterns

### Configuration Discovery
The scanner looks for configurations in this priority order:
1. Global configuration: `~/.config/ragit/{api,build,query}.json`
2. User configuration: `~/.ragit/config.json`
3. Project configuration: `.ragit/config.json`
4. Alternative project config: `ragit_config.json`
5. Environment files: `.env`, `.env.local`, `ragit.env`

### Key Features
- **Git-like workflow**: Uses similar commands (`rag init`, `rag add`, `rag build`, `rag query`)
- **Multi-provider support**: Can configure multiple AI providers simultaneously
- **Modular configuration**: Separate config files for different aspects (API, build, query)
- **Knowledge-base sharing**: Supports cloning and pushing knowledge-bases like git
- **Advanced RAG features**: TF-IDF scoring, multi-turn queries, image support

## References
- [Ragit GitHub Repository](https://github.com/baehyunsol/ragit)
- [Ragit Configuration Documentation](https://github.com/baehyunsol/ragit/blob/main/docs/config.md)
- [Ragit README](https://github.com/baehyunsol/ragit/blob/main/README.md)
- [Ragit Crates Structure](https://github.com/baehyunsol/ragit/tree/main/crates)