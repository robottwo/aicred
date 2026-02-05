# Phase 2: Scanner Simplification - In Progress

**Started:** 2026-02-04  
**Branch:** `code-cleanup`  
**Status:** 50% complete

## Goals

- ✅ Delete `scanner/` module (no-op stub)
- ✅ Rename `scanners/` → `discovery/`
- ⏳ Extract `BaseScanner` to reduce duplication
- ⏳ Update scanner implementations
- ⏳ Clean architecture

## Progress

### Step 1: Delete scanner/ module ✅

**Problem:** The `scanner/` module was a 156-line stub that admitted in comments:
> "Scanner-specific scanning is now handled by scan_with_scanners in lib.rs.  
> This method just initializes the result structure for compatibility"

**Solution:**
- Moved `ScannerConfig` and `DEFAULT_MAX_FILE_SIZE` to `scanners/mod.rs`
- Deleted `core/src/scanner/` directory entirely
- Updated `lib.rs` to remove Scanner struct usage
- **Result:** 156 lines deleted, cleaner architecture

### Step 2: Rename scanners → discovery ✅

**Problem:** "scanners" is ambiguous - scan what? files? configs? directories?

**Solution:**
- Renamed `core/src/scanners` → `core/src/discovery`
- Updated all imports throughout codebase
- Added backward compatibility re-export in `scanners.rs`
- Updated module documentation

**Benefits:**
- Clearer name: "Discovery system for finding AI credentials"
- Better describes purpose
- Backward compatible (old imports still work with deprecation warning)

**Result:** All 111 tests passing, zero breaking changes

## Impact So Far

| Metric | Before Phase 2 | After Step 2 | Change |
|--------|----------------|--------------|---------|
| Scanner modules | 2 (scanner + scanners) | 1 (discovery) | -50% |
| Lines of code (scanner stub) | 156 | 0 | -156 |
| Module clarity | Ambiguous | Clear | ✅ |
| Backward compatibility | N/A | Yes | ✅ |
| Tests passing | 111 | 111 | ✅ |

## Next Steps

### Step 3: Extract BaseScanner

Current scanner files show significant duplication:
- `claude_desktop.rs`: 619 LOC
- `roo_code.rs`: 869 LOC
- `gsh.rs`: 988 LOC
- `langchain.rs`: 570 LOC
- `ragit.rs`: 448 LOC

Common patterns across all:
1. JSON/YAML config file parsing
2. Path construction (platform-specific)
3. Key extraction from parsed configs
4. DiscoveredKey/DiscoveredCredential creation
5. Error handling

**Plan:**
- Create `BaseScanner` struct with common operations
- Extract config file finding/parsing logic
- Extract credential creation helpers
- **Estimated reduction:** 30-40% of scanner code

### Step 4: Refactor scanner implementations

Once BaseScanner exists:
- Update each scanner to use it
- Remove duplicated code
- Simplify implementations

**Estimated time remaining:** 2-3 hours

## Files Changed

### Deleted
- `core/src/scanner/mod.rs` (156 lines)

### Modified
- `core/src/lib.rs` - Updated imports, removed Scanner usage
- `core/src/discovery/mod.rs` - Added ScannerConfig, updated docs
- All discovery scanner files - Renamed from scanners/

### Created
- `core/src/scanners.rs` - Backward compatibility re-export

## Testing

All tests passing:
```
running 111 tests
test result: ok. 111 passed; 0 failed; 0 ignored
```

## Code Quality

- ✅ Zero clippy warnings
- ✅ Clean builds
- ✅ Backward compatible
- ✅ Better naming

## Commits

1. `53ce136` - Phase 2 Step 1: Delete scanner/ module stub (-156 LOC)
2. `b7e5a70` - Phase 2 Step 2: Rename scanners → discovery

## Summary

Phase 2 is progressing well:
- ✅ Removed unnecessary abstraction layer (scanner/ stub)
- ✅ Improved naming (discovery vs scanners)
- ✅ Maintained backward compatibility
- ✅ All tests passing

Ready to continue with BaseScanner extraction when Dan gives the go-ahead.
