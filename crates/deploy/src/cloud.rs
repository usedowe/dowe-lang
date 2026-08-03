use crate::error::{DeployError, DeployResult};
use crate::files::{collect_files, write_file};
use crate::model::{DeployEnvironment, DeploySurface, DeployTarget};
use crate::package;
use reqwest::Url;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

const MAGIC: &[u8; 8] = b"DOWEBIN1";
const DEFAULT_API_URL: &str = "https://api.dowe.cloud";

pub struct CloudSession {
    client: Client,
    api_url: Url,
    token: String,
}

pub struct CloudArtifact {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
    pub name: String,
}

pub struct CloudPublication {
    pub deployment_id: String,
    pub url: String,
}

#[derive(Deserialize, Serialize)]
struct LocalSession {
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentReservation {
    deployment_id: String,
    upload_url: String,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentRequest<'a> {
    name: &'a str,
    environment: DeployEnvironment,
    surface: DeploySurface,
    sha256: &'a str,
    size: u64,
}

impl CloudSession {
    pub fn resolve_and_validate() -> DeployResult<Self> {
        let token = resolve_token()?;
        Self::validate_token(token)
    }

    fn validate_token(token: String) -> DeployResult<Self> {
        let api_url = resolve_api_url()?;
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| DeployError::new("failed to initialize the Dowe Cloud client"))?;
        let session = Self {
            client,
            api_url,
            token,
        };
        let response = session
            .client
            .get(session.endpoint("v1/cli/session")?)
            .bearer_auth(&session.token)
            .send()
            .map_err(|_| DeployError::new("could not reach Dowe Cloud to validate the session"))?;
        if !response.status().is_success() {
            return Err(DeployError::new(
                "Dowe Cloud authentication failed; run `dowe login` or set DOWE_CLOUD_TOKEN",
            ));
        }
        Ok(session)
    }

    pub fn publish(
        &self,
        artifact: &CloudArtifact,
        surface: DeploySurface,
        environment: DeployEnvironment,
    ) -> DeployResult<CloudPublication> {
        let response = self
            .client
            .post(self.endpoint("v1/deployments")?)
            .bearer_auth(&self.token)
            .json(&DeploymentRequest {
                name: &artifact.name,
                environment,
                surface,
                sha256: &artifact.hash,
                size: artifact.size,
            })
            .send()
            .map_err(|_| DeployError::new("Dowe Cloud could not reserve the deployment"))?;
        if !response.status().is_success() {
            return Err(DeployError::new(
                "Dowe Cloud rejected the deployment reservation",
            ));
        }
        let reservation = response.json::<DeploymentReservation>().map_err(|_| {
            DeployError::new("Dowe Cloud returned an invalid deployment reservation")
        })?;
        let upload_url = validate_upload_url(&reservation.upload_url)?;
        let bytes = fs::read(&artifact.path)?;
        let response = self
            .client
            .put(upload_url)
            .bearer_auth(&self.token)
            .header("content-type", "application/vnd.dowe.application")
            .body(bytes)
            .send()
            .map_err(|_| DeployError::new("Dowe Registry artifact upload failed"))?;
        if !response.status().is_success() {
            return Err(DeployError::new(
                "Dowe Registry rejected the artifact upload",
            ));
        }
        let response = self
            .client
            .post(self.endpoint(&format!(
                "v1/deployments/{}/activate",
                reservation.deployment_id
            ))?)
            .bearer_auth(&self.token)
            .send()
            .map_err(|_| DeployError::new("Dowe Cloud could not activate the deployment"))?;
        if !response.status().is_success() {
            return Err(DeployError::new(
                "Dowe Cloud rejected deployment activation",
            ));
        }
        Ok(CloudPublication {
            deployment_id: reservation.deployment_id,
            url: reservation.url,
        })
    }

    fn endpoint(&self, path: &str) -> DeployResult<Url> {
        self.api_url
            .join(path)
            .map_err(|_| DeployError::new("invalid Dowe Cloud API endpoint"))
    }
}

pub fn authenticate_cloud_session(token: &str) -> DeployResult<()> {
    let token = token.trim();
    if token.is_empty() {
        return Err(DeployError::new("Dowe Cloud token must not be empty"));
    }
    CloudSession::validate_token(token.to_string())?;
    persist_session(token)
}

fn persist_session(token: &str) -> DeployResult<()> {
    let path = session_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| DeployError::new("Dowe Cloud session directory is invalid"))?;
    fs::create_dir_all(parent)?;
    let mut bytes = serde_json::to_vec_pretty(&LocalSession {
        token: token.to_string(),
    })?;
    bytes.push(b'\n');
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

pub fn generate_artifact(
    root: &Path,
    output: &Path,
    surface: DeploySurface,
    environment: DeployEnvironment,
    access_protected: bool,
) -> DeployResult<CloudArtifact> {
    let staging = tempfile::tempdir()?;
    let app = staging.path().join("app");
    package::copy_app(root, &app)?;
    let path = output.join("app.dowebin");
    let bytes = encode_bundle(&app)?;
    let hash = sha256_hex(&bytes);
    write_file(&path, &bytes)?;
    let name = project_name(root)?;
    let mut manifest = serde_json::to_string_pretty(&json!({
        "version": 1,
        "target": DeployTarget::Dowe,
        "surface": surface,
        "environment": environment,
        "accessProtected": access_protected,
        "artifact": "app.dowebin",
        "sha256": hash,
        "size": bytes.len(),
        "name": name,
    }))?;
    manifest.push('\n');
    write_file(&output.join("deploy.json"), manifest)?;
    Ok(CloudArtifact {
        path,
        hash,
        size: bytes.len() as u64,
        name,
    })
}

pub(crate) fn application_binary(root: &Path) -> DeployResult<Vec<u8>> {
    let staging = tempfile::tempdir()?;
    let app = staging.path().join("app");
    package::copy_app(root, &app)?;
    encode_bundle(&app)
}

fn encode_bundle(root: &Path) -> DeployResult<Vec<u8>> {
    let files = collect_files(root)?;
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&(files.len() as u32).to_be_bytes());
    for relative in files {
        validate_relative_path(&relative)?;
        let name = relative
            .to_str()
            .ok_or_else(|| DeployError::new("Dowe Cloud artifact paths must be UTF-8"))?
            .replace('\\', "/");
        let name = name.as_bytes();
        let bytes = fs::read(root.join(&relative))?;
        output.extend_from_slice(&(name.len() as u32).to_be_bytes());
        output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        output.extend_from_slice(name);
        output.extend_from_slice(&bytes);
    }
    let checksum = Sha256::digest(&output);
    output.extend_from_slice(&checksum);
    Ok(output)
}

pub fn materialize_cloud_artifact(artifact: &Path, output: &Path) -> DeployResult<()> {
    let bytes = fs::read(artifact)?;
    if bytes.len() < MAGIC.len() + 4 + 32 || &bytes[..MAGIC.len()] != MAGIC {
        return Err(DeployError::new("invalid Dowe Application Binary"));
    }
    let payload_len = bytes.len() - 32;
    let expected = &bytes[payload_len..];
    let actual = Sha256::digest(&bytes[..payload_len]);
    if actual.as_slice() != expected {
        return Err(DeployError::new(
            "Dowe Application Binary checksum mismatch",
        ));
    }
    fs::create_dir_all(output)?;
    let mut cursor = MAGIC.len();
    let count = read_u32(&bytes, &mut cursor, payload_len)? as usize;
    for _ in 0..count {
        let name_len = read_u32(&bytes, &mut cursor, payload_len)? as usize;
        let file_len = read_u64(&bytes, &mut cursor, payload_len)? as usize;
        let name_end = cursor
            .checked_add(name_len)
            .filter(|end| *end <= payload_len)
            .ok_or_else(|| DeployError::new("invalid Dowe Application Binary path length"))?;
        let relative = std::str::from_utf8(&bytes[cursor..name_end])
            .map_err(|_| DeployError::new("invalid Dowe Application Binary path"))?;
        cursor = name_end;
        let file_end = cursor
            .checked_add(file_len)
            .filter(|end| *end <= payload_len)
            .ok_or_else(|| DeployError::new("invalid Dowe Application Binary file length"))?;
        let relative = PathBuf::from(relative);
        validate_relative_path(&relative)?;
        write_file(&output.join(relative), &bytes[cursor..file_end])?;
        cursor = file_end;
    }
    if cursor != payload_len {
        return Err(DeployError::new(
            "invalid trailing Dowe Application Binary data",
        ));
    }
    Ok(())
}

fn read_u32(bytes: &[u8], cursor: &mut usize, limit: usize) -> DeployResult<u32> {
    let end = cursor
        .checked_add(4)
        .filter(|end| *end <= limit)
        .ok_or_else(|| DeployError::new("truncated Dowe Application Binary"))?;
    let value = u32::from_be_bytes(bytes[*cursor..end].try_into().expect("u32 bytes"));
    *cursor = end;
    Ok(value)
}

fn read_u64(bytes: &[u8], cursor: &mut usize, limit: usize) -> DeployResult<u64> {
    let end = cursor
        .checked_add(8)
        .filter(|end| *end <= limit)
        .ok_or_else(|| DeployError::new("truncated Dowe Application Binary"))?;
    let value = u64::from_be_bytes(bytes[*cursor..end].try_into().expect("u64 bytes"));
    *cursor = end;
    Ok(value)
}

fn resolve_token() -> DeployResult<String> {
    if let Ok(token) = env::var("DOWE_CLOUD_TOKEN")
        && !token.trim().is_empty()
    {
        return Ok(token);
    }
    let path = session_path()?;
    let content = fs::read_to_string(path).map_err(|_| {
        DeployError::new(
            "Dowe Cloud authentication is required; run `dowe login` or set DOWE_CLOUD_TOKEN",
        )
    })?;
    let session = serde_json::from_str::<LocalSession>(&content).map_err(|_| {
        DeployError::new("the local Dowe Cloud session is invalid; run `dowe login` again")
    })?;
    if session.token.trim().is_empty() {
        return Err(DeployError::new(
            "the local Dowe Cloud session has no token",
        ));
    }
    Ok(session.token)
}

fn session_path() -> DeployResult<PathBuf> {
    let base = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| {
            DeployError::new("could not resolve the user directory for Dowe Cloud auth")
        })?;
    Ok(PathBuf::from(base).join(".dowe/cloud/session.json"))
}

fn resolve_api_url() -> DeployResult<Url> {
    let value = env::var("DOWE_CLOUD_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
    let mut url =
        Url::parse(&value).map_err(|_| DeployError::new("DOWE_CLOUD_API_URL is invalid"))?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err(DeployError::new(
            "DOWE_CLOUD_API_URL must use HTTPS or loopback HTTP",
        ));
    }
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn validate_upload_url(value: &str) -> DeployResult<Url> {
    let url = Url::parse(value)
        .map_err(|_| DeployError::new("Dowe Registry returned an invalid upload URL"))?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err(DeployError::new("Dowe Registry upload URL must use HTTPS"));
    }
    Ok(url)
}

fn project_name(root: &Path) -> DeployResult<String> {
    let raw = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| DeployError::new("Dowe Cloud project name is missing"))?;
    let normalized = raw
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() {
                value.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if normalized.is_empty() {
        return Err(DeployError::new("Dowe Cloud project name is invalid"));
    }
    Ok(normalized)
}

fn validate_relative_path(path: &Path) -> DeployResult<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(DeployError::new("invalid Dowe Cloud artifact path"));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{MAGIC, encode_bundle, materialize_cloud_artifact};
    use std::fs;

    #[test]
    fn bundle_is_deterministic_and_excludes_unstaged_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
        fs::create_dir(temp.path().join("server")).expect("server");
        fs::write(temp.path().join("server/app.dowe"), "fn app\n").expect("module");
        let first = encode_bundle(temp.path()).expect("first");
        let second = encode_bundle(temp.path()).expect("second");
        assert_eq!(first, second);
        assert!(first.starts_with(MAGIC));
    }

    #[test]
    fn materializes_valid_bundles_and_rejects_corruption() {
        let source = tempfile::tempdir().expect("source");
        let output = tempfile::tempdir().expect("output");
        fs::write(source.path().join("main.dowe"), "main\n").expect("main");
        let bytes = encode_bundle(source.path()).expect("bundle");
        let artifact = source.path().join("app.dowebin");
        fs::write(&artifact, &bytes).expect("artifact");
        materialize_cloud_artifact(&artifact, output.path()).expect("materialize");
        assert_eq!(
            fs::read_to_string(output.path().join("main.dowe")).expect("materialized"),
            "main\n"
        );
        let mut corrupt = bytes;
        corrupt[8] ^= 1;
        fs::write(&artifact, corrupt).expect("corrupt");
        assert!(materialize_cloud_artifact(&artifact, output.path()).is_err());
    }
}
