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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBootstrapReport {
    pub change_id: String,
    pub created: Vec<FileRecord>,
    pub preserved: Vec<FileRecord>,
}

/// A bounded, provider-neutral reference to evidence used by a task stage.
/// The reference carries identity and location only; source content remains
/// on the host and is loaded by the stage that needs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRef {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    pub fingerprint: String,
    pub source: String,
}

/// A file-level change attributed to one task baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetEntry {
    pub path: String,
    pub kind: ChangeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
    #[serde(default)]
    pub files: Vec<ChangeSetEntry>,
}

/// Stable handoff data between plan, execute, research, compact and review.
/// It intentionally excludes provider-specific messages and private reasoning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPacket {
    pub task_id: String,
    pub role: String,
    pub objective: String,
    #[serde(default)]
    pub acceptance: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_set: Option<ChangeSet>,
    #[serde(default)]
    pub validation: Vec<String>,
    pub budget_remaining_tokens: u64,
}

impl TaskPacket {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.task_id.trim().is_empty()
            || self.task_id.len() > 128
            || self.task_id.chars().any(char::is_control)
            || self.role.trim().is_empty()
            || self.role.len() > 32
            || self.role.chars().any(char::is_control)
            || self.objective.trim().is_empty()
            || self.objective.len() > 8192
            || self.objective.chars().any(char::is_control)
            || self.files.len() > 64
            || self.evidence.len() > 64
            || self.acceptance.len() > 32
            || self.constraints.len() > 32
            || self.validation.len() > 32
        {
            return Err("task packet is empty or exceeds its bounds");
        }
        if self.files.iter().any(|path| {
            path.is_empty() || path.len() > 512 || path.chars().any(char::is_control)
        })
            || self.acceptance.iter().any(|value| bounded_text(value, 2048))
            || self.constraints.iter().any(|value| bounded_text(value, 2048))
            || self.validation.iter().any(|value| bounded_text(value, 2048))
            || self.evidence.iter().any(|reference| {
                reference.path.is_empty()
                    || reference.path.len() > 512
                    || reference.fingerprint.is_empty()
                    || reference.fingerprint.len() > 128
                    || reference.source.is_empty()
                    || reference.source.len() > 64
                    || reference.path.chars().any(char::is_control)
                    || reference.fingerprint.chars().any(char::is_control)
                    || reference.source.chars().any(char::is_control)
                    || reference
                        .start_line
                        .zip(reference.end_line)
                        .is_some_and(|(start, end)| start == 0 || end < start)
            })
        {
            return Err("task packet contains an invalid path or evidence reference");
        }
        if let Some(change_set) = &self.change_set {
            if change_set
                .baseline
                .as_ref()
                .is_some_and(|value| value.is_empty() || value.len() > 128 || value.chars().any(char::is_control))
                || change_set.files.len() > 128
                || change_set.files.iter().any(|entry| {
                    entry.path.is_empty()
                        || entry.path.len() > 512
                        || entry.path.chars().any(char::is_control)
                        || entry
                            .before_fingerprint
                            .as_ref()
                            .is_some_and(|value| value.is_empty() || value.len() > 128 || value.chars().any(char::is_control))
                        || entry
                            .after_fingerprint
                            .as_ref()
                            .is_some_and(|value| value.is_empty() || value.len() > 128 || value.chars().any(char::is_control))
                })
            {
                return Err("task packet change set exceeds its bounds");
            }
        }
        Ok(())
    }
}

fn bounded_text(value: &str, max: usize) -> bool {
    value.is_empty() || value.len() > max || value.chars().any(char::is_control)
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
    #[serde(default)]
    pub codegraph_binding: Option<dowe_codegraph::CodeGraphBinding>,
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
    #[serde(default)]
    pub governance_task: Option<crate::orchestration::TaskRecord>,
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
