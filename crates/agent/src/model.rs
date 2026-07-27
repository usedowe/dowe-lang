use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const OPENAI_GPT_55: &str = "openai/gpt-5.5";
pub const MINIMAX_M3: &str = "minimax/minimax-m3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRequestType {
    Clarify,
    SpecPlan,
    VisionUi,
    Implementation,
}

impl AgentRequestType {
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Clarify | Self::Implementation => MINIMAX_M3,
            Self::SpecPlan | Self::VisionUi => OPENAI_GPT_55,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clarify => "clarify",
            Self::SpecPlan => "spec_plan",
            Self::VisionUi => "vision_ui",
            Self::Implementation => "implementation",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "clarify" | "clarification" | "questions" => Some(Self::Clarify),
            "spec_plan" | "plan" | "planning" | "spec" => Some(Self::SpecPlan),
            "vision_ui" | "vision" | "ui_image" | "image" => Some(Self::VisionUi),
            "implementation" | "implement" | "code" => Some(Self::Implementation),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreparedRequest {
    pub request: AgentRequest,
    pub context: AgentContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContext {
    pub language: String,
    pub prompt_words: usize,
    pub needs_reference_image: bool,
    pub skills: Vec<AgentSkillSummary>,
    pub codegraph: Option<crate::context::AgentCodeGraphSummary>,
    pub images: Vec<AgentImageInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRequest {
    pub request_id: String,
    #[serde(rename = "requestType", alias = "request_type")]
    pub request_type: AgentRequestType,
    pub model: String,
    pub messages: Vec<AgentMessage>,
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<AgentToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRequestMetadata {
    pub language: String,
    pub token_policy: String,
    pub skill_count: usize,
    pub image_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentServerResponse {
    pub request_id: String,
    #[serde(rename = "requestType", alias = "request_type")]
    pub request_type: AgentRequestType,
    pub model: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDesktopEvent {
    pub event: AgentDesktopEventKind,
    pub request_id: String,
    #[serde(rename = "requestType")]
    pub request_type: AgentRequestType,
    pub model: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDesktopEventKind {
    RequestPrepared,
    ResponseReceived,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkillSummary {
    pub name: String,
    pub source: String,
    pub path: Option<String>,
    pub description: String,
    pub context: String,
    pub token_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentImageInput {
    pub path: String,
    pub mime_type: String,
    pub data_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessage {
    pub role: String,
    pub content: AgentMessageContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentMessageContent {
    Text(String),
    Parts(Vec<AgentMessagePart>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentMessagePart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: AgentToolFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentPrepareOptions {
    pub request_type: Option<AgentRequestType>,
    pub model: Option<String>,
    pub image_paths: Vec<PathBuf>,
    pub stream: bool,
}
