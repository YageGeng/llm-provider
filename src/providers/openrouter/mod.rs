//! OpenRouter Inference API client and Rig integration
//!
//! # Example
//! ```
//! use llm_provider::{prelude::CompletionClient, providers::openrouter};
//!
//! let client = openrouter::Client::new("YOUR_API_KEY")
//!     .expect("Failed to create OpenRouter client");
//!
//! let claude = client.completion_model(openrouter::CLAUDE_3_7_SONNET);
//! ```

pub mod client;
pub mod completion;
pub mod streaming;

pub use client::*;
pub use completion::*;
