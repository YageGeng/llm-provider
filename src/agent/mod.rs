//! This module contains the implementation of the [Agent] struct and its builder.
//!
//! The [Agent] struct represents an LLM agent, which combines an LLM model with a preamble (system prompt),
//! a set of context documents, and a set of tools. Note: both context documents and tools can be either
//! static (i.e.: they are always provided) or dynamic (i.e.: they are RAGged at prompt-time).
//!
//! The [Agent] struct is highly configurable, allowing the user to define anything from
//! a simple bot with a specific system prompt to a complex RAG system with a set of dynamic
//! context documents and tools.
//!
//! The [Agent] struct implements the [crate::completion::Completion] and [crate::completion::Prompt] traits,
//! allowing it to be used for generating completions responses and prompts. The [Agent] struct also
//! implements the [crate::completion::Chat] trait, which allows it to be used for generating chat completions.
//!
//! The [AgentBuilder] implements the builder pattern for creating instances of [Agent].
//! It allows configuring the model, preamble, context documents, tools, temperature, and additional parameters
//! before building the agent.
//!
//! # Example
//! ```rust,ignore
//! use llm_provider::{
//!     completion::{Chat, Completion, Prompt},
//!     providers::openai,
//! };
//!
//! let openai = openai::Client::from_env();
//!
//! // Configure the agent
//! let agent = openai.agent("gpt-4o")
//!     .preamble("System prompt")
//!     .context("Context document 1")
//!     .context("Context document 2")
//!     .tool(tool1)
//!     .tool(tool2)
//!     .temperature(0.8)
//!     .additional_params(json!({"foo": "bar"}))
//!     .build();
//!
//! // Use the agent for completions and prompts
//! // Generate a chat completion response from a prompt and chat history
//! let chat_response = agent.chat("Prompt", chat_history)
//!     .await
//!     .expect("Failed to chat with Agent");
//!
//! // Generate a prompt completion response from a simple prompt
//! let chat_response = agent.prompt("Prompt")
//!     .await
//!     .expect("Failed to prompt the Agent");
//!
//! // Generate a completion request builder from a prompt and chat history. The builder
//! // will contain the agent's configuration (i.e.: preamble, context documents, tools,
//! // model parameters, etc.), but these can be overwritten.
//! let completion_req_builder = agent.completion("Prompt", chat_history)
//!     .await
//!     .expect("Failed to create completion request builder");
//!
//! let response = completion_req_builder
//!     .temperature(0.9) // Overwrite the agent's temperature
//!     .send()
//!     .await
//!     .expect("Failed to send completion request");
//! ```
//!
//! RAG Agent example
//! Embedding- and vector-store-based RAG examples were removed because `llm_provider`
//! no longer includes those APIs.
mod builder;
mod completion;
pub(crate) mod prompt_request;
mod tool;

pub use crate::message::Text;
pub use builder::{AgentBuilder, NoToolConfig, WithBuilderTools, WithToolServerHandle};
pub use completion::Agent;
pub use prompt_request::hooks::{HookAction, PromptHook, ToolCallHookAction};
pub use prompt_request::streaming::{
    FinalResponse, MultiTurnStreamItem, StreamingError, StreamingPromptRequest, StreamingResult,
    stream_to_stdout,
};
pub use prompt_request::{PromptRequest, PromptResponse, TypedPromptRequest, TypedPromptResponse};
