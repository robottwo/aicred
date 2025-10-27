// Allow clippy lints for Python bindings
#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::manual_range_contains)]

use genai_keyfinder_core::{scan as core_scan, ScanOptions};
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
    // Remove custom field for now to avoid Clone issues
    // pub custom: Option<HashMap<String, Py<PyAny>>>,
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
            // custom: None,
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
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Model ID cannot be empty",
            ));
        }
        if self.provider_instance_id.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Provider instance ID cannot be empty",
            ));
        }
        if self.name.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Model name cannot be empty",
            ));
        }
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Temperature must be between 0.0 and 2.0",
                ));
            }
        }
        if let Some(window) = self.context_window {
            if window == 0 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Context window cannot be zero",
                ));
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

    fn add_key(&mut self, key: Py<PyAny>) {
        if self.keys.is_none() {
            self.keys = Some(Vec::new());
        }
        self.keys.as_mut().unwrap().push(key);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_keys(&mut self, keys: Vec<Py<PyAny>>) {
        if self.keys.is_none() {
            self.keys = Some(Vec::new());
        }
        self.keys.as_mut().unwrap().extend(keys);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_model(&mut self, model: &PyModel) {
        let metadata = Python::with_gil(|py| {
            model.metadata.as_ref().map(|meta| {
                meta.iter()
                    .map(|(k, v)| (k.clone(), v.clone_ref(py)))
                    .collect::<HashMap<String, Py<PyAny>>>()
            })
        });

        self.models.push(PyModel {
            model_id: model.model_id.clone(),
            provider_instance_id: model.provider_instance_id.clone(),
            name: model.name.clone(),
            quantization: model.quantization.clone(),
            context_window: model.context_window,
            capabilities: model.capabilities.clone(),
            temperature: model.temperature,
            tags: model.tags.clone(),
            cost: model.cost.clone(),
            metadata,
        });
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    fn add_models(&mut self, models: Vec<Py<PyAny>>) -> PyResult<()> {
        // Acquire the Python GIL to work with Py<PyAny> objects
        Python::with_gil(|py| {
            for model_obj in models {
                // Extract the Python Model wrapper from the Py<PyAny> object as Py<Model>
                let model: Py<Model> = model_obj.extract(py)?;

                // Extract the inner PyModel from the Model wrapper
                let py_model_ref = model.borrow(py);

                // Call add_model to properly handle the cloning with cost and metadata
                self.add_model(&py_model_ref.0);
            }

            Ok(())
        })
    }

    fn key_count(&self) -> usize {
        self.keys.as_ref().map(|keys| keys.len()).unwrap_or(0)
    }

    fn model_count(&self) -> usize {
        self.models.len()
    }

    fn validate(&self) -> PyResult<()> {
        if self.id.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Instance ID cannot be empty",
            ));
        }
        if self.display_name.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Display name cannot be empty",
            ));
        }
        if self.provider_type.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Provider type cannot be empty",
            ));
        }
        if self.base_url.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Base URL cannot be empty",
            ));
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
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Provider instance with ID '{}' already exists",
                instance.id
            )));
        }

        self.instances.insert(
            instance.id.clone(),
            PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None,         // Simplified - keys will be added separately
                models: Vec::new(), // Will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            },
        );
        Ok(())
    }

    fn add_or_replace_instance(&mut self, instance: &PyProviderInstance) {
        self.instances.insert(
            instance.id.clone(),
            PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None,         // Simplified - keys will be added separately
                models: Vec::new(), // Will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            },
        );
    }

    fn get_instance(&self, id: &str) -> Option<PyProviderInstance> {
        self.instances.get(id).map(|instance| PyProviderInstance {
            id: instance.id.clone(),
            display_name: instance.display_name.clone(),
            provider_type: instance.provider_type.clone(),
            base_url: instance.base_url.clone(),
            keys: None,         // Simplified - keys will be added separately
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
        self.instances
            .values()
            .map(|instance| PyProviderInstance {
                id: instance.id.clone(),
                display_name: instance.display_name.clone(),
                provider_type: instance.provider_type.clone(),
                base_url: instance.base_url.clone(),
                keys: None,         // Simplified - keys will be added separately
                models: Vec::new(), // Simplified - models will be added separately
                metadata: instance.metadata.clone(),
                active: instance.active,
                created_at: instance.created_at.clone(),
                updated_at: instance.updated_at.clone(),
            })
            .collect()
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
                keys: None,         // Simplified - keys will be added separately
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
                keys: None,         // Simplified - keys will be added separately
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
                keys: None,         // Simplified - keys will be added separately
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
        let mut types: Vec<String> = self
            .instances
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
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                errors.join("; "),
            ))
        }
    }

    fn clear(&mut self) {
        self.instances.clear();
    }

    fn merge(&mut self, other: &PyProviderInstances) {
        for (id, instance) in &other.instances {
            self.instances.insert(
                id.clone(),
                PyProviderInstance {
                    id: instance.id.clone(),
                    display_name: instance.display_name.clone(),
                    provider_type: instance.provider_type.clone(),
                    base_url: instance.base_url.clone(),
                    keys: None,         // Simplified - keys will be added separately
                    models: Vec::new(), // Will be added separately
                    metadata: instance.metadata.clone(),
                    active: instance.active,
                    created_at: instance.created_at.clone(),
                    updated_at: instance.updated_at.clone(),
                },
            );
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
fn migrate_provider_configs(configs: Vec<Py<PyAny>>) -> PyResult<ProviderInstances> {
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

    Ok(ProviderInstances(instances))
}

/// Wrapper function to provide 'scan' function with expected name
#[pyfunction]
#[pyo3(signature = (home_dir=None, include_full_values=false, max_file_size=1048576, only_providers=None, exclude_providers=None))]
fn scan(
    home_dir: Option<String>,
    include_full_values: bool,
    max_file_size: usize,
    only_providers: Option<Vec<String>>,
    exclude_providers: Option<Vec<String>>,
) -> PyResult<Py<PyAny>> {
    // Validate home_dir if provided
    if let Some(ref home_dir_str) = home_dir {
        let path = PathBuf::from(home_dir_str);
        if !path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "Home directory does not exist: {}",
                home_dir_str
            )));
        }
        if !path.is_dir() {
            return Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "Home directory is not a directory: {}",
                home_dir_str
            )));
        }
        if std::fs::read_dir(&path).is_err() {
            return Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "Cannot read home directory: {}",
                home_dir_str
            )));
        }
    }

    scan_py(
        home_dir,
        include_full_values,
        max_file_size,
        only_providers,
        exclude_providers,
    )
}

/// Wrapper class to provide TokenCost with expected name
#[pyclass]
#[derive(Debug, Clone)]
pub struct TokenCost(PyTokenCost);

#[pymethods]
impl TokenCost {
    #[new]
    #[pyo3(signature = (input_cost_per_million=None, output_cost_per_million=None, cached_input_cost_modifier=None))]
    fn new(
        input_cost_per_million: Option<f64>,
        output_cost_per_million: Option<f64>,
        cached_input_cost_modifier: Option<f64>,
    ) -> Self {
        Self(PyTokenCost::new(
            input_cost_per_million,
            output_cost_per_million,
            cached_input_cost_modifier,
        ))
    }

    #[getter]
    fn input_cost_per_million(&self) -> Option<f64> {
        self.0.input_cost_per_million
    }

    #[setter]
    fn set_input_cost_per_million(&mut self, value: Option<f64>) {
        self.0.input_cost_per_million = value;
    }

    #[getter]
    fn output_cost_per_million(&self) -> Option<f64> {
        self.0.output_cost_per_million
    }

    #[setter]
    fn set_output_cost_per_million(&mut self, value: Option<f64>) {
        self.0.output_cost_per_million = value;
    }

    #[getter]
    fn cached_input_cost_modifier(&self) -> Option<f64> {
        self.0.cached_input_cost_modifier
    }

    #[setter]
    fn set_cached_input_cost_modifier(&mut self, value: Option<f64>) {
        self.0.cached_input_cost_modifier = value;
    }
}

/// Wrapper class to provide Capabilities with expected name
#[pyclass]
#[derive(Debug)]
pub struct Capabilities(PyCapabilities);

#[pymethods]
impl Capabilities {
    #[new]
    #[pyo3(signature = (text_generation=false, image_generation=false, audio_processing=false, video_processing=false, code_generation=false, function_calling=false, fine_tuning=false, streaming=false, multimodal=false, tool_use=false))]
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
        Self(PyCapabilities::new(
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
        ))
    }

    #[getter]
    fn text_generation(&self) -> bool {
        self.0.text_generation
    }

    #[setter]
    fn set_text_generation(&mut self, value: bool) {
        self.0.text_generation = value;
    }

    #[getter]
    fn image_generation(&self) -> bool {
        self.0.image_generation
    }

    #[setter]
    fn set_image_generation(&mut self, value: bool) {
        self.0.image_generation = value;
    }

    #[getter]
    fn audio_processing(&self) -> bool {
        self.0.audio_processing
    }

    #[setter]
    fn set_audio_processing(&mut self, value: bool) {
        self.0.audio_processing = value;
    }

    #[getter]
    fn video_processing(&self) -> bool {
        self.0.video_processing
    }

    #[setter]
    fn set_video_processing(&mut self, value: bool) {
        self.0.video_processing = value;
    }

    #[getter]
    fn code_generation(&self) -> bool {
        self.0.code_generation
    }

    #[setter]
    fn set_code_generation(&mut self, value: bool) {
        self.0.code_generation = value;
    }

    #[getter]
    fn function_calling(&self) -> bool {
        self.0.function_calling
    }

    #[setter]
    fn set_function_calling(&mut self, value: bool) {
        self.0.function_calling = value;
    }

    #[getter]
    fn fine_tuning(&self) -> bool {
        self.0.fine_tuning
    }

    #[setter]
    fn set_fine_tuning(&mut self, value: bool) {
        self.0.fine_tuning = value;
    }

    #[getter]
    fn streaming(&self) -> bool {
        self.0.streaming
    }

    #[setter]
    fn set_streaming(&mut self, value: bool) {
        self.0.streaming = value;
    }

    #[getter]
    fn multimodal(&self) -> bool {
        self.0.multimodal
    }

    #[setter]
    fn set_multimodal(&mut self, value: bool) {
        self.0.multimodal = value;
    }

    #[getter]
    fn tool_use(&self) -> bool {
        self.0.tool_use
    }

    #[setter]
    fn set_tool_use(&mut self, value: bool) {
        self.0.tool_use = value;
    }
}

/// Wrapper class to provide Model with expected name
#[pyclass]
#[derive(Debug)]
pub struct Model(PyModel);

#[pymethods]
impl Model {
    #[new]
    fn new(model_id: String, provider_instance_id: String, name: String) -> Self {
        Self(PyModel::new(model_id, provider_instance_id, name))
    }

    #[getter]
    fn model_id(&self) -> String {
        self.0.model_id.clone()
    }

    #[setter]
    fn set_model_id(&mut self, value: String) {
        self.0.model_id = value;
    }

    #[getter]
    fn provider_instance_id(&self) -> String {
        self.0.provider_instance_id.clone()
    }

    #[setter]
    fn set_provider_instance_id(&mut self, value: String) {
        self.0.provider_instance_id = value;
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[setter]
    fn set_name(&mut self, value: String) {
        self.0.name = value;
    }

    #[getter]
    fn quantization(&self) -> Option<String> {
        self.0.quantization.clone()
    }

    #[setter]
    fn set_quantization(&mut self, value: Option<String>) {
        self.0.quantization = value;
    }

    #[getter]
    fn context_window(&self) -> Option<u32> {
        self.0.context_window
    }

    #[setter]
    fn set_context_window(&mut self, value: Option<u32>) {
        self.0.context_window = value;
    }

    #[getter]
    fn temperature(&self) -> Option<f32> {
        self.0.temperature
    }

    fn set_temperature(&mut self, temperature: f64) {
        self.0.temperature = Some((temperature * 100.0).round() as f32 / 100.0);
    }

    #[getter]
    fn tags(&self) -> Option<Vec<String>> {
        self.0.tags.clone()
    }

    fn add_tag(&mut self, tag: String) {
        if self.0.tags.is_none() {
            self.0.tags = Some(Vec::new());
        }
        if let Some(ref mut tags) = self.0.tags {
            tags.push(tag);
        }
    }

    fn set_tags(&mut self, tags: Vec<String>) {
        self.0.tags = Some(tags);
    }

    #[getter]
    fn cost(&self) -> Option<TokenCost> {
        self.0.cost.as_ref().map(|c| TokenCost(c.clone()))
    }

    fn set_cost(&mut self, cost: TokenCost) {
        self.0.cost = Some(cost.0);
    }

    #[getter]
    fn metadata(&self) -> Option<HashMap<String, Py<PyAny>>> {
        Python::with_gil(|py| {
            self.0.metadata.as_ref().map(|metadata_map| {
                let mut result = HashMap::new();
                for (key, value) in metadata_map {
                    // Clone the Py<PyAny> value into the new HashMap
                    result.insert(key.clone(), value.clone_ref(py));
                }
                result
            })
        })
    }

    #[setter]
    fn set_metadata(&mut self, value: Option<HashMap<String, Py<PyAny>>>) {
        self.0.metadata = value;
    }

    fn validate(&self) -> PyResult<()> {
        self.0.validate()
    }

    fn supports_text_generation(&self) -> bool {
        self.0.supports_text_generation()
    }

    fn supports_image_generation(&self) -> bool {
        self.0.supports_image_generation()
    }
}

/// Wrapper class to provide ProviderInstance with expected name
#[pyclass]
#[derive(Debug)]
pub struct ProviderInstance(PyProviderInstance);

#[pymethods]
impl ProviderInstance {
    #[new]
    fn new(id: String, display_name: String, provider_type: String, base_url: String) -> Self {
        Self(PyProviderInstance::new(
            id,
            display_name,
            provider_type,
            base_url,
        ))
    }

    #[getter]
    fn id(&self) -> String {
        self.0.id.clone()
    }

    #[setter]
    fn set_id(&mut self, value: String) {
        self.0.id = value;
    }

    #[getter]
    fn display_name(&self) -> String {
        self.0.display_name.clone()
    }

    #[setter]
    fn set_display_name(&mut self, value: String) {
        self.0.display_name = value;
    }

    #[getter]
    fn provider_type(&self) -> String {
        self.0.provider_type.clone()
    }

    #[setter]
    fn set_provider_type(&mut self, value: String) {
        self.0.provider_type = value;
    }

    #[getter]
    fn base_url(&self) -> String {
        self.0.base_url.clone()
    }

    #[setter]
    fn set_base_url(&mut self, value: String) {
        self.0.base_url = value;
    }

    #[getter]
    fn active(&self) -> bool {
        self.0.active
    }

    fn set_active(&mut self, active: bool) {
        self.0.active = active;
    }

    #[getter]
    fn models(&self) -> Vec<Model> {
        self.0
            .models
            .iter()
            .map(|m| {
                Model(PyModel {
                    model_id: m.model_id.clone(),
                    provider_instance_id: m.provider_instance_id.clone(),
                    name: m.name.clone(),
                    quantization: m.quantization.clone(),
                    context_window: m.context_window,
                    capabilities: m.capabilities.clone(),
                    temperature: m.temperature,
                    tags: m.tags.clone(),
                    cost: m.cost.clone(),
                    metadata: None, // Simplified - metadata not cloneable
                })
            })
            .collect()
    }

    #[getter]
    fn keys(&self) -> PyResult<Vec<Py<PyAny>>> {
        Python::with_gil(|py| {
            match &self.0.keys {
                Some(keys) => {
                    // Create a new vector with cloned Py objects
                    let mut result = Vec::new();
                    for key in keys {
                        result.push(key.clone_ref(py));
                    }
                    Ok(result)
                }
                None => Ok(Vec::new()),
            }
        })
    }

    fn add_key(&mut self, key: Py<PyAny>) {
        self.0.add_key(key)
    }

    fn add_keys(&mut self, keys: Vec<Py<PyAny>>) {
        self.0.add_keys(keys)
    }

    fn add_model(&mut self, model: &Model) {
        self.0.add_model(&model.0)
    }

    fn add_models(&mut self, models: Vec<Py<PyAny>>) -> PyResult<()> {
        self.0.add_models(models)
    }

    fn key_count(&self) -> usize {
        self.0.key_count()
    }

    fn model_count(&self) -> usize {
        self.0.model_count()
    }

    fn validate(&self) -> PyResult<()> {
        self.0.validate()
    }
}

/// Wrapper class to provide ProviderInstances with expected name
#[pyclass]
#[derive(Debug)]
pub struct ProviderInstances(PyProviderInstances);

#[pymethods]
impl ProviderInstances {
    #[new]
    fn new() -> Self {
        Self(PyProviderInstances::new())
    }

    fn add_instance(&mut self, instance: &ProviderInstance) -> PyResult<()> {
        self.0.add_instance(&instance.0)
    }

    fn add_or_replace_instance(&mut self, instance: &ProviderInstance) {
        self.0.add_or_replace_instance(&instance.0)
    }

    fn get_instance(&self, id: &str) -> Option<ProviderInstance> {
        self.0.get_instance(id).map(ProviderInstance)
    }

    fn remove_instance(&mut self, id: &str) -> Option<ProviderInstance> {
        self.0.remove_instance(id).map(ProviderInstance)
    }

    fn all_instances(&self) -> Vec<ProviderInstance> {
        self.0
            .all_instances()
            .into_iter()
            .map(ProviderInstance)
            .collect()
    }

    fn instances_by_type(&self, provider_type: &str) -> Vec<ProviderInstance> {
        self.0
            .instances_by_type(provider_type)
            .into_iter()
            .map(ProviderInstance)
            .collect()
    }

    fn active_instances(&self) -> Vec<ProviderInstance> {
        self.0
            .active_instances()
            .into_iter()
            .map(ProviderInstance)
            .collect()
    }

    fn active_instances_by_type(&self, provider_type: &str) -> Vec<ProviderInstance> {
        self.0
            .active_instances_by_type(provider_type)
            .into_iter()
            .map(ProviderInstance)
            .collect()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn instance_ids(&self) -> Vec<String> {
        self.0.instance_ids()
    }

    fn provider_types(&self) -> Vec<String> {
        self.0.provider_types()
    }

    fn validate(&self) -> PyResult<()> {
        self.0.validate()
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn merge(&mut self, other: &ProviderInstances) {
        self.0.merge(&other.0)
    }

    fn __repr__(&self) -> String {
        self.0.__repr__()
    }

    fn __str__(&self) -> String {
        self.0.__repr__()
    }
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
) -> PyResult<Py<PyAny>> {
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

    let result = core_scan(&options)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // Convert to JSON and then to Python dict
    let json = serde_json::to_string(&result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        let json_module = py.import("json")?;
        let loads = json_module.getattr("loads")?;
        Ok(loads.call1((json,))?.extract::<Py<PyAny>>()?)
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
        "litellm",
        "groq",
    ]
}

/// List available application scanners
#[pyfunction]
fn list_scanners() -> Vec<&'static str> {
    vec!["roo-code", "claude-desktop", "ragit", "langchain", "gsh"]
}

/// GenAI Key Finder - Python bindings
///
/// A library for discovering GenAI API keys and configurations
#[pymodule]
fn genai_keyfinder(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add the wrapper classes with expected names (these are what tests expect)
    m.add_class::<TokenCost>()?;
    m.add_class::<Capabilities>()?;
    m.add_class::<Model>()?;
    m.add_class::<ProviderInstance>()?;
    m.add_class::<ProviderInstances>()?;

    // Add functions
    m.add_function(wrap_pyfunction!(scan, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(list_providers, m)?)?;
    m.add_function(wrap_pyfunction!(list_scanners, m)?)?;
    m.add_function(wrap_pyfunction!(migrate_provider_configs, m)?)?;

    Ok(())
}
