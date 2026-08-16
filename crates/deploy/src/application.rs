use crate::access::DeployAccess;
use crate::cloud;
use crate::embedded::{
    DOCKER_TRAILER_MAGIC, encode_embedded_payload, materialize_application, read_embedded_payload,
    set_executable, validate_access_metadata, validate_client_environment,
};
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeploySurface};
use crate::ssh::validate_linux_amd64_runtime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

pub(crate) const EXECUTABLE_NAME: &str = "dowe-app";

#[derive(Clone, Debug)]
pub(crate) struct ApplicationPackage {
    pub executable: PathBuf,
    pub sha256: String,
    pub size: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedApplicationMetadata {
    pub surface: DeploySurface,
    pub environment: DeployEnvironment,
    pub access_hash: Option<String>,
    pub bind: String,
    pub client_environment: Vec<(String, String)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutableMetadata<'a> {
    surface: DeploySurface,
    environment: DeployEnvironment,
    access_hash: Option<&'a str>,
    bind: &'a str,
    client_environment: &'a [(String, String)],
}

pub(crate) fn generate_embedded_application(
    root: &Path,
    output: &Path,
    surface: DeploySurface,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    client_environment: &[(String, String)],
    bind: &str,
    runtime: &[u8],
) -> DeployResult<ApplicationPackage> {
    validate_surface(surface)?;
    validate_linux_amd64_runtime(
        runtime,
        DOCKER_TRAILER_MAGIC,
        "embedded Docker applications",
    )?;
    validate_bind(bind)?;
    let application = cloud::application_binary(root)?;
    let metadata = serde_json::to_vec(&ExecutableMetadata {
        surface,
        environment,
        access_hash: access.map(|value| value.password_hash.as_str()),
        bind,
        client_environment,
    })?;
    let executable =
        encode_embedded_payload(runtime, &application, &metadata, DOCKER_TRAILER_MAGIC);
    let executable_path = output.join(EXECUTABLE_NAME);
    write_file(&executable_path, &executable)?;
    set_executable(&executable_path)?;
    Ok(ApplicationPackage {
        executable: executable_path,
        sha256: format!("{:x}", Sha256::digest(&executable)),
        size: executable.len(),
    })
}

pub fn materialize_embedded_application_executable(
    executable: &Path,
    output: &Path,
) -> DeployResult<Option<EmbeddedApplicationMetadata>> {
    let Some(payload) =
        read_embedded_payload(executable, DOCKER_TRAILER_MAGIC, "Docker application")?
    else {
        return Ok(None);
    };
    let metadata = serde_json::from_slice::<EmbeddedApplicationMetadata>(&payload.metadata)
        .map_err(|_| DeployError::new("invalid embedded application metadata"))?;
    validate_metadata(&metadata)?;
    materialize_application(
        output,
        &payload.application,
        &metadata.client_environment,
        metadata.environment,
        "application",
    )?;
    Ok(Some(metadata))
}

fn validate_metadata(metadata: &EmbeddedApplicationMetadata) -> DeployResult<()> {
    validate_surface(metadata.surface)?;
    validate_bind(&metadata.bind)?;
    validate_access_metadata(
        metadata.environment,
        metadata.access_hash.as_deref(),
        "application",
    )?;
    validate_client_environment(&metadata.client_environment, "application")
}

fn validate_surface(surface: DeploySurface) -> DeployResult<()> {
    if !matches!(surface, DeploySurface::Server | DeploySurface::Web) {
        return Err(DeployError::new(
            "invalid embedded application deploy surface",
        ));
    }
    Ok(())
}

fn validate_bind(bind: &str) -> DeployResult<()> {
    let address = bind
        .parse::<SocketAddr>()
        .map_err(|_| DeployError::new("invalid embedded application bind address"))?;
    if address.ip() != IpAddr::from([0, 0, 0, 0]) || address.port() == 0 {
        return Err(DeployError::new(
            "invalid embedded application bind address",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        EmbeddedApplicationMetadata, generate_embedded_application,
        materialize_embedded_application_executable,
    };
    use crate::embedded::encode_embedded_payload;
    use crate::model::{DeployEnvironment, DeploySurface};

    #[test]
    fn server_application_round_trips_custom_bind_and_source() {
        let project = tempfile::tempdir().expect("project");
        let output = tempfile::tempdir().expect("output");
        std::fs::write(
            project.path().join("main.dowe"),
            "main\n  server port:8081\n    route \"/status\"\n      response text:\"OK\"\n",
        )
        .expect("main");
        let package = generate_embedded_application(
            project.path(),
            output.path(),
            DeploySurface::Server,
            DeployEnvironment::Live,
            None,
            &[("PUBLIC_URL".into(), "https://example.com".into())],
            "0.0.0.0:8081",
            &linux_application_runtime(),
        )
        .expect("package");
        let materialized = tempfile::tempdir().expect("materialized");
        let metadata =
            materialize_embedded_application_executable(&package.executable, materialized.path())
                .expect("materialize")
                .expect("metadata");

        assert_eq!(
            metadata,
            EmbeddedApplicationMetadata {
                surface: DeploySurface::Server,
                environment: DeployEnvironment::Live,
                access_hash: None,
                bind: "0.0.0.0:8081".into(),
                client_environment: vec![("PUBLIC_URL".into(), "https://example.com".into())],
            }
        );
        assert!(materialized.path().join("main.dowe").is_file());
        assert_eq!(
            std::fs::read_to_string(materialized.path().join(".env")).expect("environment"),
            "PUBLIC_URL=\"https://example.com\"\n"
        );
    }

    #[test]
    fn web_application_round_trips_without_a_server() {
        let project = tempfile::tempdir().expect("project");
        let output = tempfile::tempdir().expect("output");
        std::fs::create_dir_all(project.path().join("views")).expect("views");
        std::fs::write(
            project.path().join("main.dowe"),
            "import viewRoutes from \"@/views/routes\"\n\nmain\n  views:viewRoutes\n",
        )
        .expect("main");
        std::fs::write(
            project.path().join("views/routes.dowe"),
            "views viewRoutes\n",
        )
        .expect("views");
        let package = generate_embedded_application(
            project.path(),
            output.path(),
            DeploySurface::Web,
            DeployEnvironment::Live,
            None,
            &[],
            "0.0.0.0:8080",
            &linux_application_runtime(),
        )
        .expect("package");
        let materialized = tempfile::tempdir().expect("materialized");

        let metadata =
            materialize_embedded_application_executable(&package.executable, materialized.path())
                .expect("materialize")
                .expect("metadata");

        assert_eq!(metadata.surface, DeploySurface::Web);
        assert_eq!(metadata.bind, "0.0.0.0:8080");
        assert!(materialized.path().join("views/routes.dowe").is_file());
    }

    #[test]
    fn server_application_rejects_a_runtime_without_application_support() {
        let project = tempfile::tempdir().expect("project");
        let output = tempfile::tempdir().expect("output");
        std::fs::write(
            project.path().join("main.dowe"),
            "main\n  server port:8080\n    route \"/status\"\n      response text:\"OK\"\n",
        )
        .expect("main");
        let mut runtime = linux_application_runtime();
        runtime[64..72].copy_from_slice(b"DOWESSH1");

        let error = generate_embedded_application(
            project.path(),
            output.path(),
            DeploySurface::Server,
            DeployEnvironment::Live,
            None,
            &[],
            "0.0.0.0:8080",
            &runtime,
        )
        .expect_err("unsupported runtime");

        assert!(error.to_string().contains("embedded Docker applications"));
    }

    #[test]
    fn server_materializer_ignores_desktop_executables() {
        let executable = tempfile::NamedTempFile::new().expect("executable");
        let bytes = encode_embedded_payload(
            &linux_application_runtime(),
            b"desktop application",
            b"{}",
            b"DOWEAPP1",
        );
        std::fs::write(executable.path(), bytes).expect("desktop executable");
        let output = tempfile::tempdir().expect("output");

        assert!(
            materialize_embedded_application_executable(executable.path(), output.path())
                .expect("materialize")
                .is_none()
        );
    }

    fn linux_application_runtime() -> Vec<u8> {
        let mut runtime = vec![0u8; 96];
        runtime[..4].copy_from_slice(b"\x7fELF");
        runtime[4] = 2;
        runtime[5] = 1;
        runtime[18..20].copy_from_slice(&62u16.to_le_bytes());
        runtime[64..72].copy_from_slice(super::DOCKER_TRAILER_MAGIC);
        runtime
    }
}
