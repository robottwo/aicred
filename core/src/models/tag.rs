//! Tag model for shared identifiers that can be applied to multiple provider/model combinations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tag is a shared identifier that can be applied to multiple provider/model combinations.
/// Tags are non-unique and can be reused across different targets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    /// Unique identifier for this tag.
    pub id: String,

    /// Human-readable name for the tag.
    pub name: String,

    /// Optional description of what this tag represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional color for UI display purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Additional metadata for this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// When this tag was created.
    pub created_at: DateTime<Utc>,

    /// When this tag was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    /// Creates a new tag with required fields.
    #[must_use]
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            color: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new tag with a description.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new tag with a color.
    #[must_use]
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self.updated_at = Utc::now();
        self
    }

    /// Creates a new tag with metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        self
    }

    /// Sets the description for this tag.
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Sets the color for this tag.
    pub fn set_color(&mut self, color: Option<String>) {
        self.color = color;
        self.updated_at = Utc::now();
    }

    /// Sets metadata for this tag.
    pub fn set_metadata(&mut self, metadata: Option<HashMap<String, String>>) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// Gets metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.as_ref().and_then(|meta| meta.get(key))
    }

    /// Validates the tag configuration.
    ///
    /// # Errors
    /// Returns an error if the tag ID or name is empty, or if the name is too long.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Tag ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Tag name cannot be empty".to_string());
        }
        if self.name.len() > 100 {
            return Err("Tag name cannot exceed 100 characters".to_string());
        }
        if let Some(ref color) = self.color {
            if color.len() > 20 {
                return Err("Tag color cannot exceed 20 characters".to_string());
            }
        }
        if let Some(ref description) = self.description {
            if description.len() > 500 {
                return Err("Tag description cannot exceed 500 characters".to_string());
            }
        }

        Ok(())
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("tag-1".to_string(), "Production".to_string());

        assert_eq!(tag.id, "tag-1");
        assert_eq!(tag.name, "Production");
        assert!(tag.description.is_none());
        assert!(tag.color.is_none());
        assert!(tag.metadata.is_none());
        assert!(tag.created_at <= Utc::now());
        assert!(tag.updated_at <= Utc::now());
    }

    #[test]
    fn test_tag_with_description() {
        let tag = Tag::new("tag-2".to_string(), "Development".to_string())
            .with_description("Development environment".to_string());

        assert_eq!(tag.name, "Development");
        assert_eq!(tag.description, Some("Development environment".to_string()));
    }

    #[test]
    fn test_tag_with_color() {
        let tag =
            Tag::new("tag-3".to_string(), "Testing".to_string()).with_color("#FF0000".to_string());

        assert_eq!(tag.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_tag_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "environment".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let tag = Tag::new("tag-4".to_string(), "Critical".to_string()).with_metadata(metadata);

        assert_eq!(
            tag.get_metadata("category"),
            Some(&"environment".to_string())
        );
        assert_eq!(tag.get_metadata("priority"), Some(&"high".to_string()));
        assert_eq!(tag.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_tag_validation() {
        let valid_tag = Tag::new("valid-tag".to_string(), "Valid Tag".to_string());
        assert!(valid_tag.validate().is_ok());

        let empty_id_tag = Tag::new(String::new(), "Valid Name".to_string());
        assert!(empty_id_tag.validate().is_err());

        let empty_name_tag = Tag::new("valid-id".to_string(), String::new());
        assert!(empty_name_tag.validate().is_err());

        let long_name_tag = Tag::new("valid-id".to_string(), "a".repeat(101));
        assert!(long_name_tag.validate().is_err());

        let long_color_tag =
            Tag::new("valid-id".to_string(), "Valid Name".to_string()).with_color("a".repeat(21));
        assert!(long_color_tag.validate().is_err());

        let long_description_tag = Tag::new("valid-id".to_string(), "Valid Name".to_string())
            .with_description("a".repeat(501));
        assert!(long_description_tag.validate().is_err());
    }

    #[test]
    fn test_tag_setters() {
        let mut tag = Tag::new("test-tag".to_string(), "Test Tag".to_string());

        tag.set_description(Some("Updated description".to_string()));
        assert_eq!(tag.description, Some("Updated description".to_string()));

        tag.set_color(Some("#00FF00".to_string()));
        assert_eq!(tag.color, Some("#00FF00".to_string()));

        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        tag.set_metadata(Some(metadata.clone()));
        assert_eq!(tag.metadata, Some(metadata));
    }
}
