async fn send_ws_event(
    socket: &mut WebSocket,
    event: &str,
    request_id: &str,
    request_type: &str,
    model: &str,
    payload: Value,
    content: Option<String>,
) -> Result<(), ()> {
    let mut output = Map::new();
    output.insert("event".to_string(), Value::String(event.to_string()));
    output.insert(
        "requestId".to_string(),
        Value::String(request_id.to_string()),
    );
    if !request_type.is_empty() {
        output.insert(
            "requestType".to_string(),
            Value::String(request_type.to_string()),
        );
    }
    if !model.is_empty() {
        output.insert("model".to_string(), Value::String(model.to_string()));
    }
    let payload = if request_type.is_empty() && model.is_empty() {
        safe_client_payload(event, &payload)
    } else {
        payload
    };
    output.insert("payload".to_string(), payload);
    if let Some(content) = content {
        output.insert("content".to_string(), Value::String(content));
    }
    send_ws_value(socket, Value::Object(output)).await
}

fn safe_client_payload(event: &str, payload: &Value) -> Value {
    if event == "error" {
        return payload
            .get("error")
            .filter(|value| value.is_object())
            .cloned()
            .map(|error| {
                let mut output = Map::new();
                output.insert("error".to_string(), error);
                Value::Object(output)
            })
            .unwrap_or_else(|| Value::Object(Map::new()));
    }
    Value::Object(Map::new())
}

async fn send_ws_value(socket: &mut WebSocket, value: Value) -> Result<(), ()> {
    socket
        .send(Message::Text(value.to_string().into()))
        .await
        .map_err(|_| ())
}

fn done_payload() -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    Value::Object(output)
}

