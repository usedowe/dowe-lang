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


