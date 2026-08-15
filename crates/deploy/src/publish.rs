use crate::error::{DeployError, DeployResult};
use crate::model::DeployEnvironment;
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

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
    environment: DeployEnvironment,
    dry_run: bool,
) -> DeployResult<Vec<String>> {
    let command = cloudflare_pages_command(output, project_name, environment);
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

pub fn publish_vercel(
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    dry_run: bool,
) -> DeployResult<Vec<String>> {
    let command = vercel_command(project_name, environment);
    if !dry_run {
        let npm_cache = tempfile::tempdir()?;
        let mut process = Command::new(&command[0]);
        process.args(&command[1..]);
        process.current_dir(output);
        configure_npm_cache(&mut process, npm_cache.path());
        run_vercel(&mut process)?;
    }
    Ok(command)
}

pub(crate) fn vercel_command(project_name: &str, environment: DeployEnvironment) -> Vec<String> {
    let mut command = vec![
        "npx".to_string(),
        "--yes".to_string(),
        "vercel".to_string(),
        "deploy".to_string(),
        "--prebuilt".to_string(),
        "--yes".to_string(),
        "--name".to_string(),
        project_name.to_string(),
    ];
    if environment == DeployEnvironment::Live {
        command.push("--prod".to_string());
    } else {
        command.push("--target".to_string());
        command.push(environment.as_str().to_string());
    }
    command
}

pub(crate) fn cloudflare_pages_command(
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
) -> Vec<String> {
    let mut command = vec![
        "npx".to_string(),
        "--yes".to_string(),
        "wrangler".to_string(),
        "pages".to_string(),
        "deploy".to_string(),
        output.join("assets").display().to_string(),
        "--project-name".to_string(),
        project_name.to_string(),
    ];
    if environment != DeployEnvironment::Live {
        command.push("--branch".to_string());
        command.push(environment.as_str().to_string());
    }
    command
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

fn run_vercel(command: &mut Command) -> DeployResult<()> {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(DeployError::new(format!(
            "vercel deploy failed with status {status}"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(DeployError::new(
            "vercel deploy requires `npx vercel`; install Node.js/npm or provide an npx-compatible runtime. Dowe itself does not require Node.js",
        )),
        Err(error) => Err(DeployError::new(error.to_string())),
    }
}

pub fn publish_ios(artifact: &Path, dry_run: bool) -> DeployResult<Vec<String>> {
    let report = vec![
        "xcrun".into(),
        "altool".into(),
        "--upload-app".into(),
        "-f".into(),
        artifact.display().to_string(),
        "-t".into(),
        "ios".into(),
        "--apiKey".into(),
        "$DOWE_APP_STORE_API_KEY".into(),
        "--apiIssuer".into(),
        "$DOWE_APP_STORE_API_ISSUER".into(),
    ];
    if dry_run {
        return Ok(report);
    }
    let key = required_env("DOWE_APP_STORE_API_KEY")?;
    let issuer = required_env("DOWE_APP_STORE_API_ISSUER")?;
    let status = Command::new("xcrun")
        .args([
            "altool",
            "--upload-app",
            "-f",
            artifact.to_string_lossy().as_ref(),
            "-t",
            "ios",
            "--apiKey",
            &key,
            "--apiIssuer",
            &issuer,
        ])
        .status()?;
    if !status.success() {
        return Err(DeployError::new(format!(
            "App Store Connect upload failed with status {status}"
        )));
    }
    Ok(report)
}

pub fn publish_android(
    artifact: &Path,
    package: &str,
    track: &str,
    dry_run: bool,
) -> DeployResult<Vec<String>> {
    validate_track(track)?;
    let report = vec![
        "google-play".into(),
        "upload-bundle".into(),
        artifact.display().to_string(),
        "--package".into(),
        package.into(),
        "--track".into(),
        track.into(),
        "--access-token".into(),
        "$DOWE_GOOGLE_PLAY_ACCESS_TOKEN".into(),
    ];
    if dry_run {
        return Ok(report);
    }
    let token = required_env("DOWE_GOOGLE_PLAY_ACCESS_TOKEN")?;
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(request_error)?;
    let base = format!(
        "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{package}"
    );
    let edit = response_json(
        client
            .post(format!("{base}/edits"))
            .bearer_auth(&token)
            .json(&json!({}))
            .send()
            .map_err(request_error)?,
    )?;
    let edit_id = edit
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| DeployError::new("Google Play did not return an edit id"))?;
    let upload_url = format!(
        "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/{package}/edits/{edit_id}/bundles?uploadType=media"
    );
    let upload = response_json(
        client
            .post(upload_url)
            .bearer_auth(&token)
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(fs::read(artifact)?)
            .send()
            .map_err(request_error)?,
    )?;
    let version = upload
        .get("versionCode")
        .and_then(|value| {
            value
                .as_i64()
                .map(|value| value.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
        })
        .ok_or_else(|| DeployError::new("Google Play did not return a bundle version code"))?;
    let track_body = serde_json::to_string(&json!({
        "track": track,
        "releases": [{ "status": "completed", "versionCodes": [version] }]
    }))?;
    response_json(
        client
            .put(format!("{base}/edits/{edit_id}/tracks/{track}"))
            .bearer_auth(&token)
            .header(CONTENT_TYPE, "application/json")
            .body(track_body)
            .send()
            .map_err(request_error)?,
    )?;
    response_json(
        client
            .post(format!("{base}/edits/{edit_id}:commit"))
            .bearer_auth(&token)
            .json(&json!({}))
            .send()
            .map_err(request_error)?,
    )?;
    Ok(report)
}

fn response_json(response: Response) -> DeployResult<Value> {
    let status = response.status();
    let body = response.text().map_err(request_error)?;
    if !status.is_success() {
        return Err(DeployError::new(format!(
            "Google Play request failed with status {status}: {body}"
        )));
    }
    Ok(serde_json::from_str(&body)?)
}

fn request_error(error: reqwest::Error) -> DeployError {
    DeployError::new(format!("Google Play request failed: {error}"))
}

fn required_env(name: &str) -> DeployResult<String> {
    env_value(name).ok_or_else(|| DeployError::new(format!("{name} is required for publication")))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn validate_track(track: &str) -> DeployResult<()> {
    if track.is_empty()
        || track
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || value == '-' || value == '_'))
    {
        return Err(DeployError::new(
            "Google Play track contains invalid characters",
        ));
    }
    Ok(())
}
