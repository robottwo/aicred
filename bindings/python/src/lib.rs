use genai_keyfinder_core::{scan, ScanOptions};
// TODO: Core types will be mapped to Py* wrapper types when implementing full functionality
// Currently only scan and ScanOptions are used directly
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Token cost tracking for model usage.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTokenCost {
    #[pyo3(get, set)]
    pub input_cost_per_million: Option<f64>,
    #[pyo3(get, set)]
    pub output_cost_per_million: Option<f64>,
    #[pyo3(get, set)]
    pub cached_input_cost_modifier: Option<f64>,
}

#[pymethods]
impl PyTokenCost {
    #[new]
    fn new(
        input_cost_per_million: Option<f64>,
        output_cost_per_million: Option<f64>,
        cached_input_cost_modifier: Option<f64>,
    ) -> Self {
        Self {
            input_cost_per_million,
            output_cost_per_million,
            cached_input_cost_modifier,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "TokenCost(input_cost_per_million={:?}, output_cost_per_million={:?}, cached_input_cost_modifier={:?})",
            self.input_cost_per_million, self.output_cost_per_million, self.cached_input_cost_modifier
        )
    }
}

/// Model capabilities and features.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyCapabilities {
    #[pyo3(get, set)]
    pub text_generation: bool,
    #[pyo3(get, set)]
    pub image_generation: bool,
    #[pyo3(get, set)]
    pub audio_processing: bool,
    #[pyo3(get, set)]
    pub video_processing: bool,
    #[pyo3(get, set)]
    pub code_generation: bool,
    #[pyo3(get, set)]
    pub function_calling: bool,
    #[pyo3(get, set)]
    pub fine_tuning: bool,
    #[pyo3(get, set)]
    pub streaming: bool,
    #[pyo3(get, set)]
    pub multimodal: bool,
    #[pyo3(get, set)]
    pub tool_use: bool,
    #[pyo3(get, set)]
    pub custom: Option<HashMap<String, PyObject>>,
}

#[pymethods]
impl PyCapabilities {
    #[new]
    fn new(
        text_generation: bool,
        image_generation: bool,
        audio_processing: bool,
        video_processing: bool,
        code_generation: bool,
        function_calling: bool,
        fine_tuning: bool,
        streaming: bool,
        multimodal: bool,
        tool_use: bool,
    ) -> Self {
        Self {
            text_generation,
            image_generation,
            audio_processing,
            video_processing,
            code_generation,
            function_calling,
            fine_tuning,
            streaming,
            multimodal,
            tool_use,
            custom: None,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Capabilities(text_generation={}, image_generation={}, audio_processing={}, video_processing={}, code_generation={}, function_calling={}, fine_tuning={}, streaming={}, multimodal={}, tool_use={})",
            self.text_generation, self.image_generation, self.audio_processing, self.video_processing,
            self.code_generation, self.function_calling, self.fine_tuning, self.streaming,
            self.multimodal, self.tool_use
        )
    }
}

/// Enhanced AI model configuration with temperature, tags, and cost tracking.
#[pyclass]
#[derive(Debug)]
pub struct PyModel {
    #[pyo3(get, set)]
    pub model_id: String,
    #[pyo3(get, set)]
    pub provider_instance_id: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub quantization: Option<String>,
    #[pyo3(get, set)]
    pub context_window: Option<u32>,
    #[pyo3(get, set)]
    pub capabilities: Option<PyCapabilities>,
    #[pyo3(get, set)]
    pub temperature: Option<f32>,
    #[pyo3(get, set)]
    pub tags: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub cost: Option<PyTokenCost>,
    #[pyo3(get, set)]
    pub metadata: Option<HashMap<String, Py<PyAny>>>,
}

#[pymethods]
impl PyModel {
    #[new]
    fn new(model_id: String, provider_instance_id: String, name: String) -> Self {
        Self {
            model_id,
            provider_instance_id,
            name,
            quantization: None,
            context_window: None,
            capabilities: None,
            temperature: None,
            tags: None,
            cost: None,
            metadata: None,
        }
    }

    fn validate(&self) -> PyResult<()> {
        if self.model_id.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Model ID cannot be empty"));
        }
        if self.provider_instance_id.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Provider instance ID cannot be empty"));
        }
        if self.name.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Model name cannot be empty"));
        }
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Temperature must be between 0.0 and 2.0"));
            }
        }
        if let Some(window) = self.context_window {
            if window == 0 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Context window cannot be zero"));
            }
        }
        Ok(())
    }

    fn supports_text_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .map(|caps| caps.text_generation)
            .unwrap_or(false)
    }

    fn supports_image_generation(&self) -> bool {
        self.capabilities
            .as_ref()
            .map(|caps| caps.image_generation)
            .unwrap_or(false)
    }

    fn __repr__(&self) -> String {
        format!(
            "Model(model_id='{}', provider_instance_id='{}', name='{}', temperature={:?}, tags={:?})",
            self.model_id, self.provider_instance_id, self.name, self.temperature, self.tags
        )
    }
}

/// Provider instance configuration with enhanced metadata and model management.
#[pyclass]
#[derive(Debug)]
pub struct PyProviderInstance {
    pub id: String,
    pub display_name: String,
    pub provider_type: String,
    pub base_url: String,
    pub keys: Option<Vec<Py<PyAny>>>,
    pub models: Vec<PyModel>,
    pub metadata: Option<HashMap<String, String>>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[pymethods]
impl PyProviderInstance {
    #[new]
    fn new(id: String, display_name: String, provider_type: String, base_url: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            display_name,
            provider_type,
            base_url,
            keys: None,
            models: Vec::new(),
            metadata: None,
            active: true,
            created_at: now.clone(),
            updated_at: Some(now),
        }
    }

    fn add_key(&mut self, key: PyObject) {
        if self.keys.is_none() {
            self.keys = Some(Vec::new());
        }
        self.keys.as_mut().unwrap().push(key);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_keys(&mut self, keys: Vec<PyObject>) {
        if self.keys.is_none() {
            self.keys = Some(Vec::new());
        }
        self.keys.as_mut().unwrap().extend(keys);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_model(&mut self, model: &PyModel) {
        self.models.push(PyModel {
            model_id: model.model_id.clone(),
            provider_instance_id: model.provider_instance_id.clone(),
            name: model.name.clone(),
            quantization: model.quantization.clone(),
            context_window: model.context_window,
            capabilities: None, // Simplified for now
            temperature: model.temperature,
            tags: model.tags.clone(),
            cost: None, // Simplified for now
            metadata: None, // Simplified for now
        });
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_models(&mut self, models: Vec<PyObject>) -> PyResult<()> {
        // Simplified approach - just add models one by one
        for _model_obj in models {
            // For now, we'll skip complex extraction and just add a placeholder
            // In a real implementation, you'd extract the model properly
            let placeholder = PyModel::new(
                "placeholder".to_string(),
                self.id.clone(),
                "placeholder".to_string()
            );
            self.models.push(placeholder);
        }
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    fn key_count(&self) -> usize {
        self.keys.as_ref().map(|keys| keys.len()).unwrap_or(0)
    }

    fn model_count(&self) -> usize {
        self.models.len()
    }

    fn validate(&self) -> PyResult<()> {
        if self.id.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Instance ID cannot be empty"));
        }
        if self.display_name.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Display name cannot be empty"));
        }
        if self.provider_type.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Provider type cannot be empty"));
        }
        if self.base_url.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Base URL cannot be empty"));
        }

        // Validate models
        for model in &self.models {
            model.validate()?;
        }

        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "ProviderInstance(id='{}', display_name='{}', provider_type='{}', base_url='{}', active={}, models={})",
            self.id, self.display_name, self.provider_type, self.base_url, self.active, self.models.len()
        )
    }
}

/// Collection of provider instances with lookup and filtering capabilities.
#[pyclass]
#[derive(Debug)]
pub struct PyProviderInstances {
    instances: HashMap<String, PyProviderInstance>,
}

#[pymethods]
impl PyProviderInstances {
    #[new]
    fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    fn add_instance(&mut self, instance: &PyProviderInstance) -> PyResult<()> {
        if self.instances.contains_key(&instance.id) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Provider instance with ID '{}' already exists", instance.id)
            ));
        }
        
        self.instances.insert(instance.id.clone(), PyProviderInstance {
            id: instance.id.clone(),
            display_name: instance.display_name.clone(),
            provider_type: instance.provider_type.clone(),
            base_url: instance.base_url.clone(),
            keys: None, // Simplified - keys will be added separately
            models: Vec::new(), // Will be added separately
            metadata: instance.metadata.clone(),
            active: instance.active,
            created_at: instance.created_at.clone(),
            updated_at: instance.updated_at.clone(),
        });
        Ok(())
    }

    fn add_or_replace_instance(&mut self, instance: &PyProviderInstance) {
        self.instances.insert(instance.id.clone(), PyProviderInstance {
            id: instance.id.clone(),
            display_name: instance.display_name.clone(),
            provider_type: instance.provider_type.clone(),
            base_url: instance.base_url.clone(),
            keys: None, // Simplified - keys will be added separately
            models: Vec::new(), // Will be added separately
            metadata: instance.metadata.clone(),
            active: instance.active,
            created_at: instance.created_at.clone(),
            updated_at: instance.updated_at.clone(),
        });
    }

    fn get_instance(&self, id: &str) -> Option<PyProviderInstance> {
        self.instances.get(id).map(|instance| PyProviderInstance {
            id: instance.id.clone(),
            display_name: instance.display_name.clone(),
            provider_type: instance.provider_type.clone(),
            base_url: instance.base_url.clone(),
            keys: None, // Simplified - keys will be added separately
            models: Vec::new(), // Simplified - models will be added separately
            metadata: instance.metadata.clone(),
            active: instance.active,
            created_at: instance.created_at.clone(),
            updated_at: instance.updated_at.clone(),
        })
    }

    fn remove_instance(&mut self, id: &str) -> Option<PyProviderInstance> {
        self.instances.remove(id)
    }

    fn all_instances(&self) -> Vec<PyProviderInstance> {
        self.instances.values().map(|instance| PyProviderInstance {
            id: instance.id.clone(),
            display_name: instance.display_name.clone(),
            provider_type: instance.provider_type.clone(),
            base_url: instance.base_url.clone(),
            keys: None, // Simplified - keys will be added separately
            models: Vec::new(), // Simplified - models will be added separately
            metadata: instance.metadata.clone(),
            active: instance.active,
            created_at: instance.created_at.clone(),
            updated_at: instance.updated_at.clone(),
        }).collect()
    }

    fn instances_by_type(&self, provider_type: &str) -> Vec<PyProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.provider_type == provider_type)
            .map(|instance| PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None, // Simplified - keys will be added separately
                models: Vec::new(), // Simplified - models will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            })
            .collect()
    }

    fn active_instances(&self) -> Vec<PyProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.active)
            .map(|instance| PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None, // Simplified - keys will be added separately
                models: Vec::new(), // Simplified - models will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            })
            .collect()
    }

    fn active_instances_by_type(&self, provider_type: &str) -> Vec<PyProviderInstance> {
        self.instances
            .values()
            .filter(|instance| instance.active && instance.provider_type == provider_type)
            .map(|instance| PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None, // Simplified - keys will be added separately
                models: Vec::new(), // Simplified - models will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            })
            .collect()
    }

    fn len(&self) -> usize {
        self.instances.len()
    }

    fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    fn instance_ids(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }

    fn provider_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.instances
            .values()
            .map(|instance| instance.provider_type.clone())
            .collect();
        types.sort();
        types.dedup();
        types
    }

    fn validate(&self) -> PyResult<()> {
        let mut errors = Vec::new();
        
        for instance in self.instances.values() {
            if let Err(e) = instance.validate() {
                errors.push(format!("Instance '{}': {}", instance.id, e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(errors.join("; ")))
        }
    }

    fn clear(&mut self) {
        self.instances.clear();
    }

    fn merge(&mut self, other: &PyProviderInstances) {
        for (id, instance) in &other.instances {
            self.instances.insert(id.clone(), PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None, // Simplified - keys will be added separately
                models: Vec::new(), // Will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            });
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProviderInstances(count={}, types={:?})",
            self.len(),
            self.provider_types()
        )
    }
}

/// Migration utilities for converting legacy configurations to new instance-based architecture.
#[pyfunction]
fn migrate_provider_configs(configs: Vec<PyObject>) -> PyResult<PyProviderInstances> {
    let mut instances = PyProviderInstances::new();
    
    // This is a placeholder implementation
    // In a real implementation, we would convert legacy ProviderConfig objects
    // to ProviderInstance objects
    
    Python::with_gil(|py| {
        for config in configs {
            // Try to extract basic information from the config object
            if let Ok(id) = config.getattr(py, "id") {
                if let Ok(id_str) = id.extract::<String>(py) {
                    let instance = PyProviderInstance::new(
                        id_str,
                        "Migrated Instance".to_string(),
                        "unknown".to_string(),
                        "https://api.example.com".to_string(),
                    );
                    let _ = instances.add_instance(&instance);
                }
            }
        }
    });
    
    Ok(instances)
}

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
    m.add_class::<PyTokenCost>()?;
    m.add_class::<PyCapabilities>()?;
    m.add_class::<PyModel>()?;
    m.add_class::<PyProviderInstance>()?;
    m.add_class::<PyProviderInstances>()?;
    
    m.add_function(wrap_pyfunction!(scan_py, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(list_providers, m)?)?;
    m.add_function(wrap_pyfunction!(list_scanners, m)?)?;
    m.add_function(wrap_pyfunction!(migrate_provider_configs, m)?)?;
    
    Ok(())
}
