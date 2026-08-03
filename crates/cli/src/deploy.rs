use crate::menus;
use crate::usage::USAGE;
use dowe_deploy::{
    DeployEnvironment, DeployOptions, DeploySurface, DeployTarget, default_docker_image_name,
    deploy,
};
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
        let Some(environment) = menus::prompt_deploy_environment()? else {
            return Ok(());
        };
        let Some(surface) = menus::prompt_deploy_surface(&root, environment)? else {
            return Ok(());
        };
        let Some(target) = menus::prompt_deploy_target(surface)? else {
            return Ok(());
        };
        let mut options = DeployOptions::new(root, target);
        options.environment = environment;
        options.surface = Some(surface);
        configure_interactive_target(&mut options)?;
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
    let interactive_environment = if interactive {
        menus::prompt_deploy_environment()?
    } else {
        None
    };
    if interactive && interactive_environment.is_none() {
        return Ok(());
    }
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
        (DeploySurface::Android, None) => Some(DeployTarget::Android),
        (DeploySurface::Ios, None) => Some(DeployTarget::Ios),
        (DeploySurface::Server, None) => None,
    };
    let mut options = parse_deploy_flags(args, root, default_target)?;
    options.surface = Some(surface);
    if let Some(environment) = interactive_environment {
        options.environment = environment;
    }
    if options.target != DeployTarget::Dowe && options.target.surface() != surface {
        return Err(format!(
            "deploy target `{}` does not belong to the `{surface}` deploy surface",
            options.target
        )
        .into());
    }
    if interactive {
        configure_interactive_target(&mut options)?;
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
            | (DeploySurface::Server, DeployTarget::Dowe)
            | (DeploySurface::Server, DeployTarget::Ssh)
            | (DeploySurface::Web, DeployTarget::Dowe)
            | (DeploySurface::Android, DeployTarget::Android)
            | (DeploySurface::Ios, DeployTarget::Ios)
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
    let mut track = None;
    let mut environment = DeployEnvironment::Live;
    let mut ssh_host = None;
    let mut ssh_user = None;
    let mut ssh_key_file = None;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                target = Some(required_value(args, index, "--target")?.parse::<DeployTarget>()?);
                index += 2;
            }
            "--environment" => {
                environment =
                    required_value(args, index, "--environment")?.parse::<DeployEnvironment>()?;
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
            "--track" => {
                track = Some(required_value(args, index, "--track")?.to_string());
                index += 2;
            }
            "--host" => {
                ssh_host = Some(required_value(args, index, "--host")?.to_string());
                index += 2;
            }
            "--user" => {
                ssh_user = Some(required_value(args, index, "--user")?.to_string());
                index += 2;
            }
            "--key-file" => {
                ssh_key_file = Some(PathBuf::from(required_value(args, index, "--key-file")?));
                index += 2;
            }
            _ => return Err(USAGE.into()),
        }
    }
    let target = target.ok_or("dowe deploy requires --target")?;
    if track.is_some() && target != DeployTarget::Android {
        return Err("--track is only valid for the Android deploy target".into());
    }
    if target != DeployTarget::Ssh
        && (ssh_host.is_some() || ssh_user.is_some() || ssh_key_file.is_some())
    {
        return Err(
            "--host, --user and --key-file are only valid for the SSH deploy target".into(),
        );
    }
    let mut options = DeployOptions::new(root, target);
    options.environment = environment;
    options.name = name;
    options.publish = publish;
    options.dry_run = dry_run;
    options.registry = registry;
    options.image = image;
    options.track = track;
    options.ssh_host = ssh_host;
    options.ssh_user = ssh_user;
    options.ssh_key_file = ssh_key_file;
    Ok(options)
}

fn configure_interactive_target(
    options: &mut DeployOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    match options.target {
        DeployTarget::Docker => {
            let default_image = default_docker_image_name(&options.root);
            options.registry = Some(menus::prompt_docker_registry()?);
            options.image = Some(menus::prompt_docker_image(&default_image)?);
        }
        DeployTarget::Ssh => {
            options.ssh_host = Some(menus::prompt_ssh_host()?);
            options.ssh_user = Some(menus::prompt_ssh_user()?);
            options.ssh_key_file = menus::prompt_ssh_key_file()?;
        }
        _ => {}
    }
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
        "{} {} deploy package written to {}",
        report.environment,
        report.target,
        report.output_dir.display()
    );
    if report.published {
        println!("{} {} deploy published", report.environment, report.target);
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
    if let Some(artifact) = report.artifact.as_deref() {
        println!(
            "{} artifact written to {}",
            report.target,
            artifact.display()
        );
    }
    if let Some(url) = report.url.as_deref() {
        println!("Dowe Cloud URL: {url}");
    }
    if report.target == DeployTarget::Ssh && !report.published {
        if let Some(command) = report.command.as_ref() {
            println!("SSH install command: {}", command.join(" "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_deploy_options;
    use super::should_auto_publish;
    use dowe_deploy::DeployEnvironment;
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
    fn parses_stage_environment() {
        let args = vec![
            "--target".to_string(),
            "cloudflare-pages".to_string(),
            "--environment".to_string(),
            "stage".to_string(),
        ];

        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.environment, DeployEnvironment::Stage);
    }

    #[test]
    fn parses_ssh_publish_options_without_password() {
        let args = vec![
            "--target".to_string(),
            "ssh".to_string(),
            "--host".to_string(),
            "server.example.com".to_string(),
            "--user".to_string(),
            "deploy".to_string(),
            "--key-file".to_string(),
            "/keys/deploy".to_string(),
            "--publish".to_string(),
        ];

        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Ssh);
        assert_eq!(options.ssh_host.as_deref(), Some("server.example.com"));
        assert_eq!(options.ssh_user.as_deref(), Some("deploy"));
        assert_eq!(options.ssh_key_file, Some(PathBuf::from("/keys/deploy")));
        assert!(options.publish);
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
    fn parses_android_store_track() {
        let args = vec![
            "--target".to_string(),
            "android".to_string(),
            "--track".to_string(),
            "beta".to_string(),
            "--publish".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Android);
        assert_eq!(options.track.as_deref(), Some("beta"));
        assert!(options.publish);
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
        assert!(should_auto_publish(
            DeploySurface::Android,
            DeployTarget::Android
        ));
        assert!(should_auto_publish(DeploySurface::Ios, DeployTarget::Ios));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Dowe
        ));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Ssh
        ));
        assert!(should_auto_publish(DeploySurface::Web, DeployTarget::Dowe));
    }
}
