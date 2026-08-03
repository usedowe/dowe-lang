use crate::access::DeployAccess;
use crate::cloud;
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeployTarget};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const TRAILER_MAGIC: &[u8; 8] = b"DOWESSH1";
const TRAILER_VERSION: u64 = 1;
const TRAILER_SIZE: usize = 112;
const DEFAULT_RELEASE_BASE_URL: &str = "https://get.dowe.dev";
const MAX_ARCHIVE_SIZE: u64 = 256 * 1024 * 1024;
const MAX_RUNTIME_SIZE: u64 = 512 * 1024 * 1024;
const REMOTE_SCRIPT: &str = r#"set -eu
upload=$1
service=$2
run_user=$3
binary=$4
cleanup() {
  rm -f "$upload"
}
trap cleanup EXIT HUP INT TERM
if [ ! -r /etc/os-release ]; then
  echo "SSH deploy requires Debian or Ubuntu" >&2
  exit 1
fi
. /etc/os-release
case "${ID:-}" in
  debian|ubuntu) ;;
  *) echo "SSH deploy requires Debian or Ubuntu" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ;;
  *) echo "SSH deploy requires a Linux amd64 host" >&2; exit 1 ;;
esac
command -v systemctl >/dev/null 2>&1 || { echo "SSH deploy requires systemd" >&2; exit 1; }
command -v sudo >/dev/null 2>&1 || { echo "SSH deploy requires sudo" >&2; exit 1; }
id "$run_user" >/dev/null 2>&1 || { echo "SSH deploy user does not exist" >&2; exit 1; }
sudo -v
install_dir="/opt/dowe/$service"
unit="/etc/systemd/system/$service.service"
group=$(id -gn "$run_user")
sudo install -d -m 0755 -o root -g root "$install_dir"
sudo install -m 0755 -o root -g root "$upload" "$install_dir/$binary"
unit_file=$(mktemp)
trap 'rm -f "$unit_file"; cleanup' EXIT HUP INT TERM
printf '%s\n' \
  '[Unit]' \
  "Description=Dowe SSH deployment $service" \
  'After=network-online.target' \
  'Wants=network-online.target' \
  '' \
  '[Service]' \
  'Type=simple' \
  "User=$run_user" \
  "Group=$group" \
  "WorkingDirectory=$install_dir" \
  "EnvironmentFile=-/etc/dowe/$service.env" \
  "Environment=DOWE_SSH_APP_ROOT=/var/lib/dowe/$service/app" \
  "ExecStart=$install_dir/$binary" \
  'Restart=always' \
  'RestartSec=3' \
  'NoNewPrivileges=true' \
  'PrivateTmp=true' \
  'ProtectSystem=strict' \
  'ProtectHome=true' \
  "ReadWritePaths=/var/lib/dowe/$service" \
  '' \
  '[Install]' \
  'WantedBy=multi-user.target' > "$unit_file"
sudo install -d -m 0755 /etc/dowe
sudo install -d -m 0755 -o root -g root /var/lib/dowe "/var/lib/dowe/$service"
sudo install -d -m 0755 -o "$run_user" -g "$group" "/var/lib/dowe/$service/app"
sudo install -m 0644 -o root -g root "$unit_file" "$unit"
sudo systemctl daemon-reload
sudo systemctl enable --now "$service.service"
sudo systemctl --no-pager --full status "$service.service"
"#;

#[derive(Clone, Debug)]
pub(crate) struct SshPackage {
    pub executable: PathBuf,
    service_name: String,
    binary_name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct SshDestination {
    host: String,
    user: String,
    key_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedSshMetadata {
    pub environment: DeployEnvironment,
    pub access_hash: Option<String>,
    pub bind: String,
    pub client_environment: Vec<(String, String)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutableMetadata<'a> {
    environment: DeployEnvironment,
    access_hash: Option<&'a str>,
    bind: &'a str,
    client_environment: &'a [(String, String)],
}

impl SshDestination {
    pub(crate) fn resolve(
        host: Option<&str>,
        user: Option<&str>,
        key_file: Option<&Path>,
    ) -> DeployResult<Self> {
        let host = host
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| DeployError::new("SSH publish requires --host"))?;
        let user = user
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| DeployError::new("SSH publish requires --user"))?;
        validate_host(host)?;
        validate_user(user)?;
        let key_file = key_file
            .map(|path| {
                let metadata = fs::metadata(path).map_err(|_| {
                    DeployError::new(format!("SSH key file does not exist: {}", path.display()))
                })?;
                if !metadata.is_file() {
                    return Err(DeployError::new(format!(
                        "SSH key file is not a regular file: {}",
                        path.display()
                    )));
                }
                path.canonicalize().map_err(DeployError::from)
            })
            .transpose()?;
        Ok(Self {
            host: host.to_string(),
            user: user.to_string(),
            key_file,
        })
    }

    fn target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    fn auth_args(&self) -> Vec<String> {
        match self.key_file.as_ref() {
            Some(path) => vec![
                "-i".into(),
                path.display().to_string(),
                "-o".into(),
                "IdentitiesOnly=yes".into(),
            ],
            None => vec![
                "-o".into(),
                "PreferredAuthentications=password,keyboard-interactive".into(),
                "-o".into(),
                "PubkeyAuthentication=no".into(),
            ],
        }
    }
}

pub(crate) fn generate_ssh(
    root: &Path,
    output: &Path,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    client_environment: &[(String, String)],
    runtime: &[u8],
) -> DeployResult<SshPackage> {
    generate_ssh_with_runtime(
        root,
        output,
        environment,
        access,
        client_environment,
        runtime,
    )
}

fn generate_ssh_with_runtime(
    root: &Path,
    output: &Path,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    client_environment: &[(String, String)],
    runtime: &[u8],
) -> DeployResult<SshPackage> {
    let binary_name = project_slug(root)?;
    let service_name = format!("dowe-{binary_name}-{}", environment.as_str());
    validate_linux_amd64_elf(runtime)?;
    let application = cloud::application_binary(root)?;
    let metadata = serde_json::to_vec(&ExecutableMetadata {
        environment,
        access_hash: access.map(|value| value.password_hash.as_str()),
        bind: "0.0.0.0:8080",
        client_environment,
    })?;
    let executable = encode_executable(runtime, &application, &metadata);
    let executable_path = output.join(&binary_name);
    write_file(&executable_path, &executable)?;
    set_executable(&executable_path)?;
    let sha256 = format!("{:x}", Sha256::digest(&executable));
    let mut manifest = serde_json::to_string_pretty(&json!({
        "version": 1,
        "target": DeployTarget::Ssh,
        "environment": environment,
        "platform": "linux/amd64",
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "service": service_name,
        "executable": binary_name,
        "sha256": sha256,
        "size": executable.len(),
        "accessProtected": access.is_some(),
    }))?;
    manifest.push('\n');
    write_file(&output.join("deploy.json"), manifest)?;
    Ok(SshPackage {
        executable: executable_path,
        service_name,
        binary_name,
    })
}

pub(crate) fn publish_ssh(
    package: &SshPackage,
    destination: &SshDestination,
    dry_run: bool,
) -> DeployResult<Vec<String>> {
    let remote_upload = format!("/tmp/{}.upload", package.service_name);
    let mut ssh_args = destination.auth_args();
    let mut scp_args = destination.auth_args();
    let control = tempfile::tempdir()?;
    let control_path = if cfg!(unix) {
        let path = control.path().join("c").display().to_string();
        for args in [&mut ssh_args, &mut scp_args] {
            args.extend([
                "-o".into(),
                "ControlMaster=auto".into(),
                "-o".into(),
                "ControlPersist=60".into(),
                "-o".into(),
                format!("ControlPath={path}"),
            ]);
        }
        Some(path)
    } else {
        None
    };
    let install_args = format!(
        "sh -s -- {} {} {} {}",
        shell_word(&remote_upload),
        shell_word(&package.service_name),
        shell_word(&destination.user),
        shell_word(&package.binary_name),
    );
    let remote_command = format!(
        "sh -c {} -- {} {} {} {}",
        shell_word(REMOTE_SCRIPT),
        shell_word(&remote_upload),
        shell_word(&package.service_name),
        shell_word(&destination.user),
        shell_word(&package.binary_name),
    );
    let mut reported = vec!["ssh".to_string()];
    reported.extend(destination.auth_args());
    reported.extend([destination.target(), install_args]);
    if dry_run {
        return Ok(reported);
    }
    scp_args.extend([
        package.executable.display().to_string(),
        format!("{}:{remote_upload}", destination.target()),
    ]);
    if let Err(error) = run_inherited("scp", &scp_args) {
        close_control(control_path.as_deref(), destination);
        return Err(error);
    }
    ssh_args.extend(["-tt".into(), destination.target(), remote_command]);
    let status = Command::new("ssh")
        .args(&ssh_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| DeployError::new(format!("failed to start ssh: {error}")))?;
    close_control(control_path.as_deref(), destination);
    if !status.success() {
        return Err(DeployError::new(format!(
            "SSH installation failed with status {status}"
        )));
    }
    Ok(reported)
}

fn close_control(path: Option<&str>, destination: &SshDestination) {
    let Some(path) = path else {
        return;
    };
    let _ = Command::new("ssh")
        .args([
            "-o",
            &format!("ControlPath={path}"),
            "-O",
            "exit",
            &destination.target(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn materialize_embedded_ssh_executable(
    executable: &Path,
    output: &Path,
) -> DeployResult<Option<EmbeddedSshMetadata>> {
    let mut file = fs::File::open(executable)?;
    let length = usize::try_from(file.metadata()?.len())
        .map_err(|_| DeployError::new("embedded SSH executable is too large"))?;
    if length < TRAILER_SIZE {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-(TRAILER_SIZE as i64)))?;
    let mut trailer = [0u8; TRAILER_SIZE];
    file.read_exact(&mut trailer)?;
    if &trailer[..8] != TRAILER_MAGIC {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(length);
    file.read_to_end(&mut bytes)?;
    let Some((application, metadata)) = decode_executable(&bytes)? else {
        return Ok(None);
    };
    let metadata = serde_json::from_slice::<EmbeddedSshMetadata>(metadata)
        .map_err(|_| DeployError::new("invalid embedded SSH metadata"))?;
    validate_metadata(&metadata)?;
    reset_runtime_root(output)?;
    let artifact = output.join("app.dowebin");
    write_file(&artifact, application)?;
    cloud::materialize_cloud_artifact(&artifact, output)?;
    fs::remove_file(artifact)?;
    write_client_environment(output, &metadata.client_environment)?;
    Ok(Some(metadata))
}

fn validate_metadata(metadata: &EmbeddedSshMetadata) -> DeployResult<()> {
    if metadata.bind != "0.0.0.0:8080" {
        return Err(DeployError::new("invalid embedded SSH bind address"));
    }
    match (metadata.environment, metadata.access_hash.as_deref()) {
        (DeployEnvironment::Live, None) => {}
        (DeployEnvironment::Stage | DeployEnvironment::Uat, Some(hash)) if is_sha256_hex(hash) => {}
        _ => return Err(DeployError::new("invalid embedded SSH access metadata")),
    }
    if metadata
        .client_environment
        .iter()
        .any(|(name, _)| !is_environment_name(name))
    {
        return Err(DeployError::new("invalid embedded SSH environment name"));
    }
    Ok(())
}

fn reset_runtime_root(output: &Path) -> DeployResult<()> {
    fs::create_dir_all(output)?;
    for entry in fs::read_dir(output)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if entry.file_name() == ".dowe" && file_type.is_dir() && !file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() && !file_type.is_symlink() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn write_client_environment(output: &Path, values: &[(String, String)]) -> DeployResult<()> {
    let mut content = String::new();
    for (name, value) in values {
        if !is_environment_name(name) {
            return Err(DeployError::new("invalid embedded SSH environment name"));
        }
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        content.push_str(name);
        content.push_str("=\"");
        content.push_str(&escaped);
        content.push_str("\"\n");
    }
    if !content.is_empty() {
        write_file(&output.join(".env"), content)?;
    }
    Ok(())
}

fn encode_executable(runtime: &[u8], application: &[u8], metadata: &[u8]) -> Vec<u8> {
    let application_offset = runtime.len() as u64;
    let metadata_offset = application_offset + application.len() as u64;
    let mut output =
        Vec::with_capacity(runtime.len() + application.len() + metadata.len() + TRAILER_SIZE);
    output.extend_from_slice(runtime);
    output.extend_from_slice(application);
    output.extend_from_slice(metadata);
    output.extend_from_slice(TRAILER_MAGIC);
    output.extend_from_slice(&TRAILER_VERSION.to_le_bytes());
    output.extend_from_slice(&application_offset.to_le_bytes());
    output.extend_from_slice(&(application.len() as u64).to_le_bytes());
    output.extend_from_slice(&metadata_offset.to_le_bytes());
    output.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    output.extend_from_slice(&Sha256::digest(application));
    output.extend_from_slice(&Sha256::digest(metadata));
    output
}

fn decode_executable(bytes: &[u8]) -> DeployResult<Option<(&[u8], &[u8])>> {
    if bytes.len() < TRAILER_SIZE {
        return Ok(None);
    }
    let trailer = &bytes[bytes.len() - TRAILER_SIZE..];
    if &trailer[..8] != TRAILER_MAGIC {
        return Ok(None);
    }
    if read_u64(trailer, 8)? != TRAILER_VERSION {
        return Err(DeployError::new(
            "unsupported embedded SSH executable version",
        ));
    }
    let application_offset = usize_value(read_u64(trailer, 16)?)?;
    let application_length = usize_value(read_u64(trailer, 24)?)?;
    let metadata_offset = usize_value(read_u64(trailer, 32)?)?;
    let metadata_length = usize_value(read_u64(trailer, 40)?)?;
    let payload_end = bytes.len() - TRAILER_SIZE;
    let application_end = checked_end(application_offset, application_length, payload_end)?;
    let metadata_end = checked_end(metadata_offset, metadata_length, payload_end)?;
    if application_end != metadata_offset || metadata_end != payload_end {
        return Err(DeployError::new("invalid embedded SSH executable layout"));
    }
    let application = &bytes[application_offset..application_end];
    let metadata = &bytes[metadata_offset..metadata_end];
    if Sha256::digest(application).as_slice() != &trailer[48..80] {
        return Err(DeployError::new(
            "embedded SSH application checksum mismatch",
        ));
    }
    if Sha256::digest(metadata).as_slice() != &trailer[80..112] {
        return Err(DeployError::new("embedded SSH metadata checksum mismatch"));
    }
    Ok(Some((application, metadata)))
}

pub(crate) fn prepare_linux_runtime() -> DeployResult<Vec<u8>> {
    let url = format!(
        "{}/v{}/linux-amd64.tar.gz",
        DEFAULT_RELEASE_BASE_URL,
        env!("CARGO_PKG_VERSION")
    );
    let response = reqwest::blocking::get(&url)
        .map_err(|_| DeployError::new("failed to download the Dowe Linux runtime"))?;
    if !response.status().is_success() {
        return Err(DeployError::new(format!(
            "Dowe Linux runtime download failed with status {}",
            response.status()
        )));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_ARCHIVE_SIZE)
    {
        return Err(DeployError::new(
            "Dowe Linux runtime archive exceeds the size limit",
        ));
    }
    let mut archive = Vec::new();
    response
        .take(MAX_ARCHIVE_SIZE + 1)
        .read_to_end(&mut archive)
        .map_err(|_| DeployError::new("failed to read the Dowe Linux runtime archive"))?;
    if archive.len() as u64 > MAX_ARCHIVE_SIZE {
        return Err(DeployError::new(
            "Dowe Linux runtime archive exceeds the size limit",
        ));
    }
    extract_runtime(&archive)
}

fn extract_runtime(archive: &[u8]) -> DeployResult<Vec<u8>> {
    let decoder = GzDecoder::new(Cursor::new(archive));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|_| DeployError::new("invalid Dowe Linux runtime archive"))?;
    for entry in entries {
        let entry = entry.map_err(|_| DeployError::new("invalid Dowe Linux runtime entry"))?;
        let path = entry
            .path()
            .map_err(|_| DeployError::new("invalid Dowe Linux runtime path"))?;
        if path.file_name().and_then(|value| value.to_str()) == Some("dowe")
            && entry.header().entry_type().is_file()
        {
            if entry.size() > MAX_RUNTIME_SIZE {
                return Err(DeployError::new(
                    "Dowe Linux runtime exceeds the size limit",
                ));
            }
            let mut runtime = Vec::new();
            entry.take(MAX_RUNTIME_SIZE + 1).read_to_end(&mut runtime)?;
            if runtime.len() as u64 > MAX_RUNTIME_SIZE {
                return Err(DeployError::new(
                    "Dowe Linux runtime exceeds the size limit",
                ));
            }
            return Ok(runtime);
        }
    }
    Err(DeployError::new(
        "Dowe Linux runtime archive is missing `dowe`",
    ))
}

fn validate_linux_amd64_elf(runtime: &[u8]) -> DeployResult<()> {
    if runtime.len() < 20
        || &runtime[..4] != b"\x7fELF"
        || runtime[4] != 2
        || runtime[5] != 1
        || u16::from_le_bytes([runtime[18], runtime[19]]) != 62
    {
        return Err(DeployError::new(
            "SSH deploy requires a Linux amd64 Dowe runtime",
        ));
    }
    if !runtime
        .windows(TRAILER_MAGIC.len())
        .any(|window| window == TRAILER_MAGIC)
    {
        return Err(DeployError::new(
            "the Dowe Linux runtime does not support embedded SSH applications",
        ));
    }
    Ok(())
}

fn project_slug(root: &Path) -> DeployResult<String> {
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| DeployError::new("SSH project name is missing"))?;
    let slug = name
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() {
                value.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = slug.trim_matches('-');
    if slug.is_empty() || slug.len() > 63 {
        return Err(DeployError::new("SSH project name is invalid"));
    }
    Ok(slug.to_string())
}

fn validate_host(value: &str) -> DeployResult<()> {
    if value.len() > 253
        || value.starts_with('-')
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | ':' | '[' | ']'))
        })
    {
        return Err(DeployError::new("SSH host is invalid"));
    }
    Ok(())
}

fn validate_user(value: &str) -> DeployResult<()> {
    if value.len() > 64
        || value.starts_with('-')
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
    {
        return Err(DeployError::new("SSH user is invalid"));
    }
    Ok(())
}

fn shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn run_inherited(program: &str, args: &[String]) -> DeployResult<()> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| DeployError::new(format!("failed to start {program}: {error}")))?;
    if !status.success() {
        return Err(DeployError::new(format!(
            "{program} failed with status {status}"
        )));
    }
    Ok(())
}

fn read_u64(bytes: &[u8], offset: usize) -> DeployResult<u64> {
    let end = offset
        .checked_add(8)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| DeployError::new("truncated embedded SSH executable"))?;
    Ok(u64::from_le_bytes(
        bytes[offset..end].try_into().expect("u64 bytes"),
    ))
}

fn usize_value(value: u64) -> DeployResult<usize> {
    usize::try_from(value).map_err(|_| DeployError::new("embedded SSH offset is too large"))
}

fn checked_end(offset: usize, length: usize, limit: usize) -> DeployResult<usize> {
    offset
        .checked_add(length)
        .filter(|end| *end <= limit)
        .ok_or_else(|| DeployError::new("invalid embedded SSH executable bounds"))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_uppercase())
        && characters.all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

#[cfg(unix)]
fn set_executable(path: &Path) -> DeployResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> DeployResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        EmbeddedSshMetadata, REMOTE_SCRIPT, SshDestination, decode_executable, encode_executable,
        generate_ssh_with_runtime, materialize_embedded_ssh_executable, publish_ssh,
        reset_runtime_root, validate_linux_amd64_elf,
    };
    use crate::model::DeployEnvironment;
    use sha2::{Digest, Sha256};
    #[cfg(unix)]
    use std::io::Write;
    #[cfg(unix)]
    use std::process::{Command, Stdio};

    #[test]
    fn executable_trailer_round_trips_and_rejects_tampering() {
        let runtime = linux_runtime();
        let application = b"application";
        let metadata = serde_json::to_vec(&EmbeddedSshMetadata {
            environment: DeployEnvironment::Live,
            access_hash: None,
            bind: "0.0.0.0:8080".into(),
            client_environment: Vec::new(),
        })
        .expect("metadata");
        let executable = encode_executable(&runtime, application, &metadata);
        let (decoded_application, decoded_metadata) = decode_executable(&executable)
            .expect("decode")
            .expect("embedded");
        assert_eq!(decoded_application, application);
        assert_eq!(decoded_metadata, metadata);

        let mut corrupted = executable;
        corrupted[runtime.len()] ^= 1;
        assert!(decode_executable(&corrupted).is_err());
    }

    #[test]
    fn destination_validates_host_user_and_key_file() {
        assert!(SshDestination::resolve(Some("server.example.com"), Some("deploy"), None).is_ok());
        assert!(SshDestination::resolve(Some("-oProxyCommand=x"), Some("deploy"), None).is_err());
        assert!(
            SshDestination::resolve(Some("server.example.com"), Some("bad user"), None).is_err()
        );
        assert!(
            SshDestination::resolve(
                Some("server.example.com"),
                Some("deploy"),
                Some(std::path::Path::new("/missing/key")),
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_must_be_linux_amd64_elf() {
        assert!(validate_linux_amd64_elf(&linux_runtime()).is_ok());
        assert!(validate_linux_amd64_elf(&Sha256::digest(b"not elf")).is_err());
    }

    #[test]
    fn generated_executable_materializes_the_packaged_application() {
        let project = tempfile::tempdir().expect("project");
        let output = tempfile::tempdir().expect("output");
        std::fs::write(
            project.path().join("main.dowe"),
            "main\n  server port:8080\n    route \"/status\"\n      response text:\"OK\"\n",
        )
        .expect("main");
        let package = generate_ssh_with_runtime(
            project.path(),
            output.path(),
            DeployEnvironment::Live,
            None,
            &[(
                "PUBLIC_URL".into(),
                "https://example.com/path?x=1&y=2".into(),
            )],
            &linux_runtime(),
        )
        .expect("package");
        let materialized = tempfile::tempdir().expect("materialized");
        std::fs::create_dir_all(materialized.path().join(".dowe")).expect("state directory");
        std::fs::write(materialized.path().join(".dowe/state"), "persistent").expect("state");
        std::fs::write(materialized.path().join("stale.dowe"), "stale").expect("stale source");
        let metadata =
            materialize_embedded_ssh_executable(&package.executable, materialized.path())
                .expect("materialize")
                .expect("metadata");

        assert_eq!(metadata.environment, DeployEnvironment::Live);
        assert!(materialized.path().join("main.dowe").is_file());
        assert!(materialized.path().join(".dowe/state").is_file());
        assert!(!materialized.path().join("stale.dowe").exists());
        assert_eq!(
            std::fs::read_to_string(materialized.path().join(".env")).expect("client environment"),
            "PUBLIC_URL=\"https://example.com/path?x=1&y=2\"\n"
        );
        assert!(output.path().join("deploy.json").is_file());
        assert!(
            !std::fs::read_to_string(output.path().join("deploy.json"))
                .expect("manifest")
                .contains("password")
        );
    }

    #[test]
    fn dry_run_reports_sanitized_password_auth_without_connecting() {
        let output = tempfile::tempdir().expect("output");
        let executable = output.path().join("app");
        std::fs::write(&executable, linux_runtime()).expect("executable");
        let package = super::SshPackage {
            executable,
            service_name: "dowe-app-live".into(),
            binary_name: "app".into(),
        };
        let destination = SshDestination::resolve(Some("server.example.com"), Some("deploy"), None)
            .expect("destination");

        let command = publish_ssh(&package, &destination, true).expect("dry run");

        assert_eq!(command[0], "ssh");
        assert!(
            command
                .iter()
                .any(|value| value == "PubkeyAuthentication=no")
        );
        assert!(command.iter().all(|value| !value.contains("password=")));
        assert!(
            command
                .iter()
                .all(|value| !value.contains("sudo systemctl"))
        );
        assert!(REMOTE_SCRIPT.contains("debian|ubuntu"));
        assert!(REMOTE_SCRIPT.contains("systemctl enable --now"));
        assert!(REMOTE_SCRIPT.contains("Restart=always"));
        assert!(REMOTE_SCRIPT.contains("DOWE_SSH_APP_ROOT=/var/lib/dowe/$service/app"));
        assert!(REMOTE_SCRIPT.contains("ReadWritePaths=/var/lib/dowe/$service"));
    }

    #[cfg(unix)]
    #[test]
    fn remote_installer_is_valid_posix_shell() {
        let mut child = Command::new("sh")
            .arg("-n")
            .stdin(Stdio::piped())
            .spawn()
            .expect("shell");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(REMOTE_SCRIPT.as_bytes())
            .expect("script");
        assert!(child.wait().expect("status").success());
    }

    #[cfg(unix)]
    #[test]
    fn runtime_root_does_not_preserve_a_symlinked_generated_directory() {
        let root = tempfile::tempdir().expect("root");
        let external = tempfile::tempdir().expect("external");
        std::fs::write(external.path().join("state"), "preserved").expect("state");
        std::os::unix::fs::symlink(external.path(), root.path().join(".dowe")).expect("symlink");

        reset_runtime_root(root.path()).expect("reset");

        assert!(!root.path().join(".dowe").exists());
        assert!(external.path().join("state").is_file());
    }

    fn linux_runtime() -> Vec<u8> {
        let mut runtime = vec![0u8; 80];
        runtime[..4].copy_from_slice(b"\x7fELF");
        runtime[4] = 2;
        runtime[5] = 1;
        runtime[18..20].copy_from_slice(&62u16.to_le_bytes());
        runtime[64..72].copy_from_slice(super::TRAILER_MAGIC);
        runtime
    }
}
