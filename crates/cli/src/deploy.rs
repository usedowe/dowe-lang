use crate::menus;
use crate::usage::USAGE;
use dowe_deploy::{DeployOptions, DeploySurface, DeployTarget, default_docker_image_name, deploy};
use std::env;
use std::path::PathBuf;

pub(crate) fn run_deploy_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    if let Some(surface) = args
        .first()
        .and_then(|value| value.parse::<DeploySurface>().ok())
    {
        return run_surface_deploy(surface, &args[1..], root);
    }
    let Some(options) = parse_deploy_options(args, root.clone())? else {
        if !menus::is_interactive_terminal() {
            return Err(
                "dowe deploy requires --target when no interactive terminal is available".into(),
            );
        }
        let Some(surface) = menus::prompt_deploy_surface(&root)? else {
            return Ok(());
        };
        let Some(target) = menus::prompt_deploy_target(surface)? else {
            return Ok(());
        };
        let mut options = DeployOptions::new(root, target);
        configure_interactive_docker(&mut options)?;
        options.publish = should_auto_publish(surface, target);
        let report = deploy(options)?;
        print_report(&report);
        return Ok(());
    };
    let report = deploy(options)?;
    print_report(&report);
    Ok(())
}

fn run_surface_deploy(
    surface: DeploySurface,
    args: &[String],
    root: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let interactive = args.is_empty() && menus::is_interactive_terminal();
    let interactive_target = if interactive {
        menus::prompt_deploy_target(surface)?
    } else {
        None
    };
    if interactive && interactive_target.is_none() {
        return Ok(());
    }
    let default_target = match (surface, interactive_target) {
        (_, Some(target)) => Some(target),
        (DeploySurface::Web, None) => Some(DeployTarget::CloudflarePages),
        (DeploySurface::Server, None) => None,
    };
    let mut options = parse_deploy_flags(args, root, default_target)?;
    if options.target.surface() != surface {
        return Err(format!(
            "deploy target `{}` does not belong to the `{surface}` deploy surface",
            options.target
        )
        .into());
    }
    if interactive {
        configure_interactive_docker(&mut options)?;
        options.publish = should_auto_publish(surface, options.target);
    }
    let report = deploy(options)?;
    print_report(&report);
    Ok(())
}

fn should_auto_publish(surface: DeploySurface, target: DeployTarget) -> bool {
    matches!(
        (surface, target),
        (DeploySurface::Web, DeployTarget::CloudflarePages)
            | (DeploySurface::Server, DeployTarget::Cloudflare)
    )
}

fn parse_deploy_options(
    args: &[String],
    root: PathBuf,
) -> Result<Option<DeployOptions>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }
    Ok(Some(parse_deploy_flags(args, root, None)?))
}

fn parse_deploy_flags(
    args: &[String],
    root: PathBuf,
    default_target: Option<DeployTarget>,
) -> Result<DeployOptions, Box<dyn std::error::Error>> {
    let mut index = 0usize;
    let mut target = default_target;
    let mut name = None;
    let mut publish = false;
    let mut dry_run = false;
    let mut registry = None;
    let mut image = None;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                target = Some(required_value(args, index, "--target")?.parse::<DeployTarget>()?);
                index += 2;
            }
            "--name" => {
                name = Some(required_value(args, index, "--name")?.to_string());
                index += 2;
            }
            "--publish" => {
                publish = true;
                index += 1;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--registry" => {
                registry = Some(required_value(args, index, "--registry")?.to_string());
                index += 2;
            }
            "--image" => {
                image = Some(required_value(args, index, "--image")?.to_string());
                index += 2;
            }
            _ => return Err(USAGE.into()),
        }
    }
    let target = target.ok_or("dowe deploy requires --target")?;
    let mut options = DeployOptions::new(root, target);
    options.name = name;
    options.publish = publish;
    options.dry_run = dry_run;
    options.registry = registry;
    options.image = image;
    Ok(options)
}

fn configure_interactive_docker(
    options: &mut DeployOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if options.target != DeployTarget::Docker {
        return Ok(());
    }
    let default_image = default_docker_image_name(&options.root);
    options.registry = Some(menus::prompt_docker_registry()?);
    options.image = Some(menus::prompt_docker_image(&default_image)?);
    Ok(())
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    name: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("{name} requires a value").into())
}

fn print_report(report: &dowe_deploy::DeployReport) {
    println!(
        "{} deploy package written to {}",
        report.target,
        report.output_dir.display()
    );
    if report.published {
        println!("{} deploy published", report.target);
    }
    if let Some(image) = report.image_ref.as_deref() {
        if report.image_built {
            println!("Docker image built as {image}");
        } else {
            println!("Docker image not built; context and Dockerfile are available for {image}");
            if let Some(command) = report.command.as_ref() {
                println!("Docker build command: {}", command.join(" "));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_deploy_options;
    use super::should_auto_publish;
    use dowe_deploy::DeploySurface;
    use dowe_deploy::DeployTarget;
    use std::path::PathBuf;

    #[test]
    fn parses_cloudflare_publish_options() {
        let args = vec![
            "--target".to_string(),
            "cloudflare".to_string(),
            "--name".to_string(),
            "docs-app".to_string(),
            "--publish".to_string(),
            "--dry-run".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Cloudflare);
        assert_eq!(options.name.as_deref(), Some("docs-app"));
        assert!(options.publish);
        assert!(options.dry_run);
    }

    #[test]
    fn parses_docker_registry_and_image() {
        let args = vec![
            "--target".to_string(),
            "docker".to_string(),
            "--registry".to_string(),
            "ghcr.io/dowe".to_string(),
            "--image".to_string(),
            "docs-app:stable".to_string(),
            "--dry-run".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Docker);
        assert_eq!(options.registry.as_deref(), Some("ghcr.io/dowe"));
        assert_eq!(options.image.as_deref(), Some("docs-app:stable"));
        assert!(options.dry_run);
    }

    #[test]
    fn leaves_target_to_menu_without_args() {
        assert!(
            parse_deploy_options(&[], PathBuf::from("/project"))
                .expect("parse")
                .is_none()
        );
    }

    #[test]
    fn defaults_web_surface_to_cloudflare_pages() {
        let options = super::parse_deploy_flags(
            &["--publish".to_string()],
            PathBuf::from("/project"),
            Some(DeployTarget::CloudflarePages),
        )
        .expect("options");

        assert_eq!(options.target, DeployTarget::CloudflarePages);
        assert!(options.publish);
    }

    #[test]
    fn interactive_web_pages_deploy_publishes_automatically() {
        assert!(should_auto_publish(
            DeploySurface::Web,
            DeployTarget::CloudflarePages
        ));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Cloudflare
        ));
        assert!(!should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Docker
        ));
    }
}
