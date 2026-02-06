# Internal Migration Status - In Progress

**Started:** 2026-02-05  
**Approach:** Option 1 - Complete internal migration  
**Status:** ~30% complete, structural incompatibilities blocking progress

## What's Been Done (30 minutes)

### Step 1: File Renaming ✅
- Renamed `*_new.rs` → canonical names:
  - `credentials_new.rs` → `credentials.rs`
  - `labels_new.rs` → `labels.rs`
  - `providers_new.rs` → `providers.rs`
  - `models_new.rs` → `models.rs`
  - `scan_new.rs` → `scan.rs`

### Step 2: Export Updates ✅
- Updated `models/mod.rs` to export new types as primary
- Updated `lib.rs` to export new types as primary API
- Kept old types re-exported for backward compatibility

### Step 3: Compatibility Methods - Partial ✅
Added to `ProviderInstance`:
- `get_api_key()` → Returns `Option<&String>`
- `has_non_empty_api_key()` → Returns `bool`
- `new()` → Constructor
- `validate()` → Validation stub

Added to `Model`:
- `new()` → Basic constructor
- `validate()` → Validation stub

## Current Blockers (~259 compilation errors remaining)

### Issue 1: ProviderInstance Structure (85 errors)
**Problem:** Old vs New structure mismatch

**Old ProviderInstance:**
```rust
pub struct ProviderInstance {
    models: Vec<Model>,  // Complex Model objects
    // Had methods: add_model(), set_api_key(), has_api_key(), model_count()
}
```

**New ProviderInstance:**
```rust
pub struct ProviderInstance {
    models: Vec<String>,  // Just model IDs
    api_key: String,      // Direct field
    // Missing: add_model(), set_api_key(), has_api_key(), model_count()
}
```

**Affected Code:**
- 33 calls to `add_model()`
- 20 calls to `set_api_key()`
- 6 calls to `has_api_key()`
- 2 calls to `model_count()`
- 12+ places expecting `model.model_id` field (but models are now Strings)

**Fix Required:** Add these methods to new `ProviderInstance`, converting String <-> Model as needed

### Issue 2: Environment Enum (12 errors)
**Problem:** Missing `Production` variant

**Old:**
```rust
enum Environment {
    SystemConfig,
    UserConfig,
    ProjectConfig,
    EnvironmentVariable,
    Production,  // ❌ Missing in new
}
```

**New:**
```rust
enum Environment {
    SystemConfig,
    UserConfig,
    ProjectConfig { project_path: String },
    EnvironmentVariable,
    // No Production variant
}
```

**Fix Required:** Add `Production` variant or map old code to new variant

### Issue 3: Model/ModelMetadata Confusion (11 errors)
**Problem:** Old code expects metadata fields on Model directly

**Old code expects:**
```rust
model.id
model.name  
model.pricing
model.context_length
```

**New structure:**
```rust
model.id         // ✅ Exists
model.name       // ✅ Exists
model.pricing    // ✅ Exists
model.metadata.context_window  // ❌ Was context_length
```

**Fix Required:** Add field aliases or update call sites

### Issue 4: Type Conversions (18 errors)
**Problem:** Missing From/Into implementations

**Needed:**
- `From<ProviderConfig> for ProviderInstance`
- `From<ProviderInstance> for ProviderConfig`
- String conversion for ModelMetadata fields

**Fix Required:** Implement conversion traits

### Issue 5: HashMap Methods (11 errors)
**Problem:** Code calling `.as_ref()` on HashMap

**Example:**
```rust
let map: HashMap<K, V> = ...;
map.as_ref()  // ❌ HashMap doesn't have as_ref()
```

**Fix Required:** Determine intent and fix call sites

## Remaining Work Estimate

### Quick Fixes (30-45 minutes)
- Add `Production` to Environment enum
- Add missing methods to ProviderInstance (add_model, set_api_key, etc.)
- Fix field name mismatches (context_length → context_window)

### Medium Fixes (1-2 hours)
- Implement conversion traits (From/Into)
- Fix HashMap.as_ref() issues
- Handle models Vec<String> vs Vec<Model> throughout codebase

### Testing & Validation (1 hour)
- Run all 111 tests
- Fix any runtime issues
- Verify backward compatibility

**Total Remaining:** 2.5-4 hours

## Path Forward

### Option A: Continue Migration (2.5-4 hours)
**Pros:**
- Complete the job
- Single source of truth
- Clean internal codebase

**Cons:**
- 2.5-4 more hours of work
- Risk of introducing subtle bugs
- Already invested 30 minutes

**Total Time:** 3-4.5 hours

### Option B: Revert & Ship v0.2.0 Now
**Pros:**
- Production-ready immediately
- Zero risk
- Can do migration in v0.2.1

**Cons:**
- Dual API remains
- `*_new` naming persists
- Work done so far wasted (sunk cost)

**Total Time:** 5 minutes (git reset)

## Recommendation

**Continue with Option A** - We're 30% done, understand the issues, and have clear fixes.

**Why:**
1. We've identified all the blockers
2. Fixes are straightforward (add methods, add enum variants, fix field names)
3. 2.5-4 hours total is reasonable
4. Dan explicitly chose Option 1
5. Better to finish properly than ship with awkward internal state

**Next Steps:**
1. Add missing methods to ProviderInstance (~30 min)
2. Fix Environment enum (~5 min)
3. Fix field name issues (~15 min)
4. Implement conversion traits (~30 min)
5. Fix HashMap issues (~20 min)
6. Run tests and fix remaining issues (~1-2 hours)

**ETA to complete:** 2.5-4 hours from now

## Alternative: Phased Approach

If 4 hours is too long now:

**Phase A (now, 1 hour):**
- Add missing methods to ProviderInstance
- Fix Environment enum
- Get lib tests passing

**Phase B (later, 1 hour):**
- Fix discovery tests
- Fix provider tests

**Phase C (later, 1 hour):**
- Final cleanup
- All tests green

This allows incremental progress with stopping points.
