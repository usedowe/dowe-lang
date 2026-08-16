use dowe_compiler::{
    ProjectCapabilities, compile_dev_server, compile_dev_web, compile_for_server_environment,
    compile_for_web_environment,
};
use dowe_runtime::{ProductionAccess, serve_production_with_access};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const SERVER_USAGE: &str = "Usage: dowe server (--root <path>|--artifact <path>) [--surface server|web] [--bind <ip:port>] [--environment stage|uat --access-hash <sha256>]";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerSurface {
    Server,
    Web,
}

#[derive(Debug, PartialEq, Eq)]
struct ServerOptions {
    root: PathBuf,
    artifact: Option<PathBuf>,
    surface: ServerSurface,
    bind: SocketAddr,
    access: Option<ProductionAccess>,
}

pub(crate) async fn run_embedded_server() -> Result<bool, Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let temporary = std::env::var_os("DOWE_SSH_APP_ROOT")
        .is_none()
        .then(tempfile::tempdir)
        .transpose()?;
    let root = match std::env::var_os("DOWE_SSH_APP_ROOT").map(PathBuf::from) {
        Some(root)
            if root.is_absolute()
                && root.starts_with("/var/lib/dowe")
                && root.components().count() >= 6 =>
        {
            root
        }
        Some(_) => {
            return Err("DOWE_SSH_APP_ROOT must stay below /var/lib/dowe/<service>/app".into());
        }
        None => temporary
            .as_ref()
            .map(|root| root.path().to_path_buf())
            .ok_or("embedded SSH runtime root is missing")?,
    };
    let root = root.canonicalize()?;
    if std::env::var_os("DOWE_SSH_APP_ROOT").is_some() && !root.starts_with("/var/lib/dowe") {
        return Err("DOWE_SSH_APP_ROOT resolves outside /var/lib/dowe".into());
    }
    let application = dowe_deploy::materialize_embedded_application_executable(&executable, &root)?;
    let (surface, environment, access_hash, bind) = match application {
        Some(metadata) => (
            metadata.surface,
            metadata.environment,
            metadata.access_hash,
            metadata.bind,
        ),
        None => {
            let Some(metadata) =
                dowe_deploy::materialize_embedded_ssh_executable(&executable, &root)?
            else {
                return Ok(false);
            };
            (
                dowe_deploy::DeploySurface::Server,
                metadata.environment,
                metadata.access_hash,
                metadata.bind,
            )
        }
    };
    let project = compile_embedded_project(&root, surface, environment)?;
    let server_surface = match surface {
        dowe_deploy::DeploySurface::Server => ServerSurface::Server,
        dowe_deploy::DeploySurface::Web => ServerSurface::Web,
        dowe_deploy::DeploySurface::Android | dowe_deploy::DeploySurface::Ios => unreachable!(),
    };
    if let Some(error) = production_capability_error(server_surface, project.capabilities) {
        return Err(error.into());
    }
    let access = match access_hash.as_deref() {
        Some(hash) => Some(ProductionAccess::new(environment.as_str(), hash)?),
        None => None,
    };
    serve_production_with_access(project, bind.parse()?, access).await?;
    Ok(true)
}

fn compile_embedded_project(
    root: &Path,
    surface: dowe_deploy::DeploySurface,
    environment: dowe_deploy::DeployEnvironment,
) -> Result<dowe_compiler::CompiledProject, Box<dyn std::error::Error>> {
    Ok(match surface {
        dowe_deploy::DeploySurface::Server => {
            compile_for_server_environment(&root, environment.compile_environment())?
        }
        dowe_deploy::DeploySurface::Web => {
            compile_for_web_environment(&root, environment.compile_environment())?
        }
        dowe_deploy::DeploySurface::Android | dowe_deploy::DeploySurface::Ios => {
            return Err("embedded deploy surface is not supported by the server runtime".into());
        }
    })
}

pub(crate) async fn run_server_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_server_options(args)?;
    let artifact_root = options
        .artifact
        .as_ref()
        .map(|artifact| {
            let root = tempfile::tempdir()?;
            dowe_deploy::materialize_cloud_artifact(artifact, root.path())?;
            Ok::<_, Box<dyn std::error::Error>>(root)
        })
        .transpose()?;
    let root = artifact_root
        .as_ref()
        .map(|root| root.path())
        .unwrap_or(&options.root);
    let project = match options.surface {
        ServerSurface::Server => compile_dev_server(root)?,
        ServerSurface::Web => compile_dev_web(root)?,
    };
    if let Some(error) = production_capability_error(options.surface, project.capabilities) {
        return Err(error.into());
    }
    serve_production_with_access(project, options.bind, options.access).await?;
    Ok(())
}

fn production_capability_error(
    surface: ServerSurface,
    capabilities: ProjectCapabilities,
) -> Option<&'static str> {
    match surface {
        ServerSurface::Server => (!capabilities.server)
            .then_some("dowe server --surface server requires `server` in main.dowe"),
        ServerSurface::Web => (!capabilities.views)
            .then_some("dowe server --surface web requires `views` in main.dowe"),
    }
}

fn parse_server_options(args: &[String]) -> Result<ServerOptions, Box<dyn std::error::Error>> {
    let mut root = None;
    let mut artifact = None;
    let mut surface = ServerSurface::Server;
    let mut bind = "0.0.0.0:8080".parse::<SocketAddr>()?;
    let mut index = 0usize;
    let mut environment = None;
    let mut access_hash = None;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                root = Some(PathBuf::from(required_value(args, index, "--root")?));
                index += 2;
            }
            "--artifact" => {
                artifact = Some(PathBuf::from(required_value(args, index, "--artifact")?));
                index += 2;
            }
            "--surface" => {
                surface = match required_value(args, index, "--surface")? {
                    "server" => ServerSurface::Server,
                    "web" => ServerSurface::Web,
                    _ => return Err(SERVER_USAGE.into()),
                };
                index += 2;
            }
            "--bind" => {
                bind = required_value(args, index, "--bind")?.parse()?;
                index += 2;
            }
            "--environment" => {
                environment = Some(required_value(args, index, "--environment")?.to_string());
                index += 2;
            }
            "--access-hash" => {
                access_hash = Some(required_value(args, index, "--access-hash")?.to_string());
                index += 2;
            }
            _ => return Err(SERVER_USAGE.into()),
        }
    }
    let access = match (environment, access_hash) {
        (None, None) => None,
        (Some(environment), Some(access_hash)) => {
            Some(ProductionAccess::new(environment, &access_hash)?)
        }
        _ => return Err("--environment and --access-hash must be provided together".into()),
    };
    if root.is_some() == artifact.is_some() {
        return Err(SERVER_USAGE.into());
    }
    Ok(ServerOptions {
        root: root.unwrap_or_default(),
        artifact,
        surface,
        bind,
        access,
    })
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    name: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("{name} requires a value").into())
}

#[cfg(test)]
mod tests {
    use super::{
        ServerSurface, compile_embedded_project, parse_server_options, production_capability_error,
    };
    use dowe_compiler::ProjectCapabilities;
    use dowe_deploy::{DeployEnvironment, DeploySurface};
    use std::fs;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn embedded_compilation_uses_the_selected_deploy_profile() {
        let temp = TempDir::new().expect("tempdir");
        fs::write(
            temp.path().join("main.dowe"),
            "main\n  server port:8080\n    route \"/status\"\n      response text:\"OK\"\n",
        )
        .expect("main");
        fs::write(
            temp.path().join(".env.example"),
            "DOWE_TEST_EMBEDDED_URL=\n",
        )
        .expect("env example");
        fs::write(
            temp.path().join(".env.live"),
            "DOWE_TEST_EMBEDDED_URL=https://live.example.com\n",
        )
        .expect("live environment");
        fs::write(
            temp.path().join(".env.stage"),
            "DOWE_TEST_EMBEDDED_URL=https://stage.example.com\n",
        )
        .expect("stage environment");

        let project =
            compile_embedded_project(temp.path(), DeploySurface::Server, DeployEnvironment::Stage)
                .expect("embedded project");

        assert_eq!(
            project
                .environment_config
                .variable("DOWE_TEST_EMBEDDED_URL")
                .and_then(|variable| variable.resolved_value.as_deref()),
            Some("https://stage.example.com")
        );
    }

    #[test]
    fn parses_native_server_options() {
        let options = parse_server_options(&[
            "--root".to_string(),
            "/app".to_string(),
            "--bind".to_string(),
            "127.0.0.1:9090".to_string(),
        ])
        .expect("options");

        assert_eq!(options.root, PathBuf::from("/app"));
        assert!(options.artifact.is_none());
        assert_eq!(options.surface, ServerSurface::Server);
        assert_eq!(
            options.bind,
            "127.0.0.1:9090".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn defaults_native_server_bind() {
        let options =
            parse_server_options(&["--root".to_string(), "/app".to_string()]).expect("options");

        assert_eq!(options.bind, "0.0.0.0:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn requires_native_server_root() {
        assert!(parse_server_options(&[]).is_err());
    }

    #[test]
    fn parses_cloud_artifact_server_options() {
        let options = parse_server_options(&[
            "--artifact".to_string(),
            "/artifacts/app.dowebin".to_string(),
        ])
        .expect("options");
        assert_eq!(
            options.artifact,
            Some(PathBuf::from("/artifacts/app.dowebin"))
        );
        assert!(
            parse_server_options(&[
                "--root".to_string(),
                "/app".to_string(),
                "--artifact".to_string(),
                "/app.dowebin".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn parses_web_surface_option() {
        let options = parse_server_options(&[
            "--root".to_string(),
            "/app".to_string(),
            "--surface".to_string(),
            "web".to_string(),
        ])
        .expect("options");

        assert_eq!(options.surface, ServerSurface::Web);
    }

    #[test]
    fn parses_protected_environment_options_atomically() {
        let options = parse_server_options(&[
            "--root".to_string(),
            "/app".to_string(),
            "--environment".to_string(),
            "stage".to_string(),
            "--access-hash".to_string(),
            "0".repeat(64),
        ])
        .expect("options");

        assert!(options.access.is_some());
        assert!(
            parse_server_options(&[
                "--root".to_string(),
                "/app".to_string(),
                "--environment".to_string(),
                "stage".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn production_server_validates_the_selected_surface() {
        assert_eq!(
            production_capability_error(
                ServerSurface::Web,
                ProjectCapabilities {
                    server: false,
                    views: true,
                },
            ),
            None
        );
        assert!(
            production_capability_error(
                ServerSurface::Server,
                ProjectCapabilities {
                    server: false,
                    views: true,
                },
            )
            .is_some()
        );
        assert!(
            production_capability_error(
                ServerSurface::Web,
                ProjectCapabilities {
                    server: true,
                    views: false,
                },
            )
            .is_some()
        );
    }
}
