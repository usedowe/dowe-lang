use super::request::build_request;
use super::{
    Approval, HarnessConfig, HarnessRole, HarnessSession, HarnessStore, HarnessTools, HarnessTurn,
    ModelSelection, ToolResult, response_turn, select_units,
};
use crate::codegraph_enrichment::{SemanticEnrichment, SemanticStatus, enrich_codegraph};
use crate::{
    AgentError, AgentMessage, AgentMessageContent, AgentPrepareOptions, AgentRequest,
    AgentRequestType, AgentResult, AgentServerResponse, AgentUsageTotals, GeneratedImage,
    agent_model_details, prepare_agent_request,
};
use base64::Engine;
use dowe_codegraph::{CodeGraphBinding, ensure_persistent_codegraph, read_persistent_codegraph};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

mod shell_host;
mod transport;

pub trait HarnessHost {
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        Ok(None)
    }
    fn take_request_events(&mut self) -> Vec<Value> {
        Vec::new()
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn super::HarnessTerminal>> {
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
            Ok(json!({
                "status": "unavailable",
                "reason": "the active host has no Dowe compiler validation adapter",
            }))
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

include!("engine/turn.rs");
include!("engine/round.rs");
include!("engine/preparation.rs");
include!("engine/call_execution.rs");
include!("engine/write_batch.rs");
include!("engine/receipts_and_events.rs");
include!("engine/compaction.rs");
