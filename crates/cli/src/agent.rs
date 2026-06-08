mod chat;
mod harness;

use crate::menus;
use crate::usage::USAGE;

pub(crate) async fn run_agent_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("harness") => harness::run_agent_harness_command(&args[1..]).await,
        Some("chat") => chat::run_agent_chat_command(&args[1..]).await,
        None if menus::is_interactive_terminal() => {
            let Some(command) = menus::prompt_agent_command()? else {
                return Ok(());
            };
            match command.as_str() {
                "chat" => chat::run_agent_chat_command(&[]).await,
                "harness" => harness::run_agent_harness_command(&[]).await,
                _ => Err(USAGE.into()),
            }
        }
        Some(_) => chat::run_agent_chat_command(args).await,
        None => Err(USAGE.into()),
    }
}
