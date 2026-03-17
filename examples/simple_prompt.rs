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
