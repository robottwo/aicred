//! Provider plugins for various AI services and applications.

// Allow clippy lints for the providers module

pub mod anthropic;
pub mod groq;
pub mod huggingface;
pub mod litellm;
pub mod ollama;
pub mod openai;
pub mod openrouter;

pub use anthropic::AnthropicPlugin;
pub use groq::GroqPlugin;
pub use huggingface::HuggingFacePlugin;
pub use litellm::LiteLLMPlugin;
pub use ollama::OllamaPlugin;
pub use openai::OpenAIPlugin;
pub use openrouter::OpenRouterPlugin;
