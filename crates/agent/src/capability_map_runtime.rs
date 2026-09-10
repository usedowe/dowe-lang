use crate::{AgentError, AgentResult};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

const MAX_PATHS: usize = 64;
const MAX_CAPABILITIES: usize = 64;
const MAX_INDEX_BYTES: usize = 64 * 1024;
const MAX_PAGE_BYTES: usize = 32 * 1024;
const FINGERPRINT_FILE: &str = ".fingerprints.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMapUpdate {
    pub changed: Vec<String>,
    pub skipped: Vec<String>,
}

/// Update the project-local capability pages for confirmed source effects.
///
/// Paths must be project-relative. Paths outside the project, generated
/// CodeGraph files, and the map's own bookkeeping files are ignored. Existing
/// pages are preserved and only their generated source list is refreshed.
pub(crate) fn sync_capability_map(
    root: impl AsRef<Path>,
    paths: &[String],
) -> AgentResult<CapabilityMapUpdate> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(AgentError::new("capability map root is not a directory"));
    }

    let mut normalized = BTreeSet::new();
    let mut skipped = Vec::new();
    for value in paths.iter().take(MAX_PATHS) {
        if let Some(path) = safe_relative(value) {
            if should_index(&path) {
                normalized.insert(path);
            } else {
                skipped.push(value.clone());
            }
        } else {
            skipped.push(value.clone());
        }
    }
    if normalized.is_empty() {
        return Ok(CapabilityMapUpdate {
            changed: Vec::new(),
            skipped,
        });
    }

    let capabilities = root.join(".agents").join("capabilities");
    ensure_directory(&root.join(".agents"))?;
    ensure_directory(&capabilities)?;

    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for path in normalized {
        grouped.entry(capability_id(&path)).or_default().push(path);
    }

    let mut existing_ids = discover_ids(&capabilities)?;
    let new_count = grouped
        .keys()
        .filter(|id| !existing_ids.contains(*id))
        .count();
    if existing_ids.len().saturating_add(new_count) > MAX_CAPABILITIES {
        return Err(AgentError::new("capability map has reached its page limit"));
    }
    let mut changed = Vec::new();
    for (id, mut paths) in grouped {
        paths.sort();
        let page = capabilities.join(format!("{id}.md"));
        let old = read_regular(&page)?;
        let body = render_page(&id, &paths, old.as_deref());
        if body.len() > MAX_PAGE_BYTES {
            return Err(AgentError::new("capability page exceeds its size limit"));
        }
        if old.as_deref() != Some(body.as_str()) {
            atomic_write(&page, body.as_bytes())?;
            changed.push(format!(".agents/capabilities/{id}.md"));
        }
        existing_ids.insert(id);
    }

    let index = render_index(&existing_ids);
    if index.len() > MAX_INDEX_BYTES {
        return Err(AgentError::new(
            "capability map index exceeds its size limit",
        ));
    }
    let index_path = capabilities.join("index.md");
    if read_regular(&index_path)?.as_deref() != Some(index.as_str()) {
        atomic_write(&index_path, index.as_bytes())?;
        changed.push(".agents/capabilities/index.md".into());
    }

    let fingerprints = collect_fingerprints(root, &capabilities, &existing_ids)?;
    let fingerprint_bytes = serde_json::to_vec(&fingerprints)
        .map_err(|error| AgentError::new(error.to_string()))?;
    if fingerprint_bytes.len() > MAX_INDEX_BYTES {
        return Err(AgentError::new(
            "capability map fingerprints exceed its size limit",
        ));
    }
    let fingerprint_path = capabilities.join(FINGERPRINT_FILE);
    let old_fingerprints = read_regular(&fingerprint_path)?;
    let new_fingerprints = String::from_utf8_lossy(&fingerprint_bytes);
    if old_fingerprints.as_deref() != Some(new_fingerprints.as_ref()) {
        atomic_write(&fingerprint_path, &fingerprint_bytes)?;
        changed.push(format!(".agents/capabilities/{FINGERPRINT_FILE}"));
    }

    Ok(CapabilityMapUpdate { changed, skipped })
}

/// Detect source files changed outside the native agent and mark their pages
/// pending. Only paths already recorded by the map are inspected.
pub(crate) fn refresh_capability_map(root: impl AsRef<Path>) -> AgentResult<Vec<String>> {
    let root = root.as_ref();
    let capabilities = root.join(".agents").join("capabilities");
    if !capabilities.is_dir() {
        return Ok(Vec::new());
    }
    let fingerprint_file = capabilities.join(FINGERPRINT_FILE);
    let Some(raw) = read_regular(&fingerprint_file)? else {
        return Ok(Vec::new());
    };
    let previous: BTreeMap<String, String> = serde_json::from_str(&raw)
        .map_err(|error| AgentError::new(error.to_string()))?;
    if previous.len() > MAX_PATHS * MAX_CAPABILITIES {
        return Err(AgentError::new(
            "capability map fingerprints exceed its entry limit",
        ));
    }
    let mut current = previous.clone();
    let mut stale = Vec::new();
    for (path, expected) in previous {
        let actual = fingerprint_path(root, &path);
        if actual != expected {
            stale.push(path.clone());
            current.insert(path.clone(), actual);
            let page = capabilities.join(format!("{}.md", capability_id(&path)));
            if let Some(old) = read_regular(&page)? {
                let pending = mark_pending(&old);
                if pending != old {
                    atomic_write(&page, pending.as_bytes())?;
                }
            }
        }
    }
    if !stale.is_empty() {
        let bytes = serde_json::to_vec(&current)
            .map_err(|error| AgentError::new(error.to_string()))?;
        atomic_write(&fingerprint_file, &bytes)?;
    }
    Ok(stale)
}

/// Seed an absent map from a small local inventory. This is intentionally
/// deterministic and bounded; it creates a useful Doc-as-Code index without
/// sending the repository to a model or rewriting an existing map.
pub(crate) fn bootstrap_capability_map(root: impl AsRef<Path>) -> AgentResult<CapabilityMapUpdate> {
    let root = root.as_ref();
    let index = root.join(".agents/capabilities/index.md");
    if index.is_file() {
        return Ok(CapabilityMapUpdate { changed: Vec::new(), skipped: Vec::new() });
    }
    let mut paths = Vec::new();
    collect_seed_paths(root, root, &mut paths, 64);
    sync_capability_map(root, &paths)
}

fn safe_relative(value: &str) -> Option<String> {
    let path = Path::new(value);
    if path.is_absolute() || value.trim().is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn should_index(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if name.starts_with(".env")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".p12")
        || path.split('/').any(|part| matches!(part, "secrets" | "credentials"))
    {
        return false;
    }
    if path.starts_with(".dowe/")
        || path == ".dowe"
        || path == ".agents"
        || path == ".agents/capabilities"
        || path == ".agents/capabilities/.fingerprints.json"
        || path.starts_with(".agents/capabilities/")
        || matches!(path, ".agents/architecture.md" | ".agents/workflows.md")
    {
        return false;
    }
    !path.ends_with('/')
}

fn collect_seed_paths(root: &Path, directory: &Path, paths: &mut Vec<String>, limit: usize) {
    if paths.len() >= limit {
        return;
    }
    let Ok(mut entries) = fs::read_dir(directory).map(|entries| entries.flatten().collect::<Vec<_>>()) else {
        return;
    };
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        if paths.len() >= limit {
            break;
        }
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink()
            || matches!(name.as_str(), ".git" | "target" | "node_modules" | ".DS_Store")
            || relative.starts_with(".dowe")
            || relative.starts_with(".agents/capabilities")
        {
            continue;
        }
        if metadata.is_dir() {
            if name.starts_with('.') && name != ".agents" {
                continue;
            }
            collect_seed_paths(root, &path, paths, limit);
            continue;
        }
        let relative = relative.to_string_lossy().replace('\\', "/");
        let supported = matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "dowe" | "md" | "toml" | "yaml" | "yml" | "json" | "js" | "jsx" | "ts" | "tsx" | "py" | "go" | "java" | "kt" | "swift" | "css" | "html")
        );
        if supported && should_index(&relative) {
            paths.push(relative);
        }
    }
}

fn capability_id(path: &str) -> String {
    let domain = path.split('/').next().unwrap_or("project");
    let mut id = domain
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    if id.is_empty() {
        id = "project".into();
    }
    if id.len() > 40 {
        id.truncate(40);
    }
    id
}

fn render_page(id: &str, paths: &[String], old: Option<&str>) -> String {
    let title = id.replace(['-', '_'], " ");
    let status = old.map(managed_status).unwrap_or_else(|| "proposed".into());
    let mut sources = old.map(managed_sources).unwrap_or_default();
    sources.extend(paths.iter().cloned());
    sources.sort();
    sources.dedup();
    let mut body = format!(
        "# {title}\n\n<!-- Generated by dowe agent from confirmed file effects. Edit the prose outside the managed block. -->\n\n## Status\n\n{status}\n\n## Sources\n\n<!-- dowe-agent:sources:start -->\n"
    );
    for path in sources {
        body.push_str("- `");
        body.push_str(&path.replace('`', "'"));
        body.push_str("`\n");
    }
    body.push_str("<!-- dowe-agent:sources:end -->\n\n## Contract\n\nDescribe the inputs, outputs, validation, and known limitations for this capability.\n");
    if let Some(old) = old {
        if let Some(contract) = old.split("## Contract\n\n").nth(1) {
            let contract = contract
                .split("\n<!-- dowe-agent:managed-end -->")
                .next()
                .unwrap_or(contract)
                .trim();
            if !contract.is_empty() && !contract.starts_with("Describe the inputs") {
                body.push_str(contract);
                body.push('\n');
            }
        }
    }
    body.push_str("\n<!-- dowe-agent:managed-end -->\n");
    body
}

fn managed_status(old: &str) -> String {
    old.split("## Status\n\n")
        .nth(1)
        .and_then(|value| value.lines().next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(64).collect())
        .unwrap_or_else(|| "proposed".into())
}

fn managed_sources(old: &str) -> Vec<String> {
    let Some(block) = old
        .split("<!-- dowe-agent:sources:start -->\n")
        .nth(1)
        .and_then(|value| value.split("<!-- dowe-agent:sources:end -->").next())
    else {
        return Vec::new();
    };
    block
        .lines()
        .filter_map(|line| line.strip_prefix("- `")?.strip_suffix('`'))
        .map(str::to_owned)
        .collect()
}

fn render_index(ids: &BTreeSet<String>) -> String {
    let mut index = String::from(
        "# Capability Map\n\nThis index is maintained by the Dowe agent from confirmed changes. Pages are advisory documentation; specs, source, and tests remain authoritative.\n\n",
    );
    for id in ids {
        index.push_str("- [");
        index.push_str(&id.replace(['-', '_'], " "));
        index.push_str("](./");
        index.push_str(id);
        index.push_str(".md)\n");
    }
    index
}

fn collect_fingerprints(
    root: &Path,
    capabilities: &Path,
    ids: &BTreeSet<String>,
) -> AgentResult<BTreeMap<String, String>> {
    let mut fingerprints = BTreeMap::new();
    for id in ids.iter().take(MAX_CAPABILITIES) {
        let page = capabilities.join(format!("{id}.md"));
        let Some(content) = read_regular(&page)? else {
            continue;
        };
        for path in managed_sources(&content).into_iter().take(MAX_PATHS) {
            fingerprints.insert(path.clone(), fingerprint_path(root, &path));
        }
    }
    Ok(fingerprints)
}

fn fingerprint_path(root: &Path, relative: &str) -> String {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return "<invalid>".into();
    }
    let full = root.join(path);
    let Ok(metadata) = fs::symlink_metadata(&full) else {
        return "<missing>".into();
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return "<unavailable>".into();
    }
    let Ok(bytes) = fs::read(&full) else {
        return "<unavailable>".into();
    };
    let mut hasher = Sha256::new();
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(&bytes[..bytes.len().min(1024 * 1024)]);
    format!(
        "sha256:{}",
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn mark_pending(old: &str) -> String {
    let Some(status) = old
        .split("## Status\n\n")
        .nth(1)
        .and_then(|value| value.lines().next())
    else {
        return old.to_owned();
    };
    if status == "pending" {
        return old.to_owned();
    }
    old.replacen(
        &format!("## Status\n\n{status}"),
        "## Status\n\npending",
        1,
    )
}

fn discover_ids(dir: &Path) -> AgentResult<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(dir).map_err(|error| AgentError::new(error.to_string()))? {
        let entry = entry.map_err(|error| AgentError::new(error.to_string()))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| AgentError::new(error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(AgentError::new("capability map cannot contain symlinks"));
        }
        if metadata.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && path.file_stem().and_then(|stem| stem.to_str()) != Some("index")
        {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                ids.insert(stem.into());
            }
        }
    }
    Ok(ids)
}

fn ensure_directory(path: &Path) -> AgentResult<()> {
    if path.exists() {
        let metadata =
            fs::symlink_metadata(path).map_err(|error| AgentError::new(error.to_string()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AgentError::new(
                "capability map path must be a regular directory",
            ));
        }
        return Ok(());
    }
    fs::create_dir(path).map_err(|error| AgentError::new(error.to_string()))
}

fn read_regular(path: &Path) -> AgentResult<Option<String>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(AgentError::new(error.to_string())),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AgentError::new(
            "capability map file must be regular and non-symlink",
        ));
    }
    let bytes = fs::read(path).map_err(|error| AgentError::new(error.to_string()))?;
    if bytes.len() > MAX_PAGE_BYTES {
        return Err(AgentError::new(
            "capability map file exceeds its size limit",
        ));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| AgentError::new("capability map file must be UTF-8"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> AgentResult<()> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let suffix = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let temporary = path.with_extension(format!("md.{suffix}.tmp"));
    fs::write(&temporary, bytes).map_err(|error| AgentError::new(error.to_string()))?;
    fs::rename(&temporary, path).map_err(|error| AgentError::new(error.to_string()))
}
