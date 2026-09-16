use super::Schedule;
use serde::{Deserialize, Serialize};

pub type CoordinatorResult<T> = Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Ask,
    Plan,
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Requirements,
    Planning,
    AwaitingApproval,
    Ready,
    Executing,
    Verification,
    Review,
    Deliverable,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub question: String,
    pub blocking: bool,
    pub answer: Option<String>,
}

impl Requirement {
    pub fn blocking(id: &str, question: &str) -> Self {
        Self {
            id: id.into(),
            question: question.into(),
            blocking: true,
            answer: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckEvidence {
    pub criterion: String,
    pub source: String,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSummary {
    pub ready: bool,
    pub applied: bool,
    pub files: Vec<String>,
    pub new_files: Vec<String>,
    pub patch_bytes: u64,
    pub new_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSummary {
    pub task_count: u32,
    pub task_attempts: u32,
    pub retry_count: u32,
    pub session_count: u32,
    pub review_sessions: u32,
    pub charged_tokens: u64,
    pub cost_usd: f64,
    pub cost_unknown: bool,
    pub estimated: bool,
    pub provider_attempts: u64,
    pub budget_exhausted: bool,
}

impl CheckEvidence {
    pub fn passed(criterion: &str, source: &str) -> Self {
        Self {
            criterion: criterion.into(),
            source: source.into(),
            passed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinator {
    pub(super) id: String,
    pub(super) intent: Intent,
    pub(super) objective: String,
    pub(super) phase: Phase,
    pub(super) revision: u64,
    pub(super) requirements: Vec<Requirement>,
    pub(super) schedule: Option<Schedule>,
    pub(super) acceptance: Vec<String>,
    pub(super) approved_revision: Option<u64>,
    pub(super) evidence: Vec<CheckEvidence>,
    pub(super) reviewer: Option<String>,
    #[serde(default)]
    pub(super) integration: Option<IntegrationSummary>,
    #[serde(default)]
    pub(super) summary: Option<WorkflowSummary>,
}

pub(super) fn text(value: &str, max: usize) -> CoordinatorResult<()> {
    if value.trim().is_empty()
        || value.len() > max
        || value.chars().any(|c| c.is_control() && c != '\n')
    {
        Err("text is empty, contains controls or exceeds its limit".into())
    } else {
        Ok(())
    }
}
