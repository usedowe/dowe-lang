use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn dowe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dowe"))
}

#[test]
fn rejects_the_removed_skills_command() {
    let skills = dowe()
        .args(["agent", "skills", "list"])
        .output()
        .expect("skills command");
    let update_flag = dowe()
        .args(["agent", "init", "--update"])
        .output()
        .expect("update flag");

    assert!(!skills.status.success());
    assert!(!update_flag.status.success());
}

#[test]
fn initializes_external_agent_project_from_cli() {
    let temp = TempDir::new().expect("tempdir");
    let output = dowe()
        .args(["agent", "init"])
        .current_dir(temp.path())
        .output()
        .expect("agent init");
    let stdout = String::from_utf8(output.stdout).expect("stdout");

    assert!(output.status.success());
    assert!(temp.path().join("AGENTS.md").is_file());
    assert!(temp.path().join("CLAUDE.md").is_file());
    assert!(temp.path().join(".agents/manifest.json").is_file());
    assert!(
        temp.path()
            .join(".agents/skills/dowe-views/SKILL.md")
            .is_file()
    );
    assert_eq!(
        fs::read_dir(temp.path().join(".agents/skills"))
            .expect("skills")
            .count(),
        5
    );
    assert!(stdout.contains("created AGENTS.md"));
    assert!(stdout.contains("created CLAUDE.md"));
    assert!(stdout.contains("installed 5 Dowe skills"));
    assert!(!temp.path().join(".dowe").exists());
}

#[test]
fn updates_an_initialized_external_agent_project() {
    let temp = TempDir::new().expect("tempdir");
    assert!(
        dowe()
            .args(["agent", "init"])
            .current_dir(temp.path())
            .status()
            .expect("init")
            .success()
    );
    fs::write(
        temp.path().join(".agents/skills/dowe-core/SKILL.md"),
        "stale",
    )
    .expect("stale");

    let output = dowe()
        .args(["agent", "update"])
        .current_dir(temp.path())
        .output()
        .expect("agent update");
    let stdout = String::from_utf8(output.stdout).expect("stdout");

    assert!(output.status.success());
    assert!(
        fs::read_to_string(temp.path().join(".agents/skills/dowe-core/SKILL.md"))
            .expect("updated")
            .starts_with("---\nname: dowe-core\n")
    );
    assert!(stdout.contains("updated 5 Dowe skills"));
}

#[test]
fn rejects_implicit_legacy_chat_from_agent_command() {
    let output = dowe()
        .args(["agent", "build a dashboard"])
        .output()
        .expect("agent command");
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(!output.status.success());
    assert!(stderr.contains("Usage: dowe"));
    assert!(!stderr.contains("llm_server_request_failed"));
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
    assert!(stdout.contains("source skill-data/examples/"));
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
    assert_eq!(context["skills"].as_array().expect("skills").len(), 5);
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
