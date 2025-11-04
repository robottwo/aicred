//! Integration tests for the core tagging and labeling data structures.
//!
//! These tests validate the integration between core data structures,
//! serialization, validation, and business logic.

use aicred_core::models::{Label, LabelAssignment, Tag, TagAssignment};
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
        Label::new(format!("label-{}", name), name.to_string())
            .with_description(format!("Test label: {}", name))
            .with_color("#00ff00".to_string())
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
        assert!(label.validate().is_ok());
        assert_eq!(label.name, "primary");
        assert_eq!(label.color, Some("#00ff00".to_string()));

        // Test invalid label validation
        let invalid_label = Label::new("".to_string(), "".to_string());
        assert!(invalid_label.validate().is_err());

        let long_name_label = Label::new("valid-id".to_string(), "a".repeat(101));
        assert!(long_name_label.validate().is_err());
    }

    #[test]
    fn test_tag_assignment_creation() {
        let tag = create_test_tag("development");

        // Test instance assignment
        let instance_assignment = TagAssignment::new_to_instance(
            "assignment-1".to_string(),
            tag.id.clone(),
            "instance-1".to_string(),
        );

        assert!(instance_assignment.validate().is_ok());
        assert!(instance_assignment.targets_instance("instance-1"));
        assert!(!instance_assignment.targets_instance("instance-2"));

        // Test model assignment
        let model_assignment = TagAssignment::new_to_model(
            "assignment-2".to_string(),
            tag.id.clone(),
            "instance-1".to_string(),
            "gpt-4".to_string(),
        );

        assert!(model_assignment.validate().is_ok());
        assert!(model_assignment.targets_model("instance-1", "gpt-4"));
        assert!(!model_assignment.targets_model("instance-1", "gpt-3.5"));
    }

    #[test]
    fn test_label_assignment_creation() {
        let label = create_test_label("production-primary");

        // Test instance assignment
        let instance_assignment = LabelAssignment::new_to_instance(
            "assignment-1".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-1".to_string(),
        );

        assert!(instance_assignment.validate().is_ok());
        assert!(instance_assignment.targets_instance("instance-1"));
        assert_eq!(instance_assignment.uniqueness_key(), label.id);

        // Test model assignment
        let model_assignment = LabelAssignment::new_to_model(
            "assignment-2".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-1".to_string(),
            "claude-3".to_string(),
        );

        assert!(model_assignment.validate().is_ok());
        assert!(model_assignment.targets_model("instance-1", "claude-3"));
        assert_eq!(model_assignment.uniqueness_key(), label.id);
    }

    #[test]
    fn test_label_conflict_detection() {
        let label = create_test_label("unique-label");

        let assignment1 = LabelAssignment::new_to_instance(
            "assignment-1".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-1".to_string(),
        );

        let assignment2 = LabelAssignment::new_to_instance(
            "assignment-2".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-2".to_string(),
        );

        let assignment3 = LabelAssignment::new_to_instance(
            "assignment-3".to_string(),
            "different-label-id".to_string(),
            "Different Label".to_string(),
            "instance-1".to_string(),
        );

        // Same label should conflict
        assert!(assignment1.conflicts_with(&assignment2));
        assert!(assignment2.conflicts_with(&assignment1));

        // Different labels should not conflict
        assert!(!assignment1.conflicts_with(&assignment3));
        assert!(!assignment3.conflicts_with(&assignment1));
    }

    #[test]
    fn test_metadata_handling() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "environment".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let tag = create_test_tag("metadata-test").with_metadata(metadata.clone());
        let label = create_test_label("metadata-label").with_metadata(metadata);

        // Test tag metadata
        assert_eq!(
            tag.get_metadata("category"),
            Some(&"environment".to_string())
        );
        assert_eq!(tag.get_metadata("priority"), Some(&"high".to_string()));
        assert_eq!(tag.get_metadata("nonexistent"), None);

        // Test label metadata
        assert_eq!(
            label.get_metadata("category"),
            Some(&"environment".to_string())
        );
        assert_eq!(label.get_metadata("priority"), Some(&"high".to_string()));
        assert_eq!(label.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_serialization_deserialization() {
        let tag = create_test_tag("serialization-test");
        let label = create_test_label("serialization-label");

        // Add metadata for comprehensive testing
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), "value".to_string());

        let tag_with_meta = tag.with_metadata(metadata.clone());
        let label_with_meta = label.with_metadata(metadata);

        // Test tag serialization
        let tag_json = serde_json::to_string(&tag_with_meta).expect("Failed to serialize tag");
        let deserialized_tag: Tag =
            serde_json::from_str(&tag_json).expect("Failed to deserialize tag");

        assert_eq!(deserialized_tag.name, tag_with_meta.name);
        assert_eq!(deserialized_tag.color, tag_with_meta.color);
        assert_eq!(
            deserialized_tag.get_metadata("test"),
            Some(&"value".to_string())
        );

        // Test label serialization
        let label_json =
            serde_json::to_string(&label_with_meta).expect("Failed to serialize label");
        let deserialized_label: Label =
            serde_json::from_str(&label_json).expect("Failed to deserialize label");

        assert_eq!(deserialized_label.name, label_with_meta.name);
        assert_eq!(deserialized_label.color, label_with_meta.color);
        assert_eq!(
            deserialized_label.get_metadata("test"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_assignment_target_methods() {
        let tag = create_test_tag("target-test");

        let instance_assignment = TagAssignment::new_to_instance(
            "instance-assignment".to_string(),
            tag.id.clone(),
            "test-instance".to_string(),
        );

        let model_assignment = TagAssignment::new_to_model(
            "model-assignment".to_string(),
            tag.id.clone(),
            "test-instance".to_string(),
            "test-model".to_string(),
        );

        // Test instance assignment methods
        assert_eq!(instance_assignment.target.instance_id(), "test-instance");
        assert_eq!(instance_assignment.target.model_id(), None);
        assert!(instance_assignment.target.matches("test-instance", None));
        assert!(!instance_assignment
            .target
            .matches("test-instance", Some("model")));
        assert!(!instance_assignment.target.matches("other-instance", None));

        // Test model assignment methods
        assert_eq!(model_assignment.target.instance_id(), "test-instance");
        assert_eq!(model_assignment.target.model_id(), Some("test-model"));
        assert!(!model_assignment.target.matches("test-instance", None));
        assert!(model_assignment
            .target
            .matches("test-instance", Some("test-model")));
        assert!(!model_assignment
            .target
            .matches("test-instance", Some("other-model")));
        assert!(!model_assignment
            .target
            .matches("other-instance", Some("test-model")));
    }

    #[test]
    fn test_assignment_descriptions() {
        let tag = create_test_tag("description-test");

        let instance_assignment = TagAssignment::new_to_instance(
            "instance-desc".to_string(),
            tag.id.clone(),
            "openai-prod".to_string(),
        );

        let model_assignment = TagAssignment::new_to_model(
            "model-desc".to_string(),
            tag.id.clone(),
            "openai-prod".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(
            instance_assignment.target_description(),
            "provider instance 'openai-prod'"
        );

        assert_eq!(
            model_assignment.target_description(),
            "model 'gpt-4' in provider instance 'openai-prod'"
        );
    }

    #[test]
    fn test_large_dataset_creation() {
        let start = std::time::Instant::now();

        // Create large number of tags
        let mut tags = Vec::new();
        for i in 0..1000 {
            tags.push(create_test_tag(&format!("bulk-tag-{}", i)));
        }

        // Create large number of labels
        let mut labels = Vec::new();
        for i in 0..1000 {
            labels.push(create_test_label(&format!("bulk-label-{}", i)));
        }

        // Create large number of assignments
        let mut assignments = Vec::new();
        for i in 0..1000 {
            assignments.push(TagAssignment::new_to_instance(
                format!("bulk-assignment-{}", i),
                format!("tag-bulk-tag-{}", i % 100), // Reuse some tag IDs
                format!("instance-{}", i % 50),      // Reuse some instance IDs
            ));
        }

        let duration = start.elapsed();

        // Verify all objects were created
        assert_eq!(tags.len(), 1000);
        assert_eq!(labels.len(), 1000);
        assert_eq!(assignments.len(), 1000);

        // Performance assertion (should complete quickly)
        assert!(
            duration.as_millis() < 1000,
            "Large dataset creation took too long: {:?}",
            duration
        );

        // Verify all objects are valid
        for tag in &tags {
            assert!(tag.validate().is_ok());
        }

        for label in &labels {
            assert!(label.validate().is_ok());
        }

        for assignment in &assignments {
            assert!(assignment.validate().is_ok());
        }
    }

    #[test]
    fn test_concurrent_assignment_simulation() {
        let tag = create_test_tag("concurrent-test");

        // Simulate multiple assignments to the same tag (should be allowed for tags)
        let assignments = vec![
            TagAssignment::new_to_instance(
                "assignment-1".to_string(),
                tag.id.clone(),
                "instance-1".to_string(),
            ),
            TagAssignment::new_to_instance(
                "assignment-2".to_string(),
                tag.id.clone(),
                "instance-2".to_string(),
            ),
            TagAssignment::new_to_model(
                "assignment-3".to_string(),
                tag.id.clone(),
                "instance-1".to_string(),
                "model-1".to_string(),
            ),
            TagAssignment::new_to_model(
                "assignment-4".to_string(),
                tag.id.clone(),
                "instance-1".to_string(),
                "model-2".to_string(),
            ),
        ];

        // All assignments should be valid
        for assignment in &assignments {
            assert!(assignment.validate().is_ok());
        }

        // Verify targeting
        assert_eq!(assignments[0].targets_instance("instance-1"), true);
        assert_eq!(assignments[1].targets_instance("instance-2"), true);
        assert_eq!(assignments[2].targets_model("instance-1", "model-1"), true);
        assert_eq!(assignments[3].targets_model("instance-1", "model-2"), true);
    }

    #[test]
    fn test_label_uniqueness_enforcement() {
        let label = create_test_label("unique-test");

        // Labels should have global uniqueness scope
        assert_eq!(label.uniqueness_scope(), "global");
        assert!(label.can_assign_to("any-target"));

        // Create assignments with same label (should conflict)
        let assignment1 = LabelAssignment::new_to_instance(
            "assignment-1".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-1".to_string(),
        );
        let assignment2 = LabelAssignment::new_to_instance(
            "assignment-2".to_string(),
            label.id.clone(),
            label.name.clone(),
            "instance-2".to_string(),
        );

        assert!(assignment1.conflicts_with(&assignment2));
        assert_eq!(assignment1.uniqueness_key(), label.id);
        assert_eq!(assignment2.uniqueness_key(), label.id);
    }

    #[test]
    fn test_error_conditions() {
        // Test various error conditions

        // Empty IDs
        let empty_tag = Tag::new("".to_string(), "valid-name".to_string());
        assert!(empty_tag.validate().is_err());

        let empty_label = Label::new("valid-id".to_string(), "".to_string());
        assert!(empty_label.validate().is_err());

        // Empty assignment fields
        let empty_assignment =
            TagAssignment::new_to_instance("".to_string(), "".to_string(), "instance".to_string());
        assert!(empty_assignment.validate().is_err());

        // Empty model ID in model assignment
        let empty_model_assignment = TagAssignment::new_to_model(
            "valid-id".to_string(),
            "valid-tag".to_string(),
            "valid-instance".to_string(),
            "".to_string(),
        );
        assert!(empty_model_assignment.validate().is_err());

        // Valid objects should pass
        let valid_tag = create_test_tag("valid-test");
        assert!(valid_tag.validate().is_ok());

        let valid_assignment = TagAssignment::new_to_instance(
            "valid-id".to_string(),
            "valid-tag".to_string(),
            "valid-instance".to_string(),
        );
        assert!(valid_assignment.validate().is_ok());
    }
}
