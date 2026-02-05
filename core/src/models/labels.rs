//! Semantic labeling system for provider:model combinations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A semantic label (e.g., "fast", "smart", "cheap").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Label {
    /// Label name (unique identifier)
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// When this label was created
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Assignment linking a label to a provider:model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelAssignment {
    /// Label name being assigned
    pub label_name: String,
    /// Target of the assignment
    pub target: LabelTarget,
    /// When this assignment was made
    pub assigned_at: DateTime<Utc>,
    /// Who made the assignment (optional)
    pub assigned_by: Option<String>,
}

/// Target of a label assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LabelTarget {
    /// Entire provider instance
    ProviderInstance {
        /// Instance ID
        instance_id: String
    },
    /// Specific model within an instance
    ProviderModel {
        /// Instance ID
        instance_id: String,
        /// Model ID
        model_id: String
    },
}

impl LabelTarget {
    /// Gets the instance ID from any target variant
    #[must_use]
    pub fn instance_id(&self) -> &str {
        match self {
            Self::ProviderInstance { instance_id } | Self::ProviderModel { instance_id, .. } => instance_id,
        }
    }
    
    /// Gets the model ID if this targets a specific model
    #[must_use]
    pub fn model_id(&self) -> Option<&str> {
        match self {
            Self::ProviderModel { model_id, .. } => Some(model_id),
            Self::ProviderInstance { .. } => None,
        }
    }
}

/// Combined view of label with its assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelWithAssignments {
    /// The label metadata
    pub label: Label,
    /// All assignments for this label
    pub assignments: Vec<LabelAssignment>,
}

impl LabelWithAssignments {
    /// Creates a new label with assignments
    #[must_use]
    pub const fn new(label: Label, assignments: Vec<LabelAssignment>) -> Self {
        Self { label, assignments }
    }
    
    /// Checks if this label has any assignments
    #[must_use]
    pub const fn has_assignments(&self) -> bool {
        !self.assignments.is_empty()
    }
    
    /// Gets the number of assignments
    #[must_use]
    pub const fn assignment_count(&self) -> usize {
        self.assignments.len()
    }
}

// =============================================================================
// Backward Compatibility Type Aliases (Feature-Gated)
// =============================================================================

#[cfg(feature = "compat_v0_1")]
/// Deprecated: Use `Label` instead
pub type Tag = Label;

#[cfg(feature = "compat_v0_1")]
/// Deprecated: Use `LabelAssignment` instead
pub type TagAssignment = LabelAssignment;

#[cfg(feature = "compat_v0_1")]
/// Deprecated: Use `LabelTarget` instead
pub type TagAssignmentTarget = LabelTarget;
