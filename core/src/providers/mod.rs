//! Provider plugins for various AI services and applications.

// Allow clippy lints for the providers module
#![allow(clippy::unused_self)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::float_cmp)]
#![allow(clippy::module_inception)]

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
