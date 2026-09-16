use super::*;

pub(super) fn store_dir(root: &Path) -> CodeGraphResult<PathBuf> {
    let dir = root.join(STORE);
    reject_path(root, &dir)?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
pub(super) fn reject_path(root: &Path, path: &Path) -> CodeGraphResult<()> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| CodeGraphError::new("CodeGraph path escapes project root"))?;
    if rel.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(CodeGraphError::new("CodeGraph path traversal rejected"));
    }
    let mut p = root.to_path_buf();
    for c in rel.components() {
        p.push(c);
        if fs::symlink_metadata(&p).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(CodeGraphError::at_path(
                &p,
                "CodeGraph path must not contain symlinks",
            ));
        }
    }
    Ok(())
}
pub(super) fn read_current(
    dir: &Path,
    root: &Path,
    mode: CodeGraphMode,
) -> CodeGraphResult<CodeGraphSnapshot> {
    let name = String::from_utf8(read_bounded(root, &dir.join("CURRENT"), 256)?)
        .map_err(|_| CodeGraphError::new("invalid CodeGraph pointer encoding"))?;
    if !name.trim().starts_with("generation-")
        || !name
            .trim()
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(CodeGraphError::new("invalid CodeGraph generation name"));
    }
    let generation = dir.join(name.trim());
    reject_path(root, &generation)?;
    let manifest: GraphManifest = serde_json::from_slice(&read_bounded(
        root,
        &generation.join("manifest.json"),
        16 * 1024 * 1024,
    )?)?;
    if manifest.version != VERSION
        || manifest.schema != VERSION
        || manifest.root != root.to_string_lossy()
        || manifest.mode != mode
    {
        return Err(CodeGraphError::new("CodeGraph generation is incompatible"));
    }
    let nodes = serde_json::from_slice(&read_bounded(
        root,
        &generation.join("nodes.json"),
        64 * 1024 * 1024,
    )?)?;
    let edges = serde_json::from_slice(&read_bounded(
        root,
        &generation.join("edges.json"),
        64 * 1024 * 1024,
    )?)?;
    Ok(CodeGraphSnapshot {
        graph: CodeGraph {
            mode,
            root: ".".into(),
            nodes,
            edges,
        },
        clean_graph: read_bounded(root, &generation.join("clean.json"), 64 * 1024 * 1024)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
        manifest,
        freshness: GraphFreshness::Stale,
        changed: vec![],
        deleted: vec![],
        error: None,
        generation: Some(name.trim().to_string()),
    })
}
pub(super) fn write_generation(
    dir: &Path,
    manifest: &GraphManifest,
    graph: &CodeGraph,
    clean_graph: &crate::clean::CleanGraph,
) -> CodeGraphResult<String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| CodeGraphError::new("system clock precedes Unix epoch"))?
        .as_nanos();
    let name = format!(
        "generation-{}-{}-{nonce}",
        manifest.revision,
        std::process::id()
    );
    let tmp = dir.join(format!(".{name}.tmp"));
    let generation = dir.join(&name);
    fs::create_dir(&tmp)?;
    for (file, value) in [
        ("manifest.json", serde_json::to_vec_pretty(manifest)?),
        ("nodes.json", serde_json::to_vec_pretty(&graph.nodes)?),
        ("edges.json", serde_json::to_vec_pretty(&graph.edges)?),
        ("clean.json", serde_json::to_vec_pretty(clean_graph)?),
        (
            "index.json",
            serde_json::to_vec_pretty(
                &graph
                    .nodes
                    .iter()
                    .filter_map(|n| n.path.as_ref().map(|p| (p, n.id.clone())))
                    .collect::<BTreeMap<_, _>>(),
            )?,
        ),
    ] {
        let limit = if file == "manifest.json" { 16 } else { 64 } * 1024 * 1024;
        if value.len() > limit {
            return Err(CodeGraphError::new(
                "CodeGraph generation exceeds its size limit",
            ));
        }
        durable_write(&tmp.join(file), &value)?;
    }
    sync_directory(&tmp)?;
    fs::rename(&tmp, &generation)?;
    sync_directory(dir)?;
    let pointer = dir.join("CURRENT.tmp");
    let root = Path::new(&manifest.root);
    reject_path(root, &pointer)?;
    reject_path(root, &dir.join("CURRENT"))?;
    if pointer.exists() {
        fs::remove_file(&pointer)?;
    }
    durable_write(&pointer, format!("{name}\n").as_bytes())?;
    fs::rename(pointer, dir.join("CURRENT"))?;
    sync_directory(dir)?;
    prune_generations(dir, &name)?;
    Ok(name)
}

fn read_bounded(root: &Path, path: &Path, limit: u64) -> CodeGraphResult<Vec<u8>> {
    use std::io::Read;
    reject_path(root, path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(CodeGraphError::at_path(
            path,
            "CodeGraph file exceeds its size limit or is not regular",
        ));
    }
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(CodeGraphError::at_path(
            path,
            "CodeGraph file exceeds its size limit or is not regular",
        ));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(CodeGraphError::at_path(
            path,
            "CodeGraph file grew beyond its size limit",
        ));
    }
    Ok(bytes)
}

fn durable_write(path: &Path, bytes: &[u8]) -> CodeGraphResult<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> CodeGraphResult<()> {
    #[cfg(unix)]
    fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(super) fn prune_generations(dir: &Path, current: &str) -> CodeGraphResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_generation = name.starts_with("generation-") && name != current;
        let is_temporary =
            (name.starts_with(".generation-") && name.ends_with(".tmp")) || name == "CURRENT.tmp";
        if !is_generation && !is_temporary {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(CodeGraphError::at_path(
                &path,
                "CodeGraph generation must not be a symlink",
            ));
        }
        if metadata.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
