pub(super) async fn collect_studio_sources(
    root: &Path,
    directory: &Path,
    output: &mut Vec<StudioSourceEntry>,
) -> RuntimeResult<()> {
    let mut pending = vec![directory.to_path_buf()];
    let mut directory_count = 0usize;
    while let Some(directory) = pending.pop() {
        directory_count += 1;
        if directory_count > STUDIO_CONTEXT_MAX_DIRECTORIES {
            return Err(RuntimeError::new(
                "Studio context directory count exceeds its limit",
            ));
        }
        let mut directory_entries = tokio::fs::read_dir(&directory).await.map_err(|error| {
            RuntimeError::new(format!("could not inspect project folder: {error}"))
        })?;
        let mut entries = Vec::new();
        while let Some(entry) = directory_entries.next_entry().await.map_err(|error| {
            RuntimeError::new(format!("could not inspect project folder: {error}"))
        })? {
            entries.push(entry);
        }
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || matches!(name.as_str(), "AGENTS.md" | "CLAUDE.md") {
                continue;
            }
            let path = entry.path();
            let metadata = tokio::fs::symlink_metadata(&path).await.map_err(|error| {
                RuntimeError::new(format!("could not inspect project file: {error}"))
            })?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                if !should_skip_project_directory(&name) {
                    pending.push(path);
                }
                continue;
            }
            if !metadata.is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("dowe")
            {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|error| {
                    RuntimeError::new(format!("could not inspect project path: {error}"))
                })?
                .to_string_lossy()
                .replace('\\', "/");
            if visible_studio_path(&relative) {
                output.push(StudioSourceEntry {
                    relative,
                    path,
                    size: metadata.len(),
                });
                if output.len() > STUDIO_CONTEXT_MAX_SOURCE_FILES {
                    return Err(RuntimeError::new(
                        "Studio context source file count exceeds its limit",
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(super) async fn read_studio_source(
    entry: &StudioSourceEntry,
) -> RuntimeResult<StudioSourceContent> {
    if entry.size > STUDIO_CONTEXT_MAX_SOURCE_READ_BYTES as u64 {
        return Err(RuntimeError::new(
            "Studio context source file exceeds the 2 MiB limit",
        ));
    }
    let bytes = tokio::fs::read(&entry.path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project source: {error}")))?;
    let digest = hex_digest(&Sha256::digest(&bytes));
    let content = String::from_utf8(bytes)
        .map_err(|_| RuntimeError::new("Studio context source must be UTF-8"))?;
    Ok(StudioSourceContent { content, digest })
}

pub(super) fn validate_studio_image(image: &str) -> RuntimeResult<()> {
    if image.is_empty() {
        return Ok(());
    }
    let encoded = [
        "data:image/png;base64,",
        "data:image/jpeg;base64,",
        "data:image/webp;base64,",
    ]
    .iter()
    .find_map(|prefix| image.strip_prefix(prefix));
    let Some(encoded) = encoded else {
        return Err(RuntimeError::new(
            "Studio context image must be a bounded PNG, JPEG, or WebP data URL",
        ));
    };
    if image.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new(
            "Studio context image must be a bounded PNG, JPEG, or WebP data URL",
        ));
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|_| RuntimeError::new("Studio context image base64 is invalid"))?;
    if bytes.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new(
            "Studio context image must be a bounded PNG, JPEG, or WebP data URL",
        ));
    }
    Ok(())
}

pub(super) fn context_selected_paths(args: &Value) -> RuntimeResult<Vec<String>> {
    let Some(values) = args.get("selectedPaths") else {
        return Ok(Vec::new());
    };
    let Some(values) = values.as_array() else {
        return Err(RuntimeError::new(
            "Studio context selectedPaths must be an array",
        ));
    };
    let mut paths = Vec::new();
    for value in values {
        let Some(value) = value.as_str() else {
            return Err(RuntimeError::new(
                "Studio context selectedPaths must contain strings",
            ));
        };
        validate_project_relative_path(value)?;
        if !value.ends_with(".dowe") || !visible_studio_path(value) {
            return Err(RuntimeError::new(
                "Studio context selected path is not a visible Dowe source file",
            ));
        }
        paths.push(value.replace('\\', "/"));
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub(super) fn studio_context_query_terms(query: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "add",
        "and",
        "change",
        "create",
        "for",
        "implement",
        "the",
        "una",
        "para",
        "con",
    ];
    query
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|term| term.len() > 2 && !STOP_WORDS.contains(term))
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn studio_context_path_score(path: &str, terms: &[String]) -> usize {
    let lower = path.to_ascii_lowercase();
    terms
        .iter()
        .map(|term| usize::from(lower.contains(term)) * 100)
        .sum()
}


