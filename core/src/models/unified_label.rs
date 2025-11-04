use crate::utils::ProviderModelTuple;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unified label that combines label metadata and assignment information.
/// Labels only exist when they are assigned to a provider:model tuple.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedLabel {
    /// Human-readable name for the label (used as both display name and identifier)
    pub label_name: String,

    /// The provider:model tuple this label is assigned to
    pub target: ProviderModelTuple,

    /// Optional description of what this label represents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional color for UI display purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Additional metadata for this label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// When this label assignment was created
    pub created_at: DateTime<Utc>,

    /// When this label assignment was last updated
    pub updated_at: DateTime<Utc>,
}

impl UnifiedLabel {
    /// Creates a new unified label assignment
    pub fn new(label_name: String, target: ProviderModelTuple) -> Self {
        let now = Utc::now();
        Self {
            label_name,
            target,
            description: None,
            color: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new unified label with a description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new unified label with a color
    #[must_use]
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new unified label with additional metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        self
    }

    /// Updates the target provider:model tuple
    #[must_use]
    pub fn with_target(mut self, target: ProviderModelTuple) -> Self {
        self.target = target;
        self.updated_at = Utc::now();
        self
    }
}
