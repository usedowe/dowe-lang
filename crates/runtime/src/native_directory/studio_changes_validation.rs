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

