use crate::access::DeployAccess;
use crate::application::{EXECUTABLE_NAME, generate_embedded_application};
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeploySurface, DeployTarget};
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
    environment: DeployEnvironment,
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
        let tag = match environment {
            DeployEnvironment::Live => "latest",
            DeployEnvironment::Stage => "stage",
            DeployEnvironment::Uat => "uat",
        };
        format!("{image}:{tag}")
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

pub fn generate_docker(
    root: &Path,
    output: &Path,
    image: &DockerImage,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    surface: DeploySurface,
    server_port: u16,
    http_port: Option<u16>,
    client_environment: &[(String, String)],
    linux_runtime: Option<&[u8]>,
) -> DeployResult<()> {
    let mut ports = vec![server_port];
    if let Some(port) = http_port {
        ports.push(port);
        ports.sort_unstable();
    }
    let mut manifest = json!({
        "version": 1,
        "surface": surface,
        "target": DeployTarget::Docker,
        "platform": DOCKER_PLATFORM,
        "registry": image.registry,
        "image": image.image,
        "imageRef": image.reference,
        "environment": environment,
        "accessProtected": access.is_some(),
        "ports": ports
    });
    match surface {
        DeploySurface::Server | DeploySurface::Web => {
            let runtime =
                linux_runtime.ok_or_else(|| DeployError::new("Docker Linux runtime is missing"))?;
            let bind = format!("0.0.0.0:{server_port}");
            let package = generate_embedded_application(
                root,
                output,
                surface,
                environment,
                access,
                client_environment,
                &bind,
                runtime,
            )?;
            write_file(
                &output.join("Dockerfile"),
                embedded_dockerfile(server_port, http_port),
            )?;
            manifest["runtime"] = json!("embedded");
            manifest["runtimeVersion"] = json!(env!("CARGO_PKG_VERSION"));
            manifest["executable"] = json!(EXECUTABLE_NAME);
            manifest["sha256"] = json!(package.sha256);
            manifest["size"] = json!(package.size);
            if package.executable != output.join(EXECUTABLE_NAME) {
                return Err(DeployError::new("Docker executable path is invalid"));
            }
        }
        DeploySurface::Android | DeploySurface::Ios => {
            return Err(DeployError::new(
                "docker deploy only supports Server and Web",
            ));
        }
    }
    let mut manifest = serde_json::to_string_pretty(&manifest)?;
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

fn embedded_dockerfile(server_port: u16, http_port: Option<u16>) -> String {
    let mut ports = vec![server_port];
    if let Some(port) = http_port {
        ports.push(port);
        ports.sort_unstable();
    }
    let exposed_ports = ports
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "FROM {DISTROLESS_IMAGE}\nWORKDIR /app\nCOPY --chmod=0755 --chown=nonroot:nonroot {EXECUTABLE_NAME} /usr/local/bin/{EXECUTABLE_NAME}\nEXPOSE {exposed_ports}\nUSER nonroot:nonroot\nENTRYPOINT [\"/usr/local/bin/{EXECUTABLE_NAME}\"]\n"
    )
}
