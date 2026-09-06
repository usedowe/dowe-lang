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

/// Dispatch up to `limit` currently eligible deliveries.
pub async fn dispatch_pending_notifications(
    store: &NotificationStore,
    config: &NotificationDispatcherConfig,
    worker: &str,
    limit: usize,
) -> NotificationDispatchReport {
    let mut report = NotificationDispatchReport::default();
    for _ in 0..limit {
        let claimed = match store.claim(worker, now_seconds(), 60) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => break,
            Err(error) => {
                report.errors.push(error.to_string());
                break;
            }
        };
        let installation = match store.installation(&claimed.installation_id) {
            Ok(value) => value,
            Err(error) => {
                report.errors.push(error.to_string());
                let _ = store.mark_retry(&claimed.id, &error.to_string(), now_seconds());
                continue;
            }
        };
        match dispatch_delivery(config, &installation, &claimed).await {
            Ok(provider_id) => {
                let _ = store.mark_provider_accepted(&claimed.id, &provider_id);
                report.accepted += 1;
            }
            Err(error @ DispatchError::InvalidToken(_)) => {
                let _ = store.revoke_delivery_installation(&claimed.id);
                report.revoked += 1;
                report.errors.push(error.message());
            }
            Err(error @ DispatchError::Retry(_)) => {
                let _ = store.mark_retry(&claimed.id, &error.message(), now_seconds());
                report.retried += 1;
                report.errors.push(error.message());
            }
        }
    }
    report
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NotificationDispatchReport {
    pub accepted: usize,
    pub retried: usize,
    pub revoked: usize,
    pub errors: Vec<String>,
}

async fn dispatch_delivery(
    config: &NotificationDispatcherConfig,
    installation: &Installation,
    delivery: &Delivery,
) -> Result<String, DispatchError> {
    match installation.provider {
        NotificationProvider::WebPush => send_web_push(
            config.web_push.as_ref().ok_or_else(|| {
                DispatchError::Retry("web push credentials are not configured".into())
            })?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "web-push".to_string()),
        NotificationProvider::Apns => send_apns(
            config.apns.as_ref().ok_or_else(|| {
                DispatchError::Retry("APNs credentials are not configured".into())
            })?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "apns".to_string()),
        NotificationProvider::Fcm => send_fcm(
            config
                .fcm
                .as_ref()
                .ok_or_else(|| DispatchError::Retry("FCM credentials are not configured".into()))?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "fcm".to_string()),
        NotificationProvider::Wns => send_wns(
            config
                .wns
                .as_ref()
                .ok_or_else(|| DispatchError::Retry("WNS credentials are not configured".into()))?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "wns".to_string()),
        NotificationProvider::DoweLinux => send_linux_wss(
            config.linux.as_ref().unwrap_or(&LinuxDispatcherConfig {
                connect_timeout: Duration::from_secs(10),
            }),
            installation,
            delivery,
        )
        .await
        .map(|()| "dowe-linux-wss".to_string()),
    }
}

async fn send_web_push(
    config: &WebPushConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let subscription: WebPushSubscription =
        serde_json::from_str(&installation.token).map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push subscription: {error}"))
        })?;
    let p256dh = URL_SAFE_NO_PAD
        .decode(subscription.keys.p256dh.as_bytes())
        .map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push p256dh: {error}"))
        })?;
    let auth = URL_SAFE_NO_PAD
        .decode(subscription.keys.auth.as_bytes())
        .map_err(|error| DispatchError::InvalidToken(format!("invalid Web Push auth: {error}")))?;
    if auth.len() != 16 {
        return Err(DispatchError::InvalidToken(
            "invalid Web Push auth length".into(),
        ));
    }
    let public_key = PublicKey::from_sec1_bytes(&p256dh).map_err(|error| {
        DispatchError::InvalidToken(format!("invalid Web Push public key: {error}"))
    })?;
    let auth = Auth::clone_from_slice(&auth);
    let private_key = URL_SAFE_NO_PAD
        .decode(config.vapid_private_key.as_bytes())
        .map_err(|error| DispatchError::Retry(format!("invalid VAPID key: {error}")))?;
    let key_pair = ES256KeyPair::from_bytes(&private_key)
        .map_err(|error| DispatchError::Retry(format!("invalid VAPID key: {error}")))?;
    let body =
        serde_json::to_vec(payload).map_err(|error| DispatchError::Retry(error.to_string()))?;
    let ttl = payload
        .expires_at
        .map(|expires| expires.saturating_sub(now_seconds()).min(u32::MAX as u64) as u32)
        .unwrap_or(86_400);
    let message = WebPushBuilder::new(
        subscription.endpoint.parse().map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push endpoint: {error}"))
        })?,
        public_key,
        auth,
    )
    .with_valid_duration(std::time::Duration::from_secs(u64::from(ttl.max(1))))
    .with_vapid(&key_pair, &config.vapid_subject)
    .build(body)
    .map_err(|error| DispatchError::Retry(error.to_string()))?;
    let mut request = Client::new().post(message.uri().to_string());
    for (name, value) in message.headers() {
        let value = value
            .to_str()
            .map_err(|error| DispatchError::Retry(format!("Web Push header: {error}")))?;
        request = request.header(name.as_str(), value);
    }
    let response = request
        .body(message.into_body())
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("Web Push request: {error}")))?;
    classify_http_response(response, "Web Push").await
}

#[derive(Debug, Deserialize)]
struct WebPushSubscription {
    endpoint: String,
    keys: WebPushSubscriptionKeys,
}

#[derive(Debug, Deserialize)]
struct WebPushSubscriptionKeys {
    p256dh: String,
    auth: String,
}

#[derive(Serialize)]
struct ApnsClaims<'a> {
    iss: &'a str,
    iat: u64,
}

async fn send_apns(
    config: &ApnsConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let header = Header {
        alg: Algorithm::ES256,
        kid: Some(config.key_id.clone()),
        ..Header::default()
    };
    let token = encode(
        &header,
        &ApnsClaims {
            iss: &config.team_id,
            iat: now_seconds(),
        },
        &EncodingKey::from_ec_pem(config.private_key.as_bytes())
            .map_err(|error| DispatchError::Retry(format!("invalid APNs key: {error}")))?,
    )
    .map_err(|error| DispatchError::Retry(format!("APNs JWT: {error}")))?;
    let body = json!({
        "aps": {
            "alert": {"title": payload.title, "body": payload.body},
            "category": payload.category,
            "thread-id": payload.tag,
            "sound": "default"
        },
        "dowe": payload,
    });
    let endpoint = format!(
        "{}/3/device/{}",
        config.endpoint.trim_end_matches('/'),
        installation.token
    );
    let response = Client::new()
        .post(endpoint)
        .header("authorization", format!("bearer {token}"))
        .header("apns-topic", &config.topic)
        .header("apns-push-type", "alert")
        .header("apns-priority", "10")
        .json(&body)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("APNs request: {error}")))?;
    classify_http_response(response, "APNs").await
}

#[derive(Serialize)]
struct ServiceAccountClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
}

async fn send_fcm(
    config: &FcmConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let now = now_seconds();
    let jwt = encode(
        &Header::new(Algorithm::RS256),
        &ServiceAccountClaims {
            iss: &config.client_email,
            scope: "https://www.googleapis.com/auth/firebase.messaging",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now.saturating_add(3600),
        },
        &EncodingKey::from_rsa_pem(config.private_key.as_bytes())
            .map_err(|error| DispatchError::Retry(format!("invalid FCM key: {error}")))?,
    )
    .map_err(|error| DispatchError::Retry(format!("FCM JWT: {error}")))?;
    let token = Client::new()
        .post("https://oauth2.googleapis.com/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", jwt.as_str()),
        ]))
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth: {error}")))?
        .error_for_status()
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth status: {error}")))?
        .json::<OAuthResponse>()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth response: {error}")))?;
    let endpoint = format!(
        "{}/v1/projects/{}/messages:send",
        config.endpoint.trim_end_matches('/'),
        config.project_id
    );
    let body = json!({
        "message": {
            "token": installation.token,
            "notification": {"title": payload.title, "body": payload.body},
            "data": {
                "dowe": serde_json::to_string(payload).map_err(|error| DispatchError::Retry(error.to_string()))?
            }
        }
    });
    let response = Client::new()
        .post(endpoint)
        .bearer_auth(token.access_token)
        .json(&body)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM request: {error}")))?;
    classify_http_response(response, "FCM").await
}

#[derive(Deserialize)]
struct WnsOAuthResponse {
    access_token: String,
}

async fn send_wns(
    config: &WnsConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let token_url = format!(
        "{}/{}/oauth2/v2.0/token",
        config.endpoint.trim_end_matches('/'),
        config.tenant_id
    );
    let access = Client::new()
        .post(token_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("grant_type", "client_credentials"),
            ("scope", "https://wns.windows.com/.default"),
        ]))
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth: {error}")))?
        .error_for_status()
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth status: {error}")))?
        .json::<WnsOAuthResponse>()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth response: {error}")))?;
    let escaped_title = xml_escape(&payload.title);
    let escaped_body = xml_escape(&payload.body);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><toast><visual><binding template=\"ToastGeneric\"><text>{escaped_title}</text><text>{escaped_body}</text></binding></visual></toast>"
    );
    let response = Client::new()
        .post(&installation.token)
        .bearer_auth(access.access_token)
        .header("Content-Type", "text/xml")
        .header("X-WNS-Type", "wns/toast")
        .header("X-WNS-RequestForStatus", "true")
        .body(xml)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS request: {error}")))?;
    classify_http_response(response, "WNS").await
}

async fn classify_http_response(
    response: reqwest::Response,
    provider: &str,
) -> Result<(), DispatchError> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    let error = format!("{provider} returned {status}: {}", truncate(&body, 512));
    if matches!(status, StatusCode::GONE | StatusCode::NOT_FOUND)
        || status == StatusCode::BAD_REQUEST
            && (body.contains("UNREGISTERED")
                || body.contains("BadDeviceToken")
                || body.contains("Unregistered"))
    {
        Err(DispatchError::InvalidToken(error))
    } else {
        Err(DispatchError::Retry(error))
    }
}

async fn send_linux_wss(
    config: &LinuxDispatcherConfig,
    installation: &Installation,
    delivery: &Delivery,
) -> Result<(), DispatchError> {
    let target: LinuxTarget = serde_json::from_str(&installation.token).map_err(|error| {
        DispatchError::InvalidToken(format!("invalid Linux receiver token: {error}"))
    })?;
    let (mut socket, _) = timeout(config.connect_timeout, connect_async(&target.url))
        .await
        .map_err(|_| DispatchError::Retry("Linux receiver connection timed out".into()))?
        .map_err(|error| DispatchError::Retry(format!("Linux receiver connection: {error}")))?;
    socket
        .send(Message::Text(
            json!({"type":"authenticate","installationId":installation.id,"token":target.token})
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| DispatchError::Retry(format!("Linux receiver auth: {error}")))?;
    socket
        .send(Message::Text(
            json!({"type":"notification","deliveryId":delivery.id,"payload":delivery.payload})
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| DispatchError::Retry(format!("Linux receiver send: {error}")))?;
    while let Some(message) = timeout(config.connect_timeout, socket.next())
        .await
        .map_err(|_| DispatchError::Retry("Linux receiver acknowledgement timed out".into()))?
        .transpose()
        .map_err(|error| DispatchError::Retry(format!("Linux receiver response: {error}")))?
    {
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text)
                .map_err(|error| DispatchError::Retry(format!("Linux receiver JSON: {error}")))?;
            if value.get("type").and_then(Value::as_str) == Some("ack")
                && value.get("deliveryId").and_then(Value::as_str) == Some(delivery.id.as_str())
            {
                return Ok(());
            }
            if value.get("type").and_then(Value::as_str) == Some("error") {
                return Err(DispatchError::Retry(
                    value
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("Linux receiver error")
                        .to_string(),
                ));
            }
        }
    }
    Err(DispatchError::Retry(
        "Linux receiver closed before acknowledgement".into(),
    ))
}

#[derive(Debug, Deserialize)]
struct LinuxTarget {
    url: String,
    token: String,
}

#[derive(Clone, Debug)]
pub struct LinuxReceiverConfig {
    pub url: String,
    pub token: String,
    pub installation_id: String,
    pub reconnect_min: Duration,
    pub reconnect_max: Duration,
}

/// Run the opt-in Linux background receiver. It reconnects with bounded
/// exponential backoff and acknowledges only after `notify-send` succeeds.
pub async fn run_linux_notification_receiver(config: LinuxReceiverConfig) -> RuntimeResult<()> {
    let mut delay = config.reconnect_min;
    loop {
        match connect_linux_receiver(&config).await {
            Ok(()) => delay = config.reconnect_min,
            Err(error) => {
                eprintln!("dowe linux notification receiver: {error}");
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(config.reconnect_max);
            }
        }
    }
}

async fn connect_linux_receiver(config: &LinuxReceiverConfig) -> RuntimeResult<()> {
    let (mut socket, _) = connect_async(&config.url)
        .await
        .map_err(|error| RuntimeError::new(format!("WSS connection failed: {error}")))?;
    socket
        .send(Message::Text(
            json!({"type":"authenticate","installationId":config.installation_id,"token":config.token})
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| RuntimeError::new(format!("WSS authentication failed: {error}")))?;
    while let Some(message) = socket
        .next()
        .await
        .transpose()
        .map_err(|error| RuntimeError::new(format!("WSS receive failed: {error}")))?
    {
        let Message::Text(text) = message else {
            continue;
        };
        let envelope: LinuxEnvelope = serde_json::from_str(&text)
            .map_err(|error| RuntimeError::new(format!("WSS notification JSON failed: {error}")))?;
        if envelope.kind != "notification" {
            continue;
        }
        let payload = envelope
            .payload
            .ok_or_else(|| RuntimeError::new("WSS notification payload is missing"))?;
        payload
            .validate()
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        present_linux_notification(&payload).await?;
        socket
            .send(Message::Text(
                json!({"type":"ack","deliveryId":envelope.delivery_id})
                    .to_string()
                    .into(),
            ))
            .await
            .map_err(|error| RuntimeError::new(format!("WSS acknowledgement failed: {error}")))?;
    }
    Err(RuntimeError::new("WSS receiver disconnected"))
}

#[derive(Debug, Deserialize)]
struct LinuxEnvelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "deliveryId")]
    delivery_id: String,
    payload: Option<NotificationPayload>,
}

pub async fn present_linux_notification(payload: &NotificationPayload) -> RuntimeResult<()> {
    let status = tokio::process::Command::new("notify-send")
        .arg("--app-name=Dowe")
        .arg(&payload.title)
        .arg(&payload.body)
        .status()
        .await
        .map_err(|error| RuntimeError::new(format!("notify-send unavailable: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(RuntimeError::new(format!(
            "notify-send exited with {status}"
        )))
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn form_body(values: &[(&str, &str)]) -> String {
    values
        .iter()
        .map(|(key, value)| format!("{}={}", form_escape(key), form_escape(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn form_escape(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            escaped.push(byte as char);
        } else {
            escaped.push_str(&format!("%{byte:02X}"));
        }
    }
    escaped
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

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
