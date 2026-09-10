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
