use crate::menus;
use crate::usage::USAGE;
use dowe_deploy::{BuildOptions, BuildTarget, build};
use std::env;

pub(crate) async fn run_build_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let mut target = None;
    let mut dry_run = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                target = Some(
                    args.get(index + 1)
                        .ok_or("--target requires a value")?
                        .parse::<BuildTarget>()?,
                );
                index += 2;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            _ => return Err(USAGE.into()),
        }
    }
    if target.is_none() && menus::is_interactive_terminal() {
        target = menus::prompt_build_target()?;
        if target.is_none() {
            return Ok(());
        }
    }
    let target =
        target.ok_or("dowe build requires --target when no interactive terminal is available")?;
    let mut options = BuildOptions::new(root, target);
    options.dry_run = dry_run;
    println!(
        "Preparing {} {}...",
        target,
        if dry_run { "build plan" } else { "build" }
    );
    let report = tokio::task::spawn_blocking(move || build(options)).await??;
    if report.built {
        println!(
            "{} build written to {}",
            report.target,
            report.artifact.display()
        );
    } else {
        println!(
            "{} build plan written to {}",
            report.target,
            report.output_dir.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use dowe_deploy::BuildTarget;

    #[test]
    fn build_targets_parse_from_cli_values() {
        for value in ["android", "ios", "macos", "windows", "linux"] {
            assert_eq!(
                value.parse::<BuildTarget>().expect("target").as_str(),
                value
            );
        }
    }
}
