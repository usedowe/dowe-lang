fn copy_relative_files(source: &Path, target: &Path, files: &[String]) -> RuntimeResult<()> {
    for relative in files {
        validate_relative_path(Path::new(relative), "Android cache file")?;
        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source.join(relative), destination)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if fs::hard_link(entry.path(), &destination).is_err() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn copy_cache_entry(source: &Path, target: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "manifest.json" || name == "last-used" {
            continue;
        }
        let destination = target.join(&name);
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if fs::hard_link(entry.path(), &destination).is_err() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn copy_tree_excluding(
    source: &Path,
    target: &Path,
    excluded: &BTreeSet<String>,
) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for relative in collect_relative_files(source, "class")? {
        let relative_string = relative.to_string_lossy().to_string();
        if excluded.contains(&relative_string) {
            continue;
        }
        let destination = target.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::hard_link(source.join(&relative), &destination).is_err() {
            fs::copy(source.join(&relative), destination)?;
        }
    }
    Ok(())
}

fn collect_relative_files(root: &Path, extension: &str) -> RuntimeResult<Vec<PathBuf>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_relative_files_inner(root, root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_relative_files_inner(
    root: &Path,
    current: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> RuntimeResult<()> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_relative_files_inner(root, &path, extension, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(
                path.strip_prefix(root)
                    .expect("collected Android cache file")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

fn update_path_digest(hash: &mut Sha256, path: &Path) -> RuntimeResult<()> {
    let canonical = path.canonicalize()?;
    update_digest(hash, canonical.to_string_lossy().as_bytes());
    if canonical.is_dir() {
        for relative in collect_relative_files(&canonical, "class")? {
            update_digest(hash, relative.to_string_lossy().as_bytes());
            update_digest(hash, &fs::read(canonical.join(relative))?);
        }
    } else {
        update_digest(hash, &fs::read(canonical)?);
    }
    Ok(())
}

fn digest_bytes(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn update_digest(hash: &mut Sha256, value: &[u8]) {
    hash.update(value.len().to_le_bytes());
    hash.update(value);
}
