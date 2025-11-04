# Migration Guide

This guide provides instructions for migrating to AICred version 0.2.0, which introduces the comprehensive tagging and labeling system. This version maintains full backward compatibility while adding powerful new organization features.

## Overview

Version 0.2.0 introduces:

- **Tagging System**: Non-unique identifiers for categorization
- **Labeling System**: Unique identifiers for designation
- **Enhanced CLI**: New commands for tag and label management
- **GUI Integration**: Visual tag and label management interface
- **Backward Compatibility**: All existing configurations continue to work

## Pre-Migration Checklist

Before upgrading, ensure you have:

- [ ] Backed up your current AICred configuration
- [ ] Documented any custom provider configurations
- [ ] Tested the new version in a non-production environment
- [ ] Reviewed the new tagging and labeling features

## Backup Current Configuration

Create a backup of your current AICred configuration:

```bash
# Create backup directory
mkdir -p ~/aicred-backup-$(date +%Y%m%d)

# Backup configuration files
cp -r ~/.config/aicred ~/aicred-backup-$(date +%Y%m%d)/

# Backup any custom provider configurations
cp -r ~/.aicred ~/aicred-backup-$(date +%Y%m%d)/ 2>/dev/null || true

# Verify backup
ls -la ~/aicred-backup-$(date +%Y%m%d)/
```

## Upgrade Process

### 1. Install New Version

#### From Source
```bash
# Pull latest changes
git pull origin main

# Build and install
cargo build --release
cargo install --path .
```

#### From Package Manager
```bash
# Homebrew (macOS)
brew upgrade aicred

# Cargo
cargo install aicred

# Or download from releases
# https://github.com/your-org/aicred/releases
```

### 2. Verify Installation

```bash
# Check version
aicred version

# Should show version 0.2.0 or higher
```

### 3. Test Basic Functionality

```bash
# Test scan functionality
aicred scan --format summary

# Test existing commands
aicred providers
aicred instances list
```

## Post-Migration Configuration

### New Configuration Files

The new version creates additional configuration files:

```
~/.config/aicred/
├── tags.yaml              # NEW: Tag definitions
├── tag_assignments.yaml   # NEW: Tag assignments
├── labels.yaml            # NEW: Label definitions
├── label_assignments.yaml # NEW: Label assignments
├── provider_instances.yaml # EXISTING: Provider instances
└── provider_configs.yaml   # EXISTING: Provider configurations
```

### Configuration File Validation

The system automatically validates configuration files:

```bash
# Validate all configurations
aicred instances validate

# Check for any issues
aicred scan --dry-run --format json
```

## Migration Scenarios

### Scenario 1: Fresh Installation

For new installations, no migration is needed:

1. Install AICred 0.2.0
2. Configure provider instances as usual
3. Optionally add tags and labels for organization

```bash
# Example: Add tags for organization
aicred tags add --name "Production" --color "#ff0000"
aicred tags add --name "Development" --color "#00ff00"

# Assign tags to instances
aicred tags assign --name "Production" --instance-id my-openai
```

### Scenario 2: Existing Configuration

For existing configurations:

1. **Automatic Migration**: Configuration files are automatically compatible
2. **Optional Enhancement**: Add tags and labels to organize existing instances

```bash
# List existing instances
aicred instances list

# Add environment tags
aicred tags add --name "Production" --color "#ff0000"
aicred tags add --name "Staging" --color "#ffa500"

# Organize instances with tags
aicred tags assign --name "Production" --instance-id openai-prod
aicred tags assign --name "Staging" --instance-id openai-staging
```

### Scenario 3: Complex Provider Configurations

For complex multi-key configurations:

1. **Backward Compatibility**: Existing multi-key configurations work unchanged
2. **Simplified View**: Provider instances show simplified single-key view
3. **Migration Path**: Gradual migration to new instance model available

```bash
# View existing provider configurations
aicred instances list --verbose

# Create simplified instances from complex configs
# (Manual process - see Provider Instance Migration below)
```

## Provider Instance Migration

### Understanding the Change

**Before (v0.1.x)**:
- Complex `ProviderConfig` with multiple keys
- Multiple API keys per provider
- Complex validation and management

**After (v0.2.0)**:
- Simplified `ProviderInstance` with single key
- Easier management and assignment
- Full backward compatibility

### Migration Strategy

The system maintains both models during transition:

1. **Existing Configurations**: Continue to work unchanged
2. **New Features**: Use simplified `ProviderInstance` model
3. **Gradual Migration**: Move to new model at your own pace

### Manual Migration Steps

To migrate complex configurations to the new model:

```bash
# 1. Export existing configuration
aicred instances list --format json > current-instances.json

# 2. Analyze configuration
cat current-instances.json | jq '.[0]'

# 3. Create simplified instances
# (Manual process based on your specific needs)

# Example: Create instance from first valid key
aicred instances add \
  --id "openai-primary" \
  --name "OpenAI Primary" \
  --provider-type "openai" \
  --base-url "https://api.openai.com/v1" \
  --models "gpt-4,gpt-3.5-turbo"
```

## Tag and Label Migration

### Creating Initial Tags and Labels

After upgrading, create an organizational structure:

```bash
# Environment-based tags
aicred tags add --name "Production" --color "#ff0000" --description "Production environment"
aicred tags add --name "Staging" --color "#ffa500" --description "Staging environment"
aicred tags add --name "Development" --color "#00ff00" --description "Development environment"
aicred tags add --name "Testing" --color "#0000ff" --description "Testing environment"

# Team-based tags
aicred tags add --name "Team-Alpha" --color "#6c8cff" --description "Alpha team instances"
aicred tags add --name "Team-Beta" --color "#17c964" --description "Beta team instances"

# AI Development workflow labels
aicred labels add --name "Coding" --color "#17c964" --description "General coding and development tasks"
aicred labels add --name "Fast Coding" --color "#f5a524" --description "Quick prototyping and rapid development"
aicred labels add --name "Planning" --color "#9b6cff" --description "Strategic planning and architecture work"

# Legacy labels (for backward compatibility)
aicred labels add --name "Primary" --color "#17c964" --description "Primary provider instance"
aicred labels add --name "Backup" --color "#f5a524" --description "Backup provider instance"
aicred labels add --name "Deprecated" --color "#ff6b6b" --description "Deprecated instance"
```

### Organizing Existing Instances

```bash
# Assign environment tags
aicred tags assign --name "Production" --instance-id openai-prod
aicred tags assign --name "Staging" --instance-id openai-staging
aicred tags assign --name "Development" --instance-id openai-dev

# Assign team tags
aicred tags assign --name "Team-Alpha" --instance-id openai-prod
aicred tags assign --name "Team-Beta" --instance-id anthropic-prod

# Assign AI development workflow labels
aicred labels assign --name "Coding" --instance-id openai-dev
aicred labels assign --name "Fast Coding" --instance-id openai-prototype
aicred labels assign --name "Planning" --instance-id openai-architecture

# Assign legacy primary/backup labels (for backward compatibility)
aicred labels assign --name "Primary" --instance-id openai-prod
aicred labels assign --name "Backup" --instance-id openai-backup
```

## GUI Migration

### New GUI Features

The GUI now includes:

1. **Tag Management Interface**: Visual tag creation and management
2. **Label Management Interface**: Visual label creation and management
3. **Assignment Modal**: Easy tag/label assignment to instances and models
4. **Enhanced Instance View**: Show tags and labels in instance listings

### GUI Setup

```bash
# Build GUI (if building from source)
cd gui
npm install
npm run tauri build

# Run GUI
npm run dev
```

### GUI Migration Steps

1. **Launch GUI**: The GUI will automatically detect existing configurations
2. **Review Instances**: Existing instances appear without tags/labels
3. **Create Tags/Labels**: Use the GUI to create organizational structure
4. **Assign Tags/Labels**: Use the assignment modal to organize instances

## Troubleshooting Migration Issues

### Common Issues and Solutions

#### Issue 1: Configuration File Errors

**Symptom**:
```
Error: Failed to load configuration
```

**Solution**:
```bash
# Check configuration file syntax
cat ~/.config/aicred/tags.yaml | head -10

# Validate YAML syntax
python -c "import yaml; yaml.safe_load(open('~/.config/aicred/tags.yaml'))"

# Restore from backup if needed
cp ~/aicred-backup-*/tags.yaml ~/.config/aicred/tags.yaml
```

#### Issue 2: Tag/Label Assignment Failures

**Symptom**:
```
Error: Tag with name 'Production' not found
```

**Solution**:
```bash
# List existing tags
aicred tags list

# Check instance names
aicred instances list

# Verify assignment syntax
aicred tags assign --name "Production" --instance-id correct-instance-name
```

#### Issue 3: GUI Not Loading Tags/Labels

**Symptom**: GUI shows empty tag/label lists

**Solution**:
```bash# Check file permissions
ls -la ~/.config/aicred/

# Fix permissions if needed
chmod 644 ~/.config/aicred/*.yaml

# Restart GUI
```

#### Issue 4: Performance Issues

**Symptom**: Slow tag/label operations

**Solution**:
```bash
# Clean up unused tags/labels
aicred tags list
aicred labels list

# Remove unused items
aicred tags remove --name "unused-tag"
aicred labels remove --name "unused-label"

# Check for assignment conflicts
aicred labels list --verbose
```

### Validation Commands

```bash
# Validate all configurations
aicred instances validate

# Check tag integrity
aicred tags list --verbose

# Check label integrity
aicred labels list --verbose

# Test assignment operations
aicred tags assign --name "test" --instance-id test-instance
aicred tags unassign --name "test" --instance-id test-instance
```

### Recovery Procedures

#### Complete Reset

If migration completely fails:

```bash
# Stop AICred services
pkill aicred

# Restore from backup
rm -rf ~/.config/aicred
cp -r ~/aicred-backup-*/aicred ~/.config/

# Reinstall old version if needed
cargo install --version 0.1.0 aicred
```

#### Partial Recovery

For partial issues:

```bash
# Backup current state
cp -r ~/.config/aicred ~/.config/aicred-broken

# Restore specific files
cp ~/aicred-backup-*/tags.yaml ~/.config/aicred/tags.yaml
cp ~/aicred-backup-*/labels.yaml ~/.config/aicred/labels.yaml

# Test functionality
aicred tags list
aicred labels list
```

## Rollback Procedure

If you need to rollback to the previous version:

### 1. Stop Current Version

```bash
# Stop any running AICred processes
pkill aicred
```

### 2. Restore Configuration

```bash
# Restore configuration from backup
rm -rf ~/.config/aicred
cp -r ~/aicred-backup-*/aicred ~/.config/
```

### 3. Reinstall Previous Version

```bash
# Uninstall current version
cargo uninstall aicred

# Install previous version
cargo install --version 0.1.0 aicred

# Or build from previous tag
git checkout v0.1.0
cargo build --release
cargo install --path .
```

### 4. Verify Rollback

```bash
# Check version
aicred version

# Test functionality
aicred scan --format summary
aicred instances list
```

## Performance Considerations

### Post-Migration Optimization

```bash
# Clean up unused configurations
aicred instances list --verbose

# Remove unused tags and labels
aicred tags list
aicred labels list

# Optimize assignment structure
# (System automatically optimizes)
```

### Monitoring Performance

```bash
# Check configuration file sizes
du -sh ~/.config/aicred/*

# Monitor operation times
time aicred tags list
time aicred labels list
time aicred instances list
```

## Security Considerations

### Configuration Security

- Tag/label configurations contain no sensitive data
- Metadata should not include API keys or secrets
- Configuration files should have appropriate permissions (644)

```bash
# Set secure permissions
chmod 644 ~/.config/aicred/*.yaml
chmod 755 ~/.config/aicred
```

### Access Control

- Tag/label management requires local file system access
- No remote access to tag/label configurations
- Users can only manage tags/labels on their local system

## Best Practices Post-Migration

### Tag and Label Naming

```bash
# Use descriptive names
aicred tags add --name "Production-OpenAI" --description "OpenAI production instances"

# Use consistent naming conventions
aicred tags add --name "env-prod" --color "#ff0000"
aicred tags add --name "env-staging" --color "#ffa500"

# Avoid special characters in names
# Use hyphens instead of spaces when possible
```

### Organization Strategy

```bash
# Environment-based organization
aicred tags add --name "Production" --color "#ff0000"
aicred tags add --name "Staging" --color "#ffa500"
aicred tags add --name "Development" --color "#00ff00"

# Team-based organization
aicred tags add --name "Team-Platform" --color "#6c8cff"
aicred tags add --name "Team-ML" --color "#17c964"

# Use labels for unique designations
aicred labels add --name "Primary" --color "#17c964"
aicred labels add --name "Backup" --color "#f5a524"
```

### Regular Maintenance

```bash
# Weekly: Review and clean up unused tags/labels
aicred tags list
aicred labels list

# Monthly: Validate configuration integrity
aicred instances validate

# Quarterly: Review organization strategy
# Update tags/labels based on changing needs
```

## Support and Resources

### Getting Help

- **Documentation**: [Tagging System Guide](tagging-system-guide.md)
- **API Reference**: [API Reference](api-reference.md)
- **User Guide**: [User Guide](user-guide.md)
- **Architecture**: [Architecture Documentation](architecture.md)

### Reporting Issues

If you encounter migration issues:

1. Check this migration guide first
2. Review the troubleshooting section
3. Create a GitHub issue with:
   - AICred version
   - Operating system
   - Error messages
   - Steps to reproduce
   - Configuration files (sanitized)

### Community Resources

- **GitHub Discussions**: Community support and feature requests
- **Documentation**: Comprehensive guides and references
- **Examples**: Sample configurations and workflows

## Conclusion

The migration to AICred 0.2.0 is designed to be smooth and backward-compatible. The new tagging and labeling system provides powerful organization features while maintaining all existing functionality. Take time to explore the new features and gradually implement them in your workflow.

For additional support, refer to the comprehensive documentation or reach out to the community through GitHub discussions.