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

pub(crate) fn prompt_init_app_name() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("App name")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_init_bundle() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("App bundle")
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_init_destination() -> Result<Option<bool>, Box<dyn std::error::Error>> {
    let choices = ["Current directory", "New subfolder"];
    Ok(Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Install project in")
        .items(&choices)
        .default(0)
        .interact_opt()?
        .map(|index| index == 1))
}

pub(crate) fn prompt_init_icon_source() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let source = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Icon SVG path (optional)")
        .allow_empty(true)
        .interact_text()?;
    Ok((!source.trim().is_empty()).then_some(source))
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

