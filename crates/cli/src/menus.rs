use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use dowe_deploy::{
    BuildTarget, DeployEnvironment, DeploySurface, DeployTarget, available_build_targets,
    available_deploy_surfaces, deploy_targets_for_surface,
};
use dowe_icons::{IconRounded, IconTarget};
use dowe_runtime::{
    DevTarget, DevTargetDeviceSelection, DevTargetSelection, HostOs, ProjectTemplate,
    available_android_devices, available_ios_simulators, available_project_templates,
};
use std::io::IsTerminal;

pub(crate) fn is_interactive_terminal() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub(crate) fn prompt_root_command() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commands = root_commands();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Dowe")
        .items(&commands)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| commands[index].to_string()))
}

pub(crate) fn prompt_init_template() -> Result<Option<ProjectTemplate>, Box<dyn std::error::Error>>
{
    let templates = available_project_templates();
    let items = templates
        .iter()
        .map(|template| template.label())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select init template")
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| templates[index]))
}

pub(crate) fn prompt_init_i18n() -> Result<Option<bool>, Box<dyn std::error::Error>> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable i18n")
        .default(false)
        .interact_opt()?)
}

pub(crate) fn prompt_init_reinstall() -> Result<Option<bool>, Box<dyn std::error::Error>> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("main.dowe already exists. Reinstall and replace managed project files")
        .default(false)
        .interact_opt()?)
}

pub(crate) fn prompt_icon_source() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Icon SVG path")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_icon_background() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Background color (#RRGGBB)")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_icon_rounded() -> Result<Option<IconRounded>, Box<dyn std::error::Error>> {
    let values = IconRounded::ALL;
    let items = values.map(IconRounded::as_str);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Icon rounded")
        .items(&items)
        .default(0)
        .interact_opt()?;
    Ok(selection.map(|index| values[index]))
}

pub(crate) fn prompt_icon_targets() -> Result<Option<Vec<IconTarget>>, Box<dyn std::error::Error>> {
    let targets = IconTarget::ALL;
    let items = targets.map(IconTarget::as_str);
    loop {
        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Icon targets")
            .items(&items)
            .defaults(&[true; 4])
            .interact_opt()?;
        let Some(indexes) = selection else {
            return Ok(None);
        };
        if indexes.is_empty() {
            eprintln!("Select at least one icon target.");
            continue;
        }
        return Ok(Some(
            indexes.into_iter().map(|index| targets[index]).collect(),
        ));
    }
}

pub(crate) fn prompt_d1_migrations_output() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("D1 icon catalog migrations directory")
        .default("server/migrations".to_string())
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_agent_command() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commands = agent_commands();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Dowe agent")
        .items(&commands)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| commands[index].to_string()))
}

pub(crate) fn prompt_agent_prompt() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let prompt = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Describe what Dowe should build")
        .allow_empty(false)
        .interact_text()?;

    Ok(Some(prompt))
}

pub(crate) fn prompt_agent_example_query() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let query = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Search Dowe examples")
        .allow_empty(false)
        .interact_text()?;

    Ok(Some(query))
}

pub(crate) fn prompt_deploy_surface(
    root: &std::path::Path,
    environment: DeployEnvironment,
) -> Result<Option<DeploySurface>, Box<dyn std::error::Error>> {
    let surfaces = available_deploy_surfaces(root, environment)?;
    if surfaces.is_empty() {
        return Err(
            "main.dowe does not configure a deploy surface; add `server` or `views` under `main`"
                .into(),
        );
    }
    let items = surfaces
        .iter()
        .map(|surface| surface.label())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select deploy surface")
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| surfaces[index]))
}

pub(crate) fn prompt_deploy_environment()
-> Result<Option<DeployEnvironment>, Box<dyn std::error::Error>> {
    let environments = DeployEnvironment::ALL;
    let items = environments.map(DeployEnvironment::label);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select deploy environment")
        .items(&items)
        .default(0)
        .interact_opt()?;
    Ok(selection.map(|index| environments[index]))
}

pub(crate) fn prompt_build_target() -> Result<Option<BuildTarget>, Box<dyn std::error::Error>> {
    let targets = available_build_targets();
    let items = targets
        .iter()
        .map(|target| target.label())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select build target")
        .items(&items)
        .default(0)
        .interact_opt()?;
    Ok(selection.map(|index| targets[index]))
}

pub(crate) fn prompt_deploy_target(
    surface: DeploySurface,
    default_target: Option<DeployTarget>,
) -> Result<Option<DeployTarget>, Box<dyn std::error::Error>> {
    let targets = deploy_targets_for_surface(surface);
    let items = targets
        .iter()
        .map(|target| target.label())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {surface} deploy target"))
        .items(&items)
        .default(deploy_target_default_index(targets, default_target))
        .interact_opt()?;

    Ok(selection.map(|index| targets[index]))
}

pub(crate) fn deploy_target_default_index(
    targets: &[DeployTarget],
    default_target: Option<DeployTarget>,
) -> usize {
    default_target
        .and_then(|target| targets.iter().position(|candidate| *candidate == target))
        .unwrap_or(0)
}

pub(crate) fn prompt_docker_registry(default: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Docker registry")
        .default(default.to_string())
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_docker_image(default: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Docker image name")
        .default(default.to_string())
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_ssh_host() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("SSH host")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_ssh_user() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("SSH user")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_ssh_key_file() -> Result<Option<std::path::PathBuf>, Box<dyn std::error::Error>>
{
    let methods = ["Password", "Key file"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("SSH authentication")
        .items(&methods)
        .default(0)
        .interact()?;
    if selection == 0 {
        return Ok(None);
    }
    let path = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("SSH key file")
        .allow_empty(false)
        .interact_text()?;
    Ok(Some(std::path::PathBuf::from(path)))
}

pub(crate) fn prompt_harness_command() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commands = harness_commands();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Dowe agent harness")
        .items(&commands)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| commands[index].to_string()))
}

pub(crate) fn prompt_codegraph_command() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commands = codegraph_commands();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Dowe CodeGraph")
        .items(&commands)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| commands[index].to_string()))
}

pub(crate) fn prompt_database_command() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commands = database_commands();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Database command")
        .items(&commands)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| commands[index].to_string()))
}

pub(crate) fn prompt_database_command_args(
    command: &str,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let mut args = vec![command.to_string()];
    match command {
        "start" | "list" | "bench" | "migrate" | "seeders" => {}
        "create-account" => {
            args.push(prompt_database_text("Database name")?);
            args.push(prompt_database_text("Database account")?);
        }
        "init" => args.push(prompt_database_text("Database name")?),
        "inspect" | "compact" => args.push(prompt_database_text("Database name")?),
        "query" => {
            args.push(prompt_database_text("Database name")?);
            args.push(prompt_database_text("Database query")?);
        }
        "index" => {
            args.push(prompt_database_text("Database name")?);
            args.push(prompt_database_text("Table name")?);
            args.push(prompt_database_text("Field name")?);
        }
        _ => return Ok(None),
    }
    Ok(Some(args))
}

fn prompt_database_text(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_dev_targets(
    host: HostOs,
    targets: &[DevTarget],
    defaults: &DevTargetSelection,
) -> Result<Option<DevTargetSelection>, Box<dyn std::error::Error>> {
    let items = targets
        .iter()
        .map(|target| target.as_str())
        .collect::<Vec<_>>();
    let default_states = dev_target_default_states(&targets, defaults);

    loop {
        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select dev targets")
            .items(&items)
            .defaults(&default_states)
            .interact_opt()?;

        let Some(indexes) = selection else {
            return Ok(None);
        };

        let selected = indexes
            .into_iter()
            .map(|index| targets[index])
            .collect::<Vec<_>>();

        if selected.is_empty() {
            eprintln!("Select at least one dev target.");
            continue;
        }

        return Ok(Some(DevTargetSelection::new(selected, host)?));
    }
}

pub(crate) fn prompt_dev_target_devices(
    targets: &DevTargetSelection,
    quit_simulators_on_exit_default: bool,
) -> Result<Option<DevTargetDeviceSelection>, Box<dyn std::error::Error>> {
    let mut devices = DevTargetDeviceSelection::default();

    if targets.contains(DevTarget::Android) {
        let options = available_android_devices()?;
        if options.is_empty() {
            return Err("Android target has no connected devices or virtual devices".into());
        }
        let items = options
            .iter()
            .map(|option| option.label())
            .collect::<Vec<_>>();
        let Some(index) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Android emulator or device")
            .items(&items)
            .default(0)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.android = Some(options[index].selection().clone());
    }

    if targets.contains(DevTarget::Ios) {
        let options = available_ios_simulators()?;
        if options.is_empty() {
            return Err("iOS target has no available simulators".into());
        }
        let items = options
            .iter()
            .map(|option| option.label())
            .collect::<Vec<_>>();
        let Some(index) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select iOS simulator")
            .items(&items)
            .default(0)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.ios = Some(options[index].selection());
    }

    if should_prompt_simulator_quit(targets) {
        let Some(quit_simulators_on_exit) = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Simulator quit")
            .default(quit_simulators_on_exit_default)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.quit_simulators_on_exit = quit_simulators_on_exit;
    }

    Ok(Some(devices))
}

pub(crate) fn should_prompt_simulator_quit(targets: &DevTargetSelection) -> bool {
    targets.contains(DevTarget::Android) || targets.contains(DevTarget::Ios)
}

pub(crate) fn dev_target_default_states(
    targets: &[DevTarget],
    defaults: &DevTargetSelection,
) -> Vec<bool> {
    targets
        .iter()
        .map(|target| defaults.contains(*target))
        .collect()
}

pub(crate) fn root_commands() -> [&'static str; 18] {
    [
        "dev",
        "agent",
        "ai",
        "build",
        "cache",
        "codegraph",
        "d1",
        "database",
        "deploy",
        "icons",
        "init",
        "login",
        "queue",
        "test",
        "uninstall",
        "upgrade",
        "vector",
        "version",
    ]
}

pub(crate) fn agent_commands() -> [&'static str; 2] {
    ["init", "update"]
}

pub(crate) fn harness_commands() -> [&'static str; 3] {
    ["init", "check", "status"]
}

pub(crate) fn codegraph_commands() -> [&'static str; 4] {
    ["build", "check", "report", "baseline"]
}

pub(crate) fn database_commands() -> [&'static str; 11] {
    [
        "start",
        "create-account",
        "init",
        "list",
        "inspect",
        "query",
        "index",
        "compact",
        "bench",
        "migrate",
        "seeders",
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        agent_commands, codegraph_commands, database_commands, deploy_target_default_index,
        dev_target_default_states, harness_commands, root_commands, should_prompt_simulator_quit,
    };
    use dowe_deploy::{DeploySurface, DeployTarget, deploy_targets_for_surface};
    use dowe_runtime::{DevTarget, DevTargetSelection, HostOs};

    #[test]
    fn root_menu_contains_root_cli_workflows() {
        assert_eq!(
            root_commands(),
            [
                "dev",
                "agent",
                "ai",
                "build",
                "cache",
                "codegraph",
                "d1",
                "database",
                "deploy",
                "icons",
                "init",
                "login",
                "queue",
                "test",
                "uninstall",
                "upgrade",
                "vector",
                "version"
            ]
        );
    }

    #[test]
    fn deploy_menu_contains_surface_targets() {
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Web),
            [
                DeployTarget::Dowe,
                DeployTarget::Docker,
                DeployTarget::CloudflarePages,
                DeployTarget::Vercel
            ]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Server),
            [
                DeployTarget::Dowe,
                DeployTarget::Docker,
                DeployTarget::Ssh,
                DeployTarget::Cloudflare,
                DeployTarget::Vercel
            ]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Android),
            [DeployTarget::Android]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Ios),
            [DeployTarget::Ios]
        );
    }

    #[test]
    fn deploy_target_menu_restores_saved_target_or_uses_first_available() {
        let targets = [
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::Cloudflare,
        ];

        assert_eq!(
            deploy_target_default_index(&targets, Some(DeployTarget::Docker)),
            1
        );
        assert_eq!(
            deploy_target_default_index(&targets, Some(DeployTarget::Vercel)),
            0
        );
        assert_eq!(deploy_target_default_index(&targets, None), 0);
    }

    #[test]
    fn agent_menu_contains_recommended_external_agent_workflows() {
        assert_eq!(agent_commands(), ["init", "update"]);
    }

    #[test]
    fn harness_menu_contains_interactive_safe_commands() {
        assert_eq!(harness_commands(), ["init", "check", "status"]);
    }

    #[test]
    fn codegraph_menu_contains_interactive_safe_commands() {
        assert_eq!(
            codegraph_commands(),
            ["build", "check", "report", "baseline"]
        );
    }

    #[test]
    fn database_menu_contains_all_database_commands() {
        assert_eq!(
            database_commands(),
            [
                "start",
                "create-account",
                "init",
                "list",
                "inspect",
                "query",
                "index",
                "compact",
                "bench",
                "migrate",
                "seeders"
            ]
        );
    }

    #[test]
    fn dev_target_menu_uses_supplied_defaults() {
        let targets = [
            DevTarget::Server,
            DevTarget::Web,
            DevTarget::Desktop,
            DevTarget::Android,
        ];
        let defaults =
            DevTargetSelection::new([DevTarget::Desktop, DevTarget::Android], HostOs::Linux)
                .expect("defaults");

        assert_eq!(
            dev_target_default_states(&targets, &defaults),
            [false, false, true, true]
        );
    }

    #[test]
    fn dev_target_menu_restores_persisted_server_selection() {
        let targets = [DevTarget::Server, DevTarget::Web, DevTarget::Desktop];
        let defaults =
            DevTargetSelection::new([DevTarget::Server, DevTarget::Desktop], HostOs::Linux)
                .expect("defaults");

        assert_eq!(
            dev_target_default_states(&targets, &defaults),
            [true, false, true]
        );
    }

    #[test]
    fn simulator_quit_prompt_requires_a_mobile_target() {
        let mobile =
            DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("mobile selection");
        let web = DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("web selection");

        assert!(should_prompt_simulator_quit(&mobile));
        assert!(!should_prompt_simulator_quit(&web));
    }
}
