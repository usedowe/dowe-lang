use super::{HarnessHost, HarnessRole, HarnessTerminal, HarnessTools, ShellObserver, ToolCall};
use crate::{AgentError, AgentResult};
use serde_json::json;

pub(super) async fn execute_clean_call(
    tools: &mut HarnessTools,
    call: ToolCall,
    host: &mut impl HarnessHost,
) -> AgentResult<serde_json::Value> {
    if matches!(
        call.name.as_str(),
        "read_file"
            | "list_files"
            | "search"
            | "convert_svg"
            | "get_skill"
            | "find_component"
            | "get_component_contract"
            | "search_codegraph"
            | "get_node"
            | "get_source"
            | "get_dependencies"
            | "get_consumers"
            | "get_related"
            | "get_impact"
    ) {
        return tools.execute_read(&call);
    }
    if call.name == "validate_dowe_project" {
        return host.validate_dowe_project(&tools.root).await;
    }
    if call.name == "capture_web_screenshot" {
        let approval = tools
            .prepare(&call, HarnessRole::Execute)?
            .ok_or_else(|| AgentError::new("screenshot did not produce an approval"))?;
        return match host.approve(&approval).await? {
            Some(true) => {
                operation_started(host, &call)?;
                tools.capture_web_screenshot(approval)
            }
            Some(false) => {
                tools.reject(approval)?;
                Ok(json!({"status":"rejected"}))
            }
            None => {
                tools.reject(approval)?;
                Err(AgentError::new("approval required"))
            }
        };
    }
    if call.name == "execute_browser_actions" {
        let approval = tools
            .prepare(&call, HarnessRole::Execute)?
            .ok_or_else(|| AgentError::new("browser actions did not produce an approval"))?;
        return match host.approve(&approval).await? {
            Some(true) => {
                operation_started(host, &call)?;
                tools.run_browser_actions(approval).await
            }
            Some(false) => {
                tools.reject(approval)?;
                Ok(json!({"status":"rejected"}))
            }
            None => {
                tools.reject(approval)?;
                Err(AgentError::new("approval required"))
            }
        };
    }
    if matches!(call.name.as_str(), "write_file" | "edit_file") {
        let approvals =
            tools.prepare_text_write_batch(std::slice::from_ref(&call), HarnessRole::Execute)?;
        let decision = host.approve_batch(&approvals).await?;
        return match decision {
            Some(true) => {
                operation_started(host, &call)?;
                tools
                    .apply_text_write_batch(approvals)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Err(AgentError::new("write batch returned no result")))
            }
            Some(false) => {
                for approval in approvals {
                    tools.reject(approval)?;
                }
                Ok(json!({"status":"rejected"}))
            }
            None => {
                for approval in approvals {
                    tools.reject(approval)?;
                }
                Err(AgentError::new("approval required"))
            }
        };
    }
    if call.name == "shell" {
        let approval = tools
            .prepare(&call, HarnessRole::Execute)?
            .ok_or_else(|| AgentError::new("clean build runner does not support this tool"))?;
        let decision = host.approve(&approval).await?;
        return match decision {
            Some(true) => {
                operation_started(host, &call)?;
                let mut observer = HostShellObserver { host };
                tools.run_shell_observed(approval, &mut observer).await
            }
            Some(false) => {
                tools.reject(approval)?;
                Ok(json!({"status":"rejected"}))
            }
            None => {
                tools.reject(approval)?;
                Err(AgentError::new("approval required"))
            }
        };
    }
    if call.name == "run_validation" {
        let approval = tools
            .prepare(&call, HarnessRole::Execute)?
            .ok_or_else(|| AgentError::new("validation did not produce an approval"))?;
        return match host.approve(&approval).await? {
            Some(true) => {
                operation_started(host, &call)?;
                tools.apply_validation(approval)
            }
            Some(false) => {
                tools.reject(approval)?;
                Ok(json!({"status":"rejected"}))
            }
            None => {
                tools.reject(approval)?;
                Err(AgentError::new("approval required"))
            }
        };
    }
    let approval = tools
        .prepare(&call, HarnessRole::Execute)?
        .ok_or_else(|| AgentError::new("clean build runner does not support this tool"))?;
    match host.approve(&approval).await? {
        Some(true) => {
            operation_started(host, &call)?;
            tools.apply_write(approval)
        }
        Some(false) => {
            tools.reject(approval)?;
            Ok(json!({"status":"rejected"}))
        }
        None => {
            tools.reject(approval)?;
            Err(AgentError::new("approval required"))
        }
    }
}

fn operation_started(host: &mut impl HarnessHost, call: &ToolCall) -> AgentResult<()> {
    host.event(&json!({
        "event": "operation_started",
        "call_id": call.id,
        "operation": call.name,
        "state": "executing_tool"
    }))
}

struct HostShellObserver<'a, H: HarnessHost> {
    host: &'a mut H,
}

impl<H: HarnessHost> ShellObserver for HostShellObserver<'_, H> {
    fn event(&mut self, event: &serde_json::Value) -> AgentResult<()> {
        self.host.event(event)
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        self.host.open_terminal()
    }
}
