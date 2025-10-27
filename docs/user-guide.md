# User Guide

This guide walks through using GenAI Key Finder via the CLI, Python, Go, Rust library, and the GUI. It also covers common workflows and troubleshooting.

## CLI Usage

The CLI binary is `keyfinder`.

### Commands

- `keyfinder scan` — Scan for GenAI credentials and configurations
- `keyfinder list` — List available providers and application scanners
- `keyfinder version` — Show version information

### Scan Options

```bash
# Basic scan with default table output
keyfinder scan

# Output formats: table (default), json, ndjson, summary
keyfinder scan --format json
keyfinder scan --format ndjson
keyfinder scan --format summary

# Set home directory to scan
keyfinder scan --home /path/to/home

# Include full secret values (DANGEROUS - use with caution)
keyfinder scan --include-values

# Filter by providers
keyfinder scan --only openai,anthropic
keyfinder scan --exclude huggingface

# Limit file size (bytes)
keyfinder scan --max-bytes-per-file 2097152

# Dry run (no file reads), print what would be scanned
keyfinder scan --dry-run

# Write an audit log
keyfinder scan --audit-log scan-audit.log
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

The GenAI KeyFinder uses a **two-phase approach**:

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
pip install genai-keyfinder
```

Basic usage:
```python
import genai_keyfinder

# Default scan (user home)
result = genai_keyfinder.scan()
print(f"Found {len(result['keys'])} keys")

# Filter providers and include full values (dangerous)
result = genai_keyfinder.scan(
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
- `genai_keyfinder.scan(home_dir=None, include_full_values=False, max_file_size=1048576, only_providers=None, exclude_providers=None) -> dict`
- `genai_keyfinder.version() -> str`
- `genai_keyfinder.list_providers() -> list[str]`
- `genai_keyfinder.list_scanners() -> list[str]`

Return schema matches the JSON example in CLI output.

## Go Usage

Install:
```bash
go get github.com/robottwo/aicred/bindings/go/genai_keyfinder
```

Example:
```go
package main

import (
  "fmt"
  "log"

  "github.com/robottwo/aicred/bindings/go/genai_keyfinder"
)

func main() {
  result, err := genai_keyfinder.Scan(genai_keyfinder.ScanOptions{
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
genai-keyfinder-core = "0.1.0"
```

Basic scan:
```rust
use genai_keyfinder_core::{scan, ScanOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ScanOptions::default();
    let result = scan(options)?;
    println!("Found {} keys", result.total_keys());
    Ok(())
}
```

Filter providers:
```rust
use genai_keyfinder_core::{scan, ScanOptions};

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
use genai_keyfinder_core::{
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
keyfinder scan --format summary --audit-log audit.log
```

### Targeted Providers:
```bash
keyfinder scan --only openai,anthropic --format json
```

### Application-Specific Scanning:
```bash
# Scan for specific applications
keyfinder scan --only roo-code,claude-desktop
```

### Provider Key Discovery:
```bash
# Scan for provider keys in standard locations
keyfinder scan --only openai,anthropic,huggingface
```

### Dry Run Planning:
```bash
keyfinder scan --dry-run --format json > would-scan.json
```

### Programmatic Use (Python):
```python
import genai_keyfinder

# Scan for specific applications
res = genai_keyfinder.scan(only_providers=["roo-code", "claude-desktop"])
if not res["keys"]:
    print("No credentials found")

# Scan for provider keys
res = genai_keyfinder.scan(only_providers=["openai", "anthropic"])
print(f"Found {len(res['keys'])} provider keys")
```

## Troubleshooting

### No output or empty results:
- Ensure the home directory is correct: `--home /path`
- Increase file size limit: `--max-bytes-per-file 2097152`
- Verify provider list: `keyfinder list`
- Check if scanners are available: `keyfinder list --scanners`
- Try scanning for applications: `keyfinder scan --only roo-code,claude-desktop`

### Permission errors when reading files:
- Run with appropriate permissions or exclude problematic paths
- Use `--dry-run` to see planned targets

### Python import issues:
- Verify Python version (3.8+)
- On source builds, ensure `maturin develop` succeeded in `bindings/python`

### Go linking issues:
- Make sure the FFI library is built (`cargo build -p genai-keyfinder-ffi --release`)
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

## Configuration Migration

The GenAI Key Finder now supports automatic migration from legacy `ProviderConfig` format to the new `ProviderInstance` format. This migration happens automatically when loading old configuration files.

### Automatic Migration
When you load a configuration file, the system automatically detects if it's in the old format and migrates it:

```python
import genai_keyfinder

# Old format will be automatically migrated
result = genai_keyfinder.scan()
```

### Manual Migration (Advanced)
For more control over the migration process, you can use the migration API:

```rust
use genai_keyfinder_core::models::{ProviderConfigMigrator, MigrationConfig};

let migration_config = MigrationConfig::new()
    .with_instance_prefix("migrated".to_string())
    .with_auto_activation(true);

// Migrate configurations
let (instances, result) = ProviderConfigMigrator::migrate_configs(
    old_configs,
    "openai",
    "https://api.openai.com",
    &migration_config
)?;
```

### Migration Benefits
- **Backward Compatibility**: Old configurations continue to work
- **Enhanced Metadata**: New instances include provider type and base URL
- **Better Organization**: Instance-based management with unique IDs
- **Future-Proof**: Ready for upcoming features like multi-region support

See the [ProviderConfig Migration Guide](provider-config-migration.md) for detailed migration information.

## Notes on Redaction

- By default, full values are not serialized; the `full_value` is skipped.
- The core provides consistent SHA-256 hashes for deduplication.
- Display redactions as a generic `****` unless you explicitly include full values during scanning and compute previews client-side.