//! Provider plugins for various AI services and applications.

// Allow clippy lints for the providers module
#![allow(clippy::unused_self)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::float_cmp)]
#![allow(clippy::module_inception)]

pub mod anthropic;
pub mod aws_bedrock;
pub mod azure;
pub mod cohere;
pub mod deepinfra;
pub mod deepseek;
pub mod fireworks;
pub mod google;
pub mod grok;
pub mod groq;
pub mod huggingface;
pub mod litellm;
pub mod metadata;
pub mod mistral;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod perplexity;
pub mod registry;
pub mod replicate;
pub mod together;
pub mod zai;

pub use anthropic::AnthropicPlugin;
pub use aws_bedrock::AwsBedrockPlugin;
pub use azure::AzurePlugin;
pub use cohere::CoherePlugin;
pub use deepinfra::DeepInfraPlugin;
pub use deepseek::DeepSeekPlugin;
pub use fireworks::FireworksPlugin;
pub use google::GooglePlugin;
pub use grok::GrokPlugin;
pub use groq::GroqPlugin;
pub use huggingface::HuggingFacePlugin;
pub use litellm::LiteLLMPlugin;
pub use metadata::{ProviderMetadata, ProviderMetadataRegistry};
pub use mistral::MistralPlugin;
pub use moonshot::MoonshotPlugin;
pub use ollama::OllamaPlugin;
pub use openai::OpenAIPlugin;
pub use openrouter::OpenRouterPlugin;
pub use perplexity::PerplexityPlugin;
pub use registry::ProviderRegistry;
pub use replicate::ReplicatePlugin;
pub use together::TogetherPlugin;
pub use zai::ZAIPlugin;
