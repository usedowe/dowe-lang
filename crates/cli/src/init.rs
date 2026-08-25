use crate::menus;
use crate::usage::USAGE;
use dowe_agent::{DoweProjectInitReport, init_dowe_project};
use dowe_runtime::{InitProjectOptions, ProjectTemplate, has_dowe_project_marker};
use std::env;

pub(crate) fn run_init_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let parsed_options = parse_init_options(args)?;
    let interactive = menus::is_interactive_terminal();
    let reinstall = if has_dowe_project_marker(&root) {
        if !interactive {
            return Err(
                "cannot initialize this Dowe project because `main.dowe` already exists; run `dowe init` in an interactive terminal to confirm reinstallation"
                    .into(),
            );
        }
        let Some(confirmed) = menus::prompt_init_reinstall()? else {
            return Ok(());
        };
        if !confirmed {
            return Ok(());
        }
        true
    } else {
        false
    };
    let options = match parsed_options {
        Some(options) => options,
        None if interactive => {
            let Some(options) = prompt_init_options()? else {
                return Ok(());
            };
            options
        }
        None => {
            return Err(
                "dowe init requires --template when no interactive terminal is available".into(),
            );
        }
    }
    .with_reinstall(reinstall);

    let report = init_dowe_project(root, options)?;
    print_init_report(&report);
    Ok(())
}

fn prompt_init_options() -> Result<Option<InitProjectOptions>, Box<dyn std::error::Error>> {
    let Some(template) = menus::prompt_init_template()? else {
        return Ok(None);
    };
    let Some(i18n) = menus::prompt_init_i18n()? else {
        return Ok(None);
    };
    Ok(Some(InitProjectOptions::new(template).with_i18n(i18n)))
}

fn parse_init_options(
    args: &[String],
) -> Result<Option<InitProjectOptions>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }

    let mut index = 0usize;
    let mut template = None;
    let mut i18n = false;

    while index < args.len() {
        match args[index].as_str() {
            "--template" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--template requires a template name".into());
                };
                if template.replace(value.clone()).is_some() {
                    return Err("dowe init accepts one --template value".into());
                }
                index += 2;
            }
            "--i18n" => {
                if i18n {
                    return Err("dowe init accepts --i18n once".into());
                }
                i18n = true;
                index += 1;
            }
            _ => return Err(USAGE.into()),
        }
    }

    let Some(template) = template else {
        return Err("dowe init requires --template when init options are supplied".into());
    };
    let template = resolve_template_name(&template)?;
    Ok(Some(InitProjectOptions::new(template).with_i18n(i18n)))
}

fn resolve_template_name(value: &str) -> Result<ProjectTemplate, Box<dyn std::error::Error>> {
    match value {
        "cloudflare-crud" | "docker-crud" => Ok(ProjectTemplate::Crud),
        "cloudflare-blank" | "docker-blank" => Ok(ProjectTemplate::Blank),
        _ => Ok(value.parse::<ProjectTemplate>()?),
    }
}

fn print_init_report(report: &DoweProjectInitReport) {
    let action = if report.project.reinstalled() {
        "Reinstalled"
    } else {
        "Initialized"
    };
    if report.project.i18n_enabled() {
        println!(
            "{action} Dowe project with `{}` template and i18n.",
            report.project.template()
        );
    } else {
        println!(
            "{action} Dowe project with `{}` template.",
            report.project.template()
        );
    }
    println!(
        "{} {} project files and {} agent files.",
        if report.project.reinstalled() {
            "Replaced"
        } else {
            "Created"
        },
        report.project.created().len(),
        report.agent.created.len()
    );
    println!("Next: dowe dev --target server --target web");
}

#[cfg(test)]
mod tests {
    use super::parse_init_options;
    use dowe_runtime::ProjectTemplate;

    #[test]
    fn parses_canonical_init_flags() {
        for (args, template, i18n) in [
            (vec!["--template", "crud"], ProjectTemplate::Crud, false),
            (
                vec!["--template", "crud", "--i18n"],
                ProjectTemplate::Crud,
                true,
            ),
            (
                vec!["--template", "blank", "--i18n"],
                ProjectTemplate::Blank,
                true,
            ),
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            let options = parse_init_options(&args).expect("parse").expect("options");

            assert_eq!(options.template(), template);
            assert_eq!(options.i18n_enabled(), i18n);
        }
    }
    #[test]
    fn leaves_template_to_interactive_menu_without_flags() {
        let template = parse_init_options(&[]).expect("parse");

        assert!(template.is_none());
    }

    #[test]
    fn rejects_removed_example_flag() {
        let args = vec!["--example".to_string(), "clinic-desk".to_string()];
        let error = parse_init_options(&args).expect_err("error");

        assert!(error.to_string().contains("Usage:"));
    }

    #[test]
    fn rejects_multiple_template_values() {
        let args = vec![
            "--template".to_string(),
            "blank".to_string(),
            "--template".to_string(),
            "blank".to_string(),
        ];
        let error = parse_init_options(&args).expect_err("error");

        assert!(error.to_string().contains("one --template value"));
    }
    #[test]
    fn rejects_removed_database_option() {
        let args = vec!["--database".to_string(), "db".to_string()];
        let error = parse_init_options(&args).expect_err("error");

        assert!(error.to_string().contains("Usage:"));
    }
}
