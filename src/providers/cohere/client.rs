use crate::{
    client::{
        self, BearerAuth, Capabilities, Capable, DebugExt, Nothing, Provider, ProviderBuilder,
        ProviderClient,
    },
    http_client::{self, HttpClientExt},
    wasm_compat::*,
};

use super::CompletionModel;
use serde::Deserialize;

// ================================================================
// Main Cohere Client
// ================================================================

#[derive(Debug, Default, Clone, Copy)]
pub struct CohereExt;

#[derive(Debug, Default, Clone, Copy)]
pub struct CohereBuilder;

type CohereApiKey = BearerAuth;

pub type Client<H = reqwest::Client> = client::Client<CohereExt, H>;
pub type ClientBuilder<H = reqwest::Client> = client::ClientBuilder<CohereBuilder, CohereApiKey, H>;

impl Provider for CohereExt {
    type Builder = CohereBuilder;
    const VERIFY_PATH: &'static str = "/models";
}

impl<H> Capabilities<H> for CohereExt {
    type Completion = Capable<CompletionModel<H>>;
    type ModelListing = Nothing;
}

impl DebugExt for CohereExt {}

impl ProviderBuilder for CohereBuilder {
    type Extension<H>
        = CohereExt
    where
        H: HttpClientExt;
    type ApiKey = CohereApiKey;

    const BASE_URL: &'static str = "https://api.cohere.ai";

    fn build<H>(
        _builder: &client::ClientBuilder<Self, Self::ApiKey, H>,
    ) -> http_client::Result<Self::Extension<H>>
    where
        H: HttpClientExt,
    {
        Ok(CohereExt)
    }
}

impl ProviderClient for Client {
    type Input = CohereApiKey;

    fn from_env() -> Self
    where
        Self: Sized,
    {
        let key = std::env::var("COHERE_API_KEY").expect("COHERE_API_KEY not set");
        Self::new(key).unwrap()
    }

    fn from_val(input: Self::Input) -> Self
    where
        Self: Sized,
    {
        Self::new(input).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

impl<T> Client<T> where T: HttpClientExt + Clone + WasmCompatSend + WasmCompatSync + 'static {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_client_initialization() {
        let _client =
            crate::providers::cohere::Client::new("dummy-key").expect("Client::new() failed");
        let _client_from_builder = crate::providers::cohere::Client::builder()
            .api_key("dummy-key")
            .build()
            .expect("Client::builder() failed");
    }
}
