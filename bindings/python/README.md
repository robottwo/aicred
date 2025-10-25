# GenAI Key Finder - Python Bindings

Python bindings for the genai-keyfinder library.

## Installation

```bash
pip install genai-keyfinder
```

Or build from source:

```bash
cd bindings/python
pip install maturin
maturin develop
```

## Usage

```python
import genai_keyfinder

# Scan for credentials
result = genai_keyfinder.scan()

# Print results
print(f"Found {len(result['keys'])} keys")
for key in result['keys']:
    print(f"{key['provider']}: {key['redacted']}")
```

## API Reference

### `scan()`

Scan for GenAI credentials and configurations.

**Parameters:**
- `home_dir` (str, optional): Home directory to scan
- `include_full_values` (bool): Include full secret values. Default: False
- `max_file_size` (int): Maximum file size in bytes. Default: 1048576
- `only_providers` (list[str], optional): Only scan these providers
- `exclude_providers` (list[str], optional): Exclude these providers

**Returns:** Dictionary with scan results

### `version()`

Get library version string.

### `list_providers()`

List available provider plugins.

### `list_scanners()`

List available application scanners.

## Security

By default, all secrets are redacted. Only use `include_full_values=True` in secure environments.