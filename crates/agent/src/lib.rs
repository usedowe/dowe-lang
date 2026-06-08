mod client;
mod context;
mod error;
mod images;
mod model;
mod prompts;
mod request;
mod skills;
mod tools;

pub use client::send_agent_request;
pub use context::{AgentCodeGraphNodeSummary, AgentCodeGraphSummary, summarize_codegraph};
pub use error::{AgentError, AgentResult};
pub use images::{encode_image, encode_image_paths};
pub use model::{
    AgentContext, AgentDesktopEvent, AgentDesktopEventKind, AgentImageInput, AgentMessage,
    AgentMessageContent, AgentMessagePart, AgentPrepareOptions, AgentPreparedRequest, AgentRequest,
    AgentRequestMetadata, AgentRequestType, AgentServerResponse, AgentSkillSummary,
    AgentToolDefinition, AgentToolFunction, ImageUrl, MINIMAX_M3, OPENAI_GPT_55,
};
pub use request::{
    default_llm_server_url, infer_language, infer_request_type, prepare_agent_request,
};
pub use skills::generation_skill_summaries;
pub use tools::agent_tool_definitions;

#[cfg(test)]
mod tests;
