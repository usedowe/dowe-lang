use super::{Approval, HarnessTerminal, ModelSelection};
use crate::{AgentError, AgentRequest, AgentResult, AgentServerResponse, GeneratedImage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

pub trait HarnessHost {
    fn parallel_worker(&mut self) -> AgentResult<Option<super::ParallelHarnessHost>> {
        Ok(None)
    }
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        Ok(None)
    }
    fn take_request_events(&mut self) -> Vec<Value> {
        Vec::new()
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        Err(AgentError::new(
            "interactive shell requires a local terminal adapter; no process started",
        ))
    }
    fn send(
        &mut self,
        request: &AgentRequest,
    ) -> impl std::future::Future<Output = AgentResult<AgentServerResponse>>;
    fn approve(
        &mut self,
        approval: &Approval,
    ) -> impl std::future::Future<Output = AgentResult<Option<bool>>>;
    fn approve_batch(
        &mut self,
        approvals: &[Approval],
    ) -> impl std::future::Future<Output = AgentResult<Option<bool>>> {
        async move {
            if approvals.is_empty() {
                return Ok(Some(true));
            }
            for approval in approvals {
                match self.approve(approval).await? {
                    Some(true) => {}
                    decision => return Ok(decision),
                }
            }
            Ok(Some(true))
        }
    }
    fn generate_image(
        &mut self,
        selection: &ModelSelection,
        approval: &Approval,
    ) -> impl std::future::Future<Output = AgentResult<GeneratedImage>> {
        let _ = (selection, approval);
        async { Err(AgentError::new("image generation provider is unavailable")) }
    }
    fn ask_clarification(
        &mut self,
        question: &crate::ClarificationQuestion,
    ) -> impl std::future::Future<Output = AgentResult<Option<String>>> {
        let _ = question;
        async { Ok(None) }
    }
    fn validate_dowe_project(
        &mut self,
        root: &Path,
    ) -> impl std::future::Future<Output = AgentResult<Value>> {
        let _ = root;
        async {
            Ok(
                json!({"status":"unavailable","reason":"the active host has no Dowe compiler validation adapter"}),
            )
        }
    }
    fn event(&mut self, event: &Value) -> AgentResult<()>;
    fn secrets(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessOutcome {
    Completed,
    ApprovalRequired,
    BudgetExhausted,
    Canceled,
    ClarificationRequired,
    ValidationFailed,
}
