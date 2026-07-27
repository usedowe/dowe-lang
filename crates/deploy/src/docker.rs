use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::DeployTarget;
use crate::package::copy_app;
use serde_json::json;
use std::path::Path;
use std::process::{Command, Stdio};

pub const DEFAULT_DOCKER_REGISTRY: &str = "docker.io";
pub const DOCKER_PLATFORM: &str = "linux/amd64";
const DISTROLESS_IMAGE: &str = "gcr.io/distroless/cc-debian12:nonroot";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockerImage {
    pub registry: String,
    pub image: String,
    pub reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockerBuildOutcome {
    pub command: Vec<String>,
    pub built: bool,
}

pub fn resolve_docker_image(
    root: &Path,
    registry: Option<&str>,
    image: Option<&str>,
) -> DeployResult<DockerImage> {
    let registry = registry
        .unwrap_or(DEFAULT_DOCKER_REGISTRY)
        .trim_end_matches('/');
    validate_registry(registry)?;
    let default_image = default_docker_image_name(root);
    let image = image.unwrap_or(&default_image);
    validate_image(image)?;
    let image = if image_has_tag(image) {
        image.to_string()
    } else {
        format!("{image}:latest")
    };
    Ok(DockerImage {
        registry: registry.to_string(),
        reference: format!("{registry}/{image}"),
        image,
    })
}

pub fn default_docker_image_name(root: &Path) -> String {
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dowe-app");
    let mut normalized = String::new();
    let mut last_was_separator = false;
    for value in name.chars().flat_map(char::to_lowercase) {
        if value.is_ascii_lowercase() || value.is_ascii_digit() {
            normalized.push(value);
            last_was_separator = false;
        } else if !last_was_separator && !normalized.is_empty() {
            normalized.push('-');
            last_was_separator = true;
        }
    }
    while normalized.ends_with('-') {
        normalized.pop();
    }
    if normalized.is_empty() {
        "dowe-app".to_string()
    } else {
        normalized
    }
}

pub fn generate_docker(root: &Path, output: &Path, image: &DockerImage) -> DeployResult<()> {
    copy_app(root, &output.join("app"))?;
    write_file(&output.join("Dockerfile"), release_dockerfile())?;
    let mut manifest = serde_json::to_string_pretty(&json!({
        "version": 1,
        "target": DeployTarget::Docker,
        "platform": DOCKER_PLATFORM,
        "registry": image.registry,
        "image": image.image,
        "imageRef": image.reference,
        "runtime": "release"
    }))?;
    manifest.push('\n');
    write_file(&output.join("deploy.json"), manifest)
}

pub fn build_docker_image(
    output: &Path,
    image: &DockerImage,
    dry_run: bool,
) -> DeployResult<DockerBuildOutcome> {
    let command = docker_build_command(output, &image.reference);
    if dry_run || !docker_daemon_available()? {
        return Ok(DockerBuildOutcome {
            command,
            built: false,
        });
    }
    let status = Command::new(&command[0]).args(&command[1..]).status()?;
    if !status.success() {
        return Err(DeployError::new(format!(
            "docker image build failed with status {status}"
        )));
    }
    Ok(DockerBuildOutcome {
        command,
        built: true,
    })
}

pub(crate) fn docker_build_command(output: &Path, image: &str) -> Vec<String> {
    vec![
        "docker".to_string(),
        "build".to_string(),
        "--platform".to_string(),
        DOCKER_PLATFORM.to_string(),
        "--tag".to_string(),
        image.to_string(),
        output.display().to_string(),
    ]
}

fn docker_daemon_available() -> DeployResult<bool> {
    match Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => Ok(status.success()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn validate_registry(value: &str) -> DeployResult<()> {
    if value.is_empty()
        || value.contains("://")
        || value.contains("..")
        || value.contains("//")
        || value.starts_with(['/', '.', '-', ':'])
        || value.ends_with(['/', '.', '-', ':'])
        || value.chars().any(|character| {
            !(character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || "./-:".contains(character))
        })
    {
        return Err(DeployError::new(
            "docker registry must be a lowercase host or host/path without a URL scheme",
        ));
    }
    Ok(())
}

fn validate_image(value: &str) -> DeployResult<()> {
    if value.is_empty()
        || value.len() > 255
        || value.contains("..")
        || value.contains("//")
        || value.starts_with(['/', '.', '-', ':'])
        || value.ends_with(['/', '.', '-', ':'])
        || value.chars().any(|character| {
            !(character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || "._/-:".contains(character))
        })
    {
        return Err(DeployError::new(
            "docker image must use lowercase repository segments and an optional tag",
        ));
    }
    let last = value.rsplit('/').next().unwrap_or(value);
    if last.matches(':').count() > 1 {
        return Err(DeployError::new("docker image contains an invalid tag"));
    }
    Ok(())
}

fn image_has_tag(value: &str) -> bool {
    value
        .rsplit('/')
        .next()
        .is_some_and(|part| part.contains(':'))
}

fn release_dockerfile() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let archive_url = format!("https://get.dowe.dev/v{version}/linux-amd64.tar.gz");
    format!(
        "FROM debian:bookworm-slim AS dowe-runtime\nARG DOWE_ARCHIVE_URL={archive_url}\nRUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl tar && curl -fsSL \"$DOWE_ARCHIVE_URL\" -o /dowe.tar.gz && tar -xzf /dowe.tar.gz -C /tmp && mv /tmp/dowe /dowe && chmod 0755 /dowe && rm -rf /var/lib/apt/lists/* /dowe.tar.gz /tmp/assets\nFROM {DISTROLESS_IMAGE}\nWORKDIR /app\nCOPY --from=dowe-runtime /dowe /usr/local/bin/dowe\nCOPY --chown=nonroot:nonroot app /app\nEXPOSE 8080\nUSER nonroot:nonroot\nENTRYPOINT [\"/usr/local/bin/dowe\",\"server\",\"--root\",\"/app\",\"--bind\",\"0.0.0.0:8080\"]\n"
    )
}
