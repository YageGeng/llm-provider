//! Client implementations for the LLM providers supported by Rig.
//!
//! Each provider has its own submodule with a `Client` for initializing completion models.
//! All providers share the unified [CompletionModel](crate::completion::CompletionModel) trait.
//!
//! # Example
//! ```
//! use rig::{providers::openai, agent::AgentBuilder};
//!
//! let openai = openai::Client::new("your-openai-api-key");
//!
//! let gpt_4o = openai.completion_model("gpt-4o");
//!
//! let agent = AgentBuilder::new(gpt_4o)
//!     .preamble("You are a helpful assistant.")
//!     .build();
//! ```
pub mod anthropic;
pub mod azure;
pub mod cohere;
pub mod deepseek;
pub mod galadriel;
pub mod gemini;
pub mod groq;
pub mod huggingface;
pub mod hyperbolic;
pub mod llamafile;
pub mod mira;
pub mod mistral;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod perplexity;
pub mod together;
pub mod xai;
