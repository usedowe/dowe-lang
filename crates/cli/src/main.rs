mod agent;
mod build;
mod cache_cli;
mod codegraph;
mod d1_cli;
mod database_cli;
mod deploy;
mod dev;
mod icons;
mod init;
mod menus;
mod server;
mod spawn_cli;
mod test_cli;
mod uninstall;
mod upgrade;
mod usage;
mod vector_cli;
mod version;

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
    if dowe_runtime::run_development_desktop_host_from_env()? {
        return Ok(());
    }
    if dowe_runtime::run_embedded_desktop_app().await? {
        return Ok(());
    }
    if dowe_runtime::run_background_worker_from_env().await? {
        return Ok(());
    }
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None => run_root_menu().await,
        Some("init") => init::run_init_command(&args[1..]),
        Some("icons") => icons::run_icons_command(&args[1..]),
        Some("dev") => dev::run_dev_command(&args[1..]).await,
        Some("test") => test_cli::run_test_command(&args[1..]),
        Some("build") => build::run_build_command(&args[1..]).await,
        Some("deploy") => deploy::run_deploy_command(&args[1..]),
        Some("agent") => agent::run_agent_command(&args[1..]).await,
        Some("codegraph") => codegraph::run_codegraph_command(&args[1..]).await,
        Some("d1") => d1_cli::run_d1_command(&args[1..]),
        Some("cache") => cache_cli::run_cache_command(&args[1..]).await,
        Some("spawn") => spawn_cli::run_spawn_command(args[1..].to_vec()).await,
        Some("database") => database_cli::run_database_command(&args[1..]).await,
        Some("server") => server::run_server_command(&args[1..]).await,
        Some("vector") => vector_cli::run_vector_command(&args[1..]).await,
        Some("uninstall") => uninstall::run_uninstall_command(&args[1..]),
        Some("upgrade") => upgrade::run_upgrade_command(&args[1..]).await,
        Some("version") | Some("--version") | Some("-V") => {
            version::run_version_command(&args[1..])
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
        "icons" => icons::run_icons_command(&[]),
        "dev" => dev::run_dev_command(&[]).await,
        "test" => test_cli::run_test_command(&[]),
        "build" => build::run_build_command(&[]).await,
        "deploy" => deploy::run_deploy_command(&[]),
        "agent" => agent::run_agent_command(&[]).await,
        "codegraph" => codegraph::run_codegraph_command(&[]).await,
        "d1" => d1_cli::run_d1_command(&[]),
        "cache" => cache_cli::run_cache_command(&[]).await,
        "database" => database_cli::run_database_command(&[]).await,
        "vector" => vector_cli::run_vector_command(&[]).await,
        "uninstall" => uninstall::run_uninstall_command(&[]),
        "upgrade" => upgrade::run_upgrade_command(&[]).await,
        "version" => version::run_version_command(&[]),
        _ => Err(USAGE.into()),
    }
}
