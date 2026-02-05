# Internal API Migration Plan

**Goal:** Remove dual API support, use only new types internally while maintaining backward compatibility for external users.

## Current State

**New types (_new.rs files):**
- credentials_new.rs - `DiscoveredCredential`, `CredentialValue`, etc.
- labels_new.rs - `LabelNew`, `LabelAssignmentNew`, etc.
- providers_new.rs - `ProviderNew`, `ProviderInstanceNew`, etc.
- models_new.rs - `ModelNew`, `ModelMetadataNew`, etc.
- scan_new.rs - (scan_result renamed)

**Old files to remove:**
- discovered_key.rs → merged into credentials_new.rs
- provider_key.rs → merged into credentials_new.rs
- label.rs → merged into labels_new.rs
- label_assignment.rs → merged into labels_new.rs
- tag.rs → merged into labels_new.rs
- tag_assignment.rs → merged into labels_new.rs
- unified_label.rs → merged into labels_new.rs
- provider.rs → replaced by providers_new.rs
- provider_instance.rs → replaced by providers_new.rs
- provider_instances.rs → replaced by providers_new.rs
- model.rs → replaced by models_new.rs
- model_metadata.rs → merged into models_new.rs
- scan_result.rs → replaced by scan_new.rs
- provider_config.rs → deprecated, remove

## Migration Strategy

### Step 1: Rename _new.rs files to canonical names
```bash
mv credentials_new.rs credentials.rs
mv labels_new.rs labels.rs
mv providers_new.rs providers.rs
mv models_new.rs models.rs
mv scan_new.rs scan.rs
```

### Step 2: Update new files - remove "New" suffix from type names
In each renamed file, change:
- `DiscoveredCredential` stays (already good)
- `LabelNew` → `Label`
- `LabelAssignmentNew` → `LabelAssignment`
- `ProviderNew` → `Provider`
- `ProviderInstanceNew` → `ProviderInstance`
- `ModelNew` → `Model`
- etc.

### Step 3: Update mod.rs exports
```rust
// Primary exports (new types)
pub use credentials::{DiscoveredCredential, CredentialValue, ...};
pub use labels::{Label, LabelAssignment, ...};
pub use providers::{Provider, ProviderInstance, ProviderCollection, ...};
pub use models::{Model, ModelMetadata, ...};

// Backward compatibility aliases
#[deprecated(since = "0.2.0", note = "Use DiscoveredCredential")]
pub type DiscoveredKey = DiscoveredCredential;

#[deprecated(since = "0.2.0", note = "Use Label")]
pub type Tag = Label;

// etc.
```

### Step 4: Update all internal imports
Find and replace throughout core/src:
- `models::discovered_key::` → `models::credentials::`
- `DiscoveredKey` → `DiscoveredCredential`
- `LabelNew` → `Label`
- `ProviderInstanceNew` → `ProviderInstance`
- `ModelNew` → `Model`
- etc.

### Step 5: Delete old files
```bash
rm discovered_key.rs provider_key.rs
rm label.rs label_assignment.rs tag.rs tag_assignment.rs unified_label.rs
rm provider.rs provider_instance.rs provider_instances.rs provider_config.rs
rm model.rs model_metadata.rs
rm scan_result.rs
```

### Step 6: Update lib.rs exports
Keep backward compatibility at the public API level:
```rust
// Primary exports (new names)
pub use models::{
    DiscoveredCredential,
    Label,
    LabelAssignment,
    Provider,
    ProviderInstance,
    Model,
    // ...
};

// Backward compatibility (deprecated)
#[allow(deprecated)]
pub use models::{
    DiscoveredKey,  // = DiscoveredCredential
    Tag,            // = Label
    // ...
};
```

## Testing Strategy

After each step:
1. `cargo build --package aicred-core`
2. `cargo test --package aicred-core`
3. Check for compilation errors
4. Fix imports as needed

## Rollback

Each step is a separate commit. Can revert individually if needed.

## Estimated Time

- Step 1: 5 minutes (rename files)
- Step 2: 30-45 minutes (update type names in files)
- Step 3: 15 minutes (update mod.rs)
- Step 4: 1-2 hours (update all internal imports - lots of files)
- Step 5: 5 minutes (delete old files)
- Step 6: 15 minutes (update lib.rs)

**Total:** ~2-3 hours

## Benefits

- Single source of truth for each type
- Cleaner codebase (no _new suffix)
- Internal code uses canonical names
- External users still get backward compatibility
- Easier to eventually remove deprecated aliases in v0.3.0
