use crate::icons::generate_and_report;
use crate::menus;
use crate::usage::USAGE;
use dowe_runtime::{
    InitProjectOptions, InitProjectReport, ProjectTemplate, has_dowe_project_marker, init_project,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn run_init_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let parsed_options = parse_init_options(args)?;
    let interactive = menus::is_interactive_terminal();
    let (options, icon_source, destination) = match parsed_options {
        Some(options) => (options, None, root.clone()),
        None if interactive => {
            let Some((options, icon_source, destination)) = prompt_init_options(&root)? else {
                return Ok(());
            };
            (options, icon_source, destination)
        }
        None => {
            return Err(
                "dowe init requires --template when no interactive terminal is available".into(),
            );
        }
    };
    let reinstall = if should_check_reinstall(
        &root,
        &destination,
        has_dowe_project_marker(&destination),
    ) {
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
    let report = init_project(&destination, options.with_reinstall(reinstall))?;
    print_init_report(&report);
    if let Some(source) = icon_source {
        generate_and_report(
            &destination,
            &source,
            "#ffffff".to_string(),
            dowe_icons::IconRounded::None,
            dowe_icons::IconTarget::ALL.to_vec(),
        )?;
    }
    Ok(())
}

fn prompt_init_options(
    root: &Path,
) -> Result<Option<(InitProjectOptions, Option<String>, PathBuf)>, Box<dyn std::error::Error>> {
    let Some(template) = menus::prompt_init_template()? else {
        return Ok(None);
    };
    let app_name = menus::prompt_init_app_name()?;
    let bundle = menus::prompt_init_bundle()?;
    let Some(new_subfolder) = menus::prompt_init_destination()? else {
        return Ok(None);
    };
    let destination = if new_subfolder {
        let slug = safe_app_slug(&app_name)?;
        let destination = root.join(slug);
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(format!(
                "cannot initialize project because destination already exists: {}",
                destination.display()
            )
            .into());
        }
        destination
    } else {
        root.to_path_buf()
    };
    let Some(i18n) = menus::prompt_init_i18n()? else {
        return Ok(None);
    };
    let icon_source = menus::prompt_init_icon_source()?;
    Ok(Some((
        InitProjectOptions::new(template)
            .with_app_identity(app_name, bundle)
            .with_i18n(i18n),
        icon_source,
        destination,
    )))
}

fn should_check_reinstall(root: &Path, destination: &Path, marker_exists: bool) -> bool {
    destination == root && marker_exists
}

fn safe_app_slug(app_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut slug = String::new();
    for character in app_name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if character.is_ascii_whitespace() || matches!(character, '-' | '_') {
            if !slug.ends_with('-') && !slug.is_empty() {
                slug.push('-');
            }
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        return Err("app name does not produce a safe destination subfolder name".into());
    }
    Ok(slug)
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

fn print_init_report(report: &InitProjectReport) {
    let action = if report.reinstalled() {
        "Reinstalled"
    } else {
        "Initialized"
    };
    if report.i18n_enabled() {
        println!(
            "{action} Dowe project with `{}` template and i18n.",
            report.template()
        );
    } else {
        println!(
            "{action} Dowe project with `{}` template.",
            report.template()
        );
    }
    println!(
        "{} {} project files.",
        if report.reinstalled() {
            "Replaced"
        } else {
            "Created"
        },
        report.created().len()
    );
    println!("Next: dowe dev --target server --target web");
}

#[cfg(test)]
mod tests {
    use super::{parse_init_options, safe_app_slug, should_check_reinstall};
    use dowe_runtime::ProjectTemplate;
    use std::path::Path;

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
    fn checks_reinstall_only_for_an_existing_current_directory() {
        let root = Path::new("/workspace");
        let subfolder = root.join("my-app");

        assert!(should_check_reinstall(root, root, true));
        assert!(!should_check_reinstall(root, root, false));
        assert!(!should_check_reinstall(root, &subfolder, true));
    }

    #[test]
    fn creates_safe_subfolder_slugs_from_app_names() {
        assert_eq!(safe_app_slug("My App").expect("slug"), "my-app");
        assert_eq!(safe_app_slug("  My_App  ").expect("slug"), "my-app");
    }

    #[test]
    fn rejects_empty_subfolder_slugs() {
        assert!(safe_app_slug("!!!").is_err());
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
