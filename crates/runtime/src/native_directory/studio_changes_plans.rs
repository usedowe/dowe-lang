pub(super) fn parse_studio_change_plan(value: &str) -> RuntimeResult<Vec<StudioChangeSpec>> {
    let plan = serde_json::from_str::<Value>(value)
        .map_err(|_| RuntimeError::new("Studio change plan is not valid JSON"))?;
    let files = plan
        .get("files")
        .or_else(|| plan.get("changes"))
        .and_then(Value::as_array)
        .ok_or_else(|| RuntimeError::new("Studio change plan must contain a files array"))?;
    if files.is_empty() || files.len() > STUDIO_STAGE_MAX_FILES {
        return Err(RuntimeError::new(
            "Studio change plan file count is outside its limit",
        ));
    }
    let mut paths = BTreeSet::new();
    let mut total_bytes = 0usize;
    let mut changes = Vec::with_capacity(files.len());
    for file in files {
        let object = file
            .as_object()
            .ok_or_else(|| RuntimeError::new("Studio change entries must be objects"))?;
        let path = object
            .get("path")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| RuntimeError::new("Studio change path is required"))?
            .replace('\\', "/");
        if path.len() > 256 || !paths.insert(path.clone()) {
            return Err(RuntimeError::new(
                "Studio change paths must be unique and bounded",
            ));
        }
        validate_project_relative_path(&path)?;
        if !visible_studio_path(&path) || !path.ends_with(".dowe") {
            return Err(RuntimeError::new(
                "Studio changes may only target visible Dowe source files",
            ));
        }
        let expected_owner = studio_change_owner(&path)
            .ok_or_else(|| RuntimeError::new("Studio change path has no allowed owner"))?;
        let owner = object
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| RuntimeError::new("Studio change owner is required"))?;
        if owner != expected_owner {
            return Err(RuntimeError::new(
                "Studio change owner does not match the source path",
            ));
        }
        let operation = object
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or("upsert")
            .to_string();
        if !matches!(operation.as_str(), "upsert" | "delete") {
            return Err(RuntimeError::new("Studio change operation is invalid"));
        }
        let content = object
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if operation == "upsert" && content.is_empty() {
            return Err(RuntimeError::new(
                "Studio upsert changes require non-empty content",
            ));
        }
        if content.len() > STUDIO_STAGE_MAX_FILE_BYTES {
            return Err(RuntimeError::new(
                "Studio change file exceeds its size limit",
            ));
        }
        total_bytes = total_bytes.saturating_add(content.len());
        if total_bytes > STUDIO_STAGE_MAX_TOTAL_BYTES {
            return Err(RuntimeError::new(
                "Studio change content exceeds its total size limit",
            ));
        }
        changes.push(StudioChangeSpec {
            path,
            operation,
            owner: owner.to_string(),
            content,
        });
    }
    Ok(changes)
}

pub(super) fn studio_change_owner(path: &str) -> Option<&'static str> {
    if path == "main.dowe" || path.starts_with("types/") {
        Some("core")
    } else if path == "theme.dowe" {
        Some("theme")
    } else if path.starts_with("views/") {
        Some("views")
    } else if path.starts_with("server/") {
        Some("server")
    } else if path.starts_with("tests/") {
        Some("tests")
    } else {
        None
    }
}

pub(super) async fn canonical_studio_root(value: &str) -> RuntimeResult<PathBuf> {
    let root = tokio::fs::canonicalize(value).await.map_err(|error| {
        RuntimeError::new(format!("could not inspect Studio workspace: {error}"))
    })?;
    let metadata = tokio::fs::metadata(&root).await.map_err(|error| {
        RuntimeError::new(format!("could not inspect Studio workspace: {error}"))
    })?;
    if !metadata.is_dir() {
        return Err(RuntimeError::new(
            "Studio workspace root must be a directory",
        ));
    }
    if !tokio::fs::try_exists(root.join("main.dowe"))
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
    {
        return Err(RuntimeError::new("Studio workspace must contain main.dowe"));
    }
    Ok(root)
}

pub(super) async fn ensure_studio_stage_directory(root: &Path) -> RuntimeResult<()> {
    let private = root.join(".dowe");
    if let Ok(metadata) = tokio::fs::symlink_metadata(&private).await
        && metadata.file_type().is_symlink()
    {
        return Err(RuntimeError::new(
            "Studio private directory cannot be a symlink",
        ));
    }
    tokio::fs::create_dir_all(&private)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let staging = private.join(STUDIO_STAGE_DIRECTORY);
    if let Ok(metadata) = tokio::fs::symlink_metadata(&staging).await
        && metadata.file_type().is_symlink()
    {
        return Err(RuntimeError::new(
            "Studio staging directory cannot be a symlink",
        ));
    }
    tokio::fs::create_dir_all(staging)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))
}

pub(super) fn studio_stage_root(root: &Path, stage_id: &str) -> RuntimeResult<PathBuf> {
    validate_studio_run_id(stage_id)?;
    Ok(root
        .join(".dowe")
        .join(STUDIO_STAGE_DIRECTORY)
        .join(stage_id))
}

pub(super) fn required_stage_id(args: &Value) -> RuntimeResult<String> {
    let value = required_string(args, "stageId")?;
    validate_studio_run_id(&value)?;
    Ok(value)
}


