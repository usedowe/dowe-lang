use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct DevEventBus {
    session_id: String,
    sender: broadcast::Sender<DevEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevEvent {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "type")]
    pub event_type: DevEventType,
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub message: Option<String>,
    pub paths: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevEventType {
    WatchReady,
    ChangeDetected,
    RebuildStarted,
    RebuildSucceeded,
    RebuildFailed,
    TargetRestarting,
    TargetReady,
    ModuleBuildStarted,
    ModuleBuildFailed,
    ModuleUpdate,
    ModuleApplied,
    Reload,
    Shutdown,
}

impl DevEventBus {
    pub fn new(session_id: impl Into<String>) -> Self {
        let (sender, _) = broadcast::channel(256);
        Self {
            session_id: session_id.into(),
            sender,
        }
    }

    pub fn emit(
        &self,
        event_type: DevEventType,
        target: Option<impl Into<String>>,
        message: Option<impl Into<String>>,
        paths: Vec<String>,
    ) {
        let event = DevEvent {
            session_id: self.session_id.clone(),
            event_type,
            target: target.map(Into::into),
            version: None,
            message: message.map(Into::into),
            paths,
        };
        let _ = self.sender.send(event);
    }

    pub fn emit_module_update(
        &self,
        target: impl Into<String>,
        version: impl Into<String>,
        paths: Vec<String>,
    ) {
        let event = DevEvent {
            session_id: self.session_id.clone(),
            event_type: DevEventType::ModuleUpdate,
            target: Some(target.into()),
            version: Some(version.into()),
            message: None,
            paths,
        };
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DevEvent> {
        self.sender.subscribe()
    }
}

impl Default for DevEventBus {
    fn default() -> Self {
        Self::new("dev")
    }
}

#[cfg(test)]
mod tests {
    use super::{DevEventBus, DevEventType};

    #[tokio::test]
    async fn emits_serializable_events() {
        let bus = DevEventBus::new("test");
        let mut receiver = bus.subscribe();

        bus.emit(
            DevEventType::ChangeDetected,
            Some("server"),
            Some("changed"),
            vec!["main.dowe".to_string()],
        );

        let event = receiver.recv().await.expect("event");
        let serialized = serde_json::to_string(&event).expect("json");

        assert!(serialized.contains(r#""sessionId":"test""#));
        assert!(serialized.contains(r#""type":"change_detected""#));
        assert!(serialized.contains(r#""target":"server""#));
        assert!(serialized.contains(r#""main.dowe""#));
        assert!(!serialized.contains(r#""version""#));
    }

    #[tokio::test]
    async fn emits_versioned_module_updates() {
        let bus = DevEventBus::new("test");
        let mut receiver = bus.subscribe();

        bus.emit_module_update("web", "abc123", vec!["pages/index.dowe".to_string()]);

        let event = receiver.recv().await.expect("event");
        let serialized = serde_json::to_string(&event).expect("json");

        assert!(serialized.contains(r#""type":"module_update""#));
        assert!(serialized.contains(r#""version":"abc123""#));
    }
}
