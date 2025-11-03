# Tagging and Labeling System Guide

The AICred tagging and labeling system provides a powerful way to organize, categorize, and manage your GenAI provider instances and models. This guide covers all aspects of the system, from basic concepts to advanced usage patterns.

## Overview

The system provides two types of identifiers:

- **Tags**: Non-unique identifiers that can be applied to multiple targets
- **Labels**: Unique identifiers that can only be assigned to one target at a time

## Core Concepts

### Tags

Tags are designed for categorization and organization. They are:

- **Non-unique**: Multiple targets can have the same tag
- **Flexible**: Can be applied to provider instances or specific models
- **Descriptive**: Include name, description, color, and metadata
- **Persistent**: Stored in configuration files and survive system restarts

### Labels

Labels are designed for unique identification and designation. They are:

- **Unique**: Only one target can have a specific label at a time
- **Exclusive**: Enforce uniqueness constraints across all targets
- **Definitive**: Used to mark primary, backup, deprecated, etc.
- **Safe**: Prevent accidental duplication of important designations

## Data Models

### Tag Structure

```rust
pub struct Tag {
    pub id: String,                    // Unique identifier (auto-generated)
    pub name: String,                  // Human-readable name
    pub description: Option<String>,   // Optional description
    pub color: Option<String>,         // Optional color for UI display
    pub metadata: Option<HashMap<String, String>>, // Additional metadata
    pub created_at: DateTime<Utc>,     // Creation timestamp
    pub updated_at: DateTime<Utc>,     // Last update timestamp
}
```

### Label Structure

```rust
pub struct Label {
    pub id: String,                    // Unique identifier (auto-generated)
    pub name: String,                  // Human-readable name
    pub description: Option<String>,   // Optional description
    pub color: Option<String>,         // Optional color for UI display
    pub metadata: Option<HashMap<String, String>>, // Additional metadata
    pub created_at: DateTime<Utc>,     // Creation timestamp
    pub updated_at: DateTime<Utc>,     // Last update timestamp
}
```

### Assignment Structures

#### Tag Assignment

```rust
pub struct TagAssignment {
    pub id: String,                    // Unique assignment ID
    pub tag_id: String,               // Reference to tag
    pub target: TagAssignmentTarget,  // Target (instance or model)
    pub metadata: Option<HashMap<String, String>>, // Assignment metadata
    pub created_at: DateTime<Utc>,    // Assignment timestamp
    pub updated_at: DateTime<Utc>,    // Last update timestamp
}

pub enum TagAssignmentTarget {
    ProviderInstance { instance_id: String },
    Model { instance_id: String, model_id: String },
}
```

#### Label Assignment

```rust
pub struct LabelAssignment {
    pub id: String,                    // Unique assignment ID
    pub label_id: String,             // Reference to label
    pub target: LabelAssignmentTarget, // Target (instance or model)
    pub metadata: Option<HashMap<String, String>>, // Assignment metadata
    pub created_at: DateTime<Utc>,    // Assignment timestamp
    pub updated_at: DateTime<Utc>,    // Last update timestamp
}

pub enum LabelAssignmentTarget {
    ProviderInstance { instance_id: String },
    Model { instance_id: String, model_id: String },
}
```

## Configuration Storage

Tags and labels are stored in YAML files in the AICred configuration directory:

```
~/.config/aicred/
├── tags.yaml              # Tag definitions
├── tag_assignments.yaml   # Tag assignments
├── labels.yaml            # Label definitions
└── label_assignments.yaml # Label assignments
```

### Example Tag Configuration (tags.yaml)

```yaml
- id: "tag-abc123"
  name: "Production"
  description: "Production environment instances"
  color: "#ff0000"
  metadata:
    environment: "prod"
    team: "platform"
  created_at: "2025-01-20T10:30:00Z"
  updated_at: "2025-01-20T10:30:00Z"
- id: "tag-def456"
  name: "Development"
  description: "Development environment instances"
  color: "#00ff00"
  metadata:
    environment: "dev"
    team: "developers"
  created_at: "2025-01-20T10:30:00Z"
  updated_at: "2025-01-20T10:30:00Z"
```

### Example Tag Assignment Configuration (tag_assignments.yaml)

```yaml
- id: "assignment-tag-abc123-openai-prod"
  tag_id: "tag-abc123"
  target:
    provider_instance:
      instance_id: "openai-prod"
  metadata:
    assigned_by: "admin"
    reason: "production deployment"
  created_at: "2025-01-20T10:30:00Z"
  updated_at: "2025-01-20T10:30:00Z"
- id: "assignment-tag-abc123-gpt4-model"
  tag_id: "tag-abc123"
  target:
    model:
      instance_id: "openai-prod"
      model_id: "gpt-4"
  metadata:
    assigned_by: "admin"
    reason: "production model"
  created_at: "2025-01-20T10:30:00Z"
  updated_at: "2025-01-20T10:30:00Z"
```

## CLI Usage

### Tag Management Commands

#### List Tags
```bash
aicred tags list
```

Shows all configured tags with their details, colors, and assignment counts.

#### Add Tag
```bash
aicred tags add --name "Production" --color "#ff0000" --description "Production environment"
```

Creates a new tag with optional color and description.

#### Update Tag
```bash
aicred tags update --name "Production" --color "#00ff00" --description "Updated description"
```

Updates an existing tag's properties.

#### Remove Tag
```bash
# Remove with confirmation if assigned
aicred tags remove --name "Production"

# Force remove (removes all assignments)
aicred tags remove --name "Production" --force
```

#### Assign Tag to Instance
```bash
aicred tags assign --name "Production" --instance-id my-openai
```

#### Assign Tag to Model
```bash
aicred tags assign --name "GPT-4" --instance-id my-openai --model-id gpt-4
```

#### Unassign Tag
```bash
aicred tags unassign --name "Production" --instance-id my-openai
aicred tags unassign --name "GPT-4" --instance-id my-openai --model-id gpt-4
```

### Label Management Commands

#### List Labels
```bash
aicred labels list
```

Shows all configured labels with their assignment status.

#### Add Label
```bash
aicred labels add --name "Primary" --color "#17c964" --description "Primary provider instance"
```

Creates a new label with uniqueness constraints.

#### Update Label
```bash
aicred labels update --name "Primary" --color "#00ff00"
```

#### Remove Label
```bash
# Remove (must be unassigned first)
aicred labels remove --name "Primary"

# Force remove (removes assignment if exists)
aicred labels remove --name "Primary" --force
```

#### Assign Label
```bash
# Assign to instance
aicred labels assign --name "Primary" --instance-id my-openai

# Assign to model
aicred labels assign --name "Fast-Model" --instance-id my-openai --model-id gpt-3.5-turbo
```

#### Unassign Label
```bash
aicred labels unassign --name "Primary" --instance-id my-openai
```

## Usage Patterns and Best Practices

### Environment-Based Organization

Use tags to categorize instances by environment:

```bash
# Create environment tags
aicred tags add --name "Production" --color "#ff0000"
aicred tags add --name "Staging" --color "#ffa500"
aicred tags add --name "Development" --color "#00ff00"
aicred tags add --name "Testing" --color "#0000ff"

# Assign environment tags
aicred tags assign --name "Production" --instance-id openai-prod
aicred tags assign --name "Staging" --instance-id openai-staging
aicred tags assign --name "Development" --instance-id openai-dev
aicred tags assign --name "Testing" --instance-id openai-test
```

### Team-Based Organization

Use tags to categorize by team or ownership:

```bash
# Create team tags
aicred tags add --name "Team-Alpha" --color "#6c8cff"
aicred tags add --name "Team-Beta" --color "#17c964"
aicred tags add --name "Team-Gamma" --color "#f5a524"

# Assign team tags
aicred tags assign --name "Team-Alpha" --instance-id openai-prod
aicred tags assign --name "Team-Beta" --instance-id anthropic-prod
```

### AI Development Workflow Labels

Use labels to organize instances by development workflow:

```bash
# Create AI development workflow labels
aicred labels add --name "Coding" --color "#17c964" --description "General coding and development tasks"
aicred labels add --name "Fast Coding" --color "#f5a524" --description "Quick prototyping and rapid development"
aicred labels add --name "Planning" --color "#9b6cff" --description "Strategic planning and architecture work"

# Assign workflow labels
aicred labels assign --name "Coding" --instance-id openai-dev
aicred labels assign --name "Fast Coding" --instance-id openai-prototype
aicred labels assign --name "Planning" --instance-id openai-architecture
```

### Primary/Backup Designation (Legacy)

For backward compatibility, you can still use primary/backup designation:

```bash
# Create designation labels
aicred labels add --name "Primary" --color "#17c964"
aicred labels add --name "Backup" --color "#f5a524"
aicred labels add --name "Deprecated" --color "#ff6b6b"

# Assign primary label (only one can have it)
aicred labels assign --name "Primary" --instance-id openai-prod

# Assign backup label
aicred labels assign --name "Backup" --instance-id openai-backup

# When promoting backup to primary, unassign first
aicred labels unassign --name "Primary" --instance-id openai-prod
aicred labels assign --name "Primary" --instance-id openai-backup
```

### Model-Specific Tagging

Tag specific models for organization:

```bash
# Create model tags
aicred tags add --name "GPT-4" --color "#9b6cff"
aicred tags add --name "GPT-3.5" --color "#00d4aa"
aicred tags add --name "Claude-3" --color "#ff8c42"

# Assign to specific models
aicred tags assign --name "GPT-4" --instance-id openai-prod --model-id gpt-4
aicred tags assign --name "GPT-3.5" --instance-id openai-prod --model-id gpt-3.5-turbo
aicred tags assign --name "Claude-3" --instance-id anthropic-prod --model-id claude-3-opus
```

### Metadata Usage

Use metadata for additional organization:

```bash
# Add tags with metadata
aicred tags add --name "High-Priority" --description "High priority workloads"
# Metadata can be added via configuration file editing
```

Edit the tags.yaml file to add metadata:

```yaml
- id: "tag-high-priority"
  name: "High-Priority"
  description: "High priority workloads"
  color: "#ff0000"
  metadata:
    priority: "high"
    sla: "99.9"
    team: "critical-systems"
  created_at: "2025-01-20T10:30:00Z"
  updated_at: "2025-01-20T10:30:00Z"
```

## GUI Usage

The Tauri GUI provides visual management of tags and labels:

### Tag Management Interface

1. **Tag List**: View all configured tags with colors and assignment counts
2. **Add/Edit Form**: Create or modify tags with color picker
3. **Assignment Management**: Assign/unassign tags to instances and models
4. **Visual Indicators**: Color-coded tags for easy identification

### Label Management Interface

1. **Label List**: View all configured labels with assignment status
2. **Add/Edit Form**: Create or modify labels with color picker
3. **Assignment Status**: Visual indicators for assigned/unassigned labels
4. **Uniqueness Enforcement**: Clear messaging when labels are already assigned

### Assignment Modal

The GUI includes an assignment modal for managing tag/label assignments:

1. **Target Selection**: Choose instance or model targets
2. **Assignment Preview**: See current assignments before making changes
3. **Bulk Operations**: Assign multiple tags/labels at once
4. **Validation**: Real-time validation of assignment constraints

## API Integration

### Core Library Usage

```rust
use aicred_core::models::{Tag, TagAssignment, Label, LabelAssignment};

// Load tags
let tags = load_tags()?;

// Create new tag
let tag = Tag::new("tag-production".to_string(), "Production".to_string())
    .with_description("Production environment".to_string())
    .with_color("#ff0000".to_string());

// Assign tag to instance
let assignment = TagAssignment::new_to_instance(
    "assignment-1".to_string(),
    tag.id.clone(),
    "openai-prod".to_string(),
);
```

### Python Bindings

```python
import aicred

# Load tags
tags = aicred.list_tags()

# Create tag (via CLI or direct file manipulation)
# Tags are managed through CLI commands in Python bindings
```

### Go Bindings

```go
package main

import (
    "fmt"
    "github.com/robottwo/aicred/bindings/go"
)

func main() {
    // Tags and labels are managed through CLI commands
    // Use the CLI for tag/label management in Go applications
}
```

## Validation and Constraints

### Tag Validation

- Tag ID and name cannot be empty
- Tag name cannot exceed 100 characters
- Color cannot exceed 20 characters
- Description cannot exceed 500 characters
- Metadata values must be strings

### Label Validation

- Label ID and name cannot be empty
- Label name cannot exceed 100 characters
- Color cannot exceed 20 characters
- Description cannot exceed 500 characters
- Labels must be unique across all targets

### Assignment Validation

- Assignment ID cannot be empty
- Tag/Label ID must reference existing tag/label
- Instance ID cannot be empty for instance assignments
- Model ID cannot be empty for model assignments
- Target must exist (instance or model)

## Error Handling

### Common Error Scenarios

1. **Tag/Label Not Found**
   ```
   Error: Tag with name 'Production' not found
   ```
   Solution: Check tag name with `aicred tags list`

2. **Label Already Assigned**
   ```
   Error: Label 'Primary' is already assigned to openai-prod
   ```
   Solution: Unassign the label first with `aicred labels unassign`

3. **Cannot Delete Assigned Label**
   ```
   Error: Cannot delete label 'Primary' because it is currently assigned
   ```
   Solution: Unassign the label before deletion

4. **Invalid Target**
   ```
   Error: Instance ID is required when specifying a model
   ```
   Solution: Provide both instance ID and model ID for model assignments

### Recovery Procedures

1. **Corrupted Configuration Files**
   - Backup files are automatically created
   - Restore from backup or recreate configurations
   - Use `aicred tags list` to verify data integrity

2. **Assignment Conflicts**
   - Use `aicred tags list` and `aicred labels list` to check current state
   - Unassign conflicting tags/labels
   - Reassign as needed

3. **Validation Errors**
   - Check configuration file syntax
   - Ensure all required fields are present
   - Verify ID uniqueness and referential integrity

## Migration and Backup

### Configuration Backup

Tags and labels are stored in YAML files that can be backed up:

```bash
# Backup configuration
cp ~/.config/aicred/tags.yaml ~/backup/tags.yaml
cp ~/.config/aicred/tag_assignments.yaml ~/backup/tag_assignments.yaml
cp ~/.config/aicred/labels.yaml ~/backup/labels.yaml
cp ~/.config/aicred/label_assignments.yaml ~/backup/label_assignments.yaml
```

### Configuration Restore

```bash
# Restore configuration
cp ~/backup/tags.yaml ~/.config/aicred/tags.yaml
cp ~/backup/tag_assignments.yaml ~/.config/aicred/tag_assignments.yaml
cp ~/backup/labels.yaml ~/.config/aicred/labels.yaml
cp ~/backup/label_assignments.yaml ~/.config/aicred/label_assignments.yaml
```

### Migration from Previous Versions

The tagging system is new in version 0.2.0. Existing configurations will continue to work without tags/labels. To add tagging to existing setups:

1. Install the new version
2. Start using tag/label commands
3. Organize existing instances with tags and labels

## Performance Considerations

### Storage Efficiency

- Tags and labels are stored in compact YAML files
- Assignment tracking is optimized for quick lookups
- Metadata is optional and only stored when needed

### Query Performance

- Tag/label queries are fast due to simple data structures
- Assignment lookups are optimized by target type
- No database overhead for simple use cases

### Scalability

- System handles hundreds of tags/labels efficiently
- Assignment operations are O(1) for most cases
- Memory usage scales linearly with number of tags/labels

## Security Considerations

### Data Protection

- Tag/label configurations contain no sensitive data
- Metadata should not include secrets or API keys
- Configuration files should have appropriate permissions

### Access Control

- Tag/label management requires local file system access
- No remote access to tag/label configurations
- Users can only manage tags/labels on their local system

### Audit Trail

- All tag/label operations are logged with timestamps
- Assignment changes are tracked with creation/update times
- Metadata can include audit information

## Troubleshooting

### Common Issues

1. **Tags/Labels Not Appearing**
   - Check configuration file permissions
   - Verify YAML syntax
   - Restart application

2. **Assignment Failures**
   - Ensure target instance/model exists
   - Check for naming conflicts
   - Verify uniqueness constraints for labels

3. **Performance Issues**
   - Limit number of tags/labels per target
   - Use metadata sparingly
   - Clean up unused tags/labels

### Debug Commands

```bash
# List all tags with details
aicred tags list --verbose

# List all labels with assignment status
aicred labels list --verbose

# Check configuration file syntax
cat ~/.config/aicred/tags.yaml | yaml-validator

# Verify assignment integrity
aicred instances list --show-tags --show-labels
```

## Future Enhancements

### Planned Features

1. **Tag/Label Categories**: Hierarchical organization
2. **Bulk Operations**: Assign/unassign multiple items at once
3. **Import/Export**: JSON/YAML import/export functionality
4. **Search and Filter**: Find tags/labels by various criteria
5. **Templates**: Predefined tag/label sets for common patterns

### Extension Points

1. **Custom Metadata Types**: Support for structured metadata
2. **Assignment Rules**: Automatic assignment based on rules
3. **Integration APIs**: External system integration
4. **Reporting**: Tag/label usage analytics

## Conclusion

The tagging and labeling system provides a flexible, powerful way to organize and manage your GenAI provider instances and models. By understanding the concepts, following best practices, and using the appropriate tools, you can create a well-organized, maintainable configuration that scales with your needs.

For additional help, refer to:
- [User Guide](user-guide.md) for basic usage
- [API Reference](api-reference.md) for technical details
- [Migration Guide](migration-guide.md) for upgrade information
- [Architecture Documentation](architecture.md) for system design