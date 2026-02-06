# AICred Code Audit & Simplification Proposal

**Date:** 2026-02-04  
**Branch:** `code-cleanup`  
**Total Lines:** ~36,000 Rust LOC across 86 files

## Executive Summary

AICred is a well-structured Rust project for discovering and managing AI provider credentials. The codebase shows good engineering practices but has accumulated complexity that impacts maintainability. This audit identifies key areas for simplification and architectural improvements.

## Current Architecture

### Component Overview
```
aicred/
├── core/              # 16.5k LOC - Core scanning & provider logic
│   ├── models/        # 18 model files - data structures
│   ├── providers/     # 7 provider plugins
│   ├── scanners/      # 6 application scanners
│   ├── parser/        # Config file parsing
│   └── plugins/       # Plugin system
├── cli/               # CLI tool
├── ffi/               # C FFI layer
├── bindings/
│   ├── python/        # PyO3 bindings
│   └── go/            # CGO bindings
└── gui/               # Tauri desktop app
```

### Strengths
1. **Clear separation of concerns** - Core, CLI, bindings well separated
2. **Comprehensive test coverage** - 17 test files with integration tests
3. **Good error handling** - Custom Error type with thiserror
4. **Plugin architecture** - Extensible provider/scanner system
5. **Security-first** - Keys redacted by default (SHA-256 hashing)

## Issues & Code Smells

### 1. Excessive Clippy Disables (CRITICAL)
**Location:** Throughout codebase (lib.rs, main.rs, models/mod.rs)

```rust
#![allow(clippy::option_if_let_else)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unused_self)]
// ... 15+ more allows
```

**Impact:**
- Masks real code quality issues
- Prevents future improvements
- Suggests underlying design problems

**Recommendation:**
- Remove ALL blanket allows
- Fix underlying issues instead of suppressing warnings
- Enable clippy::pedantic gradually with specific justifications

### 2. Model Proliferation (HIGH)
**Location:** `core/src/models/` - 18 separate model files

**Issues:**
- **Tag vs Label confusion** - Both `tag.rs`, `tag_assignment.rs` AND `label.rs`, `label_assignment.rs` exist
- **Deprecated models** - `ProviderConfig` marked deprecated but still in use
- **Overlapping concepts** - `ConfigInstance`, `ProviderInstance`, `ProviderKey` all represent similar concepts
- **Excessive granularity** - Separate files for `model_metadata.rs` (159 LOC) and `model.rs` (297 LOC)

**Current Model Count:**
```
config_instance.rs (12.3k)
discovered_key.rs
label.rs + label_assignment.rs (593 LOC combined)
tag.rs + tag_assignment.rs (408 LOC combined)
provider.rs
provider_config.rs (deprecated)
provider_instance.rs (645 LOC)
provider_instances.rs (400 LOC)
provider_key.rs
model.rs
model_metadata.rs
scan_result.rs
unified_label.rs
```

**Recommendation:**
- **Consolidate models:**
  ```rust
  // Merge into fewer files:
  provider.rs       // Provider, ProviderInstance, AuthMethod
  credentials.rs    // DiscoveredKey, ProviderKey (merge concepts)
  labeling.rs       // Label, Tag, Assignment (unified concept)
  models.rs         // Model, ModelMetadata, Capabilities
  results.rs        // ScanResult, ScanSummary
  config.rs         // ConfigInstance (if still needed)
  ```
- **Eliminate tag/label duplication** - Pick ONE concept
- **Remove deprecated code** immediately

### 3. Scanner Architecture Confusion (HIGH)
**Location:** `core/src/scanner/mod.rs`, `core/src/scanners/mod.rs`

**Issues:**
- `Scanner` struct (singular) in `scanner/mod.rs` - only 150 LOC, mostly boilerplate
- `ScannerRegistry` in `scanners/mod.rs` - actual scanning logic
- Comment admits architecture confusion:
  ```rust
  // Scanner-specific scanning is now handled by scan_with_scanners in lib.rs
  // This method just initializes the result structure for compatibility
  ```
- Scanner::scan() method is essentially a no-op now

**Recommendation:**
- **Eliminate `scanner/` module entirely** - move functionality to `scanners/mod.rs`
- Rename `scanners/` to `discovery/` for clarity
- Direct architecture:
  ```rust
  // In discovery/mod.rs
  pub struct DiscoveryEngine {
      provider_registry: ProviderRegistry,
      app_scanners: Vec<Box<dyn AppScanner>>,
  }
  ```

### 4. Plugin System Over-Engineering (MEDIUM)
**Location:** `core/src/plugins/mod.rs` - 472 LOC

**Issues:**
- `PluginRegistry` wraps a simple `HashMap<String, Box<dyn ProviderPlugin>>`
- Most methods are thin wrappers around HashMap operations
- Adds indirection without clear value
- `CommonConfigPlugin` trait used in only one place

**Current:**
```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn ProviderPlugin>>,
}

impl PluginRegistry {
    pub fn get(&self, name: &str) -> Option<&dyn ProviderPlugin> { ... }
    pub fn list(&self) -> Vec<&str> { ... }
    pub fn register(&mut self, plugin: Box<dyn ProviderPlugin>) { ... }
}
```

**Recommendation:**
- Simplify to direct HashMap with helper functions:
  ```rust
  pub type ProviderRegistry = HashMap<String, Box<dyn ProviderPlugin>>;
  
  pub fn register_builtin_providers() -> ProviderRegistry {
      let mut registry = HashMap::new();
      registry.insert("openai".into(), Box::new(OpenAIPlugin) as Box<dyn ProviderPlugin>);
      // ...
      registry
  }
  ```
- If complex logic needed later, add it then (YAGNI principle)

### 5. Excessive Async (MEDIUM)
**Location:** Provider plugins, model probing

**Issues:**
- Most provider validation is synchronous
- `async-trait` dependency adds complexity
- Tokio included with `features = ["full"]` (heavyweight)
- Model probing is the only true async operation

**Recommendation:**
- Make model probing explicitly async-only
- Keep validation synchronous
- Use `tokio::runtime::Runtime::new()` only where needed
- Reduce Tokio feature set to minimum required

### 6. Inconsistent Naming (LOW-MEDIUM)
**Examples:**
- `ProviderInstance` vs `ConfigInstance` - overlapping concepts
- `ScannerPlugin` vs `ProviderPlugin` - different naming patterns
- `register_builtin_plugins()` vs `register_builtin_scanners()`
- `ValueType` enum in discovered_key.rs (Full/Redacted) vs general concept

**Recommendation:**
- Standardize on consistent terminology:
  - Provider = external API service (OpenAI, Anthropic)
  - Instance = configured provider endpoint (API key, base URL)
  - Scanner = app-specific config discoverer
  - Credential = discovered API key

### 7. Dead Code & Unused Features (MEDIUM)
**Findings:**
```rust
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
```

**Recommendation:**
- Remove ALL `allow(dead_code)` attributes
- Run `cargo clippy --fix` with warnings as errors
- Delete genuinely unused code
- Make tests compile without dead_code allows

## Architectural Simplification Plan

### Phase 1: Model Consolidation (Week 1)

**Goal:** Reduce 18 model files to 6-7 focused modules

1. **Merge overlapping concepts:**
   ```rust
   // providers.rs (merge provider.rs + provider_instance.rs + provider_instances.rs)
   pub struct Provider { /* metadata */ }
   pub struct ProviderInstance { /* configured instance */ }
   pub struct ProviderCollection { /* instances.yaml representation */ }
   
   // credentials.rs (merge discovered_key.rs + provider_key.rs)
   pub struct DiscoveredCredential { /* found during scan */ }
   pub enum CredentialValue { Full(String), Redacted(String) }
   
   // labels.rs (merge tag.rs + label.rs + unified_label.rs + assignments)
   pub struct Label { /* single concept for categorization */ }
   pub struct LabelAssignment { /* links labels to instances */ }
   
   // models.rs (merge model.rs + model_metadata.rs)
   pub struct Model { /* LLM model with metadata */ }
   
   // scan.rs (scan_result.rs only)
   pub struct ScanResult { /* scan output */ }
   
   // config.rs (if needed - consider removing)
   pub struct ConfigInstance { /* legacy? */ }
   ```

2. **Delete deprecated code:**
   - Remove `provider_config.rs` entirely
   - Migrate any remaining usages

3. **Unify Tag/Label:**
   - Choose ONE term (suggest "Label" - more intuitive)
   - Migrate all references
   - Delete duplicate implementations

**Success Metric:** Reduce models/ from 18 files to ≤7 files

### Phase 2: Scanner Simplification (Week 1-2)

**Goal:** Single, clear discovery architecture

1. **Eliminate scanner/ module:**
   ```bash
   git rm -r core/src/scanner/
   ```

2. **Rename scanners/ → discovery/:**
   ```rust
   // core/src/discovery/mod.rs
   pub struct DiscoveryEngine {
       providers: ProviderRegistry,
       app_scanners: Vec<Box<dyn AppScanner>>,
   }
   
   impl DiscoveryEngine {
       pub fn scan(&self, home_dir: &Path, options: &ScanOptions) -> Result<ScanResult> {
           // Direct, clear implementation
       }
   }
   ```

3. **Consolidate scanner implementations:**
   - Current: `claude_desktop.rs` (619 LOC), `roo_code.rs` (869 LOC), `gsh.rs` (988 LOC)
   - Each follows similar pattern with copy-paste
   - Extract common scanner base struct
   - Reduce duplication

**Success Metric:** Single discovery entry point, <30% code reduction in scanners

### Phase 3: Plugin System Simplification (Week 2)

**Goal:** Remove unnecessary abstractions

1. **Replace PluginRegistry:**
   ```rust
   // Before: 472 LOC with registry wrapper
   // After: ~50 LOC with direct HashMap + helpers
   
   pub type ProviderRegistry = HashMap<String, Box<dyn ProviderPlugin>>;
   
   pub fn register_builtin_providers() -> ProviderRegistry { ... }
   pub fn get_provider(registry: &ProviderRegistry, name: &str) -> Option<&dyn ProviderPlugin> { ... }
   ```

2. **Simplify plugin traits:**
   - Remove `CommonConfigPlugin` if only one implementation
   - Inline small trait methods
   - Reduce trait requirements to minimum

**Success Metric:** 60% reduction in plugin system LOC

### Phase 4: Clean Up Technical Debt (Week 2-3)

1. **Remove ALL blanket clippy allows:**
   ```bash
   # Find all allow directives
   rg '#!\[allow\(' --type rust
   
   # Fix each category:
   # 1. option_if_let_else → refactor to clearer patterns
   # 2. struct_excessive_bools → use enums or builder pattern
   # 3. too_many_lines → split large functions
   # 4. unused_* → delete or use the code
   ```

2. **Fix async architecture:**
   - Make sync validation truly sync
   - Isolate async model probing
   - Reduce Tokio footprint:
     ```toml
     tokio = { version = "1.0", features = ["rt", "net", "time"] }
     ```

3. **Eliminate dead code:**
   ```bash
   cargo clippy -- -W dead_code
   cargo clippy -- -W unused-imports
   cargo clippy -- -W unused-variables
   ```

4. **Standardize naming:**
   - Create TERMINOLOGY.md with canonical terms
   - Bulk rename with confidence (good test coverage)
   - Use clippy::enum_variant_names for consistency

**Success Metric:** Zero clippy warnings with default lint level

### Phase 5: Documentation & Polish (Week 3)

1. **Update architecture docs:**
   - Reflect simplified structure
   - Remove outdated diagrams
   - Document new organization

2. **Improve API documentation:**
   - Remove `#![allow(clippy::missing_errors_doc)]`
   - Document all public APIs
   - Add module-level docs

3. **Create migration guide:**
   - Document breaking changes
   - Provide upgrade path for library users
   - Update all examples

## File-Level Recommendations

### Immediate Deletions
```bash
# Deprecated/unused files to remove:
core/src/models/provider_config.rs          # Deprecated
core/src/scanner/mod.rs                     # To be merged
```

### Files to Merge
```
# Model consolidation:
models/tag.rs + models/label.rs → models/labels.rs
models/tag_assignment.rs + models/label_assignment.rs + models/unified_label.rs → models/labels.rs
models/provider.rs + models/provider_instance.rs + models/provider_instances.rs → models/providers.rs
models/model.rs + models/model_metadata.rs → models/models.rs
models/discovered_key.rs + models/provider_key.rs → models/credentials.rs

# Reduce from 18 files to ~6 files
```

### Files to Simplify
```
core/src/lib.rs                # Remove excessive clippy allows (currently 17 allows)
core/src/plugins/mod.rs        # 472 LOC → ~100 LOC (reduce abstraction)
core/src/env_resolver.rs       # 795 LOC - audit for simplification
cli/src/main.rs                # 18k LOC - consider splitting commands further
```

## Metrics & Success Criteria

### Current State
- **Total LOC:** ~36,000
- **Model files:** 18
- **Clippy allows:** 30+ blanket suppression rules
- **Module depth:** 4 levels (scanner vs scanners confusion)
- **Test files:** 17 (good!)

### Target State (Post-Cleanup)
- **Total LOC:** ~28,000 (22% reduction)
- **Model files:** 6-7 (60% reduction)
- **Clippy allows:** 0 blanket suppression (100% reduction)
- **Module depth:** 3 levels (clear hierarchy)
- **Test files:** 17 (maintain coverage)

### Measurements
```bash
# Before
find core/src/models -name "*.rs" | wc -l  # 18 files
rg '#!\[allow\(' --type rust | wc -l       # ~30+ allows
tokei core/src                              # ~16.5k LOC

# After (target)
find core/src/models -name "*.rs" | wc -l  # ≤7 files
rg '#!\[allow\(' --type rust | wc -l       # 0 allows
tokei core/src                              # ~12k LOC
```

## Risk Assessment

### Low Risk
- Model file consolidation (good test coverage)
- Clippy allow removal (can be done incrementally)
- Dead code deletion

### Medium Risk
- Scanner architecture change (affects CLI commands)
- Plugin system simplification (affects extensions)
- Async refactoring (needs careful testing)

### High Risk
- Tag/Label unification (breaking API change)
- Provider model merges (widely used)

**Mitigation:**
- Make changes in feature branches
- Run full test suite after each phase
- Keep git history clean for easy rollback
- Consider semantic versioning for breaking changes

## Implementation Order

### Week 1: Foundation
1. Document current API surface (public exports)
2. Add integration tests for critical paths
3. Create feature flags for backward compatibility
4. Start model consolidation (low risk items first)

### Week 2: Core Refactoring
5. Scanner architecture simplification
6. Plugin system reduction
7. Remove deprecated code
8. Async cleanup

### Week 3: Quality & Polish
9. Fix all clippy warnings
10. Add missing documentation
11. Update examples
12. Performance benchmarking

### Week 4: Validation
13. Full test suite (including property tests)
14. Integration testing with real configs
15. Documentation review
16. Release preparation

## Immediate Quick Wins

Can be done in first day:

```bash
# 1. Remove deprecated provider_config.rs
git rm core/src/models/provider_config.rs
rg -l 'provider_config' core/ | xargs sed -i '' '/use.*provider_config/d'

# 2. Fix obvious dead code warnings
cargo clippy --fix -- -W dead_code

# 3. Consolidate tag/label decision (choose "label")
# (requires manual refactoring but clear path)

# 4. Remove scanner/mod.rs stub
git rm core/src/scanner/mod.rs
# Move functionality to scanners/mod.rs
```

## Long-Term Architecture Vision

```
aicred/
├── core/
│   ├── models/          # 6-7 focused model files
│   │   ├── providers.rs
│   │   ├── credentials.rs
│   │   ├── labels.rs
│   │   ├── models.rs
│   │   ├── scan.rs
│   │   └── mod.rs
│   ├── providers/       # 7 provider implementations (unchanged)
│   ├── discovery/       # renamed from scanners/
│   ├── parser/          # config file parsing (unchanged)
│   ├── error.rs
│   ├── env_resolver.rs
│   └── lib.rs           # clean public API
├── cli/                 # command-line interface
├── ffi/                 # C API (only if needed)
├── bindings/            # language bindings
└── gui/                 # optional GUI

# Clean, clear, maintainable
```

## Questions for Stakeholder

1. **Tag vs Label:** Which term should we standardize on?
2. **FFI/Bindings:** Are Python/Go bindings actively used? Can we simplify FFI?
3. **GUI:** Is Tauri GUI actively maintained? Consider archiving if unused
4. **Breaking Changes:** Acceptable for 0.2.0 release?
5. **Timeline:** 4-week refactor acceptable or need faster delivery?

## Conclusion

AICred has a solid foundation but accumulated technical debt that impacts maintainability. The proposed simplifications will:

- **Reduce cognitive load** - fewer files, clearer structure
- **Improve code quality** - eliminate suppressed warnings
- **Easier onboarding** - more intuitive architecture
- **Better maintainability** - less duplication, clearer patterns

Estimated effort: **3-4 weeks** for systematic refactoring with proper testing.

**Recommended Approach:** Incremental phases with feature flags for backward compatibility during transition.
