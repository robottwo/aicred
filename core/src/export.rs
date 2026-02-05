//! Shell environment export functionality with template support
//!
//! This module provides functionality to export discovered AI API keys and configurations
//! as shell environment variables using flexible templates.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Export configuration defining how environment variables should be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Template file path
    pub template_path: Option<String>,

    /// Output format: bash, fish, powershell
    pub format: ExportFormat,

    /// Whether to include full secret values
    pub include_secrets: bool,

    /// Prefix for all exported variables
    pub prefix: Option<String>,

    /// Custom variable mappings
    pub variables: HashMap<String, String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            template_path: None,
            format: ExportFormat::Bash,
            include_secrets: false,
            prefix: None,
            variables: HashMap::new(),
        }
    }
}

/// Shell format for export statements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Bash,
    Fish,
    PowerShell,
}

impl ExportFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ExportFormat::Bash),
            "zsh" => Ok(ExportFormat::Bash), // bash format works for zsh too
            "fish" => Ok(ExportFormat::Fish),
            "powershell" | "pwsh" => Ok(ExportFormat::PowerShell),
            _ => Err(Error::ExportError(format!(
                "Unsupported format: {}. Supported: bash, fish, powershell",
                s
            ))),
        }
    }
}

/// Template for generating shell export statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTemplate {
    /// Variable definitions
    pub variables: Vec<TemplateVariable>,
    /// Header comments to include
    pub header: Option<String>,
    /// Footer comments to include
    pub footer: Option<String>,
}

/// Variable definition in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,
    /// Variable value (may contain placeholders)
    pub value: String,
    /// Description of the variable
    pub description: Option<String>,
    /// Whether this variable is required
    pub required: bool,
}

impl ExportTemplate {
    /// Load a template from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::ExportError(format!("Failed to read template file: {}", e)))?;

        Self::from_str(&content)
    }

    /// Parse a template from a string
    pub fn from_str(content: &str) -> Result<Self> {
        serde_yaml::from_str(content)
            .map_err(|e| Error::ExportError(format!("Failed to parse template: {}", e)))
    }

    /// Render the template with the provided context
    pub fn render(&self, ctx: &ExportContext, format: ExportFormat) -> Result<String> {
        let mut output = String::new();

        // Add header if present
        if let Some(header) = &self.header {
            for line in header.lines() {
                output.push_str(&format.render_comment(line));
                output.push('\n');
            }
            output.push('\n');
        }

        // Render each variable
        for var in &self.variables {
            let value = self.render_value(&var.value, ctx)?;
            let line = format.render_export(&var.name, &value);
            output.push_str(&line);
            output.push('\n');

            // Add description as comment if present
            if let Some(desc) = &var.description {
                output.push_str(&format.render_comment(desc));
                output.push('\n');
            }
            output.push('\n');
        }

        // Add footer if present
        if let Some(footer) = &self.footer {
            if !output.is_empty() {
                output.push('\n');
            }
            for line in footer.lines() {
                output.push_str(&format.render_comment(line));
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Render a template value with context substitution
    fn render_value(&self, template: &str, ctx: &ExportContext) -> Result<String> {
        let mut result = template.to_string();

        // Replace {{provider.KEY}} style placeholders
        for (provider, vars) in &ctx.provider_vars {
            for (key, value) in vars {
                let placeholder = format!("{{{{{}.{}}}}}", provider, key);
                result = result.replace(&placeholder, value);
            }
        }

        // Replace {{var.NAME}} style placeholders
        for (key, value) in &ctx.custom_vars {
            let placeholder = format!("{{{{var.{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

impl ExportFormat {
    /// Render a comment in this format
    fn render_comment(&self, text: &str) -> String {
        match self {
            ExportFormat::Bash => format!("# {}", text),
            ExportFormat::Fish => format!("# {}", text),
            ExportFormat::PowerShell => format!("# {}", text),
        }
    }

    /// Render an export statement in this format
    fn render_export(&self, name: &str, value: &str) -> String {
        let escaped = match self {
            ExportFormat::Bash => Self::escape_bash(value),
            ExportFormat::Fish => Self::escape_fish(value),
            ExportFormat::PowerShell => Self::escape_powershell(value),
        };

        match self {
            ExportFormat::Bash => format!("export {}='{}'", name, escaped),
            ExportFormat::Fish => format!("set -gx {} '{}'", name, escaped),
            ExportFormat::PowerShell => format!("$env:{} = '{}'", name, escaped),
        }
    }

    /// Escape value for bash/zsh
    fn escape_bash(value: &str) -> String {
        value.replace('\'', "'\\''")
    }

    /// Escape value for fish
    fn escape_fish(value: &str) -> String {
        value.replace('\'', "\\'")
    }

    /// Escape value for PowerShell
    fn escape_powershell(value: &str) -> String {
        value.replace('\'', "''")
    }
}

/// Context for template rendering
#[derive(Debug, Clone, Default)]
pub struct ExportContext {
    /// Provider-specific variables (provider_name -> variable_name -> value)
    pub provider_vars: HashMap<String, HashMap<String, String>>,

    /// Custom variables
    pub custom_vars: HashMap<String, String>,
}

impl ExportContext {
    /// Create a new export context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a provider variable
    pub fn add_provider_var(&mut self, provider: &str, key: &str, value: &str) {
        self.provider_vars
            .entry(provider.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    /// Add a custom variable
    pub fn add_custom_var(&mut self, key: &str, value: &str) {
        self.custom_vars.insert(key.to_string(), value.to_string());
    }
}

/// Generate default export template
pub fn default_template() -> ExportTemplate {
    ExportTemplate {
        header: Some("AI API Keys - Generated by aicred\nDo not commit to version control!".to_string()),
        footer: Some("End of aicred exports".to_string()),
        variables: vec![
            TemplateVariable {
                name: "OPENAI_API_KEY".to_string(),
                value: "{{openai.api_key}}".to_string(),
                description: Some("OpenAI API key".to_string()),
                required: false,
            },
            TemplateVariable {
                name: "ANTHROPIC_API_KEY".to_string(),
                value: "{{anthropic.api_key}}".to_string(),
                description: Some("Anthropic API key".to_string()),
                required: false,
            },
            TemplateVariable {
                name: "GOOGLE_API_KEY".to_string(),
                value: "{{google.api_key}}".to_string(),
                description: Some("Google AI API key".to_string()),
                required: false,
            },
        ],
    }
}

/// Export environment variables without a template
pub fn export_vars(
    vars: &HashMap<String, String>,
    format: ExportFormat,
    include_secrets: bool,
) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format.render_comment("AI API Keys - Generated by aicred"));
    output.push('\n');
    output.push_str(&format.render_comment("Do not commit to version control!"));
    output.push('\n');
    output.push('\n');

    for (key, value) in vars {
        let display_value = if !include_secrets && (key.contains("API_KEY") || key.contains("SECRET")) {
            mask_value(value)
        } else {
            value.clone()
        };

        output.push_str(&format.render_export(key, &display_value));
        output.push('\n');
    }

    Ok(output)
}

/// Mask a sensitive value for display
fn mask_value(value: &str) -> String {
    if value.len() > 8 {
        format!("{}***{}", &value[..4], &value[value.len() - 4..])
    } else {
        "****".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_str() {
        assert!(matches!(ExportFormat::from_str("bash").unwrap(), ExportFormat::Bash));
        assert!(matches!(ExportFormat::from_str("fish").unwrap(), ExportFormat::Fish));
        assert!(matches!(ExportFormat::from_str("powershell").unwrap(), ExportFormat::PowerShell));
        assert!(ExportFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_export_context() {
        let mut ctx = ExportContext::new();
        ctx.add_provider_var("openai", "api_key", "sk-test123");
        ctx.add_custom_var("custom_var", "value");

        assert_eq!(
            ctx.provider_vars.get("openai").unwrap().get("api_key").unwrap(),
            "sk-test123"
        );
        assert_eq!(ctx.custom_vars.get("custom_var").unwrap(), "value");
    }

    #[test]
    fn test_escape_bash() {
        assert_eq!(ExportFormat::Bash.escape_bash("test'value"), "test'\\''value");
    }

    #[test]
    fn test_escape_fish() {
        assert_eq!(ExportFormat::Fish.escape_fish("test'value"), "test\\'value");
    }

    #[test]
    fn test_escape_powershell() {
        assert_eq!(ExportFormat::PowerShell.escape_powershell("test'value"), "test''value");
    }

    #[test]
    fn test_mask_value() {
        assert_eq!(mask_value("sk-test123"), "sk***123");
        assert_eq!(mask_value("short"), "****");
    }
}
