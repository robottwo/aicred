//! Label assignment model for linking labels to provider instances or models with uniqueness constraints.

use crate::utils::ProviderModelTuple;
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
    /// Label is assigned to a specific provider:model tuple.
    ProviderModelTuple { tuple: ProviderModelTuple },
}

impl LabelAssignmentTarget {
    /// Gets the instance ID for this target.
    #[must_use]
    pub fn instance_id(&self) -> &str {
        match self {
            LabelAssignmentTarget::ProviderInstance { instance_id } => instance_id,
            LabelAssignmentTarget::Model { instance_id, .. } => instance_id,
            LabelAssignmentTarget::ProviderModelTuple { tuple } => &tuple.provider,
        }
    }

    /// Gets the model ID if this is a model assignment.
    #[must_use]
    pub fn model_id(&self) -> Option<&str> {
        match self {
            LabelAssignmentTarget::ProviderInstance { .. } => None,
            LabelAssignmentTarget::Model { model_id, .. } => Some(model_id),
            LabelAssignmentTarget::ProviderModelTuple { tuple } => Some(&tuple.model),
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
            (LabelAssignmentTarget::ProviderModelTuple { tuple }, target_model_id) => {
                // For provider:model tuple matching, we need to check both provider and model
                // The tuple provider should match the instance_id, and tuple model should match the basename of the full model ID
                match target_model_id {
                    None => tuple.provider() == instance_id, // Provider-level match
                    Some(model) => {
                        // Check provider match
                        if tuple.provider() != instance_id {
                            return false;
                        }

                        // For model matching, check if there's a provider prefix
                        if let Some(first_slash_pos) = model.find('/') {
                            // Model has a provider prefix like "openai/gpt-4" or "claude/deepseek-v3.2-exp"
                            let model_provider = &model[..first_slash_pos];
                            let model_basename = &model[first_slash_pos + 1..];

                            // The provider prefix must match our tuple provider
                            if model_provider != tuple.provider() {
                                return false;
                            }

                            // The basename must match our tuple model
                            tuple.model() == model_basename
                        } else {
                            // No provider prefix, just match basename
                            tuple.model() == model
                        }
                    }
                }
            }
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
            LabelAssignmentTarget::ProviderModelTuple { tuple } => {
                format!("provider:model tuple '{}'", tuple)
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

    /// The label being assigned (hashed ID for uniqueness).
    pub label_id: String,

    /// The original label name for display purposes.
    pub label_name: String,

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
    pub fn new_to_instance(
        id: String,
        label_id: String,
        label_name: String,
        instance_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            label_id,
            label_name,
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
        label_name: String,
        instance_id: String,
        model_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            label_id,
            label_name,
            target: LabelAssignmentTarget::Model {
                instance_id,
                model_id,
            },
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new label assignment to a provider:model tuple.
    #[must_use]
    pub fn new_to_provider_model_tuple(
        id: String,
        label_id: String,
        label_name: String,
        tuple: ProviderModelTuple,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            label_id,
            label_name,
            target: LabelAssignmentTarget::ProviderModelTuple { tuple },
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
            LabelAssignmentTarget::ProviderModelTuple { tuple } => {
                if tuple.provider().is_empty() {
                    return Err("Provider ID cannot be empty".to_string());
                }
                if tuple.model().is_empty() {
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

    /// Checks if this assignment targets a specific provider:model tuple.
    #[must_use]
    pub fn targets_provider_model_tuple(&self, tuple: &ProviderModelTuple) -> bool {
        match &self.target {
            LabelAssignmentTarget::ProviderModelTuple {
                tuple: target_tuple,
            } => target_tuple == tuple,
            _ => false,
        }
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
            "Test Label".to_string(),
            "instance-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-1");
        assert_eq!(assignment.label_id, "label-1");
        assert_eq!(assignment.label_name, "Test Label");
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
            "Test Model Label".to_string(),
            "instance-1".to_string(),
            "model-1".to_string(),
        );

        assert_eq!(assignment.id, "assignment-2");
        assert_eq!(assignment.label_id, "label-2");
        assert_eq!(assignment.label_name, "Test Model Label");
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
            "Metadata Label".to_string(),
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
            "Unique Label".to_string(),
            "instance-1".to_string(),
        );

        let assignment2 = LabelAssignment::new_to_model(
            "assignment-2".to_string(),
            "unique-label".to_string(),
            "Unique Label".to_string(),
            "instance-2".to_string(),
            "model-1".to_string(),
        );

        let assignment3 = LabelAssignment::new_to_instance(
            "assignment-3".to_string(),
            "different-label".to_string(),
            "Different Label".to_string(),
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
            "Valid Label".to_string(),
            "valid-instance".to_string(),
        );
        assert!(valid_assignment.validate().is_ok());

        let empty_id_assignment = LabelAssignment::new_to_instance(
            String::new(),
            "valid-label".to_string(),
            "Valid Label".to_string(),
            "valid-instance".to_string(),
        );
        assert!(empty_id_assignment.validate().is_err());

        let empty_label_id_assignment = LabelAssignment::new_to_instance(
            "valid-id".to_string(),
            String::new(),
            "Valid Label".to_string(),
            "valid-instance".to_string(),
        );
        assert!(empty_label_id_assignment.validate().is_err());

        let empty_instance_assignment = LabelAssignment::new_to_instance(
            "valid-id".to_string(),
            "valid-label".to_string(),
            "Valid Label".to_string(),
            String::new(),
        );
        assert!(empty_instance_assignment.validate().is_err());

        let empty_model_assignment = LabelAssignment::new_to_model(
            "valid-id".to_string(),
            "valid-label".to_string(),
            "Valid Label".to_string(),
            "valid-instance".to_string(),
            String::new(),
        );
        assert!(empty_model_assignment.validate().is_err());
    }

    #[test]
    fn test_provider_model_tuple_assignment_matching() {
        use crate::utils::ProviderModelTuple;

        // Create a provider:model tuple assignment
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let assignment = LabelAssignment::new_to_provider_model_tuple(
            "assignment-1".to_string(),
            "label-1".to_string(),
            "Test Label".to_string(),
            tuple.clone(),
        );

        // Test matching with full model ID (with provider prefix)
        assert!(assignment.targets_model("openai", "openai/gpt-4"));
        assert!(!assignment.targets_model("openai", "openai/gpt-4-turbo")); // Different model should not match

        // Test matching with basename only
        assert!(assignment.targets_model("openai", "gpt-4"));
        assert!(!assignment.targets_model("instance-1", "gpt-4-turbo")); // Different model should not match

        // Test provider-level matching - should match instances with the same provider
        assert!(assignment.targets_instance("openai")); // Should match the provider from the tuple
        assert!(!assignment.targets_instance("anthropic")); // Should not match different provider
    }

    #[test]
    fn test_provider_model_tuple_assignment_with_complex_model_names() {
        use crate::utils::ProviderModelTuple;

        // Test with complex model names that have multiple slashes
        let tuple = ProviderModelTuple::parse("openrouter:deepseek-v3.2-exp").unwrap();
        let assignment = LabelAssignment::new_to_provider_model_tuple(
            "assignment-1".to_string(),
            "label-1".to_string(),
            "Complex Label".to_string(),
            tuple.clone(),
        );

        // Test various model ID formats
        assert!(assignment.targets_model("openrouter", "openrouter/deepseek-v3.2-exp")); // Matching provider prefix
        assert!(assignment.targets_model("openrouter", "deepseek-v3.2-exp")); // Basename only

        // Should not match different models or providers
        assert!(!assignment.targets_model("openrouter", "openrouter/deepseek-v3.2-beta")); // Different model
        assert!(!assignment.targets_model("openrouter", "deepseek/deepseek-v3.2-exp")); // Different provider prefix
        assert!(!assignment.targets_model("openrouter", "claude/deepseek-v3.2-exp"));
        // Different provider prefix
    }

    #[test]
    fn test_label_assignment_target_description() {
        use crate::utils::ProviderModelTuple;

        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple_target = LabelAssignmentTarget::ProviderModelTuple { tuple };

        assert_eq!(
            tuple_target.description(),
            "provider:model tuple 'openai:gpt-4'"
        );
    }

    #[test]
    fn test_label_assignment_target_methods() {
        use crate::utils::ProviderModelTuple;

        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple_target = LabelAssignmentTarget::ProviderModelTuple {
            tuple: tuple.clone(),
        };

        // Test instance_id() returns the provider
        assert_eq!(tuple_target.instance_id(), "openai");

        // Test model_id() returns the model
        assert_eq!(tuple_target.model_id(), Some("gpt-4"));

        // Test matches() with various scenarios
        assert!(tuple_target.matches("openai", None)); // Provider-level match
        assert!(tuple_target.matches("openai", Some("gpt-4"))); // Exact match
        assert!(tuple_target.matches("openai", Some("openai/gpt-4"))); // Full model ID match with matching provider prefix
        assert!(!tuple_target.matches("openai", Some("anthropic/gpt-4"))); // Different provider prefix should not match
        assert!(!tuple_target.matches("anthropic", Some("gpt-4"))); // Different provider
        assert!(!tuple_target.matches("openai", Some("claude-3"))); // Different model
    }
}
