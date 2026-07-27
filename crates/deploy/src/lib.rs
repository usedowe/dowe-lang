mod cloudflare;
mod cloudflare_wasm;
mod database;
mod docker;
mod error;
mod files;
mod model;
mod package;
mod publish;

pub use docker::{DEFAULT_DOCKER_REGISTRY, DOCKER_PLATFORM, default_docker_image_name};
pub use error::{DeployError, DeployResult};
pub use model::{
    DeployOptions, DeployReport, DeploySurface, DeployTarget, available_deploy_surfaces,
    deploy_targets_for_surface,
};

use dowe_compiler::compile_dev;
use files::{collect_files, reset_dir, target_dir, web_target_dir};
use std::path::Path;

pub fn deploy(options: DeployOptions) -> DeployResult<DeployReport> {
    let root = options.root.canonicalize()?;
    let project = compile_dev(&root)?;
    if options.target.surface() == DeploySurface::Web && !project.capabilities.views {
        return Err(DeployError::new(format!(
            "deploy target `{}` requires `views` in main.dowe",
            options.target
        )));
    }
    if options.target.surface() == DeploySurface::Server && !project.capabilities.server {
        return Err(DeployError::new(format!(
            "deploy target `{}` requires `server` in main.dowe",
            options.target
        )));
    }
    let output = deploy_output_dir(&root, options.target)?;
    let cloudflare_pages_name = (options.target == DeployTarget::CloudflarePages)
        .then(|| cloudflare::pages_project_name(&project, options.name.as_deref()))
        .transpose()?;
    let docker_image = (options.target == DeployTarget::Docker)
        .then(|| {
            docker::resolve_docker_image(
                &root,
                options.registry.as_deref(),
                options.image.as_deref(),
            )
        })
        .transpose()?;
    if options.publish && options.target == DeployTarget::Docker {
        return Err(DeployError::new(
            "docker deploy builds and tags a local image; registry push is not configured",
        ));
    }
    reset_dir(&output)?;

    match options.target {
        DeployTarget::Static => package::generate_static(&root, &output)?,
        DeployTarget::Docker => docker::generate_docker(
            &root,
            &output,
            docker_image
                .as_ref()
                .ok_or_else(|| DeployError::new("docker image is missing"))?,
        )?,
        DeployTarget::Cloudflare => {
            cloudflare::generate_cloudflare(&project, &output, options.name.as_deref())?
        }
        DeployTarget::CloudflarePages => {
            let project_name = cloudflare_pages_name
                .as_deref()
                .ok_or_else(|| DeployError::new("cloudflare pages project name is missing"))?;
            package::generate_cloudflare_pages(&root, &output, project_name)?;
        }
    }
    database::write_database_artifacts(&project, &output, options.target.surface())?;

    let mut command = None;
    let mut image_built = false;
    if let Some(image) = docker_image.as_ref() {
        let outcome = docker::build_docker_image(&output, image, options.dry_run)?;
        command = Some(outcome.command);
        image_built = outcome.built;
    } else if options.publish {
        match options.target {
            DeployTarget::Cloudflare => {
                command = Some(publish::publish_cloudflare(&output, options.dry_run)?);
            }
            DeployTarget::CloudflarePages => {
                let project_name = cloudflare_pages_name
                    .as_deref()
                    .ok_or_else(|| DeployError::new("cloudflare pages project name is missing"))?;
                command = Some(publish::publish_cloudflare_pages(
                    &output,
                    project_name,
                    options.dry_run,
                )?);
            }
            DeployTarget::Static => {
                return Err(DeployError::new(
                    "static deploy generates a dist package and does not publish",
                ));
            }
            DeployTarget::Docker => {
                unreachable!("docker publish is rejected before package generation");
            }
        }
    }

    Ok(DeployReport {
        target: options.target,
        output_dir: output.clone(),
        files: collect_files(&output)?,
        command,
        published: options.publish && !options.dry_run,
        image_ref: docker_image.map(|image| image.reference),
        image_built,
    })
}

pub fn deploy_output_dir(
    root: impl AsRef<Path>,
    target: DeployTarget,
) -> DeployResult<std::path::PathBuf> {
    match target.surface() {
        DeploySurface::Web => {
            if target == DeployTarget::CloudflarePages {
                web_target_dir(root.as_ref(), target.as_str())
            } else {
                target_dir(root.as_ref(), target.as_str())
            }
        }
        DeploySurface::Server => target_dir(root.as_ref(), target.as_str()),
    }
}

#[cfg(test)]
mod tests;
