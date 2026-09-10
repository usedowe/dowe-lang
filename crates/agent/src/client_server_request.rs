pub async fn send_agent_request(
    server_url: &str,
    request: &AgentRequest,
) -> AgentResult<AgentServerResponse> {
    if request.extra.contains_key("thinkingLevel") {
        return Err(AgentError::new(
            "thinking selection requires a native provider request",
        ));
    }
    let url = format!("{}/api/v1/agent", server_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(url)
        .json(request)
        .send()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    let status = response.status();
    let payload = response
        .json::<Value>()
        .await
        .unwrap_or_else(|error| Value::String(error.to_string()));

    if !status.is_success() {
        return Err(AgentError::new(format!(
            "llm server returned {}: {}",
            status_text(status),
            payload
        )));
    }

    serde_json::from_value::<AgentServerResponse>(payload)
        .map_err(|error| AgentError::new(error.to_string()))
}

