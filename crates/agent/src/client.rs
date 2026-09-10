use crate::error::{AgentError, AgentResult};
use crate::model::{
    AgentMessage, AgentMessageContent, AgentMessagePart, AgentRequest, AgentServerResponse,
    AgentToolDefinition,
};
use crate::oauth::openai_codex_account_id;
use crate::provider::{
    AgentAuthKind, AgentProviderDefinition, AgentProviderProtocol, ResolvedProviderAuth,
    normalize_model_id, protocol_for_model, provider_base_url, provider_definition, ProviderRegistry,
};
use base64::Engine;
use reqwest::StatusCode;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Duration;

include!("client_server_request.rs");

#[path = "client_stream.rs"]
mod streaming;
pub use streaming::NativeRequestEvent;

include!("client_image_generation.rs");
include!("client_request_building.rs");
include!("client_request_bodies.rs");
include!("client_content_and_streaming.rs");
include!("client_stream_aggregation.rs");
include!("client_helpers.rs");
#[cfg(test)]
include!("client_tests.rs");
