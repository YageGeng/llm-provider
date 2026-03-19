//! Azure OpenAI API client and Rig integration
//!
//! # Example
//! ```
//! use llm_provider::providers::azure;
//! use llm_provider::prelude::CompletionClient;
//!
//! let client = azure::Client::builder()
//!     .api_key("test")
//!     .azure_endpoint("test".to_string()) // add your endpoint here!
//!     .build()
//!     .expect("Failed to build Azure client");
//!
//! let gpt4o = client.completion_model(azure::GPT_4O);
//! ```
//!
//! ## Authentication
//! The authentication type used for the `azure` module is [`AzureOpenAIAuth`].
//!
//! By default, using a type that implements `Into<String>` as the input for the client builder will turn the type into a bearer auth token.
//! If you want to use an API key, you need to use the type specifically.

use std::fmt::Debug;

use super::openai::send_compatible_streaming_request;
use crate::client::{
    self, ApiKey, Capabilities, Capable, DebugExt, Nothing, Provider, ProviderBuilder,
    ProviderClient,
};
use crate::http_client::{self, HttpClientExt, bearer_auth_header};
use crate::streaming::StreamingCompletionResponse;
use crate::{
    completion::{self, CompletionError, CompletionRequest},
    json_utils,
    providers::openai,
    telemetry::SpanCombinator,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::{Level, enabled, info_span};
use tracing_futures::Instrument;
// ================================================================
// Main Azure OpenAI Client
// ================================================================

const DEFAULT_API_VERSION: &str = "2024-10-21";

#[derive(Debug, Clone)]
pub struct AzureExt {
    endpoint: String,
    api_version: String,
}

impl DebugExt for AzureExt {
    fn fields(&self) -> impl Iterator<Item = (&'static str, &dyn std::fmt::Debug)> {
        [
            ("endpoint", (&self.endpoint as &dyn Debug)),
            ("api_version", (&self.api_version as &dyn Debug)),
        ]
        .into_iter()
    }
}

// TODO: @FayCarsons - this should be a type-safe builder,
// but that would require extending the `ProviderBuilder`
// to have some notion of complete vs incomplete states in a
// given extension builder
#[derive(Debug, Clone)]
pub struct AzureExtBuilder {
    endpoint: Option<String>,
    api_version: String,
}

impl Default for AzureExtBuilder {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_version: DEFAULT_API_VERSION.into(),
        }
    }
}

pub type Client<H = reqwest::Client> = client::Client<AzureExt, H>;
pub type ClientBuilder<H = reqwest::Client> =
    client::ClientBuilder<AzureExtBuilder, AzureOpenAIAuth, H>;

impl Provider for AzureExt {
    type Builder = AzureExtBuilder;

    /// Verifying Azure auth without consuming tokens is not supported
    const VERIFY_PATH: &'static str = "";
}

impl<H> Capabilities<H> for AzureExt {
    type Completion = Capable<CompletionModel<H>>;
    type ModelListing = Nothing;
}

impl ProviderBuilder for AzureExtBuilder {
    type Extension<H>
        = AzureExt
    where
        H: HttpClientExt;
    type ApiKey = AzureOpenAIAuth;

    const BASE_URL: &'static str = "";

    fn build<H>(
        builder: &client::ClientBuilder<Self, Self::ApiKey, H>,
    ) -> http_client::Result<Self::Extension<H>>
    where
        H: HttpClientExt,
    {
        let AzureExtBuilder {
            endpoint,
            api_version,
            ..
        } = builder.ext().clone();

        match endpoint {
            Some(endpoint) => Ok(AzureExt {
                endpoint,
                api_version,
            }),
            None => Err(http_client::Error::Instance(
                "Azure client must be provided an endpoint prior to building".into(),
            )),
        }
    }

    fn finish<H>(
        &self,
        mut builder: client::ClientBuilder<Self, Self::ApiKey, H>,
    ) -> http_client::Result<client::ClientBuilder<Self, Self::ApiKey, H>> {
        use AzureOpenAIAuth::*;

        let auth = builder.get_api_key().clone();

        match auth {
            Token(token) => bearer_auth_header(builder.headers_mut(), token.as_str())?,
            ApiKey(key) => {
                let k = http::HeaderName::from_static("api-key");
                let v = http::HeaderValue::from_str(key.as_str())?;

                builder.headers_mut().insert(k, v);
            }
        }

        Ok(builder)
    }
}

impl<H> ClientBuilder<H> {
    /// API version to use (e.g., "2024-10-21" for GA, "2024-10-01-preview" for preview)
    pub fn api_version(mut self, api_version: &str) -> Self {
        self.ext_mut().api_version = api_version.into();

        self
    }
}

impl<H> client::ClientBuilder<AzureExtBuilder, AzureOpenAIAuth, H> {
    /// Azure OpenAI endpoint URL, for example: https://{your-resource-name}.openai.azure.com
    pub fn azure_endpoint(self, endpoint: String) -> ClientBuilder<H> {
        self.over_ext(|AzureExtBuilder { api_version, .. }| AzureExtBuilder {
            endpoint: Some(endpoint),
            api_version,
        })
    }
}

/// The authentication type for Azure OpenAI. Can either be an API key or a token.
/// String types will automatically be coerced to a bearer auth token by default.
#[derive(Clone)]
pub enum AzureOpenAIAuth {
    ApiKey(String),
    Token(String),
}

impl ApiKey for AzureOpenAIAuth {}

impl std::fmt::Debug for AzureOpenAIAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => write!(f, "API key <REDACTED>"),
            Self::Token(_) => write!(f, "Token <REDACTED>"),
        }
    }
}

impl<S> From<S> for AzureOpenAIAuth
where
    S: Into<String>,
{
    fn from(token: S) -> Self {
        AzureOpenAIAuth::Token(token.into())
    }
}

impl<T> Client<T>
where
    T: HttpClientExt,
{
    fn endpoint(&self) -> &str {
        &self.ext().endpoint
    }

    fn api_version(&self) -> &str {
        &self.ext().api_version
    }

    fn post_chat_completion(
        &self,
        deployment_id: &str,
    ) -> http_client::Result<http_client::Builder> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint(),
            deployment_id.trim_start_matches('/'),
            self.api_version()
        );

        self.post(&url)
    }
}

pub struct AzureOpenAIClientParams {
    api_key: String,
    version: String,
    header: String,
}

impl ProviderClient for Client {
    type Input = AzureOpenAIClientParams;

    /// Create a new Azure OpenAI client from the `AZURE_API_KEY` or `AZURE_TOKEN`, `AZURE_API_VERSION`, and `AZURE_ENDPOINT` environment variables.
    fn from_env() -> Self {
        let auth = if let Ok(api_key) = std::env::var("AZURE_API_KEY") {
            AzureOpenAIAuth::ApiKey(api_key)
        } else if let Ok(token) = std::env::var("AZURE_TOKEN") {
            AzureOpenAIAuth::Token(token)
        } else {
            panic!("Neither AZURE_API_KEY nor AZURE_TOKEN is set");
        };

        let api_version = std::env::var("AZURE_API_VERSION").expect("AZURE_API_VERSION not set");
        let azure_endpoint = std::env::var("AZURE_ENDPOINT").expect("AZURE_ENDPOINT not set");

        Self::builder()
            .api_key(auth)
            .azure_endpoint(azure_endpoint)
            .api_version(&api_version)
            .build()
            .unwrap()
    }

    fn from_val(
        AzureOpenAIClientParams {
            api_key,
            version,
            header,
        }: Self::Input,
    ) -> Self {
        let auth = AzureOpenAIAuth::ApiKey(api_key.to_string());

        Self::builder()
            .api_key(auth)
            .azure_endpoint(header)
            .api_version(&version)
            .build()
            .unwrap()
    }
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

// ================================================================
// Azure OpenAI Completion API
// ================================================================

/// `o1` completion model
pub const O1: &str = "o1";
/// `o1-preview` completion model
pub const O1_PREVIEW: &str = "o1-preview";
/// `o1-mini` completion model
pub const O1_MINI: &str = "o1-mini";
/// `gpt-4o` completion model
pub const GPT_4O: &str = "gpt-4o";
/// `gpt-4o-mini` completion model
pub const GPT_4O_MINI: &str = "gpt-4o-mini";
/// `gpt-4o-realtime-preview` completion model
pub const GPT_4O_REALTIME_PREVIEW: &str = "gpt-4o-realtime-preview";
/// `gpt-4-turbo` completion model
pub const GPT_4_TURBO: &str = "gpt-4";
/// `gpt-4` completion model
pub const GPT_4: &str = "gpt-4";
/// `gpt-4-32k` completion model
pub const GPT_4_32K: &str = "gpt-4-32k";
/// `gpt-4-32k` completion model
pub const GPT_4_32K_0613: &str = "gpt-4-32k";
/// `gpt-3.5-turbo` completion model
pub const GPT_35_TURBO: &str = "gpt-3.5-turbo";
/// `gpt-3.5-turbo-instruct` completion model
pub const GPT_35_TURBO_INSTRUCT: &str = "gpt-3.5-turbo-instruct";
/// `gpt-3.5-turbo-16k` completion model
pub const GPT_35_TURBO_16K: &str = "gpt-3.5-turbo-16k";

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AzureOpenAICompletionRequest {
    model: String,
    pub messages: Vec<openai::Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<openai::ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<crate::providers::openai::ToolChoice>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub additional_params: Option<serde_json::Value>,
}

impl TryFrom<(&str, CompletionRequest)> for AzureOpenAICompletionRequest {
    type Error = CompletionError;

    fn try_from((model, req): (&str, CompletionRequest)) -> Result<Self, Self::Error> {
        let model = req.model.clone().unwrap_or_else(|| model.to_string());
        //FIXME: Must fix!
        if req.tool_choice.is_some() {
            tracing::warn!(
                "Tool choice is currently not supported in Azure OpenAI. This should be fixed by Rig 0.25."
            );
        }

        let mut full_history: Vec<openai::Message> = match &req.preamble {
            Some(preamble) => vec![openai::Message::system(preamble)],
            None => vec![],
        };

        if let Some(docs) = req.normalized_documents() {
            let docs: Vec<openai::Message> = docs.try_into()?;
            full_history.extend(docs);
        }

        let chat_history: Vec<openai::Message> = req
            .chat_history
            .clone()
            .into_iter()
            .map(|message| message.try_into())
            .collect::<Result<Vec<Vec<openai::Message>>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        full_history.extend(chat_history);

        let tool_choice = req
            .tool_choice
            .clone()
            .map(crate::providers::openai::ToolChoice::try_from)
            .transpose()?;

        let additional_params = if let Some(schema) = req.output_schema {
            let name = schema
                .as_object()
                .and_then(|o| o.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("response_schema")
                .to_string();
            let mut schema_value = schema.to_value();
            openai::sanitize_schema(&mut schema_value);
            let response_format = serde_json::json!({
                "response_format": {
                    "type": "json_schema",
                    "json_schema": {
                        "name": name,
                        "strict": true,
                        "schema": schema_value
                    }
                }
            });
            Some(match req.additional_params {
                Some(existing) => json_utils::merge(existing, response_format),
                None => response_format,
            })
        } else {
            req.additional_params
        };

        Ok(Self {
            model: model.to_string(),
            messages: full_history,
            temperature: req.temperature,
            tools: req
                .tools
                .clone()
                .into_iter()
                .map(openai::ToolDefinition::from)
                .collect::<Vec<_>>(),
            tool_choice,
            additional_params,
        })
    }
}

#[derive(Clone)]
pub struct CompletionModel<T = reqwest::Client> {
    client: Client<T>,
    /// Name of the model (e.g.: gpt-4o-mini)
    pub model: String,
}

impl<T> CompletionModel<T> {
    pub fn new(client: Client<T>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }
}

impl<T> completion::CompletionModel for CompletionModel<T>
where
    T: HttpClientExt + Clone + Default + std::fmt::Debug + Send + 'static,
{
    type Response = openai::CompletionResponse;
    type StreamingResponse = openai::StreamingCompletionResponse;
    type Client = Client<T>;

    fn make(client: &Self::Client, model: impl Into<String>) -> Self {
        Self::new(client.clone(), model.into())
    }

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<completion::CompletionResponse<openai::CompletionResponse>, CompletionError> {
        let span = if tracing::Span::current().is_disabled() {
            info_span!(
                target: "llm_provider::completions",
                "chat",
                gen_ai.operation.name = "chat",
                gen_ai.provider.name = "azure.openai",
                gen_ai.request.model = self.model,
                gen_ai.system_instructions = &completion_request.preamble,
                gen_ai.response.id = tracing::field::Empty,
                gen_ai.response.model = tracing::field::Empty,
                gen_ai.usage.output_tokens = tracing::field::Empty,
                gen_ai.usage.input_tokens = tracing::field::Empty,
                gen_ai.usage.cached_tokens = tracing::field::Empty,
            )
        } else {
            tracing::Span::current()
        };

        let request =
            AzureOpenAICompletionRequest::try_from((self.model.as_ref(), completion_request))?;

        if enabled!(Level::TRACE) {
            tracing::trace!(target: "llm_provider::completions",
                "Azure OpenAI completion request: {}",
                serde_json::to_string_pretty(&request)?
            );
        }

        let body = serde_json::to_vec(&request)?;

        let req = self
            .client
            .post_chat_completion(&self.model)?
            .body(body)
            .map_err(http_client::Error::from)?;

        async move {
            let response = self.client.send::<_, Bytes>(req).await?;

            let status = response.status();
            let response_body = response.into_body().into_future().await?.to_vec();

            if status.is_success() {
                match serde_json::from_slice::<ApiResponse<openai::CompletionResponse>>(
                    &response_body,
                )? {
                    ApiResponse::Ok(response) => {
                        let span = tracing::Span::current();
                        span.record_response_metadata(&response);
                        span.record_token_usage(&response.usage);
                        if enabled!(Level::TRACE) {
                            tracing::trace!(target: "llm_provider::completions",
                                "Azure OpenAI completion response: {}",
                                serde_json::to_string_pretty(&response)?
                            );
                        }
                        response.try_into()
                    }
                    ApiResponse::Err(err) => Err(CompletionError::ProviderError(err.message)),
                }
            } else {
                Err(CompletionError::ProviderError(
                    String::from_utf8_lossy(&response_body).to_string(),
                ))
            }
        }
        .instrument(span)
        .await
    }

    async fn stream(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError> {
        let preamble = completion_request.preamble.clone();
        let mut request =
            AzureOpenAICompletionRequest::try_from((self.model.as_ref(), completion_request))?;

        let params = json_utils::merge(
            request.additional_params.unwrap_or(serde_json::json!({})),
            serde_json::json!({"stream": true, "stream_options": {"include_usage": true} }),
        );

        request.additional_params = Some(params);

        if enabled!(Level::TRACE) {
            tracing::trace!(target: "llm_provider::completions",
                "Azure OpenAI completion request: {}",
                serde_json::to_string_pretty(&request)?
            );
        }

        let body = serde_json::to_vec(&request)?;

        let req = self
            .client
            .post_chat_completion(&self.model)?
            .body(body)
            .map_err(http_client::Error::from)?;

        let span = if tracing::Span::current().is_disabled() {
            info_span!(
                target: "llm_provider::completions",
                "chat_streaming",
                gen_ai.operation.name = "chat_streaming",
                gen_ai.provider.name = "azure.openai",
                gen_ai.request.model = self.model,
                gen_ai.system_instructions = &preamble,
                gen_ai.response.id = tracing::field::Empty,
                gen_ai.response.model = tracing::field::Empty,
                gen_ai.usage.output_tokens = tracing::field::Empty,
                gen_ai.usage.input_tokens = tracing::field::Empty,
                gen_ai.usage.cached_tokens = tracing::field::Empty,
            )
        } else {
            tracing::Span::current()
        };

        tracing_futures::Instrument::instrument(
            send_compatible_streaming_request(self.client.clone(), req),
            span,
        )
        .await
    }
}
