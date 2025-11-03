# AICred

🔍 A cross-platform library for discovering GenAI API keys and configurations across various providers and applications.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/robottwo/aicred/workflows/CI/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/ci.yml)
[![Python](https://github.com/robottwo/aicred/workflows/Python%20Bindings/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/python.yml)
[![Go](https://github.com/robottwo/aicred/workflows/Go%20Bindings/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/go.yml)
[![GUI](https://github.com/robottwo/aicred/workflows/GUI%20Application/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/gui.yml)
[![Security](https://github.com/robottwo/aicred/workflows/Security%20Audit/badge.svg)](https://github.com/robottwo/aicred/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/robottwo/aicred/branch/main/graph/badge.svg)](https://codecov.io/gh/robottwo/aicred)

## Features

- 🔐 **Security-First**: Keys are redacted by default with SHA-256 hashing
- 🔌 **Plugin Architecture**: Extensible system for adding new providers
- 🌍 **Cross-Platform**: Works on Linux, macOS, and Windows
- 🚀 **Multiple Interfaces**: Library, CLI, Python, Go, and GUI
- 📊 **Rich Output**: JSON, NDJSON, table, and summary formats
- 🎯 **Smart Detection**: High-confidence key identification

## Supported Providers

- OpenAI
- Anthropic (Claude)
- Hugging Face
- Ollama
- LangChain
- LiteLLM

## Supported Applications

- Roo Code (VSCode extension)
- Claude Desktop
- Ragit
- LangChain applications

## Quick Start

### CLI

```bash
# Install
cargo install aicred

# Scan for credentials
aicred scan

# Scan with options
aicred scan --format json --only openai,anthropic
```

### Python

```bash
pip install aicred
```

```python
import aicred

result = aicred.scan()
print(f"Found {len(result['keys'])} keys")
```

### Go

```bash
go get github.com/robottwo/aicred/bindings/go
```

```go
import "github.com/robottwo/aicred/bindings/go"

result, err := aicred.Scan(aicred.ScanOptions{})
```

### Rust Library

```toml
[dependencies]
aicred-core = "0.1.0"
```

```rust
use aicred_core::{scan, ScanOptions};

let result = scan(ScanOptions::default())?;
```

## Installation

See detailed installation instructions in [docs/installation.md](docs/installation.md)

## Documentation

- [Architecture](docs/architecture.md)
- [Installation Guide](docs/installation.md)
- [User Guide](docs/user-guide.md)
- [API Reference](docs/api-reference.md)
- [Plugin Development](docs/plugin-development.md)
- [Security](docs/security.md)

## Project Structure

```
aicred/
├── core/              # Core Rust library
├── ffi/               # C-API layer
├── cli/               # Command-line tool
├── gui/               # Tauri desktop application
├── bindings/
│   ├── python/        # Python bindings
│   └── go/            # Go bindings
└── docs/              # Documentation
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Security

⚠️ **Important**: By default, all secrets are redacted. Only use `include_full_values` in secure environments.

For security concerns, please see [docs/security.md](docs/security.md).
