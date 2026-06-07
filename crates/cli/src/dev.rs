use crate::menus;
use crate::usage::USAGE;
use dowe_runtime::{
    DevTarget, DevTargetSelection, HostOs, RuntimeError, default_dev_targets,
    load_dev_target_selection, run_dev, save_dev_target_selection,
};
use std::env;

pub(crate) async fn run_dev_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let host = HostOs::current();
    let root = env::current_dir()?;
    let selection = match parse_dev_targets(args, host)? {
        Some(selection) => selection,
        None if menus::is_interactive_terminal() => {
            let defaults = load_dev_target_selection(&root, host)?
                .unwrap_or_else(|| default_dev_targets(host));
            let Some(selection) = menus::prompt_dev_targets(host, &defaults)? else {
                return Ok(());
            };
            save_dev_target_selection(&root, &selection)?;
            selection
        }
        None => {
            return Err(
                "dowe dev requires --target when no interactive terminal is available".into(),
            );
        }
    };

    run_dev(root, selection).await?;
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
