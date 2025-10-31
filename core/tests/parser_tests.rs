use genai_keyfinder_core::parser::{ConfigParser, FileFormat};
use std::path::Path;

#[test]
fn test_toml_detection_with_table_array() {
    let toml_content = r#"
[[servers]]
host = "example.com"
port = 8080

[database]
server = "192.168.1.1"
ports = [8001, 8001, 8002]
"#;

    let path = Path::new("test.toml");
    let format = ConfigParser::detect_format(path, toml_content).unwrap();
    assert_eq!(format, FileFormat::Toml);
}

#[test]
fn test_toml_detection_with_dotted_keys() {
    let toml_content = r#"
server.host = "example.com"
server.port = 8080
database.server = "192.168.1.1"
"#;

    let path = Path::new("test.toml");
    let format = ConfigParser::detect_format(path, toml_content).unwrap();
    assert_eq!(format, FileFormat::Toml);
}

#[test]
fn test_ini_detection_with_sections() {
    let ini_content = r#"
[section1]
key1 = value1
key2 = value2

[section2]
key3 = value3
"#;

    let path = Path::new("test.ini");
    let format = ConfigParser::detect_format(path, ini_content).unwrap();
    assert_eq!(format, FileFormat::Ini);
}

#[test]
fn test_ambiguous_content_defaults_to_ini() {
    // Content that could be either but should be INI since no TOML-specific markers
    let ambiguous_content = r#"
[section]
key = value
"#;

    let path = Path::new("test.conf");
    let format = ConfigParser::detect_format(path, ambiguous_content).unwrap();
    assert_eq!(format, FileFormat::Ini);
}

#[test]
fn test_toml_with_quoted_keys() {
    let toml_content = r#"
"quoted.key" = "value"
"another key" = "value2"
"#;

    let path = Path::new("test.toml");
    let format = ConfigParser::detect_format(path, toml_content).unwrap();
    assert_eq!(format, FileFormat::Toml);
}

#[test]
fn test_json_array_handling() {
    let json_content = r#"
{
    "servers": [
        {"host": "server1.com", "port": 8080},
        {"host": "server2.com", "port": 8081}
    ],
    "api_keys": ["key1", "key2", "key3"]
}
"#;

    let path = Path::new("test.json");
    let format = ConfigParser::detect_format(path, json_content).unwrap();
    assert_eq!(format, FileFormat::Json);

    // Test that arrays are properly parsed
    let result = ConfigParser::parse_config(path, json_content).unwrap();
    assert!(result.contains_key("servers[0].host"));
    assert!(result.contains_key("servers[1].host"));
    assert!(result.contains_key("api_keys[0]"));
    assert!(result.contains_key("api_keys[1]"));
    assert!(result.contains_key("api_keys[2]"));
}
