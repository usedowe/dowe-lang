use crate::usage::USAGE;
use reqwest::Client;
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_BASE_URL: &str = "https://get.dowe.dev";

pub(crate) async fn run_upgrade_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() {
        return Err(USAGE.into());
    }

    let base_url = upgrade_base_url();
    let current_version = env!("CARGO_PKG_VERSION");
    let client = Client::new();
    let latest_raw = client
        .get(latest_version_url(&base_url))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let latest_version = normalize_version(&latest_raw)?;

    match compare_versions(&latest_version, current_version)? {
        Ordering::Greater => {
            println!("upgrading dowe {current_version} to {latest_version}");
            let installer_url = installer_url(&base_url, current_installer_kind()?);
            let installer_path = download_installer(&client, &installer_url).await?;
            let result = run_installer(&installer_path, &base_url);
            let cleanup_result = fs::remove_file(&installer_path);
            if result.is_ok() {
                cleanup_result?;
            }
            result?;
            Ok(())
        }
        _ => {
            println!("dowe is already up to date ({current_version})");
            Ok(())
        }
    }
}

fn upgrade_base_url() -> String {
    env::var("DOWE_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn latest_version_url(base_url: &str) -> String {
    format!("{}/latest/version", base_url.trim_end_matches('/'))
}

fn installer_url(base_url: &str, kind: InstallerKind) -> String {
    match kind {
        InstallerKind::Unix => format!("{}/install", base_url.trim_end_matches('/')),
        InstallerKind::Windows => format!("{}/install.ps1", base_url.trim_end_matches('/')),
    }
}

async fn download_installer(
    client: &Client,
    installer_url: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bytes = client
        .get(installer_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let path = temp_installer_path(installer_url);
    fs::write(&path, bytes)?;
    Ok(path)
}

fn temp_installer_path(installer_url: &str) -> PathBuf {
    let suffix = if installer_url.ends_with(".ps1") {
        "ps1"
    } else {
        "sh"
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "dowe-upgrade-{}-{now}.{suffix}",
        std::process::id()
    ))
}

fn run_installer(path: &Path, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = installer_command(path)?;
    command.env("DOWE_BASE_URL", base_url);
    command.env("DOWE_VERSION", "latest");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("dowe installer failed with {status}").into())
    }
}

fn installer_command(path: &Path) -> Result<Command, Box<dyn std::error::Error>> {
    match current_installer_kind()? {
        InstallerKind::Unix => {
            let mut command = Command::new("bash");
            command.arg(path);
            Ok(command)
        }
        InstallerKind::Windows => {
            let mut command = Command::new("powershell");
            command
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(path);
            Ok(command)
        }
    }
}

fn current_installer_kind() -> Result<InstallerKind, Box<dyn std::error::Error>> {
    if cfg!(windows) {
        return Ok(InstallerKind::Windows);
    }
    if cfg!(unix) {
        return Ok(InstallerKind::Unix);
    }
    Err("dowe upgrade is not supported on this platform".into())
}

fn normalize_version(raw: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value = raw.trim().trim_start_matches('v');
    parse_semver(value)?;
    Ok(value.to_string())
}

fn compare_versions(left: &str, right: &str) -> Result<Ordering, Box<dyn std::error::Error>> {
    Ok(parse_semver(left)?.cmp(&parse_semver(right)?))
}

fn parse_semver(value: &str) -> Result<Semver, Box<dyn std::error::Error>> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(format!("invalid Dowe version: {value}").into());
    }
    Ok(Semver {
        major: parse_version_part(parts[0], value)?,
        minor: parse_version_part(parts[1], value)?,
        patch: parse_version_part(parts[2], value)?,
    })
}

fn parse_version_part(part: &str, source: &str) -> Result<u64, Box<dyn std::error::Error>> {
    if part.is_empty() || !part.chars().all(|value| value.is_ascii_digit()) {
        return Err(format!("invalid Dowe version: {source}").into());
    }
    Ok(part.parse()?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstallerKind {
    Unix,
    Windows,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Semver {
    major: u64,
    minor: u64,
    patch: u64,
}

#[cfg(test)]
mod tests {
    use super::{
        InstallerKind, compare_versions, installer_url, latest_version_url, normalize_version,
    };
    use std::cmp::Ordering;

    #[test]
    fn trims_latest_version_metadata() {
        assert_eq!(normalize_version("v1.0.0\n").expect("version"), "1.0.0");
    }

    #[test]
    fn compares_semver_numbers() {
        assert_eq!(
            compare_versions("1.0.10", "1.0.2").expect("compare"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.0", "1.0.0").expect("compare"),
            Ordering::Equal
        );
    }

    #[test]
    fn rejects_invalid_version_metadata() {
        let error = normalize_version("latest").expect_err("error");

        assert!(error.to_string().contains("invalid Dowe version"));
    }

    #[test]
    fn builds_metadata_url_without_double_slash() {
        assert_eq!(
            latest_version_url("https://get.dowe.dev/"),
            "https://get.dowe.dev/latest/version"
        );
    }

    #[test]
    fn builds_installer_urls() {
        assert_eq!(
            installer_url("https://get.dowe.dev/", InstallerKind::Unix),
            "https://get.dowe.dev/install"
        );
        assert_eq!(
            installer_url("https://get.dowe.dev", InstallerKind::Windows),
            "https://get.dowe.dev/install.ps1"
        );
    }
}
