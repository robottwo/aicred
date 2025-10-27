//! Core data models for the genai-keyfinder library.

pub mod config_instance;
pub mod discovered_key;
pub mod migration;
pub mod model;
pub mod provider;
pub mod provider_config;
pub mod provider_instance;
pub mod provider_instances;
pub mod provider_key;
pub mod scan_result;

#[cfg(test)]
mod tests;

pub use config_instance::ConfigInstance;
pub use discovered_key::{Confidence, DiscoveredKey, ValueType};
pub use migration::{MigrationConfig, MigrationResult, ProviderConfigMigrator};
pub use model::{Capabilities, Model, TokenCost};
pub use provider::{AuthMethod, Provider, RateLimit};
#[deprecated(since = "4.0.0", note = "Use ProviderInstance and ProviderInstances instead")]
pub use provider_config::ProviderConfig;
pub use provider_instance::ProviderInstance;
pub use provider_instances::ProviderInstances;
pub use provider_key::{Environment, ProviderKey, ValidationStatus};
pub use scan_result::{ScanResult, ScanSummary};
