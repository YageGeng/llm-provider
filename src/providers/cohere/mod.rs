//! Cohere API client and Rig integration
//!
//! # Example
//! ```
//! use llm_provider::{prelude::CompletionClient, providers::cohere};
//!
//! let client = cohere::Client::new("YOUR_API_KEY")
//!     .expect("Failed to create Cohere client");
//!
//! let command_r = client.completion_model("command-r");
//! ```

pub mod client;
pub mod completion;
pub mod streaming;

pub use client::{ApiErrorResponse, ApiResponse, Client};
pub use completion::CompletionModel;
