pub(super) async fn copy_studio_workspace(root: &Path, destination: &Path) -> RuntimeResult<()> {
    let mut entries = tokio::fs::read_dir(root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read Studio workspace: {error}")))?;
    let mut children = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
    {
        children.push(entry);
    }
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || matches!(name.as_str(), "AGENTS.md" | "CLAUDE.md") {
            continue;
        }
        let source = entry.path();
        let metadata = tokio::fs::symlink_metadata(&source)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if should_skip_project_directory(&name) {
                continue;
            }
            Box::pin(copy_studio_directory(&source, destination.join(&name))).await?;
        } else if metadata.is_file()
            && source.extension().and_then(|extension| extension.to_str()) == Some("dowe")
        {
            let relative = name;
            if visible_studio_path(&relative) {
                let target = destination.join(&relative);
                write_studio_file_atomic(
                    &target,
                    &tokio::fs::read(&source).await.map_err(|error| {
                        RuntimeError::new(format!("could not copy Studio workspace: {error}"))
                    })?,
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub(super) async fn copy_studio_directory(
    source: &Path,
    destination: PathBuf,
) -> RuntimeResult<()> {
    tokio::fs::create_dir_all(&destination)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut entries = tokio::fs::read_dir(source)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut children = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
    {
        children.push(entry);
    }
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || matches!(name.as_str(), "AGENTS.md" | "CLAUDE.md") {
            continue;
        }
        let path = entry.path();
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let target = destination.join(&name);
        if metadata.is_dir() {
            if !should_skip_project_directory(&name) {
                Box::pin(copy_studio_directory(&path, target)).await?;
            }
        } else if metadata.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("dowe")
        {
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            write_studio_file_atomic(&target, &bytes).await?;
        }
    }
    Ok(())
}

pub(super) async fn snapshot_studio_changes(
    root: &Path,
    snapshot_root: &Path,
    changes: &[StudioChangeSpec],
) -> RuntimeResult<()> {
    for change in changes {
        let target = studio_target_path(root, &change.path, true).await?;
        if !tokio::fs::try_exists(&target)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?
        {
            continue;
        }
        let bytes = tokio::fs::read(&target).await.map_err(|error| {
            RuntimeError::new(format!("could not snapshot Studio file: {error}"))
        })?;
        if bytes.len() > STUDIO_STAGE_MAX_FILE_BYTES {
            return Err(RuntimeError::new(
                "Studio snapshot file exceeds its size limit",
            ));
        }
        let snapshot = snapshot_root.join(&change.path);
        write_studio_file_atomic(&snapshot, &bytes).await?;
    }
    Ok(())
}

pub(super) async fn apply_changes_to_stage(
    workspace_root: &Path,
    changes: &[StudioChangeSpec],
) -> RuntimeResult<()> {
    for change in changes {
        let target = studio_target_path(workspace_root, &change.path, true).await?;
        match change.operation.as_str() {
            "delete" => {
                if tokio::fs::try_exists(&target)
                    .await
                    .map_err(|error| RuntimeError::new(error.to_string()))?
                {
                    tokio::fs::remove_file(&target).await.map_err(|error| {
                        RuntimeError::new(format!("could not stage deletion: {error}"))
                    })?;
                }
            }
            "upsert" => write_studio_file_atomic(&target, change.content.as_bytes()).await?,
            _ => return Err(RuntimeError::new("Studio change operation is invalid")),
        }
    }
    Ok(())
}

pub(super) fn changes_from_metadata(metadata: &Value) -> RuntimeResult<Vec<StudioChangeSpec>> {
    let value = metadata
        .get("changePlan")
        .ok_or_else(|| RuntimeError::new("Studio stage change plan is missing"))?;
    let serialized = serde_json::to_string(value)
        .map_err(|_| RuntimeError::new("Studio stage change plan cannot be serialized"))?;
    parse_studio_change_plan(&serialized)
}

pub(super) async fn studio_target_path(
    root: &Path,
    relative: &str,
    allow_missing: bool,
) -> RuntimeResult<PathBuf> {
    validate_project_relative_path(relative)?;
    if !visible_studio_path(relative) || !relative.ends_with(".dowe") {
        return Err(RuntimeError::new(
            "Studio staging paths must be visible Dowe source files",
        ));
    }
    let target = root.join(relative);
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(name) = component else {
            return Err(RuntimeError::new("Studio staging path must be relative"));
        };
        current.push(name);
        let metadata = match tokio::fs::symlink_metadata(&current).await {
            Ok(metadata) => metadata,
            Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(RuntimeError::new(error.to_string())),
        };
        if metadata.file_type().is_symlink() {
            return Err(RuntimeError::new(
                "Studio staging paths cannot contain symlinks",
            ));
        }
    }
    if !allow_missing {
        let metadata = tokio::fs::metadata(&target)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        if !metadata.is_file() {
            return Err(RuntimeError::new("Studio staging target is not a file"));
        }
    }
    Ok(target)
}

pub(super) async fn write_studio_file_atomic(path: &Path, content: &[u8]) -> RuntimeResult<()> {
    if content.len() > STUDIO_STAGE_MAX_FILE_BYTES {
        return Err(RuntimeError::new(
            "Studio staged file exceeds its size limit",
        ));
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?;
    }
    let temporary = path.with_file_name(format!(".dowe-studio-{}.tmp", generate_ulid()));
    tokio::fs::write(&temporary, content)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    if let Err(error) = tokio::fs::rename(&temporary, path).await {
        let _ = tokio::fs::remove_file(path).await;
        tokio::fs::rename(&temporary, path)
            .await
            .map_err(|_| RuntimeError::new(format!("could not publish staged file: {error}")))?;
    }
    Ok(())
}

pub(super) async fn write_stage_metadata(stage_root: &Path, metadata: &Value) -> RuntimeResult<()> {
    let serialized = serde_json::to_vec(metadata)
        .map_err(|_| RuntimeError::new("Studio stage metadata cannot be serialized"))?;
    write_studio_metadata_atomic(&stage_root.join("metadata.json"), &serialized).await
}

pub(super) async fn write_studio_metadata_atomic(path: &Path, content: &[u8]) -> RuntimeResult<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?;
    }
    let temporary = path.with_file_name(format!(".metadata-{}.tmp", generate_ulid()));
    tokio::fs::write(&temporary, content)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    tokio::fs::rename(&temporary, path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not publish Studio metadata: {error}")))
}

pub(super) async fn read_stage_metadata(stage_root: &Path) -> RuntimeResult<Value> {
    let path = stage_root.join("metadata.json");
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read Studio stage: {error}")))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(RuntimeError::new(
            "Studio stage metadata exceeds its size limit",
        ));
    }
    let metadata = serde_json::from_slice::<Value>(&bytes)
        .map_err(|_| RuntimeError::new("Studio stage metadata is invalid"))?;
    if !metadata.is_object() {
        return Err(RuntimeError::new("Studio stage metadata must be an object"));
    }
    Ok(metadata)
}

pub(super) fn safe_stage_metadata(mut metadata: Value) -> Value {
    if let Some(object) = metadata.as_object_mut() {
        object.remove("workspaceRoot");
        object.remove("stagedRoot");
    }
    metadata
}

pub(super) async fn studio_context_fingerprint(
    root: &Path,
    query: &str,
    selected_paths: &[String],
) -> RuntimeResult<String> {
    let context = prepare_studio_context(&json!({
        "path": root,
        "query": query,
        "selectedPaths": selected_paths,
        "detail": "compact",
    }))
    .await?;
    context
        .get("sourceFingerprint")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| RuntimeError::new("Studio context fingerprint is missing"))
}

pub(super) async fn studio_context_fingerprint_from_metadata(
    root: &Path,
    metadata: &Value,
) -> RuntimeResult<String> {
    let query = metadata
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let selected_paths = metadata
        .get("selectedPaths")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    studio_context_fingerprint(root, query, &selected_paths).await
}


