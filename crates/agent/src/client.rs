use crate::error::{AgentError, AgentResult};
use crate::model::{AgentRequest, AgentServerResponse};
use reqwest::StatusCode;
use serde_json::Value;

pub async fn send_agent_request(
    server_url: &str,
    request: &AgentRequest,
) -> AgentResult<AgentServerResponse> {
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

fn status_text(status: StatusCode) -> String {
    status
        .canonical_reason()
        .map(|reason| format!("{} {reason}", status.as_u16()))
        .unwrap_or_else(|| status.as_u16().to_string())
}
