use genai_keyfinder_core::{scan, ScanOptions};
use pyo3::prelude::*;
use std::path::PathBuf;

/// Scan for GenAI credentials and configurations
///
/// Args:
///     home_dir (str, optional): Home directory to scan. Defaults to user's home.
///     include_full_values (bool): Include full secret values. Default: False
///     max_file_size (int): Maximum file size to read in bytes. Default: 1048576
///     only_providers (list[str], optional): Only scan these providers
///     exclude_providers (list[str], optional): Exclude these providers
///
/// Returns:
///     dict: Scan results with keys and config_instances
///
/// Example:
///     >>> import genai_keyfinder
///     >>> result = genai_keyfinder.scan()
///     >>> print(f"Found {len(result['keys'])} keys")
#[pyfunction]
#[pyo3(signature = (home_dir=None, include_full_values=false, max_file_size=1048576, only_providers=None, exclude_providers=None))]
fn scan_py(
    home_dir: Option<String>,
    include_full_values: bool,
    max_file_size: usize,
    only_providers: Option<Vec<String>>,
    exclude_providers: Option<Vec<String>>,
) -> PyResult<PyObject> {
    let home_path = match home_dir {
        Some(h) => Some(PathBuf::from(h)),
        None => dirs_next::home_dir(),
    };

    let options = ScanOptions {
        home_dir: home_path,
        include_full_values,
        max_file_size,
        only_providers,
        exclude_providers,
    };

    let result = scan(options)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // Convert to JSON and then to Python dict
    let json = serde_json::to_string(&result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Python::with_gil(|py| {
        let json_module = py.import_bound("json")?;
        let loads = json_module.getattr("loads")?;
        loads.call1((json,))?.extract()
    })
}

/// Get library version
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// List available providers
#[pyfunction]
fn list_providers() -> Vec<&'static str> {
    vec![
        "openai",
        "anthropic",
        "huggingface",
        "ollama",
        "langchain",
        "litellm",
    ]
}

/// List available application scanners
#[pyfunction]
fn list_scanners() -> Vec<&'static str> {
    vec!["roo-code", "claude-desktop", "ragit", "langchain-app"]
}

/// GenAI Key Finder - Python bindings
///
/// A library for discovering GenAI API keys and configurations
#[pymodule]
fn genai_keyfinder(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scan_py, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(list_providers, m)?)?;
    m.add_function(wrap_pyfunction!(list_scanners, m)?)?;
    Ok(())
}
