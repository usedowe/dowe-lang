pub(super) fn truncate_studio_text(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

pub(super) async fn cached_studio_analysis(
    root: &Path,
    workspace_fingerprint: &str,
) -> (Vec<Value>, Value) {
    let cache_path = root
        .join(".dowe")
        .join("studio-context-cache")
        .join(format!(
            "{workspace_fingerprint}-{}.json",
            env!("CARGO_PKG_VERSION")
        ));
    if let Ok(bytes) = tokio::fs::read(&cache_path).await
        && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        && value.get("compilerVersion").and_then(Value::as_str) == Some(env!("CARGO_PKG_VERSION"))
        && let (Some(diagnostics), Some(codegraph)) = (
            value.get("diagnostics").and_then(Value::as_array),
            value.get("codegraph"),
        )
    {
        return (diagnostics.clone(), codegraph.clone());
    }
    let diagnostic_root = root.to_path_buf();
    let graph_root = root.to_path_buf();
    let (diagnostics, codegraph) = tokio::join!(
        studio_compile_diagnostics(diagnostic_root),
        studio_codegraph_summary(graph_root),
    );
    let cached = json!({
        "compilerVersion": env!("CARGO_PKG_VERSION"),
        "diagnostics": diagnostics,
        "codegraph": codegraph,
    });
    let private_directory_safe = tokio::fs::symlink_metadata(root.join(".dowe"))
        .await
        .map(|metadata| !metadata.file_type().is_symlink())
        .unwrap_or(true);
    if private_directory_safe {
        if let Ok(bytes) = serde_json::to_vec(&cached) {
            let _ = write_studio_metadata_atomic(&cache_path, &bytes).await;
        }
    }
    (
        cached
            .get("diagnostics")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        cached
            .get("codegraph")
            .cloned()
            .unwrap_or_else(|| json!({})),
    )
}

pub(super) async fn studio_compile_diagnostics(root: PathBuf) -> Vec<Value> {
    let compile_root = root.clone();
    match run_studio_blocking(move || dowe_compiler::compile_dev(&compile_root)).await {
        Ok(Ok(_)) => Vec::new(),
        Ok(Err(error)) => vec![json!({
            "code": "compiler",
            "severity": "error",
            "path": "main.dowe",
            "message": sanitize_studio_text(&error.to_string(), &root),
        })],
        Err(error) => vec![json!({
            "code": "compiler_task",
            "severity": "error",
            "path": "main.dowe",
            "message": sanitize_studio_text(&error, &root),
        })],
    }
}

pub(super) async fn studio_codegraph_summary(root: PathBuf) -> Value {
    match run_studio_blocking({
        let root = root.clone();
        move || build_codegraph(root, BuildOptions::default())
    })
    .await
    {
        Ok(Ok(graph)) => {
            let nodes = graph
                .nodes
                .iter()
                .filter_map(|node| {
                    let path = node
                        .path
                        .as_deref()
                        .and_then(|path| safe_studio_metadata_path(path, &root));
                    let owner = node
                        .owner
                        .as_deref()
                        .and_then(|owner| safe_studio_metadata_path(owner, &root))
                        .unwrap_or_default();
                    (path.is_some() || !owner.is_empty()).then(|| {
                        json!({
                            "kind": format!("{:?}", node.kind).to_ascii_lowercase(),
                            "path": path.unwrap_or_default(),
                            "name": node.name,
                            "owner": owner,
                            "totalLines": node.metrics.as_ref().map_or(0, |metrics| metrics.total_lines),
                        })
                    })
                })
                .take(STUDIO_CONTEXT_MAX_GRAPH_NODES)
                .collect::<Vec<_>>();
            json!({
                "mode": format!("{:?}", graph.mode).to_ascii_lowercase(),
                "nodeCount": graph.nodes.len(),
                "edgeCount": graph.edges.len(),
                "relevantNodes": nodes,
            })
        }
        Ok(Err(error)) => json!({
            "mode": "unknown",
            "nodeCount": 0,
            "edgeCount": 0,
            "relevantNodes": [],
            "error": sanitize_studio_text(&error.to_string(), &root),
        }),
        Err(error) => json!({
            "mode": "unknown",
            "nodeCount": 0,
            "edgeCount": 0,
            "relevantNodes": [],
            "error": sanitize_studio_text(&error, &root),
        }),
    }
}


