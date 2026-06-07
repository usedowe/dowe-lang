mod cloudflare;
mod error;
mod files;
mod model;
mod package;
mod publish;

pub use error::{DeployError, DeployResult};
pub use model::{DeployOptions, DeployReport, DeployTarget};

use dowe_compiler::compile_dev;
use files::{collect_files, reset_dir, target_dir};
use std::path::Path;

pub fn deploy(options: DeployOptions) -> DeployResult<DeployReport> {
    let root = options.root.canonicalize()?;
    let project = compile_dev(&root)?;
    let output = target_dir(&root, options.target.as_str())?;
    reset_dir(&output)?;

    match options.target {
        DeployTarget::Static => package::generate_static(&root, &output)?,
        DeployTarget::Docker => package::generate_docker(&root, &output)?,
        DeployTarget::Ssh => package::generate_ssh(&root, &output)?,
        DeployTarget::Cloudflare => {
            cloudflare::generate_cloudflare(&project, &output, options.name.as_deref())?
        }
    }

    let command = if options.publish {
        match options.target {
            DeployTarget::Cloudflare => {
                Some(publish::publish_cloudflare(&output, options.dry_run)?)
            }
            DeployTarget::Ssh => Some(publish::publish_ssh(&output, &options)?),
            DeployTarget::Static => {
                return Err(DeployError::new(
                    "static deploy generates a dist package and does not publish",
                ));
            }
            DeployTarget::Docker => {
                return Err(DeployError::new(
                    "docker deploy generates a build context; registry publication is not configured",
                ));
            }
        }
    } else {
        None
    };

    Ok(DeployReport {
        target: options.target,
        output_dir: output.clone(),
        files: collect_files(&output)?,
        command,
        published: options.publish,
    })
}

pub fn deploy_output_dir(
    root: impl AsRef<Path>,
    target: DeployTarget,
) -> DeployResult<std::path::PathBuf> {
    target_dir(root.as_ref(), target.as_str())
}

#[cfg(test)]
mod tests;
