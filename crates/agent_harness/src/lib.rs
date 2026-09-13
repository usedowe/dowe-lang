mod error;
mod model;
mod orchestration;
mod paths;
mod templates;
mod workflow;

pub use dowe_codegraph::CodeGraphBinding;
pub use error::{HarnessError, HarnessResult};
pub use model::{
    ChangeBootstrapReport, ChangeKind, ChangeSet, ChangeSetEntry, CheckReport, DetectedMode,
    Diagnostic, DiagnosticSeverity, EvidenceRef, FileRecord, HarnessManifest, HarnessMode,
    InitOptions, InitReport, PlanOptions, PlanReport, PlanState, StatusReport, TaskPacket,
    TddState, ValidationReport,
};
pub use orchestration::{
    AcknowledgementState, AgentExecutionKind, AgentRecord, AgentRole, AgentState,
    AllowedEditSurface, DeliveryState, NativeReceiptEvent, OrchestrationError, OrchestrationResult,
    Orchestrator, Receipt, ReceiptPath, ReceiptState, ReviewOutcome, ReviewState, SessionRecord,
    SessionState, TaskRecord, TaskState, ValidationState, WorkerRecord, WorkerRole, WorkerState,
    project_native_receipt_event,
};
pub use workflow::{
    bootstrap_sdd_change, check_harness, detect_mode, init_project_harness, plan_from_spec,
    read_manifest, read_plan_state, read_status, transition_tdd_state, validate_plan,
    write_plan_state,
};

#[cfg(test)]
pub(crate) use paths::{WriteMode, write_agent_file};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    include!("test_fixtures.rs");
    include!("lib_tests.rs");
}
