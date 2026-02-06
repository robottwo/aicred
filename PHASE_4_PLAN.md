# Phase 4: Technical Debt Cleanup - PLAN

**Goal:** Zero clippy warnings at pedantic level  
**Current State:** 35+ clippy allows across codebase  
**Estimated Time:** 5-7 days  
**Strategy:** Start with easy wins, work toward harder fixes

## Current Allows (lib.rs)

```rust
#![allow(clippy::option_if_let_else)]           // Easy - use combinator methods
#![allow(clippy::missing_errors_doc)]            // Medium - add docs
#![allow(clippy::struct_excessive_bools)]        // Hard - refactor structs
#![allow(clippy::cast_precision_loss)]           // Easy - add casts or suppress specific
#![allow(clippy::unnecessary_wraps)]             // Medium - return T instead of Result<T>
#![allow(clippy::match_wildcard_for_single_variants)]  // Easy - be explicit
#![allow(clippy::significant_drop_tightening)]   // Medium - scope tightening
#![allow(clippy::unused_self)]                   // Easy - remove or make fn
#![allow(clippy::if_same_then_else)]            // Easy - deduplicate
#![allow(clippy::implicit_clone)]                // Easy - explicit .clone()
#![allow(clippy::too_many_lines)]               // Hard - extract functions
#![allow(clippy::needless_borrow)]              // Easy - remove &
#![allow(clippy::module_inception)]              // Easy - rename
#![allow(clippy::float_cmp)]                     // Easy - use approx comparison
#![allow(clippy::len_zero)]                      // Easy - use .is_empty()
#![allow(unused_imports)]                        // Easy - remove
#![allow(unused_variables)]                      // Easy - use or prefix _
```

## Step-by-Step Plan

### Step 1: Low-Hanging Fruit (Day 1) - 2 hours

**Target:** Easy mechanical fixes

1. **unused_imports** - `cargo fix --allow-dirty`
2. **unused_variables** - prefix with `_` or remove
3. **needless_borrow** - remove unnecessary `&`
4. **len_zero** - replace `.len() == 0` with `.is_empty()`
5. **implicit_clone** - add explicit `.clone()` calls
6. **module_inception** - rename conflicting modules

**Expected:** ~6-7 allows removed, ~50-100 warnings fixed

### Step 2: Documentation (Day 1-2) - 3 hours

**Target:** missing_errors_doc

For every `pub fn` that returns `Result<T>`:
```rust
/// Does something.
///
/// # Errors
/// Returns `Error::NotFound` if the file doesn't exist.
/// Returns `Error::ParseError` if the file is malformed.
pub fn load_config(path: &Path) -> Result<Config> { ... }
```

**Template:**
- Read function
- Identify error paths
- Document each error case
- Add to docstring

**Expected:** ~50-80 functions documented, 1 allow removed

### Step 3: Control Flow Simplification (Day 2-3) - 4 hours

**Target:**
- option_if_let_else
- match_wildcard_for_single_variants
- if_same_then_else

**Before:**
```rust
let value = if let Some(x) = opt {
    x
} else {
    default
};
```

**After:**
```rust
let value = opt.unwrap_or(default);
```

**Tools:**
- `cargo clippy --fix` for some
- Manual review for complex cases
- Extract shared logic for if_same_then_else

**Expected:** 3 allows removed, ~20-40 instances fixed

### Step 4: Type & Precision Issues (Day 3) - 2 hours

**Target:**
- cast_precision_loss
- float_cmp
- unnecessary_wraps

**cast_precision_loss:**
```rust
// If precision loss is acceptable, add inline allow
#[allow(clippy::cast_precision_loss)]
let ratio = count as f64 / total as f64;
```

**float_cmp:**
```rust
// Before
if score == 0.5 { ... }

// After
const EPSILON: f64 = 1e-10;
if (score - 0.5).abs() < EPSILON { ... }
```

**unnecessary_wraps:**
- Either: Make function fallible (add real error paths)
- Or: Change return type from `Result<T>` to `T`

**Expected:** 3 allows removed or localized

### Step 5: Function Extraction (Day 4-5) - 6 hours

**Target:** too_many_lines (>100 lines)

**Process:**
1. Identify long functions: `cargo clippy 2>&1 | grep "too_many_lines"`
2. For each function:
   - Identify logical sections
   - Extract helpers
   - Keep main function as high-level orchestration
3. Aim for <80 lines per function

**Example:**
```rust
// Before: 300-line function
pub fn process_scan(options: &ScanOptions) -> Result<ScanResult> {
    // ... 100 lines of setup ...
    // ... 100 lines of scanning ...
    // ... 100 lines of result building ...
}

// After: Multiple focused functions
pub fn process_scan(options: &ScanOptions) -> Result<ScanResult> {
    let configs = discover_configs(&options)?;
    let creds = extract_credentials(&configs)?;
    let validated = validate_credentials(creds)?;
    build_scan_result(validated)
}
```

**Expected:** 1 allow removed, 5-10 functions refactored

### Step 6: Struct Refactoring (Day 5-6) - 6 hours

**Target:** struct_excessive_bools

**Current issues:**
- Structs with 4+ bool fields
- Hard to understand what combinations are valid

**Strategy A - Use enums:**
```rust
// Before
pub struct Config {
    pub enable_chat: bool,
    pub enable_completion: bool,
    pub enable_embedding: bool,
}

// After
pub enum Capability {
    Chat,
    Completion,
    Embedding,
    All,
}

pub struct Config {
    pub capabilities: HashSet<Capability>,
}
```

**Strategy B - Use bitflags:**
```rust
use bitflags::bitflags;

bitflags! {
    pub struct Capabilities: u32 {
        const CHAT = 0b0001;
        const COMPLETION = 0b0010;
        const EMBEDDING = 0b0100;
    }
}
```

**Expected:** 1 allow removed, 3-5 structs refactored

### Step 7: Async Cleanup (Day 6) - 3 hours

**Target:** Remove async-trait, minimize Tokio features

**Current:**
```toml
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
```

**After:**
```toml
tokio = { version = "1.0", features = ["rt", "net", "time"] }
# async-trait removed - use native async fn in trait (Rust 1.75+)
```

**Code changes:**
```rust
// Before
#[async_trait]
pub trait ProviderPlugin {
    async fn probe_models(&self, key: &str) -> Result<Vec<String>>;
}

// After (Rust 1.75+)
pub trait ProviderPlugin {
    async fn probe_models(&self, key: &str) -> Result<Vec<String>>;
}
```

**Expected:** Simpler dependencies, potentially faster compile times

### Step 8: Remaining Allows (Day 7) - 3 hours

**Target:** unused_self, significant_drop_tightening

**unused_self:**
- Make into associated function if self never used
- Or: Keep and document why (e.g., trait requirement)

**significant_drop_tightening:**
```rust
// Before
let guard = mutex.lock().unwrap();
// ... 50 lines later ...
drop(guard);

// After
{
    let guard = mutex.lock().unwrap();
    // ... use guard ...
} // guard dropped here
```

**Expected:** 2 allows removed or justified with inline comments

## Testing Strategy

After each step:
```bash
# 1. Check for new warnings
cargo clippy --all-features -- -D warnings

# 2. Run tests
cargo test --all-features

# 3. Check benchmarks haven't regressed
cargo bench --bench scan_benchmark

# 4. Verify no panics in debug mode
RUST_BACKTRACE=1 cargo test
```

## Rollback Points

Each step is a separate commit:
- `Step 1: Low-hanging fruit fixes`
- `Step 2: Add error documentation`
- `Step 3: Simplify control flow`
- etc.

Can revert any step if it causes problems.

## Success Criteria

**Must Have:**
- [ ] Zero clippy warnings with pedantic + nursery
- [ ] All tests passing
- [ ] No performance regression (benchmarks within 5%)
- [ ] Clean commit history (one commit per step)

**Nice to Have:**
- [ ] Improved code coverage
- [ ] Faster compile times (fewer dependencies)
- [ ] Smaller binary size

## Risk Mitigation

**High Risk Changes:**
- Struct refactoring (changes public API)
- Function extraction (could break logic)
- Async trait changes (compatibility)

**Mitigation:**
- Extra test coverage before refactoring
- Gradual rollout (one struct at a time)
- Keep deprecated types as aliases during transition

## Timeline

| Day | Steps | Hours | Completions |
|-----|-------|-------|-------------|
| 1 | Steps 1-2 | 5 | Low-hanging fruit + docs |
| 2-3 | Step 3 | 4 | Control flow |
| 3 | Step 4 | 2 | Type/precision |
| 4-5 | Step 5 | 6 | Function extraction |
| 5-6 | Step 6 | 6 | Struct refactoring |
| 6 | Step 7 | 3 | Async cleanup |
| 7 | Step 8 | 3 | Remaining allows |
| **Total** | **8 steps** | **29 hours** | **~7 days** |

## Out of Scope (Phase 5)

- Example updates
- README updates
- Architecture docs
- Migration guide updates

Those will be done in Phase 5 (Documentation & Polish).

---

**Ready to begin?** Starting with Step 1: Low-Hanging Fruit
