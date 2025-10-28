from typing import Optional, List, Dict, Any

class TokenCost:
    """Token cost tracking for model usage."""
    
    def __init__(self, 
                 input_cost_per_million: Optional[float] = None,
                 output_cost_per_million: Optional[float] = None,
                 cached_input_cost_modifier: Optional[float] = None) -> None: ...
    
    input_cost_per_million: Optional[float]
    output_cost_per_million: Optional[float]
    cached_input_cost_modifier: Optional[float]
    
    def __repr__(self) -> str: ...

class Capabilities:
    """Model capabilities and features."""
    
    def __init__(self,
                 text_generation: bool = False,
                 image_generation: bool = False,
                 audio_processing: bool = False,
                 video_processing: bool = False,
                 code_generation: bool = False,
                 function_calling: bool = False,
                 fine_tuning: bool = False,
                 streaming: bool = False,
                 multimodal: bool = False,
                 tool_use: bool = False) -> None: ...
    
    text_generation: bool
    image_generation: bool
    audio_processing: bool
    video_processing: bool
    code_generation: bool
    function_calling: bool
    fine_tuning: bool
    streaming: bool
    multimodal: bool
    tool_use: bool
    custom: Dict[str, Any]
    
    def __repr__(self) -> str: ...

class Model:
    """Enhanced AI model configuration with temperature, tags, and cost tracking."""
    
    def __init__(self, model_id: str, provider_instance_id: str, name: str) -> None: ...
    
    model_id: str
    provider_instance_id: str
    name: str
    quantization: Optional[str]
    context_window: Optional[int]
    capabilities: Optional[Capabilities]
    temperature: Optional[float]
    tags: Optional[List[str]]
    cost: Optional[TokenCost]
    metadata: Optional[Dict[str, Any]]
    
    def set_quantization(self, quantization: str) -> None: ...
    def set_context_window(self, size: int) -> None: ...
    def set_capabilities(self, capabilities: Capabilities) -> None: ...
    def set_temperature(self, temperature: float) -> None: ...
    def set_tags(self, tags: List[str]) -> None: ...
    def set_cost(self, cost: TokenCost) -> None: ...
    def set_metadata(self, metadata: Dict[str, Any]) -> None: ...
    def validate(self) -> None: ...
    def supports_text_generation(self) -> bool: ...
    def supports_image_generation(self) -> bool: ...
    def __repr__(self) -> str: ...

class ProviderInstance:
    """Provider instance configuration with enhanced metadata and model management."""
    
    def __init__(self, id: str, display_name: str, provider_type: str, base_url: str) -> None: ...
    
    id: str
    display_name: str
    provider_type: str
    base_url: str
    keys: List[Any]
    models: List[Model]
    metadata: Optional[Dict[str, str]]
    active: bool
    created_at: str
    updated_at: str
    
    def add_key(self, key: Any) -> None: ...
    def add_keys(self, keys: List[Any]) -> None: ...
    def add_model(self, model: Model) -> None: ...
    def add_models(self, models: List[Model]) -> None: ...
    def set_metadata(self, metadata: Dict[str, str]) -> None: ...
    def set_active(self, active: bool) -> None: ...
    def key_count(self) -> int: ...
    def model_count(self) -> int: ...
    def validate(self) -> None: ...
    def __repr__(self) -> str: ...

class ProviderInstances:
    """Collection of provider instances with lookup and filtering capabilities."""
    
    def __init__(self) -> None: ...
    
    def add_instance(self, instance: ProviderInstance) -> None: ...
    def add_or_replace_instance(self, instance: ProviderInstance) -> None: ...
    def get_instance(self, id: str) -> Optional[ProviderInstance]: ...
    def remove_instance(self, id: str) -> Optional[ProviderInstance]: ...
    def all_instances(self) -> List[ProviderInstance]: ...
    def instances_by_type(self, provider_type: str) -> List[ProviderInstance]: ...
    def active_instances(self) -> List[ProviderInstance]: ...
    def active_instances_by_type(self, provider_type: str) -> List[ProviderInstance]: ...
    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
    def instance_ids(self) -> List[str]: ...
    def provider_types(self) -> List[str]: ...
    def validate(self) -> None: ...
    def clear(self) -> None: ...
    def merge(self, other: ProviderInstances) -> None: ...
    def __repr__(self) -> str: ...

def scan(
    home_dir: Optional[str] = None,
    include_full_values: bool = False,
    max_file_size: int = 1048576,
    only_providers: Optional[List[str]] = None,
    exclude_providers: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """
    Scan for GenAI credentials and configurations.
    
    Args:
        home_dir: Home directory to scan. Defaults to user's home directory.
        include_full_values: Include full secret values (DANGEROUS). Default: False
        max_file_size: Maximum file size to read in bytes. Default: 1MB
        only_providers: Only scan these providers
        exclude_providers: Exclude these providers
    
    Returns:
       Dictionary containing:
       - keys: List of discovered keys
       - config_instances: List of config instances
       - home_directory: Scanned home directory
       - scan_started_at: Timestamp of scan
       - providers_scanned: List of providers scanned
    
    Example:
        >>> result = scan()
        >>> print(f"Found {len(result['keys'])} keys")
        >>> for key in result['keys']:
        ...     print(f"{key['provider']}: {key['redacted']}")
    """
    ...

def version() -> str:
    """Get library version."""
    ...

def list_providers() -> List[str]:
    """List available provider plugins."""
    ...

def list_scanners() -> List[str]:
    """List available application scanners."""
    ...

def migrate_provider_configs(configs: List[Any]) -> ProviderInstances:
    """
    Migration utilities for converting legacy configurations to new instance-based architecture.
    
    Args:
        configs: List of legacy provider configuration objects
        
    Returns:
        ProviderInstances collection with migrated instances
    """
    ...