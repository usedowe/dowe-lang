mod chat;
mod context;
mod examples;
mod harness;
mod mcp;

use crate::usage::USAGE;

pub(crate) async fn run_agent_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("examples") => examples::run_agent_examples_command(&args[1..]),
        Some("providers") => chat::run_agent_providers_command(&args[1..]),
        Some("context") => context::run_agent_context_command(&args[1..]),
        Some("mcp") => mcp::run_agent_mcp_command(&args[1..]),
        Some("harness") => harness::run_agent_harness_command(&args[1..]).await,
        Some("chat") => chat::run_agent_chat_command(&args[1..]).await,
        Some("init") | Some("update") | Some("skills") => Err(USAGE.into()),
        None => chat::run_agent_session().await,
        Some(_) => chat::run_agent_chat_command(args).await,
    }
}
