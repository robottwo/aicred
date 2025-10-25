//! Core data models for the genai-keyfinder library.

pub mod config_instance;
pub mod discovered_key;
pub mod model;
pub mod provider;
pub mod provider_config;
pub mod provider_key;
pub mod scan_result;

#[cfg(test)]
mod tests;

pub use config_instance::ConfigInstance;
pub use discovered_key::{Confidence, DiscoveredKey, ValueType};
pub use model::{Capabilities, Model};
pub use provider::{AuthMethod, Provider, RateLimit};
pub use provider_config::ProviderConfig;
pub use provider_key::{Environment, ProviderKey, ValidationStatus};
pub use scan_result::{ScanResult, ScanSummary};
