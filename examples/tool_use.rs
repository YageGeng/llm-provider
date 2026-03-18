/// Example of Tool Use with an OpenAI Agent.
///
/// Defines a simple addition calculator tool. The Agent automatically invokes it
/// when answering math questions, then returns the result in natural language.
///
/// Requires the `OPENAI_API_KEY` environment variable.
use llm_provider::agent::AgentBuilder;
use llm_provider::client::ProviderClient;
use llm_provider::completion::{Prompt, ToolDefinition};
use llm_provider::prelude::CompletionClient;
use llm_provider::providers::openai;
use llm_provider::tool::Tool;
use serde::{Deserialize, Serialize};

/// Input arguments for the addition operation.
#[derive(Deserialize)]
struct AddArgs {
    /// The first number to add
    x: f64,
    /// The second number to add
    y: f64,
}

/// Error type for math tool failures.
#[derive(Debug, thiserror::Error)]
#[error("Math error")]
struct MathError;

/// Addition calculator tool that accepts two numbers and returns their sum.
#[derive(Deserialize, Serialize)]
struct Adder;

impl Tool for Adder {
    const NAME: &'static str = "add";

    type Error = MathError;
    type Args = AddArgs;
    type Output = f64;

    /// Returns the JSON Schema definition so the LLM knows when and how to call this tool.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "add".to_string(),
            description: "Add two numbers together and return the sum".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": {
                        "type": "number",
                        "description": "The first number to add"
                    },
                    "y": {
                        "type": "number",
                        "description": "The second number to add"
                    }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(args.x + args.y)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let openai_client = openai::Client::from_env();
    let model = openai_client.completion_model("gpt-4o");

    let agent = AgentBuilder::new(model)
        .preamble(
            "You are a helpful math assistant. Use the provided tools to perform calculations.",
        )
        .tool(Adder)
        .build();

    let question = "What is 1234.5 + 6789.3?";
    println!("Question: {question}");

    let response = agent
        .prompt(question)
        .await
        .expect("Failed to prompt agent");

    println!("Answer: {response}");
}
