//! Core data models for the aicred library.
// Allow clippy lints for the models module
#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::len_zero)]

pub mod config_instance;
pub mod config_validator;
pub mod discovered_key;
pub mod label;
pub mod label_assignment;
pub mod model;
pub mod model_metadata;
pub mod provider;
pub mod provider_config;
pub mod provider_instance;
pub mod provider_instances;
pub mod provider_key;
pub mod scan_result;
pub mod tag;
pub mod tag_assignment;
pub mod unified_label;
pub mod validation_result;

#[cfg(test)]
mod tests;

pub use config_instance::ConfigInstance;
pub use discovered_key::{Confidence, DiscoveredKey, ValueType};
pub use label::Label;
pub use label_assignment::{LabelAssignment, LabelAssignmentTarget};
pub use model::{Capabilities, Model, TokenCost};
pub use model_metadata::{ModelArchitecture, ModelMetadata, ModelPricing};
pub use provider::{AuthMethod, Provider, RateLimit};
#[deprecated(
    since = "0.1.0",
    note = "Use ProviderInstance and ProviderInstances instead"
)]
pub use provider_config::ProviderConfig;
pub use provider_instance::ProviderInstance;
pub use provider_instances::ProviderInstances;
pub use provider_key::{Environment, ProviderKey, ValidationStatus};
pub use scan_result::{ScanResult, ScanSummary};
pub use tag::Tag;
pub use tag_assignment::{TagAssignment, TagAssignmentTarget};
pub use unified_label::UnifiedLabel;
