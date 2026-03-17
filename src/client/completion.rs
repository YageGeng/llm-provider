use crate::completion::CompletionModel;

/// A provider client with completion capabilities.
/// Clone is required for conversions between client types.
pub trait CompletionClient {
    /// The type of CompletionModel used by the client.
    type CompletionModel: CompletionModel<Client = Self>;

    /// Create a completion model with the given model.
    ///
    /// # Example with OpenAI
    /// ```
    /// use rig::prelude::*;
    /// use rig::providers::openai::{Client, self};
    ///
    /// // Initialize the OpenAI client
    /// let openai = Client::new("your-open-ai-api-key");
    ///
    /// let gpt4 = openai.completion_model(openai::GPT4);
    /// ```
    fn completion_model(&self, model: impl Into<String>) -> Self::CompletionModel {
        Self::CompletionModel::make(self, model)
    }
}
