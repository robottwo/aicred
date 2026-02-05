# Hybrid Approach: Delete Files, Keep API Compatibility

## Strategy

1. **Delete old model files** (cleanup)
2. **Export type aliases** from new types (compatibility)
3. **Zero breaking changes** for users
4. **Clean internal codebase**

## Implementation

### Step 1: Create Type Aliases in mod.rs

```rust
// In core/src/models/mod.rs

// Re-export new types as primary
pub use credentials_new::*;
pub use labels_new::*;
pub use providers_new::*;
pub use models_new::*;

// Backward compatibility aliases (no new files needed!)
pub type DiscoveredKey = DiscoveredCredential;
pub type Tag = LabelNew;
pub type TagAssignment = LabelAssignment;
// etc.
```

### Step 2: Delete Old Files

```bash
rm core/src/models/discovered_key.rs
rm core/src/models/provider_key.rs
rm core/src/models/tag.rs
rm core/src/models/label.rs
rm core/src/models/tag_assignment.rs
rm core/src/models/label_assignment.rs
rm core/src/models/unified_label.rs
rm core/src/models/provider.rs
rm core/src/models/provider_instance.rs
rm core/src/models/provider_instances.rs
rm core/src/models/model.rs
rm core/src/models/model_metadata.rs
```

### Step 3: Rename New Files (Remove "_new" suffix)

```bash
mv core/src/models/credentials_new.rs core/src/models/credentials.rs
mv core/src/models/labels_new.rs core/src/models/labels.rs
mv core/src/models/providers_new.rs core/src/models/providers.rs
mv core/src/models/models_new.rs core/src/models/models.rs
mv core/src/models/scan_new.rs core/src/models/scan.rs
```

## Result

- ✅ Clean codebase (5 files instead of 23)
- ✅ Old API still works (`DiscoveredKey`, etc.)
- ✅ New API available (`DiscoveredCredential`, etc.)
- ✅ Zero breaking changes
- ✅ Internal cleanup complete

## Benefits vs Dual-File Approach

**Current (Dual-File):**
- 18 old files + 5 new files = 23 total
- Old code in old files, new code in new files
- Clean separation but cluttered

**Hybrid (Proposed):**
- 5 files total
- Old API works via type aliases
- New API is primary
- Much cleaner

## Breaking Changes

**None!** Users can still use:
```rust
use aicred_core::DiscoveredKey;  // Works via type alias
use aicred_core::DiscoveredCredential;  // Direct type
```

Both work, just one is an alias to the other.

## Answer to Your Question

**Yes, let's delete the old files** and use type aliases for backward compatibility.

Then move to Phase 2.

**Time:** ~30 minutes to implement this cleanup  
**Benefit:** Clean codebase + backward compatibility  
**Risk:** Very low (type aliases are simple)
