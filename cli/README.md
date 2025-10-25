# GenAI Key Finder CLI

A command-line interface for discovering GenAI API keys and configurations across your system.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The binary will be available at `target/release/keyfinder`.

## Usage

### Basic Commands

#### Scan for credentials
```bash
keyfinder scan
```

#### List available providers and scanners
```bash
keyfinder list
```

#### Show version information
```bash
keyfinder version
```

### Scan Options

#### Output Formats
Choose from multiple output formats:

```bash
# Table format (default)
keyfinder scan

# JSON format
keyfinder scan --format json

# NDJSON format (one JSON object per line)
keyfinder scan --format ndjson

# Summary format
keyfinder scan --format summary
```

#### Custom Home Directory
```bash
keyfinder scan --home /path/to/directory
```

#### Provider Filtering
```bash
# Only scan specific providers
keyfinder scan --only openai,anthropic

# Exclude specific providers
keyfinder scan --exclude huggingface
```

#### Security Options
```bash
# Include full secret values (DANGEROUS - use with caution)
keyfinder scan --include-values

# Dry run - show what would be scanned
keyfinder scan --dry-run

# Write audit log
keyfinder scan --audit-log scan.log
```

#### File Size Limits
```bash
# Maximum file size to read (default: 1MB)
keyfinder scan --max-bytes-per-file 2097152
```

## Output Formats

### Table Format (Default)
```
=== Discovered Keys ===
Provider             Source                                   Type            Confidence
------------------------------------------------------------------------------------------
openai               /home/user/.env                         api_key         0.95
anthropic            /home/user/config.json                  api_key         0.88

=== Config Instances ===
Application          Path                                              Keys
------------------------------------------------------------------------------------------
roo-code             /home/user/.config/roo-code/config.json         2

Total: 2 keys, 1 config instances
```

### JSON Format
```json
{
  "keys": [
    {
      "provider": "openai",
      "source": "/home/user/.env",
      "value_type": "api_key",
      "confidence": 0.95,
      "value": "sk-..."
    }
  ],
  "config_instances": [...],
  "scanned_at": "2024-01-20T10:30:00Z",
  "home_dir": "/home/user",
  "providers_scanned": ["openai", "anthropic", "huggingface"]
}
```

### Summary Format
```
Scan Summary
  Home Directory: /home/user
  Scan Time: 2024-01-20T10:30:00Z
  Providers Scanned: openai, anthropic, huggingface

Results:
  Keys Found: 2
  Config Instances: 1

By Provider:
  openai: 1
  anthropic: 1
```

## Exit Codes

- `0`: Keys or config instances found
- `1`: No keys or config instances found
- `2`: Error occurred

## Security Considerations

⚠️ **WARNING**: The `--include-values` flag will display full secret values in the output. Use with extreme caution and only in secure environments.

- Never commit scan results with full values to version control
- Use audit logs to track what was discovered
- Consider using the `--dry-run` flag first to see what would be scanned
- Be mindful of file permissions when writing audit logs

## Supported Providers

- **OpenAI**: API keys and organization IDs
- **Anthropic**: Claude API keys
- **Hugging Face**: Access tokens
- **Ollama**: Local model configurations
- **LangChain**: API keys and configurations
- **LiteLLM**: Proxy configurations

## Supported Application Scanners

- **Roo Code**: VSCode extension configurations
- **Claude Desktop**: Desktop application configs
- **Ragit**: RAG application configurations
- **LangChain**: Application-specific configs

## Examples

### Comprehensive Scan
```bash
keyfinder scan --format table --audit-log comprehensive-scan.log
```

### Targeted Scan
```bash
keyfinder scan --only openai,anthropic --format summary
```

### Security Audit
```bash
keyfinder scan --dry-run --format json > potential-targets.json
```

### CI/CD Integration
```bash
# Fail if any credentials are found (exit code 0 means found)
if keyfinder scan --format summary; then
    echo "Credentials found - review required"
    exit 1
fi
```

## License

See the main project LICENSE file.