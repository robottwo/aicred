//! Label assignment model for linking labels to provider instances or models with uniqueness constraints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the target of a label assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LabelAssignmentTarget {
    /// Label is assigned to a provider instance.
    ProviderInstance { instance_id: String },
    /// Label is assigned to a specific model within a provider instance.
    Model {
        instance_id: String,
        model_id: String,
    },
}

impl LabelAssignmentTarget {
    /// Gets the instance ID for this target.
    #[must_use]
    pub fn instance_id(&self) -> &str {
        match self {
            LabelAssignmentTarget::ProviderInstance { instance_id } => instance_id,
            LabelAssignmentTarget::Model { instance_id, .. } => instance_id,
        }
    }

    /// Gets the model ID if this is a model assignment.
    #[must_use]
    pub fn model_id(&self) -> Option<&str> {
        match self {
            LabelAssignmentTarget::ProviderInstance { .. } => None,
            LabelAssignmentTarget::Model { model_id, .. } => Some(model_id),
        }
    }

    /// Checks if this target matches the given instance and optional model.
    #[must_use]
    pub fn matches(&self, instance_id: &str, model_id: Option<&str>) -> bool {
        match (self, model_id) {
            (
                LabelAssignmentTarget::ProviderInstance {
                    instance_id: target_instance,
                },
                None,
            ) => target_instance == instance_id,
            (
                LabelAssignmentTarget::Model {
                    instance_id: target_instance,
                    model_id: target_model,
                },
                Some(model),
            ) => target_instance == instance_id && target_model == model,
            _ => false,
        }
    }

    /// Gets a human-readable description of this target.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            LabelAssignmentTarget::ProviderInstance { instance_id } => {
                format!("provider instance '{}'", instance_id)
            }
            LabelAssignmentTarget::Model {
                instance_id,
                model_id,
            } => {
                format!(
                    "model '{}' in provider instance '{}'",
                    model_id, instance_id
                )
            }
        }
    }
}

/// A label assignment links a label to a specific target (provider instance or model).
/// Labels are unique across all targets, so only one assignment can exist per label.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LabelAssignment {
    /// Unique identifier for this assignment.
    pub id: String,

    /// The label being assigned.
    pub label_id: String,

    /// The target to which the label is assigned.
    pub target: LabelAssignmentTarget,

    /// Optional metadata for this specific assignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// When this assignment was created.
    pub created_at: DateTime<Utc>,

    /// When this assignment was last updated.
    pub updated_at: DateTime<Utc>,
}

impl LabelAssignment {
    /// Creates a new label assignment to a provider instance.
    #[must_use]
    pub fn new_to_instance(id: String, label_id: String, instance_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            label_id,
            target: LabelAssignmentTarget::ProviderInstance { instance_id },
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new label assignment to a specific model.
    #[must_use]
    pub fn new_to_model(
        id: String,
        label_id: String,
        instance_id: String,
        model_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            label_id,
            target: LabelAssignmentTarget::Model {
                instance_id,
                model_id,
            },
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new label assignment with metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        self
    }

    /// Sets metadata for this assignment.
    pub fn set_metadata(&mut self, metadata: Option<HashMap<String, String>>) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// Gets metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.as_ref().and_then(|meta| meta.get(key))
    }

    /// Validates the label assignment.
    ///
    /// # Errors
    /// Returns an error if the assignment ID or label ID is empty.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Assignment ID cannot be empty".to_string());
        }
        if self.label_id.is_empty() {
            return Err("Label ID cannot be empty".to_string());
        }

        // Validate target
        match &self.target {
            LabelAssignmentTarget::ProviderInstance { instance_id } => {
                if instance_id.is_empty() {
                    return Err("Instance ID cannot be empty".to_string());
                }
            }
            LabelAssignmentTarget::Model {
                instance_id,
                model_id,
            } => {
                if instance_id.is_empty() {
                    return Err("Instance ID cannot be empty".to_string());
                }
                if model_id.is_empty() {
                    return Err("Model ID cannot be empty".to_string());
                }
            }
        }

        Ok(())
    }

    /// Checks if this assignment targets the given instance.
    #[must_use]
    pub fn targets_instance(&self, instance_id: &str) -> bool {
        self.target.instance_id() == instance_id
    }

    /// Checks if this assignment targets the given model.
    #[must_use]
    pub fn targets_model(&self, instance_id: &str, model_id: &str) -> bool {
        self.target.matches(instance_id, Some(model_id))
    }

    /// Gets the target description for logging/debugging.
    #[must_use]
    pub fn target_description(&self) -> String {
        self.target.description()
    }

    /// Gets the uniqueness key for this label assignment.
    /// Since labels are globally unique, this returns the label_id.
    #[must_use]
    pub fn uniqueness_key(&self) -> &str {
        &self.label_id
    }

    /// Checks if this assignment conflicts with another label assignment.
    /// Label assignments conflict if they have the same label_id.
    #[must_use]
    pub fn conflicts_with(&self, other: &LabelAssignment) -> bool {
        self.label_id == other.label_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_assignment_to_instance() {
        let assignment = LabelAssignment::new_to_instance(
            "assignment-1".to_string(),
            "label-1".to_string(),
            "instance-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-1");
        assert_eq!(assignment.label_id, "label-1");
        assert!(assignment.targets_instance("instance-1"));
        assert!(!assignment.targets_instance("instance-2"));
        assert!(assignment.metadata.is_none());
        assert_eq!(assignment.uniqueness_key(), "label-1");
    }

    #[test]
    fn test_label_assignment_to_model() {
        let assignment = LabelAssignment::new_to_model(
            "assignment-2".to_string(),
            "label-2".to_string(),
            "instance-1".to_string(),
            "model-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-2");
        assert_eq!(assignment.label_id, "label-2");
        assert!(assignment.targets_instance("instance-1"));
        assert!(assignment.targets_model("instance-1", "model-1"));
        assert!(!assignment.targets_model("instance-1", "model-2"));
        assert!(!assignment.targets_model("instance-2", "model-1"));
        assert_eq!(assignment.uniqueness_key(), "label-2");
    }

    #[test]
    fn test_label_assignment_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("assigned_by".to_string(), "admin".to_string());
        metadata.insert("reason".to_string(), "production deployment".to_string());

        let assignment = LabelAssignment::new_to_instance(
            "assignment-3".to_string(),
            "label-3".to_string(),
            "instance-2".to_string(),
        )
        .with_metadata(metadata);

        assert_eq!(
            assignment.get_metadata("assigned_by"),
            Some(&"admin".to_string())
        );
        assert_eq!(
            assignment.get_metadata("reason"),
            Some(&"production deployment".to_string())
        );
        assert_eq!(assignment.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_label_assignment_conflicts() {
        let assignment1 = LabelAssignment::new_to_instance(
            "assignment-1".to_string(),
            "unique-label".to_string(),
            "instance-1".to_string(),
        );

        let assignment2 = LabelAssignment::new_to_model(
            "assignment-2".to_string(),
            "unique-label".to_string(),
            "instance-2".to_string(),
            "model-1".to_string(),
        );

        let assignment3 = LabelAssignment::new_to_instance(
            "assignment-3".to_string(),
            "different-label".to_string(),
            "instance-1".to_string(),
        );

        // Same label_id should conflict
        assert!(assignment1.conflicts_with(&assignment2));
        assert!(assignment2.conflicts_with(&assignment1));

        // Different label_id should not conflict
        assert!(!assignment1.conflicts_with(&assignment3));
        assert!(!assignment3.conflicts_with(&assignment1));
    }

    #[test]
    fn test_assignment_target_methods() {
        let instance_target = LabelAssignmentTarget::ProviderInstance {
            instance_id: "instance-1".to_string(),
        };
        assert_eq!(instance_target.instance_id(), "instance-1");
        assert_eq!(instance_target.model_id(), None);
        assert!(instance_target.matches("instance-1", None));
        assert!(!instance_target.matches("instance-1", Some("model-1")));
        assert!(!instance_target.matches("instance-2", None));

        let model_target = LabelAssignmentTarget::Model {
            instance_id: "instance-1".to_string(),
            model_id: "model-1".to_string(),
        };
        assert_eq!(model_target.instance_id(), "instance-1");
        assert_eq!(model_target.model_id(), Some("model-1"));
        assert!(!model_target.matches("instance-1", None));
        assert!(model_target.matches("instance-1", Some("model-1")));
        assert!(!model_target.matches("instance-1", Some("model-2")));
        assert!(!model_target.matches("instance-2", Some("model-1")));
    }

    #[test]
    fn test_assignment_target_description() {
        let instance_target = LabelAssignmentTarget::ProviderInstance {
            instance_id: "openai-prod".to_string(),
        };
        assert_eq!(
            instance_target.description(),
            "provider instance 'openai-prod'"
        );

        let model_target = LabelAssignmentTarget::Model {
            instance_id: "openai-prod".to_string(),
            model_id: "gpt-4".to_string(),
        };
        assert_eq!(
            model_target.description(),
            "model 'gpt-4' in provider instance 'openai-prod'"
        );
    }

    #[test]
    fn test_label_assignment_validation() {
        let valid_assignment = LabelAssignment::new_to_instance(
            "valid-assignment".to_string(),
            "valid-label".to_string(),
            "valid-instance".to_string(),
        );
        assert!(valid_assignment.validate().is_ok());

        let empty_id_assignment = LabelAssignment::new_to_instance(
            String::new(),
            "valid-label".to_string(),
            "valid-instance".to_string(),
        );
        assert!(empty_id_assignment.validate().is_err());

        let empty_label_id_assignment = LabelAssignment::new_to_instance(
            "valid-id".to_string(),
            String::new(),
            "valid-instance".to_string(),
        );
        assert!(empty_label_id_assignment.validate().is_err());

        let empty_instance_assignment = LabelAssignment::new_to_instance(
            "valid-id".to_string(),
            "valid-label".to_string(),
            String::new(),
        );
        assert!(empty_instance_assignment.validate().is_err());

        let empty_model_assignment = LabelAssignment::new_to_model(
            "valid-id".to_string(),
            "valid-label".to_string(),
            "valid-instance".to_string(),
            String::new(),
        );
        assert!(empty_model_assignment.validate().is_err());
    }
}
