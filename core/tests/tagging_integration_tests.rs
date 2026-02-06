//! Integration tests for the core tagging and labeling data structures.
//!
//! These tests validate the integration between core data structures,
//! serialization, validation, and business logic.

use aicred_core::models::{Label, LabelAssignment, LabelTarget};
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_label(name: &str) -> Label {
        Label {
            name: name.to_string(),
            description: Some(format!("Test label: {}", name)),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_label_creation() {
        // Test valid label creation
        let label = create_test_label("primary");
        assert_eq!(label.name, "primary");
        assert_eq!(label.description, Some("Test label: primary".to_string()));

        // New Label structure doesn't have validation or color field
        // Basic validation is just checking name is not empty
        assert!(!label.name.is_empty());
    }

    #[test]
    fn test_label_assignment_creation() {
        let label = create_test_label("fast");

        // Instance-level assignment
        let instance_assignment = LabelAssignment {
            label_name: label.name.clone(),
            target: LabelTarget::ProviderInstance {
                instance_id: "provider-123".to_string(),
            },
            assigned_at: Utc::now(),
            assigned_by: None,
        };

        assert_eq!(instance_assignment.target.instance_id(), "provider-123");
        assert_eq!(instance_assignment.target.model_id(), None);

        // Model-level assignment
        let model_assignment = LabelAssignment {
            label_name: label.name.clone(),
            target: LabelTarget::ProviderModel {
                instance_id: "provider-123".to_string(),
                model_id: "model-456".to_string(),
            },
            assigned_at: Utc::now(),
            assigned_by: None,
        };

        assert_eq!(model_assignment.target.instance_id(), "provider-123");
        assert_eq!(model_assignment.target.model_id(), Some("model-456"));
    }

    #[test]
    fn test_multiple_labels_same_target() {
        let fast_label = create_test_label("fast");
        let cheap_label = create_test_label("cheap");

        let instance_id = "provider-123".to_string();

        let assignment1 = LabelAssignment {
            label_name: fast_label.name.clone(),
            target: LabelTarget::ProviderInstance {
                instance_id: instance_id.clone(),
            },
            assigned_at: Utc::now(),
            assigned_by: Some("user-1".to_string()),
        };

        let assignment2 = LabelAssignment {
            label_name: cheap_label.name.clone(),
            target: LabelTarget::ProviderInstance {
                instance_id: instance_id.clone(),
            },
            assigned_at: Utc::now(),
            assigned_by: Some("user-2".to_string()),
        };

        assert_eq!(assignment1.label_name, "fast");
        assert_eq!(assignment2.label_name, "cheap");
        assert_eq!(assignment1.target.instance_id(), instance_id);
        assert_eq!(assignment2.target.instance_id(), instance_id);
    }

    #[test]
    fn test_label_serialization() {
        let label = Label {
            name: "production".to_string(),
            description: Some("Production environment label".to_string()),
            created_at: Utc::now(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("environment".to_string(), "prod".to_string());
                m.insert("tier".to_string(), "1".to_string());
                m
            },
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&label).unwrap();
        assert!(json.contains("production"));

        // Test deserialization
        let deserialized: Label = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "production");
        assert_eq!(deserialized.description, label.description);
    }

    #[test]
    fn test_label_assignment_serialization() {
        let assignment = LabelAssignment {
            label_name: "prod-tag".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "provider-123".to_string(),
                model_id: "model-456".to_string(),
            },
            assigned_at: Utc::now(),
            assigned_by: Some("system".to_string()),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&assignment).unwrap();
        assert!(json.contains("prod-tag"));

        // Test deserialization
        let deserialized: LabelAssignment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label_name, "prod-tag");
        assert_eq!(deserialized.target.instance_id(), "provider-123");
        assert_eq!(deserialized.target.model_id(), Some("model-456"));
    }

    #[test]
    fn test_label_with_assignments_integration() {
        use aicred_core::models::LabelWithAssignments;

        let label = create_test_label("fast");

        let assignments = vec![
            LabelAssignment {
                label_name: label.name.clone(),
                target: LabelTarget::ProviderInstance {
                    instance_id: "provider-1".to_string(),
                },
                assigned_at: Utc::now(),
                assigned_by: None,
            },
            LabelAssignment {
                label_name: label.name.clone(),
                target: LabelTarget::ProviderModel {
                    instance_id: "provider-2".to_string(),
                    model_id: "model-1".to_string(),
                },
                assigned_at: Utc::now(),
                assigned_by: Some("user-1".to_string()),
            },
        ];

        let label_with_assignments = LabelWithAssignments {
            label,
            assignments: assignments.clone(),
        };

        assert_eq!(label_with_assignments.assignments.len(), 2);
        assert!(label_with_assignments.has_assignments());
        assert_eq!(label_with_assignments.assignment_count(), 2);
    }
}
