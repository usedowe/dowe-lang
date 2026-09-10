use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use dowe_agent::native_harness::{
    Approval, ClarificationQuestion, HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole,
    HarnessSession, HarnessStore, ModelSelection, Redactor, SessionRecord, compact_harness_session,
    run_harness_turn,
};
use dowe_agent::{
    AgentAuthStore, AgentError, AgentRequest, AgentResult, AgentServerResponse, AgentUsageTotals,
};
use dowe_agent_harness::{ReviewOutcome, TaskRecord};
use serde_json::{Value, json};

mod formatting;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(super) struct NativeSession {
    store: HarnessStore,
    session: HarnessSession,
    config: HarnessConfig,
    pub(super) image_paths: Vec<std::path::PathBuf>,
    watchers: dowe_agent::native_harness::HarnessWatchers,
}

include!("native_session_lifecycle.rs");
include!("native_session_commands.rs");
include!("native_session_run.rs");

mod activity;
mod capabilities;
mod lifecycle;
mod provider;
mod recovery;
mod terminal;
mod watchers;

include!("native_terminal_host.rs");
