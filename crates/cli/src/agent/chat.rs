use crate::menus::is_interactive_terminal;
use crate::usage::USAGE;
use dialoguer::{
    Input, Password, Select,
    console::{measure_text_width, truncate_str},
    theme::ColorfulTheme,
};
use dowe_agent::{
    AgentAuthStore, AgentConversation, AgentCredential, AgentDesktopEvent, AgentDesktopEventKind,
    AgentPreferencesStore, AgentPrepareOptions, AgentProviderInfo, AgentRequestType,
    AgentUsageTotals, ResolvedProviderAuth, ThinkingLevel, agent_model_details,
    agent_response_text, builtin_provider_info, default_llm_server_url, login_openai_codex,
    login_openrouter, models_with_local_overrides, normalize_model_id, prepare_agent_request,
    provider_default_model, provider_definition, provider_exists, refresh_openai_codex_credential,
    resolve_provider_auth, send_agent_request, send_native_agent_request, token_needs_refresh,
};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
include!("chat_session.rs");
include!("chat_request.rs");
include!("chat_args_and_attachments.rs");
include!("chat_provider_selection.rs");
include!("chat_model_selection.rs");
include!("chat_menu_formatting.rs");
include!("chat_args_tests.rs");
include!("chat_attachment_tests.rs");
include!("chat_input_helpers.rs");
