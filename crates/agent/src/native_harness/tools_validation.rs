fn validate_declared_command(command: &str) -> AgentResult<()> {
    let argv = command.split_whitespace().collect::<Vec<_>>();
    let allowed = matches!(
        argv.as_slice(),
        ["cargo", "check", ..]
            | ["cargo", "test", ..]
            | ["cargo", "clippy", ..]
            | ["dowe", "check", ..]
            | ["dowe", "test", ..]
            | ["dowe", "codegraph", "check", ..]
            | ["harness", "check", ..]
    );
    if argv.is_empty() || !allowed || command.len() > 4096 {
        return Err(AgentError::new(
            "validation command must be an allowlisted cargo/dowe/harness argv command",
        ));
    }
    if argv.iter().any(|arg| {
        arg.chars()
            .any(|character| matches!(character, ';' | '|' | '&' | '$' | '`' | '\n' | '\r'))
    }) {
        return Err(AgentError::new("validation command contains shell syntax"));
    }
    Ok(())
}

fn run_declared_validation(root: &std::path::Path, command: &str) -> AgentResult<Value> {
    validate_declared_command(command)?;
    let mut argv = command.split_whitespace();
    let executable = argv.next().expect("validated command is non-empty");
    let output = std::process::Command::new(executable)
        .args(argv)
        .current_dir(root)
        .env_remove("DOWE_AGENT_BROWSER")
        .output()
        .map_err(|error| AgentError::new(format!("validation failed to start: {error}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let truncate = |value: &str| value.chars().take(16_384).collect::<String>();
    Ok(serde_json::json!({
        "status": if output.status.success() { "passed" } else { "failed" },
        "exit_code": output.status.code().unwrap_or(-1),
        "command": command,
        "stdout": truncate(&stdout),
        "stderr": truncate(&stderr)
    }))
}

impl HarnessTools {
    pub(crate) fn apply_validation(&mut self, approval: super::Approval) -> AgentResult<Value> {
        self.consume(&approval)?;
        let command = approval.call.arguments["command"]
            .as_str()
            .ok_or_else(|| AgentError::new("validation command missing"))?;
        run_declared_validation(&self.root, command)
    }
}
