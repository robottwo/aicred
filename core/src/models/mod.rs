//! Core data models for the aicred library.
// Allow clippy lints for the models module

// ==== NEW CONSOLIDATED MODELS (v0.2.0) ====
pub mod credentials_new;
pub mod labels_new;
pub mod models_new;
pub mod providers_new;
pub mod scan_new;

// ==== OLD MODELS (v0.1.x - to be removed) ====
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

#[cfg(test)]
mod tests;

// ==== NEW API (v0.2.0) - Preferred ====
pub use credentials_new::{
    Confidence as ConfidenceNew,
    CredentialValue,
    DiscoveredCredential,
    Environment as EnvironmentNew,
    ValidationStatus as ValidationStatusNew,
    ValueType as ValueTypeNew,
};
pub use labels_new::{
    Label as LabelNew,
    LabelAssignment as LabelAssignmentNew,
    LabelTarget,
    LabelWithAssignments,
};
pub use models_new::{
    Model as ModelNew,
    ModelCapabilities,
    ModelMetadata as ModelMetadataNew,
    ModelPricing as ModelPricingNew,
    TokenCost as TokenCostNew,
};
pub use providers_new::{
    AuthMethod as AuthMethodNew,
    Capabilities as CapabilitiesNew,
    Provider as ProviderNew,
    ProviderCollection,
    ProviderInstance as ProviderInstanceNew,
    RateLimit as RateLimitNew,
};

// ==== OLD API (v0.1.x) - Deprecated, will be removed in v0.3.0 ====
#[deprecated(since = "0.2.0", note = "Use DiscoveredCredential from credentials_new")]
pub use discovered_key::{Confidence, DiscoveredKey, ValueType};

#[deprecated(since = "0.2.0", note = "Use LabelNew from labels_new - tags renamed to labels")]
pub use label::Label;

#[deprecated(since = "0.2.0", note = "Use LabelAssignmentNew from labels_new")]
pub use label_assignment::{LabelAssignment, LabelAssignmentTarget};

#[deprecated(since = "0.2.0", note = "Use ModelNew from models_new")]
pub use model::{Capabilities, Model, TokenCost};

#[deprecated(since = "0.2.0", note = "Use ModelMetadataNew from models_new")]
pub use model_metadata::{ModelArchitecture, ModelMetadata, ModelPricing};

#[deprecated(since = "0.2.0", note = "Use ProviderNew from providers_new")]
pub use provider::{AuthMethod, Provider, RateLimit};

#[deprecated(since = "0.1.0", note = "Use ProviderInstanceNew instead")]
pub use provider_config::ProviderConfig;

#[deprecated(since = "0.2.0", note = "Use ProviderInstanceNew from providers_new")]
pub use provider_instance::ProviderInstance;

#[deprecated(since = "0.2.0", note = "Use ProviderCollection from providers_new")]
pub use provider_instances::ProviderInstances;

#[deprecated(since = "0.2.0", note = "Use ValidationStatusNew from credentials_new")]
pub use provider_key::{Environment, ProviderKey, ValidationStatus};

pub use scan_result::{ScanResult, ScanSummary};

#[deprecated(since = "0.2.0", note = "Tags renamed to Labels - use LabelNew instead")]
pub use tag::Tag;

#[deprecated(since = "0.2.0", note = "Tags renamed to Labels - use LabelAssignmentNew instead")]
pub use tag_assignment::{TagAssignment, TagAssignmentTarget};

#[deprecated(since = "0.2.0", note = "Use LabelWithAssignments instead")]
pub use unified_label::UnifiedLabel;

// For backwards compatibility without deprecation warnings
pub use config_instance::ConfigInstance;
