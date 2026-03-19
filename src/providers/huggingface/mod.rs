//! Create a new completion model with the given name
//!
//! # Example
//! ```
//! use llm_provider::{prelude::CompletionClient, providers::huggingface::{self, completion}};
//!
//! // Initialize the Huggingface client
//! let client = huggingface::Client::new("your-huggingface-api-key")
//!     .expect("Failed to create Hugging Face client");
//!
//! let completion_model = client.completion_model(completion::GEMMA_2);
//! ```

pub mod client;
pub mod completion;
pub mod streaming;

pub use client::{Client, ClientBuilder, SubProvider};
