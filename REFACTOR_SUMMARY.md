# AICred Refactoring Summary

**Branch:** `code-cleanup`  
**Documents:** See `CODE_AUDIT.md` and `REFACTOR_PLAN.md` for details

## The Problem

AICred has accumulated technical debt that impacts maintainability:
- 30+ blanket clippy suppressions masking code quality issues
- 18 model files with overlapping concepts (tag vs label confusion)
- Scanner architecture with admitted no-op stubs
- Over-engineered plugin system (472 LOC wrapping a HashMap)

## The Solution

Systematic 5-phase refactoring over 3-4 weeks:

### Phase 1: Model Consolidation (5-7 days)
- **18 model files → 6 files** (67% reduction)
- Unify Tag/Label into single "Label" concept
- Delete deprecated `ProviderConfig`
- Merge overlapping types (DiscoveredKey + ProviderKey → DiscoveredCredential)

### Phase 2: Scanner Simplification (4-5 days)
- **Delete `scanner/` module** (no-op stub)
- **Rename `scanners/` → `discovery/`**
- Extract `BaseScanner` to eliminate 30% code duplication
- Single clear entry point: `DiscoveryEngine`

### Phase 3: Plugin System Reduction (3-4 days)
- **472 LOC → ~100 LOC** (79% reduction)
- Replace wrapper class with direct HashMap + helper functions
- Remove `CommonConfigPlugin` (unused abstraction)
- Apply YAGNI principle

### Phase 4: Technical Debt Cleanup (5-7 days)
- **Remove ALL 30+ clippy allows**
- Fix `struct_excessive_bools` → use enums/bitflags
- Fix `too_many_lines` → extract helper functions
- Clean async architecture (only model probing needs async)
- Reduce Tokio features from `["full"]` to `["rt", "net", "time"]`
- Delete all dead code

### Phase 5: Documentation & Polish (3-4 days)
- Complete API documentation (no `missing_docs` warnings)
- Create `MIGRATION_0.1_to_0.2.md` guide
- Update architecture docs
- Add backward compatibility via `compat_v0_1` feature flag

## Metrics

### Before (Current)
```
Total LOC:       36,000
Model files:     18
Clippy allows:   30+
Plugin system:   472 LOC
Dead code:       Unknown (masked by allows)
```

### After (Target)
```
Total LOC:       28,000  (-22%)
Model files:     6       (-67%)
Clippy allows:   0       (-100%)
Plugin system:   100 LOC (-79%)
Dead code:       0       (eliminated)
```

## Timeline

| Phase | Days | Status |
|-------|------|--------|
| 0. Setup | 1 | Not started |
| 1. Models | 5-7 | Not started |
| 2. Scanner | 4-5 | Not started |
| 3. Plugin | 3-4 | Not started |
| 4. Debt | 5-7 | Not started |
| 5. Docs | 3-4 | Not started |
| **Total** | **21-28** | **0% complete** |

## Risk Mitigation

### Low Risk
- Model consolidation (good test coverage)
- Clippy warning fixes (incremental)
- Dead code deletion

### Medium Risk
- Scanner architecture (affects CLI)
- Plugin simplification (affects extensions)

### High Risk
- Tag/Label unification (breaking API change)

**Mitigation:**
- Feature flags for backward compatibility
- Comprehensive regression tests before each phase
- Keep git history clean for easy rollback
- Semantic versioning (0.2.0 release)

## Quick Wins (Can Do Today)

These can be completed in 1-2 hours:

```bash
# 1. Delete deprecated file
git rm core/src/models/provider_config.rs

# 2. Fix obvious dead code
cargo clippy --fix -- -W dead_code

# 3. Create terminology decision doc
cat > TERMINOLOGY.md << EOF
# Canonical Terms
- Label (not Tag)
- Credential (not Key)
- Instance (for provider configs)
EOF

# 4. Add baseline metrics
cargo test > test-baseline.txt
cargo bench > bench-baseline.txt
```

## Breaking Changes

Version 0.2.0 will include breaking changes:

**Renamed Types:**
- `DiscoveredKey` → `DiscoveredCredential`
- `ValueType` → `CredentialValue`
- `Tag` → `Label`
- `TagAssignment` → `LabelAssignment`

**Removed Types:**
- `ProviderConfig` (deprecated, replaced by `ProviderInstance`)
- `PluginRegistry` (replaced by direct HashMap)
- `Scanner` (replaced by `DiscoveryEngine`)

**Changed APIs:**
- `scan()` function signature simplified
- Plugin registration via function, not method

**Backward Compatibility:**
```toml
# Add to Cargo.toml
[dependencies]
aicred-core = { version = "0.2", features = ["compat_v0_1"] }
```

## Testing Strategy

After each phase:
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Benchmarks within 5% of baseline
- [ ] CLI functional test
- [ ] Python bindings build
- [ ] Go bindings build
- [ ] Documentation builds

## Communication

**Daily:** Status update in team channel  
**Weekly:** Review meeting with stakeholders  
**Per Phase:** Demo of completed work

## Success Criteria

✅ **Code Quality:**
- Zero clippy warnings at pedantic level
- No dead code or unused imports
- Consistent naming per TERMINOLOGY.md

✅ **Maintainability:**
- 67% fewer model files
- 79% less plugin boilerplate
- <300 LOC average file length

✅ **Performance:**
- Benchmarks within 5% of baseline
- No regression in scan speed
- Binary size unchanged or smaller

✅ **Documentation:**
- >95% doc coverage
- Complete migration guide
- Updated architecture docs

## Next Steps

1. **Review** this plan with Dan ✅
2. **Decide** on timeline (3-4 weeks acceptable?)
3. **Approve** breaking changes for 0.2.0
4. **Begin** Phase 0 (setup & baseline)
5. **Execute** phases 1-5 systematically

## Questions for Dan

1. **Timeline:** Is 3-4 weeks acceptable? Or need faster delivery?
2. **Breaking Changes:** OK to release as 0.2.0 with breaking changes?
3. **FFI/Bindings:** Are Python/Go bindings actively used? Should we maintain them?
4. **GUI:** Is Tauri GUI being maintained? Can we archive it?
5. **Release Strategy:** Beta release first, or direct to 0.2.0?

---

**Files to Review:**
- `CODE_AUDIT.md` - Full analysis of issues (524 lines)
- `REFACTOR_PLAN.md` - Detailed implementation plan (1978 lines)
- This file - Executive summary

**Ready to start?** Say the word and I'll begin Phase 0 (setup).
