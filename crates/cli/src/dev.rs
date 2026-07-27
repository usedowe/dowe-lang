use crate::menus;
use crate::usage::USAGE;
use dowe_runtime::{
    DevRunOptions, DevTarget, DevTargetSelection, HostOs, RuntimeError,
    available_dev_targets_for_project, default_dev_targets_for_project,
    load_dev_target_preferences_for_project, run_dev_with_options, save_dev_target_preferences,
    validate_dev_target_selection_for_project,
};
use std::env;

pub(crate) async fn run_dev_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let host = HostOs::current();
    let root = env::current_dir()?;
    let available_targets = available_dev_targets_for_project(&root, host)?;
    let (selection, quit_simulators_on_exit_default, persist_preferences) =
        match parse_dev_targets(args, host)? {
            Some(selection) => {
                validate_dev_target_selection_for_project(&root, host, &selection)?;
                (selection, true, false)
            }
            None if menus::is_interactive_terminal() => {
                let preferences = load_dev_target_preferences_for_project(&root, host)?;
                let defaults = preferences
                    .as_ref()
                    .map(|preferences| preferences.selection.clone())
                    .map(Ok)
                    .unwrap_or_else(|| default_dev_targets_for_project(&root, host))?;
                let quit_simulators_on_exit = preferences
                    .as_ref()
                    .map(|preferences| preferences.quit_simulators_on_exit)
                    .unwrap_or(true);
                let Some(selection) =
                    menus::prompt_dev_targets(host, &available_targets, &defaults)?
                else {
                    return Ok(());
                };
                save_dev_target_preferences(&root, &selection, quit_simulators_on_exit)?;
                (selection, quit_simulators_on_exit, true)
            }
            None => {
                return Err(
                    "dowe dev requires --target when no interactive terminal is available".into(),
                );
            }
        };

    let devices = if menus::is_interactive_terminal() {
        let Some(devices) =
            menus::prompt_dev_target_devices(&selection, quit_simulators_on_exit_default)?
        else {
            return Ok(());
        };
        if persist_preferences && menus::should_prompt_simulator_quit(&selection) {
            save_dev_target_preferences(&root, &selection, devices.quit_simulators_on_exit)?;
        }
        devices
    } else {
        Default::default()
    };

    run_dev_with_options(root, selection, DevRunOptions { devices }).await?;
    Ok(())
}

fn parse_dev_targets(
    args: &[String],
    host: HostOs,
) -> Result<Option<DevTargetSelection>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }

    let mut index = 0usize;
    let mut targets = Vec::new();

    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--target requires a target name".into());
                };
                let target = value.parse::<DevTarget>().map_err(RuntimeError::new)?;
                targets.push(target);
                index += 2;
            }
            _ => return Err(USAGE.into()),
        }
    }

    Ok(Some(DevTargetSelection::new(targets, host)?))
}

#[cfg(test)]
mod tests {
    use super::parse_dev_targets;
    use dowe_runtime::{DevTarget, HostOs};

    #[test]
    fn parses_explicit_dev_targets() {
        let args = vec![
            "--target".to_string(),
            "web".to_string(),
            "--target".to_string(),
            "server".to_string(),
            "--target".to_string(),
            "web".to_string(),
        ];
        let selection = parse_dev_targets(&args, HostOs::Linux)
            .expect("parse")
            .expect("selection");

        assert_eq!(selection.targets(), &[DevTarget::Server, DevTarget::Web]);
    }

    #[test]
    fn leaves_target_selection_to_interactive_menu_without_flags() {
        let selection = parse_dev_targets(&[], HostOs::Linux).expect("parse");

        assert!(selection.is_none());
    }

    #[test]
    fn rejects_missing_target_value() {
        let args = vec!["--target".to_string()];
        let error = parse_dev_targets(&args, HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("--target requires"));
    }

    #[test]
    fn rejects_ios_target_on_linux() {
        let args = vec!["--target".to_string(), "ios".to_string()];
        let error = parse_dev_targets(&args, HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("ios"));
    }
}
