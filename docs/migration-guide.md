# Migration Guide

## ProviderInstance API Key Refactoring (Current Development Cycle)

### Overview

The [`ProviderInstance`](core/src/models/provider_instance.rs:25) model has been refactored to use a simplified API key storage model. This is a **breaking change** accepted in the current development cycle with **no deprecation path**.

### What Changed

**Before:**
- Multiple keys stored in `keys: Vec<ProviderKey>`
- Complex multi-key management

**After:**
- Single API key stored in `api_key: Option<String>`
- Simplified key management with metadata preservation
- New helper methods for key presence checks

### New API

#### Fields
- [`api_key: Option<String>`](core/src/models/provider_instance.rs:41) - Single optional API key

#### Helper Methods
- [`has_api_key(&self) -> bool`](core/src/models/provider_instance.rs:150) - Checks if an API key is present (including empty strings)
- [`has_non_empty_api_key(&self) -> bool`](core/src/models/provider_instance.rs:156) - Checks if a non-empty API key is present
- [`set_api_key(&mut self, api_key: String)`](core/src/models/provider_instance.rs:137) - Sets the API key
- [`get_api_key(&self) -> Option<&String>`](core/src/models/provider_instance.rs:144) - Gets a reference to the API key

### Metadata Preservation

The refactoring includes **metadata-preserving conversions** between [`ProviderConfig`](core/src/models/provider_config.rs:11) (multi-key) and [`ProviderInstance`](core/src/models/provider_instance.rs:25) (single-key):

#### From ProviderConfig to ProviderInstance
When converting from [`ProviderConfig`](core/src/models/provider_config.rs:11) to [`ProviderInstance`](core/src/models/provider_instance.rs:25):
- The first valid key's value is extracted to `api_key`
- Key metadata is preserved in the instance's `metadata` HashMap:
  - `environment` - Environment type (development/staging/production/testing/custom)
  - `confidence` - Confidence level (Low/Medium/High/VeryHigh)
  - `validation_status` - Validation state (Unknown/Valid/Invalid/Expired/Revoked/RateLimited)
  - `discovered_at` - RFC3339 timestamp
  - `source` - Source file path
  - `line_number` - Line number (if available)
  - `key_metadata` - Additional JSON metadata (if available)

#### From ProviderInstance to ProviderConfig
When converting from [`ProviderInstance`](core/src/models/provider_instance.rs:25) to [`ProviderConfig`](core/src/models/provider_config.rs:11):
- The `api_key` is wrapped in a [`ProviderKey`](core/src/models/provider_key.rs:11) with ID "default"
- All preserved metadata is restored from the instance's `metadata` HashMap
- Safe defaults are used for any missing or malformed metadata fields
- Parsing errors are logged but don't fail the conversion

### Migration for Integrators

If you were using multi-key APIs:

1. **Update your code to use the single-key model:**
   ```rust
   // Old approach (no longer supported)
   // instance.keys.iter().find(|k| k.id == "production")
   
   // New approach
   if instance.has_non_empty_api_key() {
       let key = instance.get_api_key().unwrap();
       // Use the key
   }
   ```

2. **Access preserved metadata if needed:**
   ```rust
   if let Some(metadata) = &instance.metadata {
       if let Some(env) = metadata.get("environment") {
           println!("Key environment: {}", env);
       }
       if let Some(confidence) = metadata.get("confidence") {
           println!("Key confidence: {}", confidence);
       }
   }
   ```

3. **Use helper methods for key checks:**
   ```rust
   // Check for any key (including empty)
   if instance.has_api_key() {
       // Key field is present
   }
   
   // Check for non-empty key (recommended)
   if instance.has_non_empty_api_key() {
       // Key is present and not empty
   }
   ```

### No Deprecation Path

This is a **breaking change** with no deprecation path. The multi-key model has been completely replaced with the single-key model in the current development cycle. If you need multi-key support, you should:

1. Manage multiple [`ProviderInstance`](core/src/models/provider_instance.rs:25) objects (one per key)
2. Use the instance `id` field to distinguish between different keys/environments
3. Leverage the preserved metadata to track key provenance and validation status

### Backward Compatibility

The conversion traits ([`From<ProviderConfig>`](core/src/models/provider_instance.rs:227) and [`From<ProviderInstance>`](core/src/models/provider_instance.rs:293)) ensure that:
- Existing [`ProviderConfig`](core/src/models/provider_config.rs:11) data can be converted to [`ProviderInstance`](core/src/models/provider_instance.rs:25)
- Metadata is preserved during round-trip conversions
- Invalid or malformed metadata is handled gracefully with safe defaults

---

## Legacy Information

Migration functionality for older configuration formats has been removed. Invalid configurations will be automatically replaced with default settings.