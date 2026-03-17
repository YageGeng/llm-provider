# llm-provider

A unified LLM provider interface for Rust, trimmed from [rig-core](https://github.com/0xPlaygrounds/rig).

This project focuses on providing a clean, consistent API for calling LLM completion models across multiple providers. Embedding, vector store, image generation, audio generation, and transcription support have been removed — only the core completion (model request) capabilities are retained.

> Thanks to [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig) for the excellent foundation.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Example](#quick-example)
- [Supported Providers](#supported-providers)
- [Acknowledgements](#acknowledgements)

## Features

- Unified completion request interface across different providers
- Streaming and non-streaming response support
- Built-in Agent abstraction with multi-turn conversation and tool calling
- Structured data extraction (extractor)
- Full [GenAI Semantic Convention](https://opentelemetry.io/docs/specs/semconv/gen-ai/) telemetry support
- 17+ model providers under one unified interface
- MCP (Model Context Protocol) tool integration
- Full WASM compatibility
- Minimal boilerplate to integrate LLM capabilities

## Installation

```bash
cargo add llm-provider
```

## Quick Example

```rust
use llm_provider::agent::AgentBuilder;
use llm_provider::client::ProviderClient;
use llm_provider::completion::Prompt;
use llm_provider::prelude::CompletionClient;
use llm_provider::providers::openai;

#[tokio::main]
async fn main() {
    let openai_client = openai::Client::from_env();

    let model = openai_client.completion_model("gpt-4o");

    let agent = AgentBuilder::new(model).build();

    let response = agent
        .prompt("Who are you?")
        .await
        .expect("Failed to prompt GPT-4");

    println!("GPT-4: {response}");
}
```

Note: Using `#[tokio::main]` requires enabling tokio's `macros` and `rt-multi-thread` features (`cargo add tokio --features macros,rt-multi-thread`).

## Supported Providers

Built-in support for the following LLM providers:

- Anthropic
- Azure
- Cohere
- Deepseek
- Galadriel
- Gemini
- Groq
- Huggingface
- Hyperbolic
- Llamafile
- Mira
- Mistral
- Moonshot
- Ollama
- OpenAI
- OpenRouter
- Perplexity
- Together
- xAI

All providers share the unified `CompletionModel` trait interface. You can also implement this trait to integrate your own custom provider.

## Acknowledgements

This project is trimmed from [rig-core](https://github.com/0xPlaygrounds/rig) (developed by [0xPlaygrounds](https://github.com/0xPlaygrounds)). Thanks to the original team for their excellent work building a solid foundation for the Rust LLM ecosystem.
