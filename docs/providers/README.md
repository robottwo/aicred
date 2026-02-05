# AI Provider Documentation

AICred supports 21 AI providers for unified configuration management. This document provides detailed information about each supported provider.

## Table of Contents

- [OpenAI](#openai)
- [Anthropic](#anthropic)
- [Groq](#groq)
- [Hugging Face](#hugging-face)
- [Ollama](#ollama)
- [OpenRouter](#openrouter)
- [LiteLLM](#litellm)
- [AWS Bedrock](#aws-bedrock)
- [Azure OpenAI](#azure-openai)
- [Cohere](#cohere)
- [DeepInfra](#deepinfra)
- [DeepSeek](#deepseek)
- [Fireworks AI](#fireworks-ai)
- [Google AI](#google-ai)
- [Grok (xAI)](#grok-xai)
- [Mistral AI](#mistral-ai)
- [Moonshot AI](#moonshot-ai)
- [Perplexity AI](#perplexity-ai)
- [Replicate](#replicate)
- [Together AI](#together-ai)
- [ZAI](#zai)

---

## OpenAI

**Provider ID:** `openai`

**Base URL:** `https://api.openai.com/v1`

**Key Prefix:** `sk-`

**Default Models:**
- `gpt-4o`
- `gpt-4o-mini`

### API Key Format

OpenAI API keys typically start with `sk-` and are 51 characters long.

### Example Configuration

```yaml
my-openai:
  provider_type: openai
  base_url: https://api.openai.com/v1
  api_key: sk-proj-...
  models: [gpt-4o, gpt-4o-mini]
```

### Supported Model Types

- Chat completion
- Embedding
- Image generation (via DALL-E)

---

## Anthropic

**Provider ID:** `anthropic`

**Base URL:** `https://api.anthropic.com`

**Key Prefix:** `sk-ant-`

**Default Models:**
- `claude-3-5-sonnet-20241022`

### API Key Format

Anthropic API keys start with `sk-ant-` and are 91 characters long.

### Example Configuration

```yaml
my-anthropic:
  provider_type: anthropic
  base_url: https://api.anthropic.com
  api_key: sk-ant-...
  models: [claude-3-5-sonnet-20241022]
```

### Supported Model Types

- Chat completion (Claude 3.x series)

---

## Groq

**Provider ID:** `groq`

**Base URL:** `https://api.groq.com/openai/v1`

**Key Prefix:** `gsk_`

**Default Models:**
- `llama-3.3-70b-versatile`

### API Key Format

Groq API keys start with `gsk_` and are 52 characters long.

### Example Configuration

```yaml
my-groq:
  provider_type: groq
  base_url: https://api.groq.com/openai/v1
  api_key: gsk_...
  models: [llama-3.3-70b-versatile]
```

### Supported Model Types

- Chat completion
- Fast inference

---

## Hugging Face

**Provider ID:** `huggingface`

**Base URL:** `https://api-inference.huggingface.co`

**Key Prefix:** `hf_`

**Default Models:**
- `gpt2`
- `bert-base-uncased`

### API Key Format

Hugging Face tokens start with `hf_` and are 37 characters long.

### Example Configuration

```yaml
my-huggingface:
  provider_type: huggingface
  base_url: https://api-inference.huggingface.co
  api_key: hf_...
  models: [gpt2, bert-base-uncased]
```

### Supported Model Types

- Chat completion
- Embedding
- Custom model inference

---

## Ollama

**Provider ID:** `ollama`

**Base URL:** `http://localhost:11434`

**Requires Auth:** No (local runner)

**Default Models:**
- `llama3`
- `mistral`

### Example Configuration

```yaml
my-ollama:
  provider_type: ollama
  base_url: http://localhost:11434
  models: [llama3, mistral]
```

### Supported Model Types

- Chat completion
- Local model execution

---

## OpenRouter

**Provider ID:** `openrouter`

**Base URL:** `https://openrouter.ai/api/v1`

**Key Prefix:** `sk-or-`

**Default Models:**
- `anthropic/claude-3.5-sonnet`

### API Key Format

OpenRouter API keys start with `sk-or-` and are 51 characters long.

### Example Configuration

```yaml
my-openrouter:
  provider_type: openrouter
  base_url: https://openrouter.ai/api/v1
  api_key: sk-or-...
  models: [anthropic/claude-3.5-sonnet]
```

### Supported Model Types

- Chat completion (access to 100+ models)

---

## LiteLLM

**Provider ID:** `litellm`

**Base URL:** `http://localhost:4000`

**Key Prefix:** `sk-`

**Default Models:**
- `gpt-4`

### Example Configuration

```yaml
my-litellm:
  provider_type: litellm
  base_url: http://localhost:4000
  api_key: sk-...
  models: [gpt-4]
```

### Supported Model Types

- Chat completion
- Unified API proxy for multiple providers

---

## AWS Bedrock

**Provider ID:** `aws-bedrock`

**Base URL:** `https://bedrock-runtime.us-east-1.amazonaws.com`

**Key Prefix:** `AKIA`

**Default Models:**
- `anthropic.claude-3-5-sonnet-20241022-v2:0`

### API Key Format

AWS Bedrock uses AWS access keys that start with `AKIA` and are 20 characters long.

### Example Configuration

```yaml
my-bedrock:
  provider_type: aws-bedrock
  base_url: https://bedrock-runtime.us-east-1.amazonaws.com
  api_key: AKIAIOSFODNN7EXAMPLE
  models: [anthropic.claude-3-5-sonnet-20241022-v2:0]
```

### Supported Model Types

- Chat completion
- Embedding
- Access to multiple foundation models (Anthropic, Amazon, AI21, Cohere, Meta, Mistral)

---

## Azure OpenAI

**Provider ID:** `azure`

**Base URL:** `https://YOUR_RESOURCE.openai.azure.com`

**Default Models:**
- `gpt-4`

### Example Configuration

```yaml
my-azure:
  provider_type: azure
  base_url: https://YOUR_RESOURCE.openai.azure.com
  api_key: YOUR_AZURE_KEY
  models: [gpt-4]
```

### Supported Model Types

- Chat completion
- Embedding
- Azure-specific deployments

---

## Cohere

**Provider ID:** `cohere`

**Base URL:** `https://api.cohere.ai/v1`

**Default Models:**
- `command-r-plus`

### Example Configuration

```yaml
my-cohere:
  provider_type: cohere
  base_url: https://api.cohere.ai/v1
  api_key: YOUR_COHERE_KEY
  models: [command-r-plus]
```

### Supported Model Types

- Chat completion
- Embedding
- Text generation

---

## DeepInfra

**Provider ID:** `deepinfra`

**Base URL:** `https://api.deepinfra.com/v1/openai`

**Default Models:**
- `meta-llama/Meta-Llama-3.1-70B-Instruct`

### Example Configuration

```yaml
my-deepinfra:
  provider_type: deepinfra
  base_url: https://api.deepinfra.com/v1/openai
  api_key: YOUR_DEEPINFRA_KEY
  models: [meta-llama/Meta-Llama-3.1-70B-Instruct]
```

### Supported Model Types

- Chat completion
- Fast inference

---

## DeepSeek

**Provider ID:** `deepseek`

**Base URL:** `https://api.deepseek.com/v1`

**Default Models:**
- `deepseek-chat`

### Example Configuration

```yaml
my-deepseek:
  provider_type: deepseek
  base_url: https://api.deepseek.com/v1
  api_key: YOUR_DEEPSEEK_KEY
  models: [deepseek-chat]
```

### Supported Model Types

- Chat completion
- Code generation

---

## Fireworks AI

**Provider ID:** `fireworks`

**Base URL:** `https://api.fireworks.ai/inference/v1`

**Key Prefix:** `fw_`

**Default Models:**
- `accounts/fireworks/models/llama-v3p1-70b-instruct`

### Example Configuration

```yaml
my-fireworks:
  provider_type: fireworks
  base_url: https://api.fireworks.ai/inference/v1
  api_key: fw_...
  models: [accounts/fireworks/models/llama-v3p1-70b-instruct]
```

### Supported Model Types

- Chat completion
- Fast inference

---

## Google AI

**Provider ID:** `google`

**Base URL:** `https://generativelanguage.googleapis.com/v1beta`

**Default Models:**
- `gemini-1.5-pro`

### Example Configuration

```yaml
my-google:
  provider_type: google
  base_url: https://generativelanguage.googleapis.com/v1beta
  api_key: YOUR_GOOGLE_API_KEY
  models: [gemini-1.5-pro]
```

### Supported Model Types

- Chat completion
- Multimodal (text, images, audio, video)
- Function calling

---

## Grok (xAI)

**Provider ID:** `grok`

**Base URL:** `https://api.x.ai/v1`

**Default Models:**
- `grok-beta`

### Example Configuration

```yaml
my-grok:
  provider_type: grok
  base_url: https://api.x.ai/v1
  api_key: YOUR_GROK_KEY
  models: [grok-beta]
```

### Supported Model Types

- Chat completion
- Real-time information

---

## Mistral AI

**Provider ID:** `mistral`

**Base URL:** `https://api.mistral.ai/v1`

**Default Models:**
- `mistral-large-latest`

### Example Configuration

```yaml
my-mistral:
  provider_type: mistral
  base_url: https://api.mistral.ai/v1
  api_key: YOUR_MISTRAL_KEY
  models: [mistral-large-latest]
```

### Supported Model Types

- Chat completion
- Embedding
- Open-source models

---

## Moonshot AI

**Provider ID:** `moonshot`

**Base URL:** `https://api.moonshot.cn/v1`

**Default Models:**
- `moonshot-v1-8k`

### Example Configuration

```yaml
my-moonshot:
  provider_type: moonshot
  base_url: https://api.moonshot.cn/v1
  api_key: YOUR_MOONSHOT_KEY
  models: [moonshot-v1-8k]
```

### Supported Model Types

- Chat completion
- Long context (up to 128k tokens)

---

## Perplexity AI

**Provider ID:** `perplexity`

**Base URL:** `https://api.perplexity.ai`

**Key Prefix:** `pplx-`

**Default Models:**
- `llama-3.1-sonar-small-128k-online`

### Example Configuration

```yaml
my-perplexity:
  provider_type: perplexity
  base_url: https://api.perplexity.ai
  api_key: pplx-...
  models: [llama-3.1-sonar-small-128k-online]
```

### Supported Model Types

- Chat completion
- Search-augmented answers
- Real-time web search

---

## Replicate

**Provider ID:** `replicate`

**Base URL:** `https://api.replicate.com/v1`

**Key Prefix:** `r8_`

**Default Models:**
- `meta/meta-llama-3.1-405b-instruct`

### Example Configuration

```yaml
my-replicate:
  provider_type: replicate
  base_url: https://api.replicate.com/v1
  api_key: r8_...
  models: [meta/meta-llama-3.1-405b-instruct]
```

### Supported Model Types

- Chat completion
- Image generation
- Custom model deployment

---

## Together AI

**Provider ID:** `together`

**Base URL:** `https://api.together.xyz/v1`

**Default Models:**
- `meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo`

### Example Configuration

```yaml
my-together:
  provider_type: together
  base_url: https://api.together.xyz/v1
  api_key: YOUR_TOGETHER_KEY
  models: [meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo]
```

### Supported Model Types

- Chat completion
- Open-source models
- Fine-tuning

---

## ZAI

**Provider ID:** `zai`

**Base URL:** `https://api.z.ai/v1`

**Default Models:**
- `zai-1`

### Example Configuration

```yaml
my-zai:
  provider_type: zai
  base_url: https://api.z.ai/v1
  api_key: YOUR_ZAI_KEY
  models: [zai-1]
```

### Supported Model Types

- Chat completion
- Fast inference

---

## Provider Metadata System

AICred includes a comprehensive provider metadata system that tracks:

- **Provider identifiers** and display names
- **Base URLs** for API endpoints
- **API key patterns** (prefixes, lengths)
- **Default models** for quick setup
- **Model types** supported by each provider

This metadata is used for:

1. **Validation**: Ensuring API keys match expected formats
2. **Discovery**: Probing APIs to discover available models
3. **Configuration**: Providing sensible defaults

## CLI Commands

### List All Providers

```bash
aicred providers
```

### List Providers with Details

```bash
aicred providers --verbose
```

### Show Provider Metadata

Provider metadata is accessible through the core library:

```rust
use aicred_core::providers::{ProviderRegistry, ProviderMetadata};

let mut registry = ProviderRegistry::new();
registry.register_builtin()?;

let metadata = registry.get_metadata("openai")?;
println!("Provider: {}", metadata.name);
println!("Base URL: {}", metadata.base_url);
println!("Default models: {:?}", metadata.default_models);
```

## Adding a New Provider

To add a new provider:

1. Create a new module in `core/src/providers/{provider}.rs`
2. Implement the `ProviderPlugin` trait
3. Add the module to `core/src/providers/mod.rs`
4. Add metadata to `core/src/providers/registry.rs`
5. Add documentation to this file

See `docs/plugin-development.md` for detailed instructions.
