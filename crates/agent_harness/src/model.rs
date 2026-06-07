use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HarnessMode {
    Dowe,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectedMode {
    Dowe,
    Project,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessManifest {
    pub schema_version: String,
    pub harness_version: String,
    pub dowe_version: String,
    pub mode: HarnessMode,
    pub project_root: String,
    pub agent_root: String,
    pub generated_evidence_root: String,
    pub spec_roots: Vec<String>,
    pub doc_roots: Vec<String>,
    pub source_roots: Vec<String>,
    pub allowed_agent_write_roots: Vec<String>,
    pub disallowed_runtime_roots: Vec<String>,
    pub validation_commands: Vec<ValidationCommand>,
    pub tdd_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCommand {
    pub id: String,
    pub kind: ValidationCommandKind,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCommandKind {
    HarnessCheck,
    CodegraphCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InitOptions {
    pub update_existing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlanOptions {
    pub refresh_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitReport {
    pub created: Vec<FileRecord>,
    pub preserved: Vec<FileRecord>,
    pub blocked: Vec<FileRecord>,
}

impl InitReport {
    pub fn new() -> Self {
        Self {
            created: Vec::new(),
            preserved: Vec::new(),
            blocked: Vec::new(),
        }
    }
}

impl Default for InitReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReport {
    pub plan_id: String,
    pub plan_path: String,
    pub state_path: String,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanState {
    pub plan_id: String,
    pub spec_path: String,
    pub spec_fingerprint: String,
    pub contracts: Vec<ContractState>,
    pub acceptance_criteria: Vec<String>,
    pub test_plan: Vec<String>,
    pub expected_initial_failures: Vec<String>,
    pub expected_failure_justification: Option<String>,
    pub implementation_scope: Vec<String>,
    pub validation_commands: Vec<String>,
    pub documentation_targets: Vec<String>,
    #[serde(default)]
    pub documentation_actions: Vec<String>,
    #[serde(default)]
    pub skill_actions: Vec<String>,
    pub state: TddState,
    pub incomplete_reasons: Vec<String>,
    pub tdd_required: bool,
}

impl PlanState {
    pub fn implementation_allowed(&self) -> bool {
        matches!(
            self.state,
            TddState::ImplementationAllowed
                | TddState::ImplementationDone
                | TddState::TestsPassing
                | TddState::Validated
                | TddState::DocsUpdated
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractState {
    pub path: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TddState {
    SpecSelected,
    ContractsChecked,
    TestsPlanned,
    TestsWritten,
    ExpectedFailureRecorded,
    ImplementationAllowed,
    ImplementationDone,
    TestsPassing,
    Validated,
    DocsUpdated,
    SpecOnly,
    DocumentationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

impl Default for CheckReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub path: String,
    pub message: String,
    pub action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub mode: DetectedMode,
    pub plans: Vec<PlanStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStatus {
    pub plan_id: String,
    pub state: TddState,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub plan_id: String,
    pub success: bool,
    pub evidence_path: String,
    pub commands: Vec<ValidationCommandEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCommandEvidence {
    pub id: String,
    pub kind: ValidationCommandKind,
    pub success: bool,
    pub summary: String,
}
