//! Core data models for the aicred library.

// ==== CONSOLIDATED MODELS (v0.2.0) ====
pub mod credentials;
pub mod labels;
pub mod models;
pub mod providers;
pub mod scan;

// ==== SPECIALIZED MODELS ====
pub mod config_instance;
pub mod config_validator;

// ==== OLD MODELS (v0.1.x - DEPRECATED, kept for backward compatibility) ====
#[allow(deprecated)]
pub mod discovered_key;
#[allow(deprecated)]
pub mod label;
#[allow(deprecated)]
pub mod label_assignment;
#[allow(deprecated)]
pub mod model;
#[allow(deprecated)]
pub mod model_metadata;
#[allow(deprecated)]
pub mod provider;
#[allow(deprecated)]
pub mod provider_config;
#[allow(deprecated)]
pub mod provider_instance;
#[allow(deprecated)]
pub mod provider_instances;
#[allow(deprecated)]
pub mod provider_key;
#[allow(deprecated)]
pub mod scan_result;
#[allow(deprecated)]
pub mod tag;
#[allow(deprecated)]
pub mod tag_assignment;
#[allow(deprecated)]
pub mod unified_label;

#[cfg(test)]
mod tests;

// ==== PRIMARY API (v0.2.0) ====

// Credentials & Discovery
pub use credentials::{
    Confidence,
    CredentialValue,
    DiscoveredCredential,
    Environment,
    ValidationStatus,
    ValueType,
};

// Labels (semantic tagging)
pub use labels::{
    Label,
    LabelAssignment,
    LabelTarget,
    LabelWithAssignments,
};

// Models & Metadata
pub use models::{
    Model,
    ModelCapabilities,
    ModelMetadata,
    ModelPricing,
    TokenCost,
};

// Providers & Instances
pub use providers::{
    AuthMethod,
    Capabilities,
    Provider,
    ProviderCollection,
    ProviderInstance,
    RateLimit,
};

// Scan Results
pub use scan::{ScanResult, ScanSummary};

// Config Instance (not deprecated)
pub use config_instance::ConfigInstance;

// ==== BACKWARD COMPATIBILITY ALIASES (v0.1.x) ====
// These type aliases allow old code to continue working.
// They point to the new types via type aliases.

// Re-export old module types for external backward compatibility
#[allow(deprecated)]
pub use discovered_key::{
    Confidence as ConfidenceOld,
    DiscoveredKey,
    ValueType as ValueTypeOld,
};

#[allow(deprecated)]
pub use label::Label as LabelOld;

#[allow(deprecated)]
pub use label_assignment::{
    LabelAssignment as LabelAssignmentOld,
    LabelAssignmentTarget,
};

#[allow(deprecated)]
pub use model::{
    Capabilities as CapabilitiesOld,
    Model as ModelOld,
    TokenCost as TokenCostOld,
};

#[allow(deprecated)]
pub use model_metadata::{
    ModelArchitecture,
    ModelMetadata as ModelMetadataOld,
    ModelPricing as ModelPricingOld,
};

#[allow(deprecated)]
pub use provider::{
    AuthMethod as AuthMethodOld,
    Provider as ProviderOld,
    RateLimit as RateLimitOld,
};

#[allow(deprecated)]
pub use provider_config::ProviderConfig;

#[allow(deprecated)]
pub use provider_instance::ProviderInstance as ProviderInstanceOld;

#[allow(deprecated)]
pub use provider_instances::ProviderInstances;

#[allow(deprecated)]
pub use provider_key::{
    Environment as EnvironmentOld,
    ProviderKey,
    ValidationStatus as ValidationStatusOld,
};

#[allow(deprecated)]
pub use scan_result::{
    ScanResult as ScanResultOld,
    ScanSummary as ScanSummaryOld,
};

#[allow(deprecated)]
pub use tag::Tag;

#[allow(deprecated)]
pub use tag_assignment::{
    TagAssignment,
    TagAssignmentTarget,
};

#[allow(deprecated)]
pub use unified_label::UnifiedLabel;
