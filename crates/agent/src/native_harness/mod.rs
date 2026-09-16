mod benchmarks;
mod browser_actions;
mod capabilities;
mod capability_catalog;
mod catalog;
mod clean_runner;
mod clean_runner_context;
mod clean_runner_execution;
mod clean_runner_read;
mod config;
mod continuation;
mod dispatch;
mod environment;
mod evaluation;
mod extensions;
mod inspection;
mod lock;
mod observability;
mod orchestration;
mod parallel_host;
mod privacy;
mod project_skills;
mod protocol;
mod quality;
mod runner_contract;
mod store;
mod task;
mod text_stream;
mod tools;
mod turns;
mod ui_quality;
mod visual_capture;
mod visual_comparison;
mod workflow;
mod workflow_store;

pub use crate::ClarificationQuestion;
pub use benchmarks::{
    BenchmarkCase, BenchmarkCaseResult, BenchmarkExecution, BenchmarkManifest, BenchmarkQuality,
    BenchmarkReadiness, BenchmarkReport, create_benchmark_scaffold, evaluate_benchmark_quality,
    evaluate_benchmark_quality_at_root, load_benchmark_manifest, persist_benchmark_report,
    report_from_executions, run_benchmark_manifest,
};
pub use browser_actions::{
    BrowserAction, BrowserActionResult, BrowserExecutionReport, execute_browser_actions,
    execute_use_case_over_browser,
};
pub use capabilities::{
    ModelCapabilities, builtin_image_generation_capability, builtin_model_capabilities,
};
pub use capability_catalog::{
    BuiltinCapabilityRecord, CAPABILITY_CATALOG_VERSION, CapabilityEvidence,
    builtin_capability_catalog, builtin_capability_evidence,
};
pub use catalog::{SkillUnit, select_units, skill_index, skill_unit, validate_catalog};
pub use clean_runner::compact_clean_session;
pub(crate) use clean_runner::{run_clean_build_task, run_clean_read_task};
pub use config::{HarnessConfig, HarnessPermissionMode, HarnessRole, ModelSelection};
pub use dispatch::run_agent_task;
pub use dowe_agent_harness::SessionRecord;
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
pub use observability::load_trace as load_workflow_trace;
pub use orchestration::{run_child_turn, run_orchestrated_turn};
pub use parallel_host::{ParallelHarnessHost, ParallelWorker};
pub use privacy::Redactor;
pub use project_skills::{
    ProjectSkill, ProjectSkillSummary, discover_project_skills, load_project_skill,
};
pub(crate) use protocol::apply_harness_turns;
pub use protocol::apply_harness_turns_for_test;
pub use quality::audit_dowe_project;
pub use runner_contract::{HarnessHost, HarnessOutcome};
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
pub(crate) use workflow::run_direct_build_task;
pub use workflow::{
    BackendArchitecturePlan, BackendEntity, BackendHandler, BackendRoute, WorkflowCheck,
    WorkflowContinuation, WorkflowIsolation, WorkflowPlan, WorkflowSourceBinding, WorkflowTask,
    draft_workflow_plan, draft_workflow_plan_with_images, read_workflow_plan, resume_workflow,
    run_workflow,
};
pub use workflow_store::WorkflowCheckpoint;

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
