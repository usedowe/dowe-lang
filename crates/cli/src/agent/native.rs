use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use dowe_agent::native_harness::{
    Approval, ClarificationQuestion, HarnessConfig, HarnessHost, HarnessOutcome,
    HarnessPermissionMode, HarnessRole, HarnessSession, HarnessStore, ModelSelection,
    ParallelHarnessHost, ParallelWorker, Redactor, SessionRecord, compact_clean_session,
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
    permission_mode: HarnessPermissionMode,
    pub(super) image_paths: Vec<std::path::PathBuf>,
    watchers: dowe_agent::native_harness::HarnessWatchers,
    activity_draft: Option<String>,
}

include!("native_session_lifecycle.rs");
include!("native_session_commands.rs");
include!("native_session_run.rs");
include!("native_session_benchmark.rs");

mod activity;
mod capabilities;
mod lifecycle;
mod provider;
mod recovery;
mod terminal;
mod watchers;

include!("native_terminal_host.rs");
