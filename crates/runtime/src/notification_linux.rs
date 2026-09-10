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


