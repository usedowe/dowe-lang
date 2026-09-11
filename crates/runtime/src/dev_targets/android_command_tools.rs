use super::{quiet_command_options, run_required};
use crate::dev::{DevTarget, HostOs};
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_dev_info;
use dowe_spawn::{SpawnConfig, StreamMode};
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub(super) fn install(sdk: &Path) -> RuntimeResult<()> {
    static INSTALLATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = INSTALLATION
        .lock()
        .map_err(|_| RuntimeError::new("Android command-line tools setup lock failed"))?;
    let destination = sdk.join("cmdline-tools/dowe");
    let script = if cfg!(windows) {
        "sdkmanager.bat"
    } else {
        "sdkmanager"
    };
    if destination.join("bin").join(script).is_file() {
        return Ok(());
    }
    let (archive, checksum) = archive_for_host(HostOs::current(), std::env::consts::ARCH)?;
    fs::create_dir_all(sdk.join("cmdline-tools"))?;
    let staging = tempfile::Builder::new()
        .prefix(".dowe-install-")
        .tempdir_in(sdk.join("cmdline-tools"))?;
    let download = staging.path().join("tools.zip");
    let url = format!("https://dl.google.com/android/repository/{archive}");
    log_dev_info("Android setup: downloading official Android command-line tools");
    let mut config = SpawnConfig::new(
        if cfg!(windows) { "curl.exe" } else { "curl" },
        [
            "--fail",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--connect-timeout",
            "30",
            "--max-time",
            "1200",
            "--output",
            &download.to_string_lossy(),
            &url,
        ],
    )
    .with_options(quiet_command_options(None, StreamMode::Inherit));
    config.options.stderr = StreamMode::Inherit;
    config.options.timeout_ms = Some(1_230_000);
    run_required(DevTarget::Android, config)?;
    verify_checksum(&download, checksum)?;
    extract_archive(&download, staging.path())?;
    if !staging
        .path()
        .join("cmdline-tools/bin")
        .join(script)
        .is_file()
    {
        return Err(RuntimeError::new(
            "Android setup: downloaded archive does not contain sdkmanager",
        ));
    }
    if destination.exists() {
        return Err(RuntimeError::new(
            "Android setup: the Dowe command-line tools directory is incomplete; move it aside and retry",
        ));
    }
    fs::rename(staging.path().join("cmdline-tools"), destination)?;
    Ok(())
}

fn archive_for_host(
    host: HostOs,
    architecture: &str,
) -> RuntimeResult<(&'static str, &'static str)> {
    match (host, architecture) {
        (HostOs::Macos, "aarch64") => Ok((
            "commandlinetools-mac_arm64-16111833_latest.zip",
            "ad03dc49bfacfd52c110b14104ea548b8a07e830",
        )),
        (HostOs::Macos, "x86_64") => Ok((
            "commandlinetools-mac_x86_64-16111833_latest.zip",
            "112cf9618794a997ff273537d55bee02c22abffe",
        )),
        (HostOs::Linux, "x86_64") => Ok((
            "commandlinetools-linux-16111833_latest.zip",
            "e025545c62a8e64c7559119566a569fb1dec5f60",
        )),
        (HostOs::Windows, "x86_64") => Ok((
            "commandlinetools-win-16111833_latest.zip",
            "57d04f2d75eb8e8fffc5000a987e5de4b5a63e9d",
        )),
        _ => Err(RuntimeError::new(
            "Android setup: automatic command-line tools installation is unavailable on this host architecture; install the official Android SDK tools and retry",
        )),
    }
}

fn verify_checksum(path: &Path, expected: &str) -> RuntimeResult<()> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha1::new();
    let mut buffer = [0; 65_536];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    let checksum = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if checksum != expected {
        return Err(RuntimeError::new(
            "Android setup: command-line tools archive failed the checksum published by Google",
        ));
    }
    Ok(())
}

fn extract_archive(path: &Path, destination: &Path) -> RuntimeResult<()> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path)?).map_err(|error| {
        RuntimeError::new(format!(
            "Android setup: invalid command-line tools archive: {error}"
        ))
    })?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| RuntimeError::new("Android setup: unsafe archive path"))?;
        if !relative.starts_with("cmdline-tools")
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(RuntimeError::new(
                "Android setup: unexpected archive path or symbolic link",
            ));
        }
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
            continue;
        }
        fs::create_dir_all(output.parent().expect("archive parent"))?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)?;
        std::io::copy(&mut entry, &mut file)?;
        file.flush()?;
        set_archive_permissions(&output, entry.unix_mode())?;
    }
    Ok(())
}

fn set_archive_permissions(path: &Path, mode: Option<u32>) -> RuntimeResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            path,
            fs::Permissions::from_mode(mode.unwrap_or(0o644) & 0o777),
        )?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_corrupted_android_command_line_tools_download() {
        let root = tempfile::tempdir().unwrap();
        let archive = root.path().join("tools.zip");
        fs::write(&archive, "corrupted").unwrap();
        assert!(verify_checksum(&archive, "0000000000000000000000000000000000000000").is_err());
    }

    #[test]
    fn selects_official_android_archive_for_host_architecture() {
        let (arm, hash) = archive_for_host(HostOs::Macos, "aarch64").unwrap();
        assert!(arm.starts_with("commandlinetools-mac_arm64-"));
        assert_eq!(hash.len(), 40);
        assert!(archive_for_host(HostOs::Linux, "armv7").is_err());
    }

    #[test]
    fn rejects_android_tools_archive_path_traversal() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("tools.zip");
        let mut archive = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        archive
            .start_file("../outside", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"bad").unwrap();
        archive.finish().unwrap();
        assert!(extract_archive(&path, root.path()).is_err());
        assert!(!root.path().parent().unwrap().join("outside").exists());
    }
    #[test]
    fn extracts_android_tools_and_preserves_executable_permissions() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("tools.zip");
        let mut archive = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
        archive
            .start_file("cmdline-tools/bin/sdkmanager", options)
            .unwrap();
        archive.write_all(b"tool").unwrap();
        archive.finish().unwrap();
        extract_archive(&path, root.path()).unwrap();
        let script = root.path().join("cmdline-tools/bin/sdkmanager");
        assert_eq!(fs::read(&script).unwrap(), b"tool");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(script).unwrap().permissions().mode() & 0o777,
                0o755
            );
        }
    }
}
