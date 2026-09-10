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


