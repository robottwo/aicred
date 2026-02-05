# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-05

### Added
- New consolidated model API with cleaner type hierarchy
- `providers::ProviderInstance` - modernized provider instance type
- `providers::ProviderCollection` - type-safe provider collection
- `models::Model` - simplified model metadata structure
- `labels::Label` - streamlined label system
- `env_resolver::LabelWithTarget` - simplified label-to-provider mapping (replaces `UnifiedLabel`)
- Comprehensive model capabilities tracking

### Changed
- **BREAKING**: `UnifiedLabel` removed (use `LabelWithTarget`)
- **BREAKING**: `ProviderConfig` removed (use `ProviderInstance` directly)
- **BREAKING**: `ProviderKey` removed (use `ProviderInstance.set_api_key()`)
- **BREAKING**: `ProviderInstance.models` now `Vec<String>` instead of `Vec<Model>`
- **BREAKING**: `ProviderInstance.metadata` now `HashMap` instead of `Option<HashMap>`
- **BREAKING**: `ProviderInstance.display_name` renamed to `id`
- **BREAKING**: `Label` structure simplified (no separate `id` or `color` fields)
- **BREAKING**: `ValidationStatus::Unknown` replaced with `ValidationStatus::Pending`
- **BREAKING**: `EnvResolver` now uses `LabelWithTarget` instead of `UnifiedLabel`
- **BREAKING**: `EnvResolver::with_labels()` now takes `Vec<LabelWithTarget>`
- **BREAKING**: `EnvResolver::new()` now takes `Vec<LabelWithTarget>`
- **BREAKING**: `EnvResolver::resolve_from_mappings()` now takes `Vec<LabelWithTarget>`
- Model capabilities updated: `chat`, `completion`, `embedding`, `vision`, `json_mode`
- Pricing fields now use `input_cost_per_token` / `output_cost_per_token` (was per-million)
- CLI `labels` command now uses `load_labels_with_targets()` instead of `load_unified_labels_with_home()`

### Deprecated
- `provider_instance::ProviderInstance` (use `providers::ProviderInstance`)
- `provider_instances::ProviderInstances` (use `providers::ProviderCollection`)
- Legacy conversion traits between old and new types
- `load_unified_labels_with_home()` function (use `load_labels_with_targets()`)

### Removed
- Duplicate model structures across modules (77% code reduction)
- Redundant metadata handling code
- Unused conversion paths
- Legacy type files:
  - `core/src/models/provider_key.rs`
  - `core/src/models/provider_config.rs`
  - `core/src/models/unified_label.rs`
  - `core/src/models/discovered_key.rs`
  - `core/src/models/tag.rs`
  - `core/src/models/tag_assignment.rs`
- Test infrastructure for legacy types:
  - `core/tests/multi_key_tests.rs`
  - `core/tests/config_storage_tests.rs`
  - `core/src/models/tests.rs`
  - `cli/tests/wrap_integration_tests.rs`
  - `cli/tests/setenv_integration_tests.rs`

### Fixed
- All Clippy warnings resolved
- Type inconsistencies in plugin system
- Metadata handling edge cases

### Migration Guide
- Update `ProviderInstance` usage to new `providers::ProviderInstance`
- Change `instance.display_name` to `instance.id`
- Access models directly as strings: `instance.models[0]` instead of `instance.models[0].model_id`
- Check metadata with `!instance.metadata.is_empty()` instead of `instance.metadata.is_some()`
- Update capabilities checks to new field names
- Replace `UnifiedLabel` with `LabelWithTarget::new(label_name, target)`
- Replace `ProviderKey` usage in tests with `instance.set_api_key()`

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Core provider plugin system
- Multi-provider API key discovery
- Configuration management
- CLI tool for scanning and managing providers

