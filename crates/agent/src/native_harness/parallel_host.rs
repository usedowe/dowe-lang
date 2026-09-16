use super::{Approval, HarnessHost, HarnessTerminal, ModelSelection};
use crate::{AgentRequest, AgentResult, AgentServerResponse, GeneratedImage};
use serde_json::Value;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = AgentResult<T>> + Send + 'a>>;

pub trait ParallelWorker: Send {
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        Ok(None)
    }
    fn take_request_events(&mut self) -> Vec<Value> {
        Vec::new()
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>>;
    fn send<'a>(&'a mut self, request: &'a AgentRequest) -> BoxFuture<'a, AgentServerResponse>;
    fn approve<'a>(&'a mut self, approval: &'a Approval) -> BoxFuture<'a, Option<bool>>;
    fn approve_batch<'a>(&'a mut self, approvals: &'a [Approval]) -> BoxFuture<'a, Option<bool>> {
        Box::pin(async move {
            for approval in approvals {
                if self.approve(approval).await? != Some(true) {
                    return Ok(None);
                }
            }
            Ok(Some(true))
        })
    }
    fn generate_image<'a>(
        &'a mut self,
        selection: &'a ModelSelection,
        approval: &'a Approval,
    ) -> BoxFuture<'a, GeneratedImage>;
    fn ask_clarification<'a>(
        &'a mut self,
        question: &'a crate::ClarificationQuestion,
    ) -> BoxFuture<'a, Option<String>>;
    fn validate_dowe_project<'a>(&'a mut self, root: &'a Path) -> BoxFuture<'a, Value>;
    fn event(&mut self, event: &Value) -> AgentResult<()>;
    fn secrets(&self) -> Vec<String> {
        Vec::new()
    }
}

pub struct ParallelHarnessHost {
    worker: Box<dyn ParallelWorker>,
}

impl ParallelHarnessHost {
    pub fn new(worker: impl ParallelWorker + 'static) -> Self {
        Self {
            worker: Box::new(worker),
        }
    }
}

impl HarnessHost for ParallelHarnessHost {
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        self.worker.supervisor()
    }

    fn take_request_events(&mut self) -> Vec<Value> {
        self.worker.take_request_events()
    }

    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        self.worker.open_terminal()
    }

    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.worker.send(request).await
    }

    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        self.worker.approve(approval).await
    }

    async fn approve_batch(&mut self, approvals: &[Approval]) -> AgentResult<Option<bool>> {
        self.worker.approve_batch(approvals).await
    }

    async fn generate_image(
        &mut self,
        selection: &ModelSelection,
        approval: &Approval,
    ) -> AgentResult<GeneratedImage> {
        self.worker.generate_image(selection, approval).await
    }

    async fn ask_clarification(
        &mut self,
        question: &crate::ClarificationQuestion,
    ) -> AgentResult<Option<String>> {
        self.worker.ask_clarification(question).await
    }

    async fn validate_dowe_project(&mut self, root: &Path) -> AgentResult<Value> {
        self.worker.validate_dowe_project(root).await
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.worker.event(event)
    }

    fn secrets(&self) -> Vec<String> {
        self.worker.secrets()
    }
}
