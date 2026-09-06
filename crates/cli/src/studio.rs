use crate::usage::USAGE;
use dowe_runtime::run_studio;
use std::env;
use std::path::PathBuf;

pub(crate) async fn run_studio_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = parse_root(args)?.unwrap_or(env::current_dir()?);
    run_studio(root).await?;
    Ok(())
}

fn parse_root(args: &[String]) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Ok(None);
    }
    if args.len() == 2 && args[0] == "--root" {
        return Ok(Some(PathBuf::from(&args[1])));
    }
    Err(USAGE.into())
}

#[cfg(test)]
mod tests {
    use super::parse_root;
    use std::path::PathBuf;

    #[test]
    fn parses_explicit_project_root() {
        let args = vec!["--root".to_string(), "/tmp/example".to_string()];
        assert_eq!(
            parse_root(&args).expect("parse"),
            Some(PathBuf::from("/tmp/example"))
        );
    }

    #[test]
    fn accepts_current_directory_default() {
        assert_eq!(parse_root(&[]).expect("parse"), None);
    }

    #[test]
    fn rejects_unknown_options() {
        let args = vec!["--target".to_string(), "web".to_string()];
        assert!(parse_root(&args).is_err());
    }
}
