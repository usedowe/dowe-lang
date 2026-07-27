use dowe_compiler::compile_dev;
use dowe_runtime::serve_production;
use std::net::SocketAddr;
use std::path::PathBuf;

const SERVER_USAGE: &str = "Usage: dowe server --root <path> [--bind <ip:port>]";

#[derive(Debug, PartialEq, Eq)]
struct ServerOptions {
    root: PathBuf,
    bind: SocketAddr,
}

pub(crate) async fn run_server_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_server_options(args)?;
    let project = compile_dev(&options.root)?;
    if !project.capabilities.server {
        return Err("dowe server requires `server` in main.dowe".into());
    }
    serve_production(project, options.bind).await?;
    Ok(())
}

fn parse_server_options(args: &[String]) -> Result<ServerOptions, Box<dyn std::error::Error>> {
    let mut root = None;
    let mut bind = "0.0.0.0:8080".parse::<SocketAddr>()?;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                root = Some(PathBuf::from(required_value(args, index, "--root")?));
                index += 2;
            }
            "--bind" => {
                bind = required_value(args, index, "--bind")?.parse()?;
                index += 2;
            }
            _ => return Err(SERVER_USAGE.into()),
        }
    }
    Ok(ServerOptions {
        root: root.ok_or(SERVER_USAGE)?,
        bind,
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
}
