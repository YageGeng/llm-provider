/// Example of extracting structured data from unstructured text using OpenAI's Extractor.
///
/// The Extractor leverages LLM tool-calling capabilities to parse natural language
/// into strongly-typed Rust structs via auto-generated JSON Schema.
///
/// Requires the `OPENAI_API_KEY` environment variable.
use llm_provider::client::ProviderClient;
use llm_provider::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Basic personal information to be extracted from text.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Person {
    /// Full name
    name: Option<String>,
    /// Age in years
    age: Option<u8>,
    /// Occupation or job title
    profession: Option<String>,
    /// City of residence
    city: Option<String>,
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

    let extractor = openai_client.extractor::<Person>("gpt-4o").build();

    let text = "John is a 28-year-old software engineer living in Shanghai.";

    println!("Input text: {text}");
    println!("Extracting structured data...\n");

    let person = extractor
        .extract(text)
        .await
        .expect("Failed to extract data");

    println!("Extraction result:");
    println!("  Name:       {:?}", person.name);
    println!("  Age:        {:?}", person.age);
    println!("  Profession: {:?}", person.profession);
    println!("  City:       {:?}", person.city);
}
