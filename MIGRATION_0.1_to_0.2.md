# Migrating from AICred 0.1.x to 0.2.0

## Overview

Version 0.2.0 introduces consolidated, cleaner data models. The old models are still available but deprecated and will be removed in v0.3.0.

## Migration Strategy

You have two options:

### Option 1: Gradual Migration (Recommended)
Use both old and new APIs side-by-side, migrating modules incrementally.

### Option 2: Quick Migration
Update all imports at once using the mapping table below.

## Type Mapping

### Credentials & Discovery

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `DiscoveredKey` | `DiscoveredCredential` | `credentials_new` |
| `ValueType` | `ValueTypeNew` / `CredentialValue` | `credentials_new` |
| `Confidence` | `ConfidenceNew` | `credentials_new` |
| `Environment` | `EnvironmentNew` | `credentials_new` |
| `ValidationStatus` | `ValidationStatusNew` | `credentials_new` |
| `ProviderKey` | Merged into `DiscoveredCredential` | `credentials_new` |

### Labels (formerly Tags)

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `Tag` | `LabelNew` | `labels_new` |
| `Label` | `LabelNew` | `labels_new` |
| `TagAssignment` | `LabelAssignmentNew` | `labels_new` |
| `LabelAssignment` | `LabelAssignmentNew` | `labels_new` |
| `TagAssignmentTarget` | `LabelTarget` | `labels_new` |
| `LabelAssignmentTarget` | `LabelTarget` | `labels_new` |
| `UnifiedLabel` | `LabelWithAssignments` | `labels_new` |

### Providers

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `Provider` | `ProviderNew` | `providers_new` |
| `ProviderInstance` | `ProviderInstanceNew` | `providers_new` |
| `ProviderInstances` | `ProviderCollection` | `providers_new` |
| `AuthMethod` | `AuthMethodNew` | `providers_new` |
| `RateLimit` | `RateLimitNew` | `providers_new` |
| `Capabilities` | `CapabilitiesNew` | `providers_new` |
| `ProviderConfig` | ❌ Removed (use `ProviderInstanceNew`) | N/A |

### Models

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `Model` | `ModelNew` | `models_new` |
| `ModelMetadata` | `ModelMetadataNew` | `models_new` |
| `ModelPricing` | `ModelPricingNew` | `models_new` |
| `TokenCost` | `TokenCostNew` | `models_new` |
| `ModelArchitecture` | Part of `ModelMetadataNew` | `models_new` |

### Scan Results

| Old Type (v0.1.x) | New Type (v0.2.0) | Module |
|-------------------|-------------------|---------|
| `ScanResult` | `ScanResult` (unchanged) | `scan_result` |
| `ScanSummary` | `ScanSummary` (unchanged) | `scan_result` |

## Code Examples

### Before (v0.1.x)

```rust
use aicred_core::models::{
    DiscoveredKey,
    Tag,
    TagAssignment,
    ProviderInstance,
    Confidence,
    ValueType,
};

let key = DiscoveredKey {
    provider: "openai".to_string(),
    source: "/path/to/config".to_string(),
    value_type: ValueType::ApiKey,
    confidence: Confidence::High,
    // ... other fields
};

let tag = Tag {
    name: "fast".to_string(),
    description: Some("Fast models".to_string()),
    created_at: chrono::Utc::now(),
    metadata: Default::default(),
};
```

### After (v0.2.0)

```rust
use aicred_core::models::{
    DiscoveredCredential,
    CredentialValue,
    LabelNew,
    LabelAssignmentNew,
    ProviderInstanceNew,
    ConfidenceNew,
    ValueTypeNew,
};

let credential = DiscoveredCredential {
    provider: "openai".to_string(),
    value: CredentialValue::redact("sk-proj-..."),
    confidence: ConfidenceNew::High,
    source_file: "/path/to/config".to_string(),
    source_line: None,
    environment: EnvironmentNew::UserConfig,
    discovered_at: chrono::Utc::now(),
    value_type: ValueTypeNew::ApiKey,
};

let label = LabelNew {
    name: "fast".to_string(),
    description: Some("Fast models".to_string()),
    created_at: chrono::Utc::now(),
    metadata: Default::default(),
};
```

## Breaking Changes

### 1. Tags → Labels

**Rationale:** "Label" is more intuitive and industry-standard (Kubernetes, Docker).

**Migration:**
```rust
// Old
use aicred_core::models::{Tag, TagAssignment};

// New
use aicred_core::models::{LabelNew as Label, LabelAssignmentNew as LabelAssignment};
```

### 2. DiscoveredKey → DiscoveredCredential

**Rationale:** "Credential" is clearer than "Key" (which could mean encryption key, map key, etc.).

**Migration:**
```rust
// Old
use aicred_core::models::DiscoveredKey;

// New
use aicred_core::models::DiscoveredCredential;
```

### 3. ValueType Split

**Changed:** `ValueType` in v0.1.x was an enum for credential types.  
**New:** 
- `ValueTypeNew` - Type of credential (ApiKey, Token, etc.)
- `CredentialValue` - Full or redacted value

**Migration:**
```rust
// Old (v0.1.x)
let value_type = ValueType::ApiKey;

// New (v0.2.0)
let value_type = ValueTypeNew::ApiKey;
let value = CredentialValue::redact("sk-...");
```

### 4. ProviderInstances → ProviderCollection

**Rationale:** "Collection" is clearer than plural form.

**Migration:**
```rust
// Old
use aicred_core::models::ProviderInstances;
let instances = ProviderInstances::new();

// New
use aicred_core::models::ProviderCollection;
let collection = ProviderCollection::new();
```

### 5. Removed ProviderConfig

**Deprecated since 0.1.0**, now removed in 0.2.0.

**Migration:**
```rust
// Old (deprecated)
use aicred_core::models::ProviderConfig;

// New
use aicred_core::models::ProviderInstanceNew;
```

## Module Organization

### Old Structure (v0.1.x)
```
models/
├── discovered_key.rs
├── provider_key.rs
├── tag.rs
├── label.rs
├── tag_assignment.rs
├── label_assignment.rs
├── unified_label.rs
├── provider.rs
├── provider_instance.rs
├── provider_instances.rs
├── model.rs
├── model_metadata.rs
└── ... (18 files total)
```

### New Structure (v0.2.0)
```
models/
├── credentials_new.rs  (consolidates discovered_key + provider_key)
├── labels_new.rs       (consolidates tag + label + assignments + unified)
├── providers_new.rs    (consolidates provider + instance + instances)
├── models_new.rs       (consolidates model + model_metadata)
├── scan_new.rs         (renamed scan_result)
└── mod.rs
```

## Deprecation Timeline

- **v0.2.0** (current): Old types deprecated, new types available
- **v0.2.x**: Both APIs coexist
- **v0.3.0** (planned): Old types removed

## Feature Flag for Extended Compatibility

If you need more time to migrate, use the `compat_v0_1` feature flag:

```toml
[dependencies]
aicred-core = { version = "0.2", features = ["compat_v0_1"] }
```

This provides type aliases:
```rust
pub type Tag = LabelNew;
pub type TagAssignment = LabelAssignmentNew;
// etc.
```

## Automated Migration Script

For large codebases, we provide a migration script:

```bash
# Download the migration script
curl -O https://raw.githubusercontent.com/robottwo/aicred/main/scripts/migrate_to_v0.2.sh

# Run on your codebase
./migrate_to_v0.2.sh /path/to/your/project

# Review changes
git diff

# If satisfied
git commit -am "Migrate to aicred 0.2.0"
```

## Testing Your Migration

After updating imports:

```bash
# Run tests
cargo test

# Check for deprecation warnings
cargo build 2>&1 | grep "deprecated"

# Use clippy to catch issues
cargo clippy -- -D deprecated
```

## Common Pitfalls

### 1. Mixing Old and New Types

**Problem:**
```rust
use aicred_core::models::{DiscoveredKey, CredentialValue};  // Mixed!
```

**Solution:** Use types from the same version:
```rust
use aicred_core::models::{DiscoveredCredential, CredentialValue};
```

### 2. Forgetting "New" Suffix

Many new types have a "New" suffix to avoid conflicts during migration:

```rust
// Correct
use aicred_core::models::{LabelNew, ModelNew, ProviderNew};

// Will use deprecated old version
use aicred_core::models::{Label, Model, Provider};
```

### 3. Import Paths

New types are in different modules:

```rust
// Old
use aicred_core::models::discovered_key::DiscoveredKey;

// New
use aicred_core::models::credentials_new::DiscoveredCredential;
// Or just:
use aicred_core::models::DiscoveredCredential;  // Re-exported in mod.rs
```

## Getting Help

- **Issues:** https://github.com/robottwo/aicred/issues
- **Discussions:** https://github.com/robottwo/aicred/discussions
- **Discord:** [Join our server](#)

## Rollback

If you encounter issues, you can stay on 0.1.x:

```toml
[dependencies]
aicred-core = "0.1"
```

Or use the `compat_v0_1` feature with 0.2.0 as mentioned above.
