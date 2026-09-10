//! Authenticated notification provider dispatch.
//!
//! The outbox remains the source of truth. This module only claims deliveries,
//! sends them through the provider owned by the installation, and records the
//! provider result. Credentials are read from the environment and never enter
//! a notification payload or a generated client artifact.

use crate::{RuntimeError, RuntimeResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use dowe_notifications::{
    Delivery, Installation, NotificationPayload, NotificationProvider, NotificationStore,
};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use web_push_native::jwt_simple::algorithms::ES256KeyPair;
use web_push_native::p256::PublicKey;
use web_push_native::{Auth, WebPushBuilder};

const DEFAULT_APNS_ENDPOINT: &str = "https://api.push.apple.com";
const DEFAULT_FCM_ENDPOINT: &str = "https://fcm.googleapis.com";
const DEFAULT_WNS_ENDPOINT: &str = "https://login.microsoftonline.com";

#[derive(Clone, Debug, Default)]
pub struct NotificationDispatcherConfig {
    pub web_push: Option<WebPushConfig>,
    pub apns: Option<ApnsConfig>,
    pub fcm: Option<FcmConfig>,
    pub wns: Option<WnsConfig>,
    pub linux: Option<LinuxDispatcherConfig>,
}

#[derive(Clone, Debug)]
pub struct WebPushConfig {
    pub vapid_subject: String,
    pub vapid_private_key: String,
}

#[derive(Clone, Debug)]
pub struct ApnsConfig {
    pub key_id: String,
    pub team_id: String,
    pub private_key: String,
    pub topic: String,
    pub production: bool,
    pub endpoint: String,
}

#[derive(Clone, Debug)]
pub struct FcmConfig {
    pub project_id: String,
    pub client_email: String,
    pub private_key: String,
    pub endpoint: String,
}

#[derive(Clone, Debug)]
pub struct WnsConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
    pub endpoint: String,
}

#[derive(Clone, Debug)]
pub struct LinuxDispatcherConfig {
    pub connect_timeout: Duration,
}

impl NotificationDispatcherConfig {
    /// Load configured providers. Missing provider credentials are left disabled;
    /// a queued delivery will be retried only when its provider is configured.
    pub fn from_env() -> RuntimeResult<Self> {
        Ok(Self {
            web_push: env_pair("DOWE_VAPID_SUBJECT", "DOWE_VAPID_PRIVATE_KEY").map(
                |(vapid_subject, vapid_private_key)| WebPushConfig {
                    vapid_subject,
                    vapid_private_key,
                },
            ),
            apns: env_quad(
                "DOWE_APNS_KEY_ID",
                "DOWE_APNS_TEAM_ID",
                "DOWE_APNS_PRIVATE_KEY",
                "DOWE_APNS_TOPIC",
            )
            .map(|(key_id, team_id, private_key, topic)| ApnsConfig {
                key_id,
                team_id,
                private_key,
                topic,
                production: env_bool("DOWE_APNS_PRODUCTION", true),
                endpoint: std::env::var("DOWE_APNS_ENDPOINT").unwrap_or_else(|_| {
                    if env_bool("DOWE_APNS_PRODUCTION", true) {
                        DEFAULT_APNS_ENDPOINT.to_string()
                    } else {
                        "https://api.sandbox.push.apple.com".to_string()
                    }
                }),
            }),
            fcm: env_quad(
                "DOWE_FCM_PROJECT_ID",
                "DOWE_FCM_CLIENT_EMAIL",
                "DOWE_FCM_PRIVATE_KEY",
                "DOWE_FCM_ENDPOINT",
            )
            .map(
                |(project_id, client_email, private_key, endpoint)| FcmConfig {
                    project_id,
                    client_email,
                    private_key,
                    endpoint: if endpoint.is_empty() {
                        DEFAULT_FCM_ENDPOINT.to_string()
                    } else {
                        endpoint
                    },
                },
            ),
            wns: env_quad(
                "DOWE_WNS_CLIENT_ID",
                "DOWE_WNS_CLIENT_SECRET",
                "DOWE_WNS_TENANT_ID",
                "DOWE_WNS_ENDPOINT",
            )
            .map(
                |(client_id, client_secret, tenant_id, endpoint)| WnsConfig {
                    client_id,
                    client_secret,
                    tenant_id,
                    endpoint: if endpoint.is_empty() {
                        DEFAULT_WNS_ENDPOINT.to_string()
                    } else {
                        endpoint
                    },
                },
            ),
            linux: Some(LinuxDispatcherConfig {
                connect_timeout: Duration::from_secs(env_u64("DOWE_LINUX_WSS_TIMEOUT_SECONDS", 10)),
            }),
        })
    }
}

fn env_pair(first: &str, second: &str) -> Option<(String, String)> {
    Some((std::env::var(first).ok()?, std::env::var(second).ok()?))
}

fn env_quad(a: &str, b: &str, c: &str, d: &str) -> Option<(String, String, String, String)> {
    Some((
        std::env::var(a).ok()?,
        std::env::var(b).ok()?,
        std::env::var(c).ok()?,
        std::env::var(d).unwrap_or_default(),
    ))
}

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .and_then(|value| match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[derive(Debug)]
enum DispatchError {
    Retry(String),
    InvalidToken(String),
}

impl DispatchError {
    fn message(&self) -> String {
        match self {
            Self::Retry(value) | Self::InvalidToken(value) => value.clone(),
        }
    }
}


include!("notification_dispatch_core.rs");
include!("notification_provider_dispatch.rs");
include!("notification_linux.rs");
include!("notification_tests.rs");
