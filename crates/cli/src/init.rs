use crate::menus;
use crate::usage::USAGE;
use dowe_runtime::{
    InitProjectOptions, InitProjectReport, ProjectTemplate, RuntimeError, init_project,
};
use std::env;

pub(crate) fn run_init_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let template = match parse_init_template(args)? {
        Some(template) => template,
        None if menus::is_interactive_terminal() => {
            let Some(template) = menus::prompt_init_template()? else {
                return Ok(());
            };
            template
        }
        None => {
            return Err(
                "dowe init requires --template or --example when no interactive terminal is available"
                    .into(),
            );
        }
    };

    let report = init_project(root, InitProjectOptions::new(template))?;
    print_init_report(&report);
    Ok(())
}

fn parse_init_template(
    args: &[String],
) -> Result<Option<ProjectTemplate>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }

    let mut index = 0usize;
    let mut selected = None;

    while index < args.len() {
        match args[index].as_str() {
            "--template" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--template requires a template name".into());
                };
                select_init_template(&mut selected, value.parse::<ProjectTemplate>()?)?;
                index += 2;
            }
            "--example" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--example requires an example name".into());
                };
                let template = value.parse::<ProjectTemplate>()?;
                if !template.is_example() {
                    return Err(
                        RuntimeError::new(format!("`{template}` is not an init example")).into(),
                    );
                }
                select_init_template(&mut selected, template)?;
                index += 2;
            }
            _ => return Err(USAGE.into()),
        }
    }

    Ok(selected)
}

fn select_init_template(
    selected: &mut Option<ProjectTemplate>,
    template: ProjectTemplate,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected.replace(template).is_some() {
        Err("dowe init accepts one --template or --example value".into())
    } else {
        Ok(())
    }
}

fn print_init_report(report: &InitProjectReport) {
    println!(
        "Initialized Dowe project with `{}` template.",
        report.template()
    );
    println!("Created {} files.", report.created().len());
    println!("Next: dowe dev --target server --target web");
}

#[cfg(test)]
mod tests {
    use super::parse_init_template;
    use dowe_runtime::ProjectTemplate;

    #[test]
    fn parses_blank_template_flag() {
        let args = vec!["--template".to_string(), "blank".to_string()];
        let template = parse_init_template(&args)
            .expect("parse")
            .expect("template");

        assert_eq!(template, ProjectTemplate::Blank);
    }

    #[test]
    fn parses_example_flag() {
        let args = vec!["--example".to_string(), "clinic-desk".to_string()];
        let template = parse_init_template(&args)
            .expect("parse")
            .expect("template");

        assert_eq!(template, ProjectTemplate::ClinicDesk);
    }

    #[test]
    fn leaves_template_to_interactive_menu_without_flags() {
        let template = parse_init_template(&[]).expect("parse");

        assert!(template.is_none());
    }

    #[test]
    fn rejects_blank_example() {
        let args = vec!["--example".to_string(), "blank".to_string()];
        let error = parse_init_template(&args).expect_err("error");

        assert!(error.to_string().contains("not an init example"));
    }

    #[test]
    fn rejects_combined_template_and_example() {
        let args = vec![
            "--template".to_string(),
            "blank".to_string(),
            "--example".to_string(),
            "clinic-desk".to_string(),
        ];
        let error = parse_init_template(&args).expect_err("error");

        assert!(error.to_string().contains("one --template or --example"));
    }
}
