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
- Comprehensive model capabilities tracking

### Changed
- **BREAKING**: `ProviderInstance.models` now `Vec<String>` instead of `Vec<Model>`
- **BREAKING**: `ProviderInstance.metadata` now `HashMap` instead of `Option<HashMap>`
- **BREAKING**: `ProviderInstance.display_name` renamed to `id`
- **BREAKING**: `Label` structure simplified (no separate `id` or `color` fields)
- **BREAKING**: `ValidationStatus::Unknown` replaced with `ValidationStatus::Pending`
- Model capabilities updated: `chat`, `completion`, `embedding`, `vision`, `json_mode`
- Pricing fields now use `input_cost_per_token` / `output_cost_per_token` (was per-million)

### Deprecated
- `provider_instance::ProviderInstance` (use `providers::ProviderInstance`)
- `provider_instances::ProviderInstances` (use `providers::ProviderCollection`)
- Legacy conversion traits between old and new types

### Removed
- Duplicate model structures across modules (77% code reduction)
- Redundant metadata handling code
- Unused conversion paths

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

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Core provider plugin system
- Multi-provider API key discovery
- Configuration management
- CLI tool for scanning and managing providers

