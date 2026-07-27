use crate::error::{DeployError, DeployResult};
use std::path::Path;
use std::process::Command;

pub fn publish_cloudflare(output: &Path, dry_run: bool) -> DeployResult<Vec<String>> {
    let (worker, command) = cloudflare_command(output, dry_run);
    let npm_cache = tempfile::tempdir()?;
    let mut process = Command::new(&command[0]);
    process.args(&command[1..]).current_dir(worker);
    configure_npm_cache(&mut process, npm_cache.path());
    run_cloudflare_worker(&mut process)?;
    Ok(command)
}

pub fn publish_cloudflare_pages(
    output: &Path,
    project_name: &str,
    dry_run: bool,
) -> DeployResult<Vec<String>> {
    let command = cloudflare_pages_command(output, project_name);
    if !dry_run {
        let npm_cache = tempfile::tempdir()?;
        let mut process = Command::new(&command[0]);
        process.args(&command[1..]);
        process.current_dir(std::env::temp_dir());
        configure_npm_cache(&mut process, npm_cache.path());
        run_cloudflare_pages(&mut process)?;
    }
    Ok(command)
}

pub(crate) fn cloudflare_pages_command(output: &Path, project_name: &str) -> Vec<String> {
    vec![
        "npx".to_string(),
        "--yes".to_string(),
        "wrangler".to_string(),
        "pages".to_string(),
        "deploy".to_string(),
        output.join("assets").display().to_string(),
        "--project-name".to_string(),
        project_name.to_string(),
    ]
}

pub(crate) fn cloudflare_command(
    output: &Path,
    dry_run: bool,
) -> (std::path::PathBuf, Vec<String>) {
    let worker = output.join("worker");
    let config = output.join("worker/wrangler.jsonc");
    let mut command = vec![
        "npx".to_string(),
        "--yes".to_string(),
        "wrangler".to_string(),
        "deploy".to_string(),
        "--config".to_string(),
        config.display().to_string(),
    ];
    if dry_run {
        command.push("--dry-run".to_string());
    }
    (worker, command)
}

pub(crate) fn configure_npm_cache(command: &mut Command, cache: &Path) {
    command
        .env("npm_config_cache", cache)
        .env("NPM_CONFIG_CACHE", cache);
}

fn run_cloudflare_pages(command: &mut Command) -> DeployResult<()> {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(DeployError::new(format!(
            "cloudflare pages deploy failed with status {status}"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(DeployError::new(
            "cloudflare pages deploy requires `npx wrangler`; install Node.js/npm or provide an npx-compatible runtime. Dowe itself does not require Node.js",
        )),
        Err(error) => Err(DeployError::new(error.to_string())),
    }
}

fn run_cloudflare_worker(command: &mut Command) -> DeployResult<()> {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(DeployError::new(format!(
            "cloudflare worker deploy failed with status {status}"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(DeployError::new(
            "cloudflare worker deploy requires `npx wrangler`; install Node.js/npm or provide an npx-compatible runtime. Dowe itself does not require Node.js or Rust",
        )),
        Err(error) => Err(DeployError::new(error.to_string())),
    }
}
