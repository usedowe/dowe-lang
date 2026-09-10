#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub state: TaskState,
    pub codegraph_binding: CodeGraphBinding,
    pub edit_surfaces: Vec<AllowedEditSurface>,
    pub workers: Vec<WorkerRecord>,
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub validation_state: ValidationState,
    #[serde(default)]
    pub validation_evidence: Option<String>,
    #[serde(default)]
    pub validation_codegraph_binding: Option<CodeGraphBinding>,
    #[serde(default)]
    pub review_state: ReviewState,
    #[serde(default)]
    pub review_codegraph_binding: Option<CodeGraphBinding>,
    #[serde(default)]
    pub acknowledgement_state: AcknowledgementState,
    #[serde(default)]
    pub delivery_state: DeliveryState,
}

impl TaskRecord {
    pub fn new(id: impl Into<String>, codegraph_binding: CodeGraphBinding, edit_surfaces: Vec<AllowedEditSurface>) -> OrchestrationResult<Self> {
        let id = id.into();
        if id.trim().is_empty() { return Err(OrchestrationError::new("task ID must not be empty")); }
        Ok(Self {
        id, state: TaskState::Planned, codegraph_binding, edit_surfaces,
        workers: Vec::new(), receipts: Vec::new(),
        validation_state: ValidationState::Pending,
        validation_evidence: None,
        validation_codegraph_binding: None,
        review_state: ReviewState::Pending,
        review_codegraph_binding: None,
        acknowledgement_state: AcknowledgementState::Pending,
        delivery_state: DeliveryState::NotRequested,
    })
    }

    pub fn add_worker(&mut self, worker: WorkerRecord) -> OrchestrationResult<()> {
        if self.workers.iter().any(|w| w.id == worker.id) { return Err(OrchestrationError::new("worker ID must be unique")); }
        if worker.edit_surfaces.iter().any(|scope| !worker_scope_allowed(scope, &self.edit_surfaces)) {
            return Err(OrchestrationError::new("worker scope is outside task edit surfaces"));
        }
        self.workers.push(worker); Ok(())
    }

    pub fn start(&mut self) -> OrchestrationResult<()> {
        if self.state != TaskState::Planned { return Err(OrchestrationError::new("task start is not a valid transition")); }
        self.state = TaskState::Running; Ok(())
    }

    pub fn record_receipt(&mut self, receipt: Receipt) -> OrchestrationResult<()> {
        let worker = self.workers.iter().find(|w| w.id == receipt.worker_id).ok_or_else(|| OrchestrationError::new("receipt worker is not part of task"))?;
        if receipt.codegraph_binding != self.codegraph_binding { return Err(OrchestrationError::new("receipt CodeGraphBinding does not match task")); }
        if receipt.mutation && worker.role == WorkerRole::ReadOnly { return Err(OrchestrationError::new("read-only worker cannot create mutation receipts")); }
        if self.receipts.iter().any(|r| r.id == receipt.id) { return Err(OrchestrationError::new("receipt ID must be unique")); }
        self.receipts.push(receipt); Ok(())
    }

    pub fn record_validation_evidence(&mut self, binding: CodeGraphBinding, evidence: impl Into<String>) -> OrchestrationResult<()> {
        if binding != self.codegraph_binding { return Err(OrchestrationError::new("validation CodeGraphBinding does not match task")); }
        let evidence = evidence.into();
        if evidence.trim().is_empty() { return Err(OrchestrationError::new("validation evidence must not be empty")); }
        if self.validation_state != ValidationState::Pending { return Err(OrchestrationError::new("validation evidence can only be recorded once")); }
        self.validation_state = ValidationState::Recorded;
        self.validation_evidence = Some(evidence);
        self.validation_codegraph_binding = Some(binding);
        Ok(())
    }

    pub fn record_review_outcome(&mut self, binding: CodeGraphBinding, outcome: ReviewOutcome) -> OrchestrationResult<()> {
        if binding != self.codegraph_binding { return Err(OrchestrationError::new("review CodeGraphBinding does not match task")); }
        if self.validation_state != ValidationState::Recorded || self.validation_codegraph_binding.as_ref() != Some(&binding) {
            return Err(OrchestrationError::new("validation evidence is required for this CodeGraphBinding before review"));
        }
        if self.review_state != ReviewState::Pending { return Err(OrchestrationError::new("review outcome can only be recorded once")); }
        self.review_state = match outcome {
            ReviewOutcome::Approved => ReviewState::Approved,
            ReviewOutcome::CorrectionRequired => ReviewState::CorrectionRequired,
            ReviewOutcome::Rejected => ReviewState::Rejected,
        };
        self.review_codegraph_binding = Some(binding);
        Ok(())
    }

    /// Records explicit human authorization; it does not perform delivery or Git operations.
    pub fn acknowledge_approved_review(&mut self, binding: CodeGraphBinding) -> OrchestrationResult<()> {
        if binding != self.codegraph_binding { return Err(OrchestrationError::new("acknowledgement CodeGraphBinding does not match task")); }
        if self.acknowledgement_state == AcknowledgementState::Acknowledged { return Err(OrchestrationError::new("review acknowledgement is single-use")); }
        if self.review_state == ReviewState::CorrectionRequired { return Err(OrchestrationError::new("correction-required review cannot be acknowledged")); }
        if self.review_state != ReviewState::Approved || self.review_codegraph_binding.as_ref() != Some(&binding) {
            return Err(OrchestrationError::new("an approved review for this CodeGraphBinding is required before acknowledgement"));
        }
        self.acknowledgement_state = AcknowledgementState::Acknowledged;
        Ok(())
    }

    pub fn request_delivery(&mut self) -> OrchestrationResult<()> {
        if self.state != TaskState::Completed { return Err(OrchestrationError::new("task must be completed before delivery")); }
        if self.acknowledgement_state != AcknowledgementState::Acknowledged { return Err(OrchestrationError::new("review acknowledgement is required before delivery")); }
        if self.delivery_state == DeliveryState::Requested { return Err(OrchestrationError::new("delivery has already been requested")); }
        self.delivery_state = DeliveryState::Requested;
        Ok(())
    }

    pub fn complete(&mut self) -> OrchestrationResult<()> {
        if self.state != TaskState::Running { return Err(OrchestrationError::new("task completion is not a valid transition")); }
        if self.workers.iter().any(|w| !w.state.terminal()) { return Err(OrchestrationError::new("all workers must be terminal before completion")); }
        if self.workers.iter().any(|w| w.required && w.state == WorkerState::Failed) { return Err(OrchestrationError::new("a required worker failed")); }
        self.state = TaskState::Completed; Ok(())
    }

    pub fn fail(&mut self) -> OrchestrationResult<()> {
        if self.state != TaskState::Running { return Err(OrchestrationError::new("task failure is not a valid transition")); }
        self.state = TaskState::Failed; Ok(())
    }

    pub fn cancel(&mut self) -> OrchestrationResult<()> {
        if !matches!(self.state, TaskState::Planned | TaskState::Running) { return Err(OrchestrationError::new("task cancellation is not a valid transition")); }
        self.state = TaskState::Cancelled; Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: String,
    pub state: SessionState,
    pub codegraph_binding: CodeGraphBinding,
    pub agents: Vec<AgentRecord>,
    pub tasks: Vec<TaskRecord>,
}

impl SessionRecord {
    pub fn new(id: impl Into<String>, codegraph_binding: CodeGraphBinding) -> OrchestrationResult<Self> {
        let id = id.into();
        if !valid_identifier(&id) { return Err(OrchestrationError::new("session ID must not be empty")); }
        Ok(Self { id, state: SessionState::Active, codegraph_binding, agents: Vec::new(), tasks: Vec::new() })
    }

    pub fn add_agent(&mut self, agent: AgentRecord) -> OrchestrationResult<()> {
        self.ensure_active()?;
        if self.agents.iter().any(|existing| existing.id == agent.id) {
            return Err(OrchestrationError::new("agent ID must be unique within session"));
        }
        match (&agent.parent_id, agent.execution_kind) {
            (None, AgentExecutionKind::Child) => return Err(OrchestrationError::new("child agent must declare a parent")),
            (Some(_), AgentExecutionKind::Coordinator | AgentExecutionKind::Native) => return Err(OrchestrationError::new("only child agents may declare a parent")),
            (Some(parent_id), AgentExecutionKind::Child) => {
                let parent = self.agents.iter().find(|existing| existing.id == *parent_id)
                    .ok_or_else(|| OrchestrationError::new("child parent is not owned by session"))?;
                if parent.role != AgentRole::Coordinator || parent.execution_kind == AgentExecutionKind::Child || parent.parent_id.is_some() {
                    return Err(OrchestrationError::new("child parent must be a same-session coordinator"));
                }
            }
            (None, _) => {}
        }
        self.agents.push(agent);
        Ok(())
    }

    pub fn add_task(&mut self, task: TaskRecord) -> OrchestrationResult<()> {
        self.ensure_active()?;
        if task.codegraph_binding != self.codegraph_binding {
            return Err(OrchestrationError::new("task CodeGraphBinding does not match session"));
        }
        if self.tasks.iter().any(|existing| existing.id == task.id) {
            return Err(OrchestrationError::new("task ID must be unique within session"));
        }
        for worker in &task.workers {
            let agent = self.agents.iter().find(|agent| agent.id == worker.id)
                .ok_or_else(|| OrchestrationError::new("task worker is not owned by session"))?;
            if agent.role != AgentRole::Worker {
                return Err(OrchestrationError::new("task worker must be a worker agent"));
            }
        }
        if task.receipts.iter().any(|receipt| receipt.codegraph_binding != self.codegraph_binding) {
            return Err(OrchestrationError::new("task receipt CodeGraphBinding does not match session"));
        }
        self.tasks.push(task);
        Ok(())
    }

    pub fn record_receipt(&mut self, task_id: &str, receipt: Receipt) -> OrchestrationResult<()> {
        self.ensure_active()?;
        if receipt.codegraph_binding != self.codegraph_binding {
            return Err(OrchestrationError::new("receipt CodeGraphBinding does not match session"));
        }
        let task = self.tasks.iter_mut().find(|task| task.id == task_id)
            .ok_or_else(|| OrchestrationError::new("task is not owned by session"))?;
        task.record_receipt(receipt)
    }

    pub fn complete(&mut self) -> OrchestrationResult<()> {
        if self.state != SessionState::Active { return Err(OrchestrationError::new("session completion is not a valid transition")); }
        self.state = SessionState::Completed;
        Ok(())
    }

    pub fn interrupt(&mut self) -> OrchestrationResult<()> {
        if self.state != SessionState::Active { return Err(OrchestrationError::new("session interruption is not a valid transition")); }
        self.state = SessionState::Interrupted;
        Ok(())
    }

    pub fn close(&mut self) -> OrchestrationResult<()> {
        if self.state == SessionState::Closed { return Err(OrchestrationError::new("session is already closed")); }
        self.state = SessionState::Closed;
        Ok(())
    }

    fn ensure_active(&self) -> OrchestrationResult<()> {
        if self.state != SessionState::Active { return Err(OrchestrationError::new("session is not active")); }
        Ok(())
    }
}
