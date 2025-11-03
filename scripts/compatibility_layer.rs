//! AICred Configuration Compatibility Layer
//!
//! This module provides backward compatibility for existing configurations
//! and utilities for upgrading to the new tagging and labeling system.

use aicred_core::models::{
    ProviderConfig, ProviderInstance, Tag, Label, TagAssignment, LabelAssignment,
    ProviderKey, Environment, ValidationStatus, Confidence
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};

/// Configuration compatibility layer
pub struct CompatibilityLayer {
    config_dir: PathBuf,
}

impl CompatibilityLayer {
    /// Create a new compatibility layer
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Migrate old ProviderConfig to new ProviderInstance model
    pub fn migrate_provider_config(&self, old_config: &ProviderConfig) -> Result<ProviderInstance> {
        // Extract the first valid key as the primary API key
        let primary_key = old_config.keys
            .iter()
            .find(|key| key.value.is_some())
            .ok_or_else(|| anyhow!("No valid API keys found in configuration"))?;

        // Create metadata preserving key information
        let mut metadata = HashMap::new();
        
        // Preserve key metadata
        if let Some(ref env) = primary_key.environment {
            metadata.insert("environment".to_string(), format!("{:?}", env));
        }
        metadata.insert("confidence".to_string(), format!("{:?}", primary_key.confidence));
        metadata.insert("validation_status".to_string(), format!("{:?}", primary_key.validation_status));
        metadata.insert("discovered_at".to_string(), primary_key.discovered_at.to_rfc3339());
        metadata.insert("source".to_string(), primary_key.source.clone());
        
        if let Some(line_number) = primary_key.line_number {
            metadata.insert("line_number".to_string(), line_number.to_string());
        }

        // Add original config metadata
        if let Some(ref config_meta) = old_config.metadata {
            for (key, value) in config_meta {
                if let Some(value_str) = value.as_str() {
                    metadata.insert(key.clone(), value_str.to_string());
                }
            }
        }

        // Create provider instance
        let instance = ProviderInstance::new(
            "migrated-instance".to_string(),
            format!("Migrated {} Instance", old_config.provider_type),
            old_config.provider_type.clone(),
            "https://api.openai.com/v1".to_string(), // Default, should be configurable
        )
        .with_metadata(metadata)
        .with_active(true);

        Ok(instance)
    }

    /// Load and migrate all provider configurations
    pub fn migrate_all_configs(&self) -> Result<Vec<ProviderInstance>> {
        let old_configs = self.load_old_provider_configs()?;
        let mut migrated_instances = Vec::new();

        for config in old_configs {
            match self.migrate_provider_config(&config) {
                Ok(instance) => migrated_instances.push(instance),
                Err(e) => {
                    eprintln!("Warning: Failed to migrate config: {}", e);
                    continue;
                }
            }
        }

        Ok(migrated_instances)
    }

    /// Load old provider configurations
    fn load_old_provider_configs(&self) -> Result<Vec<ProviderConfig>> {
        let old_config_file = self.config_dir.join("provider_configs.yaml");
        
        if !old_config_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&old_config_file)
            .context("Failed to read old provider config file")?;

        let configs: Vec<ProviderConfig> = serde_yaml::from_str(&content)
            .context("Failed to parse old provider config file")?;

        Ok(configs)
    }

    /// Create default tagging configuration
    pub fn create_default_tags(&self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        // Environment tags
        let production_tag = Tag::new(
            "tag-production".to_string(),
            "Production".to_string()
        )
        .with_description("Production environment instances")
        .with_color("#ff0000".to_string())
        .with_metadata(hashmap![
            "environment".to_string() => "production".to_string(),
            "category".to_string() => "environment".to_string()
        ]);

        let staging_tag = Tag::new(
            "tag-staging".to_string(),
            "Staging".to_string()
        )
        .with_description("Staging environment instances")
        .with_color("#ffa500".to_string())
        .with_metadata(hashmap![
            "environment".to_string() => "staging".to_string(),
            "category".to_string() => "environment".to_string()
        ]);

        let development_tag = Tag::new(
            "tag-development".to_string(),
            "Development".to_string()
        )
        .with_description("Development environment instances")
        .with_color("#00ff00".to_string())
        .with_metadata(hashmap![
            "environment".to_string() => "development".to_string(),
            "category".to_string() => "environment".to_string()
        ]);

        tags.push(production_tag);
        tags.push(staging_tag);
        tags.push(development_tag);

        Ok(tags)
    }

    /// Create default labeling configuration
    pub fn create_default_labels(&self) -> Result<Vec<Label>> {
        let mut labels = Vec::new();

        // AI Development workflow labels
        let coding_label = Label::new(
            "label-coding".to_string(),
            "Coding".to_string()
        )
        .with_description("General coding and development tasks")
        .with_color("#17c964".to_string())
        .with_metadata(hashmap![
            "category".to_string() => "workflow".to_string(),
            "priority".to_string() => "high".to_string()
        ]);

        let fast_coding_label = Label::new(
            "label-fast-coding".to_string(),
            "Fast Coding".to_string()
        )
        .with_description("Quick prototyping and rapid development")
        .with_color("#f5a524".to_string())
        .with_metadata(hashmap![
            "category".to_string() => "workflow".to_string(),
            "priority".to_string() => "medium".to_string()
        ]);

        let planning_label = Label::new(
            "label-planning".to_string(),
            "Planning".to_string()
        )
        .with_description("Strategic planning and architecture work")
        .with_color("#9b6cff".to_string())
        .with_metadata(hashmap![
            "category".to_string() => "workflow".to_string(),
            "priority".to_string() => "medium".to_string()
        ]);

        labels.push(coding_label);
        labels.push(fast_coding_label);
        labels.push(planning_label);

        Ok(labels)
    }

    /// Auto-assign tags based on instance characteristics
    pub fn auto_assign_tags(&self, instances: &[ProviderInstance], tags: &[Tag]) -> Result<Vec<TagAssignment>> {
        let mut assignments = Vec::new();

        for instance in instances {
            // Auto-assign environment tags based on instance name or metadata
            let instance_name = &instance.display_name.to_lowercase();
            let instance_id = &instance.id;

            for tag in tags {
                let should_assign = match tag.name.as_str() {
                    "Production" => instance_name.contains("prod") || instance_name.contains("production"),
                    "Staging" => instance_name.contains("staging") || instance_name.contains("stage"),
                    "Development" => instance_name.contains("dev") || instance_name.contains("development"),
                    _ => false,
                };

                if should_assign {
                    let assignment = TagAssignment::new_to_instance(
                        format!("auto-{}-{}", tag.id, instance_id),
                        tag.id.clone(),
                        instance_id.clone(),
                    )
                    .with_metadata(hashmap![
                        "assigned_by".to_string() => "auto-migration".to_string(),
                        "reason".to_string() => "automatic environment assignment".to_string()
                    ]);

                    assignments.push(assignment);
                }
            }
        }

        Ok(assignments)
    }

    /// Validate configuration integrity
    pub fn validate_integrity(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Check if configuration directory exists
        if !self.config_dir.exists() {
            report.add_warning("Configuration directory does not exist");
            return Ok(report);
        }

        // Validate provider instances
        let instances_file = self.config_dir.join("provider_instances.yaml");
        if instances_file.exists() {
            match self.validate_provider_instances() {
                Ok(()) => report.add_info("Provider instances are valid"),
                Err(e) => report.add_error(format!("Provider instances validation failed: {}", e)),
            }
        }

        // Validate tags
        let tags_file = self.config_dir.join("tags.yaml");
        if tags_file.exists() {
            match self.validate_tags() {
                Ok(()) => report.add_info("Tags are valid"),
                Err(e) => report.add_error(format!("Tags validation failed: {}", e)),
            }
        }

        // Validate labels
        let labels_file = self.config_dir.join("labels.yaml");
        if labels_file.exists() {
            match self.validate_labels() {
                Ok(()) => report.add_info("Labels are valid"),
                Err(e) => report.add_error(format!("Labels validation failed: {}", e)),
            }
        }

        // Validate assignments
        if let Err(e) = self.validate_assignments() {
            report.add_error(format!("Assignments validation failed: {}", e));
        } else {
            report.add_info("Assignments are valid");
        }

        Ok(report)
    }

    /// Validate provider instances
    fn validate_provider_instances(&self) -> Result<()> {
        let instances_file = self.config_dir.join("provider_instances.yaml");
        let content = std::fs::read_to_string(&instances_file)?;
        let instances: Vec<ProviderInstance> = serde_yaml::from_str(&content)?;

        // Check for duplicate IDs
        let mut ids = HashMap::new();
        for instance in &instances {
            if let Some(existing) = ids.insert(&instance.id, 1) {
                return Err(anyhow!("Duplicate instance ID: {}", instance.id));
            }
        }

        // Validate each instance
        for instance in &instances {
            if let Err(e) = instance.validate() {
                return Err(anyhow!("Instance validation failed for {}: {}", instance.id, e));
            }
        }

        Ok(())
    }

    /// Validate tags
    fn validate_tags(&self) -> Result<()> {
        let tags_file = self.config_dir.join("tags.yaml");
        let content = std::fs::read_to_string(&tags_file)?;
        let tags: Vec<Tag> = serde_yaml::from_str(&content)?;

        // Check for duplicate names and IDs
        let mut names = HashMap::new();
        let mut ids = HashMap::new();
        for tag in &tags {
            if let Some(existing) = names.insert(&tag.name, 1) {
                return Err(anyhow!("Duplicate tag name: {}", tag.name));
            }
            if let Some(existing) = ids.insert(&tag.id, 1) {
                return Err(anyhow!("Duplicate tag ID: {}", tag.id));
            }
        }

        // Validate each tag
        for tag in &tags {
            if let Err(e) = tag.validate() {
                return Err(anyhow!("Tag validation failed for {}: {}", tag.name, e));
            }
        }

        Ok(())
    }

    /// Validate labels
    fn validate_labels(&self) -> Result<()> {
        let labels_file = self.config_dir.join("labels.yaml");
        let content = std::fs::read_to_string(&labels_file)?;
        let labels: Vec<Label> = serde_yaml::from_str(&content)?;

        // Check for duplicate names and IDs
        let mut names = HashMap::new();
        let mut ids = HashMap::new();
        for label in &labels {
            if let Some(existing) = names.insert(&label.name, 1) {
                return Err(anyhow!("Duplicate label name: {}", label.name));
            }
            if let Some(existing) = ids.insert(&label.id, 1) {
                return Err(anyhow!("Duplicate label ID: {}", label.id));
            }
        }

        // Validate each label
        for label in &labels {
            if let Err(e) = label.validate() {
                return Err(anyhow!("Label validation failed for {}: {}", label.name, e));
            }
        }

        Ok(())
    }

    /// Validate assignments
    fn validate_assignments(&self) -> Result<()> {
        // Validate tag assignments
        let tag_assignments_file = self.config_dir.join("tag_assignments.yaml");
        if tag_assignments_file.exists() {
            let content = std::fs::read_to_string(&tag_assignments_file)?;
            let assignments: Vec<TagAssignment> = serde_yaml::from_str(&content)?;

            for assignment in &assignments {
                if let Err(e) = assignment.validate() {
                    return Err(anyhow!("Tag assignment validation failed: {}", e));
                }
            }
        }

        // Validate label assignments
        let label_assignments_file = self.config_dir.join("label_assignments.yaml");
        if label_assignments_file.exists() {
            let content = std::fs::read_to_string(&label_assignments_file)?;
            let assignments: Vec<LabelAssignment> = serde_yaml::from_str(&content)?;

            // Check for label uniqueness
            let mut assigned_labels = HashMap::new();
            for assignment in &assignments {
                if let Err(e) = assignment.validate() {
                    return Err(anyhow!("Label assignment validation failed: {}", e));
                }

                if let Some(existing) = assigned_labels.insert(&assignment.label_id, 1) {
                    return Err(anyhow!("Label '{}' is assigned multiple times", assignment.label_id));
                }
            }
        }

        Ok(())
    }

    /// Create configuration backup
    pub fn create_backup(&self) -> Result<PathBuf> {
        let backup_dir = self.config_dir.parent()
            .ok_or_else(|| anyhow!("Cannot determine backup directory"))?
            .join("aicred-backups");

        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("backup_{}", timestamp));

        if self.config_dir.exists() {
            std::fs::create_dir_all(&backup_path)?;
            copy_dir_all(&self.config_dir, &backup_path)?;
        }

        // Create backup metadata
        let metadata = BackupMetadata {
            timestamp: Utc::now(),
            version: "0.2.0".to_string(),
            backup_type: "pre_migration".to_string(),
            original_config_dir: self.config_dir.clone(),
        };

        let metadata_file = backup_path.join("metadata.json");
        let metadata_content = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(&metadata_file, metadata_content)?;

        Ok(backup_path)
    }

    /// Restore configuration from backup
    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        let backup_config_dir = backup_path.join("aicred");
        if !backup_config_dir.exists() {
            return Err(anyhow!("Backup configuration not found"));
        }

        // Remove current configuration
        if self.config_dir.exists() {
            std::fs::remove_dir_all(&self.config_dir)?;
        }

        // Restore from backup
        std::fs::create_dir_all(self.config_dir.parent().unwrap())?;
        copy_dir_all(&backup_config_dir, &self.config_dir)?;

        Ok(())
    }
}

/// Helper function to copy directories recursively
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

/// Validation report for configuration integrity
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_info(&mut self, info: String) {
        self.info.push(info);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Backup metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub backup_type: String,
    pub original_config_dir: PathBuf,
}

/// Migration statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStats {
    pub provider_configs_migrated: usize,
    pub tags_created: usize,
    pub labels_created: usize,
    pub auto_assignments_created: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl MigrationStats {
    pub fn new() -> Self {
        Self {
            provider_configs_migrated: 0,
            tags_created: 0,
            labels_created: 0,
            auto_assignments_created: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compatibility_layer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = PathBuf::from(temp_dir.path());
        
        let layer = CompatibilityLayer::new(config_dir);
        assert_eq!(layer.config_dir, config_dir);
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());
        assert!(!report.has_warnings());

        report.add_error("Test error".to_string());
        assert!(!report.is_valid());

        report.add_warning("Test warning".to_string());
        assert!(report.has_warnings());
    }

    #[test]
    fn test_migration_stats() {
        let mut stats = MigrationStats::new();
        assert_eq!(stats.provider_configs_migrated, 0);

        stats.provider_configs_migrated = 5;
        stats.tags_created = 3;
        stats.labels_created = 2;
        stats.auto_assignments_created = 10;

        assert_eq!(stats.provider_configs_migrated, 5);
        assert_eq!(stats.tags_created, 3);
        assert_eq!(stats.labels_created, 2);
        assert_eq!(stats.auto_assignments_created, 10);
    }
}