mod agent;
mod codegraph;
mod deploy;
mod dev;
mod init;
mod menus;
mod spawn_cli;
mod store_cli;
mod upgrade;
mod usage;

use std::env;
use usage::USAGE;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("ERROR {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None => run_root_menu().await,
        Some("init") => init::run_init_command(&args[1..]),
        Some("dev") => dev::run_dev_command(&args[1..]).await,
        Some("deploy") => deploy::run_deploy_command(&args[1..]),
        Some("agent") => agent::run_agent_command(&args[1..]).await,
        Some("codegraph") => codegraph::run_codegraph_command(&args[1..]).await,
        Some("spawn") => spawn_cli::run_spawn_command(args[1..].to_vec()).await,
        Some("store") => store_cli::run_store_command(&args[1..]),
        Some("upgrade") => upgrade::run_upgrade_command(&args[1..]).await,
        Some("--version") | Some("-V") => {
            println!("dowe {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

async fn run_root_menu() -> Result<(), Box<dyn std::error::Error>> {
    if !menus::is_interactive_terminal() {
        return Err(USAGE.into());
    }

    let Some(command) = menus::prompt_root_command()? else {
        return Ok(());
    };

    match command.as_str() {
        "init" => init::run_init_command(&[]),
        "dev" => dev::run_dev_command(&[]).await,
        "deploy" => deploy::run_deploy_command(&[]),
        "agent" => agent::run_agent_command(&[]).await,
        "codegraph" => codegraph::run_codegraph_command(&[]).await,
        "store" => store_cli::run_store_command(&[]),
        "upgrade" => upgrade::run_upgrade_command(&[]).await,
        _ => Err(USAGE.into()),
    }
}
