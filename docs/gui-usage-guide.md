# AICred GUI Usage Guide

## Overview

The AICred GUI provides a visual interface for managing your AI provider configurations, tags, and labels. This guide walks you through all the features and workflows available in the graphical interface.

## Getting Started

### Launching the GUI

```bash
# Launch the AICred GUI
aicred gui

# Or use the desktop application
open /Applications/AICred.app
```

### Main Interface Layout

The AICred GUI consists of several key areas:

```
┌─────────────────────────────────────────────────────────────┐
│ AICred - AI Provider Configuration Manager                  │
├─────────────────────────────────────────────────────────────┤
│ [Providers] [Tags] [Labels] [Scan] [Settings]              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Main Content Area                                          │
│  - Provider List (Providers tab)                           │
│  - Tag Management (Tags tab)                               │
│  - Label Management (Labels tab)                           │
│  - Scan Results (Scan tab)                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Provider Management

### Provider List View

The main providers tab shows all your configured AI provider instances:

```
┌─────────────────────────────────────────────────────────────┐
│ Provider Instances                                          │
├─────────────────────────────────────────────────────────────┤
│ ┌─ OpenAI Production ─────────────────────────────── [Edit] │
│ │ Status: ✅ Active    Type: OpenAI                   [Del] │
│ │ Environment: Production    Primary: Yes              │
│ │ Tags: production, primary                          │
│ │ Labels: primary                                    │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─ Anthropic Staging ───────────────────────────────────────│
│ │ Status: ⚠️ Inactive  Type: Anthropic               [Edit] │
│ │ Environment: Staging    Primary: No                 [Del] │
│ │ Tags: staging                                   │
│ │ Labels: backup                                  │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [+ Add Provider] [🔄 Refresh] [📊 Analytics]                │
└─────────────────────────────────────────────────────────────┘
```

### Provider Details

Click on any provider to view detailed information:

```
┌─────────────────────────────────────────────────────────────┐
│ Provider Details: OpenAI Production                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Basic Information:                                          │
│ • Name: OpenAI Production                                  │
│ • Type: OpenAI                                             │
│ • Status: Active                                           │
│ • Created: 2025-01-15 10:30:00                            │
│ • Updated: 2025-01-20 14:22:00                            │
│                                                             │
│ Configuration:                                              │
│ • API Endpoint: https://api.openai.com/v1                 │
│ • Environment: Production                                  │
│ • Primary: Yes                                             │
│                                                             │
│ Tags & Labels:                                              │
│ Tags: production, primary                                  │
│ Labels: primary                                            │
│                                                             │
│ Actions:                                                    │
│ [Edit Configuration] [Manage Tags] [Manage Labels]         │
│ [Test Connection] [View Logs] [Delete Provider]            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Tag Management

### Tag Management Interface

The tags tab provides comprehensive tag management capabilities:

```
┌─────────────────────────────────────────────────────────────┐
│ Tag Management                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─ Environment Tags ────────────────────────────────────────┐
│ │ 🔴 Production (5 instances)                              │
│ │ 🟠 Staging (2 instances)                                 │
│ │ 🟢 Development (8 instances)                             │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─ Custom Tags ─────────────────────────────────────────────│
│ │ 🔵 Critical (1 instance)                                 │
│ │ 🟡 Important (3 instances)                               │
│ │ ⚪ Deprecated (2 instances)                              │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [+ Create Tag] [📤 Import] [📥 Export] [🔄 Refresh]         │
└─────────────────────────────────────────────────────────────┘
```

### Create New Tag

Click "Create Tag" to open the tag creation dialog:

```
┌─────────────────────────────────────────────────────────────┐
│ Create New Tag                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Tag Name: [Production________________]                     │
│                                                             │
│ Description: [Production environment instances____________] │
│                                                             │
│ Color: [🔴 Red] [🟠 Orange] [🟢 Green] [🔵 Blue] [Custom]    │
│                                                             │
│ Metadata:                                                   │
│ Environment: [production________________]                  │
│ Category: [environment_______________]                     │
│ Priority: [high____________________]                       │
│                                                             │
│ [+ Add Metadata Field]                                     │
│                                                             │
│ Actions:                                                    │
│ [Cancel] [Create Tag]                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Tag Assignment

Select a tag to view and manage assignments:

```
┌─────────────────────────────────────────────────────────────┐
│ Tag: Production                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Tag Information:                                            │
│ • Name: Production                                         │
│ • Description: Production environment instances            │
│ • Color: Red (#ff0000)                                     │
│ • Created: 2025-01-15 10:30:00                            │
│ • Instances: 5                                             │
│                                                             │
│ Assigned Instances:                                         │
│ ┌─ OpenAI Production ─────────────────────────────── [✕]   │
│ │ Type: OpenAI    Status: Active    Primary: Yes           │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─ Anthropic Production ───────────────────────────── [✕]  │
│ │ Type: Anthropic Status: Active    Primary: No            │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [+ Assign to Instance] [📊 Usage Analytics]                │
│                                                             │
│ Actions:                                                    │
│ [Edit Tag] [Delete Tag] [Export Assignments]               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Label Management

### Label Management Interface

The labels tab shows all configured labels:

```
┌─────────────────────────────────────────────────────────────┐
│ Label Management                                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─ Designation Labels ──────────────────────────────────────┐
│ │ 🟢 Primary (1 instance)                                  │
│ │ 🟡 Backup (4 instances)                                  │
│ │ 🔴 Deprecated (2 instances)                              │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─ Status Labels ───────────────────────────────────────────│
│ │ 🟢 Active (6 instances)                                  │
│ │ 🟡 Inactive (1 instance)                                 │
│ │ 🔴 Error (0 instances)                                   │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [+ Create Label] [📤 Import] [📥 Export] [🔄 Refresh]       │
└─────────────────────────────────────────────────────────────┘
```

### Create New Label

Click "Create Label" to open the label creation dialog:

```
┌─────────────────────────────────────────────────────────────┐
│ Create New Label                                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Label Name: [Primary________________]                      │
│                                                             │
│ Description: [Primary provider instance__________________] │
│                                                             │
│ Color: [🟢 Green] [🟡 Yellow] [🔴 Red] [🔵 Blue] [Custom]    │
│                                                             │
│ Uniqueness Settings:                                        │
│ ☑️ Enforce global uniqueness (only one instance can have   │
│    this label)                                              │
│                                                             │
│ Assignment Rules:                                           │
│ ☑️ Can assign to Provider Instances                         │
│ ☑️ Can assign to Models                                     │
│                                                             │
│ Metadata:                                                   │
│ Category: [designation_____________]                       │
│ Priority: [high____________________]                       │
│                                                             │
│ [+ Add Metadata Field]                                     │
│                                                             │
│ Actions:                                                    │
│ [Cancel] [Create Label]                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Label Assignment

Select a label to view assignments:

```
┌─────────────────────────────────────────────────────────────┐
│ Label: Primary                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Label Information:                                          │
│ • Name: Primary                                            │
│ • Description: Primary provider instance                   │
│ • Color: Green (#17c964)                                   │
│ • Global Uniqueness: Enabled                               │
│ • Created: 2025-01-15 10:30:00                            │
│ • Assigned To: OpenAI Production                           │
│                                                             │
│ Current Assignment:                                         │
│ ┌─ OpenAI Production ─────────────────────────────── [✕]   │
│ │ Type: OpenAI    Status: Active    Environment: Production│
│ │ Assigned: 2025-01-15 10:30:00                         │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [+ Assign to Instance] [📊 Usage Analytics]                │
│                                                             │
│ Actions:                                                    │
│ [Edit Label] [Delete Label] [Export Assignments]           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Assignment Management

### Assignment Modal

The assignment modal provides a unified interface for managing tag and label assignments:

```
┌─────────────────────────────────────────────────────────────┐
│ Manage Assignments - OpenAI Production                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Current Tags:                                               │
│ ┌─ Production ─────────────────────────────── [Remove]     │
│ │ Environment: production    Category: environment         │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─ Primary ─────────────────────────────── [Remove]        │
│ │ Category: designation    Priority: high                  │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Current Labels:                                             │
│ ┌─ Primary ─────────────────────────────── [Remove]        │
│ │ Category: designation    Priority: high                  │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ Add New Tags:                                               │
│ [🔵 Critical] [🟡 Important] [⚪ Deprecated] [+ Custom]     │
│                                                             │
│ Add New Labels:                                             │
│ [🟡 Backup] [🔴 Deprecated] [+ Custom]                      │
│                                                             │
│ Actions:                                                    │
│ [Cancel] [Save Changes]                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Scan Results

### Scan Results View

The scan tab shows results from AI provider key discovery:

```
┌─────────────────────────────────────────────────────────────┐
│ Scan Results                                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Last Scan: 2025-01-20 15:30:00 (2 minutes ago)            │
│ Status: ✅ Completed Successfully                           │
│                                                             │
│ ┌─ OpenAI API Key Found ─────────────────────────────── [✓]│
│ │ Source: ~/.config/openai/config.json                   │
│ │ Status: ✅ Valid    Confidence: High                   │
│ │ Environment: Production    Tags: production, primary   │
│ │ Labels: primary                                    │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─ Anthropic API Key Found ────────────────────────────────│
│ │ Source: ~/.anthropic/config                            │
│ │ Status: ⚠️ Invalid  Confidence: Medium                 │
│ │ Environment: Staging    Tags: staging                  │
│ │ Labels: backup                                    │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ [🔄 Rescan] [📊 Analytics] [📤 Export] [⚙️ Configure]       │
└─────────────────────────────────────────────────────────────┘
```

## Settings and Configuration

### Settings Interface

Access settings through the Settings tab:

```
┌─────────────────────────────────────────────────────────────┐
│ Settings                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ General Settings:                                           │
│ ☑️ Auto-save configuration changes                         │
│ ☑️ Show confirmation dialogs for destructive actions       │
│ ☑️ Enable automatic tag assignment based on patterns       │
│                                                             │
│ Display Settings:                                           │
│ Theme: [Light] [Dark] [Auto]                               │
│ Language: [English] [Spanish] [French]                     │
│                                                             │
│ Notification Settings:                                      │
│ ☑️ Notify on scan completion                               │
│ ☑️ Notify on configuration errors                          │
│ ☑️ Notify on invalid API keys                              │
│                                                             │
│ Backup Settings:                                            │
│ ☑️ Auto-backup before major changes                        │
│ Backup Location: [/Users/username/aicred-backups]          │
│                                                             │
│ Actions:                                                    │
│ [Export Settings] [Import Settings] [Reset to Defaults]    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Workflow Examples

### Example 1: Setting Up a New Production Environment

1. **Create Environment Tags**:
   - Go to Tags tab
   - Click "Create Tag"
   - Name: "Production"
   - Color: Red
   - Add metadata: environment=production

2. **Create Primary Label**:
   - Go to Labels tab
   - Click "Create Label"
   - Name: "Primary"
   - Enable global uniqueness
   - Color: Green

3. **Add Provider Instance**:
   - Go to Providers tab
   - Click "Add Provider"
   - Configure OpenAI instance
   - Assign "Production" tag
   - Assign "Primary" label

### Example 2: Migrating Existing Configuration

1. **Backup Current Configuration**:
   - Go to Settings
   - Click "Export Settings"
   - Save backup file

2. **Import Configuration**:
   - Use migration tools
   - Validate migrated data
   - Review auto-assigned tags

3. **Review and Adjust**:
   - Check all provider instances
   - Verify tag/label assignments
   - Update metadata as needed

### Example 3: Bulk Tag Management

1. **Filter Provider Instances**:
   - Use search/filter in Providers tab
   - Select multiple instances

2. **Apply Tags**:
   - Right-click selected instances
   - Choose "Apply Tags"
   - Select tags to apply

3. **Review Changes**:
   - Check assignment results
   - Verify metadata updates

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + N` | Create new item (tag/label/provider) |
| `Cmd/Ctrl + S` | Save changes |
| `Cmd/Ctrl + R` | Refresh current view |
| `Cmd/Ctrl + F` | Search/filter |
| `Cmd/Ctrl + E` | Edit selected item |
| `Delete` | Delete selected item |
| `Cmd/Ctrl + A` | Select all |
| `Cmd/Ctrl + Z` | Undo |
| `Cmd/Ctrl + Y` | Redo |

## Troubleshooting

### Common Issues

**Issue**: GUI won't start
- **Solution**: Check if AICred is properly installed and dependencies are met
- **Command**: `aicred --version` to verify installation

**Issue**: Changes not saving
- **Solution**: Check file permissions in configuration directory
- **Command**: `ls -la ~/.config/aicred/`

**Issue**: Tags/labels not appearing
- **Solution**: Refresh the view or restart the GUI
- **Action**: Click refresh button or restart application

**Issue**: Assignment conflicts
- **Solution**: Check uniqueness constraints for labels
- **Action**: Review label assignment rules

### Performance Tips

1. **Large Configurations**: Use filtering to reduce displayed items
2. **Regular Backups**: Enable auto-backup in settings
3. **Memory Usage**: Close unused tabs to free memory
4. **Network Operations**: Use offline mode for local-only operations

## Advanced Features

### Custom Metadata

Add custom metadata fields to tags and labels:

```
┌─────────────────────────────────────────────────────────────┐
│ Custom Metadata Fields                                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Field Name: [Cost Center]                                  │
│ Field Type: [Text] [Number] [Date] [Boolean] [Select]      │
│ Required: ☑️                                               │
│ Default Value: [ENG-001]                                   │
│                                                             │
│ Validation Rules:                                          │
│ Pattern: [^[A-Z]{3}-[0-9]{3}$]                            │
│ Description: Cost center identifier (e.g., ENG-001)        │
│                                                             │
│ [+ Add Field] [Save] [Cancel]                              │
└─────────────────────────────────────────────────────────────┘
```

### Analytics Dashboard

View usage analytics and insights:

```
┌─────────────────────────────────────────────────────────────┐
│ Analytics Dashboard                                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Tag Usage Statistics:                                       │
│ • Most Used: Production (5 instances)                      │
│ • Least Used: Deprecated (0 instances)                     │
│ • Growth: +2 tags this month                               │
│                                                             │
│ Label Distribution:                                         │
│ • Primary: 1 instance (unique)                             │
│ • Backup: 4 instances                                      │
│ • Deprecated: 2 instances                                  │
│                                                             │
│ Provider Health:                                            │
│ • Active: 6 instances                                      │
│ • Inactive: 1 instance                                     │
│ • Error: 0 instances                                       │
│                                                             │
│ [📊 Detailed Report] [📈 Trends] [📤 Export]                │
└─────────────────────────────────────────────────────────────┘
```

This GUI provides a comprehensive interface for managing your AI provider configurations with intuitive visual tools for tag and label management.
