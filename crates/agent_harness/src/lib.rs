pub mod clean;
mod contracts;
pub mod coordinator;
mod error;
mod integration;
mod invariants;
mod model;
mod orchestration;
mod paths;
mod product;
mod research_core;
mod simulation;
mod templates;
mod verification;
mod workflow;
mod worktree;

pub use contracts::{
    ContractRegistry, ContractRegistryStatus, PersistentContract, load_contract_registry,
};
pub use dowe_codegraph::CodeGraphBinding;
pub use error::{HarnessError, HarnessResult};
pub use integration::{
    IntegrationBlock, IntegrationChange, IntegrationIssue, IntegrationReport, IntegrationTask,
    analyze_integration,
};
pub use invariants::{InvariantReport, InvariantRule, InvariantViolation, check_invariants};
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
pub use product::{
    ProductDiscovery, ProductDiscoveryUseCase, VerifiedProductUpdate, VerifiedProductUseCase,
    record_product_discovery, record_verified_product_update,
};
pub use research_core::{
    AgentStage, AutonomyLevel, EvidenceKind, EvidenceRef as ResearchEvidenceRef, ModelCandidate,
    ModelRoute, ModelRouteInput, ModelSelectionDecision, ObservabilityTrace, RiskLevel,
    StageRecord, aggregate_context_usage, autonomy_allowed, route_model, select_model_candidate,
};
pub use simulation::{
    SimulationStep, StateAssertion, UseCaseAction, UseCaseScenario, UseCaseSimulationReport,
    load_use_case_scenario, persist_use_case_scenario, simulate_use_case,
};
pub use verification::{VerificationGraph, VerificationNode};
pub use workflow::{
    bootstrap_sdd_change, check_harness, detect_mode, init_project_harness, plan_from_spec,
    read_manifest, read_plan_state, read_status, transition_tdd_state, validate_plan,
    write_plan_state,
};
pub use worktree::{
    IntegrationApplyReport, IntegrationFaultPoint, IntegrationRecovery, IsolatedWorktree,
    PreparedIntegration, WorkerResult, apply_isolated_worktrees, capture_worker_result,
    cleanup_isolated_worktrees, create_isolated_worktree, create_worktree_from_results,
    prepare_isolated_worktrees, recover_pending_integration, remove_isolated_worktree,
    validate_isolated_worktree,
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
