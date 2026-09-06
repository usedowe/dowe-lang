use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn lists_pi_compatible_providers_without_credentials() {
    let output = dowe()
        .args(["agent", "providers", "--json"])
        .output()
        .expect("providers");
    let providers: Value = serde_json::from_slice(&output.stdout).expect("providers json");

    assert!(output.status.success());
    assert_eq!(providers.as_array().expect("provider list").len(), 40);
    assert_eq!(providers[0]["id"], "amazon-bedrock");
    assert_eq!(providers[22]["id"], "openai");
}

#[test]
fn rejects_the_removed_skills_command() {
    let skills = dowe()
        .args(["agent", "skills", "list"])
        .output()
        .expect("skills command");
    let init = dowe()
        .args(["agent", "init"])
        .output()
        .expect("init command");
    let update = dowe()
        .args(["agent", "update"])
        .output()
        .expect("update command");

    assert!(!skills.status.success());
    assert!(!init.status.success());
    assert!(!update.status.success());
}

#[test]
fn agent_lifecycle_commands_are_replaced_by_the_terminal_session() {
    let temp = TempDir::new().expect("tempdir");
    for args in [["agent", "init"], ["agent", "update"]] {
        let output = dowe()
            .args(args)
            .current_dir(temp.path())
            .output()
            .expect("agent lifecycle command");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
    assert!(!temp.path().join("AGENTS.md").exists());
    assert!(!temp.path().join("CLAUDE.md").exists());
    assert!(!temp.path().join(".agents").exists());
}
#[test]
fn starts_the_embedded_terminal_agent_without_project_files() {
    let temp = TempDir::new().expect("tempdir");
    let mut child = dowe()
        .args(["agent"])
        .current_dir(temp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("agent session");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"/exit\n")
        .expect("exit");
    let output = child.wait_with_output().expect("agent output");
    let stdout = String::from_utf8(output.stdout).expect("stdout");

    assert!(output.status.success());
    assert!(stdout.contains("Dowe Agent"));
    assert!(stdout.contains("embedded in the Dowe binary"));
    assert!(!temp.path().join("AGENTS.md").exists());
    assert!(!temp.path().join("CLAUDE.md").exists());
    assert!(!temp.path().join(".agents").exists());
}

#[test]
fn human_example_search_prints_dowe_source() {
    let output = dowe()
        .args(["agent", "examples", "search", "dashboard sidebar form"])
        .output()
        .expect("example search");
    let stdout = String::from_utf8(output.stdout).expect("stdout");

    assert!(output.status.success());
    assert!(stdout.contains("Application sidebar layout"));
    assert!(stdout.contains("source dowe-agent://examples/"));
    assert!(stdout.contains("Scaffold"));
}

#[test]
fn searches_examples_and_builds_context_as_json() {
    let examples = dowe()
        .args([
            "agent",
            "examples",
            "search",
            "dashboard sidebar form",
            "--json",
        ])
        .output()
        .expect("example search");
    let context = dowe()
        .args(["agent", "context", "project", "--json"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("project context");
    let examples: Value = serde_json::from_slice(&examples.stdout).expect("examples json");
    let context: Value = serde_json::from_slice(&context.stdout).expect("context json");

    assert_eq!(examples["results"][0]["id"], "dashboard-layout");
    assert_eq!(context["skills"].as_array().expect("skills").len(), 6);
    assert!(context.get("codegraph").is_some());
}

#[test]
fn serves_mcp_on_standard_input_and_output() {
    let mut child = dowe()
        .args(["agent", "mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    stdin
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n{\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        )
        .expect("write mcp");
    drop(stdin);
    let output = child.wait_with_output().expect("mcp output");
    let responses = String::from_utf8(output.stdout).expect("mcp utf8");
    let lines = responses.lines().collect::<Vec<_>>();

    assert!(output.status.success());
    assert_eq!(lines.len(), 3);
    assert_eq!(
        serde_json::from_str::<Value>(lines[0]).expect("ping")["result"],
        serde_json::json!({})
    );
    assert_eq!(
        serde_json::from_str::<Value>(lines[1]).expect("parse error")["error"]["code"],
        -32700
    );
    assert_eq!(
        serde_json::from_str::<Value>(lines[2]).expect("tools")["result"]["tools"]
            .as_array()
            .expect("tools")
            .len(),
        4
    );
}
