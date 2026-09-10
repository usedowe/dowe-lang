pub(super) async fn prepare_studio_context(args: &Value) -> RuntimeResult<Value> {
    let requested_root = required_string(args, "path")?;
    let root = tokio::fs::canonicalize(&requested_root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    let metadata = tokio::fs::metadata(&root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if !metadata.is_dir() {
        return Err(RuntimeError::new("Studio context root must be a directory"));
    }
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    if query.len() > STUDIO_CONTEXT_MAX_QUERY_BYTES {
        return Err(RuntimeError::new(
            "Studio context query exceeds its size limit",
        ));
    }
    let image = args
        .get("image")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    validate_studio_image(&image)?;
    let detail = args
        .get("detail")
        .and_then(Value::as_str)
        .unwrap_or("full")
        .trim();
    if !matches!(detail, "compact" | "full") {
        return Err(RuntimeError::new(
            "Studio context detail must be `compact` or `full`",
        ));
    }
    let selected_paths = context_selected_paths(args)?;
    if selected_paths.len() > STUDIO_CONTEXT_MAX_FILES {
        return Err(RuntimeError::new(
            "Studio context selected file count exceeds its limit",
        ));
    }

    let mut entries = Vec::new();
    collect_studio_sources(&root, &root, &mut entries).await?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    let entries = entries
        .into_iter()
        .map(|entry| (entry.relative.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut requested = BTreeSet::new();
    for relative in ["main.dowe", "theme.dowe"] {
        if entries.contains_key(relative) {
            requested.insert(relative.to_string());
        }
    }
    for relative in selected_paths {
        if !entries.contains_key(&relative) {
            return Err(RuntimeError::new(
                "Studio context selected path is not a visible Dowe source file",
            ));
        }
        requested.insert(relative);
    }

    let query_terms = studio_context_query_terms(query);
    let mut ranked = entries
        .values()
        .filter_map(|entry| {
            let score = studio_context_path_score(&entry.relative, &query_terms);
            (score > 0).then(|| (score, entry.relative.clone()))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    for (_, relative) in ranked {
        if requested.len() >= STUDIO_CONTEXT_MAX_FILES {
            break;
        }
        requested.insert(relative);
    }

    let mut contents = BTreeMap::new();
    let mut pending = requested.iter().cloned().collect::<Vec<_>>();
    while let Some(relative) = pending.pop() {
        if contents.contains_key(&relative) {
            continue;
        }
        let Some(entry) = entries.get(&relative) else {
            continue;
        };
        let source = read_studio_source(entry).await?;
        let imports = parse_studio_imports(&source.content, &relative);
        contents.insert(relative, source);
        for import in imports {
            if contents.len() + pending.len() >= STUDIO_CONTEXT_MAX_FILES {
                break;
            }
            if entries.contains_key(&import) && requested.insert(import.clone()) {
                pending.push(import);
            }
        }
    }

    let mut files = Vec::new();
    let mut imports = BTreeSet::new();
    let mut declarations = BTreeSet::new();
    let mut source_fingerprint = Sha256::new();
    let mut total_bytes = 0usize;
    let mut truncated = false;
    for (relative, source) in &contents {
        source_fingerprint.update(relative.as_bytes());
        source_fingerprint.update([0]);
        source_fingerprint.update(source.digest.as_bytes());
        for import in parse_studio_imports(&source.content, relative) {
            if imports.len() < STUDIO_CONTEXT_MAX_IMPORTS {
                imports.insert(import);
            } else {
                truncated = true;
            }
        }
        for declaration in parse_studio_declarations(&source.content, relative) {
            if declarations.len() < STUDIO_CONTEXT_MAX_DECLARATIONS {
                declarations.insert(declaration);
            } else {
                truncated = true;
            }
        }
        if detail == "compact" {
            files.push(json!({
                "path": relative,
                "content": "",
                "sha256": source.digest,
                "truncated": false,
            }));
            continue;
        }
        if total_bytes >= STUDIO_CONTEXT_MAX_TOTAL_BYTES {
            truncated = true;
            continue;
        }
        let remaining = STUDIO_CONTEXT_MAX_TOTAL_BYTES - total_bytes;
        let content_limit = remaining.min(STUDIO_CONTEXT_MAX_FILE_BYTES);
        let raw_visible_content = truncate_studio_text(&source.content, content_limit);
        if raw_visible_content.len() < source.content.len() {
            truncated = true;
        }
        let visible_content = sanitize_studio_text(&raw_visible_content, &root);
        total_bytes = total_bytes.saturating_add(visible_content.len());
        files.push(json!({
            "path": relative,
            "content": visible_content,
            "sha256": source.digest,
            "truncated": visible_content.len() < source.content.len(),
        }));
    }

    let source_fingerprint = hex_digest(&source_fingerprint.finalize());
    let workspace_fingerprint = full_studio_source_fingerprint(&root).await?;
    let (diagnostics, codegraph) = cached_studio_analysis(&root, &workspace_fingerprint).await;
    let mode = if entries.contains_key("main.dowe") {
        "dowe"
    } else {
        "invalid"
    };
    let profile = infer_studio_profile(query, &entries, !image.is_empty());
    let request_type = infer_studio_request_type(query, !image.is_empty());
    Ok(json!({
        "protocolVersion": STUDIO_CONTEXT_PROTOCOL_VERSION,
        "profile": profile,
        "requestType": request_type,
        "image": if detail == "compact" { String::new() } else { image },
        "detail": detail,
        "workspaceId": studio_workspace_id(&root),
        "compilerVersion": env!("CARGO_PKG_VERSION"),
        "mode": mode,
        "sourceFingerprint": source_fingerprint,
        "workspaceFingerprint": workspace_fingerprint,
        "files": files,
        "imports": imports.into_iter().collect::<Vec<_>>(),
        "declarations": declarations.into_iter().collect::<Vec<_>>(),
        "diagnostics": diagnostics,
        "codegraph": codegraph,
        "truncated": truncated,
    }))
}

