#[cfg(test)]
mod tests {
    use super::{
        EmbeddedSshMetadata, REMOTE_SCRIPT, SshDestination, download_linux_runtime_on_worker,
        generate_ssh_with_runtime, materialize_embedded_ssh_executable, publish_ssh,
        validate_linux_amd64_runtime, write_environment_file,
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
            &[(
                "DATABASE_URL".into(),
                "postgres://private.example/app".into(),
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
        assert_eq!(
            std::fs::read_to_string(materialized.path().join(".env.live"))
                .expect("selected client environment"),
            "PUBLIC_URL=\"https://example.com/path?x=1&y=2\"\n"
        );
        assert!(output.path().join("deploy.json").is_file());
        assert!(
            !std::fs::read_to_string(output.path().join("deploy.json"))
                .expect("manifest")
                .contains("password")
        );
        assert!(
            !std::fs::read(&package.executable)
                .expect("executable")
                .windows(b"postgres://private.example/app".len())
                .any(|window| window == b"postgres://private.example/app")
        );
    }

    #[test]
    fn writes_server_environment_as_escaped_dotenv() {
        let mut output = Vec::new();
        write_environment_file(
            &mut output,
            &[(
                "DATABASE_URL".into(),
                "postgres://db.example/app\"line\nnext".into(),
            )],
        )
        .expect("environment file");

        assert_eq!(
            String::from_utf8(output).expect("utf8"),
            "DATABASE_URL=\"postgres://db.example/app\\\"line\\nnext\"\n"
        );
    }

    #[test]
    fn dry_run_reports_sanitized_password_auth_without_connecting() {
        let output = tempfile::tempdir().expect("output");
        let executable = output.path().join("app");
        std::fs::write(&executable, linux_runtime()).expect("executable");
        let package = super::SshPackage {
            executable,
            server_environment: Vec::new(),
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

