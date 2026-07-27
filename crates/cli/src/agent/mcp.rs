use crate::usage::USAGE;
use dowe_agent::handle_mcp_message;
use std::env;
use std::io::{self, BufRead, Write};

pub(super) fn run_agent_mcp_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() {
        return Err(USAGE.into());
    }
    let root = env::current_dir()?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if let Some(response) = handle_mcp_message(&root, &line)? {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
