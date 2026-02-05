# AICred Go Bindings

Native Go library for managing GenAI provider configurations, models, tags, and labels.

This is a pure Go implementation with idiomatic interfaces and no CGO dependencies.

## Features

- **Configuration Management**: Load, save, and validate configuration files
- **Provider Instances**: Manage individual provider configurations with models
- **Model Management**: Define AI models with capabilities and metadata
- **Tagging System**: Non-unique identifiers for categorization and organization
- **Labeling System**: Unique identifiers with scoping and uniqueness enforcement
- **Validation**: Comprehensive validation for all data structures
- **Path Utilities**: Cross-platform file path handling
- **Thread-Safe**: Thread-safe configuration and repository operations

## Installation

```bash
go get github.com/robottwo/aicred/bindings/go/aicred
```

## Quick Start

### Create a Provider Instance

```go
package main

import (
    "fmt"
    "log"

    "github.com/robottwo/aicred/bindings/go/aicred"
)

func main() {
    // Create a provider instance
    instance := aicred.NewProviderInstance(
        "openai-prod",
        "OpenAI Production",
        "openai",
        "https://api.openai.com/v1",
    )

    // Add metadata
    instance.SetMetadataValue("environment", "production")
    instance.SetMetadataValue("region", "us-east-1")

    // Add models
    model := aicred.NewModel("gpt-4", "GPT-4").
        WithContextWindow(128000).
        WithCapabilities(&aicred.Capabilities{
            TextGeneration:  true,
            FunctionCalling: true,
            Streaming:       true,
        })
    instance.AddModel(model)

    // Set API key
    instance.SetAPIKey("sk-proj-...")

    // Validate
    if err := instance.Validate(); err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Instance: %s\n", instance.DisplayName)
    fmt.Printf("Models: %d\n", instance.ModelCount())
}
```

### Configuration Management

```go
// Load default configuration
config, err := aicred.LoadDefaultConfig()
if err != nil {
    log.Fatal(err)
}

// Customize configuration
config.HomeDir = "/custom/path"
config.Security.MaxFiles = 5000
config.Security.RedactSecrets = true

// Save configuration
if err := config.Save("/path/to/config.json"); err != nil {
    log.Fatal(err)
}

// Validate configuration
if err := config.Validate(); err != nil {
    log.Fatal(err)
}
```

### Tags and Labels

```go
// Create tag repository
tagRepo := aicred.NewTagRepository()

// Add tags
production := aicred.NewTag("prod", "Production").
    WithColor("#FF0000").
    WithDescription("Production environment")
tagRepo.AddTag(production)

// Assign tag to target
assignment := aicred.NewTagAssignment("assign-1", "prod", "provider_instance", "openai-prod", "")
tagRepo.Assign(assignment)

// Get tags for target
tags := tagRepo.GetTagsForTarget("provider_instance", "openai-prod", "")
for _, tag := range tags {
    fmt.Printf("Tag: %s\n", tag.Name)
}

// Create label repository (labels are unique)
labelRepo := aicred.NewLabelRepository()

// Add label
primary := aicred.NewLabel("primary", "Primary").
    WithColor("#0000FF").
    WithDescription("Primary instance")
labelRepo.AddLabel(primary)

// Assign label to target
labelAssignment := aicred.NewLabelAssignment("l-assign-1", "primary", "provider_instance", "openai-prod", "")
labelRepo.Assign(labelAssignment)

// Get label for target
label, err := labelRepo.GetLabelForTarget("provider_instance", "openai-prod", "")
if err == nil {
    fmt.Printf("Label: %s\n", label.Name)
}
```

## API Reference

### Configuration

#### `DefaultConfig() *Config`
Returns a default configuration with sensible defaults.

#### `LoadConfig(path string) (*Config, error)`
Loads configuration from a JSON file.

#### `LoadDefaultConfig() (*Config, error)`
Loads configuration from the default location.

#### `(*Config) Save(path string) error`
Saves configuration to a JSON file.

#### `(*Config) Validate() error`
Validates the configuration.

### Provider Instances

#### `NewProviderInstance(id, displayName, providerType, baseURL string) *ProviderInstance`
Creates a new provider instance.

#### `(*ProviderInstance) AddModel(model *Model)`
Adds a model to the instance.

#### `(*ProviderInstance) SetAPIKey(apiKey string)`
Sets the API key for the instance.

#### `(*ProviderInstance) Validate() error`
Validates the instance.

### Models

#### `NewModel(modelID, name string) *Model`
Creates a new model.

#### `(*Model) WithCapabilities(capabilities *Capabilities) *Model`
Sets model capabilities (builder pattern).

#### `(*Model) WithContextWindow(size uint32) *Model`
Sets the context window size (builder pattern).

#### `(*Model) WithTemperature(temperature float32) *Model`
Sets the temperature (builder pattern).

#### `(*Model) WithTags(tags []string) *Model`
Sets tags for the model (builder pattern).

### Tags

#### `NewTag(id, name string) *Tag`
Creates a new tag.

#### `NewTagRepository() *TagRepository`
Creates a new tag repository.

#### `(*TagRepository) AddTag(tag *Tag) error`
Adds a tag to the repository.

#### `(*TagRepository) Assign(assignment *TagAssignment) error`
Assigns a tag to a target.

#### `(*TagRepository) GetTagsForTarget(targetType, instanceID, modelID string) []*Tag`
Gets all tags assigned to a target.

### Labels

#### `NewLabel(id, name string) *Label`
Creates a new label.

#### `NewLabelRepository() *LabelRepository`
Creates a new label repository.

#### `(*LabelRepository) AddLabel(label *Label) error`
Adds a label to the repository.

#### `(*LabelRepository) Assign(assignment *LabelAssignment) error`
Assigns a label to a target (enforces uniqueness).

#### `(*LabelRepository) GetLabelForTarget(targetType, instanceID, modelID string) (*Label, error)`
Gets the unique label assigned to a target.

## Data Models

### Config
Main configuration structure with security and plugin options.

### ProviderInstance
Individual provider configuration with models and metadata.

### Model
AI model definition with capabilities, costs, and metadata.

### Tag
Non-unique identifier for categorization (can be assigned to multiple targets).

### Label
Unique identifier with scoping (only one target per label).

### TagAssignment / LabelAssignment
Links tags/labels to targets (provider instances or models).

## Validation

All data structures include comprehensive validation:

```go
// Validate configuration
if err := config.Validate(); err != nil {
    log.Fatal(err)
}

// Validate provider instance
if err := instance.Validate(); err != nil {
    log.Fatal(err)
}

// Validate model
if err := model.Validate(); err != nil {
    log.Fatal(err)
}

// Validate tag
if err := tag.Validate(); err != nil {
    log.Fatal(err)
}

// Validate label
if err := label.Validate(); err != nil {
    log.Fatal(err)
}
```

## Path Utilities

Cross-platform path handling:

```go
// Get home directory
homeDir, err := aicred.GetHomeDir()

// Get config directory
configDir, err := aicred.GetConfigDir()

// Check if file exists
if aicred.FileExists(path) {
    fmt.Println("File exists")
}

// Get file extension
ext := aicred.GetFileExtension("config.json") // "json"

// Check extension
if aicred.HasExtension("file.json", "json") {
    fmt.Println("Is JSON file")
}
```

## Testing

Run all tests:

```bash
go test ./aicred/...
```

Run tests with coverage:

```bash
go test -cover ./aicred/...
```

Run specific test file:

```bash
go test -v ./aicred/config_test.go
```

## Examples

See the `examples/` directory for complete examples:

- `basic_usage`: Creating and managing provider instances
- `tags_and_labels`: Tag and label management

Run examples:

```bash
cd examples/basic_usage
go run main.go

cd examples/tags_and_labels
go run main.go
```

## Security

- All validation functions ensure data integrity
- Paths are sanitized to prevent traversal attacks
- Metadata validation prevents injection attacks
- By default, secrets are redacted in configurations
- Configuration files are created with restricted permissions (0600)

## Platform Support

- **Linux**: ✅ (x86_64, aarch64)
- **macOS**: ✅ (x86_64, aarch64/Apple Silicon)
- **Windows**: ✅ (x86_64)

## License

This project is part of the AICred project. See the main repository for license information.

## Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## Related Projects

- **AICred Core**: Rust core library
- **AICred CLI**: Command-line interface
- **AICred GUI**: Tauri-based GUI application
