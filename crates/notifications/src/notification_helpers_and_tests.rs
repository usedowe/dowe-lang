fn retry_delay(attempt: u32) -> u64 {
    (1_u64 << attempt.saturating_sub(1).min(16)).min(3600)
}

fn stable_id(key: &str, payload: &NotificationPayload) -> NotificationResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.update(payload.canonical_bytes()?);
    Ok(format!("n_{}", hex(&hasher.finalize()[..16])))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn storage_error(error: impl std::fmt::Display) -> NotificationError {
    NotificationError::Storage(error.to_string())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> NotificationResult<()> {
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, bytes).map_err(storage_error)?;
    fs::rename(&temp, path).map_err(storage_error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub local: bool,
    pub remote: bool,
    pub requires_user_process: bool,
}

pub fn platform_capabilities(platform: NotificationPlatform) -> PlatformCapabilities {
    match platform {
        NotificationPlatform::Web => PlatformCapabilities {
            local: true,
            remote: true,
            requires_user_process: false,
        },
        NotificationPlatform::Ios | NotificationPlatform::Android | NotificationPlatform::Macos => {
            PlatformCapabilities {
                local: true,
                remote: true,
                requires_user_process: false,
            }
        }
        NotificationPlatform::Windows => PlatformCapabilities {
            local: true,
            remote: true,
            requires_user_process: false,
        },
        NotificationPlatform::Linux => PlatformCapabilities {
            local: true,
            remote: true,
            requires_user_process: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn installation() -> Installation {
        Installation {
            id: "device-1".into(),
            app: "demo".into(),
            environment: "dev".into(),
            tenant: "tenant".into(),
            user: "user".into(),
            platform: NotificationPlatform::Web,
            provider: NotificationProvider::WebPush,
            token: "subscription".into(),
            registration_version: 0,
            active: true,
            preferences: BTreeMap::new(),
            updated_at: 1,
        }
    }

    fn payload() -> NotificationPayload {
        NotificationPayload {
            id: "message-1".into(),
            title: "New message".into(),
            body: "Hello".into(),
            route: Some("/chat/1".into()),
            data: serde_json::json!({"chatId":"1"}),
            category: "chat".into(),
            tag: None,
            expires_at: None,
        }
    }

    #[test]
    fn register_enqueue_and_claim_are_durable_and_idempotent() {
        let root = TempDir::new().unwrap();
        let store = NotificationStore::open(root.path(), "demo").unwrap();
        store.register(installation()).unwrap();
        let first = store
            .enqueue("chat:1", "demo", "dev", "tenant", payload(), "user")
            .unwrap();
        let second = store
            .enqueue("chat:1", "demo", "dev", "tenant", payload(), "user")
            .unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(first.deliveries.len(), 1);
        let claimed = store.claim("worker", now_seconds(), 30).unwrap().unwrap();
        assert_eq!(claimed.status, DeliveryStatus::Attempting);
        assert!(claimed.payload.expires_at.is_some());
        assert_eq!(store.deliveries().unwrap().len(), 1);
        let reopened = NotificationStore::open(root.path(), "demo").unwrap();
        assert_eq!(reopened.deliveries().unwrap()[0].id, claimed.id);
    }

    #[test]
    fn rejects_external_routes_and_provider_mismatches() {
        let mut value = payload();
        value.route = Some("https://example.com".into());
        assert!(value.validate().is_err());
        let mut device = installation();
        device.provider = NotificationProvider::Fcm;
        assert!(device.validate().is_err());
        let mut mac = installation();
        mac.platform = NotificationPlatform::Macos;
        mac.provider = NotificationProvider::Apns;
        assert!(mac.validate().is_ok());
    }
}
