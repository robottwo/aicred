# AICred

**Unified Configuration Management for AI Tools**

Stop managing the same API keys and model configurations across dozens of different AI tools. AICred provides a single source of truth for all your AI provider credentials and preferences.

[![License](https://img.shields.io/badge/license-GPL%20v3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/robottwo/aicred/workflows/CI/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/ci.yml)
[![Python](https://github.com/robottwo/aicred/workflows/Python%20Bindings/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/python.yml)
[![Go](https://github.com/robottwo/aicred/workflows/Go%20Bindings/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/go.yml)
[![GUI](https://github.com/robottwo/aicred/workflows/GUI%20Application/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/gui.yml)
[![Security](https://github.com/robottwo/aicred/workflows/Security%20Audit/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/robottwo/aicred/branch/main/graph/badge.svg)](https://codecov.io/gh/robottwo/aicred)

## The Problem

Modern AI development involves juggling multiple tools, each with its own configuration format:

- **Claude Desktop** uses `~/Library/Application Support/Claude/config.json`
- **Roo Code** uses `~/.config/roo-code/config.json`
- **LangChain** apps expect environment variables like `OPENAI_API_KEY`
- **Custom scripts** might use `.env` files, YAML configs, or JSON
- **Multiple providers** (OpenAI, Anthropic, Groq, OpenRouter) each need separate credentials

You end up with:
- 🔑 The same API keys duplicated across 10+ config files
- 🔄 Manual updates when rotating credentials or switching providers
- 🎯 No easy way to switch between "fast" and "smart" models across tools
- 🔍 Difficulty auditing which tools have access to which credentials
- 🚫 No standardization between tools and providers

## The Solution

AICred provides three key capabilities:

### 1. **Standard Configuration Format** (`~/.config/aicred/`)

A single, unified location for all your AI provider configurations that any application can read:

```yaml
# ~/.config/aicred/instances.yaml
my-openai:
  provider_type: openai
  base_url: https://api.openai.com/v1
  api_key: sk-...
  models: [gpt-4, gpt-3.5-turbo]

my-groq:
  provider_type: groq
  base_url: https://api.groq.com/openai/v1
  api_key: gsk-...
  models: [llama3-70b-8192]
```

### 2. **Configuration Discovery & Management CLI**

Scan your system to discover existing configurations and consolidate them:

```bash
# Discover all AI credentials on your system
aicred scan

# Import configurations from existing tools
aicred scan --update

# Manage your unified configuration
aicred instances list
aicred labels add --name "fast" --description "Fast models for quick tasks"
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192
```

### 3. **Automatic Application Configuration**

Use the `wrap` command to run any application with the correct environment variables:

```bash
# Run your app with the "fast" model configuration
aicred wrap --labels fast -- python my_script.py

# Your script automatically gets:
# GSH_FAST_MODEL=groq:llama3-70b-8192
# GSH_FAST_API_KEY=gsk-...
# GSH_FAST_BASE_URL=https://api.groq.com/openai/v1

# Or generate shell exports for manual use
eval "$(aicred wrap --setenv --labels fast --format bash)"
```

## Key Features

- 🔐 **Security-First**: Keys are redacted by default with SHA-256 hashing
- 🔌 **Plugin Architecture**: Extensible system for adding new providers and applications
- 🌍 **Cross-Platform**: Works on Linux, macOS, and Windows
- 🚀 **Multiple Interfaces**: CLI, Python, Go, Rust library, and GUI
- 📊 **Rich Output**: JSON, NDJSON, table, and summary formats
- 🎯 **Smart Detection**: High-confidence key identification across multiple config formats
- 🏷️ **Label System**: Assign semantic labels like "fast", "smart", "cheap" to provider:model combinations
- 🔄 **Auto-Configuration**: Automatically configure applications that don't support the standard format

## Quick Start

### Installation

```bash
# Install CLI
cargo install aicred

# Or use Homebrew (macOS)
brew install aicred

# Or use pip for Python bindings
pip install aicred
```

### Basic Workflow

```bash
# 1. Scan your system for existing AI configurations
aicred scan --format table

# 2. Import discovered configurations into unified format
aicred scan --update

# 3. Create semantic labels for different use cases
aicred labels add --name "fast" --description "Fast, cheap models"
aicred labels add --name "smart" --description "High-quality models"

# 4. Assign labels to specific provider:model combinations
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192
aicred labels assign --name "smart" --instance-id my-openai --model-id gpt-4

# 5. Run applications with automatic configuration
aicred wrap --labels fast -- python quick_task.py
aicred wrap --labels smart -- python complex_analysis.py
```

## Supported Providers

- OpenAI (GPT-4, GPT-3.5, etc.)
- Anthropic (Claude 3 Opus, Sonnet, Haiku)
- Groq (Llama 3, Mixtral, Gemma)
- OpenRouter (unified access to 100+ models)
- Hugging Face
- Ollama (local models)
- LiteLLM (proxy configurations)

## Supported Applications

AICred can discover and manage configurations for:

- **Roo Code** (VSCode extension)
- **Claude Desktop** (Anthropic's desktop app)
- **Ragit** (RAG applications)
- **LangChain** applications
- **GSH** (GenAI Shell)
- Custom applications via environment variables

## Use Cases

### For Individual Developers

```bash
# Quickly switch between providers for cost optimization
aicred labels assign --name "dev" --instance-id my-groq --model-id llama3-70b-8192
aicred wrap --labels dev -- python dev_script.py

# Use premium models for production
aicred labels assign --name "prod" --instance-id my-openai --model-id gpt-4
aicred wrap --labels prod -- python prod_script.py
```

### For Teams

```bash
# Share standardized configuration format
git clone team-repo
cp team-aicred-config.yaml ~/.config/aicred/instances.yaml

# Everyone uses the same provider:model combinations
aicred wrap --labels team-standard -- python shared_tool.py
```

### For Security Audits

```bash
# Discover all AI credentials on a system
aicred scan --format json --audit-log audit.log

# Review which applications have access to which providers
aicred instances list --verbose
```

## Language Bindings

### Python

```python
import aicred

# Scan for credentials
result = aicred.scan()
print(f"Found {len(result['keys'])} keys")

# Use in your application
for key in result['keys']:
    print(f"{key['provider']}: {key['confidence']}")
```

### Go

```go
import "github.com/robottwo/aicred/bindings/go"

result, err := aicred.Scan(aicred.ScanOptions{})
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Found %d keys\n", len(result.Keys))
```

### Rust

```rust
use aicred_core::{scan, ScanOptions};

let result = scan(ScanOptions::default())?;
println!("Found {} keys", result.total_keys());
```

## Documentation

- [Installation Guide](docs/installation.md) - Detailed installation instructions
- [User Guide](docs/user-guide.md) - Complete usage documentation
- [Architecture](docs/architecture.md) - System design and internals
- [API Reference](docs/api-reference.md) - Library API documentation
- [Plugin Development](docs/plugin-development.md) - Creating custom providers
- [Security](docs/security.md) - Security best practices

## Project Structure

```
aicred/
├── core/              # Core Rust library
├── cli/               # Command-line tool
├── gui/               # Tauri desktop application
├── bindings/
│   ├── python/        # Python bindings (PyO3)
│   └── go/            # Go bindings (CGO)
├── ffi/               # C-API layer
└── docs/              # Documentation
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on:

- Development setup
- Code style guidelines
- Adding new providers
- Testing requirements
- Pull request process

## Security

⚠️ **Important**: By default, all secrets are redacted. Only use `--include-values` in secure environments.

For security concerns, please see [docs/security.md](docs/security.md) or contact the maintainers privately.

## License

Licensed under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

## Why "AICred"?

**AI** + **Cred**entials = A unified way to manage authentication and configuration for AI tools. Think of it as a credential manager specifically designed for the unique challenges of working with multiple AI providers and tools.
