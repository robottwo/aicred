# Roadmap

## Completed

### v0.2.0 - Tagging & Enhanced Management
- [x] **Comprehensive Tagging System**: Support for categorizing provider instances with non-unique tags.
- [x] **Labeling System**: Unique identifiers for specific provider:model combinations (e.g., "fast", "production").
- [x] **GUI Integration**: Visual management for tags and labels in the Tauri application.
- [x] **Enhanced CLI**: Complete `aicred tags` and `aicred labels` command suites.
- [x] **Configuration Management**: YAML-based storage for tags/labels with backup and restore.

### v0.1.0 - Initial Release & Core Features
- [x] **Core Library**: Rust-based core with plugin architecture for providers and scanners.
- [x] **CLI Tool**: `aicred` command-line interface with JSON, NDJSON, and table output.
- [x] **GUI Application**: Cross-platform desktop app built with Tauri.
- [x] **Language Bindings**: Python (PyO3) and Go (CGo) bindings.
- [x] **Provider Support**: OpenAI, Anthropic, HuggingFace, Ollama, LangChain, LiteLLM.
- [x] **Application Scanners**: Discovery of configs for Roo Code, Claude Desktop, Ragit.
- [x] **Auto-Configuration**: `aicred wrap` command to inject credentials into any application.
- [x] **Security**: Default redaction of secrets, secure storage integration.

## In Progress / Upcoming

### Integrations & Ecosystem
- [ ] **Deep LangChain Integration**: Enhanced support for LangChain's ecosystem beyond basic provider config.
- [ ] **LlamaIndex Support**: Dedicated provider/scanner support for LlamaIndex.
- [ ] **Additional Scanners**: Support for VSCode (generic), JetBrains IDEs, and other dev tools.

### Core Features
- [ ] **Cloud Sync**: Securely sync configurations across devices (Commercial Integration).
- [ ] **Team Sharing**: Mechanisms to share authorized configurations/labels within a team.
- [ ] **Credential Rotation**: Automated rotation of API keys for supported providers.
- [ ] **Usage Analytics**: Local tracking of token usage and cost estimation.

## Future Vision

- **Enterprise SSO**: Integration with Okta/Auth0 for organizational access control.
- **Policy Enforcement**: Define and enforce policies (e.g., "No GPT-4 for dev environment").
- **Universal AI Credential Standard**: Establish a widely adopted standard for AI config discovery.
