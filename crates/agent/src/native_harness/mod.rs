mod capabilities;
mod capability_catalog;
mod catalog;
mod config;
mod continuation;
mod engine;
mod environment;
mod evaluation;
mod extensions;
mod inspection;
mod lock;
mod orchestration;
mod privacy;
mod project_skills;
mod protocol;
mod quality;
mod request;
mod store;
mod task;
mod text_stream;
mod tools;
mod turns;
mod visual_capture;
mod visual_comparison;

pub use crate::ClarificationQuestion;
pub use capabilities::{
    ModelCapabilities, builtin_image_generation_capability, builtin_model_capabilities,
};
pub use capability_catalog::{
    BuiltinCapabilityRecord, CAPABILITY_CATALOG_VERSION, CapabilityEvidence,
    builtin_capability_catalog, builtin_capability_evidence,
};
pub use catalog::{SkillUnit, select_units, skill_index, skill_unit, validate_catalog};
pub use config::{HarnessConfig, HarnessRole, ModelSelection};
pub use dowe_agent_harness::SessionRecord;
pub(crate) use engine::run_harness_turn_without_persistence;
pub use engine::{HarnessHost, HarnessOutcome, compact_harness_session, run_harness_turn};
pub use environment::set_local_environment_value;
pub use evaluation::{EVALUATION_CATEGORIES, EvaluationReport, EvaluationSample, evaluate_samples};
pub use extensions::{
    MAX_EXTENSION_CALLS, MAX_EXTENSION_REQUEST_BYTES, MAX_EXTENSION_RESPONSE_BYTES,
    MAX_EXTENSION_TOOLS, SessionExtensionCall, SessionExtensionRegistry, SessionExtensionResult,
    SessionExtensionToolDescriptor, session_extension_descriptor_fingerprint,
    session_extension_fingerprint, validate_session_extension_descriptors,
};
pub use inspection::{
    EventPage, GovernanceInspection, MAX_EVENT_BYTES, MAX_EVENTS, MAX_PAGE_BYTES, SessionEvent,
    SessionEventLimits, SessionEventReceipt, SessionInspection, SessionObserver,
    SessionObserverHandle, TasksInspection,
};
pub use orchestration::{run_child_turn, run_orchestrated_turn};
pub use privacy::Redactor;
pub use project_skills::{
    ProjectSkill, ProjectSkillSummary, discover_project_skills, load_project_skill,
};
pub(crate) use protocol::apply_harness_turns;
pub use protocol::apply_harness_turns_for_test;
pub use quality::audit_dowe_project;
pub use store::{
    HarnessQueuedTask, HarnessSession, HarnessStore, HarnessTaskState, MemoryObservation,
    MemoryValidity,
};
pub use task::{ChildExecutionRequest, HarnessTask};
pub use text_stream::RedactedTextStream;
pub use tools::{
    Approval, HarnessTerminal, HarnessTools, HarnessWatchers, ShellLease, ShellObserver,
    TerminalInput,
};
pub use turns::{HarnessTurn, ToolCall, ToolResult, response_turn};
pub use visual_capture::validate_screenshot_png;

pub(crate) use crate::capability_map::{
    bootstrap_capability_map, refresh_capability_map, sync_capability_map,
};

pub(crate) fn digest(value: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn identifier() -> String {
    use rand_core::{OsRng, RngCore};
    let mut bytes = [0; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
