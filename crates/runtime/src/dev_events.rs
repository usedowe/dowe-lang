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
            message: message.map(Into::into),
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
            vec!["src/main.dowe".to_string()],
        );

        let event = receiver.recv().await.expect("event");
        let serialized = serde_json::to_string(&event).expect("json");

        assert!(serialized.contains(r#""sessionId":"test""#));
        assert!(serialized.contains(r#""type":"change_detected""#));
        assert!(serialized.contains(r#""target":"server""#));
        assert!(serialized.contains(r#""src/main.dowe""#));
    }
}
