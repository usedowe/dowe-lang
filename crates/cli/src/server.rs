use dowe_compiler::compile_dev;
use dowe_runtime::{ProductionAccess, serve_production_with_access};
use std::net::SocketAddr;
use std::path::PathBuf;

const SERVER_USAGE: &str = "Usage: dowe server (--root <path>|--artifact <path>) [--bind <ip:port>] [--environment stage|uat --access-hash <sha256>]";

#[derive(Debug, PartialEq, Eq)]
struct ServerOptions {
    root: PathBuf,
    artifact: Option<PathBuf>,
    bind: SocketAddr,
    access: Option<ProductionAccess>,
}

pub(crate) async fn run_embedded_ssh_server() -> Result<bool, Box<dyn std::error::Error>> {
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
    let Some(metadata) = dowe_deploy::materialize_embedded_ssh_executable(&executable, &root)?
    else {
        return Ok(false);
    };
    let project = compile_dev(&root)?;
    if !project.capabilities.server {
        return Err("embedded SSH deploy requires `server` in main.dowe".into());
    }
    let access = match metadata.access_hash.as_deref() {
        Some(hash) => Some(ProductionAccess::new(metadata.environment.as_str(), hash)?),
        None => None,
    };
    serve_production_with_access(project, metadata.bind.parse()?, access).await?;
    Ok(true)
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
    let project = compile_dev(root)?;
    if !project.capabilities.server {
        return Err("dowe server requires `server` in main.dowe".into());
    }
    serve_production_with_access(project, options.bind, options.access).await?;
    Ok(())
}

fn parse_server_options(args: &[String]) -> Result<ServerOptions, Box<dyn std::error::Error>> {
    let mut root = None;
    let mut artifact = None;
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
    use super::parse_server_options;
    use std::net::SocketAddr;
    use std::path::PathBuf;

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
}
