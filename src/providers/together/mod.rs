//! Together AI API client and Rig integration
//!
//! # Example
//! Embedding examples were removed because `llm_provider` no longer includes embedding support.

pub mod client;
pub mod completion;
pub mod streaming;

pub use client::Client;
pub use completion::*;
