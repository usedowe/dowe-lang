mod access;
mod application;
mod cloud;
mod cloudflare;
mod cloudflare_wasm;
mod database;
mod desktop_runtime;
mod docker;
mod edge_queue;
mod embedded;
mod environment;
mod error;
mod files;
mod gradle;
mod model;
mod native;
mod package;
mod preferences;
mod publish;
mod ssh;
mod vercel;

pub use application::{EmbeddedApplicationMetadata, materialize_embedded_application_executable};
pub use cloud::authenticate_cloud_session;
pub use cloud::materialize_cloud_artifact;
pub use docker::{DEFAULT_DOCKER_REGISTRY, DOCKER_PLATFORM, default_docker_image_name};
pub use error::{DeployError, DeployResult};
pub use model::{
    BuildOptions, BuildReport, BuildTarget, DeployEnvironment, DeployOptions, DeployReport,
    DeploySurface, DeployTarget, available_build_targets, available_deploy_surfaces,
    deploy_targets_for_surface,
};
pub use native::build;
pub use preferences::{
    DockerDeployPreferences, deploy_target_selection_path, docker_deploy_preferences_path,
    load_deploy_target_preference, load_docker_deploy_preferences, save_deploy_target_preference,
    save_docker_deploy_preferences,
};
pub use ssh::{EmbeddedSshMetadata, materialize_embedded_ssh_executable};

use access::DeployAccess;
use dowe_compiler::{compile_for_server_environment, compile_for_web_environment};
use environment::DeployEnvironmentValues;
use files::{collect_files, reset_dir, target_dir, web_target_dir};
use std::path::Path;

pub fn deploy(options: DeployOptions) -> DeployResult<DeployReport> {
    deploy_with_runtime(options, None)
}

#[cfg(test)]
fn deploy_with_linux_runtime(
    options: DeployOptions,
    linux_runtime: &[u8],
) -> DeployResult<DeployReport> {
    deploy_with_runtime(options, Some(linux_runtime))
}

fn deploy_with_runtime(
    options: DeployOptions,
    linux_runtime_override: Option<&[u8]>,
) -> DeployResult<DeployReport> {
    let root = options.root.canonicalize()?;
    let surface = options.surface();
    if !options.target.supports_surface(surface) {
        return Err(DeployError::new(format!(
            "deploy target `{}` does not belong to the `{surface}` deploy surface",
            options.target
        )));
    }
    let cloud_session = (options.target == DeployTarget::Dowe)
        .then(cloud::CloudSession::resolve_and_validate)
        .transpose()?;
    if options.environment != DeployEnvironment::Live
        && matches!(
            options.target,
            DeployTarget::Static | DeployTarget::Android | DeployTarget::Ios
        )
    {
        return Err(DeployError::new(format!(
            "deploy target `{}` is only available in the live environment",
            options.target
        )));
    }
    let project = match surface {
        DeploySurface::Server => {
            compile_for_server_environment(&root, options.environment.compile_environment())?
        }
        DeploySurface::Web | DeploySurface::Android | DeploySurface::Ios => {
            compile_for_web_environment(&root, options.environment.compile_environment())?
        }
    };
    let access = DeployAccess::resolve(&project, options.environment)?;
    let environment_values = DeployEnvironmentValues::from_project(&project);
    if matches!(
        surface,
        DeploySurface::Web | DeploySurface::Android | DeploySurface::Ios
    ) && !project.capabilities.views
    {
        return Err(DeployError::new(format!(
            "deploy target `{}` requires `views` in main.dowe",
            options.target
        )));
    }
    if surface == DeploySurface::Server && !project.capabilities.server {
        return Err(DeployError::new(format!(
            "deploy target `{}` requires `server` in main.dowe",
            options.target
        )));
    }
    if options.target == DeployTarget::Ios && !cfg!(target_os = "macos") {
        return Err(DeployError::new(
            "deploy target `ios` is only available on macOS",
        ));
    }
    let output =
        deploy_output_dir_for_surface(&root, options.target, options.environment, surface)?;
    let cloudflare_pages_name = (options.target == DeployTarget::CloudflarePages)
        .then(|| cloudflare::pages_project_name(&project, options.name.as_deref()))
        .transpose()?;
    let vercel_project_name = (options.target == DeployTarget::Vercel)
        .then(|| vercel::project_name(&project, options.name.as_deref()))
        .transpose()?;
    let docker_image = (options.target == DeployTarget::Docker)
        .then(|| {
            docker::resolve_docker_image(
                &root,
                options.registry.as_deref(),
                options.image.as_deref(),
                options.environment,
            )
        })
        .transpose()?;
    let ssh_destination = (options.target == DeployTarget::Ssh && options.publish)
        .then(|| {
            ssh::SshDestination::resolve(
                options.ssh_host.as_deref(),
                options.ssh_user.as_deref(),
                options.ssh_key_file.as_deref(),
            )
        })
        .transpose()?;
    if options.publish && options.target == DeployTarget::Docker {
        return Err(DeployError::new(
            "docker deploy builds and tags a local image; registry push is not configured",
        ));
    }
    let needs_linux_runtime =
        options.target == DeployTarget::Ssh || options.target == DeployTarget::Docker;
    let linux_runtime = if needs_linux_runtime {
        Some(match linux_runtime_override {
            Some(runtime) => runtime.to_vec(),
            None => ssh::prepare_linux_runtime()?,
        })
    } else {
        None
    };
    reset_dir(&output)?;

    let mut artifact = None;
    let mut deployment_id = None;
    let mut url = None;
    let mut ssh_package = None;
    match options.target {
        DeployTarget::Static => package::generate_static(&root, &output)?,
        DeployTarget::Dowe => {
            let cloud_artifact = cloud::generate_artifact(
                &root,
                &output,
                surface,
                options.environment,
                access.is_some(),
            )?;
            artifact = Some(cloud_artifact.path.clone());
            if options.publish && !options.dry_run {
                let session = cloud_session
                    .as_ref()
                    .ok_or_else(|| DeployError::new("Dowe Cloud session is missing"))?;
                let publication = session.publish(&cloud_artifact, surface, options.environment)?;
                deployment_id = Some(publication.deployment_id);
                url = Some(publication.url);
            }
        }
        DeployTarget::Docker => docker::generate_docker(
            &root,
            &output,
            docker_image
                .as_ref()
                .ok_or_else(|| DeployError::new("docker image is missing"))?,
            options.environment,
            access.as_ref(),
            surface,
            project.backend.port,
            project.backend.tls.as_ref().and_then(|tls| tls.http_port),
            &environment_values.client,
            &environment_values.server_names,
            linux_runtime.as_deref(),
        )?,
        DeployTarget::Ssh => {
            let runtime = linux_runtime
                .as_deref()
                .ok_or_else(|| DeployError::new("SSH runtime is missing"))?;
            let package = ssh::generate_ssh(
                &root,
                &output,
                options.environment,
                access.as_ref(),
                &environment_values.client,
                &environment_values.server,
                runtime,
            )?;
            artifact = Some(package.executable.clone());
            ssh_package = Some(package);
        }
        DeployTarget::Cloudflare => cloudflare::generate_cloudflare(
            &project,
            &output,
            options.name.as_deref(),
            options.environment,
            access.as_ref(),
            &environment_values.client,
            &environment_values.server_names,
        )?,
        DeployTarget::CloudflarePages => {
            let project_name = cloudflare_pages_name
                .as_deref()
                .ok_or_else(|| DeployError::new("cloudflare pages project name is missing"))?;
            package::generate_cloudflare_pages(
                &root,
                &output,
                project_name,
                options.environment,
                access.as_ref(),
            )?;
        }
        DeployTarget::Vercel => {
            let project_name = vercel_project_name
                .as_deref()
                .ok_or_else(|| DeployError::new("vercel project name is missing"))?;
            vercel::generate_vercel(
                &project,
                &output,
                project_name,
                options.environment,
                access.as_ref(),
                surface,
                &environment_values.server_names,
            )?;
        }
        DeployTarget::Android => {
            artifact =
                Some(native::android_store_bundle(&project, &output, options.dry_run)?.artifact);
        }
        DeployTarget::Ios => {
            artifact = Some(
                native::build_store(&project, BuildTarget::Ios, &output, options.dry_run)?.artifact,
            );
        }
    }
    database::write_database_artifacts(&project, &output, surface)?;

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
                    options.environment,
                    options.dry_run,
                )?);
            }
            DeployTarget::Vercel => {
                let project_name = vercel_project_name
                    .as_deref()
                    .ok_or_else(|| DeployError::new("vercel project name is missing"))?;
                command = Some(publish::publish_vercel(
                    &output,
                    project_name,
                    options.environment,
                    options.dry_run,
                )?);
            }
            DeployTarget::Static => {
                return Err(DeployError::new(
                    "static deploy generates a dist package and does not publish",
                ));
            }
            DeployTarget::Dowe => {}
            DeployTarget::Ssh => {
                let package = ssh_package
                    .as_ref()
                    .ok_or_else(|| DeployError::new("SSH package is missing"))?;
                let destination = ssh_destination
                    .as_ref()
                    .ok_or_else(|| DeployError::new("SSH destination is missing"))?;
                command = Some(ssh::publish_ssh(package, destination, options.dry_run)?);
            }
            DeployTarget::Docker => {
                unreachable!("docker publish is rejected before package generation");
            }
            DeployTarget::Android => {
                let artifact = artifact
                    .as_deref()
                    .ok_or_else(|| DeployError::new("Android store artifact is missing"))?;
                command = Some(publish::publish_android(
                    artifact,
                    &project.app_config.bundle,
                    options.track.as_deref().unwrap_or("internal"),
                    options.dry_run,
                )?);
            }
            DeployTarget::Ios => {
                let artifact = artifact
                    .as_deref()
                    .ok_or_else(|| DeployError::new("iOS store artifact is missing"))?;
                command = Some(publish::publish_ios(artifact, options.dry_run)?);
            }
        }
    }

    Ok(DeployReport {
        environment: options.environment,
        target: options.target,
        output_dir: output.clone(),
        files: collect_files(&output)?,
        command,
        published: options.publish && !options.dry_run,
        image_ref: docker_image.map(|image| image.reference),
        image_built,
        artifact,
        access_protected: access.is_some(),
        deployment_id,
        url,
    })
}

pub fn deploy_output_dir(
    root: impl AsRef<Path>,
    target: DeployTarget,
    environment: DeployEnvironment,
) -> DeployResult<std::path::PathBuf> {
    deploy_output_dir_for_surface(root, target, environment, target.surface())
}

fn deploy_output_dir_for_surface(
    root: impl AsRef<Path>,
    target: DeployTarget,
    environment: DeployEnvironment,
    surface: DeploySurface,
) -> DeployResult<std::path::PathBuf> {
    let root = root.as_ref();
    if environment != DeployEnvironment::Live {
        let base = root.join(".dowe/dist").join(environment.as_str());
        return Ok(if surface == DeploySurface::Web {
            base.join("web").join(target.as_str())
        } else {
            base.join(target.as_str())
        });
    }
    match surface {
        DeploySurface::Web => {
            if matches!(
                target,
                DeployTarget::CloudflarePages | DeployTarget::Docker | DeployTarget::Vercel
            ) {
                web_target_dir(root, target.as_str())
            } else {
                target_dir(root, target.as_str())
            }
        }
        DeploySurface::Server | DeploySurface::Android | DeploySurface::Ios => {
            target_dir(root, target.as_str())
        }
    }
}

#[cfg(test)]
mod tests;
