# CLI Legacy Type Migration Plan

## Overview
Migrate CLI from legacy types (DiscoveredKey, ProviderKey, ProviderConfig, Tag, TagAssignment, UnifiedLabel) to new API types.

## Legacy → New API Mappings

### 1. DiscoveredKey → DiscoveredCredential
**File:** `cli/src/commands/scan.rs`

| DiscoveredKey Field | DiscoveredCredential Field | Notes |
|-------------------|------------------------|-------|
| provider | provider | Direct mapping |
| source | source_file | Direct mapping |
| value_type | value_type | Direct mapping (same enum) |
| confidence | confidence | Direct mapping (same enum) |
| hash | - | Hash is stored in CredentialValue::Redacted |
| discovered_at | discovered_at | Direct mapping |
| line_number (u32) | source_line (Option<usize>) | Type change |
| column_number | - | Not present in new type |
| metadata | - | Not present in new type |
| full_value (Option<String>) | value (CredentialValue) | Enum wrapper |

**Migration Strategy:**
- Replace `DiscoveredKey` usage with `DiscoveredCredential`
- Convert `full_value()` to check `CredentialValue` enum
- Convert `redacted_value()` to use CredentialValue methods
- Update field access patterns

### 2. ProviderKey → Remove
**Files:** `cli/src/commands/providers.rs`

**Migration Strategy:**
- Remove ProviderKey temporary construction
- Use direct string values for API keys
- Update Confidence and Environment type imports from new location

### 3. ProviderConfig → Remove
**Files:**
- `cli/src/commands/providers.rs` (load_instances_from_providers)
- `cli/src/utils/provider_loader.rs` (parsing logic)

**Migration Strategy:**
- Remove ProviderConfig imports and usage
- Simplify YAML parsing to only support ProviderInstance
- Remove legacy format fallback code
- Keep only direct ProviderInstance parsing

### 4. Tag → Label
**Files:**
- `cli/src/commands/tags.rs`
- `cli/src/output/table.rs`

| Tag Field | Label Field | Notes |
|----------|------------|-------|
| id | name | ID removed, use name as identifier |
| name | name | Direct mapping |
| description | description | Direct mapping |
| color | - | Removed from new type |
| metadata | metadata | Direct mapping |
| created_at | created_at | Direct mapping |
| updated_at | - | Removed from new type |

**Migration Strategy:**
- Replace Tag with Label
- Remove id field generation (use name directly)
- Remove color field (not in Label)
- Remove updated_at field
- Update validation logic

### 5. TagAssignment → LabelAssignment
**File:** `cli/src/commands/tags.rs`

| TagAssignment Field | LabelAssignment Field | Notes |
|-------------------|---------------------|-------|
| id | - | Not in new type |
| tag_id | label_name | Renamed |
| target (TagAssignmentTarget) | target (LabelTarget) | Different enum |
| metadata | - | Not in new type |
| created_at | assigned_at | Renamed |
| updated_at | - | Removed |
| assigned_by | assigned_by | Direct mapping (new) |

**TagAssignmentTarget vs LabelTarget:**
- `TagAssignmentTarget::ProviderInstance { instance_id }` → `LabelTarget::ProviderInstance { instance_id }`
- `TagAssignmentTarget::Model { instance_id, model_id }` → `LabelTarget::ProviderModel { instance_id, model_id }`

**Migration Strategy:**
- Replace TagAssignment with LabelAssignment
- Remove id field generation
- Remove updated_at field
- Update target enum usage
- Update assignment matching logic

### 6. UnifiedLabel → Label + LabelAssignment
**File:** `cli/src/commands/labels.rs`

**UnifiedLabel combines:**
- Label metadata (name, description, color, metadata)
- Assignment information (target, created_at, updated_at)

**Migration Strategy:**
- Split UnifiedLabel into separate Label and LabelAssignment
- Label stores: name, description, created_at, metadata
- LabelAssignment stores: label_name, target, assigned_at, assigned_by
- Update save/load logic to handle both types
- Update CLI commands to work with separate types

## Implementation Order

1. **Phase 1: Core Types Migration** (No file deletions)
   - Migrate scan.rs (DiscoveredKey → DiscoveredCredential)
   - Migrate tags.rs (Tag, TagAssignment → Label, LabelAssignment)
   - Migrate labels.rs (UnifiedLabel → Label + LabelAssignment)

2. **Phase 2: Provider Migration**
   - Migrate providers.rs (remove ProviderKey, ProviderConfig)
   - Migrate provider_loader.rs (remove ProviderConfig parsing)

3. **Phase 3: Output Migration**
   - Migrate table.rs (Tag → Label)

4. **Phase 4: Cleanup**
   - Delete 6 legacy type files
   - Remove exports from core/src/models/mod.rs
   - Remove exports from core/src/lib.rs
   - Verify compilation
   - Run tests

## Success Criteria
- CLI compiles with 0 errors
- 6 legacy files deleted
- No backward compat exports in public API
- Tests pass
