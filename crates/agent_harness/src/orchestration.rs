use dowe_codegraph::CodeGraphBinding;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationError(String);

impl OrchestrationError {
    fn new(message: impl Into<String>) -> Self { Self(message.into()) }
}

impl fmt::Display for OrchestrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl std::error::Error for OrchestrationError {}

pub type OrchestrationResult<T> = Result<T, OrchestrationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState { Planned, Running, Completed, Failed, Cancelled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerRole { ReadOnly, Mutating }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole { Worker, Coordinator, Reviewer }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentExecutionKind { Coordinator, Native, Child }

impl Default for AgentExecutionKind { fn default() -> Self { Self::Native } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState { Pending, Active, Completed, Interrupted }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState { Active, Completed, Interrupted, Closed }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    pub id: String,
    pub role: AgentRole,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub execution_kind: AgentExecutionKind,
    pub state: AgentState,
}

impl AgentRecord {
    pub fn new(id: impl Into<String>, role: AgentRole) -> OrchestrationResult<Self> {
        let id = id.into();
        if !valid_identifier(&id) { return Err(OrchestrationError::new("agent ID must not be empty")); }
        let execution_kind = if role == AgentRole::Coordinator { AgentExecutionKind::Coordinator } else { AgentExecutionKind::Native };
        Ok(Self { id, role, parent_id: None, execution_kind, state: AgentState::Pending })
    }

    pub fn new_child(id: impl Into<String>, parent_id: impl Into<String>) -> OrchestrationResult<Self> {
        let id = id.into();
        let parent_id = parent_id.into();
        if !valid_identifier(&id) || !valid_identifier(&parent_id) { return Err(OrchestrationError::new("child and parent agent IDs must not be empty")); }
        Ok(Self { id, role: AgentRole::Worker, parent_id: Some(parent_id), execution_kind: AgentExecutionKind::Child, state: AgentState::Pending })
    }

    pub fn start(&mut self) -> OrchestrationResult<()> {
        if self.state != AgentState::Pending { return Err(OrchestrationError::new("agent start is not a valid transition")); }
        self.state = AgentState::Active;
        Ok(())
    }

    pub fn complete(&mut self) -> OrchestrationResult<()> { self.finish(AgentState::Completed) }
    pub fn interrupt(&mut self) -> OrchestrationResult<()> { self.finish(AgentState::Interrupted) }

    fn finish(&mut self, state: AgentState) -> OrchestrationResult<()> {
        if self.state != AgentState::Active { return Err(OrchestrationError::new("agent terminal transition is not valid")); }
        self.state = state;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerState { Pending, Running, Succeeded, Failed, Cancelled }

impl WorkerState {
    fn terminal(self) -> bool { matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptState { Proposed, Accepted, Rejected }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState { Pending, Recorded }

impl Default for ValidationState {
    fn default() -> Self { Self::Pending }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState { Pending, Approved, CorrectionRequired, Rejected }

impl Default for ReviewState {
    fn default() -> Self { Self::Pending }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOutcome { Approved, CorrectionRequired, Rejected }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcknowledgementState { Pending, Acknowledged }

impl Default for AcknowledgementState {
    fn default() -> Self { Self::Pending }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState { NotRequested, Requested }

impl Default for DeliveryState {
    fn default() -> Self { Self::NotRequested }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedEditSurface {
    pub path: String,
    pub recursive: bool,
}

impl AllowedEditSurface {
    pub fn new(path: impl AsRef<str>) -> OrchestrationResult<Self> {
        Ok(Self { path: normalize_scope(path.as_ref())?, recursive: true })
    }

    pub fn exact(path: impl AsRef<str>) -> OrchestrationResult<Self> {
        Ok(Self { path: normalize_scope(path.as_ref())?, recursive: false })
    }

    pub fn allows(&self, child: impl AsRef<str>) -> bool {
        let Ok(child) = normalize_scope(child.as_ref()) else { return false; };
        if self.path == "." { return true; }
        child == self.path || (self.recursive && child.starts_with(&(self.path.clone() + "/")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptPath {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_fingerprint: Option<String>,
}

impl ReceiptPath {
    pub fn new(path: impl AsRef<str>) -> OrchestrationResult<Self> {
        Ok(Self { path: normalize_scope(path.as_ref())?, before_fingerprint: None, after_fingerprint: None })
    }

    fn with_fingerprints(
        path: impl AsRef<str>,
        before_fingerprint: Option<String>,
        after_fingerprint: Option<String>,
    ) -> OrchestrationResult<Self> {
        Ok(Self { path: normalize_scope(path.as_ref())?, before_fingerprint, after_fingerprint })
    }
}

/// A native persisted operation event projected into the orchestration model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeReceiptEvent {
    pub event: String,
    pub receipt: Receipt,
}

/// Projects one native receipt event without granting review, approval, or delivery authority.
///
/// The worker identity is deliberately supplied by the caller; it is never read from event text.
pub fn project_native_receipt_event(
    event: Value,
    expected_binding: &CodeGraphBinding,
    worker_id: impl Into<String>,
) -> OrchestrationResult<NativeReceiptEvent> {
    let event_name = event.get("event").and_then(Value::as_str)
        .ok_or_else(|| OrchestrationError::new("native receipt event is missing event"))?;
    if !matches!(event_name, "operation_started" | "operation_finished") {
        return Err(OrchestrationError::new("unsupported native receipt event"));
    }
    let receipt = event.get("receipt")
        .ok_or_else(|| OrchestrationError::new("native receipt event is missing receipt"))?;
    let object = receipt.as_object()
        .ok_or_else(|| OrchestrationError::new("native receipt must be an object"))?;
    let _operation = object.get("operation").and_then(Value::as_str)
        .filter(|value| valid_identifier(value))
        .ok_or_else(|| OrchestrationError::new("native receipt operation is malformed"))?;
    let call_id = event.get("call_id").and_then(Value::as_str)
        .filter(|value| valid_identifier(value))
        .ok_or_else(|| OrchestrationError::new("native receipt call ID is malformed"))?;
    let status = object.get("status").and_then(Value::as_str)
        .ok_or_else(|| OrchestrationError::new("native receipt status is malformed"))?;
    let state = match (event_name, status) {
        ("operation_started", "started") | (_, "incomplete") => ReceiptState::Proposed,
        ("operation_finished", "succeeded") => ReceiptState::Accepted,
        ("operation_finished", "failed") => ReceiptState::Rejected,
        _ => return Err(OrchestrationError::new("native receipt status is unknown or inconsistent")),
    };
    let binding_value = object.get("codegraphBinding")
        .ok_or_else(|| OrchestrationError::new("native receipt is missing CodeGraphBinding"))?;
    let binding: CodeGraphBinding = serde_json::from_value(binding_value.clone())
        .map_err(|_| OrchestrationError::new("native receipt CodeGraphBinding is malformed"))?;
    if &binding != expected_binding {
        return Err(OrchestrationError::new("native receipt CodeGraphBinding does not match expected binding"));
    }
    let path = object.get("path").and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty() && !value.chars().any(char::is_control))
        .ok_or_else(|| OrchestrationError::new("native receipt path is missing or malformed"))?;
    let worker_id = worker_id.into();
    if !valid_identifier(&worker_id) {
        return Err(OrchestrationError::new("native receipt worker ID is malformed"));
    }
    let fingerprint = |name: &str| -> OrchestrationResult<Option<String>> {
        match object.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) if !value.is_empty() => Ok(Some(value.clone())),
            _ => Err(OrchestrationError::new("native receipt fingerprint is malformed")),
        }
    };
    let path = ReceiptPath::with_fingerprints(path, fingerprint("beforeFingerprint")?, fingerprint("afterFingerprint")?)?;
    let receipt = Receipt::new(call_id, worker_id, path, state, true, binding)?;
    Ok(NativeReceiptEvent { event: event_name.to_owned(), receipt })
}

fn valid_identifier(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.chars().any(char::is_control)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub id: String,
    pub worker_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub path: ReceiptPath,
    pub state: ReceiptState,
    pub mutation: bool,
    pub codegraph_binding: CodeGraphBinding,
}

impl Receipt {
    pub fn new(
        id: impl Into<String>, worker_id: impl Into<String>, path: ReceiptPath,
        state: ReceiptState, mutation: bool, codegraph_binding: CodeGraphBinding,
    ) -> OrchestrationResult<Self> {
        let id = id.into();
        let worker_id = worker_id.into();
        if id.trim().is_empty() || worker_id.trim().is_empty() {
            return Err(OrchestrationError::new("receipt and worker IDs must not be empty"));
        }
        Ok(Self { id, worker_id, agent_id: None, path, state, mutation, codegraph_binding })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRecord {
    pub id: String,
    pub role: WorkerRole,
    pub state: WorkerState,
    pub required: bool,
    pub edit_surfaces: Vec<AllowedEditSurface>,
}

impl WorkerRecord {
    pub fn new(id: impl Into<String>, role: WorkerRole, edit_surfaces: Vec<AllowedEditSurface>, required: bool) -> OrchestrationResult<Self> {
        let id = id.into();
        if id.trim().is_empty() { return Err(OrchestrationError::new("worker ID must not be empty")); }
        Ok(Self { id, role, state: WorkerState::Pending, required, edit_surfaces })
    }

    pub fn start(&mut self) -> OrchestrationResult<()> {
        if self.state != WorkerState::Pending { return Err(OrchestrationError::new("worker start is not a valid transition")); }
        self.state = WorkerState::Running; Ok(())
    }
    pub fn succeed(&mut self) -> OrchestrationResult<()> { self.finish(WorkerState::Succeeded) }
    pub fn fail(&mut self) -> OrchestrationResult<()> { self.finish(WorkerState::Failed) }
    pub fn cancel(&mut self) -> OrchestrationResult<()> { self.finish(WorkerState::Cancelled) }
    fn finish(&mut self, state: WorkerState) -> OrchestrationResult<()> {
        if self.state != WorkerState::Running { return Err(OrchestrationError::new("worker completion is not a valid transition")); }
        self.state = state; Ok(())
    }
}

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

/// Pure, in-memory coordinator for session-owned orchestration state.
///
/// This type intentionally owns data and typed transitions only. Native execution,
/// providers, persistence, and child-agent management belong to a future adapter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Orchestrator {
    sessions: BTreeMap<String, SessionRecord>,
}

impl Orchestrator {
    pub fn new() -> Self { Self::default() }

    pub fn create_session(
        &mut self,
        id: impl Into<String>,
        codegraph_binding: CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        let session = SessionRecord::new(id, codegraph_binding)?;
        self.register_session(session)
    }

    pub fn register_session(&mut self, session: SessionRecord) -> OrchestrationResult<()> {
        if self.sessions.contains_key(&session.id) {
            return Err(OrchestrationError::new("session ID must be unique"));
        }
        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    pub fn add_agent(&mut self, session_id: &str, agent: AgentRecord) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.add_agent(agent)
    }

    pub fn add_task(&mut self, session_id: &str, task: TaskRecord) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.add_task(task)
    }

    pub fn start_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.start()
    }

    pub fn finish_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.complete()
    }

    pub fn cancel_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.cancel()
    }

    /// Starts a task only when the caller presents the exact session binding.
    pub fn start_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.start_task(session_id, task_id)
    }

    pub fn finish_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.finish_task(session_id, task_id)
    }

    pub fn cancel_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.cancel_task(session_id, task_id)
    }

    pub fn record_receipt(
        &mut self,
        session_id: &str,
        task_id: &str,
        receipt: Receipt,
    ) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.record_receipt(task_id, receipt)
    }

    pub fn complete_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.complete()
    }

    pub fn interrupt_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.interrupt()
    }

    pub fn close_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.close()
    }

    /// Returns an owned snapshot so later coordinator mutations cannot alter it.
    pub fn inspect_session(&self, session_id: &str) -> OrchestrationResult<SessionRecord> {
        self.session(session_id).cloned()
    }

    pub fn session_count(&self) -> usize { self.sessions.len() }

    fn session(&self, session_id: &str) -> OrchestrationResult<&SessionRecord> {
        self.sessions.get(session_id)
            .ok_or_else(|| OrchestrationError::new("session is not registered"))
    }

    fn session_mut(&mut self, session_id: &str) -> OrchestrationResult<&mut SessionRecord> {
        self.sessions.get_mut(session_id)
            .ok_or_else(|| OrchestrationError::new("session is not registered"))
    }

    fn task_mut<'a>(session: &'a mut SessionRecord, task_id: &str) -> OrchestrationResult<&'a mut TaskRecord> {
        session.tasks.iter_mut().find(|task| task.id == task_id)
            .ok_or_else(|| OrchestrationError::new("task is not owned by session"))
    }

    fn ensure_task_binding(
        &self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        let session = self.session(session_id)?;
        if &session.codegraph_binding != binding {
            return Err(OrchestrationError::new("CodeGraphBinding does not match session"));
        }
        let task = session.tasks.iter().find(|task| task.id == task_id)
            .ok_or_else(|| OrchestrationError::new("task is not owned by session"))?;
        if &task.codegraph_binding != binding {
            return Err(OrchestrationError::new("CodeGraphBinding does not match task"));
        }
        Ok(())
    }
}

fn normalize_scope(raw: &str) -> OrchestrationResult<String> {
    let raw = raw.trim().replace('\\', "/");
    if raw.is_empty() || raw.starts_with('/') || raw.starts_with("//") || raw.split('/').next().is_some_and(|part| part.contains(':')) {
        return Err(OrchestrationError::new("scope must be a non-empty project-relative path"));
    }
    let mut parts = Vec::new();
    for part in raw.split('/') {
        if part.is_empty() || part == "." { continue; }
        if part == ".." { return Err(OrchestrationError::new("scope must not contain traversal")); }
        parts.push(part);
    }
    if parts.is_empty() { return Ok(".".into()); }
    Ok(parts.join("/"))
}

fn worker_scope_allowed(worker: &AllowedEditSurface, task: &[AllowedEditSurface]) -> bool {
    task.iter().any(|surface| surface.allows(&worker.path) && (!worker.recursive || surface.recursive || worker.path == surface.path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dowe_codegraph::CodeGraphMode;

    fn binding(revision: u64) -> CodeGraphBinding { CodeGraphBinding { generation: "g1".into(), revision, root: "/project".into(), mode: CodeGraphMode::Project } }
    fn task() -> TaskRecord { TaskRecord::new("t1", binding(1), vec![AllowedEditSurface::new("src").unwrap()]).unwrap() }

    #[test] fn scope_validation_and_subset() {
        assert!(AllowedEditSurface::new("/src").is_err()); assert!(AllowedEditSurface::new("src/../x").is_err());
        let mut t = task(); assert!(t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src/lib").unwrap()], true).unwrap()).is_ok());
        assert!(t.add_worker(WorkerRecord::new("bad", WorkerRole::Mutating, vec![AllowedEditSurface::new("tests").unwrap()], true).unwrap()).is_err());
    }

    #[test] fn transitions_are_explicit_and_completion_is_gated() {
        let mut t = task();
        t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        assert!(t.complete().is_err()); t.start().unwrap(); assert!(t.complete().is_err());
        t.workers[0].start().unwrap(); assert!(t.complete().is_err()); t.workers[0].succeed().unwrap(); t.complete().unwrap(); assert_eq!(t.state, TaskState::Completed);
    }

    #[test] fn readonly_workers_cannot_create_mutation_receipts() {
        let mut t = task(); t.add_worker(WorkerRecord::new("w", WorkerRole::ReadOnly, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        let receipt = Receipt::new("r", "w", ReceiptPath::new("src/a.rs").unwrap(), ReceiptState::Proposed, true, binding(1)).unwrap();
        assert!(t.record_receipt(receipt).is_err());
    }

    #[test] fn governance_requires_validation_and_exact_candidate_binding() {
        let mut t = task();
        assert!(t.record_review_outcome(binding(1), ReviewOutcome::Approved).is_err());
        assert!(t.record_validation_evidence(binding(2), "passed").is_err());
        t.record_validation_evidence(binding(1), "passed").unwrap();
        assert!(t.record_review_outcome(binding(2), ReviewOutcome::Approved).is_err());
    }

    #[test] fn correction_required_blocks_acknowledgement() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::CorrectionRequired).unwrap();
        assert!(t.acknowledge_approved_review(binding(1)).is_err());
    }

    #[test] fn acknowledgement_is_explicit_and_single_use() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::Approved).unwrap();
        t.acknowledge_approved_review(binding(1)).unwrap();
        assert!(t.acknowledge_approved_review(binding(1)).is_err());
    }

    #[test] fn delivery_is_separate_from_review_approval_and_completion() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::Approved).unwrap();
        assert!(t.request_delivery().is_err());
        t.acknowledge_approved_review(binding(1)).unwrap();
        assert!(t.request_delivery().is_err());
        t.start().unwrap();
        t.complete().unwrap();
        t.request_delivery().unwrap();
        assert_eq!(t.delivery_state, DeliveryState::Requested);
    }

    #[test] fn serde_round_trip_preserves_binding() {
        let t = task(); let round_trip: TaskRecord = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap(); assert_eq!(round_trip.codegraph_binding, t.codegraph_binding);
    }

    #[test] fn receipt_binding_must_match() {
        let mut t = task(); t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        let receipt = Receipt::new("r", "w", ReceiptPath::new("src/a.rs").unwrap(), ReceiptState::Proposed, true, binding(2)).unwrap(); assert!(t.record_receipt(receipt).is_err());
    }

    #[test]
    fn sessions_isolate_agents_tasks_and_bindings() {
        let mut first = SessionRecord::new("s1", binding(1)).unwrap();
        let mut second = SessionRecord::new("s2", binding(2)).unwrap();
        first.add_agent(AgentRecord::new("agent-1", AgentRole::Worker).unwrap()).unwrap();
        second.add_agent(AgentRecord::new("agent-2", AgentRole::Worker).unwrap()).unwrap();
        let mut first_task = TaskRecord::new("task-1", binding(1), vec![AllowedEditSurface::new("src").unwrap()]).unwrap();
        first_task.add_worker(WorkerRecord::new("agent-1", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        assert!(first.add_task(first_task).is_ok());
        let mut foreign_worker_task = TaskRecord::new("task-2", binding(1), vec![]).unwrap();
        foreign_worker_task.add_worker(WorkerRecord::new("agent-2", WorkerRole::Mutating, vec![], true).unwrap()).unwrap();
        assert!(first.add_task(foreign_worker_task).is_err());
        assert!(second.tasks.is_empty());
    }

    #[test]
    fn sessions_reject_duplicate_agents_and_foreign_bindings() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).unwrap();
        assert!(session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
        assert!(session.add_task(TaskRecord::new("task", binding(2), vec![]).unwrap()).is_err());
        assert!(SessionRecord::new("", binding(1)).is_err());
        assert!(AgentRecord::new("", AgentRole::Worker).is_err());
    }

    #[test]
    fn session_lifecycle_closes_admission_and_transitions_explicitly() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.complete().unwrap();
        assert_eq!(session.state, SessionState::Completed);
        assert!(session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
        assert!(session.add_task(TaskRecord::new("task", binding(1), vec![]).unwrap()).is_err());
        session.close().unwrap();
        assert_eq!(session.state, SessionState::Closed);
        assert!(session.close().is_err());
        let mut interrupted = SessionRecord::new("s2", binding(1)).unwrap();
        interrupted.interrupt().unwrap();
        assert!(interrupted.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
    }

    #[test]
    fn session_serde_round_trip_preserves_ownership_and_binding() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.add_agent(AgentRecord::new("agent-1", AgentRole::Worker).unwrap()).unwrap();
        let mut task = TaskRecord::new("task-1", binding(1), vec![]).unwrap();
        task.add_worker(WorkerRecord::new("agent-1", WorkerRole::ReadOnly, vec![], true).unwrap()).unwrap();
        session.add_task(task).unwrap();
        let decoded: SessionRecord = serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(decoded, session);
        assert_eq!(decoded.agents[0].id, "agent-1");
        assert_eq!(decoded.tasks[0].workers[0].id, decoded.agents[0].id);
        assert_eq!(decoded.tasks[0].codegraph_binding, decoded.codegraph_binding);
    }

    #[test]
    fn orchestrator_two_sessions_isolate_ids_and_bindings() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            orchestrator.create_session("s2", binding(2)).unwrap();
            orchestrator.add_agent("s1", AgentRecord::new("a1", AgentRole::Worker).unwrap()).unwrap();
            orchestrator.add_agent("s2", AgentRecord::new("a2", AgentRole::Worker).unwrap()).unwrap();
            let mut task = TaskRecord::new("t1", binding(1), vec![]).unwrap();
            task.add_worker(WorkerRecord::new("a1", WorkerRole::ReadOnly, vec![], true).unwrap()).unwrap();
            orchestrator.add_task("s1", task).unwrap();
            assert!(orchestrator.start_task("s2", "t1").is_err());
            assert!(orchestrator.add_task("s1", TaskRecord::new("foreign", binding(2), vec![]).unwrap()).is_err());
        }

    #[test]
    fn orchestrator_snapshot_isolation_and_interrupt_restriction() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            let snapshot = orchestrator.inspect_session("s1").unwrap();
            orchestrator.add_agent("s1", AgentRecord::new("a1", AgentRole::Worker).unwrap()).unwrap();
            assert!(snapshot.agents.is_empty());
            orchestrator.interrupt_session("s1").unwrap();
            assert!(orchestrator.add_agent("s1", AgentRecord::new("a2", AgentRole::Worker).unwrap()).is_err());
            orchestrator.close_session("s1").unwrap();
            assert!(orchestrator.close_session("s1").is_err());
        }

    #[test]
    fn orchestrator_api_smoke() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            orchestrator.add_task("s1", TaskRecord::new("t1", binding(1), vec![]).unwrap()).unwrap();
            orchestrator.start_task_with_binding("s1", "t1", &binding(1)).unwrap();
            orchestrator.finish_task("s1", "t1").unwrap();
            assert_eq!(orchestrator.inspect_session("s1").unwrap().tasks[0].state, TaskState::Completed);
        }

    fn native_event(event: &str, status: &str) -> Value {
        serde_json::json!({
            "event": event,
            "call_id": "call-1",
            "receipt": {
                "operation": "write_file",
                "status": status,
                "path": "src/a.rs",
                "beforeFingerprint": "before",
                "afterFingerprint": "after",
                "codegraphBinding": binding(1),
            }
        })
    }

    #[test]
    fn native_receipt_projection_is_typed_and_preserves_binding_and_fingerprints() {
        let projected = project_native_receipt_event(native_event("operation_finished", "succeeded"), &binding(1), "worker-7").unwrap();
        assert_eq!(projected.event, "operation_finished");
        assert_eq!(projected.receipt.worker_id, "worker-7");
        assert_eq!(projected.receipt.state, ReceiptState::Accepted);
        assert_eq!(projected.receipt.codegraph_binding, binding(1));
        assert_eq!(projected.receipt.path.before_fingerprint.as_deref(), Some("before"));
        assert_eq!(projected.receipt.path.after_fingerprint.as_deref(), Some("after"));
    }

    #[test]
    fn native_receipt_projection_rejects_missing_receipt_and_binding_mismatch() {
        let missing = serde_json::json!({"event":"operation_started", "call_id":"call-1"});
        assert!(project_native_receipt_event(missing, &binding(1), "worker").is_err());
        assert!(project_native_receipt_event(native_event("operation_started", "started"), &binding(2), "worker").is_err());
    }

    #[test]
    fn native_receipt_projection_rejects_malformed_status_and_path() {
        assert!(project_native_receipt_event(native_event("operation_finished", "unknown"), &binding(1), "worker").is_err());
        let mut malformed = native_event("operation_finished", "succeeded");
        malformed["receipt"]["path"] = serde_json::json!("../escape");
        assert!(project_native_receipt_event(malformed, &binding(1), "worker").is_err());
        let mut malformed_id = native_event("operation_finished", "succeeded");
        malformed_id["call_id"] = serde_json::json!("");
        assert!(project_native_receipt_event(malformed_id, &binding(1), "worker").is_err());
    }

    #[test]
    fn native_receipt_projection_preserves_non_authority_for_incomplete_and_failed() {
        let incomplete = project_native_receipt_event(native_event("operation_started", "started"), &binding(1), "worker").unwrap();
        let failed = project_native_receipt_event(native_event("operation_finished", "failed"), &binding(1), "worker").unwrap();
        assert_eq!(incomplete.receipt.state, ReceiptState::Proposed);
        assert_eq!(failed.receipt.state, ReceiptState::Rejected);
        assert!(incomplete.receipt.state != ReceiptState::Accepted);
        assert!(failed.receipt.state != ReceiptState::Accepted);
    }

    #[test]
    fn native_receipt_projection_receipt_path_round_trips_through_serde() {
        let projected = project_native_receipt_event(native_event("operation_finished", "succeeded"), &binding(1), "worker").unwrap();
        let encoded = serde_json::to_string(&projected.receipt).unwrap();
        let decoded: Receipt = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, projected.receipt);
    }
}
