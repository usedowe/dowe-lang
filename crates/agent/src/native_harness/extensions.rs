//! Compiled, read-only session extension tools.
//!
//! This module intentionally contains no dynamic dispatch or loading boundary: every
//! tool is a fixed projection over `SessionObserver`.
use super::{SessionEventLimits, SessionObserver, SessionObserverHandle};
use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const MAX_EXTENSION_TOOLS: usize = 16;
pub const MAX_EXTENSION_CALLS: usize = 64;
pub const MAX_EXTENSION_REQUEST_BYTES: usize = 8 * 1024;
pub const MAX_EXTENSION_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtensionToolDescriptor {
    pub id: String,
    pub version: String,
    pub capability: String,
    pub input_limit: usize,
    pub output_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtensionCall {
    pub session_id: String,
    pub tool_id: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtensionResult {
    pub session_id: String,
    pub tool_id: String,
    pub output: Value,
}

pub struct SessionExtensionRegistry {
    observer: SessionObserver,
    handle: SessionObserverHandle,
    closed: AtomicBool,
    calls: AtomicUsize,
}

impl SessionExtensionRegistry {
    pub fn new(observer: SessionObserver, handle: SessionObserverHandle) -> AgentResult<Self> {
        validate_session_extension_descriptors(&fixed_descriptors())?;
        Ok(Self { observer, handle, closed: AtomicBool::new(false), calls: AtomicUsize::new(0) })
    }

    pub fn descriptors(&self) -> Vec<SessionExtensionToolDescriptor> { fixed_descriptors() }
    pub fn fingerprint(&self) -> String { descriptor_fingerprint(&fixed_descriptors()) }
    pub fn session_id(&self) -> &str { self.observer.session_id() }
    pub fn close(&self) { self.closed.store(true, Ordering::Release); }

    pub fn call(&self, request: SessionExtensionCall) -> AgentResult<SessionExtensionResult> {
        if self.closed.load(Ordering::Acquire) { return Err(AgentError::new("extension registry is closed")); }
        if request.session_id != self.observer.session_id() { return Err(AgentError::new("extension call belongs to another session")); }
        let encoded = serde_json::to_vec(&request.arguments)?;
        if encoded.len() > MAX_EXTENSION_REQUEST_BYTES { return Err(AgentError::new("extension request exceeds the byte limit")); }
        let call = self.calls.fetch_add(1, Ordering::AcqRel) + 1;
        if call > MAX_EXTENSION_CALLS { return Err(AgentError::new("extension call limit exceeded")); }
        if !self.handle.is_active() { return Err(AgentError::new("observer is inactive or unsubscribed")); }
        let descriptor = fixed_descriptors().into_iter().find(|tool| tool.id == request.tool_id)
            .ok_or_else(|| AgentError::new("unknown session extension tool"))?;
        if encoded.len() > descriptor.input_limit { return Err(AgentError::new("extension arguments exceed the tool input limit")); }
        let output = match descriptor.id.as_str() {
            "events.poll" => poll_events(&self.observer, &self.handle, &request.arguments),
            _ => unreachable!("validated fixed descriptor")
        }?;
        let output_bytes = serde_json::to_vec(&output)?;
        if output_bytes.len() > descriptor.output_limit || output_bytes.len() > MAX_EXTENSION_RESPONSE_BYTES {
            return Err(AgentError::new("extension response exceeds the byte limit"));
        }
        Ok(SessionExtensionResult { session_id: request.session_id, tool_id: request.tool_id, output })
    }
}

impl SessionObserver {
    pub fn create_extension_registry(&self, handle: SessionObserverHandle) -> AgentResult<SessionExtensionRegistry> {
        SessionExtensionRegistry::new(self.clone(), handle)
    }
}

fn fixed_descriptors() -> Vec<SessionExtensionToolDescriptor> {
    vec![descriptor("events.poll", "1", "session.events.read", MAX_EXTENSION_REQUEST_BYTES, MAX_EXTENSION_RESPONSE_BYTES)]
}
fn descriptor(id: &str, version: &str, capability: &str, input_limit: usize, output_limit: usize) -> SessionExtensionToolDescriptor {
    SessionExtensionToolDescriptor { id: id.into(), version: version.into(), capability: capability.into(), input_limit, output_limit }
}
pub fn validate_session_extension_descriptors(descriptors: &[SessionExtensionToolDescriptor]) -> AgentResult<()> {
    if descriptors.is_empty() || descriptors.len() > MAX_EXTENSION_TOOLS { return Err(AgentError::new("extension registration count is outside the allowed range")); }
    let mut ids = std::collections::BTreeSet::new();
    for descriptor in descriptors {
        if descriptor.id.is_empty() || !ids.insert(&descriptor.id) { return Err(AgentError::new("extension tool IDs must be unique and non-empty")); }
        if descriptor.version.is_empty() || descriptor.capability.is_empty() || descriptor.input_limit == 0 || descriptor.output_limit == 0 { return Err(AgentError::new("extension descriptor is invalid")); }
    }
    Ok(())
}
fn descriptor_fingerprint(descriptors: &[SessionExtensionToolDescriptor]) -> String {
    let mut material: Vec<String> = descriptors.iter().map(|d| format!("{}\0{}\0{}\0{}\0{}", d.id, d.version, d.capability, d.input_limit, d.output_limit)).collect();
    material.sort();
    let mut hasher = Sha256::new();
    hasher.update(material.join("\n"));
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
fn poll_events(observer: &SessionObserver, handle: &SessionObserverHandle, args: &Value) -> AgentResult<Value> {
    let object = args.as_object().ok_or_else(|| AgentError::new("events.poll arguments must be an object"))?;
    let since = object.get("since").and_then(Value::as_u64).ok_or_else(|| AgentError::new("events.poll requires an integer since"))?;
    let limits = SessionEventLimits { max_events: object.get("maxEvents").and_then(Value::as_u64).unwrap_or(super::MAX_EVENTS as u64) as usize, max_bytes: object.get("maxBytes").and_then(Value::as_u64).unwrap_or(super::MAX_PAGE_BYTES as u64) as usize };
    Ok(serde_json::to_value(observer.poll(handle, since, limits)?)?)
}

pub fn session_extension_descriptor_fingerprint() -> String { descriptor_fingerprint(&fixed_descriptors()) }

pub fn session_extension_fingerprint(descriptors: &[SessionExtensionToolDescriptor]) -> AgentResult<String> {
    validate_session_extension_descriptors(descriptors)?;
    Ok(descriptor_fingerprint(descriptors))
}

