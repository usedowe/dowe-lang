//! Shared notification contracts and a small durable outbox.
//!
//! Provider adapters belong to the runtime and native hosts. This crate owns the
//! values crossing those boundaries so local and remote notifications cannot
//! silently diverge.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const MAX_TITLE_BYTES: usize = 256;
const MAX_BODY_BYTES: usize = 4096;
const MAX_DATA_BYTES: usize = 16 * 1024;
const MAX_ROUTE_BYTES: usize = 1024;
const MAX_NAMESPACE_BYTES: usize = 128;
const MAX_TOKEN_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPlatform {
    Web,
    Ios,
    Android,
    Macos,
    Windows,
    Linux,
}

impl NotificationPlatform {
    pub const ALL: [Self; 6] = [
        Self::Web,
        Self::Ios,
        Self::Android,
        Self::Macos,
        Self::Windows,
        Self::Linux,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationProvider {
    WebPush,
    Apns,
    Fcm,
    Wns,
    DoweLinux,
}

impl NotificationProvider {
    pub fn platform(self) -> NotificationPlatform {
        match self {
            Self::WebPush => NotificationPlatform::Web,
            Self::Apns => NotificationPlatform::Ios,
            Self::Fcm => NotificationPlatform::Android,
            Self::Wns => NotificationPlatform::Windows,
            Self::DoweLinux => NotificationPlatform::Linux,
        }
    }

    pub fn supports_platform(self, platform: NotificationPlatform) -> bool {
        match self {
            Self::Apns => matches!(
                platform,
                NotificationPlatform::Ios | NotificationPlatform::Macos
            ),
            _ => self.platform() == platform,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub id: String,
    pub title: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default)]
    pub data: Value,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

fn default_category() -> String {
    "process".to_string()
}

impl NotificationPayload {
    pub fn validate(&self) -> Result<(), NotificationError> {
        validate_text("id", &self.id, 128)?;
        validate_text("title", &self.title, MAX_TITLE_BYTES)?;
        validate_text("body", &self.body, MAX_BODY_BYTES)?;
        validate_text("category", &self.category, 64)?;
        if self.category != "process" && self.category != "chat" {
            return Err(NotificationError::InvalidPayload(
                "category must be `process` or `chat`".to_string(),
            ));
        }
        if let Some(route) = &self.route {
            validate_text("route", route, MAX_ROUTE_BYTES)?;
            if !route.starts_with('/') || route.starts_with("//") || route.contains('\n') {
                return Err(NotificationError::InvalidPayload(
                    "route must be an internal absolute path".to_string(),
                ));
            }
        }
        if let Some(tag) = &self.tag {
            validate_text("tag", tag, 128)?;
        }
        let data_size = serde_json::to_vec(&self.data)
            .map_err(|error| NotificationError::InvalidPayload(error.to_string()))?
            .len();
        if data_size > MAX_DATA_BYTES {
            return Err(NotificationError::PayloadTooLarge("data"));
        }
        if self.expires_at.is_some_and(|value| value <= now_seconds()) {
            return Err(NotificationError::InvalidPayload(
                "expires_at must be in the future".to_string(),
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NotificationError> {
        self.validate()?;
        serde_json::to_vec(self)
            .map_err(|error| NotificationError::InvalidPayload(error.to_string()))
    }
}

fn validate_text(name: &'static str, value: &str, max: usize) -> Result<(), NotificationError> {
    if value.trim().is_empty() {
        return Err(NotificationError::InvalidPayload(format!(
            "{name} must not be empty"
        )));
    }
    if value.as_bytes().len() > max {
        return Err(NotificationError::PayloadTooLarge(name));
    }
    if value.chars().any(char::is_control) {
        return Err(NotificationError::InvalidPayload(format!(
            "{name} contains control characters"
        )));
    }
    if matches!(
        name,
        "namespace" | "app" | "environment" | "tenant" | "user"
    ) && (value.contains('/') || value.contains('\\') || value == "." || value == "..")
    {
        return Err(NotificationError::InvalidPayload(format!(
            "{name} contains a path separator"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installation {
    pub id: String,
    pub app: String,
    pub environment: String,
    pub tenant: String,
    pub user: String,
    pub platform: NotificationPlatform,
    pub provider: NotificationProvider,
    pub token: String,
    pub registration_version: u64,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub preferences: BTreeMap<String, bool>,
    pub updated_at: u64,
}

impl Installation {
    pub fn validate(&self) -> Result<(), NotificationError> {
        for (name, value) in [
            ("id", self.id.as_str()),
            ("app", self.app.as_str()),
            ("environment", self.environment.as_str()),
            ("tenant", self.tenant.as_str()),
            ("user", self.user.as_str()),
        ] {
            validate_text(name, value, MAX_NAMESPACE_BYTES)?;
        }
        validate_text("token", &self.token, MAX_TOKEN_BYTES)?;
        if !self.provider.supports_platform(self.platform) {
            return Err(NotificationError::InvalidPayload(
                "provider does not belong to platform".to_string(),
            ));
        }
        Ok(())
    }

    pub fn accepts(&self, category: &str) -> bool {
        self.active && self.preferences.get(category).copied().unwrap_or(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeliveryStatus {
    Queued,
    Attempting,
    RetryScheduled,
    ProviderAccepted,
    Expired,
    Revoked,
    Failed,
    Suppressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub notification_id: String,
    pub installation_id: String,
    pub registration_version: u64,
    pub payload: NotificationPayload,
    pub status: DeliveryStatus,
    pub attempts: u32,
    pub next_attempt_at: u64,
    #[serde(default)]
    pub lease_until: Option<u64>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationIntent {
    pub id: String,
    pub idempotency_key: String,
    pub app: String,
    pub environment: String,
    pub tenant: String,
    pub payload: NotificationPayload,
    pub created_at: u64,
    pub expires_at: u64,
    pub deliveries: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedState {
    installations: Vec<Installation>,
    intents: Vec<NotificationIntent>,
    deliveries: Vec<Delivery>,
}

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("invalid notification payload: {0}")]
    InvalidPayload(String),
    #[error("notification field `{0}` exceeds its limit")]
    PayloadTooLarge(&'static str),
    #[error("notification installation is not authorized")]
    Unauthorized,
    #[error("notification idempotency key conflicts with an existing intent")]
    IdempotencyConflict,
    #[error("notification storage failed: {0}")]
    Storage(String),
    #[error("notification lease is not available")]
    LeaseUnavailable,
    #[error("notification delivery is not found")]
    NotFound,
}

pub type NotificationResult<T> = Result<T, NotificationError>;

pub struct NotificationStore {
    path: PathBuf,
}

impl NotificationStore {
    pub fn open(root: impl AsRef<Path>, namespace: &str) -> NotificationResult<Self> {
        validate_text("namespace", namespace, MAX_NAMESPACE_BYTES)?;
        let base = root.as_ref().join(".dowe").join("notifications");
        fs::create_dir_all(&base).map_err(storage_error)?;
        let path = base.join(format!("{namespace}.json"));
        if !path.exists() {
            atomic_write(
                &path,
                br#"{"installations":[],"intents":[],"deliveries":[]}"#,
            )?;
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn register(&self, mut installation: Installation) -> NotificationResult<Installation> {
        installation.validate()?;
        let mut state = self.read()?;
        if let Some(existing) = state
            .installations
            .iter_mut()
            .find(|item| item.id == installation.id)
        {
            if existing.app != installation.app
                || existing.environment != installation.environment
                || existing.tenant != installation.tenant
                || existing.user != installation.user
            {
                return Err(NotificationError::Unauthorized);
            }
            installation.registration_version = existing.registration_version.saturating_add(1);
            *existing = installation.clone();
        } else {
            if installation.registration_version == 0 {
                installation.registration_version = 1;
            }
            state.installations.push(installation.clone());
        }
        self.write(&state)?;
        Ok(installation)
    }

    pub fn revoke(&self, installation_id: &str) -> NotificationResult<()> {
        let mut state = self.read()?;
        let Some(installation) = state
            .installations
            .iter_mut()
            .find(|item| item.id == installation_id)
        else {
            return Err(NotificationError::NotFound);
        };
        installation.active = false;
        installation.registration_version = installation.registration_version.saturating_add(1);
        for delivery in &mut state.deliveries {
            if delivery.installation_id == installation_id
                && matches!(
                    delivery.status,
                    DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
                )
            {
                delivery.status = DeliveryStatus::Revoked;
            }
        }
        self.write(&state)
    }

    /// Return an installation snapshot for a claimed delivery.
    pub fn installation(&self, installation_id: &str) -> NotificationResult<Installation> {
        self.read()?
            .installations
            .into_iter()
            .find(|item| item.id == installation_id)
            .ok_or(NotificationError::NotFound)
    }

    /// Disable an installation after a provider reports that its token is gone.
    pub fn revoke_delivery_installation(&self, delivery_id: &str) -> NotificationResult<()> {
        let mut state = self.read()?;
        let installation_id = state
            .deliveries
            .iter()
            .find(|delivery| delivery.id == delivery_id)
            .map(|delivery| delivery.installation_id.clone())
            .ok_or(NotificationError::NotFound)?;
        if let Some(installation) = state
            .installations
            .iter_mut()
            .find(|installation| installation.id == installation_id)
        {
            installation.active = false;
            installation.registration_version = installation.registration_version.saturating_add(1);
        }
        if let Some(delivery) = state
            .deliveries
            .iter_mut()
            .find(|delivery| delivery.id == delivery_id)
        {
            delivery.status = DeliveryStatus::Revoked;
            delivery.lease_until = None;
            delivery.last_error = Some("provider_token_invalid".to_string());
        }
        self.write(&state)
    }

    pub fn enqueue(
        &self,
        idempotency_key: &str,
        app: &str,
        environment: &str,
        tenant: &str,
        mut payload: NotificationPayload,
        user: &str,
    ) -> NotificationResult<NotificationIntent> {
        payload.validate()?;
        validate_text("idempotency_key", idempotency_key, 256)?;
        let mut state = self.read()?;
        if let Some(existing) = state
            .intents
            .iter()
            .find(|item| item.idempotency_key == idempotency_key)
        {
            let mut comparable = payload.clone();
            if comparable.expires_at.is_none() {
                comparable.expires_at = existing.payload.expires_at;
            }
            if existing.payload != comparable
                || existing.app != app
                || existing.environment != environment
                || existing.tenant != tenant
            {
                return Err(NotificationError::IdempotencyConflict);
            }
            return Ok(existing.clone());
        }
        let now = now_seconds();
        let expires_at = payload.expires_at.unwrap_or(now.saturating_add(86_400));
        payload.expires_at = Some(expires_at);
        let id = stable_id(idempotency_key, &payload)?;
        let mut delivery_ids = Vec::new();
        for installation in state.installations.iter().filter(|item| {
            item.app == app
                && item.environment == environment
                && item.tenant == tenant
                && item.user == user
                && item.accepts(&payload.category)
        }) {
            let delivery_id = format!(
                "{id}:{}:{}",
                installation.id, installation.registration_version
            );
            delivery_ids.push(delivery_id.clone());
            state.deliveries.push(Delivery {
                id: delivery_id,
                notification_id: id.clone(),
                installation_id: installation.id.clone(),
                registration_version: installation.registration_version,
                payload: payload.clone(),
                status: DeliveryStatus::Queued,
                attempts: 0,
                next_attempt_at: now,
                lease_until: None,
                provider_id: None,
                last_error: None,
            });
        }
        let intent = NotificationIntent {
            id,
            idempotency_key: idempotency_key.to_string(),
            app: app.to_string(),
            environment: environment.to_string(),
            tenant: tenant.to_string(),
            payload,
            created_at: now,
            expires_at,
            deliveries: delivery_ids,
        };
        state.intents.push(intent.clone());
        self.write(&state)?;
        Ok(intent)
    }

    pub fn claim(
        &self,
        worker: &str,
        now: u64,
        lease_seconds: u64,
    ) -> NotificationResult<Option<Delivery>> {
        validate_text("worker", worker, 128)?;
        let mut state = self.read()?;
        let mut expired_any = false;
        for delivery in &mut state.deliveries {
            if matches!(
                delivery.status,
                DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
            ) && delivery
                .payload
                .expires_at
                .is_some_and(|until| until <= now)
            {
                delivery.status = DeliveryStatus::Expired;
                delivery.lease_until = None;
                expired_any = true;
            }
        }
        let Some(index) = state.deliveries.iter().position(|delivery| {
            matches!(
                delivery.status,
                DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
            ) && delivery.next_attempt_at <= now
                && delivery.lease_until.is_none_or(|until| until <= now)
                && delivery.payload.expires_at.is_none_or(|until| until > now)
        }) else {
            if expired_any {
                self.write(&state)?;
            }
            return Ok(None);
        };
        let delivery = &mut state.deliveries[index];
        delivery.status = DeliveryStatus::Attempting;
        delivery.attempts = delivery.attempts.saturating_add(1);
        delivery.lease_until = Some(now.saturating_add(lease_seconds));
        delivery.last_error = Some(format!("lease:{worker}"));
        let claimed = delivery.clone();
        self.write(&state)?;
        Ok(Some(claimed))
    }

    pub fn mark_provider_accepted(
        &self,
        delivery_id: &str,
        provider_id: &str,
    ) -> NotificationResult<()> {
        let mut state = self.read()?;
        let delivery = state
            .deliveries
            .iter_mut()
            .find(|item| item.id == delivery_id)
            .ok_or(NotificationError::NotFound)?;
        delivery.status = DeliveryStatus::ProviderAccepted;
        delivery.provider_id = Some(provider_id.to_string());
        delivery.lease_until = None;
        delivery.last_error = None;
        self.write(&state)
    }

    pub fn mark_retry(&self, delivery_id: &str, error: &str, now: u64) -> NotificationResult<()> {
        let mut state = self.read()?;
        let delivery = state
            .deliveries
            .iter_mut()
            .find(|item| item.id == delivery_id)
            .ok_or(NotificationError::NotFound)?;
        delivery.status = if delivery.attempts >= 10 {
            DeliveryStatus::Failed
        } else {
            DeliveryStatus::RetryScheduled
        };
        delivery.next_attempt_at = now.saturating_add(retry_delay(delivery.attempts));
        delivery.lease_until = None;
        delivery.last_error = Some(error.to_string());
        self.write(&state)
    }

    pub fn deliveries(&self) -> NotificationResult<Vec<Delivery>> {
        Ok(self.read()?.deliveries)
    }

    fn read(&self) -> NotificationResult<PersistedState> {
        let bytes = fs::read(&self.path).map_err(storage_error)?;
        serde_json::from_slice(&bytes).map_err(storage_error)
    }

    fn write(&self, state: &PersistedState) -> NotificationResult<()> {
        let bytes = serde_json::to_vec_pretty(state).map_err(storage_error)?;
        atomic_write(&self.path, &bytes)
    }
}

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
