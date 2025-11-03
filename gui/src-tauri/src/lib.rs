use aicred_core::{scan, ScanOptions as CoreScanOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Import CLI command functions
use aicred_cli::commands::labels::{
    handle_add_label, handle_assign_label, handle_remove_label, handle_unassign_label,
    handle_update_label, load_label_assignments, load_labels,
};
use aicred_cli::commands::tags::{
    handle_add_tag, handle_assign_tag, handle_remove_tag, handle_unassign_tag, handle_update_tag,
    load_tag_assignments, load_tags,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanOptions {
    pub home_dir: Option<String>,
    pub include_full_values: bool,
    pub max_file_size: usize,
    pub only_providers: Option<Vec<String>>,
    pub exclude_providers: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagFormData {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelFormData {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentData {
    pub tag_name: Option<String>,
    pub label_name: Option<String>,
    pub instance_id: String,
    pub model_id: Option<String>,
}

#[tauri::command]
fn perform_scan(options: ScanOptions) -> Result<String, String> {
    let core_options = CoreScanOptions {
        home_dir: options.home_dir.map(std::path::PathBuf::from),
        include_full_values: options.include_full_values,
        max_file_size: options.max_file_size,
        only_providers: options.only_providers,
        exclude_providers: options.exclude_providers,
    };

    match scan(&core_options) {
        Ok(result) => {
            serde_json::to_string(&result).map_err(|e| format!("Failed to serialize result: {}", e))
        }
        Err(e) => Err(format!("Scan failed: {}", e)),
    }
}

#[tauri::command]
fn get_providers() -> Vec<String> {
    vec![
        "openai".to_string(),
        "anthropic".to_string(),
        "huggingface".to_string(),
        "ollama".to_string(),
        "litellm".to_string(),
    ]
}

#[tauri::command]
fn get_scanners() -> Vec<String> {
    vec![
        "roo-code".to_string(),
        "claude-desktop".to_string(),
        "ragit".to_string(),
        "langchain".to_string(),
    ]
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Tag management commands
#[tauri::command]
fn list_tags() -> Result<String, String> {
    match load_tags() {
        Ok(tags) => {
            serde_json::to_string(&tags).map_err(|e| format!("Failed to serialize tags: {}", e))
        }
        Err(e) => Err(format!("Failed to load tags: {}", e)),
    }
}

#[tauri::command]
fn add_tag(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    match handle_add_tag(name, color, description) {
        Ok(_) => Ok("Tag added successfully".to_string()),
        Err(e) => Err(format!("Failed to add tag: {}", e)),
    }
}

#[tauri::command]
fn update_tag(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    match handle_update_tag(name, color, description) {
        Ok(_) => Ok("Tag updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update tag: {}", e)),
    }
}

#[tauri::command]
fn remove_tag(name: String, force: bool) -> Result<String, String> {
    match handle_remove_tag(name, force) {
        Ok(_) => Ok("Tag removed successfully".to_string()),
        Err(e) => Err(format!("Failed to remove tag: {}", e)),
    }
}

#[tauri::command]
fn assign_tag(
    tag_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<String, String> {
    match handle_assign_tag(tag_name, instance_id, model_id) {
        Ok(_) => Ok("Tag assigned successfully".to_string()),
        Err(e) => Err(format!("Failed to assign tag: {}", e)),
    }
}

#[tauri::command]
fn unassign_tag(
    tag_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<String, String> {
    match handle_unassign_tag(tag_name, instance_id, model_id) {
        Ok(_) => Ok("Tag unassigned successfully".to_string()),
        Err(e) => Err(format!("Failed to unassign tag: {}", e)),
    }
}

#[tauri::command]
fn list_tag_assignments() -> Result<String, String> {
    match load_tag_assignments() {
        Ok(assignments) => serde_json::to_string(&assignments)
            .map_err(|e| format!("Failed to serialize tag assignments: {}", e)),
        Err(e) => Err(format!("Failed to load tag assignments: {}", e)),
    }
}

// Label management commands
#[tauri::command]
fn list_labels() -> Result<String, String> {
    match load_labels() {
        Ok(labels) => {
            serde_json::to_string(&labels).map_err(|e| format!("Failed to serialize labels: {}", e))
        }
        Err(e) => Err(format!("Failed to load labels: {}", e)),
    }
}

#[tauri::command]
fn add_label(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    match handle_add_label(name, color, description) {
        Ok(_) => Ok("Label added successfully".to_string()),
        Err(e) => Err(format!("Failed to add label: {}", e)),
    }
}

#[tauri::command]
fn update_label(
    name: String,
    color: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    match handle_update_label(name, color, description) {
        Ok(_) => Ok("Label updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update label: {}", e)),
    }
}

#[tauri::command]
fn remove_label(name: String, force: bool) -> Result<String, String> {
    match handle_remove_label(name, force) {
        Ok(_) => Ok("Label removed successfully".to_string()),
        Err(e) => Err(format!("Failed to remove label: {}", e)),
    }
}

#[tauri::command]
fn assign_label(
    label_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<String, String> {
    match handle_assign_label(label_name, instance_id, model_id) {
        Ok(_) => Ok("Label assigned successfully".to_string()),
        Err(e) => Err(format!("Failed to assign label: {}", e)),
    }
}

#[tauri::command]
fn unassign_label(
    label_name: String,
    instance_id: Option<String>,
    model_id: Option<String>,
) -> Result<String, String> {
    match handle_unassign_label(label_name, instance_id, model_id) {
        Ok(_) => Ok("Label unassigned successfully".to_string()),
        Err(e) => Err(format!("Failed to unassign label: {}", e)),
    }
}

#[tauri::command]
fn list_label_assignments() -> Result<String, String> {
    match load_label_assignments() {
        Ok(assignments) => serde_json::to_string(&assignments)
            .map_err(|e| format!("Failed to serialize label assignments: {}", e)),
        Err(e) => Err(format!("Failed to load label assignments: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            perform_scan,
            get_providers,
            get_scanners,
            get_version,
            // Tag commands
            list_tags,
            add_tag,
            update_tag,
            remove_tag,
            assign_tag,
            unassign_tag,
            list_tag_assignments,
            // Label commands
            list_labels,
            add_label,
            update_label,
            remove_label,
            assign_label,
            unassign_label,
            list_label_assignments
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
