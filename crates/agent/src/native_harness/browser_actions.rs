use super::{Approval, HarnessRole, HarnessTools, ToolCall, identifier};
use crate::{AgentError, AgentResult};
use dowe_agent_harness::{SimulationStep, UseCaseAction, UseCaseScenario, UseCaseSimulationReport};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

const MAX_ACTIONS: usize = 128;
const MAX_SELECTOR: usize = 512;
const MAX_TEXT: usize = 16 * 1024;
const MAX_DOM_BYTES: usize = 512 * 1024;
const CDP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    Fill { selector: String, value: String },
    Press { key: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActionResult {
    pub index: usize,
    pub action: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserExecutionReport {
    pub passed: bool,
    pub results: Vec<BrowserActionResult>,
    pub dom_sha256: Option<String>,
    pub dom_bytes: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserActionsArgs {
    cdp_endpoint: String,
    actions: Vec<BrowserAction>,
    reason: String,
}

impl HarnessTools {
    pub(super) fn prepare_browser_actions(
        &self,
        call: &ToolCall,
        role: HarnessRole,
    ) -> AgentResult<Approval> {
        if role != HarnessRole::Execute {
            return Err(AgentError::new(
                "browser actions are available only to the Execute role",
            ));
        }
        let args: BrowserActionsArgs = serde_json::from_value(call.arguments.clone())?;
        validate_endpoint(&args.cdp_endpoint)?;
        if args.reason.trim().is_empty() || args.reason.len() > 1024 {
            return Err(AgentError::new("browser action reason exceeds limits"));
        }
        if args.actions.is_empty() || args.actions.len() > MAX_ACTIONS {
            return Err(AgentError::new(
                "browser action list is empty or exceeds its bound",
            ));
        }
        for action in &args.actions {
            validate_action(action)?;
        }
        Ok(Approval {
            id: identifier(),
            session: self.session.clone(),
            call: call.clone(),
            details: json!({
                "cdp_endpoint": args.cdp_endpoint,
                "actions": args.actions,
                "reason": args.reason,
                "policy": "loopback CDP only; execute after host approval; DOM evidence is bounded and hashed"
            }),
            before: None,
            after: None,
            before_bytes: None,
            after_bytes: None,
            reference_images: Vec::new(),
        })
    }

    pub(super) async fn run_browser_actions(&mut self, approval: Approval) -> AgentResult<Value> {
        self.consume(&approval)?;
        let args: BrowserActionsArgs = serde_json::from_value(approval.call.arguments.clone())?;
        Ok(serde_json::to_value(
            execute_browser_actions(&args.cdp_endpoint, &args.actions).await?,
        )?)
    }
}

/// Execute bounded DOM actions through a local Chrome DevTools Protocol
/// endpoint. The endpoint must be loopback and the final DOM is hashed as
/// durable evidence; no remote browser or arbitrary network target is allowed.
pub async fn execute_browser_actions(
    endpoint: &str,
    actions: &[BrowserAction],
) -> AgentResult<BrowserExecutionReport> {
    validate_endpoint(endpoint)?;
    if actions.is_empty() || actions.len() > MAX_ACTIONS {
        return Err(AgentError::new(
            "browser action list is empty or exceeds its bound",
        ));
    }
    for action in actions {
        validate_action(action)?;
    }
    let websocket_url = resolve_websocket_url(endpoint).await?;
    validate_endpoint(&websocket_url)?;
    let (mut socket, _) = connect_async(&websocket_url)
        .await
        .map_err(|error| AgentError::new(format!("could not connect to local CDP: {error}")))?;
    let mut next_id = 1_u64;
    let mut results = Vec::with_capacity(actions.len());
    for (index, action) in actions.iter().enumerate() {
        let (method, params, label) = command_for_action(action)?;
        let response = send_command(&mut socket, &mut next_id, method, params).await;
        match response {
            Ok(_) => results.push(BrowserActionResult {
                index,
                action: label.into(),
                passed: true,
                detail: "CDP command completed".into(),
            }),
            Err(error) => {
                results.push(BrowserActionResult {
                    index,
                    action: label.into(),
                    passed: false,
                    detail: error.to_string(),
                });
                return Ok(BrowserExecutionReport {
                    passed: false,
                    results,
                    dom_sha256: None,
                    dom_bytes: None,
                });
            }
        }
    }
    let dom = send_command(
        &mut socket,
        &mut next_id,
        "Runtime.evaluate",
        json!({"expression":"document.documentElement.outerHTML","returnByValue":true}),
    )
    .await?;
    let dom = dom["result"]["result"]["value"]
        .as_str()
        .ok_or_else(|| AgentError::new("CDP did not return a DOM string"))?;
    if dom.len() > MAX_DOM_BYTES {
        return Err(AgentError::new("browser DOM evidence exceeds its bound"));
    }
    let digest = Sha256::digest(dom.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(BrowserExecutionReport {
        passed: true,
        results,
        dom_sha256: Some(digest),
        dom_bytes: Some(dom.len()),
    })
}

/// Execute a durable use-case scenario through the CDP adapter and project its
/// evidence back into the scenario report contract.
pub async fn execute_use_case_over_browser(
    endpoint: &str,
    scenario: &UseCaseScenario,
) -> AgentResult<UseCaseSimulationReport> {
    scenario
        .validate()
        .map_err(|error| AgentError::new(error.to_string()))?;
    let mut browser_actions = Vec::with_capacity(scenario.actions.len());
    for action in &scenario.actions {
        match browser_action(action) {
            Ok(action) => browser_actions.push(action),
            Err(error) => {
                return Ok(UseCaseSimulationReport {
                    scenario_id: scenario.id.clone(),
                    passed: false,
                    final_state: scenario.initial_state.clone(),
                    steps: vec![SimulationStep {
                        action_id: action.id.clone(),
                        passed: false,
                        detail: error.to_string(),
                    }],
                    failed_assertions: scenario
                        .assertions
                        .iter()
                        .map(|assertion| assertion.id.clone())
                        .collect(),
                });
            }
        }
    }
    let browser = execute_browser_actions(endpoint, &browser_actions).await?;
    let mut state = scenario.initial_state.clone();
    if !state.is_object() {
        state = Value::Object(serde_json::Map::new());
    }
    state["browser"] = serde_json::to_value(&browser)?;
    let failed_assertions = scenario
        .assertions
        .iter()
        .filter(|assertion| state.pointer(&assertion.path) != Some(&assertion.expected))
        .map(|assertion| assertion.id.clone())
        .collect::<Vec<_>>();
    let steps = browser
        .results
        .iter()
        .map(|step| SimulationStep {
            action_id: scenario.actions[step.index].id.clone(),
            passed: step.passed,
            detail: step.detail.clone(),
        })
        .collect::<Vec<_>>();
    Ok(UseCaseSimulationReport {
        scenario_id: scenario.id.clone(),
        passed: browser.passed && failed_assertions.is_empty(),
        final_state: state,
        steps,
        failed_assertions,
    })
}

fn browser_action(action: &UseCaseAction) -> AgentResult<BrowserAction> {
    match action.kind.as_str() {
        "navigate" => Ok(BrowserAction::Navigate {
            url: action
                .value
                .as_ref()
                .and_then(Value::as_str)
                .unwrap_or(&action.target)
                .to_string(),
        }),
        "click" => Ok(BrowserAction::Click {
            selector: action.target.clone(),
        }),
        "fill" => Ok(BrowserAction::Fill {
            selector: action.target.clone(),
            value: action
                .value
                .as_ref()
                .and_then(Value::as_str)
                .ok_or_else(|| AgentError::new("fill action requires a string value"))?
                .to_string(),
        }),
        "press" => Ok(BrowserAction::Press {
            key: action
                .value
                .as_ref()
                .and_then(Value::as_str)
                .unwrap_or(&action.target)
                .to_string(),
        }),
        other => Err(AgentError::new(format!(
            "unsupported browser action kind `{other}`"
        ))),
    }
}

fn validate_action(action: &BrowserAction) -> AgentResult<()> {
    match action {
        BrowserAction::Navigate { url } => validate_loopback_url(url),
        BrowserAction::Click { selector } => bounded(selector, MAX_SELECTOR, "browser selector"),
        BrowserAction::Fill { selector, value } => {
            bounded(selector, MAX_SELECTOR, "browser selector")?;
            bounded(value, MAX_TEXT, "browser input")
        }
        BrowserAction::Press { key } => bounded(key, 64, "browser key"),
    }
}

fn command_for_action(action: &BrowserAction) -> AgentResult<(&'static str, Value, &'static str)> {
    match action {
        BrowserAction::Navigate { url } => Ok(("Page.navigate", json!({"url":url}), "navigate")),
        BrowserAction::Click { selector } => Ok((
            "Runtime.evaluate",
            json!({"expression":format!("document.querySelector({}).click()", serde_json::to_string(selector)?),"awaitPromise":true}),
            "click",
        )),
        BrowserAction::Fill { selector, value } => Ok((
            "Runtime.evaluate",
            json!({"expression":format!("(()=>{{const e=document.querySelector({}); if(!e) throw new Error('selector not found'); e.focus(); e.value={}; e.dispatchEvent(new Event('input',{{bubbles:true}})); e.dispatchEvent(new Event('change',{{bubbles:true}}));}})()", serde_json::to_string(selector)?, serde_json::to_string(value)?),"awaitPromise":true}),
            "fill",
        )),
        BrowserAction::Press { key } => Ok((
            "Input.dispatchKeyEvent",
            json!({"type":"keyDown","key":key}),
            "press",
        )),
    }
}

async fn send_command(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: &mut u64,
    method: &str,
    params: Value,
) -> AgentResult<Value> {
    let id = *next_id;
    *next_id = (*next_id).saturating_add(1);
    socket
        .send(Message::Text(
            serde_json::to_string(&json!({"id":id,"method":method,"params":params}))?.into(),
        ))
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    let wait = async {
        while let Some(message) = socket.next().await {
            let message = message.map_err(|error| AgentError::new(error.to_string()))?;
            let Message::Text(text) = message else {
                continue;
            };
            let value: Value = serde_json::from_str(&text)?;
            if value["id"].as_u64() != Some(id) {
                continue;
            }
            if let Some(error) = value.get("error") {
                return Err(AgentError::new(format!("CDP command failed: {error}")));
            }
            return Ok(value);
        }
        Err(AgentError::new("CDP socket closed before command response"))
    };
    tokio::time::timeout(CDP_TIMEOUT, wait)
        .await
        .map_err(|_| AgentError::new("CDP command timed out"))?
}

async fn resolve_websocket_url(endpoint: &str) -> AgentResult<String> {
    if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
        return Ok(endpoint.to_string());
    }
    let base = endpoint.trim_end_matches('/');
    let url = if base.ends_with("/json/version") {
        base.to_string()
    } else {
        format!("{base}/json/version")
    };
    let response: Value = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| AgentError::new(error.to_string()))?
        .get(url)
        .send()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?
        .json()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    response["webSocketDebuggerUrl"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| AgentError::new("CDP version response lacks webSocketDebuggerUrl"))
}

fn validate_endpoint(endpoint: &str) -> AgentResult<()> {
    let url = reqwest::Url::parse(endpoint)
        .map_err(|_| AgentError::new("CDP endpoint is not a valid URL"))?;
    if !matches!(url.scheme(), "http" | "ws")
        || url.username() != ""
        || url.password().is_some()
        || url.port().is_none()
        || !matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
    {
        return Err(AgentError::new(
            "CDP endpoint must be an explicit loopback HTTP or WS URL",
        ));
    }
    Ok(())
}

fn validate_loopback_url(url: &str) -> AgentResult<()> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| AgentError::new("browser navigation URL is invalid"))?;
    if parsed.scheme() != "http"
        || parsed.username() != ""
        || parsed.password().is_some()
        || parsed.port().is_none()
        || !matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
    {
        return Err(AgentError::new(
            "browser navigation must target loopback HTTP",
        ));
    }
    Ok(())
}

fn bounded(value: &str, limit: usize, label: &str) -> AgentResult<()> {
    if value.is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(AgentError::new(format!("{label} is invalid or unbounded")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    include!("browser_actions_tests.rs");
}
