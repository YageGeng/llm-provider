//! Anthropic API client and Rig integration
//!
//! # Example
//! ```
//! use llm_provider::{
//!     prelude::CompletionClient,
//!     providers::anthropic::{self, completion::CLAUDE_3_5_SONNET},
//! };
//!
//! let client = anthropic::Client::new("YOUR_API_KEY")
//!     .expect("Failed to create Anthropic client");
//!
//! let sonnet = client.completion_model(CLAUDE_3_5_SONNET);
//! ```

pub mod client;
pub mod completion;
pub mod decoders;
pub mod streaming;

pub use client::{Client, ClientBuilder};
