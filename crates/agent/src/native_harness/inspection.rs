//! Read-only, project-scoped inspection projections for the native harness.
use super::{HarnessQueuedTask, HarnessSession, HarnessStore};
use crate::{AgentError, AgentResult};
use dowe_agent_harness::{AgentRecord, CodeGraphBinding, SessionState, TaskRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAX_EVENTS: usize = 32;
pub const MAX_EVENT_BYTES: usize = 8 * 1024;
pub const MAX_PAGE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInspection { pub session_id: String, pub project: String, pub revision: u64, pub interrupted: bool, pub turn_count: usize, pub event_cursor: u64, pub summary: Option<String>, pub title: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksInspection { pub session_id: String, pub tasks: Vec<HarnessQueuedTask> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceInspection { pub session_id: String, pub state: SessionState, pub codegraph_binding: CodeGraphBinding, pub agents: Vec<AgentRecord>, pub tasks: Vec<TaskRecord>, pub authority: &'static str }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionEventReceipt {
    pub operation: Option<String>, pub status: Option<String>, pub path: Option<String>,
    pub before_fingerprint: Option<String>, pub after_fingerprint: Option<String>,
    pub codegraph_binding: Option<CodeGraphBinding>,
}

/// The only event shape exposed to an SDK observer. Unknown event fields are dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub cursor: u64, pub kind: String, pub request_id: Option<String>, pub provider: Option<String>,
    pub model: Option<String>, pub role: Option<String>, pub status: Option<String>,
    pub task_id: Option<String>, pub worker_id: Option<String>, pub call_id: Option<String>,
    pub receipt: Option<SessionEventReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPage { pub session_id: String, pub since: u64, pub cursor: u64, pub events: Vec<SessionEvent>, pub has_more: bool, pub event_limit: usize, pub byte_limit: usize }

#[derive(Debug, Clone, Copy)]
pub struct SessionEventLimits { pub max_events: usize, pub max_bytes: usize }
impl Default for SessionEventLimits { fn default() -> Self { Self { max_events: MAX_EVENTS, max_bytes: MAX_PAGE_BYTES } } }
impl SessionEventLimits { fn validate(self) -> AgentResult<Self> { if self.max_events == 0 || self.max_events > MAX_EVENTS { return Err(AgentError::new("event limit is outside the allowed range")); } if self.max_bytes == 0 || self.max_bytes > MAX_PAGE_BYTES { return Err(AgentError::new("page byte limit is outside the allowed range")); } Ok(self) } }

#[derive(Debug, Default)]
struct ObserverState { active: Mutex<Option<u64>>, next: AtomicU64 }
/// Read-only, session-bound observer facade for in-process SDK consumers.
#[derive(Clone)]
pub struct SessionObserver { store: HarnessStore, session_id: String, state: Arc<ObserverState> }
#[derive(Debug)]
pub struct SessionObserverHandle { session_id: String, token: u64, state: Arc<ObserverState> }
impl SessionObserver {
    pub fn new(store: HarnessStore, session_id: impl Into<String>) -> AgentResult<Self> { let session_id = session_id.into(); store.load_session(&session_id)?; Ok(Self { store, session_id, state: Arc::new(ObserverState::default()) }) }
    pub fn session_id(&self) -> &str { &self.session_id }
    pub fn subscribe(&self) -> SessionObserverHandle { let token = self.state.next.fetch_add(1, Ordering::Relaxed) + 1; *self.state.active.lock().unwrap() = Some(token); SessionObserverHandle { session_id: self.session_id.clone(), token, state: Arc::clone(&self.state) } }
    pub fn unsubscribe(&self, handle: &SessionObserverHandle) { handle.unsubscribe(); }
    pub fn poll(&self, handle: &SessionObserverHandle, since: u64, limits: SessionEventLimits) -> AgentResult<EventPage> { if handle.session_id != self.session_id || *self.state.active.lock().unwrap() != Some(handle.token) { return Err(AgentError::new("observer is inactive or belongs to another session")); } self.poll_events(since, limits) }
    pub fn poll_events(&self, since: u64, limits: SessionEventLimits) -> AgentResult<EventPage> { let session = self.store.load_session(&self.session_id)?; event_page(&session, since, limits.validate()?) }
}
impl SessionObserverHandle { pub fn unsubscribe(&self) { let mut active = self.state.active.lock().unwrap(); if *active == Some(self.token) { *active = None; } } pub fn is_active(&self) -> bool { *self.state.active.lock().unwrap() == Some(self.token) } }
impl Drop for SessionObserverHandle { fn drop(&mut self) { self.unsubscribe(); } }

impl HarnessStore {
    pub fn inspect_session_view(&self, session_id: &str) -> AgentResult<SessionInspection> { Ok(session_projection(&self.load_session(session_id)?)) }
    pub fn inspect_tasks(&self, session_id: &str) -> AgentResult<TasksInspection> { self.load_session(session_id)?; Ok(TasksInspection { session_id: session_id.into(), tasks: self.list_tasks(session_id)? }) }
    pub fn inspect_governance(&self, session_id: &str) -> AgentResult<GovernanceInspection> { let session = self.load_session(session_id)?; let record = session.orchestration().ok_or_else(|| AgentError::new("session has no governance record"))?; Ok(GovernanceInspection { session_id: session_id.into(), state: record.state, codegraph_binding: record.codegraph_binding.clone(), agents: record.agents.clone(), tasks: record.tasks.clone(), authority: "read_only_observation" }) }
    pub fn inspect_events(&self, session_id: &str, since: u64) -> AgentResult<EventPage> { self.poll_events(session_id, since, SessionEventLimits::default()) }
    pub fn poll_events(&self, session_id: &str, since: u64, limits: SessionEventLimits) -> AgentResult<EventPage> { let session = self.load_session(session_id)?; event_page(&session, since, limits.validate()?) }
}
fn session_projection(session: &HarnessSession) -> SessionInspection { SessionInspection { session_id: session.id.clone(), project: session.project.clone(), revision: session.revision, interrupted: session.interrupted, turn_count: session.turns.len(), event_cursor: session.events.len() as u64, summary: session.summary.clone(), title: session.title.clone() } }
fn event_page(session: &HarnessSession, since: u64, limits: SessionEventLimits) -> AgentResult<EventPage> {
    let total = session.events.len() as u64; if since > total { return Err(AgentError::new("event cursor is beyond the session event stream")); }
    let mut events = Vec::new(); let mut bytes = 0; let mut cursor = since as usize;
    while cursor < session.events.len() && events.len() < limits.max_events { let event = project_event(&session.events[cursor], cursor as u64); let size = serde_json::to_vec(&event)?.len(); if size > MAX_EVENT_BYTES { return Err(AgentError::new("projected event exceeds the event byte limit")); } if !events.is_empty() && bytes + size > limits.max_bytes { break; } if events.is_empty() && size > limits.max_bytes { return Err(AgentError::new("page byte limit is smaller than one event")); } bytes += size; events.push(event); cursor += 1; }
    Ok(EventPage { session_id: session.id.clone(), since, cursor: cursor as u64, events, has_more: cursor < session.events.len(), event_limit: limits.max_events, byte_limit: limits.max_bytes })
}
fn project_event(event: &Value, cursor: u64) -> SessionEvent {
    let text = |key: &str| event.get(key).and_then(Value::as_str).map(|s| s.chars().take(256).collect());
    let receipt = event.get("receipt").and_then(Value::as_object).map(|r| SessionEventReceipt { operation: r.get("operation").and_then(Value::as_str).map(str::to_owned), status: r.get("status").and_then(Value::as_str).map(str::to_owned), path: r.get("path").and_then(Value::as_str).map(|s| s.chars().take(256).collect()), before_fingerprint: r.get("beforeFingerprint").and_then(Value::as_str).map(str::to_owned), after_fingerprint: r.get("afterFingerprint").and_then(Value::as_str).map(str::to_owned), codegraph_binding: r.get("codegraphBinding").cloned().and_then(|v| serde_json::from_value(v).ok()) });
    SessionEvent { cursor, kind: text("event").unwrap_or_else(|| "unknown".into()), request_id: text("requestId"), provider: text("provider"), model: text("model"), role: text("role"), status: text("status"), task_id: text("taskId"), worker_id: text("workerId"), call_id: text("call_id"), receipt }
}
