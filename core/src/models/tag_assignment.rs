//! Tag assignment model for linking tags to provider instances or models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the target of a tag assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TagAssignmentTarget {
    /// Tag is assigned to a provider instance.
    ProviderInstance {
        /// The provider instance ID
        instance_id: String,
    },
    /// Tag is assigned to a specific model within a provider instance.
    Model {
        /// The provider instance ID
        instance_id: String,
        /// The model ID
        model_id: String,
    },
}

impl TagAssignmentTarget {
    /// Gets the instance ID for this target.
    #[must_use]
    pub fn instance_id(&self) -> &str {
        match self {
            Self::ProviderInstance { instance_id } | Self::Model { instance_id, .. } => instance_id,
        }
    }

    /// Gets the model ID if this is a model assignment.
    #[must_use]
    pub fn model_id(&self) -> Option<&str> {
        match self {
            Self::ProviderInstance { .. } => None,
            Self::Model { model_id, .. } => Some(model_id),
        }
    }

    /// Checks if this target matches the given instance and optional model.
    #[must_use]
    pub fn matches(&self, instance_id: &str, model_id: Option<&str>) -> bool {
        match (self, model_id) {
            (
                Self::ProviderInstance {
                    instance_id: target_instance,
                },
                None,
            ) => target_instance == instance_id,
            (
                Self::Model {
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
            Self::ProviderInstance { instance_id } => {
                format!("provider instance '{instance_id}'")
            }
            Self::Model {
                instance_id,
                model_id,
            } => {
                format!("model '{model_id}' in provider instance '{instance_id}'")
            }
        }
    }
}

/// A tag assignment links a tag to a specific target (provider instance or model).
/// Tags can be assigned to multiple targets, so assignments are not unique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagAssignment {
    /// Unique identifier for this assignment.
    pub id: String,

    /// The tag being assigned.
    pub tag_id: String,

    /// The target to which the tag is assigned.
    pub target: TagAssignmentTarget,

    /// Optional metadata for this specific assignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// When this assignment was created.
    pub created_at: DateTime<Utc>,

    /// When this assignment was last updated.
    pub updated_at: DateTime<Utc>,
}

impl TagAssignment {
    /// Creates a new tag assignment to a provider instance.
    #[must_use]
    pub fn new_to_instance(id: String, tag_id: String, instance_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            tag_id,
            target: TagAssignmentTarget::ProviderInstance { instance_id },
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new tag assignment to a specific model.
    #[must_use]
    pub fn new_to_model(id: String, tag_id: String, instance_id: String, model_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            tag_id,
            target: TagAssignmentTarget::Model {
                instance_id,
                model_id,
            },
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new tag assignment with metadata.
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

    /// Validates the tag assignment.
    ///
    /// # Errors
    /// Returns an error if the assignment ID or tag ID is empty.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Assignment ID cannot be empty".to_string());
        }
        if self.tag_id.is_empty() {
            return Err("Tag ID cannot be empty".to_string());
        }

        // Validate target
        match &self.target {
            TagAssignmentTarget::ProviderInstance { instance_id } => {
                if instance_id.is_empty() {
                    return Err("Instance ID cannot be empty".to_string());
                }
            }
            TagAssignmentTarget::Model {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_assignment_to_instance() {
        let assignment = TagAssignment::new_to_instance(
            "assignment-1".to_string(),
            "tag-1".to_string(),
            "instance-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-1");
        assert_eq!(assignment.tag_id, "tag-1");
        assert!(assignment.targets_instance("instance-1"));
        assert!(!assignment.targets_instance("instance-2"));
        assert!(assignment.metadata.is_none());
    }

    #[test]
    fn test_tag_assignment_to_model() {
        let assignment = TagAssignment::new_to_model(
            "assignment-2".to_string(),
            "tag-2".to_string(),
            "instance-1".to_string(),
            "model-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-2");
        assert_eq!(assignment.tag_id, "tag-2");
        assert!(assignment.targets_instance("instance-1"));
        assert!(assignment.targets_model("instance-1", "model-1"));
        assert!(!assignment.targets_model("instance-1", "model-2"));
        assert!(!assignment.targets_model("instance-2", "model-1"));
    }

    #[test]
    fn test_tag_assignment_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("assigned_by".to_string(), "admin".to_string());
        metadata.insert("reason".to_string(), "production deployment".to_string());

        let assignment = TagAssignment::new_to_instance(
            "assignment-3".to_string(),
            "tag-3".to_string(),
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
    fn test_assignment_target_methods() {
        let instance_target = TagAssignmentTarget::ProviderInstance {
            instance_id: "instance-1".to_string(),
        };
        assert_eq!(instance_target.instance_id(), "instance-1");
        assert_eq!(instance_target.model_id(), None);
        assert!(instance_target.matches("instance-1", None));
        assert!(!instance_target.matches("instance-1", Some("model-1")));
        assert!(!instance_target.matches("instance-2", None));

        let model_target = TagAssignmentTarget::Model {
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
        let instance_target = TagAssignmentTarget::ProviderInstance {
            instance_id: "openai-prod".to_string(),
        };
        assert_eq!(
            instance_target.description(),
            "provider instance 'openai-prod'"
        );

        let model_target = TagAssignmentTarget::Model {
            instance_id: "openai-prod".to_string(),
            model_id: "gpt-4".to_string(),
        };
        assert_eq!(
            model_target.description(),
            "model 'gpt-4' in provider instance 'openai-prod'"
        );
    }

    #[test]
    fn test_tag_assignment_validation() {
        let valid_assignment = TagAssignment::new_to_instance(
            "valid-assignment".to_string(),
            "valid-tag".to_string(),
            "valid-instance".to_string(),
        );
        assert!(valid_assignment.validate().is_ok());

        let empty_id_assignment = TagAssignment::new_to_instance(
            String::new(),
            "valid-tag".to_string(),
            "valid-instance".to_string(),
        );
        assert!(empty_id_assignment.validate().is_err());

        let empty_tag_id_assignment = TagAssignment::new_to_instance(
            "valid-id".to_string(),
            String::new(),
            "valid-instance".to_string(),
        );
        assert!(empty_tag_id_assignment.validate().is_err());

        let empty_instance_assignment = TagAssignment::new_to_instance(
            "valid-id".to_string(),
            "valid-tag".to_string(),
            String::new(),
        );
        assert!(empty_instance_assignment.validate().is_err());

        let empty_model_assignment = TagAssignment::new_to_model(
            "valid-id".to_string(),
            "valid-tag".to_string(),
            "valid-instance".to_string(),
            String::new(),
        );
        assert!(empty_model_assignment.validate().is_err());
    }
}
