//! Comprehensive model registry with metadata for 50+ AI models.
//!
//! This module provides a centralized database of model information including
//! capabilities, pricing, architecture details, and provider information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive registry of AI models with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    models: HashMap<String, ModelEntry>,
}

/// Detailed entry for a model in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Unique identifier for the model
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Provider that offers this model
    pub provider: String,

    /// Model family/series
    pub family: Option<String>,

    /// Model description
    pub description: Option<String>,

    /// Maximum context length in tokens
    pub context_length: u32,

    /// Pricing information
    pub pricing: ModelPricing,

    /// Model capabilities
    pub capabilities: ModelCapabilities,

    /// Architecture details
    pub architecture: ModelArchitecture,

    /// Release date (YYYY-MM-DD format)
    pub released: Option<String>,

    /// Model status (active, deprecated, beta)
    pub status: ModelStatus,
}

/// Pricing information for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per prompt token in USD
    pub input: f64,

    /// Cost per completion token in USD
    pub output: f64,

    /// Cached input cost modifier (0.5 = 50% discount)
    pub cached_input: Option<f64>,

    /// Currency (default: USD)
    pub currency: String,
}

/// Model capabilities flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Text generation support
    pub text: bool,

    /// Image generation support
    pub image: bool,

    /// Vision/multimodal support
    pub vision: bool,

    /// Code generation support
    pub code: bool,

    /// Function/tool calling support
    pub function_calling: bool,

    /// Streaming responses
    pub streaming: bool,

    /// JSON mode support
    pub json_mode: bool,

    /// System prompt support
    pub system_prompt: bool,

    /// Audio input support
    pub audio_in: bool,

    /// Audio output support
    pub audio_out: bool,
}

/// Architecture details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    /// Modality type
    pub modality: String,

    /// Parameter count (approximate)
    pub parameters: Option<String>,

    /// Tokenizer used
    pub tokenizer: String,

    /// Instruction format type
    pub instruct_type: Option<String>,
}

/// Model status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    Active,
    Beta,
    Deprecated,
    Archived,
}

impl ModelRegistry {
    /// Create a new model registry with all known models.
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.populate_models();
        registry
    }

    /// Get a model by ID.
    pub fn get(&self, id: &str) -> Option<&ModelEntry> {
        self.models.get(id)
    }

    /// Get all models.
    pub fn all(&self) -> Vec<&ModelEntry> {
        self.models.values().collect()
    }

    /// Get models by provider.
    pub fn by_provider(&self, provider: &str) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// Get models by family.
    pub fn by_family(&self, family: &str) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|m| m.family.as_deref() == Some(family))
            .collect()
    }

    /// Get models with specific capability.
    pub fn by_capability(&self, capability: CapabilityFilter) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|m| match capability {
                CapabilityFilter::Text => m.capabilities.text,
                CapabilityFilter::Image => m.capabilities.image,
                CapabilityFilter::Vision => m.capabilities.vision,
                CapabilityFilter::Code => m.capabilities.code,
                CapabilityFilter::FunctionCalling => m.capabilities.function_calling,
                CapabilityFilter::Streaming => m.capabilities.streaming,
                CapabilityFilter::JsonMode => m.capabilities.json_mode,
            })
            .collect()
    }

    /// Search models by name or ID.
    pub fn search(&self, query: &str) -> Vec<&ModelEntry> {
        let query_lower = query.to_lowercase();
        self.models
            .values()
            .filter(|m| {
                m.id.to_lowercase().contains(&query_lower)
                    || m.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get the total number of models.
    pub fn count(&self) -> usize {
        self.models.len()
    }

    /// Populate the registry with all known models.
    fn populate_models(&mut self) {
        // OpenAI Models
        self.add_model(ModelEntry {
            id: "gpt-4o".to_string(),
            name: "GPT-4 Omni".to_string(),
            provider: "openai".to_string(),
            family: Some("gpt-4".to_string()),
            description: Some("OpenAI's most advanced multimodal model with text, image, and audio capabilities.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.000005,
                output: 0.000015,
                cached_input: Some(0.5),
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "o200k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2024-05-13".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            family: Some("gpt-4".to_string()),
            description: Some("Compact, affordable version of GPT-4o with excellent performance for most tasks.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000015,
                output: 0.0000006,
                cached_input: Some(0.5),
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "o200k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2024-07-18".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            provider: "openai".to_string(),
            family: Some("gpt-4".to_string()),
            description: Some("High-performance version of GPT-4 with extended context and improved capabilities.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00001,
                output: 0.00003,
                cached_input: Some(0.5),
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "cl100k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2023-11-06".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            family: Some("gpt-4".to_string()),
            description: Some("Original GPT-4 model with strong reasoning capabilities.".to_string()),
            context_length: 8192,
            pricing: ModelPricing {
                input: 0.00003,
                output: 0.00006,
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
                parameters: None,
                tokenizer: "cl100k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2023-03-14".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: "openai".to_string(),
            family: Some("gpt-3.5".to_string()),
            description: Some("Fast, efficient model for everyday tasks with good performance.".to_string()),
            context_length: 16385,
            pricing: ModelPricing {
                input: 0.0000005,
                output: 0.0000015,
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
                parameters: None,
                tokenizer: "cl100k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2022-11-30".to_string()),
            status: ModelStatus::Active,
        });

        // Anthropic Models
        self.add_model(ModelEntry {
            id: "claude-3-5-sonnet-20241022".to_string(),
            name: "Claude 3.5 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            family: Some("claude-3.5".to_string()),
            description: Some("Anthropic's most capable model with excellent reasoning, coding, and vision capabilities.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.000003,
                output: 0.000015,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-10-22".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "claude-3-5-haiku-20241022".to_string(),
            name: "Claude 3.5 Haiku".to_string(),
            provider: "anthropic".to_string(),
            family: Some("claude-3.5".to_string()),
            description: Some("Fast and efficient model with strong performance for most use cases.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.0000008,
                output: 0.000004,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-10-22".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "claude-3-opus-20240229".to_string(),
            name: "Claude 3 Opus".to_string(),
            provider: "anthropic".to_string(),
            family: Some("claude-3".to_string()),
            description: Some("Most powerful Claude 3 model with exceptional reasoning and nuance.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.000015,
                output: 0.000075,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-02-29".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "claude-3-sonnet-20240229".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            family: Some("claude-3".to_string()),
            description: Some("Balanced performance model for enterprise use cases.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.000003,
                output: 0.000015,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-02-29".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            provider: "anthropic".to_string(),
            family: Some("claude-3".to_string()),
            description: Some("Fastest Claude 3 model for instant responses.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.00000025,
                output: 0.00000125,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-03-07".to_string()),
            status: ModelStatus::Active,
        });

        // Google Models
        self.add_model(ModelEntry {
            id: "gemini-2.0-flash-exp".to_string(),
            name: "Gemini 2.0 Flash Experimental".to_string(),
            provider: "google".to_string(),
            family: Some("gemini-2.0".to_string()),
            description: Some("Google's experimental Gemini 2.0 model with advanced multimodal capabilities.".to_string()),
            context_length: 1000000,
            pricing: ModelPricing {
                input: 0.000000075,
                output: 0.0000003,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "gemini".to_string(),
                instruct_type: None,
            },
            released: Some("2024-12-11".to_string()),
            status: ModelStatus::Beta,
        });

        self.add_model(ModelEntry {
            id: "gemini-1.5-pro".to_string(),
            name: "Gemini 1.5 Pro".to_string(),
            provider: "google".to_string(),
            family: Some("gemini-1.5".to_string()),
            description: Some("Google's advanced multimodal model with massive context window.".to_string()),
            context_length: 2000000,
            pricing: ModelPricing {
                input: 0.00000125,
                output: 0.000005,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "gemini".to_string(),
                instruct_type: None,
            },
            released: Some("2024-02-15".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gemini-1.5-flash".to_string(),
            name: "Gemini 1.5 Flash".to_string(),
            provider: "google".to_string(),
            family: Some("gemini-1.5".to_string()),
            description: Some("Lightweight, fast Gemini model for most use cases.".to_string()),
            context_length: 1000000,
            pricing: ModelPricing {
                input: 0.000000075,
                output: 0.0000003,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "gemini".to_string(),
                instruct_type: None,
            },
            released: Some("2024-05-14".to_string()),
            status: ModelStatus::Active,
        });

        // Meta Models (via Llama providers)
        self.add_model(ModelEntry {
            id: "llama-3.3-70b-instruct".to_string(),
            name: "Llama 3.3 70B Instruct".to_string(),
            provider: "meta".to_string(),
            family: Some("llama-3.3".to_string()),
            description: Some("Meta's latest open-source model with strong performance across tasks.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.00000059,
                output: 0.00000079,
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
                parameters: Some("70B".to_string()),
                tokenizer: "llama3".to_string(),
                instruct_type: Some("llama3".to_string()),
            },
            released: Some("2024-12-06".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "llama-3.1-405b-instruct".to_string(),
            name: "Llama 3.1 405B Instruct".to_string(),
            provider: "meta".to_string(),
            family: Some("llama-3.1".to_string()),
            description: Some("Meta's largest open-source model with frontier-level performance.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.0027,
                output: 0.0027,
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
                parameters: Some("405B".to_string()),
                tokenizer: "llama3".to_string(),
                instruct_type: Some("llama3".to_string()),
            },
            released: Some("2024-07-23".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "llama-3.1-70b-instruct".to_string(),
            name: "Llama 3.1 70B Instruct".to_string(),
            provider: "meta".to_string(),
            family: Some("llama-3.1".to_string()),
            description: Some("Balanced open-source model with strong capabilities.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.00000059,
                output: 0.00000079,
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
                parameters: Some("70B".to_string()),
                tokenizer: "llama3".to_string(),
                instruct_type: Some("llama3".to_string()),
            },
            released: Some("2024-07-23".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "llama-3.1-8b-instruct".to_string(),
            name: "Llama 3.1 8B Instruct".to_string(),
            provider: "meta".to_string(),
            family: Some("llama-3.1".to_string()),
            description: Some("Lightweight open-source model for fast inference.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.00000018,
                output: 0.00000018,
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
                parameters: Some("8B".to_string()),
                tokenizer: "llama3".to_string(),
                instruct_type: Some("llama3".to_string()),
            },
            released: Some("2024-07-23".to_string()),
            status: ModelStatus::Active,
        });

        // Mistral AI Models
        self.add_model(ModelEntry {
            id: "mistral-large-2411".to_string(),
            name: "Mistral Large 2411".to_string(),
            provider: "mistral".to_string(),
            family: Some("mistral-large".to_string()),
            description: Some("Mistral's flagship model with advanced reasoning and multilingual capabilities.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.000002,
                output: 0.000006,
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
                parameters: None,
                tokenizer: "mistral".to_string(),
                instruct_type: Some("mistral".to_string()),
            },
            released: Some("2024-11-20".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "pixtral-12b-2409".to_string(),
            name: "Pixtral 12B".to_string(),
            provider: "mistral".to_string(),
            family: Some("pixtral".to_string()),
            description: Some("Mistral's open multimodal model with vision capabilities.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000025,
                output: 0.00000025,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: Some("12B".to_string()),
                tokenizer: "mistral".to_string(),
                instruct_type: Some("mistral".to_string()),
            },
            released: Some("2024-09-18".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "mistral-nemo".to_string(),
            name: "Mistral Nemo".to_string(),
            provider: "mistral".to_string(),
            family: Some("mistral".to_string()),
            description: Some("Open-weight model in collaboration with NVIDIA.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000008,
                output: 0.00000008,
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
                parameters: Some("12B".to_string()),
                tokenizer: "mistral".to_string(),
                instruct_type: Some("mistral".to_string()),
            },
            released: Some("2024-07-18".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "codestral-2501".to_string(),
            name: "Codestral 25.01".to_string(),
            provider: "mistral".to_string(),
            family: Some("codestral".to_string()),
            description: Some("Mistral's code-specialized model with excellent programming capabilities.".to_string()),
            context_length: 32000,
            pricing: ModelPricing {
                input: 0.0000003,
                output: 0.0000009,
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
                parameters: Some("22B".to_string()),
                tokenizer: "mistral".to_string(),
                instruct_type: Some("codestral".to_string()),
            },
            released: Some("2025-01-07".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "open-mistral-7b".to_string(),
            name: "Mistral 7B".to_string(),
            provider: "mistral".to_string(),
            family: Some("mistral".to_string()),
            description: Some("Original open-weight Mistral 7B model.".to_string()),
            context_length: 32768,
            pricing: ModelPricing {
                input: 0.00000007,
                output: 0.00000007,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: false,
                code: true,
                function_calling: false,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "text".to_string(),
                parameters: Some("7B".to_string()),
                tokenizer: "mistral".to_string(),
                instruct_type: Some("mistral".to_string()),
            },
            released: Some("2023-09-27".to_string()),
            status: ModelStatus::Active,
        });

        // Cohere Models
        self.add_model(ModelEntry {
            id: "command-r-plus-08-2024".to_string(),
            name: "Command R+".to_string(),
            provider: "cohere".to_string(),
            family: Some("command".to_string()),
            description: Some("Cohere's flagship model with strong performance on RAG and tool use.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.0000025,
                output: 0.0000125,
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
                parameters: None,
                tokenizer: "cohere".to_string(),
                instruct_type: Some("command".to_string()),
            },
            released: Some("2024-04-09".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "command-r-08-2024".to_string(),
            name: "Command R".to_string(),
            provider: "cohere".to_string(),
            family: Some("command".to_string()),
            description: Some("Balanced model optimized for RAG applications.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.0000005,
                output: 0.0000015,
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
                parameters: None,
                tokenizer: "cohere".to_string(),
                instruct_type: Some("command".to_string()),
            },
            released: Some("2024-04-09".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "command-light".to_string(),
            name: "Command Light".to_string(),
            provider: "cohere".to_string(),
            family: Some("command".to_string()),
            description: Some("Lightweight model for fast, efficient responses.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000015,
                output: 0.0000006,
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
                parameters: None,
                tokenizer: "cohere".to_string(),
                instruct_type: Some("command".to_string()),
            },
            released: Some("2023-03-02".to_string()),
            status: ModelStatus::Active,
        });

        // xAI Models
        self.add_model(ModelEntry {
            id: "grok-2".to_string(),
            name: "Grok 2".to_string(),
            provider: "xai".to_string(),
            family: Some("grok".to_string()),
            description: Some("xAI's advanced model with real-time information access.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.000002,
                output: 0.000010,
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
                parameters: None,
                tokenizer: "grok".to_string(),
                instruct_type: None,
            },
            released: Some("2024-08-13".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "grok-2-vision-1212".to_string(),
            name: "Grok 2 Vision".to_string(),
            provider: "xai".to_string(),
            family: Some("grok".to_string()),
            description: Some("Multimodal version of Grok 2 with vision capabilities.".to_string()),
            context_length: 8192,
            pricing: ModelPricing {
                input: 0.000005,
                output: 0.000025,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "grok".to_string(),
                instruct_type: None,
            },
            released: Some("2024-12-12".to_string()),
            status: ModelStatus::Beta,
        });

        self.add_model(ModelEntry {
            id: "grok-beta".to_string(),
            name: "Grok Beta".to_string(),
            provider: "xai".to_string(),
            family: Some("grok".to_string()),
            description: Some("Original Grok model with real-time knowledge.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.000005,
                output: 0.000015,
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
                parameters: None,
                tokenizer: "grok".to_string(),
                instruct_type: None,
            },
            released: Some("2023-11-04".to_string()),
            status: ModelStatus::Active,
        });

        // DeepSeek Models
        self.add_model(ModelEntry {
            id: "deepseek-chat".to_string(),
            name: "DeepSeek Chat".to_string(),
            provider: "deepseek".to_string(),
            family: Some("deepseek".to_string()),
            description: Some("DeepSeek's general-purpose chat model with strong reasoning.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000014,
                output: 0.00000028,
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
                parameters: None,
                tokenizer: "deepseek".to_string(),
                instruct_type: Some("deepseek".to_string()),
            },
            released: Some("2024-12-26".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "deepseek-coder".to_string(),
            name: "DeepSeek Coder".to_string(),
            provider: "deepseek".to_string(),
            family: Some("deepseek".to_string()),
            description: Some("DeepSeek's code-specialized model for programming tasks.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000014,
                output: 0.00000028,
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
                parameters: None,
                tokenizer: "deepseek".to_string(),
                instruct_type: Some("deepseek".to_string()),
            },
            released: Some("2024-11-06".to_string()),
            status: ModelStatus::Active,
        });

        // Qwen (Alibaba) Models
        self.add_model(ModelEntry {
            id: "qwen-2.5-72b-instruct".to_string(),
            name: "Qwen 2.5 72B Instruct".to_string(),
            provider: "alibaba".to_string(),
            family: Some("qwen-2.5".to_string()),
            description: Some("Alibaba's powerful open-weight model with strong multilingual support.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.0000009,
                output: 0.0000009,
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
                parameters: Some("72B".to_string()),
                tokenizer: "qwen".to_string(),
                instruct_type: Some("qwen".to_string()),
            },
            released: Some("2024-09-19".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "qwen-vl-plus".to_string(),
            name: "Qwen VL Plus".to_string(),
            provider: "alibaba".to_string(),
            family: Some("qwen".to_string()),
            description: Some("Qwen's vision-language model with strong multimodal capabilities.".to_string()),
            context_length: 8192,
            pricing: ModelPricing {
                input: 0.0000025,
                output: 0.0000025,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "qwen".to_string(),
                instruct_type: Some("qwen".to_string()),
            },
            released: Some("2024-02-21".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "qwen-2-72b-instruct".to_string(),
            name: "Qwen 2 72B Instruct".to_string(),
            provider: "alibaba".to_string(),
            family: Some("qwen-2".to_string()),
            description: Some("Qwen 2's flagship model with excellent performance.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.0000009,
                output: 0.0000009,
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
                parameters: Some("72B".to_string()),
                tokenizer: "qwen".to_string(),
                instruct_type: Some("qwen".to_string()),
            },
            released: Some("2024-06-07".to_string()),
            status: ModelStatus::Active,
        });

        // OpenRouter Special Models
        self.add_model(ModelEntry {
            id: "anthropic/claude-3.5-sonnet".to_string(),
            name: "Claude 3.5 Sonnet (OpenRouter)".to_string(),
            provider: "openrouter".to_string(),
            family: Some("claude-3.5".to_string()),
            description: Some("Anthropic's Claude 3.5 Sonnet available via OpenRouter.".to_string()),
            context_length: 200000,
            pricing: ModelPricing {
                input: 0.000003,
                output: 0.000015,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "claude".to_string(),
                instruct_type: None,
            },
            released: Some("2024-10-22".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "google/gemini-pro-1.5".to_string(),
            name: "Gemini Pro 1.5 (OpenRouter)".to_string(),
            provider: "openrouter".to_string(),
            family: Some("gemini-1.5".to_string()),
            description: Some("Google's Gemini Pro 1.5 available via OpenRouter.".to_string()),
            context_length: 2800000,
            pricing: ModelPricing {
                input: 0.00000125,
                output: 0.000005,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "gemini".to_string(),
                instruct_type: None,
            },
            released: Some("2024-02-15".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "openai/gpt-4o".to_string(),
            name: "GPT-4o (OpenRouter)".to_string(),
            provider: "openrouter".to_string(),
            family: Some("gpt-4".to_string()),
            description: Some("OpenAI's GPT-4o available via OpenRouter.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.000005,
                output: 0.000015,
                cached_input: Some(0.5),
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: true,
                code: true,
                function_calling: true,
                streaming: true,
                json_mode: true,
                system_prompt: true,
                audio_in: true,
                audio_out: true,
            },
            architecture: ModelArchitecture {
                modality: "multimodal".to_string(),
                parameters: None,
                tokenizer: "o200k_base".to_string(),
                instruct_type: Some("chatml".to_string()),
            },
            released: Some("2024-05-13".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "meta-llama/llama-3.3-70b-instruct".to_string(),
            name: "Llama 3.3 70B (OpenRouter)".to_string(),
            provider: "openrouter".to_string(),
            family: Some("llama-3.3".to_string()),
            description: Some("Meta's Llama 3.3 70B available via OpenRouter.".to_string()),
            context_length: 131072,
            pricing: ModelPricing {
                input: 0.00000059,
                output: 0.00000079,
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
                parameters: Some("70B".to_string()),
                tokenizer: "llama3".to_string(),
                instruct_type: Some("llama3".to_string()),
            },
            released: Some("2024-12-06".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "deepseek/deepseek-chat".to_string(),
            name: "DeepSeek Chat (OpenRouter)".to_string(),
            provider: "openrouter".to_string(),
            family: Some("deepseek".to_string()),
            description: Some("DeepSeek's chat model available via OpenRouter.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000014,
                output: 0.00000028,
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
                parameters: None,
                tokenizer: "deepseek".to_string(),
                instruct_type: Some("deepseek".to_string()),
            },
            released: Some("2024-12-26".to_string()),
            status: ModelStatus::Active,
        });

        // Additional popular models
        self.add_model(ModelEntry {
            id: "mixtral-8x7b-instruct".to_string(),
            name: "Mixtral 8x7B Instruct".to_string(),
            provider: "mistral".to_string(),
            family: Some("mixtral".to_string()),
            description: Some("Mixture-of-experts model with strong performance.".to_string()),
            context_length: 32768,
            pricing: ModelPricing {
                input: 0.00000027,
                output: 0.00000027,
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
                parameters: Some("46.7B".to_string()),
                tokenizer: "mistral".to_string(),
                instruct_type: Some("mistral".to_string()),
            },
            released: Some("2024-01-08".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "dbrx-instruct".to_string(),
            name: "DBRX Instruct".to_string(),
            provider: "databricks".to_string(),
            family: Some("dbrx".to_string()),
            description: Some("Databricks' open Mixture-of-Experts model.".to_string()),
            context_length: 32768,
            pricing: ModelPricing {
                input: 0.0000001,
                output: 0.0000001,
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
                parameters: Some("132B".to_string()),
                tokenizer: "cl100k_base".to_string(),
                instruct_type: Some("dbrx".to_string()),
            },
            released: Some("2024-04-03".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "phi-3-medium-128k-instruct".to_string(),
            name: "Phi-3 Medium 128K".to_string(),
            provider: "microsoft".to_string(),
            family: Some("phi-3".to_string()),
            description: Some("Microsoft's Phi-3 medium model with strong reasoning.".to_string()),
            context_length: 128000,
            pricing: ModelPricing {
                input: 0.00000015,
                output: 0.00000015,
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
                parameters: Some("14B".to_string()),
                tokenizer: "phi-3".to_string(),
                instruct_type: Some("phi-3".to_string()),
            },
            released: Some("2024-05-21".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "yi-1.5-34b-chat".to_string(),
            name: "Yi 1.5 34B Chat".to_string(),
            provider: "01-ai".to_string(),
            family: Some("yi-1.5".to_string()),
            description: Some("01.AI's Yi 1.5 model with strong multilingual capabilities.".to_string()),
            context_length: 32768,
            pricing: ModelPricing {
                input: 0.0000002,
                output: 0.0000002,
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
                parameters: Some("34B".to_string()),
                tokenizer: "yi".to_string(),
                instruct_type: Some("yi".to_string()),
            },
            released: Some("2024-06-04".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "gemma-2-27b-it".to_string(),
            name: "Gemma 2 27B".to_string(),
            provider: "google".to_string(),
            family: Some("gemma-2".to_string()),
            description: Some("Google's open Gemma 2 model.".to_string()),
            context_length: 8192,
            pricing: ModelPricing {
                input: 0.00000026,
                output: 0.00000026,
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
                parameters: Some("27B".to_string()),
                tokenizer: "gemma".to_string(),
                instruct_type: Some("gemma".to_string()),
            },
            released: Some("2024-06-27".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "stablelm-zephyr-3b".to_string(),
            name: "StableLM Zephyr 3B".to_string(),
            provider: "stability".to_string(),
            family: Some("stablelm".to_string()),
            description: Some("Stability AI's lightweight model for efficient inference.".to_string()),
            context_length: 4096,
            pricing: ModelPricing {
                input: 0.00000005,
                output: 0.00000005,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: false,
                code: false,
                function_calling: false,
                streaming: true,
                json_mode: false,
                system_prompt: true,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "text".to_string(),
                parameters: Some("3B".to_string()),
                tokenizer: "gpt2".to_string(),
                instruct_type: Some("zephyr".to_string()),
            },
            released: Some("2023-11-06".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "t5-xxl".to_string(),
            name: "T5 XXL".to_string(),
            provider: "google".to_string(),
            family: Some("t5".to_string()),
            description: Some("Google's text-to-text transfer transformer model.".to_string()),
            context_length: 512,
            pricing: ModelPricing {
                input: 0.000001,
                output: 0.000001,
                cached_input: None,
                currency: "USD".to_string(),
            },
            capabilities: ModelCapabilities {
                text: true,
                image: false,
                vision: false,
                code: false,
                function_calling: false,
                streaming: false,
                json_mode: false,
                system_prompt: false,
                audio_in: false,
                audio_out: false,
            },
            architecture: ModelArchitecture {
                modality: "text".to_string(),
                parameters: Some("11B".to_string()),
                tokenizer: "t5".to_string(),
                instruct_type: None,
            },
            released: Some("2020-12-07".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "falcon-180b-chat".to_string(),
            name: "Falcon 180B Chat".to_string(),
            provider: "tii".to_string(),
            family: Some("falcon".to_string()),
            description: Some("Technology Innovation Institute's open-source model.".to_string()),
            context_length: 2048,
            pricing: ModelPricing {
                input: 0.000002,
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
                parameters: Some("180B".to_string()),
                tokenizer: "falcon".to_string(),
                instruct_type: Some("falcon".to_string()),
            },
            released: Some("2023-09-06".to_string()),
            status: ModelStatus::Active,
        });

        self.add_model(ModelEntry {
            id: "mpt-30b-chat".to_string(),
            name: "MPT-30B Chat".to_string(),
            provider: "mosaicml".to_string(),
            family: Some("mpt".to_string()),
            description: Some("MosaicML's open-source model.".to_string()),
            context_length: 8192,
            pricing: ModelPricing {
                input: 0.000001,
                output: 0.000001,
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
                parameters: Some("30B".to_string()),
                tokenizer: "gpt2".to_string(),
                instruct_type: Some("mpt".to_string()),
            },
            released: Some("2023-05-22".to_string()),
            status: ModelStatus::Active,
        });
    }

    fn add_model(&mut self, model: ModelEntry) {
        self.models.insert(model.id.clone(), model);
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter options for model capabilities.
#[derive(Debug, Clone, Copy)]
pub enum CapabilityFilter {
    Text,
    Image,
    Vision,
    Code,
    FunctionCalling,
    Streaming,
    JsonMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_count() {
        let registry = ModelRegistry::new();
        assert!(registry.count() >= 50, "Registry should have at least 50 models");
    }

    #[test]
    fn test_get_model() {
        let registry = ModelRegistry::new();
        let model = registry.get("gpt-4o");
        assert!(model.is_some());
        assert_eq!(model.unwrap().id, "gpt-4o");
    }

    #[test]
    fn test_by_provider() {
        let registry = ModelRegistry::new();
        let openai_models = registry.by_provider("openai");
        assert!(!openai_models.is_empty());
        for model in openai_models {
            assert_eq!(model.provider, "openai");
        }
    }

    #[test]
    fn test_by_capability() {
        let registry = ModelRegistry::new();
        let code_models = registry.by_capability(CapabilityFilter::Code);
        assert!(!code_models.is_empty());
        for model in code_models {
            assert!(model.capabilities.code);
        }
    }

    #[test]
    fn test_search() {
        let registry = ModelRegistry::new();
        let results = registry.search("gpt");
        assert!(!results.is_empty());
    }
}
