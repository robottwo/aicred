# AICred Comprehensive Refactoring Plan

**Branch:** `code-cleanup`  
**Start Date:** 2026-02-04  
**Estimated Duration:** 3-4 weeks  
**Target Release:** v0.2.0

## Overview

This plan systematically addresses all issues identified in CODE_AUDIT.md through 5 phases of refactoring. Each phase is independently testable and can be completed in 3-5 days.

## Table of Contents

1. [Pre-Refactor Setup](#phase-0-pre-refactor-setup)
2. [Phase 1: Model Consolidation](#phase-1-model-consolidation)
3. [Phase 2: Scanner Simplification](#phase-2-scanner-simplification)
4. [Phase 3: Plugin System Reduction](#phase-3-plugin-system-reduction)
5. [Phase 4: Technical Debt Cleanup](#phase-4-technical-debt-cleanup)
6. [Phase 5: Documentation & Polish](#phase-5-documentation--polish)
7. [Testing Strategy](#testing-strategy)
8. [Rollback Procedures](#rollback-procedures)

---

## Phase 0: Pre-Refactor Setup

**Duration:** 1 day  
**Goal:** Establish baseline and safety nets

### Tasks

#### 0.1: Document Current Public API
```bash
# Generate API documentation
cargo doc --no-deps --open

# Export public API surface
cargo tree --depth 1 > docs/pre-refactor-deps.txt

# Document all pub use statements
rg "^pub use" core/src/lib.rs > docs/public-api-baseline.txt
```

**Deliverable:** `docs/API_BASELINE.md` documenting all public exports

#### 0.2: Add Integration Test Suite
```bash
# Create comprehensive integration test
touch core/tests/refactor_regression_tests.rs
```

**Content:**
```rust
//! Regression tests to ensure refactoring doesn't break existing functionality.

use aicred_core::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_basic_scan_flow() {
    let temp = TempDir::new().unwrap();
    let options = ScanOptions {
        home_dir: Some(temp.path().to_path_buf()),
        include_full_values: false,
        max_file_size: 1024 * 1024,
        only_providers: None,
        exclude_providers: None,
        probe_models: false,
        probe_timeout_secs: 30,
    };
    
    let result = scan(&options);
    assert!(result.is_ok());
}

#[test]
fn test_plugin_registry_basics() {
    let registry = PluginRegistry::new();
    register_builtin_plugins(&registry);
    
    assert!(registry.get("openai").is_some());
    assert!(registry.get("anthropic").is_some());
    assert!(registry.get("groq").is_some());
}

#[test]
fn test_provider_instance_validation() {
    // Test that existing validation still works
    // Will be updated as we refactor
}

// Add 10-15 more tests covering critical paths
```

#### 0.3: Benchmark Current Performance
```bash
touch core/benches/scan_benchmark.rs
```

**Content:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aicred_core::*;

fn benchmark_scan(c: &mut Criterion) {
    let temp = tempfile::TempDir::new().unwrap();
    let options = ScanOptions::default();
    
    c.bench_function("full_scan", |b| {
        b.iter(|| {
            scan(black_box(&options))
        });
    });
}

criterion_group!(benches, benchmark_scan);
criterion_main!(benches);
```

Add to `core/Cargo.toml`:
```toml
[[bench]]
name = "scan_benchmark"
harness = false

[dev-dependencies]
criterion = "0.5"
```

Run baseline:
```bash
cargo bench --bench scan_benchmark > docs/pre-refactor-benchmarks.txt
```

#### 0.4: Create Feature Flags for Compatibility
Add to `core/Cargo.toml`:
```toml
[features]
default = []
compat_v0_1 = []  # Backward compatibility with 0.1.x API
deprecated_models = []  # Keep deprecated models during migration
```

#### 0.5: Set Up CI for Branch
Ensure GitHub Actions runs on `code-cleanup` branch:
```bash
# Push branch to trigger CI
git push -u origin code-cleanup
```

**Success Criteria:**
- [ ] All existing tests pass
- [ ] Benchmarks captured
- [ ] Public API documented
- [ ] Feature flags configured
- [ ] CI running on branch

---

## Phase 1: Model Consolidation

**Duration:** 5-7 days  
**Goal:** Reduce models/ from 18 files to 6-7 files

### Step 1.1: Decision - Tag vs Label (Day 1)

**Decision:** Use "Label" as the canonical term.

**Rationale:**
- More intuitive for users ("label this as 'fast'")
- Industry standard (Kubernetes, Docker use labels)
- "Tag" has connotations with git/version control

**Migration Path:**
```rust
// Add type aliases for backward compatibility
#[cfg(feature = "compat_v0_1")]
pub type Tag = Label;

#[cfg(feature = "compat_v0_1")]
pub type TagAssignment = LabelAssignment;
```

### Step 1.2: Create New Consolidated Models (Day 1-2)

#### File: `core/src/models/providers.rs`
**Merges:** `provider.rs` + `provider_instance.rs` + `provider_instances.rs`

```rust
//! Provider metadata and instance configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about an AI provider (e.g., OpenAI, Anthropic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub description: String,
    pub auth_methods: Vec<AuthMethod>,
    pub rate_limits: Option<RateLimit>,
    pub base_url_default: String,
}

/// Authentication method for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey,
    BearerToken,
    OAuth,
    Basic { username: String },
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: Option<u32>,
    pub requests_per_day: Option<u32>,
    pub tokens_per_minute: Option<u64>,
}

/// A configured instance of a provider with credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInstance {
    pub id: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub capabilities: Capabilities,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Capabilities of a provider instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub chat: bool,
    pub completion: bool,
    pub embedding: bool,
    pub image_generation: bool,
    pub function_calling: bool,
    pub streaming: bool,
}

/// Collection of provider instances (instances.yaml representation).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCollection {
    #[serde(flatten)]
    pub instances: HashMap<String, ProviderInstance>,
}

impl ProviderCollection {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, id: String, instance: ProviderInstance) {
        self.instances.insert(id, instance);
    }
    
    pub fn get(&self, id: &str) -> Option<&ProviderInstance> {
        self.instances.get(id)
    }
    
    pub fn remove(&mut self, id: &str) -> Option<ProviderInstance> {
        self.instances.remove(id)
    }
    
    pub fn list(&self) -> Vec<&ProviderInstance> {
        self.instances.values().collect()
    }
}
```

#### File: `core/src/models/credentials.rs`
**Merges:** `discovered_key.rs` + `provider_key.rs`

```rust
//! Credential discovery and management.

use serde::{Deserialize, Serialize};

/// A credential discovered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCredential {
    pub provider: String,
    pub value: CredentialValue,
    pub confidence: Confidence,
    pub source_file: String,
    pub source_line: Option<usize>,
    pub environment: Environment,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Credential value (full or redacted for security).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialValue {
    Full(String),
    Redacted { sha256: String, prefix: String },
}

impl CredentialValue {
    pub fn redact(key: &str) -> Self {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(key.as_bytes());
        let prefix = if key.len() >= 8 {
            key[..8].to_string()
        } else {
            key.to_string()
        };
        
        Self::Redacted {
            sha256: hex::encode(hash),
            prefix,
        }
    }
    
    pub fn full(key: String) -> Self {
        Self::Full(key)
    }
}

/// Confidence level for discovered credentials.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Confidence {
    Low,      // < 0.5
    Medium,   // 0.5 - 0.7
    High,     // 0.7 - 0.9
    VeryHigh, // > 0.9
}

impl From<f32> for Confidence {
    fn from(score: f32) -> Self {
        if score < 0.5 { Self::Low }
        else if score < 0.7 { Self::Medium }
        else if score < 0.9 { Self::High }
        else { Self::VeryHigh }
    }
}

/// Environment where credential was discovered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    SystemConfig,
    UserConfig,
    ProjectConfig { project_path: String },
    EnvironmentVariable,
}

/// Validation status for a credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    NotValidated,
    Valid,
    Invalid { reason: String },
    RateLimited,
    NetworkError,
}
```

#### File: `core/src/models/labels.rs`
**Merges:** `label.rs` + `tag.rs` + `label_assignment.rs` + `tag_assignment.rs` + `unified_label.rs`

```rust
//! Semantic labeling system for provider:model combinations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A semantic label (e.g., "fast", "smart", "cheap").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Assignment linking a label to a provider:model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelAssignment {
    pub label_name: String,
    pub target: LabelTarget,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub assigned_by: Option<String>,
}

/// Target of a label assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelTarget {
    /// Entire provider instance
    ProviderInstance { instance_id: String },
    /// Specific model within an instance
    ProviderModel { instance_id: String, model_id: String },
}

impl LabelTarget {
    pub fn instance_id(&self) -> &str {
        match self {
            Self::ProviderInstance { instance_id } => instance_id,
            Self::ProviderModel { instance_id, .. } => instance_id,
        }
    }
}

/// Combined view of label with its assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelWithAssignments {
    pub label: Label,
    pub assignments: Vec<LabelAssignment>,
}

// Backward compatibility type aliases
#[cfg(feature = "compat_v0_1")]
pub type Tag = Label;

#[cfg(feature = "compat_v0_1")]
pub type TagAssignment = LabelAssignment;

#[cfg(feature = "compat_v0_1")]
pub type TagAssignmentTarget = LabelTarget;
```

#### File: `core/src/models/models.rs`
**Merges:** `model.rs` + `model_metadata.rs`

```rust
//! LLM model definitions and metadata.

use serde::{Deserialize, Serialize};

/// An LLM model with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub capabilities: ModelCapabilities,
    pub context_window: Option<u32>,
    pub pricing: Option<ModelPricing>,
    pub metadata: ModelMetadata,
}

/// Model capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub embedding: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub json_mode: bool,
}

/// Pricing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    pub currency: String,
}

/// Extended model metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub architecture: Option<String>,
    pub parameter_count: Option<u64>,
    pub training_cutoff: Option<String>,
    pub release_date: Option<String>,
}

impl Model {
    pub fn token_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        self.pricing.as_ref().map(|p| {
            (input_tokens as f64 * p.input_cost_per_token)
                + (output_tokens as f64 * p.output_cost_per_token)
        })
    }
}
```

#### File: `core/src/models/scan.rs`
**Keeps:** `scan_result.rs` (renamed, no merge needed)

```rust
//! Scan results and summary.

use serde::{Deserialize, Serialize};
use super::credentials::DiscoveredCredential;

/// Result of a credential scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub home_dir: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub credentials: Vec<DiscoveredCredential>,
    pub providers_found: Vec<String>,
    pub files_scanned: usize,
    pub directories_scanned: usize,
}

impl ScanResult {
    pub fn total_credentials(&self) -> usize {
        self.credentials.len()
    }
    
    pub fn summary(&self) -> ScanSummary {
        ScanSummary {
            total_credentials: self.credentials.len(),
            providers_found: self.providers_found.len(),
            files_scanned: self.files_scanned,
            directories_scanned: self.directories_scanned,
            duration_secs: self.completed_at
                .map(|c| (c - self.started_at).num_seconds())
                .unwrap_or(0),
        }
    }
}

/// Summary statistics for a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_credentials: usize,
    pub providers_found: usize,
    pub files_scanned: usize,
    pub directories_scanned: usize,
    pub duration_secs: i64,
}
```

#### File: `core/src/models/mod.rs` (updated)

```rust
//! Core data models for aicred.

pub mod providers;
pub mod credentials;
pub mod labels;
pub mod models;
pub mod scan;

// Re-export commonly used types
pub use providers::{
    Provider, ProviderInstance, ProviderCollection,
    AuthMethod, RateLimit, Capabilities,
};
pub use credentials::{
    DiscoveredCredential, CredentialValue, Confidence,
    Environment, ValidationStatus,
};
pub use labels::{
    Label, LabelAssignment, LabelTarget, LabelWithAssignments,
};
pub use models::{
    Model, ModelCapabilities, ModelPricing, ModelMetadata,
};
pub use scan::{
    ScanResult, ScanSummary,
};

// Backward compatibility (feature-gated)
#[cfg(feature = "compat_v0_1")]
pub use labels::{Tag, TagAssignment, TagAssignmentTarget};

// Remove these after migration:
// #[deprecated] pub use old_module::OldType;
```

### Step 1.3: Migration Script (Day 2-3)

Create `scripts/migrate_models.sh`:

```bash
#!/bin/bash
set -e

echo "🔄 Starting model consolidation migration..."

# Backup old files
mkdir -p core/src/models/.backup
cp core/src/models/*.rs core/src/models/.backup/

# Create new consolidated files (already done above)
echo "✅ New model files created"

# Update imports across codebase
echo "🔍 Updating imports..."

# Replace DiscoveredKey → DiscoveredCredential
rg -l "DiscoveredKey" core/ cli/ | xargs sed -i '' 's/DiscoveredKey/DiscoveredCredential/g'

# Replace ValueType → CredentialValue
rg -l "ValueType" core/ cli/ | xargs sed -i '' 's/ValueType/CredentialValue/g'

# Replace Tag → Label (except in feature-gated code)
rg -l "use.*Tag" core/ cli/ | xargs sed -i '' 's/use crate::models::Tag/use crate::models::Label/g'

# Update mod.rs imports
echo "📝 Updating module re-exports..."

# Run cargo check to find remaining issues
echo "🔍 Checking for compilation errors..."
cargo check --all-features 2>&1 | tee migration-errors.log

echo "✅ Migration script complete. Check migration-errors.log for issues."
```

Make executable:
```bash
chmod +x scripts/migrate_models.sh
```

### Step 1.4: Update Tests (Day 3-4)

Update test files to use new model names:

```bash
# Update test imports
rg -l "models::" core/tests/ | xargs sed -i '' \
  -e 's/models::discovered_key/models::credentials/g' \
  -e 's/models::tag/models::labels/g' \
  -e 's/models::provider_instance/models::providers/g'

# Run tests
cargo test --all-features
```

### Step 1.5: Delete Old Files (Day 4)

Once all tests pass:

```bash
# Delete consolidated files
rm core/src/models/discovered_key.rs
rm core/src/models/provider_key.rs
rm core/src/models/tag.rs
rm core/src/models/tag_assignment.rs
rm core/src/models/label_assignment.rs
rm core/src/models/unified_label.rs
rm core/src/models/provider.rs
rm core/src/models/provider_instance.rs
rm core/src/models/provider_instances.rs
rm core/src/models/model_metadata.rs

# Delete deprecated
rm core/src/models/provider_config.rs

# Delete config_instance.rs if unused
# (audit first to ensure it's truly redundant)

# Commit
git add -A
git commit -m "refactor: consolidate models from 18 to 6 files

- Merge providers: provider.rs + provider_instance.rs + provider_instances.rs → providers.rs
- Merge credentials: discovered_key.rs + provider_key.rs → credentials.rs  
- Merge labels: tag.rs + label.rs + assignments + unified → labels.rs
- Merge models: model.rs + model_metadata.rs → models.rs
- Rename scan_result.rs → scan.rs
- Delete deprecated provider_config.rs
- Add backward compatibility via feature flags"
```

### Step 1.6: Validation (Day 5)

```bash
# Full test suite
cargo test --all-features

# Check no regressions in benchmarks
cargo bench --bench scan_benchmark

# Verify public API matches baseline (with known changes)
cargo doc --no-deps
diff docs/public-api-baseline.txt <(rg "^pub use" core/src/lib.rs)

# Check clippy (may still have allows, we'll fix in Phase 4)
cargo clippy --all-features

# Integration test with CLI
cargo build --release
./target/release/aicred scan --format json
```

**Success Criteria:**
- [ ] All tests pass
- [ ] Benchmarks within 5% of baseline
- [ ] Public API changes documented
- [ ] 18 model files → 6 files
- [ ] Zero deprecated models in codebase
- [ ] Feature flag `compat_v0_1` provides backward compatibility

---

## Phase 2: Scanner Simplification

**Duration:** 4-5 days  
**Goal:** Single discovery architecture, eliminate scanner/ module

### Step 2.1: Create New Discovery Module (Day 1)

```bash
mkdir core/src/discovery
touch core/src/discovery/mod.rs
touch core/src/discovery/engine.rs
touch core/src/discovery/base.rs
```

#### File: `core/src/discovery/engine.rs`

```rust
//! Discovery engine for finding AI credentials across applications.

use crate::error::Result;
use crate::models::{ScanResult, ProviderInstance};
use crate::plugins::ProviderPlugin;
use std::collections::HashMap;
use std::path::Path;

pub type ProviderRegistry = HashMap<String, Box<dyn ProviderPlugin>>;

/// Discovery engine for scanning system for AI credentials.
pub struct DiscoveryEngine {
    providers: ProviderRegistry,
    app_scanners: Vec<Box<dyn AppScanner>>,
}

impl DiscoveryEngine {
    pub fn new(providers: ProviderRegistry) -> Self {
        Self {
            providers,
            app_scanners: Vec::new(),
        }
    }
    
    pub fn register_scanner(&mut self, scanner: Box<dyn AppScanner>) {
        self.app_scanners.push(scanner);
    }
    
    pub fn scan(&self, home_dir: &Path, options: &ScanOptions) -> Result<ScanResult> {
        let mut result = ScanResult::new(home_dir.display().to_string());
        
        for scanner in &self.app_scanners {
            if let Ok(discoveries) = scanner.scan(home_dir, options) {
                result.merge(discoveries);
            }
        }
        
        Ok(result)
    }
}

/// Trait for application-specific scanners.
pub trait AppScanner: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn scan(&self, home_dir: &Path, options: &ScanOptions) -> Result<Vec<DiscoveredCredential>>;
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub include_full_values: bool,
    pub max_file_size: usize,
    pub only_providers: Option<Vec<String>>,
    pub exclude_providers: Option<Vec<String>>,
    pub probe_models: bool,
    pub probe_timeout_secs: u64,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_full_values: false,
            max_file_size: 1024 * 1024,
            only_providers: None,
            exclude_providers: None,
            probe_models: false,
            probe_timeout_secs: 30,
        }
    }
}
```

### Step 2.2: Extract Common Scanner Base (Day 2)

Analyze existing scanners for patterns:

```bash
# Find duplicated code
cd core/src/scanners
wc -l *.rs
# claude_desktop.rs (619 LOC)
# roo_code.rs (869 LOC)  
# gsh.rs (988 LOC)
# All follow similar pattern
```

Create base implementation:

#### File: `core/src/discovery/base.rs`

```rust
//! Common base functionality for application scanners.

use crate::error::{Error, Result};
use crate::models::credentials::DiscoveredCredential;
use std::path::{Path, PathBuf};

/// Base scanner with common file operations.
pub struct BaseScanner {
    pub name: String,
    pub config_paths: Vec<PathBuf>,
}

impl BaseScanner {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config_paths: Vec::new(),
        }
    }
    
    pub fn add_config_path(&mut self, path: PathBuf) {
        self.config_paths.push(path);
    }
    
    /// Find config files that exist
    pub fn find_configs(&self, home_dir: &Path) -> Vec<PathBuf> {
        self.config_paths
            .iter()
            .filter_map(|p| {
                let full_path = home_dir.join(p);
                if full_path.exists() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Parse JSON config file
    pub fn parse_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &Path
    ) -> Result<T> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(e))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::ParseError(format!("JSON parse error: {}", e)))
    }
    
    /// Parse YAML config file
    pub fn parse_yaml<T: serde::de::DeserializeOwned>(
        &self,
        path: &Path
    ) -> Result<T> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| Error::ParseError(format!("YAML parse error: {}", e)))
    }
}

/// Helper to extract credentials from parsed config
pub fn extract_credentials_from_map(
    map: &serde_json::Value,
    provider_hint: &str,
    source_file: &Path,
) -> Vec<DiscoveredCredential> {
    let mut creds = Vec::new();
    
    if let Some(obj) = map.as_object() {
        for (key, value) in obj {
            if key.to_lowercase().contains("key") || key.to_lowercase().contains("token") {
                if let Some(val_str) = value.as_str() {
                    if !val_str.is_empty() {
                        creds.push(DiscoveredCredential {
                            provider: provider_hint.to_string(),
                            value: CredentialValue::redact(val_str),
                            confidence: Confidence::High,
                            source_file: source_file.display().to_string(),
                            source_line: None,
                            environment: Environment::UserConfig,
                            discovered_at: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
    }
    
    creds
}
```

### Step 2.3: Refactor Existing Scanners (Day 2-3)

Refactor each scanner to use the base:

#### Example: `core/src/discovery/claude_desktop.rs`

```rust
//! Claude Desktop application scanner.

use super::base::BaseScanner;
use super::engine::{AppScanner, ScanOptions};
use crate::error::Result;
use crate::models::credentials::DiscoveredCredential;
use std::path::Path;

pub struct ClaudeDesktopScanner {
    base: BaseScanner,
}

impl ClaudeDesktopScanner {
    pub fn new() -> Self {
        let mut base = BaseScanner::new("claude_desktop");
        
        #[cfg(target_os = "macos")]
        base.add_config_path("Library/Application Support/Claude/config.json".into());
        
        #[cfg(target_os = "linux")]
        base.add_config_path(".config/Claude/config.json".into());
        
        #[cfg(target_os = "windows")]
        base.add_config_path("AppData/Roaming/Claude/config.json".into());
        
        Self { base }
    }
}

impl AppScanner for ClaudeDesktopScanner {
    fn name(&self) -> &str {
        &self.base.name
    }
    
    fn description(&self) -> &str {
        "Scans Claude Desktop application configuration"
    }
    
    fn scan(&self, home_dir: &Path, _options: &ScanOptions) -> Result<Vec<DiscoveredCredential>> {
        let mut credentials = Vec::new();
        
        for config_path in self.base.find_configs(home_dir) {
            let config: serde_json::Value = self.base.parse_json(&config_path)?;
            
            // Claude-specific extraction logic
            if let Some(api_key) = config.get("apiKey").and_then(|v| v.as_str()) {
                credentials.push(DiscoveredCredential {
                    provider: "anthropic".to_string(),
                    value: CredentialValue::redact(api_key),
                    confidence: Confidence::VeryHigh,
                    source_file: config_path.display().to_string(),
                    source_line: None,
                    environment: Environment::UserConfig,
                    discovered_at: chrono::Utc::now(),
                });
            }
        }
        
        Ok(credentials)
    }
}
```

Repeat for all scanners, extracting common patterns to `base.rs`.

### Step 2.4: Update Module Structure (Day 3)

```bash
# Move scanners to discovery
mv core/src/scanners/*.rs core/src/discovery/

# Update mod.rs
cat > core/src/discovery/mod.rs << 'EOF'
//! Discovery system for finding AI credentials.

pub mod engine;
pub mod base;

// Application-specific scanners
pub mod claude_desktop;
pub mod roo_code;
pub mod gsh;
pub mod langchain;
pub mod ragit;

pub use engine::{DiscoveryEngine, AppScanner, ScanOptions, ProviderRegistry};
pub use base::BaseScanner;

// Re-export scanners
pub use claude_desktop::ClaudeDesktopScanner;
pub use roo_code::RooCodeScanner;
pub use gsh::GshScanner;
pub use langchain::LangChainScanner;
pub use ragit::RagitScanner;

/// Register all built-in application scanners.
pub fn register_builtin_scanners() -> Vec<Box<dyn AppScanner>> {
    vec![
        Box::new(ClaudeDesktopScanner::new()),
        Box::new(RooCodeScanner::new()),
        Box::new(GshScanner::new()),
        Box::new(LangChainScanner::new()),
        Box::new(RagitScanner::new()),
    ]
}
EOF
```

### Step 2.5: Delete Old Scanner Module (Day 4)

```bash
# Remove old scanner module
rm -rf core/src/scanner/

# Remove old scanners module
rm -rf core/src/scanners/

# Update lib.rs
sed -i '' '/pub mod scanner/d' core/src/lib.rs
sed -i '' '/pub mod scanners/d' core/src/lib.rs
sed -i '' '/pub use scanner/d' core/src/lib.rs
sed -i '' '/pub use scanners/d' core/src/lib.rs

# Add new discovery module
echo 'pub mod discovery;' >> core/src/lib.rs
echo 'pub use discovery::{DiscoveryEngine, AppScanner, ScanOptions};' >> core/src/lib.rs
```

### Step 2.6: Update Core API (Day 4)

Update `core/src/lib.rs` scan function:

```rust
/// Scan for AI credentials in home directory.
pub fn scan(options: &ScanOptions) -> Result<ScanResult> {
    let home_dir = options.home_dir.as_ref()
        .map(|p| p.as_path())
        .or_else(|| dirs_next::home_dir().as_deref())
        .ok_or_else(|| Error::NotFound("Home directory not found".to_string()))?;
    
    // Initialize discovery engine
    let providers = register_builtin_providers();
    let mut engine = DiscoveryEngine::new(providers);
    
    // Register scanners
    for scanner in register_builtin_scanners() {
        engine.register_scanner(scanner);
    }
    
    // Run scan
    engine.scan(home_dir, options)
}
```

### Step 2.7: Update Tests (Day 5)

Update all test imports:

```bash
# Find tests using old scanner modules
rg -l "scanner::" core/tests/
rg -l "scanners::" core/tests/

# Update to use discovery
sed -i '' 's/use.*scanner::/use crate::discovery::/g' core/tests/*.rs
sed -i '' 's/use.*scanners::/use crate::discovery::/g' core/tests/*.rs

# Run tests
cargo test --all-features
```

**Success Criteria:**
- [ ] scanner/ module deleted
- [ ] scanners/ → discovery/
- [ ] BaseScanner reduces code duplication by 30%+
- [ ] All tests pass
- [ ] Clear single entry point (DiscoveryEngine)

---

## Phase 3: Plugin System Reduction

**Duration:** 3-4 days  
**Goal:** Simplify plugin registry from 472 LOC to ~100 LOC

### Step 3.1: Analyze Current Plugin System (Day 1)

Audit `core/src/plugins/mod.rs`:

```bash
# Count actual logic vs wrapper boilerplate
wc -l core/src/plugins/mod.rs  # 472 total

# Extract interface
rg "fn.*\(&self" core/src/plugins/mod.rs
```

**Finding:** Most methods are thin HashMap wrappers.

### Step 3.2: Create Simplified Plugin System (Day 1-2)

#### File: `core/src/plugins.rs` (flatten module)

```rust
//! Provider plugin system.

use crate::error::Result;
use crate::models::{ProviderInstance, Provider};
use std::collections::HashMap;

/// Plugin for a provider (OpenAI, Anthropic, etc.).
pub trait ProviderPlugin: Send + Sync {
    /// Provider name (e.g., "openai").
    fn name(&self) -> &'static str;
    
    /// Provider metadata.
    fn provider(&self) -> Provider;
    
    /// Calculate confidence score for a key (0.0-1.0).
    fn confidence_score(&self, key: &str) -> f32;
    
    /// Validate a provider instance configuration.
    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()>;
    
    /// Probe instance for available models (optional).
    async fn probe_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

/// Registry of provider plugins (simple HashMap).
pub type ProviderRegistry = HashMap<String, Box<dyn ProviderPlugin>>;

/// Register all built-in provider plugins.
pub fn register_builtin_providers() -> ProviderRegistry {
    let mut registry = HashMap::new();
    
    macro_rules! register {
        ($plugin:expr) => {{
            let plugin = Box::new($plugin) as Box<dyn ProviderPlugin>;
            let name = plugin.name().to_string();
            registry.insert(name, plugin);
        }};
    }
    
    register!(crate::providers::OpenAIPlugin);
    register!(crate::providers::AnthropicPlugin);
    register!(crate::providers::GroqPlugin);
    register!(crate::providers::HuggingFacePlugin);
    register!(crate::providers::OllamaPlugin);
    register!(crate::providers::OpenRouterPlugin);
    register!(crate::providers::LiteLLMPlugin);
    
    registry
}

/// Get provider from registry.
pub fn get_provider(registry: &ProviderRegistry, name: &str) -> Option<&dyn ProviderPlugin> {
    registry.get(name).map(|b| &**b)
}

/// List all provider names.
pub fn list_providers(registry: &ProviderRegistry) -> Vec<&str> {
    registry.keys().map(|s| s.as_str()).collect()
}
```

### Step 3.3: Remove CommonConfigPlugin (Day 2)

If `CommonConfigPlugin` is only used in one place:

```bash
# Find usages
rg "CommonConfigPlugin" core/

# If minimal usage, inline the functionality
# Delete the trait definition
```

### Step 3.4: Update All Usages (Day 2-3)

```bash
# Update imports throughout codebase
rg -l "PluginRegistry" core/ cli/ | xargs sed -i '' \
  's/PluginRegistry::new()/register_builtin_providers()/g'

# Update lib.rs
sed -i '' 's/pub use plugins::{.*PluginRegistry.*}/pub use plugins::{ProviderRegistry, register_builtin_providers, get_provider};/g' core/src/lib.rs

# Run tests
cargo test --all-features
```

### Step 3.5: Delete Old Plugin Module (Day 3)

```bash
# Move remaining content to flat file
rm -rf core/src/plugins/
touch core/src/plugins.rs
# (content from Step 3.2 above)

# Update mod declaration in lib.rs
sed -i '' 's/pub mod plugins;/pub mod plugins;/' core/src/lib.rs  # already flat

# Commit
git add -A
git commit -m "refactor: simplify plugin system from 472 to ~100 LOC

- Replace PluginRegistry wrapper with direct HashMap
- Flatten plugins/ module to plugins.rs
- Remove CommonConfigPlugin trait (unused)
- Use helper functions instead of methods
- 78% reduction in plugin system code"
```

**Success Criteria:**
- [ ] plugins/ module → plugins.rs
- [ ] 472 LOC → ~100 LOC (78% reduction)
- [ ] Functionality unchanged
- [ ] All tests pass

---

## Phase 4: Technical Debt Cleanup

**Duration:** 5-7 days  
**Goal:** Zero clippy warnings, clean async, no dead code

### Step 4.1: Async Cleanup (Day 1-2)

#### Identify Sync vs Async Operations

```bash
# Find async functions
rg "async fn" core/src/

# Find actual .await calls
rg "\.await" core/src/
```

**Strategy:**
1. Model probing = truly async (network I/O)
2. Validation = sync (no I/O)
3. File parsing = sync

#### Refactor Provider Plugins

Make validation sync, probing async:

```rust
pub trait ProviderPlugin: Send + Sync {
    // Sync validation (no network)
    fn validate_instance(&self, instance: &ProviderInstance) -> Result<()>;
    
    // Async model probing (network I/O)
    async fn probe_models(&self, instance: &ProviderInstance) -> Result<Vec<String>>;
}
```

Remove `async-trait` dependency:

```toml
# In Cargo.toml, remove:
# async-trait = "0.1"
```

Use native async trait (Rust 1.75+):

```rust
// No macro needed anymore
pub trait ProviderPlugin: Send + Sync {
    async fn probe_models(&self, instance: &ProviderInstance) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}
```

#### Reduce Tokio Features

```toml
# Before:
tokio = { version = "1.0", features = ["full"] }

# After:
tokio = { version = "1.0", features = ["rt", "net", "time"] }
```

### Step 4.2: Remove ALL Clippy Allows (Day 2-5)

Work through each allow systematically:

#### 4.2.1: Remove `#![allow(clippy::option_if_let_else)]`

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
// or
let value = opt.unwrap_or_else(|| compute_default());
```

**Fix script:**
```bash
# Find instances
rg "if let Some.*else" core/src/ -A 5

# Refactor each to use combinator methods
# (manual review required)
```

#### 4.2.2: Remove `#![allow(clippy::struct_excessive_bools)]`

**Before:**
```rust
pub struct Config {
    pub enable_chat: bool,
    pub enable_completion: bool,
    pub enable_embedding: bool,
    pub enable_images: bool,
}
```

**After - Use Builder Pattern:**
```rust
#[derive(Default)]
pub struct ConfigBuilder {
    capabilities: HashSet<Capability>,
}

pub enum Capability {
    Chat,
    Completion,
    Embedding,
    Images,
}

impl ConfigBuilder {
    pub fn with_chat(mut self) -> Self {
        self.capabilities.insert(Capability::Chat);
        self
    }
    
    pub fn build(self) -> Config {
        Config {
            capabilities: self.capabilities,
        }
    }
}
```

**Or - Use Bitflags:**
```rust
use bitflags::bitflags;

bitflags! {
    pub struct Capabilities: u32 {
        const CHAT = 0b0001;
        const COMPLETION = 0b0010;
        const EMBEDDING = 0b0100;
        const IMAGES = 0b1000;
    }
}
```

#### 4.2.3: Remove `#![allow(clippy::too_many_lines)]`

Break large functions (>100 lines) into smaller helpers:

```rust
// Before: 300-line function
pub fn process_scan(home_dir: &Path, options: &ScanOptions) -> Result<ScanResult> {
    // ... 300 lines of logic
}

// After: Multiple focused functions
pub fn process_scan(home_dir: &Path, options: &ScanOptions) -> Result<ScanResult> {
    let configs = discover_configs(home_dir)?;
    let credentials = extract_credentials(&configs, options)?;
    let validated = validate_credentials(credentials)?;
    build_result(validated)
}

fn discover_configs(home_dir: &Path) -> Result<Vec<ConfigFile>> { ... }
fn extract_credentials(configs: &[ConfigFile], options: &ScanOptions) -> Result<Vec<Credential>> { ... }
fn validate_credentials(credentials: Vec<Credential>) -> Result<Vec<Credential>> { ... }
fn build_result(credentials: Vec<Credential>) -> Result<ScanResult> { ... }
```

#### 4.2.4: Remove Dead Code Allows

```bash
# Enable dead_code warning
sed -i '' '/#!\[allow(dead_code)\]/d' core/src/**/*.rs

# Check what's actually dead
cargo clippy -- -W dead_code 2>&1 | tee dead-code-report.txt

# For each dead function:
# 1. If truly unused → delete
# 2. If test helper → move to #[cfg(test)]
# 3. If future API → mark with #[allow(dead_code)] + comment explaining
```

#### 4.2.5: Fix Remaining Warnings

Work through remaining clippy warnings one category at a time:

```bash
# Get full report
cargo clippy --all-features 2>&1 | tee clippy-report.txt

# Fix by category
cargo clippy --all-features -- -W clippy::needless_borrow
cargo clippy --all-features -- -W clippy::implicit_clone
# ... etc
```

**Target:** Zero warnings with:
```toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

### Step 4.3: Clean Up Imports (Day 6)

```bash
# Remove unused imports
cargo clippy --fix -- -W unused_imports

# Run rustfmt to organize
cargo fmt --all

# Check for unused dependencies
cargo install cargo-udeps
cargo +nightly udeps
```

### Step 4.4: Standardize Naming (Day 7)

Create `TERMINOLOGY.md`:

```markdown
# AICred Terminology

## Canonical Terms

- **Provider** - External AI service (OpenAI, Anthropic, Groq)
- **Instance** - Configured provider endpoint with credentials
- **Credential** - API key or authentication token discovered during scan
- **Label** - Semantic tag for categorizing instances/models ("fast", "smart")
- **Model** - Specific LLM offered by a provider
- **Discovery** - Process of finding credentials in config files
- **Scanner** - Application-specific config file parser

## Avoid These Terms

- ~~Tag~~ - Use "Label" instead
- ~~Key~~ (ambiguous) - Use "Credential" or "API Key"
- ~~Config~~ (ambiguous) - Use "Instance" for provider config, "Configuration" for app settings

## Naming Conventions

- Types: `PascalCase` (e.g., `ProviderInstance`)
- Functions: `snake_case` (e.g., `validate_instance`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- Modules: `snake_case` (e.g., `discovery`)
```

Apply naming standards:

```bash
# Find inconsistent names
rg -i "providerconfig\b" core/  # Should be ProviderInstance
rg -i "keyvalue\b" core/        # Should be CredentialValue

# Bulk rename (carefully!)
# Use IDE refactoring tools or careful sed
```

**Success Criteria:**
- [ ] Zero clippy warnings at pedantic level
- [ ] async-trait removed
- [ ] Tokio features reduced to minimum
- [ ] No dead code warnings
- [ ] Consistent naming per TERMINOLOGY.md
- [ ] All imports organized

---

## Phase 5: Documentation & Polish

**Duration:** 3-4 days  
**Goal:** Complete, accurate documentation

### Step 5.1: Update Module Docs (Day 1)

Add/update module-level documentation:

```rust
//! # AICred Core
//!
//! Discovery and management of AI provider credentials.
//!
//! ## Architecture
//!
//! - **Discovery** - Scans for credentials in application configs
//! - **Providers** - Plugin system for AI services (OpenAI, Anthropic, etc.)
//! - **Models** - Core data types (providers, credentials, labels, models, scan results)
//! - **Plugins** - Provider plugin trait and registry
//!
//! ## Quick Start
//!
//! ```rust
//! use aicred_core::{scan, ScanOptions};
//!
//! let options = ScanOptions::default();
//! let result = scan(&options)?;
//! println!("Found {} credentials", result.total_credentials());
//! ```
//!
//! ## Feature Flags
//!
//! - `compat_v0_1` - Backward compatibility with 0.1.x API
```

### Step 5.2: Add Missing Error Docs (Day 1-2)

Remove `#![allow(clippy::missing_errors_doc)]` and document all Result returns:

```rust
/// Validates a provider instance configuration.
///
/// # Errors
///
/// Returns `Error::ValidationError` if:
/// - API key format is invalid
/// - Base URL is malformed
/// - Required fields are missing
pub fn validate_instance(&self, instance: &ProviderInstance) -> Result<()> {
    // ...
}
```

### Step 5.3: Update Examples (Day 2)

Update all examples in `examples/`:

```bash
ls examples/
# config_management/
# Add new examples using simplified API
```

Create `examples/basic_scan.rs`:

```rust
//! Basic scanning example using aicred.

use aicred_core::{scan, ScanOptions};

fn main() -> anyhow::Result<()> {
    // Use default options
    let options = ScanOptions::default();
    
    // Run scan
    let result = scan(&options)?;
    
    // Print results
    println!("Scan Summary:");
    println!("  Credentials found: {}", result.total_credentials());
    println!("  Providers: {:?}", result.providers_found);
    println!("  Files scanned: {}", result.files_scanned);
    
    Ok(())
}
```

### Step 5.4: Update Architecture Docs (Day 2-3)

Update `docs/architecture.md` to reflect new structure:

```markdown
# AICred Architecture (v0.2.0)

## Overview

AICred consists of four main components:

1. **Discovery Engine** - Finds credentials in application configs
2. **Provider Plugins** - Validates and probes AI services  
3. **Core Models** - Data structures (6 consolidated modules)
4. **CLI/Bindings** - User interfaces

## Module Organization

```
aicred-core/
├── discovery/     # Discovery engine + app scanners
├── providers/     # Provider plugins (OpenAI, Anthropic, etc.)
├── models/        # Core data types (6 files)
│   ├── providers.rs
│   ├── credentials.rs
│   ├── labels.rs
│   ├── models.rs
│   ├── scan.rs
│   └── mod.rs
├── plugins.rs     # Plugin trait + registry
├── parser/        # Config file parsing
├── error.rs       # Error types
├── env_resolver.rs
└── lib.rs         # Public API
```

## Data Flow

1. User calls `scan(options)`
2. `DiscoveryEngine` initialized with provider plugins
3. App scanners registered (Claude Desktop, Roo Code, etc.)
4. Each scanner searches for config files
5. Credentials extracted and validated
6. Results aggregated into `ScanResult`

[... rest of architecture doc ...]
```

### Step 5.5: Create Migration Guide (Day 3)

Create `MIGRATION_0.1_to_0.2.md`:

```markdown
# Migrating from AICred 0.1.x to 0.2.0

## Breaking Changes

### Models Consolidation

**Before (0.1.x):**
```rust
use aicred_core::models::{
    DiscoveredKey,
    Tag, TagAssignment,
    ProviderConfig,
};
```

**After (0.2.0):**
```rust
use aicred_core::models::{
    DiscoveredCredential,
    Label, LabelAssignment,
    ProviderInstance,  // ProviderConfig removed
};
```

### Scanner → Discovery

**Before (0.1.x):**
```rust
use aicred_core::{Scanner, ScannerRegistry};
let scanner = Scanner::new(registry);
```

**After (0.2.0):**
```rust
use aicred_core::{DiscoveryEngine, register_builtin_scanners};
let engine = DiscoveryEngine::new(registry);
for scanner in register_builtin_scanners() {
    engine.register_scanner(scanner);
}
```

### Plugin Registry

**Before (0.1.x):**
```rust
let mut registry = PluginRegistry::new();
registry.register(Box::new(OpenAIPlugin));
```

**After (0.2.0):**
```rust
let registry = register_builtin_providers();
// Or manually:
let mut registry = HashMap::new();
registry.insert("openai".into(), Box::new(OpenAIPlugin));
```

## Feature Flag for Compatibility

Add to `Cargo.toml` for backward compatibility:

```toml
[dependencies]
aicred-core = { version = "0.2", features = ["compat_v0_1"] }
```

This provides type aliases:
- `Tag` → `Label`
- `TagAssignment` → `LabelAssignment`
- `DiscoveredKey` → `DiscoveredCredential`

## Deprecated APIs

The following were removed in 0.2.0:
- `ProviderConfig` - Use `ProviderInstance`
- `Scanner::scan()` - Use `DiscoveryEngine::scan()`
- `ScannerRegistry` - Use `register_builtin_scanners()`

## Full Example Migration

[... provide complete before/after example ...]
```

### Step 5.6: Update README (Day 4)

Update main README.md with new architecture:

```markdown
## Architecture (v0.2.0)

AICred has been refactored for simplicity and maintainability:

- **6 core model files** (down from 18)
- **Unified "Label" system** (no tag/label confusion)
- **Discovery engine** (clear scanning architecture)
- **Simplified plugins** (direct HashMap, no wrapper classes)
- **Zero clippy warnings** (high code quality)

[... update examples to use new API ...]
```

**Success Criteria:**
- [ ] All public APIs documented
- [ ] No missing_docs warnings
- [ ] Examples updated and tested
- [ ] Migration guide complete
- [ ] Architecture docs accurate

---

## Testing Strategy

### Continuous Testing

After each phase:

```bash
# 1. Unit tests
cargo test --all-features

# 2. Integration tests
cargo test --test '*' --all-features

# 3. Doc tests
cargo test --doc

# 4. Clippy
cargo clippy --all-features -- -D warnings

# 5. Format check
cargo fmt --all -- --check

# 6. Benchmarks
cargo bench --bench scan_benchmark

# 7. Build all targets
cargo build --all-targets --all-features
```

### Regression Test Checklist

Before merging each phase:

- [ ] All pre-existing tests pass
- [ ] New tests added for refactored code
- [ ] Benchmarks within 5% of baseline
- [ ] CLI still works (`cargo run --bin aicred -- scan`)
- [ ] Python bindings build (`cd bindings/python && maturin build`)
- [ ] Go bindings build (`cd bindings/go && go build`)
- [ ] Documentation builds (`cargo doc --no-deps`)

### Property-Based Testing

Enhance `core/tests/proptests.rs`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn label_assignment_roundtrip(
        label_name in "[a-z]{3,10}",
        instance_id in "[a-z0-9\\-]{8,16}",
    ) {
        let assignment = LabelAssignment {
            label_name: label_name.clone(),
            target: LabelTarget::ProviderInstance { instance_id: instance_id.clone() },
            assigned_at: chrono::Utc::now(),
            assigned_by: None,
        };
        
        let json = serde_json::to_string(&assignment).unwrap();
        let deserialized: LabelAssignment = serde_json::from_str(&json).unwrap();
        
        assert_eq!(assignment.label_name, deserialized.label_name);
        assert_eq!(assignment.target.instance_id(), deserialized.target.instance_id());
    }
}
```

---

## Rollback Procedures

### If Phase Fails

Each phase is in its own commit:

```bash
# Rollback one phase
git reset --hard HEAD~1

# Rollback to specific phase
git reset --hard <commit-hash-before-phase>

# Keep changes but undo commit
git reset --soft HEAD~1
```

### If Critical Bug Found

```bash
# Create hotfix branch from main
git checkout main
git checkout -b hotfix/critical-issue

# Fix issue
# Test thoroughly
# Merge to main

# Then rebase cleanup branch
git checkout code-cleanup
git rebase main
```

### Testing Rollback

Before starting refactor, create rollback test:

```bash
# Save current state
git tag pre-refactor-baseline

# After each phase
git tag phase-1-complete
git tag phase-2-complete
# etc.

# Can always return to any tag
git checkout pre-refactor-baseline
```

---

## Communication Plan

### Status Updates

Post daily status to team channel:

```
🔄 AICred Refactor - Day 3/20
✅ Model consolidation: 18 → 6 files
⏳ Scanner simplification: In progress
📊 Tests passing: 42/42
🎯 Next: Finish discovery engine

Blockers: None
```

### Weekly Reviews

Every Monday, post:
- Completed phases
- Current phase progress
- Risks/blockers
- Adjusted timeline

### Documentation

Keep `REFACTOR_PLAN.md` updated with:
- Actual vs estimated time
- Challenges encountered
- Decisions made

---

## Success Metrics

### Code Quality Metrics

**Before (Baseline):**
```
Total LOC: 36,000
Model files: 18
Clippy allows: 30+
Plugin LOC: 472
Test coverage: ~75%
```

**After (Target):**
```
Total LOC: 28,000 (-22%)
Model files: 6 (-67%)
Clippy allows: 0 (-100%)
Plugin LOC: 100 (-79%)
Test coverage: >80%
```

### Performance Metrics

Benchmarks should remain within 5% of baseline:
- Scan time
- Memory usage
- Binary size

### Maintainability Metrics

- Documentation coverage: >95%
- Public API surface: <50% of original (simpler)
- Average file length: <300 LOC
- Cyclomatic complexity: <15 per function

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 0. Setup | 1 day | Baseline, tests, benchmarks |
| 1. Models | 5-7 days | 18 → 6 model files, tag/label unified |
| 2. Scanner | 4-5 days | discovery/ module, BaseScanner |
| 3. Plugin | 3-4 days | Simplified registry, 472 → 100 LOC |
| 4. Debt | 5-7 days | Zero clippy warns, async cleanup |
| 5. Docs | 3-4 days | Complete documentation |
| **Total** | **21-28 days** | **Clean, maintainable codebase** |

## Next Steps

1. Review this plan with stakeholders
2. Get approval for breaking changes (0.2.0 release)
3. Create GitHub project board with all tasks
4. Begin Phase 0 (setup)
5. Daily commits to `code-cleanup` branch
6. Weekly sync meetings

---

**Ready to proceed?** Let's start with Phase 0 setup tasks.
