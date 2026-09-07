#![cfg(any(target_os = "macos", target_os = "linux"))]

use dowe_agent::AgentResult;
use dowe_agent::native_harness::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Observer {
    events: Vec<Value>,
    terminal: Arc<Mutex<Vec<u8>>>,
}
impl ShellObserver for Observer {
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        Ok(Box::new(Terminal {
            sent: false,
            output: self.terminal.clone(),
        }))
    }
}
struct Terminal {
    sent: bool,
    output: Arc<Mutex<Vec<u8>>>,
}
impl HarnessTerminal for Terminal {
    fn poll(&mut self) -> AgentResult<Vec<TerminalInput>> {
        if self.sent {
            return Ok(vec![]);
        }
        self.sent = true;
        Ok(vec![
            TerminalInput::Resize { rows: 40, cols: 90 },
            TerminalInput::Bytes(b"local-password\n".to_vec()),
        ])
    }
    fn output(&mut self, bytes: &[u8]) -> AgentResult<()> {
        self.output.lock().unwrap().extend(bytes);
        Ok(())
    }
}
fn tools(root: &std::path::Path) -> HarnessTools {
    HarnessTools::new(
        root,
        "session",
        HarnessConfig {
            shell: Some("/bin/sh".into()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn call(command: &str, pty: bool) -> ToolCall {
    ToolCall::new(
        "shell-test",
        "shell",
        json!({"command":command,"pty":pty,"cwd":".","reason":"validate shared shell"}),
    )
}

#[tokio::test]
async fn pipes_stream_before_exit_without_exposing_split_secrets() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".env"), "TOKEN=hidden-value\n").unwrap();
    let mut tools = tools(root.path());
    let approval = tools.prepare(&call("printf 'ready\\n'; sleep 0.1; printf hidden-; sleep 0.1; printf 'value\\n'; printf 'error\\n' >&2", false), HarnessRole::Execute).unwrap().unwrap();
    let mut observer = Observer::default();
    let result = tools
        .run_shell_observed(approval, &mut observer)
        .await
        .unwrap();
    assert_eq!(result["exit_code"], 0);
    let text = serde_json::to_string(&observer.events).unwrap();
    assert!(!text.contains("hidden-value") && !text.contains("hidden-"));
    let ready = observer
        .events
        .iter()
        .position(|e| {
            e["event"] == "shell_output" && e["text"].as_str().unwrap_or("").contains("ready")
        })
        .unwrap();
    let exit = observer
        .events
        .iter()
        .position(|e| e["event"] == "shell_exited")
        .unwrap();
    assert!(ready < exit);
}

#[tokio::test]
async fn pty_accepts_local_input_and_resize_without_recording_terminal_bytes() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = tools(root.path());
    let approval = tools
        .prepare(
            &call("read value; stty size; printf '%s' \"$value\"", true),
            HarnessRole::Execute,
        )
        .unwrap()
        .unwrap();
    assert_eq!(approval.details["stdin"], "terminal");
    let mut observer = Observer::default();
    let result = tools
        .run_shell_observed(approval, &mut observer)
        .await
        .unwrap();
    let local = String::from_utf8_lossy(&observer.terminal.lock().unwrap()).to_string();
    assert!(local.contains("40 90"));
    assert!(local.contains("local-password"));
    assert_eq!(result["exit_code"], 0);
    assert!(!result.to_string().contains("local-password"));
    assert!(
        !serde_json::to_string(&observer.events)
            .unwrap()
            .contains("local-password")
    );
}

#[tokio::test]
async fn a_host_without_terminal_support_never_starts_a_pty_process() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = tools(root.path());
    let approval = tools
        .prepare(&call("touch forbidden", true), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    assert!(tools.run_shell(approval).await.is_err());
    assert!(!root.path().join("forbidden").exists());
}
