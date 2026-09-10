#[cfg(test)]
mod tests {
    use super::*;
    use dowe_notifications::{NotificationPlatform, NotificationProvider};
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[test]
    fn escapes_wns_xml() {
        assert_eq!(xml_escape("a<&>\"'"), "a&lt;&amp;&gt;&quot;&apos;");
    }

    #[test]
    fn linux_receiver_payload_is_validated_before_presentation() {
        let payload = NotificationPayload {
            id: "id".into(),
            title: "title".into(),
            body: "body".into(),
            route: None,
            data: Value::Null,
            category: "process".into(),
            tag: None,
            expires_at: None,
        };
        let envelope = serde_json::to_string(&json!({
            "type":"notification",
            "deliveryId":"delivery",
            "payload":payload,
        }))
        .unwrap();
        let parsed: LinuxEnvelope = serde_json::from_str(&envelope).unwrap();
        assert_eq!(parsed.kind, "notification");
        assert!(parsed.payload.unwrap().validate().is_ok());
    }

    #[tokio::test]
    async fn unconfigured_provider_is_retried_without_losing_outbox() {
        let root = TempDir::new().unwrap();
        let store = NotificationStore::open(root.path(), "demo").unwrap();
        store
            .register(Installation {
                id: "android-1".into(),
                app: "demo".into(),
                environment: "development".into(),
                tenant: "default".into(),
                user: "user".into(),
                platform: NotificationPlatform::Android,
                provider: NotificationProvider::Fcm,
                token: "token".into(),
                registration_version: 0,
                active: true,
                preferences: BTreeMap::new(),
                updated_at: 1,
            })
            .unwrap();
        store
            .enqueue(
                "idempotency",
                "demo",
                "development",
                "default",
                NotificationPayload {
                    id: "id".into(),
                    title: "title".into(),
                    body: "body".into(),
                    route: None,
                    data: Value::Null,
                    category: "process".into(),
                    tag: None,
                    expires_at: None,
                },
                "user",
            )
            .unwrap();
        let report = dispatch_pending_notifications(
            &store,
            &NotificationDispatcherConfig::default(),
            "test-worker",
            1,
        )
        .await;
        assert_eq!(report.retried, 1);
        assert_eq!(store.deliveries().unwrap()[0].attempts, 1);
    }
}

