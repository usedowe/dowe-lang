use crate::menus;
use crate::usage::USAGE;
use dowe_deploy::{DeployOptions, DeployTarget, deploy};
use std::env;
use std::path::PathBuf;

pub(crate) fn run_deploy_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let Some(options) = parse_deploy_options(args, root)? else {
        if !menus::is_interactive_terminal() {
            return Err(
                "dowe deploy requires --target when no interactive terminal is available".into(),
            );
        }
        let Some(target) = menus::prompt_deploy_target()? else {
            return Ok(());
        };
        let report = deploy(DeployOptions::new(env::current_dir()?, target))?;
        print_report(&report);
        return Ok(());
    };
    let report = deploy(options)?;
    print_report(&report);
    Ok(())
}

fn parse_deploy_options(
    args: &[String],
    root: PathBuf,
) -> Result<Option<DeployOptions>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }
    let mut index = 0usize;
    let mut target = None;
    let mut name = None;
    let mut publish = false;
    let mut dry_run = false;
    let mut host = None;
    let mut remote_dir = None;
    let mut server_binary = None;
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
            "--host" => {
                host = Some(required_value(args, index, "--host")?.to_string());
                index += 2;
            }
            "--remote-dir" => {
                remote_dir = Some(required_value(args, index, "--remote-dir")?.to_string());
                index += 2;
            }
            "--server-binary" => {
                server_binary = Some(PathBuf::from(required_value(
                    args,
                    index,
                    "--server-binary",
                )?));
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
    options.host = host;
    options.remote_dir = remote_dir;
    options.server_binary = server_binary;
    Ok(Some(options))
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
}

#[cfg(test)]
mod tests {
    use super::parse_deploy_options;
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
    fn leaves_target_to_menu_without_args() {
        assert!(
            parse_deploy_options(&[], PathBuf::from("/project"))
                .expect("parse")
                .is_none()
        );
    }
}
