# Migrating from AICred 0.1.x to 0.2.0

## Overview

Version 0.2.0 introduces three major improvements:
1. **Consolidated models** - Cleaner, more intuitive data types
2. **Renamed discovery system** - `scanners` → `discovery` for clarity
3. **Simplified plugin API** - Direct HashMap instead of wrapper

All old APIs remain available but deprecated. They will be removed in v0.3.0.

## What's New in 0.2.0

### 1. Model Consolidation (Phase 1)
- 18 model files → 6 files (67% reduction)
- Unified "Label" concept (no more Tag/Label confusion)
- `DiscoveredKey` → `DiscoveredCredential` (clearer naming)
- 77% code reduction in models

### 2. Discovery System Rename (Phase 2)
- `scanners` module → `discovery` module
- Scanner-specific helpers added
- Zero breaking changes (backward compat re-exports)

### 3. Simplified Plugin API (Phase 3)
- `PluginRegistry` wrapper → direct `HashMap` (`ProviderRegistry`)
- `register_builtin_plugins()` → `register_builtin_providers()`
- 69% code reduction in registry logic
- Cleaner, more idiomatic Rust

## Migration Strategy

### Option 1: Gradual Migration (Recommended)
Use both old and new APIs side-by-side, migrating incrementally.

### Option 2: Quick Migration
Update all imports at once using this guide.

---

## Part 1: Model Changes

### Type Mapping

#### Credentials & Discovery

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `DiscoveredKey` | `DiscoveredCredential` | `credentials_new` |
| `ValueType` | `ValueTypeNew` | `credentials_new` |
| `ProviderKey` | Merged into `DiscoveredCredential` | `credentials_new` |

#### Labels (formerly Tags)

| Old Type (v0.1.x) | New Type (v0.2.0) | Notes |
|-------------------|-------------------|-------|
| `Tag` | `LabelNew` | Unified concept |
| `Label` | `LabelNew` | Now single type |
| `TagAssignment` | `LabelAssignmentNew` | |
| `LabelAssignment` | `LabelAssignmentNew` | |
| `UnifiedLabel` | `LabelWithAssignments` | |

#### Providers & Models

| Old Type (v0.1.x) | New Type (v0.2.0) | Notes |
|-------------------|-------------------|-------|
| `ProviderInstance` | `ProviderInstanceNew` | |
| `ProviderInstances` | `ProviderCollection` | Clearer name |
| `Model` | `ModelNew` | |
| `ModelMetadata` | Part of `ModelNew` | Merged |

### Code Examples: Models

**Before (v0.1.x):**
```rust
use aicred_core::models::{
    DiscoveredKey,
    Tag,
    ProviderInstance,
    Confidence,
    ValueType,
};

let key = DiscoveredKey {
    provider: "openai".to_string(),
    source: "/path/to/config".to_string(),
    value_type: ValueType::ApiKey,
    confidence: Confidence::High,
    // ...
};
```

**After (v0.2.0):**
```rust
use aicred_core::models::{
    DiscoveredCredential,
    LabelNew,
    ProviderInstanceNew,
    ConfidenceNew,
    ValueTypeNew,
};

let credential = DiscoveredCredential {
    provider: "openai".to_string(),
    source_file: "/path/to/config".to_string(),
    value_type: ValueTypeNew::ApiKey,
    confidence: ConfidenceNew::High,
    // ...
};
```

---

## Part 2: Discovery System (scanners → discovery)

### Module Rename

**Before (v0.1.x):**
```rust
use aicred_core::scanners::{
    ClaudeDesktopScanner,
    RooCodeScanner,
    ScannerPlugin,
};
```

**After (v0.2.0):**
```rust
use aicred_core::discovery::{
    ClaudeDesktopScanner,
    RooCodeScanner,
    ScannerPlugin,
};
```

### Backward Compatibility

For compatibility, `scanners` is re-exported:
```rust
// This still works (deprecated)
use aicred_core::scanners::ClaudeDesktopScanner;

// But prefer the new path
use aicred_core::discovery::ClaudeDesktopScanner;
```

### New Helper Functions

v0.2.0 adds convenience helpers:

```rust
use aicred_core::discovery::{
    read_json_file,
    read_yaml_file,
    find_existing_configs,
};

// Read and parse JSON in one step
let config = read_json_file(&path)?;

// Find configs that exist
let configs = find_existing_configs(home_dir, &[
    ".config/app/config.json",
    ".app/settings.yml",
]);
```

---

## Part 3: Simplified Plugin API

### Registry Changes

**Before (v0.1.x) - Wrapper-based:**
```rust
use aicred_core::plugins::{PluginRegistry, register_builtin_plugins};

// Create empty registry
let registry = PluginRegistry::new();

// Register plugins
register_builtin_plugins(&registry)?;

// Use methods
if let Some(plugin) = registry.get("openai") {
    println!("{}", plugin.name());
}

for name in registry.list() {
    println!("{}", name);
}
```

**After (v0.2.0) - Direct HashMap:**
```rust
use aicred_core::plugins::{
    ProviderRegistry,
    register_builtin_providers,
    get_provider,
    list_providers,
};

// Get ready-to-use HashMap
let registry: ProviderRegistry = register_builtin_providers();

// Use helper functions
if let Some(plugin) = get_provider(&registry, "openai") {
    println!("{}", plugin.name());
}

for name in list_providers(&registry) {
    println!("{}", name);
}
```

### Custom Plugin Registration

**Before (v0.1.x):**
```rust
use std::sync::Arc;
use aicred_core::plugins::{PluginRegistry, ProviderPlugin};

let registry = PluginRegistry::new();
registry.register(Arc::new(MyPlugin))?;
```

**After (v0.2.0):**
```rust
use std::sync::Arc;
use aicred_core::plugins::{ProviderRegistry, ProviderPlugin};

let mut registry = ProviderRegistry::new();
let plugin = Arc::new(MyPlugin) as Arc<dyn ProviderPlugin>;
registry.insert(plugin.name().to_string(), plugin);
```

### Benefits of New API

1. **Simpler:** No wrapper class, direct HashMap usage
2. **Familiar:** Standard Rust patterns
3. **Efficient:** No Arc<RwLock<>> overhead unless you need it
4. **Composable:** Easy to merge, filter, or extend registries

### Backward Compatibility

The old `PluginRegistry` wrapper is still available (deprecated):

```rust
// Still works, but deprecated
use aicred_core::plugins::{PluginRegistry, register_builtin_plugins};
let registry = PluginRegistry::new();
register_builtin_plugins(&registry)?;
```

---

## Deprecation Timeline

| Version | Status | Notes |
|---------|--------|-------|
| **v0.2.0** (current) | Old APIs deprecated | New APIs available, both coexist |
| **v0.2.x** | Migration period | Warnings guide users |
| **v0.3.0** (planned) | Old APIs removed | Only new APIs available |

---

## Feature Flag for Compatibility

Need more time? Use the `compat_v0_1` feature flag:

```toml
[dependencies]
aicred-core = { version = "0.2", features = ["compat_v0_1"] }
```

Provides type aliases:
```rust
pub type Tag = LabelNew;
pub type TagAssignment = LabelAssignmentNew;
pub type DiscoveredKey = DiscoveredCredential;
// etc.
```

---

## Testing Your Migration

### Check for Deprecation Warnings

```bash
cargo build 2>&1 | grep -i deprecated
```

### Run Tests

```bash
cargo test
```

### Use Clippy

```bash
cargo clippy -- -D deprecated
```

---

## Common Pitfalls

### 1. Mixing Old and New Types

**Wrong:**
```rust
use aicred_core::models::{DiscoveredKey, CredentialValue};  // Mixed!
```

**Right:**
```rust
use aicred_core::models::{DiscoveredCredential, CredentialValue};
```

### 2. Forgetting Module Renames

**Old:** `use aicred_core::scanners::*;`  
**New:** `use aicred_core::discovery::*;`

### 3. Using Wrapper When HashMap Would Work

**Old way (still works but deprecated):**
```rust
let registry = PluginRegistry::new();
register_builtin_plugins(&registry)?;
```

**New way (preferred):**
```rust
let registry = register_builtin_providers();
```

---

## Quick Reference

### Model Names

- `DiscoveredKey` → `DiscoveredCredential`
- `Tag` / `Label` → `LabelNew` (unified)
- `ProviderInstances` → `ProviderCollection`

### Module Names

- `scanners` → `discovery`
- Helper functions added: `read_json_file`, `read_yaml_file`, `find_existing_configs`

### Plugin API

- `PluginRegistry` → `ProviderRegistry` (HashMap)
- `register_builtin_plugins()` → `register_builtin_providers()`
- Methods → Helper functions: `get_provider()`, `list_providers()`

---

## Getting Help

- **Issues:** https://github.com/robottwo/aicred/issues
- **Discussions:** https://github.com/robottwo/aicred/discussions
- **Examples:** See `examples/` directory for updated code

---

## Rollback

If you encounter issues:

```toml
[dependencies]
aicred-core = "0.1"  # Stay on 0.1.x
```

Or use compatibility features:

```toml
[dependencies]
aicred-core = { version = "0.2", features = ["compat_v0_1"] }
```

---

## Summary

v0.2.0 brings significant improvements:
- ✅ **67% fewer model files** (18 → 6)
- ✅ **69% less registry code** (188 → 58 LOC)
- ✅ **Clearer naming** (`discovery` not `scanners`)
- ✅ **Zero breaking changes** (old APIs still work)
- ✅ **Better ergonomics** (direct HashMap, helper functions)

All changes are backward compatible. Migrate at your own pace!
