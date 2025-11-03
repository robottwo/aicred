# Core Data Structures and Relationships

## Overview

This document provides a comprehensive overview of the core data structures in the AICred tagging and labeling system, their relationships, and how they interact to provide a robust configuration management solution.

## Core Data Models

### ProviderInstance

The foundational model representing a configured AI provider instance.

```rust
pub struct ProviderInstance {
    pub id: String,
    pub display_name: String,
    pub provider_type: String,
    pub api_endpoint: String,
    pub metadata: HashMap<String, String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Key Characteristics:**
- **Unique Identifier**: Each instance has a unique `id` for system-wide reference
- **Display Name**: Human-readable name for UI and CLI display
- **Provider Type**: Categorizes the AI provider (OpenAI, Anthropic, etc.)
- **API Endpoint**: Base URL for API communications
- **Metadata**: Flexible key-value storage for additional information
- **Active Status**: Controls whether the instance is usable

**Relationships:**
- Can have multiple `TagAssignment` entries
- Can have multiple `LabelAssignment` entries
- References in `TagAssignmentTarget::ProviderInstance`
- References in `LabelAssignmentTarget::ProviderInstance`

### Tag

Represents a categorization tag that can be applied to provider instances and models.

```rust
pub struct Tag {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Key Characteristics:**
- **Flexible Categorization**: Used for organizing and grouping instances
- **Visual Identification**: Optional color coding for UI display
- **Rich Metadata**: Additional context and properties
- **Multiple Assignments**: Can be assigned to multiple targets

**Relationships:**
- Referenced by `TagAssignment` entries
- Can target `ProviderInstance` or `Model` via `TagAssignmentTarget`
- Many-to-many relationship with provider instances and models

### Label

Represents a designation or status label with uniqueness constraints.

```rust
pub struct Label {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Key Characteristics:**
- **Unique Designations**: Often used for exclusive designations (Primary, Backup)
- **Uniqueness Constraints**: Can enforce global uniqueness across assignments
- **Status Management**: Ideal for tracking states and designations
- **Visual Coding**: Optional color for UI identification

**Relationships:**
- Referenced by `LabelAssignment` entries
- Can target `ProviderInstance` or `Model` via `LabelAssignmentTarget`
- Enforces uniqueness constraints through assignment validation

### TagAssignment

Links tags to their target entities (provider instances or models).

```rust
pub struct TagAssignment {
    pub id: String,
    pub tag_id: String,
    pub target: TagAssignmentTarget,
    pub metadata: HashMap<String, String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<String>,
}
```

**Target Types:**
```rust
pub enum TagAssignmentTarget {
    ProviderInstance(String),  // ProviderInstance ID
    Model(String),             // Model identifier
}
```

**Key Characteristics:**
- **Flexible Targeting**: Can assign tags to instances or specific models
- **Assignment Metadata**: Additional context about the assignment
- **Audit Trail**: Tracks who assigned and when
- **Multiple Tags**: Supports multiple tags per target

**Relationships:**
- References `Tag` via `tag_id`
- References target entity via `TagAssignmentTarget`
- One-to-many relationship from Tag to TagAssignment

### LabelAssignment

Links labels to their target entities with uniqueness enforcement.

```rust
pub struct LabelAssignment {
    pub id: String,
    pub label_id: String,
    pub target: LabelAssignmentTarget,
    pub metadata: HashMap<String, String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<String>,
}
```

**Target Types:**
```rust
pub enum LabelAssignmentTarget {
    ProviderInstance(String),  // ProviderInstance ID
    Model(String),             // Model identifier
}
```

**Key Characteristics:**
- **Uniqueness Enforcement**: Prevents duplicate label assignments
- **Exclusive Designations**: Supports primary/backup patterns
- **Assignment Context**: Metadata about the assignment
- **Audit Information**: Tracks assignment details

**Relationships:**
- References `Label` via `label_id`
- References target entity via `LabelAssignmentTarget`
- Enforces uniqueness constraints
- One-to-many relationship from Label to LabelAssignment

## Data Structure Relationships

### Entity Relationship Diagram

```
┌─────────────────────┐
│   ProviderInstance  │
├─────────────────────┤
│ id (PK)             │
│ display_name        │
│ provider_type       │
│ api_endpoint        │
│ metadata            │
│ active              │
│ created_at          │
│ updated_at          │
└─────────────────────┘
          │
          │ 1:N
          │
          ▼
┌─────────────────────┐     ┌─────────────────────┐
│   TagAssignment     │     │  LabelAssignment    │
├─────────────────────┤     ├─────────────────────┤
│ id (PK)             │     │ id (PK)             │
│ tag_id (FK)         │     │ label_id (FK)       │
│ target_type         │     │ target_type         │
│ target_id           │     │ target_id           │
│ metadata            │     │ metadata            │
│ assigned_at         │     │ assigned_at         │
│ assigned_by         │     │ assigned_by         │
└─────────────────────┘     └─────────────────────┘
          │                           │
          │ N:1                       │ N:1
          │                           │
          ▼                           ▼
┌─────────────────────┐     ┌─────────────────────┐
│        Tag          │     │       Label         │
├─────────────────────┤     ├─────────────────────┤
│ id (PK)             │     │ id (PK)             │
│ name                │     │ name                │
│ description         │     │ description         │
│ color               │     │ color               │
│ metadata            │     │ metadata            │
│ created_at          │     │ created_at          │
│ updated_at          │     │ updated_at          │
└─────────────────────┘     └─────────────────────┘
```

### Relationship Types

#### 1. ProviderInstance ↔ TagAssignment (1:N)
- One provider instance can have multiple tag assignments
- Each tag assignment references exactly one provider instance
- Supports bulk tag management and filtering

#### 2. ProviderInstance ↔ LabelAssignment (1:N)
- One provider instance can have multiple label assignments
- Each label assignment references exactly one provider instance
- Enables complex labeling strategies with uniqueness constraints

#### 3. Tag ↔ TagAssignment (1:N)
- One tag can be assigned to multiple targets
- Each tag assignment references exactly one tag
- Supports tag reuse across different entities

#### 4. Label ↔ LabelAssignment (1:N)
- One label can be assigned to multiple targets (unless uniqueness is enforced)
- Each label assignment references exactly one label
- Enables exclusive designations like "Primary"

#### 5. TagAssignmentTarget ↔ Entity (N:1)
- Tag assignments can target either ProviderInstance or Model
- Provides flexibility in assignment granularity
- Supports both instance-level and model-level tagging

## Validation Rules and Constraints

### ProviderInstance Validation
- `id` must be unique across all instances
- `display_name` cannot be empty
- `provider_type` must be a supported provider
- `api_endpoint` must be a valid URL
- `metadata` values must be strings

### Tag Validation
- `id` must be unique across all tags
- `name` cannot be empty and must be unique
- `color` must be a valid hex color code if provided
- `metadata` keys and values must be strings

### Label Validation
- `id` must be unique across all labels
- `name` cannot be empty and must be unique
- `color` must be a valid hex color code if provided
- `metadata` keys and values must be strings
- Uniqueness constraints enforced during assignment

### TagAssignment Validation
- `id` must be unique across all tag assignments
- `tag_id` must reference an existing tag
- `target` must reference an existing entity
- Cannot create duplicate assignments for same tag-target pair

### LabelAssignment Validation
- `id` must be unique across all label assignments
- `label_id` must reference an existing label
- `target` must reference an existing entity
- Cannot violate label uniqueness constraints
- Cannot create duplicate assignments for same label-target pair

## Usage Patterns

### Environment-Based Organization

```yaml
# Tags for environment categorization
tags:
  - id: "tag-production"
    name: "Production"
    color: "#ff0000"
    metadata:
      environment: "production"
      category: "environment"

# Provider instances with environment tags
provider_instances:
  - id: "openai-prod"
    display_name: "OpenAI Production"
    provider_type: "openai"
    tags:
      - "tag-production"
```

### Primary/Backup Designation

```yaml
# Labels for designation
labels:
  - id: "label-primary"
    name: "Primary"
    color: "#17c964"
    metadata:
      category: "designation"
      priority: "high"
    uniqueness_scope: "global"

# Provider instances with designations
provider_instances:
  - id: "openai-primary"
    display_name: "OpenAI Primary"
    labels:
      - "label-primary"
```

### Complex Metadata Relationships

```yaml
# Tags with rich metadata
tags:
  - id: "tag-cost-optimized"
    name: "Cost Optimized"
    metadata:
      category: "optimization"
      cost_tier: "low"
      performance_impact: "minimal"

# Labels with business context
labels:
  - id: "label-critical"
    name: "Critical"
    metadata:
      business_impact: "high"
      sla_required: "true"
      escalation_level: "immediate"
```

## Data Flow and Interactions

### Assignment Creation Flow

1. **Validation Phase**
   - Validate tag/label exists
   - Check target entity exists
   - Verify uniqueness constraints
   - Ensure no duplicate assignments

2. **Assignment Creation**
   - Generate unique assignment ID
   - Set assignment metadata
   - Record assignment timestamp
   - Store assigned_by information

3. **Relationship Update**
   - Link assignment to tag/label
   - Link assignment to target entity
   - Update entity metadata if needed
   - Trigger UI refresh events

### Query and Filter Patterns

#### By Tag
```rust
// Find all instances with specific tag
let instances = provider_instances
    .iter()
    .filter(|instance| {
        tag_assignments
            .iter()
            .any(|assignment| {
                assignment.tag_id == tag_id && 
                matches_target(&assignment.target, instance.id)
            })
    })
    .collect();
```

#### By Label
```rust
// Find instances with specific label
let instances = provider_instances
    .iter()
    .filter(|instance| {
        label_assignments
            .iter()
            .any(|assignment| {
                assignment.label_id == label_id && 
                matches_target(&assignment.target, instance.id)
            })
    })
    .collect();
```

#### Complex Filtering
```rust
// Find production instances that are primary
let production_primary = provider_instances
    .iter()
    .filter(|instance| {
        has_tag(instance, "production") && 
        has_label(instance, "primary")
    })
    .collect();
```

## Performance Considerations

### Indexing Strategy
- Primary indexes on all ID fields
- Composite indexes on (tag_id, target_id) for tag assignments
- Composite indexes on (label_id, target_id) for label assignments
- Indexes on frequently queried metadata fields

### Caching Layers
- Tag and label definitions cached in memory
- Assignment relationships cached for quick lookups
- Metadata fields cached for filtering operations
- UI state cached for responsive interactions

### Batch Operations
- Bulk assignment creation for migration scenarios
- Batch validation for configuration imports
- Efficient bulk queries for reporting
- Optimized bulk updates for configuration changes

## Migration and Compatibility

### Backward Compatibility
- Existing provider configurations automatically migrated
- Legacy metadata preserved during migration
- Default tags and labels created for existing instances
- Assignment relationships established based on existing data

### Data Integrity
- Referential integrity maintained across all relationships
- Cascade deletion rules for cleanup operations
- Validation checkpoints during migration
- Rollback capabilities for failed migrations

This comprehensive data structure documentation provides the foundation for understanding how the AICred tagging and labeling system organizes and manages AI provider configurations through flexible, validated relationships between core entities.
# API Reference

Authoritative reference for all public surfaces of AICred across Core (Rust), FFI/C-API, Python, Go, and the Tauri GUI interface types.

This reference aligns with the source tree:
- Core library: [core/src/lib.rs](core/src/lib.rs)
- Models: [core/src/models/](core/src/models/)
- Plugins: [core/src/plugins/mod.rs](core/src/plugins/mod.rs)
- Scanners: [core/src/scanners/mod.rs](core/src/scanners/mod.rs)
- Scanner engine: [core/src/scanner/mod.rs](core/src/scanner/mod.rs)
- FFI: [ffi/include/aicred.h](ffi/include/aicred.h), [ffi/src/lib.rs](ffi/src/lib.rs)
- Python bindings: [bindings/python/src/lib.rs](bindings/python/src/lib.rs), [bindings/python/aicred.pyi](bindings/python/aicred.pyi)
- Go bindings: [bindings/go/aicred/aicred.go](bindings/go/aicred/aicred.go)

## Core (Rust)

Crate name: `aicred-core`.

### Quick Start

```rust
use aicred_core::{scan, ScanOptions};

let result = scan(ScanOptions::default())?;
```
See [Rust example usage](core/src/lib.rs:133) in docs.

### Public API

- [scan(ScanOptions) -> Result<ScanResult>](core/src/lib.rs:153)
- [struct ScanOptions](core/src/lib.rs:58)
- [struct ScanResult](core/src/models/scan_result.rs:11)
- [struct ScanSummary](core/src/models/scan_result.rs:177)
- [struct DiscoveredKey](core/src/models/discovered_key.rs:61)
- [enum ValueType](core/src/models/discovered_key.rs:10)
- [enum Confidence](core/src/models/discovered_key.rs:37)
- [struct ConfigInstance](core/src/models/config_instance.rs:13)
- [struct Provider](core/src/models/provider.rs:33)
- [enum AuthMethod](core/src/models/provider.rs:7)
- [struct RateLimit](core/src/models/provider.rs:20)
- Plugin system
  - [trait ProviderPlugin](core/src/plugins/mod.rs:14) - **NEW**: Validates and scores keys
  - [struct PluginRegistry](core/src/plugins/mod.rs:34)
  - [fn register_builtin_plugins](core/src/lib.rs:217)
  - [struct CommonConfigPlugin](core/src/plugins/mod.rs:146)
- Scanner plugins - **NEW**: Discovery-focused plugins
  - [trait ScannerPlugin](core/src/scanners/mod.rs:18)
  - [struct ScannerRegistry](core/src/scanners/mod.rs:153)
  - [fn register_builtin_scanners](core/src/scanners/mod.rs:274)
  - [struct ScanResult](core/src/scanners/mod.rs:108)
- Scanner engine
  - [struct Scanner](core/src/scanner/mod.rs:45)
  - [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - [const DEFAULT_MAX_FILE_SIZE](core/src/scanner/mod.rs:12)

### scan(ScanOptions) -> Result<ScanResult>

- Orchestrates provider plugins and application scanners.
- Uses ScannerPlugin for key discovery and ProviderPlugin for validation.
- Applies redaction unless `include_full_values` is set.

Errors:
- [Error::ConfigError](core/src/lib.rs:127)
- [Error::PluginError](core/src/plugins/mod.rs:56)
- [Error::NotFound](core/src/scanner/mod.rs:83)
- [Error::ValidationError](core/src/scanner/mod.rs:90)
- [Error::IoError](core/src/scanner/mod.rs:177)

### ScanOptions

Fields:
- `home_dir: Option<PathBuf>` — if `None`, resolves to user home
- `include_full_values: bool` — default false; when false, full secrets are removed before serialization
- `max_file_size: usize` — default [DEFAULT_MAX_FILE_SIZE](core/src/scanner/mod.rs:12) (1MB)
- `only_providers: Option<Vec<String>>` — allowlist
- `exclude_providers: Option<Vec<String>>` — blocklist

Builders:
- [with_home_dir(PathBuf) -> Self](core/src/lib.rs:92)
- [with_full_values(bool) -> Self](core/src/lib.rs:98)
- [with_max_file_size(usize) -> Self](core/src/lib.rs:104)
- [with_only_providers(Vec<String>) -> Self](core/src/lib.rs:110)
- [with_exclude_providers(Vec<String>) -> Self](core/src/lib.rs:116)
- [get_home_dir() -> Result<PathBuf>](core/src/lib.rs:121)

### ScanResult

Serialized JSON has:
- `keys: DiscoveredKey[]`
- `config_instances: ConfigInstance[]`
- `scan_started_at: string (RFC3339 UTC)`
- `scan_completed_at: string (RFC3339 UTC)`
- `home_directory: string`
- `providers_scanned: string[]`
- `files_scanned: number`
- `directories_scanned: number`
- `metadata: Option<Map<String, serde_json::Value>>`

Helpers:
- [total_keys()](core/src/models/scan_result.rs:88)
- [total_config_instances()](core/src/models/scan_result.rs:93)
- [keys_by_provider()](core/src/models/scan_result.rs:99)
- [keys_by_type()](core/src/models/scan_result.rs:107)
- [keys_by_confidence()](core/src/models/scan_result.rs:116)
- [filter_by_provider(&str)](core/src/models/scan_result.rs:125)
- [filter_by_confidence(Confidence)](core/src/models/scan_result.rs:130)
- [filter_by_type(&ValueType)](core/src/models/scan_result.rs:138)
- [high_confidence_keys()](core/src/models/scan_result.rs:146)
- [has_keys()](core/src/models/scan_result.rs:151)
- [scan_duration()](core/src/models/scan_result.rs:156)
- [summary() -> ScanSummary](core/src/models/scan_result.rs:162)

### DiscoveredKey

Fields (serialized):
- `provider: String`
- `source: String`
- `value_type: ValueType`
- `confidence: Confidence`
- `hash: String` (SHA-256)
- `discovered_at: DateTime<Utc>`
- `line_number: Option<u32>`
- `column_number: Option<u32>`
- `metadata: Option<serde_json::Value>`

Not serialized:
- `full_value: Option<String>` is private and tagged with `#[serde(skip_serializing)]` ([field](core/src/models/discovered_key.rs:81))

Utilities:
- [new(..., full_value: String)](core/src/models/discovered_key.rs:88)
- [new_redacted(..., full_value_preview: &str)](core/src/models/discovered_key.rs:113)
- [redacted_value() -> String](core/src/models/discovered_key.rs:137) — client-side helper
- [with_full_value(include: bool) -> Self](core/src/models/discovered_key.rs:152)
- [with_position(line, col)](core/src/models/discovered_key.rs:166)
- [with_metadata(value)](core/src/models/discovered_key.rs:172)

Enums:
- [enum ValueType](core/src/models/discovered_key.rs:10) — `ApiKey`, `AccessToken`, `SecretKey`, `BearerToken`, `Custom(String)`
- [enum Confidence](core/src/models/discovered_key.rs:37) — `Low`, `Medium`, `High`, `VeryHigh`

### ConfigInstance

Fields:
- `instance_id: String`
- `app_name: String`
- `config_path: PathBuf` (serialized as path string)
- `discovered_at: DateTime<Utc>`
- `keys: Vec<DiscoveredKey>`
- `metadata: HashMap<String, String>`

See [definition](core/src/models/config_instance.rs:13).

### Provider Model

- [struct Provider](core/src/models/provider.rs:33) with `name`, `provider_type`, `base_url`, etc.
- [enum AuthMethod](core/src/models/provider.rs:7) — `ApiKey`, `OAuth`, `BearerToken`, `Custom(String)`
- [struct RateLimit](core/src/models/provider.rs:20)
### Tagging and Labeling System - **NEW**

The tagging and labeling system provides organization and categorization for provider instances and models:

#### Tag Model
- [struct Tag](core/src/models/tag.rs:10) — Non-unique identifier for categorization
  - `id: String` — Unique identifier (auto-generated)
  - `name: String` — Human-readable name
  - `description: Option<String>` — Optional description
  - `color: Option<String>` — Optional color for UI display
  - `metadata: Option<HashMap<String, String>>` — Additional metadata
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

#### Label Model
- [struct Label](core/src/models/label.rs:10) — Unique identifier for designation
  - `id: String` — Unique identifier (auto-generated)
  - `name: String` — Human-readable name
  - `description: Option<String>` — Optional description
  - `color: Option<String>` — Optional color for UI display
  - `metadata: Option<HashMap<String, String>>` — Additional metadata
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

#### Tag Assignment Model
- [struct TagAssignment](core/src/models/tag_assignment.rs:67) — Links tags to targets
  - `id: String` — Unique assignment identifier
  - `tag_id: String` — Reference to tag
  - `target: TagAssignmentTarget` — Target (instance or model)
  - `metadata: Option<HashMap<String, String>>` — Assignment metadata
  - `created_at: DateTime<Utc>` — Assignment timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [enum TagAssignmentTarget](core/src/models/tag_assignment.rs:10) — Assignment targets
  - `ProviderInstance { instance_id: String }` — Provider instance target
  - `Model { instance_id: String, model_id: String }` — Model target

#### Label Assignment Model
- [struct LabelAssignment](core/src/models/label_assignment.rs:67) — Links labels to targets with uniqueness
  - `id: String` — Unique assignment identifier
  - `label_id: String` — Reference to label
  - `target: LabelAssignmentTarget` — Target (instance or model)
  - `metadata: Option<HashMap<String, String>>` — Assignment metadata
  - `created_at: DateTime<Utc>` — Assignment timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [enum LabelAssignmentTarget](core/src/models/label_assignment.rs:10) — Assignment targets
  - `ProviderInstance { instance_id: String }` — Provider instance target
  - `Model { instance_id: String, model_id: String }` — Model target

#### Tag Methods
- [new(id, name) -> Tag](core/src/models/tag.rs:39) — Creates new tag
- [with_description(description) -> Tag](core/src/models/tag.rs:54) — Adds description
- [with_color(color) -> Tag](core/src/models/tag.rs:62) — Adds color
- [with_metadata(metadata) -> Tag](core/src/models/tag.rs:70) — Adds metadata
- [set_description(&mut self, description)](core/src/models/tag.rs:77) — Updates description
- [set_color(&mut self, color)](core/src/models/tag.rs:83) — Updates color
- [set_metadata(&mut self, metadata)](core/src/models/tag.rs:89) — Updates metadata
- [get_metadata(&self, key) -> Option<&String>](core/src/models/tag.rs:96) — Gets metadata value
- [validate(&self) -> Result<(), String>](core/src/models/tag.rs:104) — Validates tag configuration

#### Label Methods
- [new(id, name) -> Label](core/src/models/label.rs:39) — Creates new label
- [with_description(description) -> Label](core/src/models/label.rs:54) — Adds description
- [with_color(color) -> Label](core/src/models/label.rs:62) — Adds color
- [with_metadata(metadata) -> Label](core/src/models/label.rs:70) — Adds metadata
- [set_description(&mut self, description)](core/src/models/label.rs:77) — Updates description
- [set_color(&mut self, color)](core/src/models/label.rs:83) — Updates color
- [set_metadata(&mut self, metadata)](core/src/models/label.rs:89) — Updates metadata
- [get_metadata(&self, key) -> Option<&String>](core/src/models/label.rs:96) — Gets metadata value
- [validate(&self) -> Result<(), String>](core/src/models/label.rs:104) — Validates label configuration
- [can_assign_to(&self, target_id) -> bool](core/src/models/label.rs:131) — Checks assignability
- [uniqueness_scope(&self) -> &'static str](core/src/models/label.rs:138) — Gets uniqueness scope

#### Tag Assignment Methods
- [new_to_instance(id, tag_id, instance_id) -> TagAssignment](core/src/models/tag_assignment.rs:91) — Creates instance assignment
- [new_to_model(id, tag_id, instance_id, model_id) -> TagAssignment](core/src/models/tag_assignment.rs:105) — Creates model assignment
- [with_metadata(metadata) -> TagAssignment](core/src/models/tag_assignment.rs:119) — Adds assignment metadata
- [set_metadata(&mut self, metadata)](core/src/models/tag_assignment.rs:126) — Updates metadata
- [get_metadata(&self, key) -> Option<&String>](core/src/models/tag_assignment.rs:133) — Gets metadata value
- [validate(&self) -> Result<(), String>](core/src/models/tag_assignment.rs:141) — Validates assignment
- [targets_instance(&self, instance_id) -> bool](core/src/models/tag_assignment.rs:171) — Checks instance targeting
- [targets_model(&self, instance_id, model_id) -> bool](core/src/models/tag_assignment.rs:177) — Checks model targeting
- [target_description(&self) -> String](core/src/models/tag_assignment.rs:183) — Gets target description

#### Label Assignment Methods
- [new_to_instance(id, label_id, instance_id) -> LabelAssignment](core/src/models/label_assignment.rs:91) — Creates instance assignment
- [new_to_model(id, label_id, instance_id, model_id) -> LabelAssignment](core/src/models/label_assignment.rs:105) — Creates model assignment
- [with_metadata(metadata) -> LabelAssignment](core/src/models/label_assignment.rs:119) — Adds assignment metadata
- [set_metadata(&mut self, metadata)](core/src/models/label_assignment.rs:126) — Updates metadata
- [get_metadata(&self, key) -> Option<&String>](core/src/models/label_assignment.rs:133) — Gets metadata value
- [validate(&self) -> Result<(), String>](core/src/models/label_assignment.rs:141) — Validates assignment
- [targets_instance(&self, instance_id) -> bool](core/src/models/label_assignment.rs:171) — Checks instance targeting
- [targets_model(&self, instance_id, model_id) -> bool](core/src/models/label_assignment.rs:177) — Checks model targeting
- [target_description(&self) -> String](core/src/models/label_assignment.rs:184) — Gets target description
- [uniqueness_key(&self) -> &str](core/src/models/label_assignment.rs:190) — Gets uniqueness key
- [conflicts_with(&self, other) -> bool](core/src/models/label_assignment.rs:197) — Checks for conflicts

#### CLI Commands for Tags and Labels
- [tags list](cli/src/commands/tags.rs:87) — List all configured tags
- [tags add](cli/src/commands/tags.rs:126) — Add a new tag
- [tags update](cli/src/commands/tags.rs:227) — Update existing tag
- [tags remove](cli/src/commands/tags.rs:165) — Remove a tag
- [tags assign](cli/src/commands/tags.rs:269) — Assign tag to instance/model
- [tags unassign](cli/src/commands/tags.rs:352) — Unassign tag from target
- [labels list](cli/src/commands/labels.rs:87) — List all configured labels
- [labels add](cli/src/commands/labels.rs:136) — Add a new label
- [labels update](cli/src/commands/labels.rs:236) — Update existing label
- [labels remove](cli/src/commands/labels.rs:184) — Remove a label
- [labels assign](cli/src/commands/labels.rs:278) — Assign label to instance/model
- [labels unassign](cli/src/commands/labels.rs:357) — Unassign label from target

### Provider Instance Model - **REFACTORED**

The [`ProviderInstance`](core/src/models/provider_instance.rs:25) model manages individual provider configurations with a simplified single-key approach:

#### ProviderInstance Structure

- [struct ProviderInstance](core/src/models/provider_instance.rs:25) — Provider instance configuration
  - `id: String` — Unique identifier for this instance
  - `display_name: String` — Human-readable display name
  - `provider_type: String` — Provider type (e.g., "openai", "anthropic", "groq")
  - `base_url: String` — Base URL for API requests
  - `api_key: Option<String>` — **Single API key** (simplified from multi-key model)
  - `models: Vec<Model>` — Instance-specific model configurations
  - `metadata: Option<HashMap<String, String>>` — Additional metadata (preserves key metadata during conversions)
  - `active: bool` — Whether this instance is active
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

#### ProviderInstance Methods

Construction:
- [new(id, display_name, provider_type, base_url) -> Self](core/src/models/provider_instance.rs:64) — Creates new instance
- [new_with_cleaned_metadata(...) -> Self](core/src/models/provider_instance.rs:83) — Creates instance with cleaned metadata (removes redundant fields)

API Key Management:
- [set_api_key(&mut self, api_key: String)](core/src/models/provider_instance.rs:137) — Sets the API key
- [get_api_key(&self) -> Option<&String>](core/src/models/provider_instance.rs:144) — Gets API key reference
- [has_api_key(&self) -> bool](core/src/models/provider_instance.rs:150) — Checks if API key is present (including empty strings)
- [has_non_empty_api_key(&self) -> bool](core/src/models/provider_instance.rs:156) — Checks if non-empty API key is present

Model Management:
- [add_model(&mut self, model: Model)](core/src/models/provider_instance.rs:111) — Adds a model
- [add_models(&mut self, models: Vec<Model>)](core/src/models/provider_instance.rs:117) — Adds multiple models
- [model_count(&self) -> usize](core/src/models/provider_instance.rs:163) — Gets model count
- [get_model(&self, model_id: &str) -> Option<&Model>](core/src/models/provider_instance.rs:170) — Gets model by ID
- [active_models(&self) -> Vec<&Model>](core/src/models/provider_instance.rs:203) — Gets active models

Configuration:
- [with_metadata(self, metadata: HashMap<String, String>) -> Self](core/src/models/provider_instance.rs:124) — Sets metadata
- [with_active(self, active: bool) -> Self](core/src/models/provider_instance.rs:131) — Sets active status
- [validate(&self) -> Result<(), String>](core/src/models/provider_instance.rs:178) — Validates configuration
- [key_name(&self) -> &str](core/src/models/provider_instance.rs:215) — Gets the instance ID for CLI usage

#### Metadata-Preserving Conversions

The model includes bidirectional conversions with [`ProviderConfig`](core/src/models/provider_config.rs:11) that preserve key metadata:

**From ProviderConfig to ProviderInstance** ([impl](core/src/models/provider_instance.rs:227)):
- Extracts first valid key value to `api_key`
- Preserves key metadata in instance `metadata` HashMap:
  - `environment` — Environment type
  - `confidence` — Confidence level
  - `validation_status` — Validation state
  - `discovered_at` — RFC3339 timestamp
  - `source` — Source file path
  - `line_number` — Line number (if available)
  - `key_metadata` — Additional JSON metadata (if available)

**From ProviderInstance to ProviderConfig** ([impl](core/src/models/provider_instance.rs:293)):
- Wraps `api_key` in a [`ProviderKey`](core/src/models/provider_key.rs:11) with ID "default"
- Restores all preserved metadata from instance `metadata`
- Uses safe defaults for missing or malformed metadata
- Logs parsing errors without failing conversion

### Provider Configuration (Multi-Key)

The provider configuration supports multiple API keys per provider:

- [struct ProviderConfig](core/src/models/provider_config.rs:11) — Main configuration structure
  - `keys: Vec<ProviderKey>` — Multiple keys instead of single `api_key`
  - `models: Vec<String>` — Available models
  - `metadata: Option<HashMap<String, serde_yaml::Value>>` — Additional metadata
  - `version: String` — Provider version
  - `schema_version: String` — Schema version ("3.0" for multi-key)
  - `created_at: DateTime<Utc>` — Creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [struct ProviderKey](core/src/models/provider_key.rs:11) — Individual key management
  - `id: String` — Unique identifier (e.g., "default", "staging", "production")
  - `value: Option<String>` — Actual key value (null in config files, populated at scan time)
  - `discovered_at: DateTime<Utc>` — When key was found
  - `source: String` — File path where key was discovered
  - `line_number: Option<u32>` — Line number in source file
  - `confidence: Confidence` — Detection confidence enum with variants: Low, Medium, High, VeryHigh
  - `environment: Environment` — Environment context (dev/staging/prod)
  - `last_validated: Option<DateTime<Utc>>` — Last validation timestamp
  - `validation_status: ValidationStatus` — Current validation state
  - `metadata: Option<serde_json::Value>` — Additional key-specific metadata
  - `created_at: DateTime<Utc>` — Key creation timestamp
  - `updated_at: DateTime<Utc>` — Last update timestamp

- [enum ValidationStatus](core/src/models/provider_key.rs:45) — `Unknown`, `Valid`, `Invalid`, `Expired`
- [enum Environment](core/src/models/provider_key.rs:32) — `Development`, `Staging`, `Production`

ProviderConfig methods:
- [new(version: String) -> Self](core/src/models/provider_config.rs:25)
- [add_key(key: ProviderKey) -> Result<()>](core/src/models/provider_config.rs:30)
- [key_count() -> usize](core/src/models/provider_config.rs:38)
- [valid_key_count() -> usize](core/src/models/provider_config.rs:42)
- [keys_by_environment(env: Environment) -> Vec<&ProviderKey>](core/src/models/provider_config.rs:46)
- [get_key(id: &str) -> Option<&ProviderKey>](core/src/models/provider_config.rs:50)
- [from_old_format(...) -> Self](core/src/models/provider_config.rs:54) — Backward compatibility

**Note:** [`ProviderInstance`](core/src/models/provider_instance.rs:25) uses a simplified single-key model (`api_key: Option<String>`), while [`ProviderConfig`](core/src/models/provider_config.rs:11) maintains the multi-key model (`keys: Vec<ProviderKey>`). Conversions between these models preserve metadata. See the [Migration Guide](docs/migration-guide.md) for details on the refactoring.

### Plugin System

#### Provider Plugins (Validation) - **NEW ARCHITECTURE**

Provider plugins now focus on validating and scoring discovered keys:

- [trait ProviderPlugin](core/src/plugins/mod.rs:14)
  - `name(&self) -> &str` - Plugin name
  - `confidence_score(&self, key: &str) -> f32` - Score key validity (0.0-1.0)
  - `can_handle_file(&self, path: &Path) -> bool` - Check if plugin handles file
  - `provider_type(&self) -> &str` - Provider type name
- [struct PluginRegistry](core/src/plugins/mod.rs:34)
  - `register(Arc<dyn ProviderPlugin>)`
  - `get(name) -> Option<Arc<dyn ProviderPlugin>>`
  - `list() -> Vec<String>`
  - `get_plugins_for_file(&Path) -> Vec<Arc<dyn ProviderPlugin>>`

#### Scanner Plugins (Discovery) - **NEW ARCHITECTURE**

Scanner plugins handle discovery of keys and configurations:

- [trait ScannerPlugin](core/src/scanners/mod.rs:18) - **NEW**
  - `name(&self) -> &str` - Scanner name
  - `app_name(&self) -> &str` - Application name
  - `scan_paths(&self, home_dir: &Path) -> Vec<PathBuf>` - Paths to scan
  - `parse_config(&self, path: &Path, content: &str) -> Result<scanners::ScanResult>` - Parse config
  - `can_handle_file(&self, path: &Path) -> bool` - Check if scanner handles file
  - `supports_provider_scanning(&self) -> bool` - Whether scanner finds provider keys
  - `supported_providers(&self) -> Vec<String>` - Providers this scanner can find
  - `scan_provider_configs(&self, home_dir: &Path) -> Result<Vec<PathBuf>>` - Find provider configs
  - `scan_instances(&self, home_dir: &Path) -> Result<Vec<ConfigInstance>>` - Find app instances
- [struct ScannerRegistry](core/src/scanners/mod.rs:153) - **NEW**
  - `register(Arc<dyn ScannerPlugin>)`
  - `get(name) -> Option<Arc<dyn ScannerPlugin>>`
  - `list() -> Vec<String>`
  - `get_scanners_for_file(&Path) -> Vec<Arc<dyn ScannerPlugin>>`

### Scanner Engine

- [struct Scanner](core/src/scanner/mod.rs:45)
  - `scan(&self, home_dir: &Path) -> Result<ScanResult>` - Main scanning method
- [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - `max_file_size`, `follow_symlinks`, `include_extensions`, `exclude_extensions`, `exclude_files`, `scan_hidden`

## FFI / C-API

Header: [ffi/include/aicred.h](ffi/include/aicred.h)

Functions:
- `char* aicred_scan(const char* home_path, const char* options_json);` ([decl](ffi/include/aicred.h:34), [impl](ffi/src/lib.rs:101))
- `void aicred_free(char* ptr);` ([decl](ffi/include/aicred.h:43), [impl](ffi/src/lib.rs:179))
- `const char* aicred_version(void);` ([decl](ffi/include/aicred.h:50), [impl](ffi/src/lib.rs:192))
- `const char* aicred_last_error(void);` ([decl](ffi/include/aicred.h:58), [impl](ffi/src/lib.rs:206))

Options JSON example (UTF-8 C string):
```json
{
  "include_full_values": false,
  "max_file_size": 1048576,
  "only_providers": ["openai", "anthropic"],
  "exclude_providers": []
}
```

Return:
- On success: UTF-8 JSON string of Core [ScanResult](core/src/models/scan_result.rs:11) (caller must free with `aicred_free`)
- On failure: `NULL`, and `aicred_last_error()` returns the message (thread-local)

Memory:
- Caller must free any pointer returned by `aicred_scan` using `aicred_free`
- `aicred_version` and `aicred_last_error` return pointers valid until the next FFI call

## Python API

Module name: `aicred`.

Definitions:
- [scan(...)](bindings/python/aicred.pyi:3)
  - `home_dir: Optional[str] = None`
  - `include_full_values: bool = False`
  - `max_file_size: int = 1048576`
  - `only_providers: Optional[List[str]] = None`
  - `exclude_providers: Optional[List[str]] = None`
  - Returns: `Dict[str, Any]` matching Core JSON of [ScanResult](core/src/models/scan_result.rs:11)
- [version() -> str](bindings/python/aicred.pyi:36)
- [list_providers() -> List[str]](bindings/python/aicred.pyi:40) - **UPDATED**: Lists provider plugins
- [list_scanners() -> List[str]](bindings/python/aicred.pyi:44) - **NEW**: Lists scanner plugins

Implementation detail: Python uses Core [scan(ScanOptions)](bindings/python/src/lib.rs:23) and `serde_json` round-trip to construct a Python dict.

## Go API

Package: `github.com/robottwo/aicred/bindings/go`.

Types:
- [type ScanOptions struct](bindings/go/aicred/aicred.go:18)
  - `HomeDir string`
  - `IncludeFullValues bool`
  - `MaxFileSize int`
  - `OnlyProviders []string`
  - `ExcludeProviders []string`
- [type DiscoveredKey struct](bindings/go/aicred/aicred.go:27)
  - `Provider, Source, ValueType, Hash string`
  - `Confidence string` (enum values: "Low", "Medium", "High", "VeryHigh")
  - `Redacted string` (may be empty; not populated by core unless post-processed)
  - `Locked bool` (not set by core; reserved)
- [type ConfigInstance struct](bindings/go/aicred/aicred.go:39)
- [type ScanResult struct](bindings/go/aicred/aicred.go:48)

Functions:
- [func Scan(options ScanOptions) (*ScanResult, error)](bindings/go/aicred/aicred.go:57)
- [func Version() string](bindings/go/aicred/aicred.go:101)
- [func ListProviders() []string](bindings/go/aicred/aicred.go:107) - **UPDATED**: Lists provider plugins
- [func ListScanners() []string](bindings/go/aicred/aicred.go:119) - **NEW**: Lists scanner plugins

Note: Go uses the FFI; JSON shape mirrors Core. Optional fields like `Redacted` may be blank unless your application computes redaction strings.

## CLI

Binary: `aicred`.

Commands:
- [scan](cli/src/main.rs:22) — see handler [handle_scan(...)](cli/src/commands/scan.rs:8)
- [providers](cli/src/main.rs:57) — see [handle_providers(...)](cli/src/commands/providers.rs:4) - **UPDATED**: Lists both providers and scanners
- [instances](cli/src/main.rs:64) — see [handle_instances(...)](cli/src/commands/instances.rs:4) - **NEW**: Manages provider instances
- [version](cli/src/main.rs:71)

Output formats:
- JSON: [output_json(&ScanResult)](cli/src/output/json.rs:4)
- NDJSON (keys and instances): [output_ndjson(&ScanResult)](cli/src/output/ndjson.rs:4)
- Summary: [output_summary(&ScanResult)](cli/src/output/summary.rs:4)
- Table: [output_table(&ScanResult)](cli/src/output/table.rs:1)

Flags for `scan`:
- `--home` (directory)
- `--format` (`table` | `json` | `ndjson` | `summary`)
- `--include-values` (boolean)
- `--only`, `--exclude` (comma-separated lists) - **UPDATED**: Works with both provider and scanner names
- `--max-bytes-per-file` (usize)
- `--dry-run`
- `--audit-log PATH`

## Data Structures (JSON Schemas)

Informal JSON shapes (actual serialization driven by `serde`):

### DiscoveredKey:
```json
{
  "provider": "openai",
  "source": "/home/user/.env",
  "value_type": "ApiKey",
  "confidence": "High",
  "hash": "hex-64",
  "discovered_at": "2025-01-20T10:30:00Z",
  "line_number": 10,
  "column_number": 5,
  "metadata": {}
}
```

### ConfigInstance:
```json
{
  "instance_id": "instance-1",
  "app_name": "roo-code",
  "config_path": "/home/user/.config/roo-code/config.json",
  "discovered_at": "2025-01-20T10:30:00Z",
  "keys": [],
  "metadata": { "version": "1.2.3" }
}
```

### ScanResult:
```json
{
  "keys": [],
  "config_instances": [],
  "scan_started_at": "2025-01-20T10:30:00Z",
  "scan_completed_at": "2025-01-20T10:30:01Z",
  "home_directory": "/home/user",
  "providers_scanned": ["openai", "anthropic"],
  "files_scanned": 100,
  "directories_scanned": 20,
  "metadata": null
}
```

## Error Handling

The core uses a unified error type `aicred_core::error::Error` (variants used across modules):
- `ConfigError(String)` — e.g., cannot determine home dir ([usage](core/src/lib.rs:127))
- `PluginError(String)` — plugin registration/operation ([usage](core/src/plugins/mod.rs:56))
- `NotFound(String)` — invalid paths ([usage](core/src/scanner/mod.rs:83))
- `ValidationError(String)` — wrong types/expectations ([usage](core/src/scanner/mod.rs:90))
- `IoError(std::io::Error)` — filesystem operations ([usage](core/src/scanner/mod.rs:177))

FFI:
- Functions return `NULL` on error; the message is provided by [aicred_last_error()](ffi/include/aicred.h:58).

CLI:
- Exit codes:
  - `0`: keys or config instances found
  - `1`: none found
  - `2`: error occurred (set via `anyhow::bail!` paths)

## Configuration Options

ScanOptions (Core):
- See [struct ScanOptions](core/src/lib.rs:58)

ScannerConfig (Core scanner engine):
- See [struct ScannerConfig](core/src/scanner/mod.rs:16)
  - `max_file_size`, `follow_symlinks`, `include_extensions`, `exclude_extensions`, `exclude_files`, `scan_hidden`

## Redaction Model

- Full values are stored only transiently inside `DiscoveredKey` and are skipped in JSON output (security-by-default).
- Clients should use:
  - `hash` for deduplication
  - A generic preview like `"****"` for display
  - If and only if `include_full_values` was used, applications may compute previews locally (e.g., last 4 chars).

## Architecture Changes - **IMPORTANT**

### New Plugin Architecture

The architecture has been redesigned to separate concerns:

**ScannerPlugin** (Discovery):
- Handles discovery of API keys and configuration files
- Can discover keys for multiple providers
- Focuses on file parsing and key extraction
- Methods: `name()`, `app_name()`, `scan_paths()`, `parse_config()`, `can_handle_file()`
- Optional: `supports_provider_scanning()`, `supported_providers()`, `scan_provider_configs()`, `scan_instances()`

**ProviderPlugin** (Validation):
- Validates and scores discovered keys
- Assigns confidence scores to keys
- Focuses on key validation and pattern matching
- Methods: `name()`, `confidence_score()`, `can_handle_file()`, `provider_type()`

### Migration Notes

**Old Architecture**: ProviderPlugin handled both discovery and validation
**New Architecture**: Separate ScannerPlugin (discovery) and ProviderPlugin (validation)

This change provides:
- Clear separation of concerns
- More flexible plugin system
- Better support for application-specific scanning
- Improved key discovery across multiple providers

## Versioning

- Library versions: [ffi/src/lib.rs](ffi/src/lib.rs:27) exposes `aicred_version()`
- CLI `version` subcommand prints package version and core version ([handler](cli/src/main.rs:103))
