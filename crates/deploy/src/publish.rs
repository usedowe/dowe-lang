use crate::error::{DeployError, DeployResult};
use crate::files::copy_file;
use crate::model::DeployOptions;
use std::path::Path;
use std::process::Command;

pub fn publish_cloudflare(output: &Path, dry_run: bool) -> DeployResult<Vec<String>> {
    let (worker, command) = cloudflare_command(output, dry_run);
    run(Command::new(&command[0])
        .args(&command[1..])
        .current_dir(worker))?;
    Ok(command)
}

pub(crate) fn cloudflare_command(
    output: &Path,
    dry_run: bool,
) -> (std::path::PathBuf, Vec<String>) {
    let worker = output.join("worker");
    let config = output.join("worker/wrangler.jsonc");
    let mut command = vec![
        "npx".to_string(),
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

pub fn publish_ssh(output: &Path, options: &DeployOptions) -> DeployResult<Vec<String>> {
    let host = options
        .host
        .as_deref()
        .ok_or_else(|| DeployError::new("ssh publish requires --host"))?;
    let remote_dir = options
        .remote_dir
        .as_deref()
        .ok_or_else(|| DeployError::new("ssh publish requires --remote-dir"))?;
    let server_binary = options
        .server_binary
        .as_deref()
        .ok_or_else(|| DeployError::new("ssh publish requires --server-binary"))?;
    validate_host(host)?;
    validate_remote_dir(remote_dir)?;
    if !server_binary.is_file() {
        return Err(DeployError::new(format!(
            "missing dowe-server binary: {}",
            server_binary.display()
        )));
    }
    copy_file(server_binary, &output.join("dowe-server"))?;
    let prepare = format!("mkdir -p {remote_dir}");
    run(Command::new("ssh").arg(host).arg(&prepare))?;
    let destination = format!("{host}:{remote_dir}/");
    run(Command::new("scp")
        .arg("-r")
        .arg(output.join("app"))
        .arg(&destination))?;
    for file in ["deploy.json", "dowe-server", "run.sh"] {
        run(Command::new("scp").arg(output.join(file)).arg(&destination))?;
    }
    let launch = format!(
        "cd {remote_dir} && chmod +x dowe-server run.sh && if [ -f dowe-server.pid ]; then kill \"$(cat dowe-server.pid)\" || true; fi && nohup ./run.sh > dowe-server.log 2>&1 & echo $! > dowe-server.pid"
    );
    run(Command::new("ssh").arg(host).arg(&launch))?;
    Ok(vec!["ssh".to_string(), host.to_string(), launch])
}

fn run(command: &mut Command) -> DeployResult<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(DeployError::new(format!(
            "deploy command failed with status {status}"
        )))
    }
}

fn validate_host(host: &str) -> DeployResult<()> {
    if host.is_empty()
        || host
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || "._@:-".contains(value)))
    {
        return Err(DeployError::new("invalid ssh host"));
    }
    Ok(())
}

fn validate_remote_dir(path: &str) -> DeployResult<()> {
    if !path.starts_with('/')
        || path.contains("..")
        || path
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || "/._-".contains(value)))
    {
        return Err(DeployError::new("invalid ssh remote directory"));
    }
    Ok(())
}
