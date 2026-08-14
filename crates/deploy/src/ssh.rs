use crate::access::DeployAccess;
use crate::cloud;
use crate::embedded::{
    SSH_TRAILER_MAGIC, encode_embedded_payload, materialize_application, read_embedded_payload,
    set_executable, validate_access_metadata, validate_client_environment,
};
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeployTarget};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_RELEASE_BASE_URL: &str = "https://get.dowe.dev";
const MAX_ARCHIVE_SIZE: u64 = 256 * 1024 * 1024;
const MAX_RUNTIME_SIZE: u64 = 512 * 1024 * 1024;
const REMOTE_SCRIPT: &str = r#"set -eu
action=$1
run_user=$2
case "$action" in
  preflight) ;;
  install)
    upload=$3
    service=$4
    binary=$5
    cleanup() {
      rm -f "$upload"
    }
    trap cleanup EXIT HUP INT TERM
    ;;
  *) echo "SSH deploy action is invalid" >&2; exit 1 ;;
esac
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
id "$run_user" >/dev/null 2>&1 || { echo "SSH deploy user does not exist" >&2; exit 1; }
if [ "$(id -u)" -eq 0 ]; then
  as_root() {
    "$@"
  }
else
  command -v sudo >/dev/null 2>&1 || { echo "SSH deploy requires root or sudo" >&2; exit 1; }
  sudo -v
  as_root() {
    sudo "$@"
  }
fi
if [ "$action" = preflight ]; then
  exit 0
fi
install_dir="/opt/dowe/$service"
unit="/etc/systemd/system/$service.service"
group=$(id -gn "$run_user")
as_root install -d -m 0755 -o root -g root "$install_dir"
as_root install -m 0755 -o root -g root "$upload" "$install_dir/$binary"
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
as_root install -d -m 0755 /etc/dowe
as_root install -d -m 0755 -o root -g root /var/lib/dowe "/var/lib/dowe/$service"
as_root install -d -m 0755 -o "$run_user" -g "$group" "/var/lib/dowe/$service/app"
as_root install -m 0644 -o root -g root "$unit_file" "$unit"
as_root systemctl daemon-reload
as_root systemctl enable --now "$service.service"
as_root systemctl --no-pager --full status "$service.service"
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
    validate_linux_amd64_runtime(runtime, SSH_TRAILER_MAGIC, "embedded SSH applications")?;
    let application = cloud::application_binary(root)?;
    let metadata = serde_json::to_vec(&ExecutableMetadata {
        environment,
        access_hash: access.map(|value| value.password_hash.as_str()),
        bind: "0.0.0.0:8080",
        client_environment,
    })?;
    let executable = encode_embedded_payload(runtime, &application, &metadata, SSH_TRAILER_MAGIC);
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
        "sh -s -- install {} {} {} {}",
        shell_word(&destination.user),
        shell_word(&remote_upload),
        shell_word(&package.service_name),
        shell_word(&package.binary_name),
    );
    let preflight_command = format!(
        "sh -c {} -- preflight {}",
        shell_word(REMOTE_SCRIPT),
        shell_word(&destination.user),
    );
    let install_command = format!(
        "sh -c {} -- install {} {} {} {}",
        shell_word(REMOTE_SCRIPT),
        shell_word(&destination.user),
        shell_word(&remote_upload),
        shell_word(&package.service_name),
        shell_word(&package.binary_name),
    );
    let mut reported = vec!["ssh".to_string()];
    reported.extend(destination.auth_args());
    reported.extend([destination.target(), install_args]);
    if dry_run {
        return Ok(reported);
    }
    let mut preflight_args = ssh_args.clone();
    preflight_args.extend(["-tt".into(), destination.target(), preflight_command]);
    if let Err(error) = run_inherited("ssh", &preflight_args) {
        close_control(control_path.as_deref(), destination);
        return Err(DeployError::new(format!("SSH preflight failed: {error}")));
    }
    scp_args.extend([
        package.executable.display().to_string(),
        format!("{}:{remote_upload}", destination.target()),
    ]);
    if let Err(error) = run_inherited("scp", &scp_args) {
        close_control(control_path.as_deref(), destination);
        return Err(error);
    }
    ssh_args.extend(["-tt".into(), destination.target(), install_command]);
    let status = Command::new("ssh")
        .args(&ssh_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    close_control(control_path.as_deref(), destination);
    let status =
        status.map_err(|error| DeployError::new(format!("failed to start ssh: {error}")))?;
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
    let Some(payload) = read_embedded_payload(executable, SSH_TRAILER_MAGIC, "SSH")? else {
        return Ok(None);
    };
    let metadata = serde_json::from_slice::<EmbeddedSshMetadata>(&payload.metadata)
        .map_err(|_| DeployError::new("invalid embedded SSH metadata"))?;
    validate_metadata(&metadata)?;
    materialize_application(
        output,
        &payload.application,
        &metadata.client_environment,
        "SSH",
    )?;
    Ok(Some(metadata))
}

fn validate_metadata(metadata: &EmbeddedSshMetadata) -> DeployResult<()> {
    if metadata.bind != "0.0.0.0:8080" {
        return Err(DeployError::new("invalid embedded SSH bind address"));
    }
    validate_access_metadata(metadata.environment, metadata.access_hash.as_deref(), "SSH")?;
    validate_client_environment(&metadata.client_environment, "SSH")
}

pub(crate) fn prepare_linux_runtime() -> DeployResult<Vec<u8>> {
    if let Some(runtime) = installed_linux_runtime()? {
        return Ok(runtime);
    }
    let url = format!(
        "{}/v{}/linux-amd64.tar.gz",
        DEFAULT_RELEASE_BASE_URL,
        env!("CARGO_PKG_VERSION")
    );
    download_linux_runtime_on_worker(url)
}

fn installed_linux_runtime() -> DeployResult<Option<Vec<u8>>> {
    let executable = std::env::current_exe()?;
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return fs::read(executable).map(Some).map_err(DeployError::from);
    }
    let Some(install_dir) = executable.parent() else {
        return Ok(None);
    };
    let runtime = install_dir
        .join("assets")
        .join("runtimes")
        .join("linux-amd64")
        .join("dowe");
    match fs::read(runtime) {
        Ok(runtime) => Ok(Some(runtime)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn download_linux_runtime_on_worker(url: String) -> DeployResult<Vec<u8>> {
    std::thread::spawn(move || download_linux_runtime(&url))
        .join()
        .map_err(|_| DeployError::new("Dowe Linux runtime download worker failed"))?
}

fn download_linux_runtime(url: &str) -> DeployResult<Vec<u8>> {
    let response = reqwest::blocking::get(url)
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

pub(crate) fn validate_linux_amd64_runtime(
    runtime: &[u8],
    capability: &[u8],
    capability_name: &str,
) -> DeployResult<()> {
    if runtime.len() < 20
        || &runtime[..4] != b"\x7fELF"
        || runtime[4] != 2
        || runtime[5] != 1
        || u16::from_le_bytes([runtime[18], runtime[19]]) != 62
    {
        return Err(DeployError::new(
            "deploy requires a Linux amd64 Dowe runtime",
        ));
    }
    if !runtime
        .windows(capability.len())
        .any(|window| window == capability)
    {
        return Err(DeployError::new(format!(
            "the Dowe Linux runtime does not support {capability_name}"
        )));
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

#[cfg(test)]
mod tests {
    use super::{
        EmbeddedSshMetadata, REMOTE_SCRIPT, SshDestination, download_linux_runtime_on_worker,
        generate_ssh_with_runtime, materialize_embedded_ssh_executable, publish_ssh,
        validate_linux_amd64_runtime,
    };
    use crate::embedded::{
        SSH_TRAILER_MAGIC, decode_embedded_payload, encode_embedded_payload, reset_runtime_root,
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
        let executable =
            encode_embedded_payload(&runtime, application, &metadata, SSH_TRAILER_MAGIC);
        let payload = decode_embedded_payload(&executable, SSH_TRAILER_MAGIC, "SSH")
            .expect("decode")
            .expect("embedded");
        assert_eq!(payload.application, application);
        assert_eq!(payload.metadata, metadata);

        let mut corrupted = executable;
        corrupted[runtime.len()] ^= 1;
        assert!(decode_embedded_payload(&corrupted, SSH_TRAILER_MAGIC, "SSH").is_err());
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
        assert!(
            validate_linux_amd64_runtime(
                &linux_runtime(),
                SSH_TRAILER_MAGIC,
                "embedded SSH applications"
            )
            .is_ok()
        );
        assert!(
            validate_linux_amd64_runtime(
                &Sha256::digest(b"not elf"),
                SSH_TRAILER_MAGIC,
                "embedded SSH applications"
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_download_worker_is_safe_inside_tokio_runtime() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async {
            let result = std::panic::catch_unwind(|| {
                download_linux_runtime_on_worker("http://127.0.0.1:1/runtime.tar.gz".into())
            });
            assert!(result.expect("download must not panic").is_err());
        });
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

    #[test]
    fn remote_script_preflights_before_installing() {
        let action = REMOTE_SCRIPT.find("action=$1").expect("action");
        let validation = REMOTE_SCRIPT
            .find("command -v systemctl")
            .expect("validation");
        let preflight_exit = REMOTE_SCRIPT
            .find("if [ \"$action\" = preflight ]; then")
            .expect("preflight exit");
        let installation = REMOTE_SCRIPT.find("as_root install").expect("installation");

        assert!(action < validation);
        assert!(validation < preflight_exit);
        assert!(preflight_exit < installation);
    }

    #[cfg(unix)]
    #[test]
    fn remote_installer_skips_sudo_for_a_root_session() {
        let output = run_privilege_setup("0", false);

        assert!(output.status.success());
        assert_eq!(output.stdout, b"ready");
    }

    #[cfg(unix)]
    #[test]
    fn remote_installer_requires_sudo_for_a_non_root_session() {
        let output = run_privilege_setup("1000", false);

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("requires root or sudo"));
    }

    #[cfg(unix)]
    #[test]
    fn remote_installer_uses_validated_sudo_for_a_non_root_session() {
        let output = run_privilege_setup("1000", true);

        assert!(output.status.success());
        assert_eq!(output.stdout, b"ready");
    }

    #[cfg(unix)]
    fn run_privilege_setup(uid: &str, sudo_available: bool) -> std::process::Output {
        let tools = tempfile::tempdir().expect("tools");
        write_executable(
            &tools.path().join("id"),
            "#!/bin/sh\nprintf '%s\\n' \"$DOWE_TEST_UID\"\n",
        );
        if sudo_available {
            write_executable(
                &tools.path().join("sudo"),
                "#!/bin/sh\nif [ \"${1:-}\" = -v ]; then exit 0; fi\n\"$@\"\n",
            );
        }
        let start = REMOTE_SCRIPT
            .find("if [ \"$(id -u)\" -eq 0 ]; then")
            .expect("privilege setup");
        let end = REMOTE_SCRIPT[start..]
            .find("if [ \"$action\" = preflight ]; then")
            .map(|offset| start + offset)
            .expect("install setup");
        let script = format!("{}\nas_root printf ready", &REMOTE_SCRIPT[start..end]);
        Command::new("/bin/sh")
            .args(["-c", &script])
            .env("PATH", tools.path())
            .env("DOWE_TEST_UID", uid)
            .output()
            .expect("shell")
    }

    #[cfg(unix)]
    fn write_executable(path: &std::path::Path, content: &str) {
        std::fs::write(path, content).expect("executable");
        let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        std::fs::set_permissions(path, permissions).expect("permissions");
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
        runtime[64..72].copy_from_slice(SSH_TRAILER_MAGIC);
        runtime
    }
}
