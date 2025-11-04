//! Provider instances collection for managing multiple provider configurations.

use crate::models::ProviderInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collection of provider instances with lookup and filtering capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInstances {
    /// Storage for provider instances by their unique ID.
    #[serde(flatten)]
    instances: HashMap<String, ProviderInstance>,
}

impl ProviderInstances {
    /// Creates a new empty collection of provider instances.
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Creates a new collection with the given instances.
    #[must_use]
    pub fn from_instances(instances: Vec<ProviderInstance>) -> Self {
        let mut collection = Self::new();
        for instance in instances {
            if let Err(e) = collection.add_instance(instance) {
                tracing::warn!("Failed to add instance: {}", e);
            }
        }
        collection
    }

    /// Adds a provider instance to the collection.
    ///
    /// # Arguments
    /// * `instance` - The provider instance to add
    ///
    /// # Errors
    /// Returns an error if an instance with the same ID already exists.
    pub fn add_instance(&mut self, instance: ProviderInstance) -> Result<(), String> {
        if self.instances.contains_key(&instance.id) {
            return Err(format!(
                "Provider instance with ID '{}' already exists",
                instance.id
            ));
        }

        self.instances.insert(instance.id.clone(), instance);
        Ok(())
    }

    /// Adds a provider instance, replacing any existing instance with the same ID.
    pub fn add_or_replace_instance(&mut self, instance: ProviderInstance) {
        self.instances.insert(instance.id.clone(), instance);
    }

    /// Gets a provider instance by ID.
    #[must_use]
    pub fn get_instance(&self, id: &str) -> Option<&ProviderInstance> {
        self.instances.get(id)
    }

    /// Gets a mutable reference to a provider instance by ID.
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut ProviderInstance> {
        self.instances.get_mut(id)
    }

    /// Removes a provider instance by ID.
    pub fn remove_instance(&mut self, id: &str) -> Option<ProviderInstance> {
        self.instances.remove(id)
    }

    /// Gets all provider instances.
    #[must_use]
    pub fn all_instances(&self) -> Vec<&ProviderInstance> {
        self.instances.values().collect()
    }

    /// Gets all provider instances as a mutable reference.
    pub fn all_instances_mut(&mut self) -> Vec<&mut ProviderInstance> {
        self.instances.values_mut().collect()
    }

    /// Gets instances filtered by provider type.
    #[must_use]
    pub fn instances_by_type(&self, provider_type: &str) -> Vec<&ProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.provider_type == provider_type)
            .collect()
    }

    /// Gets only active provider instances.
    #[must_use]
    pub fn active_instances(&self) -> Vec<&ProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.active)
            .collect()
    }

    /// Gets only active instances of a specific provider type.
    #[must_use]
    pub fn active_instances_by_type(&self, provider_type: &str) -> Vec<&ProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.active && instance.provider_type == provider_type)
            .collect()
    }

    /// Gets instances that have valid keys.
    #[must_use]
    pub fn instances_with_valid_keys(&self) -> Vec<&ProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.has_non_empty_api_key())
            .collect()
    }

    /// Gets all models from all instances.
    #[must_use]
    pub fn all_models(&self) -> Vec<&crate::models::Model> {
        self.instances
            .values()
            .flat_map(|instance| &instance.models)
            .collect()
    }

    /// Gets all active models from active instances.
    #[must_use]
    pub fn active_models(&self) -> Vec<&crate::models::Model> {
        self.active_instances()
            .into_iter()
            .flat_map(|instance| instance.active_models())
            .collect()
    }

    /// Gets the total number of instances.
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Checks if the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Gets all instance IDs.
    #[must_use]
    pub fn instance_ids(&self) -> Vec<&String> {
        self.instances.keys().collect()
    }

    /// Gets all provider types present in the collection.
    #[must_use]
    pub fn provider_types(&self) -> Vec<&String> {
        let mut types: Vec<&String> = self
            .instances
            .values()
            .map(|instance| &instance.provider_type)
            .collect();
        types.sort();
        types.dedup();
        types
    }

    /// Validates all instances in the collection.
    ///
    /// # Errors
    /// Returns an error with validation errors for any invalid instances.
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();

        for instance in self.instances.values() {
            if let Err(e) = instance.validate() {
                errors.push(format!("Instance '{}': {}", instance.id, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Clears all instances from the collection.
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Merges another `ProviderInstances` collection into this one.
    /// Existing instances with the same ID will be replaced.
    pub fn merge(&mut self, other: Self) {
        for (id, instance) in other.instances {
            self.instances.insert(id, instance);
        }
    }
}

impl Default for ProviderInstances {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<ProviderInstance>> for ProviderInstances {
    fn from(instances: Vec<ProviderInstance>) -> Self {
        Self::from_instances(instances)
    }
}

impl From<ProviderInstances> for Vec<ProviderInstance> {
    fn from(collection: ProviderInstances) -> Self {
        collection.instances.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::discovered_key::Confidence;
    use crate::models::provider_key::{Environment, ValidationStatus};

    fn create_test_instance(id: &str, provider_type: &str, active: bool) -> ProviderInstance {
        let mut instance = ProviderInstance::new(
            id.to_string(),
            format!("Test {id}"),
            provider_type.to_string(),
            "https://api.example.com".to_string(),
        );

        if active {
            let mut key = crate::models::ProviderKey::new(
                "test-key".to_string(),
                "/test/path".to_string(),
                Confidence::High,
                Environment::Production,
            )
            .with_value("sk-test-key-12345".to_string());
            key.set_validation_status(ValidationStatus::Valid);
            instance.set_api_key(key.value.unwrap());
        } else {
            instance.active = false;
        }

        instance
    }

    #[test]
    fn test_empty_collection() {
        let collection = ProviderInstances::new();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_add_and_get_instance() {
        let mut collection = ProviderInstances::new();
        let instance = create_test_instance("test1", "openai", true);

        assert!(collection.add_instance(instance.clone()).is_ok());
        assert!(collection.add_instance(instance).is_err()); // Duplicate ID

        let retrieved = collection.get_instance("test1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test1");
    }

    #[test]
    fn test_instances_by_type() {
        let mut collection = ProviderInstances::new();

        collection
            .add_instance(create_test_instance("openai1", "openai", true))
            .unwrap();
        collection
            .add_instance(create_test_instance("openai2", "openai", true))
            .unwrap();
        collection
            .add_instance(create_test_instance("anthropic1", "anthropic", true))
            .unwrap();

        let openai_instances = collection.instances_by_type("openai");
        assert_eq!(openai_instances.len(), 2);

        let anthropic_instances = collection.instances_by_type("anthropic");
        assert_eq!(anthropic_instances.len(), 1);

        let unknown_instances = collection.instances_by_type("unknown");
        assert_eq!(unknown_instances.len(), 0);
    }

    #[test]
    fn test_active_instances() {
        let mut collection = ProviderInstances::new();

        collection
            .add_instance(create_test_instance("active1", "openai", true))
            .unwrap();
        collection
            .add_instance(create_test_instance("inactive1", "openai", false))
            .unwrap();
        collection
            .add_instance(create_test_instance("active2", "anthropic", true))
            .unwrap();

        let active = collection.active_instances();
        assert_eq!(active.len(), 2);

        let active_openai = collection.active_instances_by_type("openai");
        assert_eq!(active_openai.len(), 1);
    }

    #[test]
    fn test_instances_with_valid_keys() {
        let mut collection = ProviderInstances::new();

        collection
            .add_instance(create_test_instance("valid1", "openai", true))
            .unwrap();
        collection
            .add_instance(create_test_instance("invalid1", "openai", false))
            .unwrap();

        let with_valid_keys = collection.instances_with_valid_keys();
        assert_eq!(with_valid_keys.len(), 1);
    }

    #[test]
    fn test_all_models() {
        let mut collection = ProviderInstances::new();

        let mut instance1 = create_test_instance("test1", "openai", true);
        instance1.add_model(crate::models::Model::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
        ));

        let mut instance2 = create_test_instance("test2", "anthropic", true);
        instance2.add_model(crate::models::Model::new(
            "claude-3".to_string(),
            "Claude 3".to_string(),
        ));

        collection.add_instance(instance1).unwrap();
        collection.add_instance(instance2).unwrap();

        let all_models = collection.all_models();
        assert_eq!(all_models.len(), 2);

        let active_models = collection.active_models();
        assert_eq!(active_models.len(), 2);
    }

    #[test]
    fn test_remove_instance() {
        let mut collection = ProviderInstances::new();
        let instance = create_test_instance("test1", "openai", true);

        collection.add_instance(instance).unwrap();
        assert_eq!(collection.len(), 1);

        let removed = collection.remove_instance("test1");
        assert!(removed.is_some());
        assert_eq!(collection.len(), 0);

        let not_found = collection.remove_instance("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_merge_collections() {
        let mut collection1 = ProviderInstances::new();
        let mut collection2 = ProviderInstances::new();

        collection1
            .add_instance(create_test_instance("test1", "openai", true))
            .unwrap();
        collection2
            .add_instance(create_test_instance("test2", "anthropic", true))
            .unwrap();
        collection2
            .add_instance(create_test_instance("test1", "groq", true))
            .unwrap(); // Same ID, should replace

        collection1.merge(collection2);

        assert_eq!(collection1.len(), 2);
        assert_eq!(
            collection1.get_instance("test1").unwrap().provider_type,
            "groq"
        );
    }
}
