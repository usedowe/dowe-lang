mod capabilities;
mod capability_catalog;
mod catalog;
mod config;
mod continuation;
mod engine;
mod environment;
mod evaluation;
mod lock;
mod privacy;
mod protocol;
mod request;
mod store;
mod task;
mod text_stream;
    mod visual_capture;
mod tools;
mod turns;

pub use capabilities::{ModelCapabilities, builtin_image_generation_capability, builtin_model_capabilities};
    pub use crate::ClarificationQuestion;
pub use capability_catalog::{
    BuiltinCapabilityRecord, CAPABILITY_CATALOG_VERSION, CapabilityEvidence,
    builtin_capability_catalog, builtin_capability_evidence,
};
pub use catalog::{SkillUnit, select_units, skill_index, skill_unit, validate_catalog};
pub use config::{HarnessConfig, HarnessRole, ModelSelection};
pub use engine::{HarnessHost, HarnessOutcome, compact_harness_session, run_harness_turn};
pub use environment::set_local_environment_value;
pub use evaluation::{EVALUATION_CATEGORIES, EvaluationReport, EvaluationSample, evaluate_samples};
pub use privacy::Redactor;
pub(crate) use protocol::apply_harness_turns;
    pub use protocol::apply_harness_turns_for_test;
pub use store::{HarnessSession, HarnessStore, MemoryObservation, MemoryValidity};
pub use task::HarnessTask;
pub use text_stream::RedactedTextStream;
pub use tools::{
    Approval, HarnessTerminal, HarnessTools, HarnessWatchers, ShellLease, ShellObserver,
    TerminalInput,
};
pub use turns::{HarnessTurn, ToolCall, ToolResult, response_turn};
    pub use visual_capture::validate_screenshot_png;

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
