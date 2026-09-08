use super::*;
pub(super) fn active_studio_agents() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static ACTIVE: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) async fn studio_agent_stream(args: &Value) -> RuntimeResult<Value> {
    let backend = required_string(args, "backend")?;
    let authorization = required_string(args, "authorization")?;
    let run_id = required_string(args, "runId")?;
    let payload = required_string(args, "payload")?;
    if payload.len() > STUDIO_AGENT_MAX_PAYLOAD_BYTES {
        return Err(RuntimeError::new(
            "Studio agent payload exceeds its size limit",
        ));
    }
    validate_studio_authorization(&authorization)?;
    validate_studio_run_id(&run_id)?;
    let websocket_url = studio_agent_websocket_url(&backend, &run_id)?;
    let cancellation = Arc::new(AtomicBool::new(false));
    if let Ok(mut active) = active_studio_agents().lock() {
        if let Some(previous) = active.insert(run_id.clone(), cancellation.clone()) {
            previous.store(true, Ordering::Release);
        }
    }
    let result = tokio::time::timeout(
        STUDIO_AGENT_TIMEOUT,
        studio_agent_stream_inner(
            websocket_url,
            authorization,
            run_id.clone(),
            payload,
            cancellation,
        ),
    )
    .await
    .map_err(|_| RuntimeError::new("Studio agent request timed out"))?;
    if let Ok(mut active) = active_studio_agents().lock() {
        active.remove(&run_id);
    }
    result
}

pub(super) async fn cancel_studio_agent(args: &Value) -> RuntimeResult<Value> {
    let run_id = required_string(args, "runId")?;
    validate_studio_run_id(&run_id)?;
    let active = active_studio_agents()
        .lock()
        .ok()
        .and_then(|agents| agents.get(&run_id).cloned());
    let was_active = active.is_some();
    if let Some(active) = active {
        active.store(true, Ordering::Release);
    }
    Ok(json!({ "ok": true, "runId": run_id, "active": was_active, "status": "cancelled" }))
}

pub(super) async fn studio_agent_stream_inner(
    websocket_url: String,
    authorization: String,
    run_id: String,
    payload: String,
    cancellation: Arc<AtomicBool>,
) -> RuntimeResult<Value> {
    let mut request = websocket_url.into_client_request().map_err(|error| {
        RuntimeError::new(format!("could not create Studio agent request: {error}"))
    })?;
    let header = HeaderValue::from_str(&authorization)
        .map_err(|_| RuntimeError::new("Studio agent authorization is invalid"))?;
    request.headers_mut().insert("Authorization", header);
    let request_value = serde_json::from_str::<Value>(&payload).unwrap_or_else(|_| json!({}));
    let request_id = request_value
        .get("requestId")
        .or_else(|| request_value.get("request_id"))
        .and_then(Value::as_str)
        .unwrap_or(&run_id)
        .to_string();
    let (mut socket, _) = connect_async(request).await.map_err(|error| {
        RuntimeError::new(format!("could not connect to Studio agent: {error}"))
    })?;
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|error| {
            RuntimeError::new(format!("could not send Studio agent request: {error}"))
        })?;
    let mut events = Vec::new();
    let mut total_event_bytes = 0usize;
    let mut answer = String::new();
    loop {
        let message = tokio::select! {
            _ = wait_for_studio_agent_cancellation(cancellation.clone()) => {
                let _ = socket.close(None).await;
                return Ok(studio_agent_result(false, &request_id, &answer, "studio_agent_cancelled", events));
            }
            message = socket.next() => message,
        };
        let Some(message) = message else {
            return Ok(studio_agent_result(
                false,
                &request_id,
                &answer,
                "studio_agent_closed",
                events,
            ));
        };
        let message = message.map_err(|error| {
            RuntimeError::new(format!("Studio agent connection failed: {error}"))
        })?;
        let text = match message {
            Message::Text(text) => text.to_string(),
            Message::Binary(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).await.map_err(|error| {
                    RuntimeError::new(format!("Studio agent ping failed: {error}"))
                })?;
                continue;
            }
            Message::Close(_) => {
                return Ok(studio_agent_result(
                    false,
                    &request_id,
                    &answer,
                    "studio_agent_closed",
                    events,
                ));
            }
            Message::Pong(_) | Message::Frame(_) => continue,
        };
        if text.len() > STUDIO_AGENT_MAX_EVENT_BYTES
            || total_event_bytes.saturating_add(text.len()) > STUDIO_AGENT_MAX_TOTAL_EVENT_BYTES
        {
            return Err(RuntimeError::new(
                "Studio agent event exceeds its size limit",
            ));
        }
        total_event_bytes = total_event_bytes.saturating_add(text.len());
        let value = serde_json::from_str::<Value>(&text)
            .map_err(|_| RuntimeError::new("Studio agent returned invalid JSON"))?;
        let event = value
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let content = value
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if answer.len().saturating_add(content.len()) > STUDIO_AGENT_MAX_ANSWER_BYTES {
            return Err(RuntimeError::new(
                "Studio agent answer exceeds its size limit",
            ));
        }
        answer.push_str(&content);
        if events.len() >= STUDIO_AGENT_MAX_EVENTS {
            return Err(RuntimeError::new(
                "Studio agent event count exceeds its limit",
            ));
        }
        events.push(json!({
            "event": event,
            "requestId": value.get("requestId").and_then(Value::as_str).unwrap_or(&request_id),
            "content": content,
            "payload": "",
        }));
        if event == "error" {
            let error = value
                .get("payload")
                .and_then(|payload| payload.get("error"))
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str)
                .unwrap_or("studio_agent_error")
                .to_string();
            return Ok(studio_agent_result(
                false,
                &request_id,
                &answer,
                &error,
                events,
            ));
        }
        if event == "done" {
            return Ok(studio_agent_result(true, &request_id, &answer, "", events));
        }
    }
}

pub(super) async fn wait_for_studio_agent_cancellation(cancellation: Arc<AtomicBool>) {
    while !cancellation.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
}

pub(super) fn studio_agent_result(
    ok: bool,
    request_id: &str,
    answer: &str,
    error: &str,
    events: Vec<Value>,
) -> Value {
    json!({
        "ok": ok,
        "requestId": request_id,
        "answer": answer,
        "error": error,
        "events": events,
    })
}

pub(super) fn studio_agent_websocket_url(backend: &str, run_id: &str) -> RuntimeResult<String> {
    let mut url = reqwest::Url::parse(backend)
        .map_err(|_| RuntimeError::new("Studio backend URL is invalid"))?;
    if url.username() != "" || url.password().is_some() {
        return Err(RuntimeError::new(
            "Studio backend URL cannot contain credentials",
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| RuntimeError::new("Studio backend URL has no host"))?;
    if url.scheme() != "https" && !(url.scheme() == "http" && is_loopback_host(host)) {
        return Err(RuntimeError::new(
            "Studio agent requires HTTPS unless the backend is loopback",
        ));
    }
    let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    url.set_scheme(scheme)
        .map_err(|_| RuntimeError::new("Studio backend URL scheme is invalid"))?;
    let base_path = url.path().trim_end_matches('/');
    url.set_path(&format!("{base_path}/api/studio/runs/{run_id}/stream"));
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

pub(super) fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

pub(super) fn validate_studio_authorization(value: &str) -> RuntimeResult<()> {
    if value.len() > 512 || !value.starts_with("Bearer ") || value[7..].trim().is_empty() {
        return Err(RuntimeError::new("Studio agent authorization is invalid"));
    }
    Ok(())
}

pub(super) fn validate_studio_run_id(value: &str) -> RuntimeResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(RuntimeError::new("Studio agent run id is invalid"));
    }
    Ok(())
}
