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
