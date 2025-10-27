use genai_keyfinder_core::{scan, ScanOptions as CoreScanOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanOptions {
    pub home_dir: Option<String>,
    pub include_full_values: bool,
    pub max_file_size: usize,
    pub only_providers: Option<Vec<String>>,
    pub exclude_providers: Option<Vec<String>>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            perform_scan,
            get_providers,
            get_scanners,
            get_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
