# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2025-11-03

### Added
- **Comprehensive Tagging and Labeling System**
  - Tag management: Non-unique identifiers for categorization
  - Label management: Unique identifiers for designation
  - Tag/label assignment to provider instances and specific models
  - Rich metadata support for tags, labels, and assignments
  - Color coding and descriptions for visual organization
- **Enhanced CLI Commands**
  - `aicred tags list/add/update/remove/assign/unassign` - Complete tag management
  - `aicred labels list/add/update/remove/assign/unassign` - Complete label management
  - Validation and constraint checking for all operations
  - Confirmation prompts for destructive operations
- **GUI Integration**
  - Visual tag and label management interface
  - Color picker for tag/label customization
  - Assignment modal for easy tag/label assignment
  - Real-time updates and validation feedback
  - Enhanced instance view with tag/label display
- **Configuration Management**
  - YAML-based storage for tags and labels
  - Automatic configuration file creation and validation
  - Backup and restore capabilities
  - Migration support from previous versions
- **Developer Features**
  - Comprehensive API for tag/label operations
  - Extension points for custom validation and processing
  - Integration hooks for external systems
  - Performance optimizations for large datasets

### Changed
- **Architecture Improvements**
  - Separated scanner plugins (discovery) from provider plugins (validation)
  - Enhanced plugin system with better separation of concerns
  - Improved error handling and validation throughout the system
- **Provider Instance Model**
  - Simplified single-key approach for easier management
  - Maintained backward compatibility with existing multi-key configurations
  - Enhanced validation and conversion utilities

### Documentation
- **New Documentation**
  - [Tagging System Guide](docs/tagging-system-guide.md) - Comprehensive user guide
  - [Migration Guide](docs/migration-guide.md) - Upgrade instructions and troubleshooting
  - [Tagging Development Guide](docs/tagging-development.md) - Developer extension documentation
- **Updated Documentation**
  - [User Guide](docs/user-guide.md) - Added tag/label usage examples
  - [API Reference](docs/api-reference.md) - Added complete tag/label API documentation
  - [Architecture Documentation](docs/architecture.md) - Updated with tagging system architecture

### Migration
- **Backward Compatibility**
  - All existing configurations continue to work unchanged
  - Automatic detection and handling of old configuration formats
  - Gradual migration path to new features
- **Upgrade Tools**
  - Configuration validation and repair utilities
  - Backup and restore procedures
  - Migration scripts for complex configurations

### Testing
- **Enhanced Test Coverage**
  - Unit tests for all tag/label models and operations
  - Integration tests for complete workflows
  - Validation and constraint testing
  - Performance tests for scalability

## [0.1.0] - 2025-10-27

### Added
- Initial release
- Core library with plugin architecture
- CLI tool with multiple output formats
- Python bindings via PyO3
- Go bindings via CGo
- Tauri GUI application
- Support for OpenAI, Anthropic, HuggingFace, Ollama, LangChain, LiteLLM
- Application scanners for Roo Code, Claude Desktop, Ragit
- Comprehensive test suite (160+ tests)
- Cross-platform support (Linux, macOS, Windows)
# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-10-27

### Added
- Initial release
- Core library with plugin architecture
- CLI tool with multiple output formats
- Python bindings via PyO3
- Go bindings via CGo
- Tauri GUI application
- Support for OpenAI, Anthropic, HuggingFace, Ollama, LangChain, LiteLLM
- Application scanners for Roo Code, Claude Desktop, Ragit
- Comprehensive test suite (160+ tests)
- Cross-platform support (Linux, macOS, Windows)