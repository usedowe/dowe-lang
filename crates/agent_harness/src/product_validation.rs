use super::{ProductDiscovery, VerifiedProductUpdate};
use crate::{HarnessError, HarnessResult};
use std::path::Path;

pub(super) fn validate_update(update: &VerifiedProductUpdate) -> HarnessResult<()> {
    for value in [&update.workflow_id, &update.objective] {
        if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
            return Err(HarnessError::new(
                "verified product update contains invalid text",
            ));
        }
    }
    for path in update
        .specification_path
        .iter()
        .chain(update.contract_paths.iter())
        .chain(update.changed_files.iter())
    {
        if path.is_empty()
            || path.len() > 512
            || Path::new(path).is_absolute()
            || path.split('/').any(|part| part == ".." || part.is_empty())
        {
            return Err(HarnessError::new(
                "product evidence path must be project-relative",
            ));
        }
    }
    if update.task_ids.len() + update.check_names.len() > 512 {
        return Err(HarnessError::new("too much product evidence"));
    }
    if update.use_cases.len() > 128 {
        return Err(HarnessError::new("too many verified product use cases"));
    }
    for use_case in &update.use_cases {
        for value in [&use_case.id, &use_case.name]
            .into_iter()
            .chain(use_case.actor.iter())
            .chain(use_case.goal.iter())
        {
            if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control)
            {
                return Err(HarnessError::new(
                    "verified product use case contains invalid text",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_discovery(discovery: &ProductDiscovery) -> HarnessResult<()> {
    for value in std::iter::once(&discovery.product_name)
        .chain(discovery.category.iter())
        .chain(discovery.observed_concepts.iter())
        .chain(discovery.inferred_capabilities.iter())
        .chain(discovery.source_images.iter())
    {
        if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
            return Err(HarnessError::new("product discovery contains invalid text"));
        }
    }
    if discovery.observed_concepts.len()
        + discovery.inferred_capabilities.len()
        + discovery.observed_use_cases.len()
        + discovery.inferred_use_cases.len()
        + discovery.source_images.len()
        > 256
    {
        return Err(HarnessError::new(
            "product discovery contains too many concepts",
        ));
    }
    for use_case in discovery
        .observed_use_cases
        .iter()
        .chain(discovery.inferred_use_cases.iter())
    {
        for value in [&use_case.id, &use_case.name]
            .into_iter()
            .chain(use_case.actor.iter())
            .chain(use_case.goal.iter())
        {
            if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control)
            {
                return Err(HarnessError::new(
                    "product discovery use case contains invalid text",
                ));
            }
        }
    }
    for image in &discovery.source_images {
        if Path::new(image).is_absolute()
            || image.split('/').any(|part| part == ".." || part.is_empty())
        {
            return Err(HarnessError::new(
                "product discovery image must be project-relative",
            ));
        }
    }
    Ok(())
}

pub(super) fn reject_symlinked_product_store(root: &Path, directory: &Path) -> HarnessResult<()> {
    let metadata = std::fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(HarnessError::new(
            "product project root must be a real directory",
        ));
    }
    if directory.exists() {
        let metadata = std::fs::symlink_metadata(directory)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(HarnessError::new(".agent must be a real directory"));
        }
    }
    for name in ["knowledge.json", "product.md"] {
        let path = directory.join(name);
        if path.exists() && std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(HarnessError::new(format!(
                "product store file is a symlink: {name}"
            )));
        }
    }
    Ok(())
}

pub(super) fn slug(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output
        .trim_matches('-')
        .chars()
        .take(120)
        .collect::<String>()
}
