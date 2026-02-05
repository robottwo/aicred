//! Environment variable resolution logic for mapping labels to provider configurations.

use crate::error::Result;
use crate::models::{ProviderInstance, UnifiedLabel};
use crate::scanners::{EnvVarDeclaration, LabelMapping};
use crate::utils::ProviderModelTuple;
use std::collections::HashMap;

/// Environment variable mapping for representing label-to-env-var mappings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarMapping {
    /// Name of the label (e.g., "fast", "smart")
    pub label_name: String,
    /// Name of the environment variable group this label maps to
    pub env_var_group: String,
    /// Description of what this mapping represents
    pub description: String,
}

impl EnvVarMapping {
    /// Creates a new environment variable mapping
    #[must_use]
    pub const fn new(label_name: String, env_var_group: String, description: String) -> Self {
        Self {
            label_name,
            env_var_group,
            description,
        }
    }
}

/// Result of environment variable resolution
#[derive(Debug, Clone)]
pub struct EnvResolutionResult {
    /// Resolved environment variables
    pub variables: HashMap<String, String>,
    /// List of resolved labels
    pub resolved_labels: Vec<String>,
    /// List of unresolved labels
    pub unresolved_labels: Vec<String>,
    /// List of missing required variables
    pub missing_required: Vec<String>,
}

impl EnvResolutionResult {
    /// Creates a new empty resolution result
    #[must_use]
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            resolved_labels: Vec::new(),
            unresolved_labels: Vec::new(),
            missing_required: Vec::new(),
        }
    }

    /// Checks if the resolution was successful (no missing required variables)
    #[must_use]
    pub const fn is_successful(&self) -> bool {
        self.missing_required.is_empty()
    }

    /// Adds a resolved variable
    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// Adds a resolved label (with deduplication)
    pub fn add_resolved_label(&mut self, label: String) {
        // Only add if not already present to avoid duplicates
        if !self.resolved_labels.contains(&label) {
            self.resolved_labels.push(label);
        }
    }

    /// Adds an unresolved label
    pub fn add_unresolved_label(&mut self, label: String) {
        self.unresolved_labels.push(label);
    }

    /// Adds a missing required variable
    pub fn add_missing_required(&mut self, var: String) {
        self.missing_required.push(var);
    }
}

impl Default for EnvResolutionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variable resolver for mapping labels to provider configurations
pub struct EnvResolver {
    /// Available provider instances from scanned configurations
    provider_instances: Vec<ProviderInstance>,
    /// Label assignments
    labels: Vec<UnifiedLabel>,
    /// Environment variable schema from scanner
    env_schema: Vec<EnvVarDeclaration>,
    /// Label mappings from scanner
    label_mappings: Vec<LabelMapping>,
}

impl EnvResolver {
    /// Creates a new environment variable resolver
    #[must_use]
    pub const fn new(
        provider_instances: Vec<ProviderInstance>,
        labels: Vec<UnifiedLabel>,
        env_schema: Vec<EnvVarDeclaration>,
        label_mappings: Vec<LabelMapping>,
    ) -> Self {
        Self {
            provider_instances,
            labels,
            env_schema,
            label_mappings,
        }
    }

    /// Resolves environment variables based on the provided configuration
    ///
    /// # Errors
    /// Returns an error if resolution fails due to missing required variables
    pub fn resolve(&self, dry_run: bool) -> Result<EnvResolutionResult> {
        let mut result = EnvResolutionResult::new();

        // Map labels to provider instances
        let label_to_instance = self.map_labels_to_instances();

        // Process each label mapping
        for mapping in &self.label_mappings {
            if let Some(label) = self
                .labels
                .iter()
                .find(|l| l.label_name == mapping.label_name)
            {
                // Get the provider instance for this label
                if let Some(instance) = label_to_instance.get(&label.label_name) {
                    // Resolve environment variables for this label
                    self.resolve_label_vars(
                        &mapping.env_var_group,
                        instance,
                        &mut result,
                        dry_run,
                        label,
                    );
                    // Only mark as resolved after successful resolution
                    result.add_resolved_label(mapping.label_name.clone());
                } else {
                    result.add_unresolved_label(mapping.label_name.clone());
                }
            }
        }

        // Handle direct mappings when no label mappings are provided
        if self.label_mappings.is_empty() && !self.labels.is_empty() {
            self.resolve_direct_label_mappings(&mut result, dry_run);
        }

        // Validate required variables
        self.validate_required_variables(&mut result);

        Ok(result)
    }

    /// Maps labels to their corresponding provider instances
    fn map_labels_to_instances(&self) -> HashMap<String, &ProviderInstance> {
        let mut mapping = HashMap::new();

        for label in &self.labels {
            // Find provider instance that matches this label's target
            if let Some(instance) = self.find_matching_instance(&label.target) {
                mapping.insert(label.label_name.clone(), instance);
            }
        }

        mapping
    }

    /// Finds a provider instance that matches the given provider:model tuple
    fn find_matching_instance(&self, target: &ProviderModelTuple) -> Option<&ProviderInstance> {
        self.provider_instances.iter().find(|instance| {
            // Match by provider type first
            if instance.provider_type != target.provider {
                return false;
            }

            // If the label has a specific model, check if the instance has that model
            target.model.is_empty()
                || instance
                    .models
                    .iter()
                    .any(|model_id| model_id == &target.model)
        })
    }

    /// Resolves environment variables for a specific label
    fn resolve_label_vars(
        &self,
        env_var_group: &str,
        instance: &ProviderInstance,
        result: &mut EnvResolutionResult,
        dry_run: bool,
        label: &UnifiedLabel,
    ) {
        // Find all environment variables that belong to this group
        let group_vars: Vec<&EnvVarDeclaration> = self
            .env_schema
            .iter()
            .filter(|var| var.name.starts_with(env_var_group))
            .collect();

        for var in group_vars {
            let value = Self::resolve_variable_value(var, instance, dry_run, &label.target);
            if let Some(val) = value {
                result.add_variable(var.name.clone(), val);
            } else if var.required && !dry_run {
                result.add_missing_required(var.name.clone());
            }
        }
    }

    /// Resolves environment variables for labels when no explicit mappings are provided
    fn resolve_direct_label_mappings(&self, result: &mut EnvResolutionResult, dry_run: bool) {
        // Generate default environment variables for each label
        for label in &self.labels {
            if let Some(instance) = self.find_matching_instance(&label.target) {
                // Generate standard environment variables for this label for multiple scanners
                let scanners = vec!["GSH", "ROO_CODE", "CLAUDE_DESKTOP", "RAGIT", "LANGCHAIN"];

                for scanner in scanners {
                    // Replace hyphens with underscores and convert to uppercase
                    let clean_label_name = label.label_name.replace('-', "_").to_uppercase();
                    let prefix = format!("{scanner}_{clean_label_name}");

                    // Model variable
                    let model_var_name = format!("{prefix}_MODEL");
                    result.add_variable(
                        model_var_name,
                        format!("{}:{}", label.target.provider, label.target.model),
                    );

                    // API key variable
                    if let Some(api_key) = instance.get_api_key() {
                        let api_key_var_name = format!("{prefix}_API_KEY");
                        let api_key_value = if dry_run {
                            if api_key.len() > 8 {
                                format!("{}***{}", &api_key[..4], &api_key[api_key.len() - 4..])
                            } else {
                                "****".to_string()
                            }
                        } else {
                            api_key.clone()
                        };
                        result.add_variable(api_key_var_name, api_key_value);
                    }

                    // Base URL variable
                    if !instance.base_url.is_empty() {
                        let base_url_var_name = format!("{prefix}_BASE_URL");
                        result.add_variable(base_url_var_name, instance.base_url.clone());
                    }

                    // Handle metadata variables
                    for (key, value) in &instance.metadata {
                        let meta_var_name = format!("{}_{}", prefix, key.to_uppercase());
                        result.add_variable(meta_var_name, value.clone());
                    }
                }

                result.add_resolved_label(label.label_name.clone());
            } else {
                result.add_unresolved_label(label.label_name.clone());
            }
        }
    }

    /// Resolves the value for a specific environment variable
    fn resolve_variable_value(
        var: &EnvVarDeclaration,
        instance: &ProviderInstance,
        dry_run: bool,
        target: &ProviderModelTuple,
    ) -> Option<String> {
        // Determine the value type based on the variable name
        if var.name.ends_with("_API_KEY") {
            Self::resolve_api_key(var, instance, dry_run)
        } else if var.name.ends_with("_BASE_URL") {
            resolve_base_url(var, instance)
        } else if var.name.ends_with("_MODEL") || var.name.ends_with("_MODEL_ID") {
            resolve_model_id(var, instance, target)
        } else if var.name.ends_with("_TEMPERATURE") {
            resolve_temperature(var, instance)
        } else if var.name.ends_with("_MAX_TOKENS") {
            resolve_max_tokens(var, instance)
        } else if var.name.contains("_PARALLEL_TOOL_CALLS") {
            resolve_parallel_tool_calls(var, instance)
        } else if var.name.contains("_HEADERS") {
            resolve_headers(var, instance)
        } else {
            // For unknown variables, use default value if available
            var.default_value.clone()
        }
    }

    /// Resolves API key value
    fn resolve_api_key(
        _var: &EnvVarDeclaration,
        instance: &ProviderInstance,
        dry_run: bool,
    ) -> Option<String> {
        if dry_run {
            // In dry run mode, return a masked version of the API key
            if let Some(api_key) = instance.get_api_key() {
                return if api_key.len() > 8 {
                    Some(format!(
                        "{}***{}",
                        &api_key[..4],
                        &api_key[api_key.len() - 4..]
                    ))
                } else {
                    Some("****".to_string())
                };
            }
        }

        // In normal mode, return the actual API key
        instance.get_api_key().map(String::from)
    }
}

/// Resolves base URL value
fn resolve_base_url(var: &EnvVarDeclaration, instance: &ProviderInstance) -> Option<String> {
    // Use instance's base URL if available
    if !instance.base_url.is_empty() {
        return Some(instance.base_url.clone());
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

/// Resolves model ID value
fn resolve_model_id(
    var: &EnvVarDeclaration,
    instance: &ProviderInstance,
    target: &ProviderModelTuple,
) -> Option<String> {
    // Use the specific model from the label's target
    if !target.model.is_empty() {
        return Some(format!("{}:{}", target.provider, target.model));
    }

    // Fall back to first model from instance if no specific target
    if let Some(model_id) = instance.models.first() {
        return Some(format!("{}:{}", instance.provider_type, model_id));
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

/// Resolves temperature value
fn resolve_temperature(var: &EnvVarDeclaration, instance: &ProviderInstance) -> Option<String> {
    // Check if temperature is in metadata
    if let Some(temp) = instance.metadata.get("temperature") {
        return Some(temp.clone());
    }
    // Also check for uppercase key
    if let Some(temp) = instance.metadata.get("TEMPERATURE") {
        return Some(temp.clone());
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

/// Resolves max tokens value
fn resolve_max_tokens(var: &EnvVarDeclaration, instance: &ProviderInstance) -> Option<String> {
    // Check if max_tokens is in metadata
    if let Some(tokens) = instance.metadata.get("MAX_TOKENS") {
        return Some(tokens.clone());
    }
    // Also check for lowercase key
    if let Some(tokens) = instance.metadata.get("max_tokens") {
        return Some(tokens.clone());
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

/// Resolves parallel tool calls setting
fn resolve_parallel_tool_calls(
    var: &EnvVarDeclaration,
    instance: &ProviderInstance,
) -> Option<String> {
    // Check if parallel tool calls is in metadata
    if let Some(ptc) = instance.metadata.get("parallel_tool_calls") {
        return Some(ptc.clone());
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

/// Resolves headers value
fn resolve_headers(var: &EnvVarDeclaration, instance: &ProviderInstance) -> Option<String> {
    // Check if headers are in metadata
    if let Some(headers) = instance.metadata.get("headers") {
        return Some(headers.clone());
    }

    // Fall back to default value if specified
    var.default_value.clone()
}

impl EnvResolver {
    /// Validates that all required variables have been resolved
    fn validate_required_variables(&self, result: &mut EnvResolutionResult) {
        for var in &self.env_schema {
            if var.required && !result.variables.contains_key(&var.name) {
                result.add_missing_required(var.name.clone());
            }
        }
    }

    /// Creates an environment variable resolver from environment variable mappings
    ///
    /// This method provides a convenient way to create a resolver directly from
    /// environment variable mappings without needing to construct all the individual
    /// components separately.
    ///
    /// # Arguments
    /// * `provider_instances` - Provider instances from scanned configurations
    /// * `labels` - Label assignments
    /// * `env_var_mappings` - Environment variable mappings for label-to-env-var resolution
    /// * `dry_run` - Whether to run in dry-run mode
    ///
    /// # Returns
    /// A Result containing the environment variable resolution result
    ///
    /// # Errors
    /// Returns an error if resolution fails due to missing required variables or invalid configuration
    pub fn resolve_from_mappings(
        provider_instances: Vec<ProviderInstance>,
        labels: Vec<UnifiedLabel>,
        env_var_mappings: Vec<EnvVarMapping>,
        dry_run: bool,
    ) -> Result<EnvResolutionResult> {
        // Convert EnvVarMapping to LabelMapping
        let label_mappings: Vec<LabelMapping> = env_var_mappings
            .into_iter()
            .map(|mapping| {
                LabelMapping::new(
                    mapping.label_name,
                    mapping.env_var_group,
                    mapping.description,
                )
            })
            .collect();

        // Generate schema for the provided mappings
        let mut env_schema = Vec::new();
        for mapping in &label_mappings {
            let group = &mapping.env_var_group;
            let label = &mapping.label_name;
            env_schema.push(EnvVarDeclaration::required(
                format!("{group}_MODEL"),
                format!("Model for {label}"),
                "Model".to_string(),
            ));
            env_schema.push(EnvVarDeclaration::required(
                format!("{group}_API_KEY"),
                format!("API key for {label}"),
                "ApiKey".to_string(),
            ));
            env_schema.push(EnvVarDeclaration::optional(
                format!("{group}_BASE_URL"),
                format!("Base URL for {label}"),
                "BaseUrl".to_string(),
                None,
            ));
        }

        // Create resolver with generated schema
        let resolver = Self::new(provider_instances, labels, env_schema, label_mappings);

        resolver.resolve(dry_run)
    }
}

/// Builder for creating environment variable resolvers
pub struct EnvResolverBuilder {
    provider_instances: Vec<ProviderInstance>,
    labels: Vec<UnifiedLabel>,
    env_schema: Vec<EnvVarDeclaration>,
    label_mappings: Vec<LabelMapping>,
}

impl EnvResolverBuilder {
    /// Creates a new builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            provider_instances: Vec::new(),
            labels: Vec::new(),
            env_schema: Vec::new(),
            label_mappings: Vec::new(),
        }
    }

    /// Sets the provider instances
    #[must_use]
    pub fn with_provider_instances(mut self, instances: Vec<ProviderInstance>) -> Self {
        self.provider_instances = instances;
        self
    }

    /// Sets the label assignments
    #[must_use]
    pub fn with_labels(mut self, labels: Vec<UnifiedLabel>) -> Self {
        self.labels = labels;
        self
    }

    /// Sets the environment variable schema
    #[must_use]
    pub fn with_env_schema(mut self, schema: Vec<EnvVarDeclaration>) -> Self {
        self.env_schema = schema;
        self
    }

    /// Sets the label mappings
    #[must_use]
    pub fn with_label_mappings(mut self, mappings: Vec<LabelMapping>) -> Self {
        self.label_mappings = mappings;
        self
    }

    /// Builds the environment variable resolver
    #[must_use]
    pub fn build(self) -> EnvResolver {
        // Generate default schema and mappings if not provided
        let (env_schema, label_mappings) =
            if self.env_schema.is_empty() && self.label_mappings.is_empty() {
                Self::generate_default_schema(&self.provider_instances, &self.labels)
            } else {
                (self.env_schema, self.label_mappings)
            };

        EnvResolver::new(
            self.provider_instances,
            self.labels,
            env_schema,
            label_mappings,
        )
    }

    /// Generates default environment variable schema and label mappings
    fn generate_default_schema(
        _provider_instances: &[ProviderInstance],
        labels: &[UnifiedLabel],
    ) -> (Vec<EnvVarDeclaration>, Vec<LabelMapping>) {
        let mut env_schema = Vec::new();
        let mut label_mappings = Vec::new();

        // Create label mappings for each unique label
        let unique_labels: std::collections::HashSet<&str> = labels
            .iter()
            .map(|label| label.label_name.as_str())
            .collect();

        // Define the scanners we want to support by default
        let scanners = vec!["GSH", "ROO_CODE", "CLAUDE_DESKTOP", "RAGIT", "LANGCHAIN"];

        for label_name in unique_labels {
            for scanner in &scanners {
                // Create label mapping for each scanner
                // Convert label name to valid environment variable format (replace hyphens with underscores)
                let clean_label_name = label_name.replace('-', "_").to_uppercase();
                let env_var_group = format!("{scanner}_{clean_label_name}");
                let mapping = LabelMapping::new(
                    label_name.to_string(),
                    env_var_group.clone(),
                    format!("{scanner} {label_name} model configuration"),
                );
                label_mappings.push(mapping);

                // Create environment variable declarations
                env_schema.push(EnvVarDeclaration::required(
                    format!("{env_var_group}_MODEL"),
                    format!("Model for {scanner} {label_name} label"),
                    "Model".to_string(),
                ));
                env_schema.push(EnvVarDeclaration::required(
                    format!("{env_var_group}_API_KEY"),
                    format!("API key for {scanner} {label_name} label"),
                    "ApiKey".to_string(),
                ));
                env_schema.push(EnvVarDeclaration::optional(
                    format!("{env_var_group}_BASE_URL"),
                    format!("Base URL for {scanner} {label_name} label"),
                    "BaseUrl".to_string(),
                    None,
                ));

                // Add metadata variables (these will be populated from instance.metadata if available)
                env_schema.push(EnvVarDeclaration::optional(
                    format!("{env_var_group}_TEMPERATURE"),
                    format!("Temperature for {scanner} {label_name} label"),
                    "Temperature".to_string(),
                    None,
                ));
                env_schema.push(EnvVarDeclaration::optional(
                    format!("{env_var_group}_MAX_TOKENS"),
                    format!("Max tokens for {scanner} {label_name} label"),
                    "MaxTokens".to_string(),
                    None,
                ));
            }
        }

        (env_schema, label_mappings)
    }
}

impl Default for EnvResolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Model, ProviderInstance};

    fn create_test_provider_instance(
        provider_type: &str,
        api_key: &str,
        models: Vec<&str>,
    ) -> ProviderInstance {
        let mut instance = ProviderInstance::new(
            "test-instance".to_string(),
            provider_type.to_string(),
            provider_type.to_string(),
            format!("https://api.{provider_type}.com"),
        );
        instance.set_api_key(api_key.to_string());

        for model_id in models {
            instance.add_model(model_id.to_string());
        }

        instance
    }

    #[test]
    fn test_env_resolver_basic() {
        let provider_instances = vec![create_test_provider_instance(
            "openai",
            "sk-test123",
            vec!["gpt-4"],
        )];

        let labels = vec![UnifiedLabel::new(
            "smart".to_string(),
            ProviderModelTuple::parse("openai:gpt-4").unwrap(),
        )];

        let resolver = EnvResolverBuilder::new()
            .with_provider_instances(provider_instances)
            .with_labels(labels)
            .build();

        let result = resolver.resolve(false).unwrap();

        assert!(result.is_successful());
        assert!(result.variables.len() >= 3);
        assert_eq!(
            result.variables.get("GSH_SMART_MODEL"),
            Some(&"openai:gpt-4".to_string())
        );
        assert_eq!(
            result.variables.get("GSH_SMART_API_KEY"),
            Some(&"sk-test123".to_string())
        );
        assert_eq!(
            result.variables.get("GSH_SMART_BASE_URL"),
            Some(&"https://api.openai.com".to_string())
        );
    }

    #[test]
    fn test_dry_run_mode() {
        let provider_instances = vec![create_test_provider_instance(
            "openai",
            "sk-test123456789",
            vec!["gpt-4"],
        )];

        let labels = vec![UnifiedLabel::new(
            "smart".to_string(),
            ProviderModelTuple::parse("openai:gpt-4").unwrap(),
        )];

        let resolver = EnvResolverBuilder::new()
            .with_provider_instances(provider_instances)
            .with_labels(labels)
            .build();

        let result = resolver.resolve(true).unwrap();

        assert!(result.is_successful());
        let api_key = result.variables.get("GSH_SMART_API_KEY").unwrap();
        assert!(api_key.contains("***"));
    }

    #[test]
    fn test_missing_required_variable() {
        let provider_instances = vec![]; // No instances available

        let labels = vec![UnifiedLabel::new(
            "smart".to_string(),
            ProviderModelTuple::parse("openai:gpt-4").unwrap(),
        )];

        let resolver = EnvResolverBuilder::new()
            .with_provider_instances(provider_instances)
            .with_labels(labels)
            .build();

        let result = resolver.resolve(false).unwrap();

        assert!(!result.is_successful());
        assert_eq!(result.unresolved_labels.len(), 5);
        assert!(result.unresolved_labels.iter().all(|l| l == "smart"));
        assert!(result.unresolved_labels.iter().all(|l| l == "smart"));
    }

    #[test]
    fn test_env_var_mapping_struct() {
        let mapping = EnvVarMapping::new(
            "test_label".to_string(),
            "TEST_GROUP".to_string(),
            "Test description".to_string(),
        );

        assert_eq!(mapping.label_name, "test_label");
        assert_eq!(mapping.env_var_group, "TEST_GROUP");
        assert_eq!(mapping.description, "Test description");
    }

    #[test]
    fn test_resolve_from_mappings() {
        let provider_instances = vec![create_test_provider_instance(
            "openai",
            "sk-test123",
            vec!["gpt-4"],
        )];

        let labels = vec![UnifiedLabel::new(
            "smart".to_string(),
            ProviderModelTuple::parse("openai:gpt-4").unwrap(),
        )];

        let env_var_mappings = vec![EnvVarMapping::new(
            "smart".to_string(),
            "GSH_SMART".to_string(),
            "Smart model configuration".to_string(),
        )];

        let result =
            EnvResolver::resolve_from_mappings(provider_instances, labels, env_var_mappings, false)
                .unwrap();

        assert!(result.resolved_labels.contains(&"smart".to_string()));
        assert!(!result.variables.is_empty());
        assert!(result.variables.contains_key("GSH_SMART_API_KEY"));
    }
}
