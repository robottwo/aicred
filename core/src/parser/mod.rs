//! Parser module for detecting and parsing configuration file formats.

use crate::error::{Error, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

/// Supported configuration file formats.
#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {
    /// JSON format.
    Json,
    /// YAML format.
    Yaml,
    /// TOML format.
    Toml,
    /// INI format.
    Ini,
    /// Dotenv format.
    Dotenv,
    /// Plain text format.
    Plain,
}

/// Configuration file parser.
pub struct ConfigParser;

impl ConfigParser {
    /// Detects the format of a configuration file.
    pub fn detect_format(path: &Path, content: &str) -> Result<FileFormat> {
        // First try to detect from file extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "json" => return Ok(FileFormat::Json),
                "yaml" | "yml" => return Ok(FileFormat::Yaml),
                "toml" => return Ok(FileFormat::Toml),
                "ini" => return Ok(FileFormat::Ini),
                "env" => return Ok(FileFormat::Dotenv),
                _ => {}
            }
        }

        // Try to detect from content
        Self::detect_format_from_content(content)
    }

    /// Detects format based on file content.
    fn detect_format_from_content(content: &str) -> Result<FileFormat> {
        let trimmed = content.trim();

        // JSON detection
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            // Try to parse as JSON
            if serde_json::from_str::<JsonValue>(trimmed).is_ok() {
                return Ok(FileFormat::Json);
            }
        }

        // YAML detection - look for YAML-like patterns
        if trimmed.contains(':') && (trimmed.contains('\n') || trimmed.contains("---")) {
            // Basic YAML pattern detection
            let lines: Vec<&str> = trimmed.lines().collect();
            if lines.iter().any(|line| {
                let line = line.trim();
                line.contains(':') && !line.starts_with('#') && !line.starts_with('[')
            }) {
                return Ok(FileFormat::Yaml);
            }
        }

        // TOML detection - look for TOML-like patterns
        if trimmed.contains('=') && trimmed.contains('[') && trimmed.contains(']') {
            return Ok(FileFormat::Toml);
        }

        // INI detection - look for INI-like patterns
        if trimmed.contains('[') && trimmed.contains(']') && trimmed.contains('=') {
            return Ok(FileFormat::Ini);
        }

        // Dotenv detection - look for KEY=VALUE patterns
        if trimmed.lines().any(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#') && line.contains('=') && !line.contains(' ')
        }) {
            return Ok(FileFormat::Dotenv);
        }

        // Default to plain text
        Ok(FileFormat::Plain)
    }

    /// Parses a configuration file and extracts key-value pairs.
    pub fn parse_config(path: &Path, content: &str) -> Result<HashMap<String, String>> {
        let format = Self::detect_format(path, content)?;
        debug!("Detected format: {:?} for {}", format, path.display());

        match format {
            FileFormat::Json => Self::parse_json(content),
            FileFormat::Yaml => Self::parse_yaml(content),
            FileFormat::Toml => Self::parse_toml(content),
            FileFormat::Ini => Self::parse_ini(content),
            FileFormat::Dotenv => Self::parse_dotenv(content),
            FileFormat::Plain => Self::parse_plain(content),
        }
    }

    /// Parses JSON configuration.
    fn parse_json(content: &str) -> Result<HashMap<String, String>> {
        let json: JsonValue = serde_json::from_str(content).map_err(|e| Error::ParseError {
            path: Path::new("json").to_path_buf(),
            message: format!("Invalid JSON: {}", e),
        })?;

        let mut result = HashMap::new();
        Self::extract_json_values(&json, String::new(), &mut result);
        Ok(result)
    }

    /// Recursively extracts values from JSON.
    fn extract_json_values(
        value: &JsonValue,
        prefix: String,
        result: &mut HashMap<String, String>,
    ) {
        match value {
            JsonValue::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    Self::extract_json_values(val, new_prefix, result);
                }
            }
            JsonValue::String(s) => {
                if !prefix.is_empty() {
                    result.insert(prefix, s.clone());
                }
            }
            JsonValue::Number(n) => {
                if !prefix.is_empty() {
                    result.insert(prefix, n.to_string());
                }
            }
            JsonValue::Bool(b) => {
                if !prefix.is_empty() {
                    result.insert(prefix, b.to_string());
                }
            }
            _ => {}
        }
    }

    /// Parses YAML configuration.
    fn parse_yaml(content: &str) -> Result<HashMap<String, String>> {
        let yaml: JsonValue = serde_yaml::from_str(content).map_err(|e| Error::ParseError {
            path: Path::new("yaml").to_path_buf(),
            message: format!("Invalid YAML: {}", e),
        })?;

        let mut result = HashMap::new();
        Self::extract_json_values(&yaml, String::new(), &mut result);
        Ok(result)
    }

    /// Parses TOML configuration.
    fn parse_toml(content: &str) -> Result<HashMap<String, String>> {
        let toml: JsonValue = toml::from_str(content).map_err(|e| Error::ParseError {
            path: Path::new("toml").to_path_buf(),
            message: format!("Invalid TOML: {}", e),
        })?;

        let mut result = HashMap::new();
        Self::extract_json_values(&toml, String::new(), &mut result);
        Ok(result)
    }

    /// Parses INI configuration.
    fn parse_ini(content: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                continue;
            }

            // Key-value pair
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                let full_key = if current_section.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", current_section, key)
                };

                result.insert(full_key, value.to_string());
            }
        }

        Ok(result)
    }

    /// Parses dotenv format.
    fn parse_dotenv(content: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                result.insert(key.to_string(), value.to_string());
            }
        }

        Ok(result)
    }

    /// Parses plain text format (basic key-value detection).
    fn parse_plain(content: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            // Try to find key-value pairs
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                if !key.is_empty() && !value.is_empty() {
                    result.insert(key.to_string(), value.to_string());
                }
            } else if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                if !key.is_empty() && !value.is_empty() {
                    result.insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Merges multiple configuration maps.
    pub fn merge_configs(configs: Vec<HashMap<String, String>>) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for config in configs {
            for (key, value) in config {
                result.insert(key, value);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_format_from_extension() {
        let json_path = PathBuf::from("test.json");
        let yaml_path = PathBuf::from("test.yaml");
        let toml_path = PathBuf::from("test.toml");

        let content = "{}";

        assert_eq!(
            ConfigParser::detect_format(&json_path, content).unwrap(),
            FileFormat::Json
        );
        assert_eq!(
            ConfigParser::detect_format(&yaml_path, content).unwrap(),
            FileFormat::Yaml
        );
        assert_eq!(
            ConfigParser::detect_format(&toml_path, content).unwrap(),
            FileFormat::Toml
        );
    }

    #[test]
    fn test_detect_format_from_content() {
        let path = PathBuf::from("unknown");

        let json_content = r#"{"key": "value"}"#;
        assert_eq!(
            ConfigParser::detect_format(&path, json_content).unwrap(),
            FileFormat::Json
        );

        let yaml_content = "key: value\nother: value2";
        assert_eq!(
            ConfigParser::detect_format(&path, yaml_content).unwrap(),
            FileFormat::Yaml
        );

        let dotenv_content = "KEY=value\nOTHER=value2";
        assert_eq!(
            ConfigParser::detect_format(&path, dotenv_content).unwrap(),
            FileFormat::Dotenv
        );
    }

    #[test]
    fn test_parse_json() {
        let content = r#"{"api_key": "secret", "nested": {"key": "value"}}"#;
        let result = ConfigParser::parse_json(content).unwrap();

        assert_eq!(result.get("api_key"), Some(&"secret".to_string()));
        assert_eq!(result.get("nested.key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_yaml() {
        let content = "api_key: secret\nnested:\n  key: value";
        let result = ConfigParser::parse_yaml(content).unwrap();

        assert_eq!(result.get("api_key"), Some(&"secret".to_string()));
        assert_eq!(result.get("nested.key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_ini() {
        let content = "[section]\nkey = value\nother = value2";
        let result = ConfigParser::parse_ini(content).unwrap();

        assert_eq!(result.get("section.key"), Some(&"value".to_string()));
        assert_eq!(result.get("section.other"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_dotenv() {
        let content = "API_KEY=secret\nOTHER=\"quoted value\"";
        let result = ConfigParser::parse_dotenv(content).unwrap();

        assert_eq!(result.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(result.get("OTHER"), Some(&"quoted value".to_string()));
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = HashMap::new();
        config1.insert("key1".to_string(), "value1".to_string());

        let mut config2 = HashMap::new();
        config2.insert("key2".to_string(), "value2".to_string());

        let merged = ConfigParser::merge_configs(vec![config1, config2]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.get("key1"), Some(&"value1".to_string()));
        assert_eq!(merged.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_empty_file() {
        use std::path::Path;
        let res = ConfigParser::parse_config(Path::new("empty.txt"), "").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_parse_comments_only() {
        use std::path::Path;
        let content = "# comment line\n; another comment\n// js-style\n";
        let res = ConfigParser::parse_config(Path::new("config.ini"), content).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_ambiguous_detection_defaults_to_plain() {
        use std::path::PathBuf;
        let path = PathBuf::from("unknown.config");
        let content = "just some unrelated text without separators";
        let format = ConfigParser::detect_format(&path, content).unwrap();
        assert_eq!(format, FileFormat::Plain);
    }

    #[test]
    fn test_invalid_json_errors() {
        let bad = "{ not-json";
        let err = ConfigParser::parse_json(bad).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("Invalid JSON"));
    }
}
