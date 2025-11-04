//! Label model for unique identifiers that can only be applied to one provider/model combination at a time.

use crate::utils::ProviderModelTuple;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A label is a unique identifier that can only be applied to one provider/model combination at a time.
/// Labels enforce uniqueness across all targets in the system and are scoped to specific provider:model tuples.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    /// Unique identifier for this label.
    pub id: String,

    /// Human-readable name for the label.
    pub name: String,

    /// Optional description of what this label represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional color for UI display purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// The provider:model tuple this label is scoped to.
    /// If None, the label applies to all provider:model combinations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_model_tuple: Option<ProviderModelTuple>,

    /// Additional metadata for this label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// When this label was created.
    pub created_at: DateTime<Utc>,

    /// When this label was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Label {
    /// Creates a new label with required fields.
    #[must_use]
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            color: None,
            provider_model_tuple: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new label with a provider:model tuple scope.
    #[must_use]
    pub fn with_provider_model_tuple(mut self, tuple: ProviderModelTuple) -> Self {
        self.provider_model_tuple = Some(tuple);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new label with a description.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new label with a color.
    #[must_use]
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new label with metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        self
    }

    /// Sets the description for this label.
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Sets the color for this label.
    pub fn set_color(&mut self, color: Option<String>) {
        self.color = color;
        self.updated_at = Utc::now();
    }

    /// Sets metadata for this label.
    pub fn set_metadata(&mut self, metadata: Option<HashMap<String, String>>) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// Sets the provider:model tuple for this label.
    pub fn set_provider_model_tuple(&mut self, tuple: Option<ProviderModelTuple>) {
        self.provider_model_tuple = tuple;
        self.updated_at = Utc::now();
    }

    /// Gets metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.as_ref().and_then(|meta| meta.get(key))
    }

    /// Validates the label configuration.
    ///
    /// # Errors
    /// Returns an error if the label ID or name is empty, or if the name is too long.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Label ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Label name cannot be empty".to_string());
        }
        if self.name.len() > 100 {
            return Err("Label name cannot exceed 100 characters".to_string());
        }
        if let Some(ref color) = self.color {
            if color.len() > 20 {
                return Err("Label color cannot exceed 20 characters".to_string());
            }
        }
        if let Some(ref description) = self.description {
            if description.len() > 500 {
                return Err("Label description cannot exceed 500 characters".to_string());
            }
        }

        Ok(())
    }

    /// Checks if this label can be assigned to a target (always true for labels).
    /// This method exists for API consistency with tags.
    #[must_use]
    pub fn can_assign_to(&self, _target_id: &str) -> bool {
        true
    }

    /// Checks if this label can be assigned to a specific provider:model tuple.
    #[must_use]
    pub fn can_assign_to_tuple(&self, tuple: &ProviderModelTuple) -> bool {
        // If no tuple is specified for the label, it can be assigned to any tuple
        if let Some(ref label_tuple) = self.provider_model_tuple {
            label_tuple == tuple
        } else {
            true
        }
    }

    /// Gets the uniqueness constraint for this label.
    /// Labels are unique across all provider/model combinations, but scoped to their tuple if specified.
    #[must_use]
    pub fn uniqueness_scope(&self) -> &'static str {
        "global"
    }

    /// Gets the provider:model tuple this label is scoped to, if any.
    #[must_use]
    pub fn get_provider_model_tuple(&self) -> Option<&ProviderModelTuple> {
        self.provider_model_tuple.as_ref()
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label = Label::new("label-1".to_string(), "Production".to_string());

        assert_eq!(label.id, "label-1");
        assert_eq!(label.name, "Production");
        assert!(label.description.is_none());
        assert!(label.color.is_none());
        assert!(label.metadata.is_none());
        assert!(label.created_at <= Utc::now());
        assert!(label.updated_at <= Utc::now());
    }

    #[test]
    fn test_label_with_description() {
        let label = Label::new("label-2".to_string(), "Development".to_string())
            .with_description("Development environment".to_string());

        assert_eq!(label.name, "Development");
        assert_eq!(
            label.description,
            Some("Development environment".to_string())
        );
    }

    #[test]
    fn test_label_with_color() {
        let label = Label::new("label-3".to_string(), "Testing".to_string())
            .with_color("#FF0000".to_string());

        assert_eq!(label.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_label_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "environment".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let label =
            Label::new("label-4".to_string(), "Critical".to_string()).with_metadata(metadata);

        assert_eq!(
            label.get_metadata("category"),
            Some(&"environment".to_string())
        );
        assert_eq!(label.get_metadata("priority"), Some(&"high".to_string()));
        assert_eq!(label.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_label_validation() {
        let valid_label = Label::new("valid-label".to_string(), "Valid Label".to_string());
        assert!(valid_label.validate().is_ok());

        let empty_id_label = Label::new(String::new(), "Valid Name".to_string());
        assert!(empty_id_label.validate().is_err());

        let empty_name_label = Label::new("valid-id".to_string(), String::new());
        assert!(empty_name_label.validate().is_err());

        let long_name_label = Label::new("valid-id".to_string(), "a".repeat(101));
        assert!(long_name_label.validate().is_err());

        let long_color_label =
            Label::new("valid-id".to_string(), "Valid Name".to_string()).with_color("a".repeat(21));
        assert!(long_color_label.validate().is_err());

        let long_description_label = Label::new("valid-id".to_string(), "Valid Name".to_string())
            .with_description("a".repeat(501));
        assert!(long_description_label.validate().is_err());
    }

    #[test]
    fn test_label_setters() {
        let mut label = Label::new("test-label".to_string(), "Test Label".to_string());

        label.set_description(Some("Updated description".to_string()));
        assert_eq!(label.description, Some("Updated description".to_string()));

        label.set_color(Some("#00FF00".to_string()));
        assert_eq!(label.color, Some("#00FF00".to_string()));

        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        label.set_metadata(Some(metadata.clone()));
        assert_eq!(label.metadata, Some(metadata));
    }

    #[test]
    fn test_label_uniqueness() {
        let label = Label::new("unique-label".to_string(), "Unique Label".to_string());

        assert!(label.can_assign_to("any-target"));
        assert_eq!(label.uniqueness_scope(), "global");
    }
    fn test_label_with_provider_model_tuple() {
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple_clone = tuple.clone();
        let label = Label::new("label-1".to_string(), "Production GPT-4".to_string())
            .with_provider_model_tuple(tuple_clone);

        assert_eq!(label.provider_model_tuple, Some(tuple.clone()));
        assert!(label.can_assign_to_tuple(&tuple));
    }

    #[test]
    fn test_label_provider_model_tuple_scope() {
        let tuple1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple2 = ProviderModelTuple::parse("anthropic:claude-3").unwrap();

        let label = Label::new("label-1".to_string(), "Production".to_string())
            .with_provider_model_tuple(tuple1.clone());

        // Should be able to assign to the same tuple
        assert!(label.can_assign_to_tuple(&tuple1));

        // Should not be able to assign to a different tuple
        assert!(!label.can_assign_to_tuple(&tuple2));
    }

    #[test]
    fn test_label_without_provider_model_tuple() {
        let label = Label::new("label-1".to_string(), "Global Label".to_string());

        // Should be None when not specified
        assert_eq!(label.provider_model_tuple, None);

        // Should be able to assign to any tuple
        let tuple1 = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let tuple2 = ProviderModelTuple::parse("anthropic:claude-3").unwrap();

        assert!(label.can_assign_to_tuple(&tuple1));
        assert!(label.can_assign_to_tuple(&tuple2));
    }

    #[test]
    fn test_label_set_provider_model_tuple() {
        let mut label = Label::new("label-1".to_string(), "Test Label".to_string());

        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        label.set_provider_model_tuple(Some(tuple.clone()));

        assert_eq!(label.provider_model_tuple, Some(tuple.clone()));

        // Clear the tuple
        label.set_provider_model_tuple(None);
        assert_eq!(label.provider_model_tuple, None);
    }

    #[test]
    fn test_label_get_provider_model_tuple() {
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = Label::new("label-1".to_string(), "Test Label".to_string())
            .with_provider_model_tuple(tuple.clone());

        let retrieved_tuple = label.get_provider_model_tuple().unwrap();
        assert_eq!(retrieved_tuple, &tuple);
    }

    #[test]
    fn test_label_validation_with_provider_model_tuple() {
        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = Label::new("valid-label".to_string(), "Valid Label".to_string())
            .with_provider_model_tuple(tuple);

        assert!(label.validate().is_ok());
    }

    #[test]
    fn test_label_serialization_with_provider_model_tuple() {
        use serde_json;

        let tuple = ProviderModelTuple::parse("openai:gpt-4").unwrap();
        let label = Label::new("test-label".to_string(), "Test Label".to_string())
            .with_provider_model_tuple(tuple.clone())
            .with_description("A test label".to_string())
            .with_color("#FF0000".to_string());

        // Test JSON serialization/deserialization
        let json = serde_json::to_string(&label).unwrap();
        let deserialized: Label = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, label.id);
        assert_eq!(deserialized.name, label.name);
        assert_eq!(deserialized.provider_model_tuple, Some(tuple));
        assert_eq!(deserialized.description, label.description);
        assert_eq!(deserialized.color, label.color);
    }
}
