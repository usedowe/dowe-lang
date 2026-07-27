use crate::{IconError, IconResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) struct IconSource {
    pub root: PathBuf,
    pub relative_path: String,
    pub data: Vec<u8>,
    pub fingerprint: String,
}

impl IconSource {
    pub fn load(root: &Path, source: &Path) -> IconResult<Self> {
        let root = root
            .canonicalize()
            .map_err(|error| IconError::at_path(root, error))?;
        if !root.join("main.dowe").is_file() {
            return Err(IconError::at_path(
                &root,
                "dowe icons requires a project root containing main.dowe",
            ));
        }
        validate_relative_source(source)?;
        let path = root.join(source);
        let canonical = path
            .canonicalize()
            .map_err(|error| IconError::at_path(&path, error))?;
        if !canonical.starts_with(&root) || !canonical.is_file() {
            return Err(IconError::at_path(
                &path,
                "icon source must resolve to a regular file inside the project",
            ));
        }
        let data = fs::read(&canonical).map_err(|error| IconError::at_path(&canonical, error))?;
        if data.is_empty() {
            return Err(IconError::at_path(&canonical, "icon SVG is empty"));
        }
        let source_text = std::str::from_utf8(&data).map_err(|error| {
            IconError::at_path(&canonical, format!("invalid icon SVG: {error}"))
        })?;
        if has_external_reference(source_text)? {
            return Err(IconError::at_path(
                &canonical,
                "icon SVG must be self-contained and cannot reference external resources",
            ));
        }
        let relative = canonical
            .strip_prefix(&root)
            .map_err(|_| IconError::new("icon source must stay inside the project"))?;
        let relative_path = relative.to_string_lossy().replace('\\', "/");
        let fingerprint = Sha256::digest(&data)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(Self {
            root,
            relative_path,
            data,
            fingerprint,
        })
    }
}

fn has_external_reference(source: &str) -> IconResult<bool> {
    let parsing_options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let document = roxmltree::Document::parse_with_options(source, parsing_options)
        .map_err(|error| IconError::new(format!("invalid icon SVG: {error}")))?;
    if document
        .descendants()
        .flat_map(|node| node.attributes())
        .any(|attribute| attribute.name() == "href" && !is_embedded_reference(attribute.value()))
    {
        return Ok(true);
    }
    let lower = source.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(index) = rest.find("url(") {
        let after_open = &rest[index + 4..];
        let Some(end) = after_open.find(')') else {
            break;
        };
        let value = after_open[..end].trim().trim_matches(['\'', '"']);
        if !is_embedded_reference(value) {
            return Ok(true);
        }
        rest = &after_open[end + 1..];
    }
    Ok(false)
}

fn is_embedded_reference(value: &str) -> bool {
    let value = value.trim();
    value.starts_with('#') || value.to_ascii_lowercase().starts_with("data:")
}

fn validate_relative_source(source: &Path) -> IconResult<()> {
    if source.is_absolute()
        || source.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(IconError::new(
            "icon source must be a project-relative path without `..`",
        ));
    }
    let components = source
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let allowed =
        components.len() == 1 || components.first().is_some_and(|value| value == "assets");
    if !allowed {
        return Err(IconError::new(
            "icon source must be in the project root or under assets",
        ));
    }
    if components.first().is_some_and(|value| value == "assets")
        && components.get(1).is_some_and(|value| value == "icons")
    {
        return Err(IconError::new(
            "icon source cannot be stored inside generated assets/icons",
        ));
    }
    if source
        .extension()
        .and_then(|value| value.to_str())
        .is_none_or(|value| !value.eq_ignore_ascii_case("svg"))
    {
        return Err(IconError::new("icon source must be an SVG file"));
    }
    Ok(())
}
