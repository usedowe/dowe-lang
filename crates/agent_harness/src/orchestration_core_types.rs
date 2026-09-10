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
