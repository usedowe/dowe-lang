use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const VERSION: &str = "8.13";
const WRAPPER_JAR_URL: &str =
    "https://raw.githubusercontent.com/gradle/gradle/v8.13.0/gradle/wrapper/gradle-wrapper.jar";
const WRAPPER_JAR_SHA256: &str = "81a82aaea5abcc8ff68b3dfcb58b3c3c429378efd98e7433460610fecd7ae45f";
const DISTRIBUTION_SHA256: &str =
    "20f1b1176237254a6fc204d8434196fa11a4cfb387567519c61556e8710aed78";

pub(crate) struct GradleToolchain {
    pub wrapper_jar: PathBuf,
    pub user_home: PathBuf,
}

pub(crate) fn prepare(project_root: &Path, dry_run: bool) -> DeployResult<GradleToolchain> {
    let root = project_root.join(".dowe/toolchains/gradle");
    let wrapper = root.join(VERSION).join("gradle/wrapper");
    let wrapper_jar = wrapper.join("gradle-wrapper.jar");
    let properties = wrapper.join("gradle-wrapper.properties");
    if !dry_run {
        write_file(&properties, wrapper_properties())?;
        ensure_wrapper_jar(&wrapper_jar)?;
    }
    Ok(GradleToolchain {
        wrapper_jar,
        user_home: root.join("user-home"),
    })
}

fn ensure_wrapper_jar(path: &Path) -> DeployResult<()> {
    if path.is_file() && checksum(&fs::read(path)?) == WRAPPER_JAR_SHA256 {
        return Ok(());
    }
    let bytes = std::thread::spawn(download_wrapper_jar)
        .join()
        .map_err(|_| DeployError::new("Gradle Wrapper download worker failed"))??;
    if checksum(&bytes) != WRAPPER_JAR_SHA256 {
        return Err(DeployError::new(
            "downloaded Gradle Wrapper failed SHA-256 verification",
        ));
    }
    write_file(path, bytes)
}

fn download_wrapper_jar() -> DeployResult<Vec<u8>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| {
            DeployError::new(format!("failed to configure Gradle download: {error}"))
        })?;
    let response = client
        .get(WRAPPER_JAR_URL)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| DeployError::new(format!("failed to download Gradle Wrapper: {error}")))?;
    let bytes = response
        .bytes()
        .map_err(|error| DeployError::new(format!("failed to read Gradle Wrapper: {error}")))?;
    Ok(bytes.to_vec())
}

fn checksum(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn wrapper_properties() -> String {
    format!(
        "distributionBase=GRADLE_USER_HOME\ndistributionPath=wrapper/dists\ndistributionSha256Sum={DISTRIBUTION_SHA256}\ndistributionUrl=https\\://services.gradle.org/distributions/gradle-{VERSION}-bin.zip\nnetworkTimeout=60000\nvalidateDistributionUrl=true\nzipStoreBase=GRADLE_USER_HOME\nzipStorePath=wrapper/dists\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{DISTRIBUTION_SHA256, prepare, wrapper_properties};
    use tempfile::TempDir;

    #[test]
    fn dry_run_plans_gradle_without_downloading_wrapper() {
        let temp = TempDir::new().expect("tempdir");
        let toolchain = prepare(temp.path(), true).expect("toolchain");

        assert!(
            toolchain
                .wrapper_jar
                .ends_with(".dowe/toolchains/gradle/8.13/gradle/wrapper/gradle-wrapper.jar")
        );
        assert!(!toolchain.wrapper_jar.exists());
    }

    #[test]
    fn wrapper_properties_pin_distribution_and_checksum() {
        let properties = wrapper_properties();

        assert!(properties.contains("gradle-8.13-bin.zip"));
        assert!(properties.contains(DISTRIBUTION_SHA256));
        assert!(properties.contains("validateDistributionUrl=true"));
    }
}
