mod auth;
mod authoring;
mod client;
mod context;
mod error;
mod examples;
mod images;
mod mcp;
mod model;
mod oauth;
mod project;
mod prompts;
mod provider;
mod request;
mod skills;
mod tools;

pub use auth::{AgentAuthStore, AgentCredential, AgentCredentialStatus, default_auth_path};
pub use authoring::{
    PublicSkill, PublicSkillDocument, PublicSkillResourceDocument, get_public_skill,
    get_public_skill_resource, public_skills,
};
pub use client::{send_agent_request, send_native_agent_request};
pub use context::{
    AgentCodeGraphNodeSummary, AgentCodeGraphSummary, summarize_codegraph, summarize_codegraph_for,
};
pub use error::{AgentError, AgentResult};
pub use examples::{PublicExampleResult, PublicExampleSearch, search_public_examples};
pub use images::{encode_image, encode_image_paths};
pub use mcp::handle_mcp_message;
pub use model::{
    AgentContext, AgentDesktopEvent, AgentDesktopEventKind, AgentImageInput, AgentMessage,
    AgentMessageContent, AgentMessagePart, AgentPrepareOptions, AgentPreparedRequest, AgentRequest,
    AgentRequestMetadata, AgentRequestType, AgentServerResponse, AgentSkillSummary,
    AgentToolDefinition, AgentToolFunction, ImageUrl, MINIMAX_M3, OPENAI_GPT_55,
};
pub use oauth::{login_openai_codex, refresh_openai_codex_credential, token_needs_refresh};
pub use project::{AgentHarnessSummary, ProjectContext, project_context};
pub use provider::{
    AgentAuthKind, AgentModelDefinition, AgentProviderDefinition, AgentProviderInfo,
    AgentProviderProtocol, ResolvedProviderAuth, auth_file_has_provider, builtin_provider_ids,
    builtin_provider_info, normalize_model_id, protocol_for_model, provider_base_url,
    provider_default_model, provider_definition, provider_exists, provider_info,
    provider_is_configured, provider_models, resolve_provider_auth,
};
pub use request::{
    default_llm_server_url, infer_language, infer_request_type, prepare_agent_request,
};
pub use skills::{generation_skill_summaries, generation_skill_summaries_for};
pub use tools::agent_tool_definitions;

#[cfg(test)]
mod tests;
