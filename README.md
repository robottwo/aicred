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

## Status

🚧 **Beta** - AICred v0.2.0 is feature-complete and undergoing final testing before the first stable release. The API is stabilizing, but breaking changes are possible before v1.0.

**Current capabilities:**
- ✅ Full CLI functionality
- ✅ Python and Go bindings
- ✅ Rust library
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ 7 provider plugins + extensible architecture
- 🚧 GUI application (functional, UI refinements in progress)
- 📦 Package distribution coming soon (crates.io, PyPI, Homebrew)

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

The `wrap` command runs applications with automatically-generated environment variables, so tools can access your unified configuration without modification:

```bash
# Run your app with the "fast" model configuration
aicred wrap --labels fast -- python my_script.py

# Your script automatically gets environment variables like:
# AICRED_FAST_MODEL=groq:llama3-70b-8192
# AICRED_FAST_API_KEY=gsk-...
# AICRED_FAST_BASE_URL=https://api.groq.com/openai/v1

# Or generate shell exports for manual use
eval "$(aicred wrap --setenv --labels fast --format bash)"

# Use custom environment variable prefixes
aicred wrap --labels fast --env-prefix "MY_APP" -- python my_script.py
# Sets: MY_APP_FAST_MODEL, MY_APP_FAST_API_KEY, etc.
```

**Why this matters:** Most AI tools expect environment variables like `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc. The `wrap` command bridges your unified configuration to these tool-specific formats automatically.

## Key Features

### Security & Privacy
- 🔐 **Secrets Redacted by Default**: All keys displayed as SHA-256 hashes unless explicitly requested
- 🔍 **Audit Logging**: Track which applications access which credentials
- 🔒 **Local-First**: All data stays on your machine; no external API calls

### Developer Experience
- 🚀 **Multiple Interfaces**: CLI, Python, Go, Rust library, and GUI
- 📊 **Rich Output Formats**: JSON, NDJSON, table, and summary views
- 🎯 **Smart Detection**: High-confidence key identification across multiple config formats
- 🏷️ **Semantic Labels**: Tag provider:model combinations as "fast", "smart", "cheap", etc.
- 🔄 **Auto-Configuration**: Generate environment variables for any application

### Architecture
- 🔌 **Plugin System**: Extensible architecture for adding new providers and scanners
- 🌍 **Cross-Platform**: Native support for Linux, macOS, and Windows
- ⚡ **Performance**: Rust-powered core with zero-cost abstractions

## Quick Start

### Installation

#### Build from Source (Recommended for v0.2.0 Beta)

```bash
# Clone the repository
git clone https://github.com/robottwo/aicred
cd aicred

# Build the CLI
cargo build --release

# The binary will be at target/release/aicred
# Optional: add to PATH
sudo cp target/release/aicred /usr/local/bin/
```

#### Package Managers (Coming Soon)

Official packages for crates.io, PyPI, Homebrew, and Scoop will be available with the v0.2.0 stable release.

```bash
# Coming soon:
cargo install aicred      # Rust crates.io
pip install aicred        # Python PyPI
brew install aicred       # macOS Homebrew
scoop install aicred      # Windows Scoop
```

### First Run

After installation, AICred can discover and consolidate your existing AI configurations:

```bash
# Discover all AI credentials on your system
aicred scan --format table

# Review discovered provider instances
aicred instances list

# Import discovered configurations to the unified format
aicred scan --update

# Verify your consolidated configuration
aicred instances list --verbose
```

The unified configuration is stored in `~/.config/aicred/instances.yaml` (Linux/macOS) or `%APPDATA%\aicred\instances.yaml` (Windows).

### Advanced: Semantic Labels

AICred's label system lets you tag provider:model combinations with semantic names like "fast", "smart", or "cheap":

```bash
# Create labels for different use cases
aicred labels add --name "fast" --description "Fast, cheap models for quick tasks"
aicred labels add --name "smart" --description "High-quality models for complex work"

# Assign labels to specific provider:model combinations
aicred labels assign --name "fast" --instance-id my-groq --model-id llama3-70b-8192
aicred labels assign --name "smart" --instance-id my-openai --model-id gpt-4

# Run applications with automatic configuration
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

📚 **[Full Documentation](docs/)** - Comprehensive guides for all aspects of AICred

### Getting Started
- [Installation Guide](docs/installation.md) - Build from source and package managers
- [User Guide](docs/user-guide.md) - Complete CLI usage walkthrough
- [GUI Usage Guide](docs/gui-usage-guide.md) - Desktop application guide

### Advanced Topics
- [Architecture](docs/architecture.md) - System design and internals
- [API Reference](docs/api-reference.md) - Library API documentation
- [Plugin Development](docs/plugin-development.md) - Creating custom providers and scanners
- [Migration Guide](docs/migration-guide.md) - Upgrading from v0.1 to v0.2

### Operations
- [Security](docs/security.md) - Security best practices and audit logging
- [Tagging System Guide](docs/tagging-system-guide.md) - Advanced label management

> 💡 **Note:** Documentation is actively being refined during the beta period. Feel free to [open an issue](https://github.com/robottwo/aicred/issues) if something is unclear!

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

## Roadmap

### v0.2.0 (Current - Beta)
- [x] Core scanning and detection engine
- [x] Provider plugin system (OpenAI, Anthropic, Groq, OpenRouter, HuggingFace, Ollama, LiteLLM)
- [x] Label system for semantic model tagging
- [x] CLI with rich output formats
- [x] Python and Go bindings
- [x] Cross-platform support (Linux, macOS, Windows)
- [x] GUI application (Tauri-based)

### v0.2.0 Stable (Next)
- [ ] Publish to crates.io, PyPI, npm
- [ ] Homebrew tap and Scoop bucket
- [ ] Comprehensive end-user documentation
- [ ] Video tutorials and demos

### v0.3.0 and Beyond
- [ ] Secrets vault integration (1Password, Bitwarden, HashiCorp Vault)
- [ ] Web dashboard for team configuration management
- [ ] Cost tracking across providers
- [ ] Usage analytics and optimization recommendations
- [ ] Configuration drift detection
- [ ] Support for additional providers (Cohere, AI21, etc.)
- [ ] Browser extension for web-based AI tools

### Community Ideas
Have a feature request? [Open an issue](https://github.com/robottwo/aicred/issues) or join the discussion!

---

**Why "AICred"?** **AI** + **Cred**entials = A unified way to manage authentication and configuration for AI tools.
