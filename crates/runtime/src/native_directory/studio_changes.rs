use super::project_files::*;
use super::studio_agent::*;
use super::studio_context::*;
use super::*;
pub(super) async fn stage_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let requested_root = required_string(args, "root")?;
    let root = canonical_studio_root(&requested_root).await?;
    let run_id = required_string(args, "runId")?;
    validate_studio_run_id(&run_id)?;
    let expected_fingerprint = required_string(args, "sourceFingerprint")?;
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if query.len() > STUDIO_CONTEXT_MAX_QUERY_BYTES {
        return Err(RuntimeError::new(
            "Studio stage query exceeds its size limit",
        ));
    }
    let selected_paths = context_selected_paths(args)?;
    let change_plan = required_string(args, "changePlan")?;
    if change_plan.len() > STUDIO_STAGE_MAX_PLAN_BYTES {
        return Err(RuntimeError::new(
            "Studio change plan exceeds its size limit",
        ));
    }
    let changes = parse_studio_change_plan(&change_plan)?;
    let current_fingerprint = studio_context_fingerprint(&root, &query, &selected_paths).await?;
    if current_fingerprint != expected_fingerprint {
        return Err(RuntimeError::new(
            "Studio workspace changed since the context was prepared",
        ));
    }
    let base_workspace_fingerprint = full_studio_source_fingerprint(&root).await?;
    ensure_studio_stage_directory(&root).await?;
    let stage_id = generate_ulid();
    let stage_root = studio_stage_root(&root, &stage_id)?;
    let workspace_root = stage_root.join("workspace");
    let snapshot_root = stage_root.join("snapshot");
    if let Err(error) = tokio::fs::create_dir_all(&workspace_root).await {
        return Err(RuntimeError::new(format!(
            "could not create Studio staging directory: {error}"
        )));
    }
    if let Err(error) = copy_studio_workspace(&root, &workspace_root).await {
        let _ = tokio::fs::remove_dir_all(&stage_root).await;
        return Err(error);
    }
    if let Err(error) = snapshot_studio_changes(&root, &snapshot_root, &changes).await {
        let _ = tokio::fs::remove_dir_all(&stage_root).await;
        return Err(error);
    }
    if let Err(error) = apply_changes_to_stage(&workspace_root, &changes).await {
        let _ = tokio::fs::remove_dir_all(&stage_root).await;
        return Err(error);
    }
    let diagnostics = studio_compile_diagnostics(workspace_root.clone()).await;
    let compiler_ok = diagnostics.is_empty();
    let tests = run_studio_tests(workspace_root.clone()).await;
    let tests_ok = tests.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let visual = studio_visual_validation(&root, &changes).await;
    let visual_ok = visual.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let ownership = json!({ "ok": true, "checkedFiles": changes.len() });
    let validation = json!({
        "ok": compiler_ok && tests_ok && visual_ok,
        "ownership": ownership,
        "compiler": { "ok": compiler_ok, "diagnostics": diagnostics },
        "tests": tests,
        "visual": visual,
    });
    let (diff, diff_text) = build_studio_diff(&root, &workspace_root, &changes).await?;
    let staged_context_fingerprint =
        studio_context_fingerprint(&workspace_root, &query, &selected_paths).await?;
    let staged_workspace_fingerprint = full_studio_source_fingerprint(&workspace_root).await?;
    let plan_value = serde_json::from_str::<Value>(&change_plan)
        .map_err(|_| RuntimeError::new("Studio change plan is not valid JSON"))?;
    let plan_hash = hex_digest(&Sha256::digest(change_plan.as_bytes()));
    let metadata = json!({
        "version": 1,
        "runId": run_id,
        "stageId": stage_id,
        "status": "staged",
        "query": query,
        "selectedPaths": selected_paths,
        "changePlan": plan_value,
        "planHash": plan_hash,
        "baseFingerprint": expected_fingerprint,
        "baseWorkspaceFingerprint": base_workspace_fingerprint,
        "postFingerprint": staged_context_fingerprint,
        "postWorkspaceFingerprint": staged_workspace_fingerprint,
        "diff": diff,
        "diffText": diff_text,
        "validation": validation,
    });
    write_stage_metadata(&stage_root, &metadata).await?;
    Ok(json!({
        "ok": true,
        "stageId": stage_id,
        "status": "staged",
        "planHash": plan_hash,
        "baseFingerprint": expected_fingerprint,
        "postFingerprint": staged_context_fingerprint,
        "diff": diff,
        "diffText": diff_text,
        "validation": validation,
    }))
}

pub(super) async fn inspect_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let root = canonical_studio_root(&required_string(args, "root")?).await?;
    let stage_id = required_stage_id(args)?;
    ensure_studio_stage_directory(&root).await?;
    let stage_root = studio_stage_root(&root, &stage_id)?;
    let metadata = read_stage_metadata(&stage_root).await?;
    Ok(safe_stage_metadata(metadata))
}

pub(super) async fn apply_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let root = canonical_studio_root(&required_string(args, "root")?).await?;
    let stage_id = required_stage_id(args)?;
    ensure_studio_stage_directory(&root).await?;
    let stage_root = studio_stage_root(&root, &stage_id)?;
    let mut metadata = read_stage_metadata(&stage_root).await?;
    let status = metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status == "applied" {
        return Ok(json!({
            "ok": true,
            "stageId": stage_id,
            "status": "applied",
            "postFingerprint": metadata.get("postFingerprint").cloned().unwrap_or(Value::String(String::new())),
        }));
    }
    if status != "staged" {
        return Err(RuntimeError::new("Studio change is not awaiting approval"));
    }
    let validation_ok = metadata
        .get("validation")
        .and_then(|value| value.get("ok"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !validation_ok {
        return Err(RuntimeError::new(
            "Studio change did not pass required validation",
        ));
    }
    let expected_fingerprint = required_string(args, "sourceFingerprint")?;
    let current_fingerprint = studio_context_fingerprint_from_metadata(&root, &metadata).await?;
    if current_fingerprint
        != metadata
            .get("baseFingerprint")
            .and_then(Value::as_str)
            .unwrap_or_default()
        || current_fingerprint != expected_fingerprint
    {
        return Err(RuntimeError::new(
            "Studio workspace changed before approval",
        ));
    }
    let expected_workspace_fingerprint = metadata
        .get("baseWorkspaceFingerprint")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if full_studio_source_fingerprint(&root).await? != expected_workspace_fingerprint {
        return Err(RuntimeError::new(
            "Studio workspace changed outside the prepared context",
        ));
    }
    let changes = changes_from_metadata(&metadata)?;
    let workspace_root = stage_root.join("workspace");
    let snapshot_root = stage_root.join("snapshot");
    let mut applied_changes = Vec::new();
    for change in &changes {
        let result = apply_one_studio_change(&root, &workspace_root, change).await;
        if let Err(error) = result {
            restore_studio_changes(&root, &snapshot_root, &applied_changes).await?;
            return Err(error);
        }
        applied_changes.push(change.clone());
    }
    let post_fingerprint = match studio_context_fingerprint_from_metadata(&root, &metadata).await {
        Ok(value) => value,
        Err(error) => {
            restore_studio_changes(&root, &snapshot_root, &applied_changes).await?;
            return Err(error);
        }
    };
    let post_workspace_fingerprint = match full_studio_source_fingerprint(&root).await {
        Ok(value) => value,
        Err(error) => {
            restore_studio_changes(&root, &snapshot_root, &applied_changes).await?;
            return Err(error);
        }
    };
    metadata["status"] = Value::String("applied".to_string());
    metadata["appliedFingerprint"] = Value::String(post_fingerprint.clone());
    metadata["appliedWorkspaceFingerprint"] = Value::String(post_workspace_fingerprint);
    if let Err(error) = write_stage_metadata(&stage_root, &metadata).await {
        restore_studio_changes(&root, &snapshot_root, &applied_changes).await?;
        return Err(error);
    }
    Ok(json!({
        "ok": true,
        "stageId": stage_id,
        "status": "applied",
        "postFingerprint": post_fingerprint,
    }))
}

pub(super) async fn reject_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let root = canonical_studio_root(&required_string(args, "root")?).await?;
    let stage_id = required_stage_id(args)?;
    ensure_studio_stage_directory(&root).await?;
    let stage_root = studio_stage_root(&root, &stage_id)?;
    if !tokio::fs::try_exists(&stage_root)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
    {
        return Ok(json!({ "ok": true, "stageId": stage_id, "status": "rejected" }));
    }
    let metadata = read_stage_metadata(&stage_root).await?;
    let status = metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status == "applied" {
        return Err(RuntimeError::new(
            "Applied Studio changes cannot be rejected",
        ));
    }
    tokio::fs::remove_dir_all(&stage_root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not reject Studio change: {error}")))?;
    Ok(json!({ "ok": true, "stageId": stage_id, "status": "rejected" }))
}

pub(super) async fn rollback_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let root = canonical_studio_root(&required_string(args, "root")?).await?;
    let stage_id = required_stage_id(args)?;
    ensure_studio_stage_directory(&root).await?;
    let stage_root = studio_stage_root(&root, &stage_id)?;
    let mut metadata = read_stage_metadata(&stage_root).await?;
    if metadata.get("status").and_then(Value::as_str) == Some("rolled_back") {
        return Ok(json!({ "ok": true, "stageId": stage_id, "status": "rolled_back" }));
    }
    if metadata.get("status").and_then(Value::as_str) != Some("applied") {
        return Err(RuntimeError::new(
            "Only applied Studio changes can be rolled back",
        ));
    }
    let expected_workspace_fingerprint = metadata
        .get("appliedWorkspaceFingerprint")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if full_studio_source_fingerprint(&root).await? != expected_workspace_fingerprint {
        return Err(RuntimeError::new(
            "Studio workspace changed after approval; rollback stopped",
        ));
    }
    let changes = changes_from_metadata(&metadata)?;
    let snapshot_root = stage_root.join("snapshot");
    restore_studio_changes(&root, &snapshot_root, &changes).await?;
    let rollback_fingerprint = full_studio_source_fingerprint(&root).await?;
    metadata["status"] = Value::String("rolled_back".to_string());
    metadata["rollbackWorkspaceFingerprint"] = Value::String(rollback_fingerprint.clone());
    write_stage_metadata(&stage_root, &metadata).await?;
    Ok(json!({
        "ok": true,
        "stageId": stage_id,
        "status": "rolled_back",
        "workspaceFingerprint": rollback_fingerprint,
    }))
}

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

pub(super) async fn build_studio_diff(
    root: &Path,
    workspace_root: &Path,
    changes: &[StudioChangeSpec],
) -> RuntimeResult<(Vec<Value>, String)> {
    let mut entries = Vec::new();
    let mut text = String::new();
    for change in changes {
        let before = read_optional_studio_file(root, &change.path).await?;
        let after = read_optional_studio_file(workspace_root, &change.path).await?;
        let before_sha = before
            .as_deref()
            .map(|bytes| hex_digest(&Sha256::digest(bytes)))
            .unwrap_or_default();
        let after_sha = after
            .as_deref()
            .map(|bytes| hex_digest(&Sha256::digest(bytes)))
            .unwrap_or_default();
        let (patch, added, removed) = studio_unified_diff(
            &change.path,
            before.as_deref().unwrap_or_default(),
            after.as_deref().unwrap_or_default(),
        );
        let patch = truncate_studio_text(&patch, STUDIO_STAGE_MAX_PATCH_BYTES);
        if text.len() < STUDIO_STAGE_MAX_DIFF_BYTES {
            let remaining = STUDIO_STAGE_MAX_DIFF_BYTES - text.len();
            text.push_str(&truncate_studio_text(&patch, remaining));
        }
        entries.push(json!({
            "path": change.path,
            "operation": change.operation,
            "owner": change.owner,
            "beforeSha256": before_sha,
            "afterSha256": after_sha,
            "addedLines": added,
            "removedLines": removed,
            "patch": patch,
        }));
    }
    Ok((entries, text))
}

pub(super) async fn apply_one_studio_change(
    root: &Path,
    workspace_root: &Path,
    change: &StudioChangeSpec,
) -> RuntimeResult<()> {
    let target = studio_target_path(root, &change.path, true).await?;
    match change.operation.as_str() {
        "delete" => {
            if tokio::fs::try_exists(&target)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?
            {
                tokio::fs::remove_file(&target).await.map_err(|error| {
                    RuntimeError::new(format!("could not apply staged deletion: {error}"))
                })?;
            }
            Ok(())
        }
        "upsert" => {
            let staged = studio_target_path(workspace_root, &change.path, false).await?;
            let content = tokio::fs::read(&staged).await.map_err(|error| {
                RuntimeError::new(format!("could not read staged file: {error}"))
            })?;
            write_studio_file_atomic(&target, &content).await
        }
        _ => Err(RuntimeError::new("Studio change operation is invalid")),
    }
}

pub(super) async fn restore_studio_changes(
    root: &Path,
    snapshot_root: &Path,
    changes: &[StudioChangeSpec],
) -> RuntimeResult<()> {
    for change in changes {
        let target = studio_target_path(root, &change.path, true).await?;
        let snapshot = snapshot_root.join(&change.path);
        if tokio::fs::try_exists(&snapshot)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?
        {
            let content = tokio::fs::read(&snapshot)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            write_studio_file_atomic(&target, &content).await?;
        } else if tokio::fs::try_exists(&target)
            .await
            .map_err(|error| RuntimeError::new(error.to_string()))?
        {
            tokio::fs::remove_file(&target)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
    }
    Ok(())
}

pub(super) async fn read_optional_studio_file(
    root: &Path,
    relative: &str,
) -> RuntimeResult<Option<Vec<u8>>> {
    let path = studio_target_path(root, relative, true).await?;
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            if bytes.len() > STUDIO_STAGE_MAX_FILE_BYTES {
                return Err(RuntimeError::new("Studio diff file exceeds its size limit"));
            }
            Ok(Some(bytes))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(RuntimeError::new(error.to_string())),
    }
}

pub(super) fn studio_unified_diff(
    path: &str,
    before: &[u8],
    after: &[u8],
) -> (String, usize, usize) {
    let before = String::from_utf8_lossy(before);
    let after = String::from_utf8_lossy(after);
    if before == after {
        return (String::new(), 0, 0);
    }
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let mut patch = format!("--- a/{path}\n+++ b/{path}\n");
    for line in &before_lines {
        patch.push('-');
        patch.push_str(line);
        patch.push('\n');
    }
    for line in &after_lines {
        patch.push('+');
        patch.push_str(line);
        patch.push('\n');
    }
    (patch, after_lines.len(), before_lines.len())
}

pub(super) async fn run_studio_tests(root: PathBuf) -> Value {
    let command_root = root.clone();
    let command = tokio::task::spawn_blocking(move || {
        Command::new("dowe")
            .args(["test", "--json"])
            .current_dir(command_root)
            .output()
    });
    let output = match tokio::time::timeout(Duration::from_secs(45), command).await {
        Ok(Ok(Ok(output))) => output,
        Ok(Ok(Err(error))) => {
            return json!({
                "ok": false,
                "status": "unavailable",
                "code": "test_runner_unavailable",
                "output": sanitize_studio_text(&error.to_string(), &root),
            });
        }
        Ok(Err(error)) => {
            return json!({
                "ok": false,
                "status": "unavailable",
                "code": "test_runner_task_failed",
                "output": sanitize_studio_text(&error.to_string(), &root),
            });
        }
        Err(_) => {
            return json!({
                "ok": false,
                "status": "timeout",
                "code": "test_runner_timeout",
                "output": "Studio test runner timed out",
            });
        }
    };
    let mut output_text = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.stderr.is_empty() {
        output_text.push_str("\n");
        output_text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    output_text = sanitize_studio_text(&output_text, &root);
    output_text = truncate_studio_text(&output_text, STUDIO_STAGE_MAX_TEST_OUTPUT_BYTES);
    json!({
        "ok": output.status.success(),
        "status": if output.status.success() { "passed" } else { "failed" },
        "output": output_text,
    })
}

pub(super) async fn studio_visual_validation(root: &Path, changes: &[StudioChangeSpec]) -> Value {
    let view_changed = changes
        .iter()
        .any(|change| change.path.starts_with("views/"));
    if !view_changed {
        return json!({ "ok": true, "status": "not_applicable" });
    }
    let blueprint = root.join(".dowe").join("visual-qa");
    let configured = tokio::fs::metadata(&blueprint)
        .await
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);
    if !configured {
        return json!({
            "ok": true,
            "status": "not_applicable",
            "message": "No visual QA blueprint is configured for this workspace.",
        });
    }
    let mut screens = 0usize;
    let mut entries = match tokio::fs::read_dir(&blueprint).await {
        Ok(entries) => entries,
        Err(error) => {
            return json!({ "ok": false, "status": "invalid", "message": error.to_string() });
        }
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path().join("blueprint.json");
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        if bytes.len() > 64 * 1024 || serde_json::from_slice::<Value>(&bytes).is_err() {
            return json!({ "ok": false, "status": "invalid", "message": "Visual QA blueprint is invalid." });
        }
        screens += 1;
    }
    if screens == 0 {
        return json!({ "ok": false, "status": "invalid", "message": "Visual QA directory has no blueprints." });
    }
    json!({
        "ok": true,
        "status": "blueprint_checked",
        "screens": screens,
        "message": "Visual QA blueprints are valid; browser comparison remains a preview review step.",
    })
}
