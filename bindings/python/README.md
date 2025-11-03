# AICred - Python Bindings

Python bindings for the aicred library with enhanced provider instance and model management.

## Installation

```bash
pip install aicred
```

Or build from source:

```bash
cd bindings/python
pip install maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop
```

**Note:** The `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` environment variable is required for Python 3.13.9+ compatibility.

## Usage

### Basic Scanning

```python
import aicred

# Scan for credentials
result = aicred.scan()

# Print results
print(f"Found {len(result['keys'])} keys")
for key in result['keys']:
    print(f"{key['provider']}: {key['redacted']}")
```

### Enhanced Provider Instance Management

```python
import aicred

# Create a provider instance
provider = aicred.ProviderInstance(
    id="openai-prod",
    display_name="OpenAI Production",
    provider_type="openai",
    base_url="https://api.openai.com"
)

# Add models to the provider
model = aicred.Model(
    model_id="gpt-4",
    provider_instance_id="openai-prod",
    name="GPT-4"
)
model.set_temperature(0.7)
model.set_tags(["text-generation", "code"])

provider.add_model(model)

# Create a collection of provider instances
instances = aicred.ProviderInstances()
instances.add_instance(provider)

# Filter instances
active_openai_instances = instances.active_instances_by_type("openai")
```

### Token Cost Tracking

```python
import aicred

# Create token cost tracking
cost = aicred.TokenCost(
    input_cost_per_million=0.001,
    output_cost_per_million=0.002,
    cached_input_cost_modifier=0.1
)

# Apply cost to model
model = aicred.Model(
    model_id="gpt-4",
    provider_instance_id="openai-prod",
    name="GPT-4"
)
model.set_cost(cost)
```

### Model Capabilities

```python
import aicred

# Define model capabilities
capabilities = aicred.Capabilities(
    text_generation=True,
    code_generation=True,
    streaming=True,
    function_calling=True
)

# Create model with capabilities
model = aicred.Model(
    model_id="claude-3",
    provider_instance_id="anthropic-prod",
    name="Claude 3"
)
model.set_capabilities(capabilities)
model.set_context_window(200000)
```

### Migration from Legacy Configurations

```python
import aicred

# Migrate legacy provider configurations to new instance-based architecture
# (Placeholder - replace with actual legacy config objects)
legacy_configs = []  # List of legacy ProviderConfig objects

migrated_instances = aicred.migrate_provider_configs(legacy_configs)
```

## API Reference

### Core Functions

#### `scan()`

Scan for GenAI credentials and configurations.

**Parameters:**
- `home_dir` (str, optional): Home directory to scan
- `include_full_values` (bool): Include full secret values. Default: False
- `max_file_size` (int): Maximum file size in bytes. Default: 1048576
- `only_providers` (list[str], optional): Only scan these providers
- `exclude_providers` (list[str], optional): Exclude these providers

**Returns:** Dictionary with scan results

#### `version()`

Get library version string.

#### `list_providers()`

List available provider plugins.

#### `list_scanners()`

List available application scanners.

#### `migrate_provider_configs(configs)`

Migration utilities for converting legacy configurations to new instance-based architecture.

**Parameters:**
- `configs` (list): List of legacy provider configuration objects

**Returns:** ProviderInstances collection with migrated instances

### Enhanced Model Classes

#### `TokenCost`

Token cost tracking for model usage.

**Attributes:**
- `input_cost_per_million` (float, optional): Cost per million input tokens in USD
- `output_cost_per_million` (float, optional): Cost per million output tokens in USD
- `cached_input_cost_modifier` (float, optional): Cached input cost modifier (0.1 = 90% discount)

**Methods:**
- `__init__(input_cost_per_million=None, output_cost_per_million=None, cached_input_cost_modifier=None)`
- `__repr__()`: String representation

#### `Capabilities`

Model capabilities and features.

**Attributes:**
- `text_generation` (bool): Supports text generation
- `image_generation` (bool): Supports image generation
- `audio_processing` (bool): Supports audio processing
- `video_processing` (bool): Supports video processing
- `code_generation` (bool): Supports code generation
- `function_calling` (bool): Supports function calling
- `fine_tuning` (bool): Supports fine-tuning
- `streaming` (bool): Supports streaming responses
- `multimodal` (bool): Supports multi-modal inputs
- `tool_use` (bool): Supports tool use
- `custom` (dict): Additional custom capabilities

**Methods:**
- `__init__(text_generation=False, image_generation=False, audio_processing=False, video_processing=False, code_generation=False, function_calling=False, fine_tuning=False, streaming=False, multimodal=False, tool_use=False)`
- `__repr__()`: String representation

#### `Model`

Enhanced AI model configuration with temperature, tags, and cost tracking.

**Attributes:**
- `model_id` (str): Unique identifier for the model
- `provider_instance_id` (str): Reference to the provider instance this model belongs to
- `name` (str): Human-readable name for the model
- `quantization` (str, optional): Model quantization information
- `context_window` (int, optional): Maximum context window size in tokens
- `capabilities` (Capabilities, optional): Model capabilities and features
- `temperature` (float, optional): Temperature parameter (0.0-2.0)
- `tags` (list[str], optional): Tags for categorization and filtering
- `cost` (TokenCost, optional): Token cost tracking
- `metadata` (dict, optional): Additional metadata

**Methods:**
- `__init__(model_id, provider_instance_id, name)`: Create a new model
- `set_quantization(quantization)`: Set quantization
- `set_context_window(size)`: Set context window size
- `set_capabilities(capabilities)`: Set capabilities
- `set_temperature(temperature)`: Set temperature parameter
- `set_tags(tags)`: Set tags
- `set_cost(cost)`: Set cost tracking
- `set_metadata(metadata)`: Set additional metadata
- `validate()`: Validate model configuration
- `supports_text_generation()`: Check if model supports text generation
- `supports_image_generation()`: Check if model supports image generation
- `__repr__()`: String representation

#### `ProviderInstance`

Provider instance configuration with enhanced metadata and model management.

**Attributes:**
- `id` (str): Unique identifier for this instance
- `display_name` (str): Human-readable display name
- `provider_type` (str): Provider type (e.g., "openai", "anthropic", "groq")
- `base_url` (str): Base URL for API requests
- `keys` (list): API keys associated with this instance
- `models` (list[Model]): Instance-specific model configurations
- `metadata` (dict[str, str], optional): Additional metadata
- `active` (bool): Whether this instance is active and should be used
- `created_at` (str): When this instance was created (ISO 8601 format)
- `updated_at` (str): When this instance was last updated (ISO 8601 format)

**Methods:**
- `__init__(id, display_name, provider_type, base_url)`: Create a new provider instance
- `add_key(key)`: Add a key to this instance
- `add_keys(keys)`: Add multiple keys to this instance
- `add_model(model)`: Add a model to this instance
- `add_models(models)`: Add multiple models to this instance
- `set_metadata(metadata)`: Set metadata
- `set_active(active)`: Set active status
- `key_count()`: Get the number of keys
- `model_count()`: Get the number of models
- `validate()`: Validate the instance configuration
- `__repr__()`: String representation

#### `ProviderInstances`

Collection of provider instances with lookup and filtering capabilities.

**Methods:**
- `__init__()`: Create a new empty collection
- `add_instance(instance)`: Add a provider instance to the collection
- `add_or_replace_instance(instance)`: Add or replace a provider instance
- `get_instance(id)`: Get a provider instance by ID
- `remove_instance(id)`: Remove a provider instance by ID
- `all_instances()`: Get all provider instances
- `instances_by_type(provider_type)`: Get instances filtered by provider type
- `active_instances()`: Get only active provider instances
- `active_instances_by_type(provider_type)`: Get only active instances of a specific provider type
- `len()`: Get the total number of instances
- `is_empty()`: Check if the collection is empty
- `instance_ids()`: Get all instance IDs
- `provider_types()`: Get all provider types present in the collection
- `validate()`: Validate all instances in the collection
- `clear()`: Clear all instances from the collection
- `merge(other)`: Merge another ProviderInstances collection into this one
- `__repr__()`: String representation

## Backward Compatibility

The new instance-based architecture maintains full backward compatibility with existing Python integrations:

- The `scan()` function continues to return the same dictionary structure
- All existing function signatures remain unchanged
- Legacy code will continue to work without modifications
- New features are additive and optional

## Security

By default, all secrets are redacted. Only use `include_full_values=True` in secure environments.

## Examples

See the [examples directory](./examples/) for more usage examples.