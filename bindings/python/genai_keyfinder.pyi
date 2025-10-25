from typing import Optional, List, Dict, Any

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
        - home_dir: Scanned home directory
        - scanned_at: Timestamp of scan
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