mod error;
mod model;
mod paths;
mod templates;
mod workflow;

pub use error::{HarnessError, HarnessResult};
pub use model::{
    CheckReport, DetectedMode, Diagnostic, DiagnosticSeverity, FileRecord, HarnessManifest,
    HarnessMode, InitOptions, InitReport, PlanOptions, PlanReport, PlanState, StatusReport,
    TddState, ValidationReport,
};
pub use workflow::{
    check_harness, detect_mode, init_project_harness, plan_from_spec, read_manifest,
    read_plan_state, read_status, transition_tdd_state, validate_plan, write_plan_state,
};

#[cfg(test)]
pub(crate) use paths::{WriteMode, write_agent_file};

#[cfg(test)]
mod tests {
    include!("test_fixtures.rs");
    include!("lib_tests.rs");
}
