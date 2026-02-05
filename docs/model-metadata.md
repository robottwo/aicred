# Model Metadata Database

This document describes the comprehensive model metadata database implemented in AICred, providing detailed information about 50+ AI models from various providers.

## Overview

The model metadata database (`ModelRegistry`) is a centralized source of truth for AI model information including:

- Model identifiers and human-readable names
- Provider information
- Capabilities (text, vision, code, function calling, etc.)
- Pricing details (input/output token costs)
- Technical architecture (modality, tokenizer, parameters)
- Context window sizes
- Release dates and status

## Supported Providers

### OpenAI
- GPT-4o (Omnimodal)
- GPT-4o Mini
- GPT-4 Turbo
- GPT-3.5 Turbo

### Anthropic
- Claude 3.5 Sonnet
- Claude 3.5 Haiku
- Claude 3 Opus
- Claude 3 Sonnet
- Claude 3 Haiku

### Google
- Gemini 2.0 Flash Experimental (Beta)
- Gemini 1.5 Pro
- Gemini 1.5 Flash
- Gemma 2 27B

### Meta
- Llama 3.3 70B Instruct
- Llama 3.1 405B Instruct
- Llama 3.1 70B Instruct
- Llama 3.1 8B Instruct

### Mistral AI
- Mistral Large 2411
- Pixtral 12B (Vision)
- Mistral Nemo
- Codestral 25.01
- Mixtral 8x7B
- Open Mistral 7B

### Cohere
- Command R+
- Command R
- Command Light

### xAI
- Grok 2
- Grok 2 Vision
- Grok Beta

### DeepSeek
- DeepSeek Chat
- DeepSeek Coder

### Alibaba (Qwen)
- Qwen 2.5 72B Instruct
- Qwen VL Plus
- Qwen 2 72B Instruct

### OpenRouter
- Claude 3.5 Sonnet
- Gemini Pro 1.5
- GPT-4o
- Llama 3.3 70B
- DeepSeek Chat

### Additional Providers
- Microsoft (Phi-3)
- Databricks (DBRX)
- 01.AI (Yi)
- Stability AI (StableLM)
- TII (Falcon)
- MosaicML (MPT)

## Data Structure

### ModelEntry

```rust
pub struct ModelEntry {
    pub id: String,              // Unique identifier
    pub name: String,            // Human-readable name
    pub provider: String,        // Provider name
    pub family: Option<String>,  // Model family/series
    pub description: Option<String>,
    pub context_length: u32,    // Context window in tokens
    pub pricing: ModelPricing,
    pub capabilities: ModelCapabilities,
    pub architecture: ModelArchitecture,
    pub released: Option<String>, // Release date (YYYY-MM-DD)
    pub status: ModelStatus,
}
```

### ModelCapabilities

```rust
pub struct ModelCapabilities {
    pub text: bool,
    pub image: bool,
    pub vision: bool,
    pub code: bool,
    pub function_calling: bool,
    pub streaming: bool,
    pub json_mode: bool,
    pub system_prompt: bool,
    pub audio_in: bool,
    pub audio_out: bool,
}
```

### ModelPricing

```rust
pub struct ModelPricing {
    pub input: f64,           // Cost per input token (USD)
    pub output: f64,          // Cost per output token (USD)
    pub cached_input: Option<f64>, // Cache discount modifier
    pub currency: String,
}
```

### ModelArchitecture

```rust
pub struct ModelArchitecture {
    pub modality: String,         // "text", "multimodal", etc.
    pub parameters: Option<String>, // e.g., "70B", "405B"
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}
```

### ModelStatus

```rust
pub enum ModelStatus {
    Active,
    Beta,
    Deprecated,
    Archived,
}
```

## API Usage

### Rust

#### Initialize Registry

```rust
use aicred_core::models::model_registry::ModelRegistry;

let registry = ModelRegistry::new();
println!("Total models: {}", registry.count());
```

#### Get Model by ID

```rust
if let Some(model) = registry.get("gpt-4o") {
    println!("Name: {}", model.name);
    println!("Context: {} tokens", model.context_length);
}
```

#### List by Provider

```rust
let openai_models = registry.by_provider("openai");
for model in openai_models {
    println!("{}: {}", model.id, model.name);
}
```

#### Filter by Capability

```rust
use aicred_core::models::model_registry::CapabilityFilter;

let code_models = registry.by_capability(CapabilityFilter::Code);
for model in code_models {
    println!("{}: {}", model.name, model.id);
}
```

#### Search Models

```rust
let results = registry.search("gpt");
for model in results {
    println!("Found: {}", model.name);
}
```

### Go

#### Initialize Registry

```go
import "github.com/robottwo/aicred/bindings/go/aicred"

registry := aicred.NewModelRegistry()
fmt.Printf("Total models: %d\n", registry.Count())
```

#### Get Model by ID

```go
model, exists := registry.Get("gpt-4o")
if exists {
    fmt.Printf("Name: %s\n", model.Name)
    fmt.Printf("Context: %d tokens\n", model.ContextLength)
}
```

#### List by Provider

```go
openaiModels := registry.ByProvider("openai")
for _, model := range openaiModels {
    fmt.Printf("%s: %s\n", model.ID, model.Name)
}
```

#### Filter by Capability

```go
codeModels := registry.ByCapability(aicred.CapCode)
for _, model := range codeModels {
    fmt.Printf("%s: %s\n", model.Name, model.ID)
}
```

#### Search Models

```go
results := registry.Search("gpt")
for _, model := range results {
    fmt.Printf("Found: %s\n", model.Name)
}
```

## CLI Usage

### List All Models

```bash
aicred models list
```

### List with Details

```bash
aicred models list --verbose
```

### Filter by Provider

```bash
aicred models list --provider openai
```

### Filter by Capability

```bash
aicred models list --capability code
aicred models list --capability vision
aicred models list --capability function
```

### Search Models

```bash
aicred models list --search "gpt"
aicred models list --search "claude"
```

### Get Model Details

```bash
aicred models get gpt-4o
```

### Compare Pricing

```bash
aicred models compare gpt-4o claude-3-5-sonnet gemini-1.5-pro
```

### View Statistics

```bash
aicred models stats
```

### List by Specific Capability

```bash
aicred models capability code
aicred models capability vision
aicred models capability streaming
```

## Capability Filters

The following capability filters are available:

- `text` - Text generation capability
- `image` - Image generation capability
- `vision` - Vision/multimodal understanding
- `code` - Code generation capability
- `function` or `function-calling` - Function/tool calling
- `streaming` - Streaming response support
- `json` or `json-mode` - JSON output mode

## Pricing Information

All prices are in USD per token unless otherwise noted. Prices are approximate and may vary by provider and region.

### Cost Calculation

Example: Calculate cost for 1,000 input tokens and 500 output tokens with GPT-4o:

```
Input cost: 1000 * $0.000005 = $0.005
Output cost: 500 * $0.000015 = $0.0075
Total: $0.0125
```

### Cost Comparison

To compare pricing across multiple models:

```bash
aicred models compare gpt-4o claude-3.5-sonnet gemini-1.5-pro
```

## Context Windows

The model registry includes context window sizes for all models:

| Model | Context Window |
|-------|---------------|
| Gemini 1.5 Pro | 2,000,000 tokens |
| Gemini 1.5 Flash | 1,000,000 tokens |
| Claude 3.x | 200,000 tokens |
| GPT-4o | 128,000 tokens |
| Llama 3.x | 131,072 tokens |
| GPT-3.5 Turbo | 16,385 tokens |

## Model Status

Models are categorized by their status:

- **Active** - Generally available and recommended
- **Beta** - Experimental features, may have limitations
- **Deprecated** - Being phased out, use alternatives
- **Archived** - No longer available

## Extending the Registry

### Adding a New Model (Rust)

```rust
self.add_model(ModelEntry {
    id: "new-model-id".to_string(),
    name: "New Model".to_string(),
    provider: "provider-name".to_string(),
    family: Some("model-family".to_string()),
    description: Some("Model description".to_string()),
    context_length: 128000,
    pricing: ModelPricing {
        input: 0.000001,
        output: 0.000002,
        cached_input: None,
        currency: "USD".to_string(),
    },
    capabilities: ModelCapabilities {
        text: true,
        image: false,
        vision: false,
        code: true,
        function_calling: true,
        streaming: true,
        json_mode: true,
        system_prompt: true,
        audio_in: false,
        audio_out: false,
    },
    architecture: ModelArchitecture {
        modality: "text".to_string(),
        parameters: Some("7B".to_string()),
        tokenizer: "tokenizer-name".to_string(),
        instruct_type: Some("instruct-format".to_string()),
    },
    released: Some("2024-01-01".to_string()),
    status: ModelStatus::Active,
});
```

### Adding a New Model (Go)

```go
registry.addModel(&aicred.ModelEntry{
    ID:   "new-model-id",
    Name: "New Model",
    Provider: "provider-name",
    Family: aicred.strPtr("model-family"),
    Description: aicred.strPtr("Model description"),
    ContextLength: 128000,
    Pricing: aicred.ModelPricing{
        Input: 0.000001,
        Output: 0.000002,
        Currency: "USD",
    },
    Capabilities: aicred.ModelCapabilities{
        Text: true,
        Code: true,
        FunctionCalling: true,
        Streaming: true,
        JsonMode: true,
        SystemPrompt: true,
    },
    Architecture: aicred.ModelArchitecture{
        Modality: "text",
        Parameters: aicred.strPtr("7B"),
        Tokenizer: "tokenizer-name",
        InstructType: aicred.strPtr("instruct-format"),
    },
    Released: aicred.strPtr("2024-01-01"),
    Status: aicred.StatusActive,
})
```

## Testing

The model registry includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run tests for model_registry module
cargo test model_registry

# Run tests for specific filter
cargo test test_by_provider
```

## Data Accuracy

The model metadata database is maintained as best-effort information:

- **Pricing**: Prices are approximate and may change
- **Context Windows**: Verified from provider documentation
- **Capabilities**: Based on official provider documentation
- **Release Dates**: When publicly available

### Keeping Data Current

To update model information:

1. Verify new information from official provider sources
2. Update the relevant model entries in `core/src/models/model_registry.rs`
3. Mirror changes to Go bindings in `bindings/go/aicred/model_registry.go`
4. Run tests to ensure consistency
5. Update documentation if needed

## Future Enhancements

Potential future improvements to the model registry:

- [ ] Automatic pricing updates from provider APIs
- [ ] Benchmark performance data
- [ ] Model quality metrics
- [ ] Regional availability
- [ ] Rate limiting information
- [ ] Fine-tuning support details
- [ ] Model version history
- [ ] Deprecation timelines

## License

The model metadata database is part of the AICred project and follows the same license terms.

## Contributing

To add new models or update existing information:

1. Follow the existing code structure
2. Include tests for new functionality
3. Update documentation
4. Submit a pull request

## Related Documentation

- [Core Models Documentation](../core/src/models/README.md)
- [CLI Command Reference](../cli/src/commands/README.md)
- [Go Bindings Documentation](../bindings/go/README.md)
- [Provider Integration Guide](../docs/provider-integration.md)
