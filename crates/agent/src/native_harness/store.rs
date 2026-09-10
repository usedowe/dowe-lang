use super::lock::DataLock as AuthFileLock;
mod candidates;
mod processes;
mod recovery;
mod validity;
use super::{HarnessConfig, HarnessTurn, Redactor, SessionRecord, digest, identifier};
use crate::auth::write_private_json;
use crate::{AgentError, AgentResult};
use candidates::now;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
pub use validity::MemoryValidity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessSession {
    pub schema: u32,
    pub id: String,
    pub project: String,
    #[serde(default)]
    pub catalog: String,
    /// Embedded provider/model/capability/skill authority used to validate replay compatibility.
    #[serde(default)]
    pub authority_fingerprint: String,
    pub revision: u64,
    pub turns: Vec<HarnessTurn>,
    pub events: Vec<Value>,
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration: Option<SessionRecord>,
    pub context_start: usize,
    pub interrupted: bool,
}

impl HarnessSession {
    /// Record metadata only after the first provider response is accepted.
    pub(crate) fn set_initial_prompt_metadata(&mut self, prompt: &str) {
        if self.initial_prompt_preview.is_some() {
            return;
        }
        let preview = sanitize_session_text(prompt, 240);
        if preview.is_empty() {
            return;
        }
        self.initial_prompt_preview = Some(preview);
        let title_source = prompt.lines().find(|line| !line.trim().is_empty()).unwrap_or(prompt);
        self.title = Some(sanitize_session_text(title_source, 80));
    }

    pub fn orchestration(&self) -> Option<&SessionRecord> {
        self.orchestration.as_ref()
    }

    pub fn attach_orchestration(&mut self, record: SessionRecord) -> AgentResult<()> {
        self.validate_orchestration_id(&record)?;
        if self.orchestration.is_some() {
            return Err(AgentError::new("orchestration record is already attached"));
        }
        self.orchestration = Some(record);
        Ok(())
    }

    pub fn update_orchestration(&mut self, record: SessionRecord) -> AgentResult<()> {
        self.validate_orchestration_id(&record)?;
        if self.orchestration.is_none() {
            return Err(AgentError::new("orchestration record is not attached"));
        }
        self.orchestration = Some(record);
        Ok(())
    }

    fn validate_orchestration_id(&self, record: &SessionRecord) -> AgentResult<()> {
        if record.id != self.id {
            return Err(AgentError::new(
                "orchestration record belongs to a different native session",
            ));
        }
        Ok(())
    }

    pub fn usage(&self) -> crate::AgentUsageTotals {
        self.usage_since(0)
    }

    pub(crate) fn usage_since(&self, offset: usize) -> crate::AgentUsageTotals {
        let mut totals = crate::AgentUsageTotals::default();
        self.record_usage_since(offset, &mut totals);
        totals
    }

    pub(crate) fn record_usage_since(&self, offset: usize, totals: &mut crate::AgentUsageTotals) {
        let mut requests = std::collections::BTreeSet::new();
        for event in self.events.iter().skip(offset) {
            let attempt = event["event"] == "request_attempt";
            let id = event["requestId"].as_str();
            if !attempt && id.is_some_and(|id| requests.contains(id)) {
                continue;
            }
            if attempt
                || matches!(
                    event["event"].as_str(),
                    Some("response_received" | "context_compacted" | "auxiliary_response")
                )
            {
                if let Some(id) = id {
                    requests.insert(id);
                }
                let usage =
                    serde_json::from_value::<crate::AgentUsage>(event["usage"].clone()).ok();
                totals.record_usage(
                    event["provider"].as_str().unwrap_or_default(),
                    event["model"].as_str().unwrap_or_default(),
                    usage,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessTaskState {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessQueuedTask {
    pub id: String,
    pub session: String,
    pub prompt: String,
    pub state: HarnessTaskState,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarnessTaskQueue {
    schema: u32,
    tasks: Vec<HarnessQueuedTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObservation {
    pub id: String,
    pub project: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub confirmed: bool,
    #[serde(default)]
    pub validity: MemoryValidity,
    pub created: u64,
    #[serde(default)]
    pub updated: u64,
    #[serde(default)]
    pub files: std::collections::BTreeMap<String, String>,
    #[serde(default = "default_memory_kind")]
    pub kind: String,
}

fn default_memory_kind() -> String {
    "decision".into()
}

fn sanitize_session_text(text: &str, max_bytes: usize) -> String {
    let mut output = String::new();
    for character in text.chars() {
        if character.is_control() {
            if character.is_whitespace() {
                if !output.ends_with(' ') {
                    output.push(' ');
                }
            }
        } else {
            output.push(character);
        }
        if output.len() >= max_bytes {
            while output.len() > max_bytes {
                output.pop();
            }
            break;
        }
    }
    output.trim().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Memories {
    schema: u32,
    observations: Vec<MemoryObservation>,
}

#[derive(Clone)]
pub struct HarnessStore {
    base: PathBuf,
    root: PathBuf,
    project: String,
}

include!("store_task_and_config.rs");
include!("store_sessions_and_memory.rs");
include!("store_helpers_and_queue_tests.rs");
