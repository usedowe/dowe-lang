mod artifact;
mod error;
mod manifest;
mod mobile;
mod model;
mod render;
mod source;
mod web_desktop;

pub use error::{IconError, IconResult};
pub use model::{GenerateIconOptions, IconReport, IconRounded, IconTarget};

use artifact::IconArtifact;
use manifest::IconManifest;
use mobile::{android_artifacts, ios_artifacts};
use model::IconColor;
use render::IconRenderer;
use source::IconSource;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use web_desktop::{desktop_artifacts, web_artifacts};

pub fn generate_project_icons(options: GenerateIconOptions) -> IconResult<IconReport> {
    let source = IconSource::load(&options.root, &options.source)?;
    let background = IconColor::parse(&options.background)?;
    let targets = normalize_targets(options.targets)?;
    let renderer = IconRenderer::new(&source.data)?;
    let mut generated = Vec::new();
    for target in &targets {
        let artifacts = match target {
            IconTarget::Web => web_artifacts(&renderer, background, options.rounded)?,
            IconTarget::Desktop => desktop_artifacts(&renderer, background, options.rounded)?,
            IconTarget::Ios => ios_artifacts(&renderer, background)?,
            IconTarget::Android => android_artifacts(&renderer, background, options.rounded)?,
        };
        generated.push((*target, artifacts));
    }
    let output_dir = source.root.join("icons");
    validate_output_root(&output_dir, &targets)?;
    let manifest_path = output_dir.join("manifest.json");
    let mut manifest = IconManifest::read(&manifest_path)?;
    let mut files = Vec::new();
    for (target, artifacts) in &generated {
        replace_target_directory(&output_dir, *target, artifacts)?;
        manifest.update(
            *target,
            &source,
            background.hex(),
            options.rounded,
            artifacts,
        );
        files.extend(artifacts.iter().map(|artifact| {
            PathBuf::from("icons")
                .join(target.as_str())
                .join(&artifact.relative_path)
        }));
    }
    write_manifest(&output_dir, &manifest_path, manifest.bytes()?)?;
    files.push(PathBuf::from("icons/manifest.json"));
    files.sort();
    Ok(IconReport {
        output_dir,
        manifest: manifest_path,
        targets,
        files,
    })
}

fn normalize_targets(targets: Vec<IconTarget>) -> IconResult<Vec<IconTarget>> {
    let selected = targets.into_iter().collect::<BTreeSet<_>>();
    if selected.is_empty() {
        return Err(IconError::new("select at least one icon target"));
    }
    Ok(IconTarget::ALL
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect())
}

fn validate_output_root(output_dir: &Path, targets: &[IconTarget]) -> IconResult<()> {
    let root = output_dir
        .parent()
        .ok_or_else(|| IconError::new("icon output root has no project parent"))?;
    validate_output_directory(root, "project root must be a directory")?;
    validate_output_directory(output_dir, "icon output root must be a directory")?;
    validate_output_file(&output_dir.join("manifest.json"))?;
    for target in targets {
        let path = output_dir.join(target.as_str());
        validate_output_directory(&path, "icon target output must be a directory")?;
    }
    Ok(())
}

fn validate_output_directory(path: &Path, collision_message: &str) -> IconResult<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        return Err(IconError::at_path(path, "icon output cannot use symlinks"));
    }
    if !metadata.is_dir() {
        return Err(IconError::at_path(path, collision_message));
    }
    Ok(())
}

fn validate_output_file(path: &Path) -> IconResult<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        return Err(IconError::at_path(path, "icon output cannot use symlinks"));
    }
    if !metadata.is_file() {
        return Err(IconError::at_path(
            path,
            "icon manifest output must be a file",
        ));
    }
    Ok(())
}

fn replace_target_directory(
    output_dir: &Path,
    target: IconTarget,
    artifacts: &[IconArtifact],
) -> IconResult<()> {
    fs::create_dir_all(output_dir)?;
    let suffix = std::process::id();
    let staging = output_dir.join(format!(".{}-{suffix}-next", target.as_str()));
    let backup = output_dir.join(format!(".{}-{suffix}-previous", target.as_str()));
    remove_path(&staging)?;
    remove_path(&backup)?;
    fs::create_dir(&staging)?;
    for artifact in artifacts {
        validate_artifact_path(&artifact.relative_path)?;
        let path = staging.join(&artifact.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &artifact.content)?;
    }
    let destination = output_dir.join(target.as_str());
    if destination.is_dir() {
        fs::rename(&destination, &backup)?;
    }
    if let Err(error) = fs::rename(&staging, &destination) {
        if backup.is_dir() {
            let _ = fs::rename(&backup, &destination);
        }
        return Err(IconError::at_path(&destination, error));
    }
    remove_path(&backup)?;
    Ok(())
}

fn validate_artifact_path(path: &Path) -> IconResult<()> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(IconError::new(format!(
            "invalid generated icon path {}",
            path.display()
        )));
    }
    Ok(())
}

fn write_manifest(output_dir: &Path, manifest_path: &Path, bytes: Vec<u8>) -> IconResult<()> {
    fs::create_dir_all(output_dir)?;
    let temporary = output_dir.join(format!(".manifest-{}.json", std::process::id()));
    fs::write(&temporary, bytes)?;
    if manifest_path.is_file() {
        fs::remove_file(manifest_path)?;
    }
    fs::rename(&temporary, manifest_path)?;
    Ok(())
}

fn remove_path(path: &Path) -> IconResult<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
