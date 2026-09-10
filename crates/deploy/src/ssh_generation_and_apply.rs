fn generate_ssh_with_runtime(
    root: &Path,
    output: &Path,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    client_environment: &[(String, String)],
    server_environment: &[(String, String)],
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
    let sha256 = Sha256::digest(&executable)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
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
        server_environment: server_environment.to_vec(),
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
    let remote_env_upload = format!("/tmp/{}.env.upload", package.service_name);
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
        "sh -s -- install {} {} {} {} {}",
        shell_word(&destination.user),
        shell_word(&remote_upload),
        shell_word(&remote_env_upload),
        shell_word(&package.service_name),
        shell_word(&package.binary_name),
    );
    let preflight_command = format!(
        "sh -c {} -- preflight {}",
        shell_word(REMOTE_SCRIPT),
        shell_word(&destination.user),
    );
    let install_command = format!(
        "sh -c {} -- install {} {} {} {} {}",
        shell_word(REMOTE_SCRIPT),
        shell_word(&destination.user),
        shell_word(&remote_upload),
        shell_word(&remote_env_upload),
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
    let mut environment_file = tempfile::NamedTempFile::new()?;
    write_environment_file(environment_file.as_file_mut(), &package.server_environment)?;
    let mut environment_scp_args = destination.auth_args();
    if let Some(path) = control_path.as_ref() {
        environment_scp_args.extend([
            "-o".into(),
            "ControlMaster=auto".into(),
            "-o".into(),
            "ControlPersist=60".into(),
            "-o".into(),
            format!("ControlPath={path}"),
        ]);
    }
    environment_scp_args.extend([
        environment_file.path().display().to_string(),
        format!("{}:{remote_env_upload}", destination.target()),
    ]);
    if let Err(error) = run_inherited("scp", &environment_scp_args) {
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
        metadata.environment,
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


