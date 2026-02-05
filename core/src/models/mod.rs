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

// ==== LEGACY SUPPORT (still used internally) ====
pub mod discovered_key;  // Used by discovery modules
pub mod provider_config; // Used by conversions
pub mod provider_instance; // Used by old code
pub mod provider_instances; // Used by old code
pub mod provider_key;
pub mod tag;
pub mod tag_assignment;
pub mod unified_label; // Used by label system

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

// Config Instance
pub use config_instance::ConfigInstance;

// Legacy support (still used internally)
pub use discovered_key::DiscoveredKey;
pub use provider_config::ProviderConfig;
pub use provider_instance::ProviderInstance as ProviderInstanceOld;
pub use provider_instances::ProviderInstances;
pub use provider_key::ProviderKey;
pub use unified_label::UnifiedLabel;

// Tags (still used)
pub use tag::Tag;
pub use tag_assignment::{TagAssignment, TagAssignmentTarget};
