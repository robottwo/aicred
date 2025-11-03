# API Reference

Authoritative reference for all public surfaces of GenAI Key Finder across Core (Rust), FFI/C-API, Python, Go, and the Tauri GUI interface types.

This reference aligns with the source tree:
- Core library: [core/src/lib.rs](core/src/lib.rs)
- Models: [core/src/models/](core/src/models/)
- Plugins: [core/src/plugins/mod.rs](core/src/plugins/mod.rs)
- Scanners: [core/src/scanners/mod.rs](core/src/scanners/mod.rs)
- Scanner engine: [core/src/scanner/mod.rs](core/src/scanner/mod.rs)
- FFI: [ffi/include/genai_keyfinder.h](ffi/include/genai_keyfinder.h), [ffi/src/lib.rs](ffi/src/lib.rs)
- Python bindings: [bindings/python/src/lib.rs](bindings/python/src/lib.rs), [bindings/python/genai_keyfinder.pyi](bindings/python/genai_keyfinder.pyi)
- Go bindings: [bindings/go/genai_keyfinder/genai_keyfinder.go](bindings/go/genai_keyfinder/genai_keyfinder.go)

## Core (Rust)

Crate name: `genai-keyfinder-core`.

### Quick Start

```rust
use genai_keyfinder_core::{scan, ScanOptions};

let result = scan(ScanOptions::default())?;
```
See [Rust example usage](core/src/lib.rs:133) in docs.

### Public API

- [scan(ScanOptions) -> Result<ScanResult>](core/src/lib.rs:153)
- [struct ScanOptions](core/src/lib.rs:58)
- [struct ScanResult](core/src/models/scan_result.rs:11)
- [struct ScanSummary](core/src/models/scan_result.rs:177)
- [struct DiscoveredKey](core/src/models/discovered_key.rs:61)
- [enum ValueType](core/src/models/discovered_key.rs:10)
- [enum Confidence](core/src/models/discovered_key.rs:37)
- [struct ConfigInstance](core/src/models/config_instance.rs:13)
- [struct Provider](core/src/models/provider.rs:33)
- [enum AuthMethod](core/src/models/provider.rs:7)
- [struct RateLimit](core/src/models/provider.rs:20)
- Plugin system
  - [trait ProviderPlugin](core/src/plugins/mod.rs:14) - **NEW**: Validates and scores keys
  - [struct PluginRegistry](core/src/plugins/mod.rs:34)
  - [fn register_builtin_plugins](core/src/lib.rs:217)
  - [struct CommonConfigPlugin](core/src/plugins/mod.rs:146)
- Scanner plugins - **NEW**: Discovery-focused plugins
  - [trait ScannerPlugin](core/src/scanners/mod.rs:18)
  - [struct ScannerRegistry](core/src/scanners/mod.rs:153)
  - [fn register_builtin_scanners](core/src/scanners/mod.rs:274)
  - [struct ScanResult](core/src/scanners/mod.rs:108)
- Scanner engine
  - [struct Scanner](core/src/scanner/mod.rs:45)
  - [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - [const DEFAULT_MAX_FILE_SIZE](core/src/scanner/mod.rs:12)

### scan(ScanOptions) -> Result<ScanResult>

- Orchestrates provider plugins and application scanners.
- Uses ScannerPlugin for key discovery and ProviderPlugin for validation.
- Applies redaction unless `include_full_values` is set.

Errors:
- [Error::ConfigError](core/src/lib.rs:127)
- [Error::PluginError](core/src/plugins/mod.rs:56)
- [Error::NotFound](core/src/scanner/mod.rs:83)
- [Error::ValidationError](core/src/scanner/mod.rs:90)
- [Error::IoError](core/src/scanner/mod.rs:177)

### ScanOptions

Fields:
- `home_dir: Option<PathBuf>` — if `None`, resolves to user home
- `include_full_values: bool` — default false; when false, full secrets are removed before serialization
- `max_file_size: usize` — default [DEFAULT_MAX_FILE_SIZE](core/src/scanner/mod.rs:12) (1MB)
- `only_providers: Option<Vec<String>>` — allowlist
- `exclude_providers: Option<Vec<String>>` — blocklist

Builders:
- [with_home_dir(PathBuf) -> Self](core/src/lib.rs:92)
- [with_full_values(bool) -> Self](core/src/lib.rs:98)
- [with_max_file_size(usize) -> Self](core/src/lib.rs:104)
- [with_only_providers(Vec<String>) -> Self](core/src/lib.rs:110)
- [with_exclude_providers(Vec<String>) -> Self](core/src/lib.rs:116)
- [get_home_dir() -> Result<PathBuf>](core/src/lib.rs:121)

### ScanResult

Serialized JSON has:
- `keys: DiscoveredKey[]`
- `config_instances: ConfigInstance[]`
- `scan_started_at: string (RFC3339 UTC)`
- `scan_completed_at: string (RFC3339 UTC)`
- `home_directory: string`
- `providers_scanned: string[]`
- `files_scanned: number`
- `directories_scanned: number`
- `metadata: Option<Map<String, serde_json::Value>>`

Helpers:
- [total_keys()](core/src/models/scan_result.rs:88)
- [total_config_instances()](core/src/models/scan_result.rs:93)
- [keys_by_provider()](core/src/models/scan_result.rs:99)
- [keys_by_type()](core/src/models/scan_result.rs:107)
- [keys_by_confidence()](core/src/models/scan_result.rs:116)
- [filter_by_provider(&str)](core/src/models/scan_result.rs:125)
- [filter_by_confidence(Confidence)](core/src/models/scan_result.rs:130)
- [filter_by_type(&ValueType)](core/src/models/scan_result.rs:138)
- [high_confidence_keys()](core/src/models/scan_result.rs:146)
- [has_keys()](core/src/models/scan_result.rs:151)
- [scan_duration()](core/src/models/scan_result.rs:156)
- [summary() -> ScanSummary](core/src/models/scan_result.rs:162)

### DiscoveredKey

Fields (serialized):
- `provider: String`
- `source: String`
- `value_type: ValueType`
- `confidence: Confidence`
- `hash: String` (SHA-256)
- `discovered_at: DateTime<Utc>`
- `line_number: Option<u32>`
- `column_number: Option<u32>`
- `metadata: Option<serde_json::Value>`

Not serialized:
- `full_value: Option<String>` is private and tagged with `#[serde(skip_serializing)]` ([field](core/src/models/discovered_key.rs:81))

Utilities:
- [new(..., full_value: String)](core/src/models/discovered_key.rs:88)
- [new_redacted(..., full_value_preview: &str)](core/src/models/discovered_key.rs:113)
- [redacted_value() -> String](core/src/models/discovered_key.rs:137) — client-side helper
- [with_full_value(include: bool) -> Self](core/src/models/discovered_key.rs:152)
- [with_position(line, col)](core/src/models/discovered_key.rs:166)
- [with_metadata(value)](core/src/models/discovered_key.rs:172)

Enums:
- [enum ValueType](core/src/models/discovered_key.rs:10) — `ApiKey`, `AccessToken`, `SecretKey`, `BearerToken`, `Custom(String)`
- [enum Confidence](core/src/models/discovered_key.rs:37) — `Low`, `Medium`, `High`, `VeryHigh`

### ConfigInstance

Fields:
- `instance_id: String`
- `app_name: String`
- `config_path: PathBuf` (serialized as path string)
- `discovered_at: DateTime<Utc>`
- `keys: Vec<DiscoveredKey>`
- `metadata: HashMap<String, String>`

See [definition](core/src/models/config_instance.rs:13).

### Provider Model

- [struct Provider](core/src/models/provider.rs:33) with `name`, `provider_type`, `base_url`, etc.
- [enum AuthMethod](core/src/models/provider.rs:7) — `ApiKey`, `OAuth`, `BearerToken`, `Custom(String)`
- [struct RateLimit](core/src/models/provider.rs:20)

### Provider Instance Model - **REFACTORED**

The [`ProviderInstance`](core/src/models/provider_instance.rs:25) model manages individual provider configurations with a simplified single-key approach:

#### ProviderInstance Structure

- [struct ProviderInstance](core/src/models/provider_instance.rs:25) — Provider instance configuration
  - `id: String` — Unique identifier for this instance
  - `display_name: String` — Human-readable display name
  - `provider_type: String` — Provider type (e.g., "openai", "anthropic", "groq")
  - `base_url: String` — Base URL for API requests
  - `api_key: Option<String>` — **Single API key** (simplified from multi-key model)
  - `models: Vec<Model>` — Instance-specific model configurations
  - `metadata: Option<HashMap<String, String>>` — Additional metadata (preserves key metadata during conversions)
  - `active: bool` — Whether this instance is active
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

#### ProviderInstance Methods

Construction:
- [new(id, display_name, provider_type, base_url) -> Self](core/src/models/provider_instance.rs:64) — Creates new instance
- [new_with_cleaned_metadata(...) -> Self](core/src/models/provider_instance.rs:83) — Creates instance with cleaned metadata (removes redundant fields)

API Key Management:
- [set_api_key(&mut self, api_key: String)](core/src/models/provider_instance.rs:137) — Sets the API key
- [get_api_key(&self) -> Option<&String>](core/src/models/provider_instance.rs:144) — Gets API key reference
- [has_api_key(&self) -> bool](core/src/models/provider_instance.rs:150) — Checks if API key is present (including empty strings)
- [has_non_empty_api_key(&self) -> bool](core/src/models/provider_instance.rs:156) — Checks if non-empty API key is present

Model Management:
- [add_model(&mut self, model: Model)](core/src/models/provider_instance.rs:111) — Adds a model
- [add_models(&mut self, models: Vec<Model>)](core/src/models/provider_instance.rs:117) — Adds multiple models
- [model_count(&self) -> usize](core/src/models/provider_instance.rs:163) — Gets model count
- [get_model(&self, model_id: &str) -> Option<&Model>](core/src/models/provider_instance.rs:170) — Gets model by ID
- [active_models(&self) -> Vec<&Model>](core/src/models/provider_instance.rs:203) — Gets active models

Configuration:
- [with_metadata(self, metadata: HashMap<String, String>) -> Self](core/src/models/provider_instance.rs:124) — Sets metadata
- [with_active(self, active: bool) -> Self](core/src/models/provider_instance.rs:131) — Sets active status
- [validate(&self) -> Result<(), String>](core/src/models/provider_instance.rs:178) — Validates configuration
- [key_name(&self) -> &str](core/src/models/provider_instance.rs:215) — Gets the instance ID for CLI usage

#### Metadata-Preserving Conversions

The model includes bidirectional conversions with [`ProviderConfig`](core/src/models/provider_config.rs:11) that preserve key metadata:

**From ProviderConfig to ProviderInstance** ([impl](core/src/models/provider_instance.rs:227)):
- Extracts first valid key value to `api_key`
- Preserves key metadata in instance `metadata` HashMap:
  - `environment` — Environment type
  - `confidence` — Confidence level
  - `validation_status` — Validation state
  - `discovered_at` — RFC3339 timestamp
  - `source` — Source file path
  - `line_number` — Line number (if available)
  - `key_metadata` — Additional JSON metadata (if available)

**From ProviderInstance to ProviderConfig** ([impl](core/src/models/provider_instance.rs:293)):
- Wraps `api_key` in a [`ProviderKey`](core/src/models/provider_key.rs:11) with ID "default"
- Restores all preserved metadata from instance `metadata`
- Uses safe defaults for missing or malformed metadata
- Logs parsing errors without failing conversion

### Provider Configuration (Multi-Key)

The provider configuration supports multiple API keys per provider:

- [struct ProviderConfig](core/src/models/provider_config.rs:11) — Main configuration structure
  - `keys: Vec<ProviderKey>` — Multiple keys instead of single `api_key`
  - `models: Vec<String>` — Available models
  - `metadata: Option<HashMap<String, serde_yaml::Value>>` — Additional metadata
  - `version: String` — Provider version
  - `schema_version: String` — Schema version ("3.0" for multi-key)
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [struct ProviderKey](core/src/models/provider_key.rs:11) — Individual key management
  - `id: String` — Unique identifier (e.g., "default", "staging", "production")
  - `value: Option<String>` — Actual key value (null in config files, populated at scan time)
  - `discovered_at: DateTime<Utc>` — When key was found
  - `source: String` — File path where key was discovered
  - `line_number: Option<u32>` — Line number in source file
  - `confidence: Confidence` — Detection confidence enum with variants: Low, Medium, High, VeryHigh
  - `environment: Environment` — Environment context (dev/staging/prod)
  - `last_validated: Option<DateTime<Utc>>` — Last validation timestamp
  - `validation_status: ValidationStatus` — Current validation state
  - `metadata: Option<serde_json::Value>` — Additional key-specific metadata
  - `created_at: DateTime<Utc>` — Key creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [enum ValidationStatus](core/src/models/provider_key.rs:45) — `Unknown`, `Valid`, `Invalid`, `Expired`
- [enum Environment](core/src/models/provider_key.rs:32) — `Development`, `Staging`, `Production`

ProviderConfig methods:
- [new(version: String) -> Self](core/src/models/provider_config.rs:25)
- [add_key(key: ProviderKey) -> Result<()>](core/src/models/provider_config.rs:30)
- [key_count() -> usize](core/src/models/provider_config.rs:38)
- [valid_key_count() -> usize](core/src/models/provider_config.rs:42)
- [keys_by_environment(env: Environment) -> Vec<&ProviderKey>](core/src/models/provider_config.rs:46)
- [get_key(id: &str) -> Option<&ProviderKey>](core/src/models/provider_config.rs:50)
- [from_old_format(...) -> Self](core/src/models/provider_config.rs:54) — Backward compatibility

**Note:** [`ProviderInstance`](core/src/models/provider_instance.rs:25) uses a simplified single-key model (`api_key: Option<String>`), while [`ProviderConfig`](core/src/models/provider_config.rs:11) maintains the multi-key model (`keys: Vec<ProviderKey>`). Conversions between these models preserve metadata. See the [Migration Guide](docs/migration-guide.md) for details on the refactoring.

### Plugin System

#### Provider Plugins (Validation) - **NEW ARCHITECTURE**

Provider plugins now focus on validating and scoring discovered keys:

- [trait ProviderPlugin](core/src/plugins/mod.rs:14)
  - `name(&self) -> &str` - Plugin name
  - `confidence_score(&self, key: &str) -> f32` - Score key validity (0.0-1.0)
  - `can_handle_file(&self, path: &Path) -> bool` - Check if plugin handles file
  - `provider_type(&self) -> &str` - Provider type name
- [struct PluginRegistry](core/src/plugins/mod.rs:34)
  - `register(Arc<dyn ProviderPlugin>)`
  - `get(name) -> Option<Arc<dyn ProviderPlugin>>`
  - `list() -> Vec<String>`
  - `get_plugins_for_file(&Path) -> Vec<Arc<dyn ProviderPlugin>>`

#### Scanner Plugins (Discovery) - **NEW ARCHITECTURE**

Scanner plugins handle discovery of keys and configurations:

- [trait ScannerPlugin](core/src/scanners/mod.rs:18) - **NEW**
  - `name(&self) -> &str` - Scanner name
  - `app_name(&self) -> &str` - Application name
  - `scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>` - Paths to scan
  - `parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult>` - Parse config
  - `can_handle_file(&self, path: &Path) -> bool` - Check if scanner handles file
  - `supports_provider_scanning(&self) -> bool` - Whether scanner finds provider keys
  - `supported_providers(&self) -> Vec<String>` - Providers this scanner can find
  - `scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>>` - Find provider configs
  - `scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>>` - Find app instances
- [struct ScannerRegistry](core/src/scanners/mod.rs:153) - **NEW**
  - `register(Arc<dyn ScannerPlugin>)`
  - `get(name) -> Option<Arc<dyn ScannerPlugin>>`
  - `list() -> Vec<String>`
  - `get_scanners_for_file(&Path) -> Vec<Arc<dyn ScannerPlugin>>`

### Scanner Engine

- [struct Scanner](core/src/scanner/mod.rs:45)
  - `scan(&self, home_dir: &Path) -> Result<ScanResult>` - Main scanning method
- [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - `max_file_size`, `follow_symlinks`, `include_extensions`, `exclude_extensions`, `exclude_files`, `scan_hidden`

## FFI / C-API

Header: [ffi/include/genai_keyfinder.h](ffi/include/genai_keyfinder.h)

Functions:
- `char* keyfinder_scan(const char* home_path, const char* options_json);` ([decl](ffi/include/genai_keyfinder.h:34), [impl](ffi/src/lib.rs:101))
- `void keyfinder_free(char* ptr);` ([decl](ffi/include/genai_keyfinder.h:43), [impl](ffi/src/lib.rs:179))
- `const char* keyfinder_version(void);` ([decl](ffi/include/genai_keyfinder.h:50), [impl](ffi/src/lib.rs:192))
- `const char* keyfinder_last_error(void);` ([decl](ffi/include/genai_keyfinder.h:58), [impl](ffi/src/lib.rs:206))

Options JSON example (UTF-8 C string):
```json
{
  "include_full_values": false,
  "max_file_size": 1048576,
  "only_providers": ["openai", "anthropic"],
  "exclude_providers": []
}
```

Return:
- On success: UTF-8 JSON string of Core [ScanResult](core/src/models/scan_result.rs:11) (caller must free with `keyfinder_free`)
- On failure: `NULL`, and `keyfinder_last_error()` returns the message (thread-local)

Memory:
- Caller must free any pointer returned by `keyfinder_scan` using `keyfinder_free`
- `keyfinder_version` and `keyfinder_last_error` return pointers valid until the next FFI call

## Python API

Module name: `genai_keyfinder`.

Definitions:
- [scan(...)](bindings/python/genai_keyfinder.pyi:3)
  - `home_dir: Optional[str] = None`
  - `include_full_values: bool = False`
  - `max_file_size: int = 1048576`
  - `only_providers: Optional[List[str]] = None`
  - `exclude_providers: Optional[List[str]] = None`
  - Returns: `Dict[str, Any]` matching Core JSON of [ScanResult](core/src/models/scan_result.rs:11)
- [version() -> str](bindings/python/genai_keyfinder.pyi:36)
- [list_providers() -> List[str]](bindings/python/genai_keyfinder.pyi:40) - **UPDATED**: Lists provider plugins
- [list_scanners() -> List[str]](bindings/python/genai_keyfinder.pyi:44) - **NEW**: Lists scanner plugins

Implementation detail: Python uses Core [scan(ScanOptions)](bindings/python/src/lib.rs:23) and `serde_json` round-trip to construct a Python dict.

## Go API

Package: `github.com/robottwo/aicred/bindings/go/genai_keyfinder`.

Types:
- [type ScanOptions struct](bindings/go/genai_keyfinder/genai_keyfinder.go:18)
  - `HomeDir string`
  - `IncludeFullValues bool`
  - `MaxFileSize int`
  - `OnlyProviders []string`
  - `ExcludeProviders []string`
- [type DiscoveredKey struct](bindings/go/genai_keyfinder/genai_keyfinder.go:27)
  - `Provider, Source, ValueType, Hash string`
  - `Confidence string` (enum values: "Low", "Medium", "High", "VeryHigh")
  - `Redacted string` (may be empty; not populated by core unless post-processed)
  - `Locked bool` (not set by core; reserved)
- [type ConfigInstance struct](bindings/go/genai_keyfinder/genai_keyfinder.go:39)
- [type ScanResult struct](bindings/go/genai_keyfinder/genai_keyfinder.go:48)

Functions:
- [func Scan(options ScanOptions) (*ScanResult, error)](bindings/go/genai_keyfinder/genai_keyfinder.go:57)
- [func Version() string](bindings/go/genai_keyfinder/genai_keyfinder.go:101)
- [func ListProviders() []string](bindings/go/genai_keyfinder/genai_keyfinder.go:107) - **UPDATED**: Lists provider plugins
- [func ListScanners() []string](bindings/go/genai_keyfinder/genai_keyfinder.go:119) - **NEW**: Lists scanner plugins

Note: Go uses the FFI; JSON shape mirrors Core. Optional fields like `Redacted` may be blank unless your application computes redaction strings.

## CLI

Binary: `keyfinder`.

Commands:
- [scan](cli/src/main.rs:22) — see handler [handle_scan(...)](cli/src/commands/scan.rs:8)
- [list](cli/src/main.rs:57) — see [handle_list(verbose)](cli/src/commands/list.rs:4) - **UPDATED**: Lists both providers and scanners
- [version](cli/src/main.rs:64)

Output formats:
- JSON: [output_json(&ScanResult)](cli/src/output/json.rs:4)
- NDJSON (keys and instances): [output_ndjson(&ScanResult)](cli/src/output/ndjson.rs:4)
- Summary: [output_summary(&ScanResult)](cli/src/output/summary.rs:4)
- Table: [output_table(&ScanResult)](cli/src/output/table.rs:1)

Flags for `scan`:
- `--home` (directory)
- `--format` (`table` | `json` | `ndjson` | `summary`)
- `--include-values` (boolean)
- `--only`, `--exclude` (comma-separated lists) - **UPDATED**: Works with both provider and scanner names
- `--max-bytes-per-file` (usize)
- `--dry-run`
- `--audit-log PATH`

## Data Structures (JSON Schemas)

Informal JSON shapes (actual serialization driven by `serde`):

### DiscoveredKey:
```json
{
  "provider": "openai",
  "source": "/home/user/.env",
  "value_type": "ApiKey",
  "confidence": "High",
  "hash": "hex-64",
  "discovered_at": "2025-01-20T10:30:00Z",
  "line_number": 10,
  "column_number": 5,
  "metadata": {}
}
```

### ConfigInstance:
```json
{
  "instance_id": "instance-1",
  "app_name": "roo-code",
  "config_path": "/home/user/.config/roo-code/config.json",
  "discovered_at": "2025-01-20T10:30:00Z",
  "keys": [],
  "metadata": { "version": "1.2.3" }
}
```

### ScanResult:
```json
{
  "keys": [],
  "config_instances": [],
  "scan_started_at": "2025-01-20T10:30:00Z",
  "scan_completed_at": "2025-01-20T10:30:01Z",
  "home_directory": "/home/user",
  "providers_scanned": ["openai", "anthropic"],
  "files_scanned": 100,
  "directories_scanned": 20,
  "metadata": null
}
```

## Error Handling

The core uses a unified error type `genai_keyfinder_core::error::Error` (variants used across modules):
- `ConfigError(String)` — e.g., cannot determine home dir ([usage](core/src/lib.rs:127))
- `PluginError(String)` — plugin registration/operation ([usage](core/src/plugins/mod.rs:56))
- `NotFound(String)` — invalid paths ([usage](core/src/scanner/mod.rs:83))
- `ValidationError(String)` — wrong types/expectations ([usage](core/src/scanner/mod.rs:90))
- `IoError(std::io::Error)` — filesystem operations ([usage](core/src/scanner/mod.rs:177))

FFI:
- Functions return `NULL` on error; the message is provided by [keyfinder_last_error()](ffi/include/genai_keyfinder.h:58).

CLI:
- Exit codes:
  - `0`: keys or config instances found
  - `1`: none found
  - `2`: error occurred (set via `anyhow::bail!` paths)

## Configuration Options

ScanOptions (Core):
- See [struct ScanOptions](core/src/lib.rs:58)

ScannerConfig (Core scanner engine):
- See [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - `max_file_size`, `follow_symlinks`, `include_extensions`, `exclude_extensions`, `exclude_files`, `scan_hidden`

## Redaction Model

- Full values are stored only transiently inside `DiscoveredKey` and are skipped in JSON output (security-by-default).
- Clients should use:
  - `hash` for deduplication
  - A generic preview like `"****"` for display
  - If and only if `include_full_values` was used, applications may compute previews locally (e.g., last 4 chars).

## Architecture Changes - **IMPORTANT**

### New Plugin Architecture

The architecture has been redesigned to separate concerns:

**ScannerPlugin** (Discovery):
- Handles discovery of API keys and configuration files
- Can discover keys for multiple providers
- Focuses on file parsing and key extraction
- Methods: `name()`, `app_name()`, `scan_paths()`, `parse_config()`, `can_handle_file()`
- Optional: `supports_provider_scanning()`, `supported_providers()`, `scan_provider_configs()`, `scan_instances()`

**ProviderPlugin** (Validation):
- Validates and scores discovered keys
- Assigns confidence scores to keys
- Focuses on key validation and pattern matching
- Methods: `name()`, `confidence_score()`, `can_handle_file()`, `provider_type()`

### Migration Notes

**Old Architecture**: ProviderPlugin handled both discovery and validation
**New Architecture**: Separate ScannerPlugin (discovery) and ProviderPlugin (validation)

This change provides:
- Clear separation of concerns
- More flexible plugin system
- Better support for application-specific scanning
- Improved key discovery across multiple providers

## Versioning

- Library versions: [ffi/src/lib.rs](ffi/src/lib.rs:27) exposes `keyfinder_version()`
- CLI `version` subcommand prints package version and core version ([handler](cli/src/main.rs:96))
