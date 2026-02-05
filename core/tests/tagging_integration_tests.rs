//! Integration tests for the core tagging and labeling data structures.
//!
//! These tests validate the integration between core data structures,
//! serialization, validation, and business logic.

use aicred_core::models::{Label, LabelAssignment, LabelTarget, Tag, TagAssignment};
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_tag(name: &str) -> Tag {
        Tag::new(format!("tag-{}", name), name.to_string())
            .with_description(format!("Test tag: {}", name))
            .with_color("#ff0000".to_string())
    }

    fn create_test_label(name: &str) -> Label {
        Label {
            name: name.to_string(),
            description: Some(format!("Test label: {}", name)),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_tag_creation_and_validation() {
        // Test valid tag creation
        let tag = create_test_tag("production");
        assert!(tag.validate().is_ok());
        assert_eq!(tag.name, "production");
        assert_eq!(tag.color, Some("#ff0000".to_string()));

        // Test invalid tag validation
        let invalid_tag = Tag::new("".to_string(), "".to_string());
        assert!(invalid_tag.validate().is_err());

        let long_name_tag = Tag::new("valid-id".to_string(), "a".repeat(101));
        assert!(long_name_tag.validate().is_err());
    }

    #[test]
    fn test_label_creation_and_validation() {
        // Test valid label creation
        let label = create_test_label("primary");
        assert_eq!(label.name, "primary");
        assert_eq!(label.description, Some("Test label: primary".to_string()));
        
        // New Label structure doesn't have validation or color field
        // Basic validation is just checking name is not empty
        assert!(!label.name.is_empty());
    }

    #[test]
    fn test_tag_assignment_creation() {
        let tag = create_test_tag("production");

        let instance_assignment = TagAssignment::new_to_instance(
            "assignment-1".to_string(),
            tag.id.clone(),
            "provider-123".to_string(),
        );

        assert_eq!(instance_assignment.tag_id, tag.id);
        assert_eq!(instance_assignment.target.instance_id(), "provider-123");
        assert_eq!(instance_assignment.target.model_id(), None);

        let model_assignment = TagAssignment::new_to_model(
            "assignment-2".to_string(),
            tag.id.clone(),
            "provider-123".to_string(),
            "model-456".to_string(),
        );

        assert_eq!(model_assignment.tag_id, tag.id);
        assert_eq!(model_assignment.target.instance_id(), "provider-123");
        assert_eq!(model_assignment.target.model_id(), Some("model-456"));
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
                model_id: "gpt-4".to_string(),
            },
            assigned_at: Utc::now(),
            assigned_by: None,
        };

        assert_eq!(model_assignment.target.instance_id(), "provider-123");
        assert_eq!(model_assignment.target.model_id(), Some("gpt-4"));
    }

    #[test]
    fn test_tag_assignment_uniqueness() {
        let assignment1 = TagAssignment::new_to_instance(
            "assignment-1".to_string(),
            "tag-prod".to_string(),
            "provider-123".to_string(),
        );

        let assignment2 = TagAssignment::new_to_instance(
            "assignment-2".to_string(),
            "tag-dev".to_string(),
            "provider-123".to_string(),
        );

        // Different tags should have different tag IDs
        assert_ne!(assignment1.tag_id, assignment2.tag_id);
        
        // Different assignments should have different IDs
        assert_ne!(assignment1.id, assignment2.id);

        // Same tag to different providers
        let assignment3 = TagAssignment::new_to_instance(
            "assignment-3".to_string(),
            "tag-prod".to_string(),
            "provider-456".to_string(),
        );

        // Same tag ID
        assert_eq!(assignment1.tag_id, assignment3.tag_id);
        // But different assignment IDs
        assert_ne!(assignment1.id, assignment3.id);
    }

    #[test]
    fn test_label_serialization() {
        let label = create_test_label("fast");
        
        // Test JSON serialization
        let json = serde_json::to_string(&label).unwrap();
        let deserialized: Label = serde_json::from_str(&json).unwrap();
        
        assert_eq!(label.name, deserialized.name);
        assert_eq!(label.description, deserialized.description);
    }

    #[test]
    fn test_label_assignment_serialization() {
        let assignment = LabelAssignment {
            label_name: "fast".to_string(),
            target: LabelTarget::ProviderModel {
                instance_id: "openai-prod".to_string(),
                model_id: "gpt-4".to_string(),
            },
            assigned_at: Utc::now(),
            assigned_by: Some("user@example.com".to_string()),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&assignment).unwrap();
        let deserialized: LabelAssignment = serde_json::from_str(&json).unwrap();

        assert_eq!(assignment.label_name, deserialized.label_name);
        assert_eq!(assignment.target, deserialized.target);
    }

    #[test]
    fn test_tag_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("team".to_string(), "engineering".to_string());

        let tag = Tag::new("prod-tag".to_string(), "Production".to_string())
            .with_metadata(metadata.clone());

        assert_eq!(tag.metadata, Some(metadata));
    }

    #[test]
    fn test_label_metadata() {
        let mut label = create_test_label("smart");
        
        label.metadata.insert("priority".to_string(), "high".to_string());
        label.metadata.insert("category".to_string(), "quality".to_string());

        assert_eq!(label.metadata.get("priority"), Some(&"high".to_string()));
        assert_eq!(label.metadata.get("category"), Some(&"quality".to_string()));
    }

    #[test]
    fn test_label_target_methods() {
        // Test ProviderInstance target
        let instance_target = LabelTarget::ProviderInstance {
            instance_id: "openai-1".to_string(),
        };
        
        assert_eq!(instance_target.instance_id(), "openai-1");
        assert_eq!(instance_target.model_id(), None);

        // Test ProviderModel target
        let model_target = LabelTarget::ProviderModel {
            instance_id: "openai-1".to_string(),
            model_id: "gpt-4".to_string(),
        };
        
        assert_eq!(model_target.instance_id(), "openai-1");
        assert_eq!(model_target.model_id(), Some("gpt-4"));
    }
}
