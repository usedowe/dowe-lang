mod authoring;
mod client;
mod context;
mod error;
mod examples;
mod images;
mod installation;
mod mcp;
mod model;
mod project;
mod prompts;
mod request;
mod skills;
mod tools;

pub use authoring::{
    PublicSkill, PublicSkillDocument, PublicSkillResourceDocument, get_public_skill,
    get_public_skill_resource, public_skills,
};
pub use client::send_agent_request;
pub use context::{
    AgentCodeGraphNodeSummary, AgentCodeGraphSummary, summarize_codegraph, summarize_codegraph_for,
};
pub use error::{AgentError, AgentResult};
pub use examples::{PublicExampleResult, PublicExampleSearch, search_public_examples};
pub use images::{encode_image, encode_image_paths};
pub use installation::{
    DoweProjectInitReport, init_dowe_project, init_external_agent_project,
    update_external_agent_project,
};
pub use mcp::handle_mcp_message;
pub use model::{
    AgentContext, AgentDesktopEvent, AgentDesktopEventKind, AgentImageInput, AgentMessage,
    AgentMessageContent, AgentMessagePart, AgentPrepareOptions, AgentPreparedRequest, AgentRequest,
    AgentRequestMetadata, AgentRequestType, AgentServerResponse, AgentSkillSummary,
    AgentToolDefinition, AgentToolFunction, ImageUrl, MINIMAX_M3, OPENAI_GPT_55,
};
pub use project::{AgentHarnessSummary, ProjectContext, project_context};
pub use request::{
    default_llm_server_url, infer_language, infer_request_type, prepare_agent_request,
};
pub use skills::{generation_skill_summaries, generation_skill_summaries_for};
pub use tools::agent_tool_definitions;

#[cfg(test)]
mod tests;
