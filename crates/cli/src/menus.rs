use dialoguer::{Input, MultiSelect, Select, theme::ColorfulTheme};
use dowe_deploy::DeployTarget;
use dowe_runtime::{
    DevTarget, DevTargetSelection, HostOs, ProjectTemplate, available_dev_targets,
    available_project_templates,
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
        .map(|template| template.as_str())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select init template")
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| templates[index]))
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

pub(crate) fn prompt_deploy_target() -> Result<Option<DeployTarget>, Box<dyn std::error::Error>> {
    let targets = deploy_targets();
    let items = targets
        .iter()
        .map(|target| target.as_str())
        .collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select deploy target")
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|index| targets[index]))
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

pub(crate) fn prompt_dev_targets(
    host: HostOs,
    defaults: &DevTargetSelection,
) -> Result<Option<DevTargetSelection>, Box<dyn std::error::Error>> {
    let targets = available_dev_targets(host);
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

pub(crate) fn dev_target_default_states(
    targets: &[DevTarget],
    defaults: &DevTargetSelection,
) -> Vec<bool> {
    targets
        .iter()
        .map(|target| defaults.contains(*target))
        .collect()
}

pub(crate) fn root_commands() -> [&'static str; 8] {
    [
        "init",
        "dev",
        "deploy",
        "agent",
        "codegraph",
        "kv",
        "store",
        "upgrade",
    ]
}

pub(crate) fn deploy_targets() -> [DeployTarget; 4] {
    [
        DeployTarget::Static,
        DeployTarget::Docker,
        DeployTarget::Ssh,
        DeployTarget::Cloudflare,
    ]
}

pub(crate) fn agent_commands() -> [&'static str; 2] {
    ["chat", "harness"]
}

pub(crate) fn harness_commands() -> [&'static str; 3] {
    ["init", "check", "status"]
}

pub(crate) fn codegraph_commands() -> [&'static str; 4] {
    ["build", "check", "report", "baseline"]
}

#[cfg(test)]
mod tests {
    use super::{
        agent_commands, codegraph_commands, deploy_targets, dev_target_default_states,
        harness_commands, root_commands,
    };
    use dowe_deploy::DeployTarget;
    use dowe_runtime::{DevTarget, DevTargetSelection, HostOs};

    #[test]
    fn root_menu_contains_root_cli_workflows() {
        assert_eq!(
            root_commands(),
            [
                "init",
                "dev",
                "deploy",
                "agent",
                "codegraph",
                "kv",
                "store",
                "upgrade"
            ]
        );
    }

    #[test]
    fn deploy_menu_contains_portable_targets() {
        assert_eq!(
            deploy_targets(),
            [
                DeployTarget::Static,
                DeployTarget::Docker,
                DeployTarget::Ssh,
                DeployTarget::Cloudflare,
            ]
        );
    }

    #[test]
    fn agent_menu_contains_chat_and_harness() {
        assert_eq!(agent_commands(), ["chat", "harness"]);
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
}
