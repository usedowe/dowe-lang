use super::budget::WorkflowBudget;
use super::*;
use crate::{AgentRequest, AgentServerResponse, GeneratedImage};
use std::sync::{Arc, Mutex};

pub(super) struct BudgetHost<'a, H> {
    pub inner: &'a mut H,
    pub budget: WorkflowBudget,
    pub parallel_budget: Option<Arc<Mutex<WorkflowBudget>>>,
}

struct BudgetedParallelWorker {
    inner: super::super::ParallelHarnessHost,
    budget: Arc<Mutex<WorkflowBudget>>,
}

impl<H: HarnessHost> HarnessHost for BudgetHost<'_, H> {
    fn parallel_worker(&mut self) -> AgentResult<Option<super::super::ParallelHarnessHost>> {
        let budget = self
            .parallel_budget
            .get_or_insert_with(|| Arc::new(Mutex::new(self.budget.clone())))
            .clone();
        let Some(inner) = self.inner.parallel_worker()? else {
            return Ok(None);
        };
        Ok(Some(super::super::ParallelHarnessHost::new(
            BudgetedParallelWorker { inner, budget },
        )))
    }

    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        let mut request = request.clone();
        let reservation = match self.budget.reserve(&mut request) {
            Ok(reservation) => reservation,
            Err(error) => {
                self.inner.event(&self.budget.event())?;
                return Err(error);
            }
        };
        self.inner.event(&self.budget.event())?;
        let result = self.inner.send(&request).await;
        self.budget.settle(
            &request,
            reservation,
            result.as_ref().ok().map(|r| &r.payload),
        );
        self.inner.event(&self.budget.event())?;
        result
    }

    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        self.budget.check()?;
        self.inner.approve(approval).await
    }

    async fn approve_batch(&mut self, approvals: &[Approval]) -> AgentResult<Option<bool>> {
        self.budget.check()?;
        self.inner.approve_batch(approvals).await
    }

    async fn ask_clarification(
        &mut self,
        question: &crate::ClarificationQuestion,
    ) -> AgentResult<Option<String>> {
        self.inner.ask_clarification(question).await
    }

    async fn validate_dowe_project(
        &mut self,
        root: &std::path::Path,
    ) -> AgentResult<serde_json::Value> {
        self.inner.validate_dowe_project(root).await
    }

    async fn generate_image(
        &mut self,
        _: &ModelSelection,
        _: &Approval,
    ) -> AgentResult<crate::GeneratedImage> {
        self.budget.exhausted = true;
        Err(AgentError::new(
            "workflow image generation requires a metered provider contract",
        ))
    }

    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        self.budget.check()?;
        self.inner.open_terminal()
    }

    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        self.inner.supervisor()
    }
    fn take_request_events(&mut self) -> Vec<serde_json::Value> {
        self.inner.take_request_events()
    }
    fn secrets(&self) -> Vec<String> {
        self.inner.secrets()
    }
    fn event(&mut self, event: &serde_json::Value) -> AgentResult<()> {
        self.inner.event(event)
    }
}

impl<H> BudgetHost<'_, H> {
    pub(super) fn sync_parallel_budget(&mut self) -> AgentResult<()> {
        if let Some(budget) = &self.parallel_budget {
            self.budget = budget
                .lock()
                .map_err(|_| AgentError::new("workflow budget is poisoned"))?
                .clone();
        }
        Ok(())
    }
}

impl super::super::ParallelWorker for BudgetedParallelWorker {
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        self.inner.open_terminal()
    }

    fn send<'a>(
        &'a mut self,
        request: &'a AgentRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AgentResult<AgentServerResponse>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut request = request.clone();
            let reservation = {
                let mut budget = self
                    .budget
                    .lock()
                    .map_err(|_| AgentError::new("workflow budget is poisoned"))?;
                let reservation = budget.reserve(&mut request)?;
                self.inner.event(&budget.event())?;
                reservation
            };
            let result = self.inner.send(&request).await;
            let event = {
                let mut budget = self
                    .budget
                    .lock()
                    .map_err(|_| AgentError::new("workflow budget is poisoned"))?;
                budget.settle(
                    &request,
                    reservation,
                    result.as_ref().ok().map(|response| &response.payload),
                );
                budget.event()
            };
            self.inner.event(&event)?;
            result
        })
    }

    fn approve<'a>(
        &'a mut self,
        approval: &'a Approval,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<Option<bool>>> + Send + 'a>>
    {
        Box::pin(async move {
            self.budget
                .lock()
                .map_err(|_| AgentError::new("workflow budget is poisoned"))?
                .check()?;
            self.inner.approve(approval).await
        })
    }

    fn generate_image<'a>(
        &'a mut self,
        selection: &'a ModelSelection,
        approval: &'a Approval,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<GeneratedImage>> + Send + 'a>>
    {
        Box::pin(async move {
            let mut budget = self
                .budget
                .lock()
                .map_err(|_| AgentError::new("workflow budget is poisoned"))?;
            budget.exhausted = true;
            let _ = (selection, approval);
            Err(AgentError::new(
                "workflow image generation requires a metered provider contract",
            ))
        })
    }

    fn ask_clarification<'a>(
        &'a mut self,
        question: &'a crate::ClarificationQuestion,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<Option<String>>> + Send + 'a>>
    {
        Box::pin(async move { self.inner.ask_clarification(question).await })
    }

    fn validate_dowe_project<'a>(
        &'a mut self,
        root: &'a std::path::Path,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AgentResult<serde_json::Value>> + Send + 'a>,
    > {
        Box::pin(async move { self.inner.validate_dowe_project(root).await })
    }

    fn event(&mut self, event: &serde_json::Value) -> AgentResult<()> {
        self.inner.event(event)
    }

    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        self.inner.supervisor()
    }

    fn take_request_events(&mut self) -> Vec<serde_json::Value> {
        self.inner.take_request_events()
    }

    fn secrets(&self) -> Vec<String> {
        self.inner.secrets()
    }
}
