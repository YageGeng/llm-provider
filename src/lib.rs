#![cfg_attr(docsrs, feature(doc_cfg))]
//! Rig is a Rust library trimmed from [rig-core](https://github.com/0xPlaygrounds/rig),
//! focused on providing a **unified LLM completion API**.
//!
//! This library removes embedding, vector store, image generation, audio generation,
//! and transcription support from the original rig-core, retaining only the core
//! completion (model request) capabilities.
//!
//! # Table of Contents
//! - [Features](#features)
//! - [Quick Example](#quick-example)
//! - [Core Concepts](#core-concepts)
//! - [Supported Providers](#supported-providers)
//!
//! # Features
//! - Unified completion request interface across different providers
//! - Streaming and non-streaming response support
//! - Built-in Agent abstraction with multi-turn conversation and tool calling
//! - Structured data extraction (extractor)
//! - Full [GenAI Semantic Convention](https://opentelemetry.io/docs/specs/semconv/gen-ai/) telemetry support
//! - WASM compatible
//! - Minimal boilerplate to integrate LLM capabilities
//!
//! # Quick Example
//! ```rust,no_run
//! use llm_provider::{
//!     agent::AgentBuilder,
//!     completion::Prompt,
//!     prelude::{CompletionClient, ProviderClient},
//!     providers::openai,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create OpenAI client
//!     // Requires the OPENAI_API_KEY environment variable to be set
//!     let openai_client = openai::Client::from_env();
//!
//!     let model = openai_client.completion_model("gpt-4o");
//!     let agent = AgentBuilder::new(model).build();
//!
//!     // Prompt the model and print its response
//!     let response = agent
//!         .prompt("Who are you?")
//!         .await
//!         .expect("Failed to prompt GPT-4");
//!
//!     println!("GPT-4: {response}");
//! }
//! ```
//! Note: Using `#[tokio::main]` requires enabling tokio's `macros` and `rt-multi-thread` features
//! (`cargo add tokio --features macros,rt-multi-thread`).
//!
//! # Core Concepts
//!
//! ## Completion Model
//! Rig provides a unified interface for different LLM providers (e.g. OpenAI, Anthropic, Gemini).
//! Each provider has a `Client` struct for initializing completion models.
//! These models implement the [CompletionModel](crate::completion::CompletionModel) trait,
//! providing a common low-level interface for creating and executing completion requests.
//!
//! ## Agent
//! Rig provides high-level abstractions over LLMs via the [Agent](crate::agent::Agent) type.
//!
//! [Agent](crate::agent::Agent) can be used to build anything from simple model calls
//! to multi-turn conversation systems with tool calling support.
//!
//! # Supported Providers
//! Rig includes built-in support for the following LLM providers:
//! - Anthropic
//! - Azure
//! - Cohere
//! - Deepseek
//! - Galadriel
//! - Gemini
//! - Groq
//! - Huggingface
//! - Hyperbolic
//! - Llamafile
//! - Mira
//! - Mistral
//! - Moonshot
//! - Ollama
//! - OpenAI
//! - OpenRouter
//! - Perplexity
//! - Together
//! - xAI
//!
//! You can also implement the [CompletionModel](crate::completion::CompletionModel) trait
//! to integrate your own custom provider.
//!
//! > Thanks to [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig) for the excellent foundation.
//!

extern crate self as llm_provider;

pub mod agent;
pub mod client;
pub mod completion;
pub mod extractor;
pub mod http_client;
pub(crate) mod json_utils;
pub mod model;
pub mod one_or_many;
pub mod prelude;
pub mod providers;
pub mod streaming;
pub mod tool;
pub mod tools;
pub mod wasm_compat;

// Re-export commonly used types and traits
pub use completion::message;
pub use extractor::ExtractionResponse;
pub use one_or_many::{EmptyListError, OneOrMany};

pub mod telemetry;
