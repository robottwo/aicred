# API Baseline - Pre-Refactor

**Date:** 2026-02-04  
**Branch:** code-cleanup (pre-refactor state)

## Public API Exports

From `core/src/lib.rs`:

```rust
pub use env_resolver::{EnvResolutionResult, EnvResolver, EnvResolverBuilder, EnvVarMapping};
pub use error::{Error, Result};
pub use models::{
    AuthMethod, Capabilities, Confidence, ConfigInstance, DiscoveredKey, Model, Provider,
    RateLimit, ScanResult, ScanSummary, UnifiedLabel, ValueType,
};
pub use parser::{ConfigParser, FileFormat};
pub use plugins::{register_builtin_plugins, CommonConfigPlugin, PluginRegistry, ProviderPlugin};
pub use scanner::{Scanner, ScannerConfig, DEFAULT_MAX_FILE_SIZE};
pub use scanners::{register_builtin_scanners, ScannerPlugin, ScannerRegistry};
pub use utils::provider_model_tuple::ProviderModelTuple;
```

## Test Summary

- **Total test suites:** 31
- **Status:** All passing
- **Notable:** 1 ignored test (probe_models_async)

## Dependencies

See `docs/pre-refactor-deps.txt` for full dependency tree.

Key dependencies:
- serde 1.0
- tokio 1.0 (features = ["full"])
- async-trait 0.1
- reqwest 0.11
- tracing 0.1

## Breaking Changes Planned

### Phase 1: Model Consolidation
- `DiscoveredKey` → `DiscoveredCredential`
- `ValueType` → `CredentialValue`
- Tag types → Label types
- `ConfigInstance` → evaluation needed

### Phase 2: Scanner Changes
- `Scanner` → `DiscoveryEngine`
- `ScannerRegistry` → `register_builtin_scanners()` function
- `scanners` module → `discovery` module

### Phase 3: Plugin Changes
- `PluginRegistry` → `ProviderRegistry` (type alias to HashMap)
- `register_builtin_plugins()` remains but simplified

## Backward Compatibility

Feature flag `compat_v0_1` will provide type aliases for:
- Tag → Label
- TagAssignment → LabelAssignment
- DiscoveredKey → DiscoveredCredential
- ValueType → CredentialValue
