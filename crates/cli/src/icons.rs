use crate::menus;
use crate::usage::USAGE;
use dowe_icons::{GenerateIconOptions, IconRounded, IconTarget, generate_project_icons};
use std::env;
use std::str::FromStr;

pub(crate) fn run_icons_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_icon_args(args)?;
    let (source, background, rounded, targets) = if args.is_empty() {
        if !menus::is_interactive_terminal() {
            return Err(
                "dowe icons requires --source, --background and --rounded when no interactive terminal is available"
                    .into(),
            );
        }
        let source = menus::prompt_icon_source()?;
        let background = menus::prompt_icon_background()?;
        let Some(rounded) = menus::prompt_icon_rounded()? else {
            return Ok(());
        };
        let Some(targets) = menus::prompt_icon_targets()? else {
            return Ok(());
        };
        (source, background, rounded, targets)
    } else {
        let (Some(source), Some(background), Some(rounded)) =
            (parsed.source, parsed.background, parsed.rounded)
        else {
            return Err(
                "dowe icons requires --source, --background and --rounded when arguments are provided"
                    .into(),
            );
        };
        let targets = if parsed.targets.is_empty() {
            IconTarget::ALL.to_vec()
        } else {
            parsed.targets
        };
        (source, background, rounded, targets)
    };
    let report = generate_project_icons(
        GenerateIconOptions::new(env::current_dir()?, source, background, rounded)
            .with_targets(targets),
    )?;
    for target in &report.targets {
        let count = report
            .files
            .iter()
            .filter(|path| path.starts_with(std::path::Path::new("icons").join(target.as_str())))
            .count();
        println!(
            "Generated {count} {} icon files in icons/{}",
            target.as_str(),
            target.as_str()
        );
    }
    Ok(())
}

#[derive(Default)]
struct ParsedIconArgs {
    source: Option<String>,
    background: Option<String>,
    rounded: Option<IconRounded>,
    targets: Vec<IconTarget>,
}

fn parse_icon_args(args: &[String]) -> Result<ParsedIconArgs, Box<dyn std::error::Error>> {
    let mut parsed = ParsedIconArgs::default();
    let mut index = 0usize;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = args.get(index + 1).ok_or_else(|| match flag {
            "--source" => "--source requires a project-relative SVG path",
            "--background" => "--background requires a #RRGGBB value",
            "--rounded" => "--rounded requires none, xs, sm, md, lg, xl or full",
            "--target" => "--target requires web, desktop, ios or android",
            _ => USAGE,
        })?;
        match flag {
            "--source" => set_once(&mut parsed.source, value, "--source")?,
            "--background" => set_once(&mut parsed.background, value, "--background")?,
            "--rounded" => {
                if parsed.rounded.is_some() {
                    return Err("--rounded can be provided only once".into());
                }
                parsed.rounded = Some(IconRounded::from_str(value)?);
            }
            "--target" => parsed.targets.push(IconTarget::from_str(value)?),
            _ => return Err(USAGE.into()),
        }
        index += 2;
    }
    Ok(parsed)
}

fn set_once(
    slot: &mut Option<String>,
    value: &str,
    flag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if slot.is_some() {
        return Err(format!("{flag} can be provided only once").into());
    }
    *slot = Some(value.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_icon_args;
    use dowe_icons::{IconRounded, IconTarget};

    #[test]
    fn parses_icon_options_and_deduplicates_in_shared_api() {
        let args = [
            "--source",
            "icon.svg",
            "--background",
            "#ffffff",
            "--rounded",
            "md",
            "--target",
            "web",
            "--target",
            "ios",
        ]
        .map(str::to_string);
        let parsed = parse_icon_args(&args).expect("options");

        assert_eq!(parsed.source.as_deref(), Some("icon.svg"));
        assert_eq!(parsed.background.as_deref(), Some("#ffffff"));
        assert_eq!(parsed.rounded, Some(IconRounded::Md));
        assert_eq!(parsed.targets, [IconTarget::Web, IconTarget::Ios]);
    }

    #[test]
    fn rejects_duplicate_scalar_options() {
        let args = ["--source", "icon.svg", "--source", "other.svg"].map(str::to_string);
        let error = parse_icon_args(&args).err().expect("error");

        assert!(error.to_string().contains("only once"));
    }
}
