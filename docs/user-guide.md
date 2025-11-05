# User Guide

This guide walks through using AICred via the CLI, Python, Go, Rust library, and the GUI. It also covers common workflows and troubleshooting.

## CLI Usage

The CLI binary is `aicred`.

### Commands

- `aicred scan` — Scan for GenAI credentials and configurations
- `aicred providers` — Show available providers and application scanners
- `aicred instances` — List provider instances with their configurations
- `aicred tags` — Manage tags for organizing provider instances and models
- `aicred labels` — Manage labels for unique categorization of provider instances and models
- `aicred version` — Show version information

### Scan Options

```bash
# Basic scan with default table output
aicred scan

# Output formats: table (default), json, ndjson, summary
aicred scan --format json
aicred scan --format ndjson
aicred scan --format summary

# Set home directory to scan
aicred scan --home /path/to/home

# Include full secret values (DANGEROUS - use with caution)
aicred scan --include-values

# Filter by providers
aicred scan --only openai,anthropic
aicred scan --exclude huggingface

# Limit file size (bytes)
aicred scan --max-bytes-per-file 2097152

# Dry run (no file reads), print what would be scanned
aicred scan --dry-run

# Write an audit log
aicred scan --audit-log scan-audit.log
```

### Provider Instance Management

The `aicred instances` command allows you to manage provider instances with their configurations:

```bash
# List all provider instances (default behavior)
aicred instances

# Get detailed information about a specific instance using shorthand syntax
aicred instances my-openai

# Get instance information with full secret values (DANGEROUS - use with caution)
aicred instances my-openai --include-values

# List instances with detailed information
aicred instances list --verbose

# Filter instances by provider type
aicred instances list --provider-type openai

# Show only active instances
aicred instances list --active-only

# Add a new provider instance
aicred instances add --id my-openai --name "My OpenAI" --provider-type openai --base-url https://api.openai.com/v1 --models gpt-4,gpt-3.5-turbo

# Remove an instance
aicred instances remove --id my-openai

# Update an existing instance
aicred instances update --id my-openai --name "Updated OpenAI" --active false

# Get detailed information about a specific instance (alternative syntax)
aicred instances get --id my-openai

# Validate instance configurations
aicred instances validate
### Tag and Label Management

The tagging and labeling system helps you organize and categorize your provider instances and models:

- **Tags**: Non-unique identifiers that can be applied to multiple targets
- **Labels**: Unique identifiers that can only be assigned to one target at a time

#### Tag Management

```bash
# List all tags
aicred tags list

# Add a new tag
aicred tags add --name "Production" --color "#ff0000" --description "Production environment"

# Update an existing tag
aicred tags update --name "Production" --color "#00ff00"

# Remove a tag (with confirmation if assigned)
aicred tags remove --name "Production"

# Force remove a tag and all its assignments
aicred tags remove --name "Production" --force

# Assign a tag to a provider instance
aicred tags assign --name "Production" --instance-id my-openai

# Assign a tag to a specific model
aicred tags assign --name "GPT-4" --instance-id my-openai --model-id gpt-4

# Unassign a tag from a target
aicred tags unassign --name "Production" --instance-id my-openai
```

#### Label Management

```bash
# List all labels
aicred labels list

# Add a new label
aicred labels add --name "Primary" --color "#17c964" --description "Primary provider instance"

# Update an existing label
aicred labels update --name "Primary" --color "#00ff00"

# Remove a label (must be unassigned first)
aicred labels remove --name "Primary"

# Assign a label to a provider instance
aicred labels assign --name "Primary" --instance-id my-openai

# Assign a label to a specific model
aicred labels assign --name "Fast-Model" --instance-id my-openai --model-id gpt-3.5-turbo

# Unassign a label from a target
### Environment Variable Commands

The `wrap` and `setenv` commands provide seamless integration between your labeled provider instances and applications by automatically resolving labels to environment variables.

#### Wrap Command

The `wrap` command executes commands with environment variables automatically resolved from label mappings. This allows you to run applications with the correct API keys and configurations without manually setting environment variables.

**Basic Usage:**
```bash
# Run a command with environment variables from resolved labels
aicred wrap --labels fast -- python my_script.py

# Use multiple labels (comma-separated)
aicred wrap --labels fast,smart -- npm run dev

# Dry run to preview environment variables without executing
aicred wrap --labels fast --dry-run -- echo "Preview mode"
```

**Scanner-Specific Usage:**

Different scanners generate different environment variable patterns. Specify the scanner type to match your application's expectations:

```bash
# GSH scanner (default) - generates GSH_* variables
aicred wrap --scanner gsh --labels fast -- python app.py

# Roo Code scanner - generates ROO_CODE_* variables
aicred wrap --scanner roo-code --labels primary -- code .

# Claude Desktop scanner - generates ANTHROPIC_* variables
aicred wrap --scanner claude-desktop --labels smart -- claude-desktop

# RAGIt scanner - generates RAGIT_* variables
aicred wrap --scanner ragit --labels fast -- ragit query

# LangChain scanner - generates LANGCHAIN_* variables
aicred wrap --scanner langchain --labels smart -- python langchain_app.py
```

**Advanced Options:**
```bash
# Specify custom home directory for configuration
aicred wrap --home /path/to/config --labels fast -- python app.py

# Combine multiple options
aicred wrap --scanner gsh --labels fast,smart --dry-run -- python app.py
```

**Dry Run Output Example:**
```bash
$ aicred wrap --labels fast --dry-run -- python app.py
Environment variables that would be set:
  GSH_FAST_MODEL=groq:llama3-70b-8192
  GSH_FAST_API_KEY=gsk_...xyz
  GSH_FAST_BASE_URL=https://api.groq.com/openai/v1
```

#### SetEnv Command

The `setenv` command generates shell-specific export statements for environment variables, allowing you to source them into your current shell session or use them in scripts.

**Basic Usage:**
```bash
# Generate bash/zsh format (default)
aicred setenv --labels fast --format bash

# Generate fish shell format
aicred setenv --labels fast --format fish

# Generate PowerShell format
aicred setenv --labels fast --format powershell
```

**Output Formats:**

**Bash/Zsh:**
```bash
$ aicred setenv --labels fast --format bash
export GSH_FAST_MODEL='groq:llama3-70b-8192'
export GSH_FAST_API_KEY='gsk_...'
export GSH_FAST_BASE_URL='https://api.groq.com/openai/v1'

# Source into current shell
eval "$(aicred setenv --labels fast --format bash)"
```

**Fish Shell:**
```bash
$ aicred setenv --labels fast --format fish
set -gx GSH_FAST_MODEL 'groq:llama3-70b-8192'
set -gx GSH_FAST_API_KEY 'gsk_...'
set -gx GSH_FAST_BASE_URL 'https://api.groq.com/openai/v1'

# Source into current shell
aicred setenv --labels fast --format fish | source
```

**PowerShell:**
```powershell
PS> aicred setenv --labels fast --format powershell
$env:GSH_FAST_MODEL = 'groq:llama3-70b-8192'
$env:GSH_FAST_API_KEY = 'gsk_...'
$env:GSH_FAST_BASE_URL = 'https://api.groq.com/openai/v1'

# Execute in current session
aicred setenv --labels fast --format powershell | Invoke-Expression
```

**Multiple Labels:**
```bash
# Generate variables for multiple labels
aicred setenv --labels fast,smart --format bash
# Output:
# export GSH_FAST_MODEL='groq:llama3-70b-8192'
# export GSH_FAST_API_KEY='gsk_...'
# export GSH_FAST_BASE_URL='https://api.groq.com/openai/v1'
# export GSH_SMART_MODEL='openrouter:anthropic/claude-3-opus'
# export GSH_SMART_API_KEY='sk-or-...'
# export GSH_SMART_BASE_URL='https://openrouter.ai/api/v1'
```

**Scanner-Specific Variables:**
```bash
# GSH scanner variables
aicred setenv --scanner gsh --labels fast --format bash
# Generates: GSH_FAST_MODEL, GSH_FAST_API_KEY, GSH_FAST_BASE_URL

# Roo Code scanner variables
aicred setenv --scanner roo-code --labels primary --format bash
# Generates: ROO_CODE_API_KEY, ROO_CODE_MODEL_ID, ROO_CODE_BASE_URL

# Claude Desktop scanner variables
aicred setenv --scanner claude-desktop --labels smart --format bash
# Generates: ANTHROPIC_API_KEY, CLAUDE_MODEL_ID
```

**Dry Run Mode:**
```bash
# Preview variables with masked secrets
aicred setenv --labels fast --dry-run
# Output:
# Environment variables that would be exported:
#   GSH_FAST_MODEL=groq:llama3-70b-8192
#   GSH_FAST_API_KEY=gsk_...xyz
#   GSH_FAST_BASE_URL=https://api.groq.com/openai/v1
```

#### Label-to-Environment Variable Mapping

When you assign labels to provider instances, the system automatically generates environment variables following scanner-specific patterns:

**Standard Variable Pattern:**
- `{SCANNER}_{LABEL}_MODEL` - The provider:model tuple (e.g., `GSH_FAST_MODEL=openai:gpt-4`)
- `{SCANNER}_{LABEL}_API_KEY` - The API key for the provider (e.g., `GSH_FAST_API_KEY=sk-...`)
- `{SCANNER}_{LABEL}_BASE_URL` - The base URL for the API (e.g., `GSH_FAST_BASE_URL=https://api.openai.com/v1`)
- `{SCANNER}_{LABEL}_{METADATA_KEY}` - Any custom metadata from the provider instance

**Scanner-Specific Environment Variables:**

**GSH Scanner:**
- `GSH_{LABEL}_MODEL` - Provider:model tuple
- `GSH_{LABEL}_API_KEY` - API key
- `GSH_{LABEL}_BASE_URL` - Base URL
- `GSH_{LABEL}_TEMPERATURE` - Temperature setting (optional)
- `GSH_{LABEL}_PARALLEL_TOOL_CALLS` - Parallel tool calls setting (optional)

**Roo Code Scanner:**
- `ROO_CODE_API_KEY` - Anthropic API key
- `ROO_CODE_MODEL_ID` - Model identifier
- `ROO_CODE_BASE_URL` - API base URL
- `ROO_CODE_TEMPERATURE` - Temperature setting
- `ROO_CODE_PARALLEL_TOOL_CALLS` - Parallel tool calls setting

**Claude Desktop Scanner:**
- `ANTHROPIC_API_KEY` - Anthropic API key
- `CLAUDE_MODEL_ID` - Model identifier
- `CLAUDE_BASE_URL` - API base URL

**RAGIt Scanner:**
- `RAGIT_API_KEY` - API key for RAG operations
- `RAGIT_MODEL_ID` - Model identifier
- `RAGIT_BASE_URL` - API base URL
- `RAGIT_EMBEDDING_MODEL` - Embedding model (optional)

**LangChain Scanner:**
- `LANGCHAIN_API_KEY` - API key
- `LANGCHAIN_MODEL_ID` - Model identifier
- `LANGCHAIN_BASE_URL` - API base URL
- `LANGCHAIN_TEMPERATURE` - Temperature setting
- `LANGCHAIN_MAX_TOKENS` - Maximum tokens (optional)

**Complete Workflow Example:**
```bash
# 1. Create a label for fast models
aicred labels add --name "fast" --description "Fast model for quick tasks" --color "#00ff00"

# 2. Assign label to a specific provider instance and model
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192

# 3. Use the label in your application with wrap
aicred wrap --labels fast -- python my_app.py

# 4. Or generate environment variables for manual use
aicred setenv --labels fast --format bash
# Output:
# export GSH_FAST_MODEL='groq:llama3-70b-8192'
# export GSH_FAST_API_KEY='gsk_...'
# export GSH_FAST_BASE_URL='https://api.groq.com/openai/v1'

# 5. Source into your shell
eval "$(aicred setenv --labels fast --format bash)"

# 6. Now your application can use the environment variables
python my_app.py  # Will use GSH_FAST_* variables
```

**Multi-Label Workflow:**
```bash
# Create multiple labels for different use cases
aicred labels add --name "fast" --description "Fast model"
aicred labels add --name "smart" --description "Smart model"

# Assign to different models
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192
aicred labels assign --name "smart" --instance-id my-openrouter --model-id anthropic/claude-3-opus

# Use both labels in your application
aicred wrap --labels fast,smart -- python multi_model_app.py

# The application now has access to both:
# GSH_FAST_MODEL, GSH_FAST_API_KEY, GSH_FAST_BASE_URL
# GSH_SMART_MODEL, GSH_SMART_API_KEY, GSH_SMART_BASE_URL
```

aicred labels unassign --name "Primary" --instance-id my-openai
```

#### Tag vs Label Usage Patterns

**Use Tags when:**
- You want to categorize multiple instances with the same attribute (e.g., "Production", "Development", "Testing")
- You need to apply the same label to multiple targets
- You're organizing by environment, team, or other shared characteristics

**Use Labels when:**
- You need to uniquely identify a specific instance or model
- You want to ensure only one target has a particular designation
- You're marking something as "Primary", "Backup", "Deprecated", etc.

#### Example Workflows

**Environment-based organization:**
```bash
# Create environment tags
aicred tags add --name "Production" --color "#ff0000"
aicred tags add --name "Staging" --color "#ffa500"
aicred tags add --name "Development" --color "#00ff00"

# Assign environment tags
aicred tags assign --name "Production" --instance-id openai-prod
aicred tags assign --name "Staging" --instance-id openai-staging
aicred tags assign --name "Development" --instance-id openai-dev
```

**Primary/Backup labeling:**
```bash
# Create labels for primary/backup designation
aicred labels add --name "Primary" --color "#17c964"
aicred labels add --name "Backup" --color "#f5a524"

# Assign labels (only one can be "Primary" at a time)
aicred labels assign --name "Primary" --instance-id openai-prod
aicred labels assign --name "Backup" --instance-id openai-backup
```
```

### Output Formats

- Table: human-friendly overview
- JSON: entire result object as JSON
- NDJSON: one JSON object per line for each key and config instance
- Summary: high-level stats by provider and counts

Example JSON (shape aligned with core library serialization):
```json
{
  "keys": [
    {
      "provider": "openai",
      "source": "/home/user/.env",
      "value_type": "ApiKey",
      "confidence": "High",
      "hash": "e3b0c44298fc1c149afbf4c8996fb924...",
      "discovered_at": "2025-01-20T10:30:00Z",
      "line_number": 5,
      "column_number": 10,
      "metadata": null
    }
  ],
  "config_instances": [],
  "scan_started_at": "2025-01-20T10:30:00Z",
  "scan_completed_at": "2025-01-20T10:30:01Z",
  "home_directory": "/home/user",
  "providers_scanned": ["openai", "anthropic", "huggingface"],
  "files_scanned": 42,
  "directories_scanned": 11,
  "metadata": null
}
```

Notes:
- Full secret values are never included unless `--include-values` is set. Even then, safer formats omit them by default in many workflows.
- A short redacted preview is not directly serialized; use the SHA-256 `hash` for deduplication and display a generic `****` preview, or compute previews only when you hold the full value.

## How Key Discovery Works

The AICred uses a **two-phase approach**:

1. **Discovery Phase**: ScannerPlugin implementations scan for configuration files and extract potential API keys
2. **Validation Phase**: ProviderPlugin implementations validate discovered keys and assign confidence scores

### Scanner Plugins
Scanner plugins discover keys across different applications:
- **Application scanners**: Find keys in application-specific configs (e.g., Roo Code, Claude Desktop)
- **Provider scanners**: Find keys in provider-specific locations (e.g., .env files, provider config files)

### Provider Plugins  
Provider plugins validate and score discovered keys:
- Assign confidence scores based on key patterns and characteristics
- Validate that keys match expected formats for specific providers

## Python Usage

Install:
```bash
pip install aicred
```

Basic usage:
```python
import aicred

# Default scan (user home)
result = aicred.scan()
print(f"Found {len(result['keys'])} keys")

# Filter providers and include full values (dangerous)
result = aicred.scan(
    include_full_values=False,
    only_providers=["openai", "anthropic"],
    max_file_size=1_048_576
)

# Iterate keys
for key in result["keys"]:
    # A redacted preview string is not serialized by default. Use a placeholder.
    print(f"{key['provider']}: **** (confidence={key['confidence']})")
```

Available functions:
- `aicred.scan(home_dir=None, include_full_values=False, max_file_size=1048576, only_providers=None, exclude_providers=None) -> dict`
- `aicred.version() -> str`
- `aicred.list_providers() -> list[str]`
- `aicred.list_scanners() -> list[str]`

Return schema matches the JSON example in CLI output.

## Go Usage

Install:
```bash
go get github.com/robottwo/aicred/bindings/go
```

Example:
```go
package main

import (
  "fmt"
  "log"

  "github.com/robottwo/aicred/bindings/go"
)

func main() {
  result, err := aicred.Scan(aicred.ScanOptions{
    HomeDir:           "",     // default to user home
    IncludeFullValues: false,  // keep secrets redacted
    MaxFileSize:       1048576,
    OnlyProviders:     []string{"openai", "anthropic"},
  })
  if err != nil {
    log.Fatal(err)
  }

  fmt.Printf("Found %d keys\n", len(result.Keys))
  for _, k := range result.Keys {
    // Redacted field may be empty if full values are not included; display "****"
    redacted := k.Redacted
    if redacted == "" {
      redacted = "****"
    }
    fmt.Printf("%s: %s (hash=%s)\n", k.Provider, redacted, k.Hash)
  }
}
```

Notes:
- The Go binding maps the core JSON to Go structs. Some optional convenience fields like `Redacted` may be blank depending on configuration; rely on `Hash` for deduping.

## Rust Library Usage

Add dependency (from crates.io or path):
```toml
[dependencies]
aicred-core = "0.1.0"
```

Basic scan:
```rust
use aicred_core::{scan, ScanOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ScanOptions::default();
    let result = scan(options)?;
    println!("Found {} keys", result.total_keys());
    Ok(())
}
```

Filter providers:
```rust
use aicred_core::{scan, ScanOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ScanOptions::default()
        .with_only_providers(vec!["openai".into(), "anthropic".into()]);
    let result = scan(options)?;
    println!("Providers scanned: {:?}", result.providers_scanned);
    Ok(())
}
```

Advanced scanning with custom plugins:
```rust
use aicred_core::{
    scanner::{Scanner, ScannerConfig},
    plugins::{PluginRegistry, ProviderPlugin},
    scanners::{ScannerRegistry, ScannerPlugin},
    models::{DiscoveredKey, ScanResult},
    Result,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Custom scanner plugin
struct MyAppScanner;
impl ScannerPlugin for MyAppScanner {
    fn name(&self) -> &str { "my-app" }
    fn app_name(&self) -> &str { "My Application" }
    fn scan_paths(&self, home_dir: &Path) -> Vec<PathBuf> {
        vec![home_dir.join(".my-app").join("config.json")]
    }
    fn parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult> {
        // Parse content and return discovered keys
        Ok(scanners::ScanResult::new())
    }
    fn can_handle_file(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "json")
    }
}

// Custom provider plugin
struct MyProvider;
impl ProviderPlugin for MyProvider {
    fn name(&self) -> &str { "my-provider" }
    fn confidence_score(&self, key: &str) -> f32 { 0.9 }
}

fn main() -> Result<()> {
    // Create registries
    let provider_registry = PluginRegistry::new();
    let scanner_registry = ScannerRegistry::new();
    
    // Register custom plugins
    provider_registry.register(Arc::new(MyProvider))?;
    scanner_registry.register(Arc::new(MyAppScanner))?;
    
    // Create scanner with registries
    let scanner_config = ScannerConfig::default();
    let scanner = Scanner::with_config(provider_registry, scanner_config)
        .with_scanner_registry(scanner_registry);
    
    // Run scan
    let result = scanner.scan(&dirs_next::home_dir().unwrap())?;
    println!("Found {} keys", result.total_keys());
    Ok(())
}
```

## GUI Application

The Tauri-based GUI provides:
- Provider selection and scan options
- Start/stop scan
- Results view for keys and configuration instances
- Export results as JSON

Build:
```bash
cd gui
npm install
npm run tauri build
```

Run in dev:
```bash
cd gui
npm run dev
```

## Common Workflows

### Security Audit (no values, summary):
```bash
aicred scan --format summary --audit-log audit.log
```

### Targeted Providers:
```bash
aicred scan --only openai,anthropic --format json
```

### Application-Specific Scanning:
```bash
# Scan for specific applications
aicred scan --only roo-code,claude-desktop
```

### Provider Key Discovery:
```bash
# Scan for provider keys in standard locations
aicred scan --only openai,anthropic,huggingface
```

### Dry Run Planning:
```bash
aicred scan --dry-run --format json > would-scan.json
```

### Programmatic Use (Python):
```python
import aicred

# Scan for specific applications
res = aicred.scan(only_providers=["roo-code", "claude-desktop"])
if not res["keys"]:
    print("No credentials found")

# Scan for provider keys
res = aicred.scan(only_providers=["openai", "anthropic"])
print(f"Found {len(res['keys'])} provider keys")
```

## Troubleshooting

### No output or empty results:
- Ensure the home directory is correct: `--home /path`
- Increase file size limit: `--max-bytes-per-file 2097152`
- Verify provider list: `aicred providers`
- Check if scanners are available: `aicred providers --verbose`
- Try scanning for applications: `aicred scan --only roo-code,claude-desktop`

### Permission errors when reading files:
- Run with appropriate permissions or exclude problematic paths
- Use `--dry-run` to see planned targets

### Python import issues:
- Verify Python version (3.8+)
- On source builds, ensure `maturin develop` succeeded in `bindings/python`

### Go linking issues:
- Make sure the FFI library is built (`cargo build -p aicred-ffi --release`)
- Ensure runtime library path is resolvable (see `bindings/go/README.md`)

### GUI build issues:
- Node.js 18+ required
- Ensure Tauri CLI is installed (`npm install` in `gui`)

## Notes on Key Discovery Architecture

### How It Works
1. **Scanner plugins** discover configuration files and extract potential API keys
2. **Provider plugins** validate discovered keys and assign confidence scores
3. **Application scanners** find keys in application-specific configurations
4. **Provider scanners** find keys in standard provider locations (`.env`, provider configs)

### Scanner vs Provider Plugins
- **ScannerPlugin**: Handles discovery - finds keys in files and applications
- **ProviderPlugin**: Handles validation - scores and validates discovered keys

### Benefits of the New Architecture
- Clear separation between discovery and validation logic
- Scanner plugins can discover keys for multiple providers
- Provider plugins focus on key validation and scoring
- Better support for application-specific configuration scanning
- More flexible and extensible plugin system

## Configuration Validation and Rewrite

The AICred uses a validation-and-rewrite approach for configuration files. When invalid configurations are encountered, they are automatically replaced with default settings rather than attempting complex migrations.

### Automatic Configuration Handling
When loading configuration files, the system validates them against current requirements:

- **Valid configurations**: Loaded and used as-is
- **Invalid configurations**: Automatically replaced with default settings
- **Missing configurations**: Created with default settings

This approach ensures:
- **Simplicity**: No complex migration logic to maintain
- **Reliability**: Invalid configs are replaced with known-good defaults
- **Consistency**: All configurations follow current format requirements

## Notes on Redaction

- By default, full values are not serialized; the `full_value` is skipped.
- The core provides consistent SHA-256 hashes for deduplication.
- Display redactions as a generic `****` unless you explicitly include full values during scanning and compute previews client-side.