# AICred CLI

A command-line interface for discovering AI API keys and configurations across your system.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The binary will be available at `target/release/aicred`.

## Usage

### Basic Commands

#### Scan for credentials
```bash
aicred scan
```

#### List available providers and scanners
```bash
aicred providers
```

#### Show version information
```bash
aicred version
```

### Scan Options

#### Output Formats
Choose from multiple output formats:

```bash
# Table format (default)
aicred scan

# JSON format
aicred scan --format json

# NDJSON format (one JSON object per line)
aicred scan --format ndjson

# Summary format
aicred scan --format summary
```

#### Custom Home Directory
```bash
aicred scan --home /path/to/directory
```

#### Provider Filtering
```bash
# Only scan specific providers
aicred scan --only openai,anthropic

# Exclude specific providers
aicred scan --exclude huggingface
```

#### Security Options
```bash
# Include full secret values (DANGEROUS - use with caution)
aicred scan --include-values

# Dry run - show what would be scanned
aicred scan --dry-run

# Write audit log
aicred scan --audit-log scan.log
```

#### File Size Limits
```bash
# Maximum file size to read (default: 1MB)
aicred scan --max-bytes-per-file 2097152
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
### Wrap Command - Execute with Environment Variables

The `wrap` command executes commands with environment variables automatically resolved from label mappings.

#### Basic Usage

```bash
# Run a command with environment variables from resolved labels
aicred wrap --labels fast -- python my_script.py

# Use multiple labels (comma-separated)
aicred wrap --labels fast,smart -- npm run dev

# Dry run to preview environment variables without executing
aicred wrap --labels fast --dry-run -- echo "Preview mode"
```

#### Scanner-Specific Usage

Different scanners generate different environment variable patterns:

```bash
# GSH scanner (default) - generates GSH_* variables
aicred wrap --scanner gsh --labels fast -- python app.py

# Roo Code scanner - generates ROO_CODE_* variables
aicred wrap --scanner roo-code --labels primary -- code .

# Claude Desktop scanner - generates ANTHROPIC_* variables
aicred wrap --scanner claude-desktop --labels smart -- claude-desktop

# RAGIt scanner - generates RAGIT_* variables
aicred wrap --scanner ragit --labels fast -- ragit query

# LangChain scanner - generates LANGCHAIN_* variables
aicred wrap --scanner langchain --labels smart -- python langchain_app.py
```

#### Command-Line Flags

- `--labels <LABELS>` - Comma-separated list of label names to resolve
- `--scanner <SCANNER>` - Scanner type (gsh, roo-code, claude-desktop, ragit, langchain)
- `--dry-run` - Preview environment variables without executing command
- `--home <PATH>` - Custom home directory for configuration
- `-- <COMMAND>` - Command to execute (everything after `--`)

#### Examples

```bash
# Preview what environment variables would be set
$ aicred wrap --labels fast --dry-run -- python app.py
Environment variables that would be set:
  GSH_FAST_MODEL=groq:llama3-70b-8192
  GSH_FAST_API_KEY=gsk_...xyz
  GSH_FAST_BASE_URL=https://api.groq.com/openai/v1

# Run with multiple labels
aicred wrap --labels fast,smart -- python multi_model_app.py

# Use specific scanner
aicred wrap --scanner roo-code --labels primary -- code .

# Custom home directory
aicred wrap --home /path/to/config --labels fast -- python app.py
```

### SetEnv Command - Generate Shell Export Statements

The `setenv` command generates shell-specific export statements for environment variables.

#### Basic Usage

```bash
# Generate bash/zsh format (default)
aicred setenv --labels fast --format bash

# Generate fish shell format
aicred setenv --labels fast --format fish

# Generate PowerShell format
aicred setenv --labels fast --format powershell
```

#### Output Formats

**Bash/Zsh:**
```bash
$ aicred setenv --labels fast --format bash
export GSH_FAST_MODEL='groq:llama3-70b-8192'
export GSH_FAST_API_KEY='gsk_...'
export GSH_FAST_BASE_URL='https://api.groq.com/openai/v1'

# Source into current shell
eval "$(aicred setenv --labels fast --format bash)"
```

**Fish Shell:**
```bash
$ aicred setenv --labels fast --format fish
set -gx GSH_FAST_MODEL 'groq:llama3-70b-8192'
set -gx GSH_FAST_API_KEY 'gsk_...'
set -gx GSH_FAST_BASE_URL 'https://api.groq.com/openai/v1'

# Source into current shell
aicred setenv --labels fast --format fish | source
```

**PowerShell:**
```powershell
PS> aicred setenv --labels fast --format powershell
$env:GSH_FAST_MODEL = 'groq:llama3-70b-8192'
$env:GSH_FAST_API_KEY = 'gsk_...'
$env:GSH_FAST_BASE_URL = 'https://api.groq.com/openai/v1'

# Execute in current session
aicred setenv --labels fast --format powershell | Invoke-Expression
```

#### Command-Line Flags

- `--labels <LABELS>` - Comma-separated list of label names to resolve
- `--format <FORMAT>` - Output format (bash, fish, powershell)
- `--scanner <SCANNER>` - Scanner type (gsh, roo-code, claude-desktop, ragit, langchain)
- `--dry-run` - Preview variables with masked secrets
- `--home <PATH>` - Custom home directory for configuration

#### Examples

```bash
# Generate for multiple labels
aicred setenv --labels fast,smart --format bash

# Preview with masked secrets
aicred setenv --labels fast --dry-run

# Use specific scanner
aicred setenv --scanner roo-code --labels primary --format bash

# Generate for fish shell
aicred setenv --labels fast --format fish | source
```

### Environment Variable Mapping

When you assign labels to provider instances, the system generates environment variables following scanner-specific patterns.

#### Standard Variable Pattern

- `{SCANNER}_{LABEL}_MODEL` - Provider:model tuple (e.g., `GSH_FAST_MODEL=openai:gpt-4`)
- `{SCANNER}_{LABEL}_API_KEY` - API key for the provider
- `{SCANNER}_{LABEL}_BASE_URL` - Base URL for the API
- `{SCANNER}_{LABEL}_{METADATA_KEY}` - Custom metadata from provider instances

#### Scanner-Specific Variables

**GSH Scanner:**
```bash
GSH_FAST_MODEL=groq:llama3-70b-8192
GSH_FAST_API_KEY=gsk_...
GSH_FAST_BASE_URL=https://api.groq.com/openai/v1
GSH_FAST_TEMPERATURE=0.7
GSH_FAST_PARALLEL_TOOL_CALLS=true
```

**Roo Code Scanner:**
```bash
ROO_CODE_API_KEY=sk-ant-...
ROO_CODE_MODEL_ID=claude-3-opus
ROO_CODE_BASE_URL=https://api.anthropic.com
ROO_CODE_TEMPERATURE=0.7
ROO_CODE_PARALLEL_TOOL_CALLS=true
```

**Claude Desktop Scanner:**
```bash
ANTHROPIC_API_KEY=sk-ant-...
CLAUDE_MODEL_ID=claude-3-opus
CLAUDE_BASE_URL=https://api.anthropic.com
```

**RAGIt Scanner:**
```bash
RAGIT_API_KEY=sk-...
RAGIT_MODEL_ID=gpt-4
RAGIT_BASE_URL=https://api.openai.com/v1
RAGIT_EMBEDDING_MODEL=text-embedding-ada-002
```

**LangChain Scanner:**
```bash
LANGCHAIN_API_KEY=sk-...
LANGCHAIN_MODEL_ID=gpt-4
LANGCHAIN_BASE_URL=https://api.openai.com/v1
LANGCHAIN_TEMPERATURE=0.7
LANGCHAIN_MAX_TOKENS=4096
```

#### Complete Workflow Example

```bash
# 1. Create a label for fast models
aicred labels add --name "fast" --description "Fast model for quick tasks" --color "#00ff00"

# 2. Assign label to a specific provider instance and model
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192

# 3. Use the label in your application with wrap
aicred wrap --labels fast -- python my_app.py

# 4. Or generate environment variables for manual use
aicred setenv --labels fast --format bash
# Output:
# export GSH_FAST_MODEL='groq:llama3-70b-8192'
# export GSH_FAST_API_KEY='gsk_...'
# export GSH_FAST_BASE_URL='https://api.groq.com/openai/v1'

# 5. Source into your shell
eval "$(aicred setenv --labels fast --format bash)"

# 6. Now your application can use the environment variables
python my_app.py  # Will use GSH_FAST_* variables
```

### Label Management Examples

```bash
# Create a label for fast models
aicred labels add --name "fast" --description "Fast model for quick tasks" --color "#00ff00"

# Assign label to a specific model
aicred labels assign --name "fast" --instance-id my-openai --model-id gpt-3.5-turbo

# List all labels
aicred labels list

# Unassign a label
aicred labels unassign --name "fast" --instance-id my-openai
```

### Instance Management Examples

```bash
# List all instances
aicred instances

# Get detailed information about an instance
aicred instances my-openai

# Add a new instance
aicred instances add --id my-anthropic --name "My Anthropic" --provider-type anthropic --base-url https://api.anthropic.com --models claude-3-opus,claude-3-sonnet

# Update an instance
aicred instances update --id my-openai --active false

# Remove an instance
aicred instances remove --id my-old-instance
```

### Tag Management Examples

```bash
# Create tags for different environments
aicred tags add --name "Production" --color "#ff0000" --description "Production environment"
aicred tags add --name "Development" --color "#00ff00" --description "Development environment"

# Assign tags to instances
aicred tags assign --name "Production" --instance-id openai-prod
aicred tags assign --name "Development" --instance-id openai-dev

# List all tags
aicred tags list

# Unassign a tag
aicred tags unassign --name "Development" --instance-id openai-dev
```

## Examples

### Comprehensive Scan
```bash
aicred scan --format table --audit-log comprehensive-scan.log
```

### Targeted Scan
```bash
aicred scan --only openai,anthropic --format summary
```

### Security Audit
```bash
aicred scan --dry-run --format json > potential-targets.json
```

### CI/CD Integration
```bash
# Fail if any credentials are found (exit code 0 means found)
if aicred scan --format summary; then
    echo "Credentials found - review required"
    exit 1
fi
```

## License

See the main project LICENSE file.