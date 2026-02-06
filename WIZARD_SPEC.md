# AICred Wizard Mode - Feature Specification

**Version:** 1.0  
**Date:** 2026-02-06  
**Status:** Draft  
**Branch:** `wizard`  

---

## Table of Contents

1. [Overview](#overview)
2. [Goals & Non-Goals](#goals--non-goals)
3. [User Experience](#user-experience)
4. [Technical Design](#technical-design)
5. [Implementation Plan](#implementation-plan)
6. [Edge Cases & Error Handling](#edge-cases--error-handling)
7. [Testing Strategy](#testing-strategy)
8. [Future Enhancements](#future-enhancements)

---

## Overview

The **Wizard Mode** provides an interactive, guided setup experience for first-time AICred users. Instead of manually running multiple commands (`scan`, `instances add`, `labels set`), users get a single, cohesive onboarding flow that:

1. Scans their system for existing AI configurations
2. Presents discovered credentials in an organized way
3. Guides them through selecting and importing providers
4. Helps them configure default models for common use cases
5. Creates a working configuration file ready for immediate use

**Trigger conditions:**
- User runs `aicred` with no subcommand and no existing config file → auto-launch wizard
- User explicitly runs `aicred wizard` → force wizard even if config exists

---

## Goals & Non-Goals

### Goals

✅ **Zero-to-productive in <2 minutes** - New users should go from installation to working config quickly  
✅ **Discoverability** - Surface existing credentials users may have forgotten about  
✅ **Sensible defaults** - Pre-populate choices based on scan results  
✅ **Education** - Teach users about key AICred concepts (instances, labels, models)  
✅ **Safety** - Never display raw API keys; always use redacted view  
✅ **Graceful degradation** - Work even if scan finds nothing  
✅ **Idempotent** - Running wizard multiple times should be safe  

### Non-Goals

❌ **API key creation** - Wizard won't generate new keys or authenticate with providers  
❌ **Advanced configuration** - No support for custom metadata, tags, or complex filtering in wizard  
❌ **Multi-user setup** - Wizard is for single-user local configuration only  
❌ **Migration tool** - Not designed to migrate from other credential managers  

---

## User Experience

### Flow Diagram

```
START
  │
  ├─→ Welcome Screen
  │   - Explain what wizard does
  │   - Show what will be scanned
  │
  ├─→ Scan Phase
  │   - Run system scan
  │   - Show progress indicator
  │   - Display summary of findings
  │
  ├─→ Review Discovered Credentials
  │   - Show table of discovered keys (redacted)
  │   - Group by provider type
  │   - Let user select which to import
  │
  ├─→ Configure Instances
  │   For each selected provider:
  │   - Confirm/edit instance ID
  │   - Confirm/edit display name
  │   - Confirm/edit base URL
  │   - Select available models (or probe for them)
  │   - Mark as active/inactive
  │
  ├─→ Set Default Models (Optional)
  │   - "Which model for fast/cheap tasks?"
  │   - "Which model for high-quality/smart tasks?"
  │   - Create label assignments
  │
  ├─→ Summary & Confirmation
  │   - Show what will be written
  │   - Preview config file location
  │   - Confirm or go back to edit
  │
  ├─→ Write Configuration
  │   - Create ~/.config/aicred/instances.yaml
  │   - Create ~/.config/aicred/labels.yaml (if labels set)
  │   - Show success message
  │
  └─→ Next Steps
      - Show example commands to try
      - Offer to run `aicred instances list`
END
```

### Screen-by-Screen Mockups

#### 1. Welcome Screen

```
╭─────────────────────────────────────────────────────────╮
│                                                         │
│  🚀 Welcome to AICred Setup Wizard                      │
│                                                         │
│  This wizard will help you:                            │
│   • Discover existing AI credentials on your system    │
│   • Import provider configurations                     │
│   • Set up default models for common tasks             │
│                                                         │
│  We'll scan these locations:                           │
│   • ~/.config/roo-code/                                │
│   • ~/Library/Application Support/Claude/             │
│   • Environment variables (OPENAI_API_KEY, etc.)       │
│   • ~/.env files                                       │
│                                                         │
│  No API keys will be displayed in plain text.          │
│                                                         │
╰─────────────────────────────────────────────────────────╯

Press ENTER to start, or Ctrl+C to cancel
```

#### 2. Scan Phase

```
Scanning your system for AI credentials...

✓ Checked ~/.config/roo-code/config.json
✓ Checked ~/Library/Application Support/Claude/config.json  
✓ Checked environment variables
⠋ Scanning ~/.env files...

Found 5 credentials across 3 providers
```

#### 3. Review Discovered Credentials

```
╭─────────────────────────────────────────────────────────────────────╮
│  Discovered Credentials                                             │
╰─────────────────────────────────────────────────────────────────────╯

Select credentials to import (Space to toggle, Enter to confirm):

  [✓] OpenAI API Key
      Source: ~/.config/roo-code/config.json
      Key (SHA-256): sk-...a3f2 (HIGH confidence)
      
  [✓] Anthropic API Key  
      Source: ~/Library/Application Support/Claude/config.json
      Key (SHA-256): sk-ant-...7b2e (HIGH confidence)
      
  [ ] Groq API Key
      Source: ~/.env
      Key (SHA-256): gsk-...9c1a (MEDIUM confidence)
      
  [✓] OpenRouter API Key
      Source: Environment (OPENROUTER_API_KEY)
      Key (SHA-256): sk-or-...4d8f (HIGH confidence)

4/4 selected
```

#### 4. Configure Instance - Example (OpenAI)

```
╭─────────────────────────────────────────────────────────╮
│  Configure OpenAI Instance (1/3)                        │
╰─────────────────────────────────────────────────────────╯

Instance ID: my-openai
  (Used to reference this provider in commands)

Display Name: OpenAI (Personal)

Base URL: https://api.openai.com/v1

API Key: sk-...a3f2 ✓ (from Roo Code config)

Available Models:
  We can probe the OpenAI API to get your available models,
  or you can enter them manually.
  
  [✓] Auto-detect models (requires API call)
  [ ] Enter manually
  
Probing OpenAI API...
✓ Found 12 models

Select models to enable:
  [✓] gpt-4-turbo
  [✓] gpt-4
  [✓] gpt-3.5-turbo
  [ ] gpt-3.5-turbo-16k
  [ ] text-davinci-003
  ...

Mark as active? [Y/n]: Y
```

#### 5. Set Default Models

```
╭─────────────────────────────────────────────────────────╮
│  Set Default Models (Optional)                          │
╰─────────────────────────────────────────────────────────╯

AICred's label system lets you assign semantic names to
provider:model combinations. This makes it easy to switch
between "fast" and "smart" models across your tools.

Examples:
  • "fast" → groq:llama3-70b-8192 (cheap, quick responses)
  • "smart" → openai:gpt-4 (high quality, reasoning)

Would you like to set up default labels? [Y/n]: Y

─────────────────────────────────────────────────────────

Label: "fast" (for quick, cheap tasks)

Available options:
  1. groq:llama3-70b-8192
  2. openai:gpt-3.5-turbo
  3. anthropic:claude-3-haiku-20240307

Select (1-3) [1]: 1

─────────────────────────────────────────────────────────

Label: "smart" (for high-quality tasks)

Available options:
  1. openai:gpt-4
  2. anthropic:claude-3-5-sonnet-20241022
  3. openrouter:deepseek-v3.2-exp

Select (1-3) [1]: 2

✓ Labels configured:
  • fast → groq:llama3-70b-8192
  • smart → anthropic:claude-3-5-sonnet-20241022
```

#### 6. Summary & Confirmation

```
╭─────────────────────────────────────────────────────────╮
│  Configuration Summary                                  │
╰─────────────────────────────────────────────────────────╯

The following will be saved to:
  ~/.config/aicred/instances.yaml
  ~/.config/aicred/labels.yaml

Provider Instances:
  ✓ my-openai (OpenAI Personal)
    - 12 models enabled
    - Status: Active
    
  ✓ my-anthropic (Anthropic)
    - 8 models enabled
    - Status: Active
    
  ✓ my-openrouter (OpenRouter)
    - 150+ models enabled
    - Status: Active

Labels:
  ✓ fast → groq:llama3-70b-8192
  ✓ smart → anthropic:claude-3-5-sonnet-20241022

Save and finish? [Y/n]: Y
```

#### 7. Success Screen

```
╭─────────────────────────────────────────────────────────╮
│  ✓ Configuration Complete!                              │
╰─────────────────────────────────────────────────────────╯

Your AICred configuration has been saved to:
  ~/.config/aicred/

What's next?

Try these commands:
  
  # View your providers
  aicred instances list
  
  # Check your labels
  aicred labels list
  
  # Use the wrap command to run apps with your config
  aicred wrap --labels fast -- python my_script.py

For more help, run: aicred --help

Happy building! 🚀
```

---

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────┐
│         CLI Entry Point (main.rs)           │
│  - Detect no config → auto-launch wizard    │
│  - Or handle `aicred wizard` command        │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│      Wizard Controller (wizard/mod.rs)      │
│  - Orchestrate the flow                     │
│  - State management                         │
│  - Error handling & rollback                │
└─────────────────┬───────────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
        ▼                   ▼
┌──────────────┐    ┌──────────────┐
│  Scan Phase  │    │  UI Layer    │
│   (core)     │    │ (inquire)    │
└──────────────┘    └──────────────┘
        │                   │
        └─────────┬─────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│    Configuration Writer (config/mod.rs)     │
│  - Serialize to YAML                        │
│  - Atomic writes                            │
│  - Backup existing configs                  │
└─────────────────────────────────────────────┘
```

### Data Structures

```rust
/// Wizard session state
pub struct WizardState {
    /// Scan results
    pub scan_result: ScanResult,
    
    /// User-selected credentials to import
    pub selected_credentials: Vec<DiscoveredCredential>,
    
    /// Configured provider instances
    pub instances: Vec<ProviderInstance>,
    
    /// Label assignments (optional)
    pub labels: HashMap<String, (String, String)>, // label -> (instance_id, model_id)
    
    /// Wizard configuration options
    pub options: WizardOptions,
}

#[derive(Debug, Clone)]
pub struct WizardOptions {
    /// Skip prompts, use defaults (for --yes flag)
    pub auto_accept: bool,
    
    /// Skip model probing
    pub skip_probe: bool,
    
    /// Probe timeout in seconds
    pub probe_timeout: u64,
    
    /// Skip label setup
    pub skip_labels: bool,
    
    /// Output verbosity
    pub verbose: bool,
}

/// Result of a wizard run
pub struct WizardResult {
    /// Number of instances created
    pub instances_created: usize,
    
    /// Number of labels created
    pub labels_created: usize,
    
    /// Path to config file
    pub config_path: PathBuf,
    
    /// Any warnings or notes
    pub warnings: Vec<String>,
}
```

### Module Structure

```
cli/src/
├── commands/
│   └── wizard/
│       ├── mod.rs          # Main wizard controller
│       ├── scan.rs         # Scan phase logic
│       ├── review.rs       # Credential review & selection
│       ├── configure.rs    # Instance configuration prompts
│       ├── labels.rs       # Label setup prompts
│       ├── summary.rs      # Show summary & confirmation
│       └── ui.rs           # Shared UI components (progress, tables, etc.)
└── main.rs                 # Entry point with auto-detect logic
```

### Dependencies

Add to `cli/Cargo.toml`:

```toml
[dependencies]
# Existing deps...

# Interactive prompts
inquire = "0.7"          # Main prompt library
indicatif = "0.17"       # Progress bars
console = "0.15"         # Terminal utilities (colors, styling)

# Optional: For more polished UI
# cliclack = "0.3"       # Alternative prettier prompt library
```

**Rationale for `inquire`:**
- More mature and feature-rich than `cliclack`
- Better autocomplete support
- Good validation and error handling
- Active maintenance
- Used by many popular Rust CLI tools

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

**PR 1: Basic Wizard Structure**
- [ ] Add `wizard` subcommand to CLI
- [ ] Create wizard module structure
- [ ] Implement auto-detection logic (no config → launch wizard)
- [ ] Add `inquire` dependency
- [ ] Create basic welcome screen

**PR 2: Scan Integration**
- [ ] Integrate existing scan functionality into wizard
- [ ] Add progress indicators during scan
- [ ] Format scan results for wizard display
- [ ] Handle empty scan results gracefully

### Phase 2: Core Flow (Week 2)

**PR 3: Credential Review & Selection**
- [ ] Implement multi-select credential picker
- [ ] Group credentials by provider
- [ ] Show confidence levels and sources
- [ ] Handle pre-selection (all high-confidence by default)

**PR 4: Instance Configuration**
- [ ] Instance-by-instance configuration prompts
- [ ] Default value pre-population from scan
- [ ] Validation (instance IDs, URLs, etc.)
- [ ] Model probing integration
- [ ] Manual model entry fallback

### Phase 3: Labels & Finalization (Week 3)

**PR 5: Label Setup**
- [ ] Optional label creation flow
- [ ] Pre-defined label templates ("fast", "smart", "cheap")
- [ ] Model selection from available instances
- [ ] Label validation

**PR 6: Summary & Write**
- [ ] Configuration summary screen
- [ ] Confirmation prompt
- [ ] Atomic YAML writes
- [ ] Backup existing configs
- [ ] Success screen with next steps

### Phase 4: Polish & Testing (Week 4)

**PR 7: Error Handling & Edge Cases**
- [ ] Handle scan failures
- [ ] Handle probe failures
- [ ] Support config file exists (offer merge vs. replace)
- [ ] Add `--yes` flag for non-interactive mode
- [ ] Add `--skip-labels` flag

**PR 8: Documentation & Tests**
- [ ] Update README with wizard instructions
- [ ] Add wizard demo GIF/video
- [ ] Integration tests for wizard flow
- [ ] Unit tests for each phase
- [ ] Update CHANGELOG

---

## Edge Cases & Error Handling

### 1. **No Existing Config (Fresh Install)**
**Behavior:** Auto-launch wizard on first `aicred` run  
**Fallback:** If user declines wizard, create minimal empty config

### 2. **Config Already Exists**
**Behavior:**
- `aicred` → normal CLI behavior (no wizard)
- `aicred wizard` → prompt user:
  ```
  Configuration file already exists at ~/.config/aicred/instances.yaml
  
  What would you like to do?
    1. Merge (add new providers, keep existing)
    2. Replace (overwrite existing config)
    3. Cancel
  ```

### 3. **Scan Finds Nothing**
**Behavior:**
- Show "No existing credentials found" message
- Offer manual provider setup:
  ```
  No AI credentials were found on your system.
  
  Would you like to manually add a provider? [Y/n]: Y
  
  Provider type (openai, anthropic, groq, openrouter, etc.): openai
  Instance ID: my-openai
  API Key (will be saved securely): sk-...
  ```

### 4. **Probe Failures**
**Behavior:**
- Show warning: "Could not probe OpenAI API (network error)"
- Offer options:
  1. Retry
  2. Skip probing (enter models manually)
  3. Leave models empty (can add later)

### 5. **Invalid Input**
**Behavior:**
- Inline validation with helpful error messages
- Example:
  ```
  Instance ID: my openai
  ✗ Instance IDs cannot contain spaces. Use hyphens or underscores.
  
  Instance ID: my-openai
  ✓
  ```

### 6. **Write Failures**
**Behavior:**
- Show clear error: "Could not write config file (permission denied)"
- Suggest fix: "Try running: mkdir -p ~/.config/aicred && chmod 755 ~/.config/aicred"
- Offer to save config to alternative location

### 7. **Partial Completion**
**Behavior:**
- If user exits early (Ctrl+C), offer to save partial progress:
  ```
  Wizard interrupted. Save partial configuration? [Y/n]: Y
  ✓ Saved 2/3 instances to ~/.config/aicred/instances.yaml
  
  To complete setup, run: aicred wizard
  ```

### 8. **Multiple Keys for Same Provider**
**Behavior:**
- Group by provider, let user name instances:
  ```
  Found 2 OpenAI API keys:
  
  Key 1 (from Roo Code): sk-...a3f2
  Instance ID: openai-roo
  
  Key 2 (from environment): sk-...9b1e  
  Instance ID: openai-env
  ```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_state_initialization() {
        let state = WizardState::new();
        assert!(state.selected_credentials.is_empty());
        assert!(state.instances.is_empty());
    }
    
    #[test]
    fn test_instance_id_validation() {
        assert!(validate_instance_id("my-openai").is_ok());
        assert!(validate_instance_id("my openai").is_err());
        assert!(validate_instance_id("123-start-with-number").is_err());
    }
    
    #[test]
    fn test_label_parsing() {
        let result = parse_label_assignment("fast=groq:llama3-70b-8192");
        assert!(result.is_ok());
        let (label, instance, model) = result.unwrap();
        assert_eq!(label, "fast");
        assert_eq!(instance, "groq");
        assert_eq!(model, "llama3-70b-8192");
    }
}
```

### Integration Tests

```rust
#[test]
fn test_wizard_full_flow_with_mock_input() {
    // Simulate user input sequence
    let inputs = vec![
        "Y",           // Confirm start
        " ",           // Select first credential
        "Enter",       // Confirm selection
        "my-openai",   // Instance ID
        "Y",           // Auto-detect models
        "Y",           // Set up labels
        "1",           // Select fast model
        "2",           // Select smart model
        "Y",           // Confirm and save
    ];
    
    let result = run_wizard_with_inputs(inputs);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().instances_created, 1);
}

#[test]
fn test_wizard_handles_no_scan_results() {
    let mock_scan = ScanResult::empty();
    let result = run_wizard_with_scan(mock_scan);
    assert!(result.is_ok());
    // Should offer manual provider setup
}

#[test]
fn test_wizard_config_file_backup() {
    // Create existing config
    create_test_config();
    
    // Run wizard
    let result = run_wizard();
    
    // Verify backup was created
    assert!(Path::new("~/.config/aicred/instances.yaml.backup").exists());
}
```

### Manual Testing Checklist

- [ ] Fresh install (no config) → auto-launches wizard
- [ ] Fresh install, decline wizard → creates empty config
- [ ] Existing config, run `aicred wizard` → offers merge/replace
- [ ] Scan finds credentials → displays correctly
- [ ] Scan finds nothing → offers manual setup
- [ ] Probe succeeds → shows models
- [ ] Probe fails → offers retry/skip/manual
- [ ] Invalid instance ID → shows inline error
- [ ] Ctrl+C during wizard → offers save partial
- [ ] Multiple keys same provider → prompts for instance names
- [ ] Label setup (optional) → can skip
- [ ] Summary screen → accurate preview
- [ ] Config write succeeds → shows success message
- [ ] Config write fails → shows helpful error + suggestions

---

## Future Enhancements

### Post-v1 (Nice to Have)

1. **Wizard Replay Mode**
   ```bash
   aicred wizard --replay
   # Re-runs wizard, shows previous choices as defaults
   ```

2. **Template Configs**
   ```bash
   aicred wizard --template data-science
   # Pre-configures labels for data science workflows
   # (fast, smart, embeddings, vision, etc.)
   ```

3. **Migration Wizard**
   ```bash
   aicred wizard --migrate-from langchain
   # Imports configs from LangChain, LiteLLM, etc.
   ```

4. **Team Setup**
   ```bash
   aicred wizard --team-config team.yaml
   # Loads shared team configuration template
   ```

5. **Wizard Shortcuts**
   ```bash
   aicred wizard --quick
   # Speed run: auto-accept defaults, skip all prompts
   
   aicred wizard --provider openai
   # Jump directly to adding a specific provider
   ```

6. **Health Check Integration**
   ```bash
   # At end of wizard, offer to test connections
   Would you like to test your provider connections? [Y/n]: Y
   
   Testing my-openai... ✓ (200 OK, 12 models available)
   Testing my-anthropic... ✓ (200 OK, 8 models available)
   Testing my-groq... ✗ (401 Unauthorized - check API key)
   ```

---

## Appendix: Research & References

### CLI Wizard Patterns (Researched)

**Good Examples:**
- `npm init` - Interactive package.json creation
- `openclaw onboard` - OpenClaw's own onboarding wizard
- `git init` - Simple repository setup
- `cargo init` - Rust project initialization
- `gh auth login` - GitHub CLI authentication flow

**Best Practices Identified:**
1. **Start with a clear welcome** - Explain what the wizard does
2. **Show progress** - Users should know where they are in the flow
3. **Pre-fill when possible** - Use scan results as defaults
4. **Allow skipping** - Not everyone needs every feature
5. **Validate inline** - Show errors immediately, not at the end
6. **Summarize before commit** - Give users a chance to review
7. **Provide next steps** - Don't leave users wondering what's next
8. **Make it re-runnable** - Wizards shouldn't be "one shot only"

### Library Comparison (inquire vs. dialoguer vs. cliclack)

| Feature | inquire | dialoguer | cliclack |
|---------|---------|-----------|----------|
| Text input | ✓ | ✓ | ✓ |
| Select | ✓ | ✓ | ✓ |
| Multi-select | ✓ | ✓ | ✓ |
| Autocomplete | ✓ | ✗ | ✗ |
| Validation | ✓ | ✓ | ✓ |
| Themes/Styling | ✓ | ✓ | ✓✓ (prettier) |
| Maintenance | Active | Active | Newer |
| Maturity | High | High | Medium |

**Recommendation:** Start with `inquire` for features + stability. Can switch to `cliclack` later for aesthetics if desired.

---

## Approval & Sign-Off

**Stakeholder Review:**
- [ ] Product review (feature scope)
- [ ] Engineering review (technical approach)
- [ ] UX review (flow and prompts)
- [ ] Security review (credential handling)

**Acceptance Criteria:**
- [ ] Wizard can discover and import existing credentials
- [ ] Wizard can configure provider instances from scratch
- [ ] Wizard creates valid YAML configuration files
- [ ] Wizard handles errors gracefully
- [ ] Wizard provides clear next steps
- [ ] All tests pass
- [ ] Documentation updated

---

**Document Version History:**
- v1.0 (2026-02-06): Initial specification

