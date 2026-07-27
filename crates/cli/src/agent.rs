mod chat;
mod context;
mod examples;
mod harness;
mod init;
mod mcp;

use crate::menus;
use crate::usage::USAGE;

pub(crate) async fn run_agent_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("init") => init::run_agent_init_command(&args[1..]),
        Some("update") => init::run_agent_update_command(&args[1..]),
        Some("examples") => examples::run_agent_examples_command(&args[1..]),
        Some("context") => context::run_agent_context_command(&args[1..]),
        Some("mcp") => mcp::run_agent_mcp_command(&args[1..]),
        Some("harness") => harness::run_agent_harness_command(&args[1..]).await,
        Some("chat") => chat::run_agent_chat_command(&args[1..]).await,
        None if menus::is_interactive_terminal() => {
            let Some(command) = menus::prompt_agent_command()? else {
                return Ok(());
            };
            match command.as_str() {
                "init" => init::run_agent_init_command(&[]),
                "update" => init::run_agent_update_command(&[]),
                _ => Err(USAGE.into()),
            }
        }
        Some(_) => Err(USAGE.into()),
        None => Err(USAGE.into()),
    }
}
