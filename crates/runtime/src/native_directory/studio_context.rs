use super::project_files::*;
use super::studio_changes::*;
use super::*;
pub(super) async fn inspect_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project path: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Ok(json!("invalid"));
    }
    let main = path.join("main.dowe");
    if tokio::fs::symlink_metadata(&main)
        .await
        .map(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Ok(json!(if is_valid_dowe_project(&path).await {
            "dowe"
        } else {
            "invalid"
        }));
    }
    Ok(json!(if is_empty_directory(&path).await? {
        "empty"
    } else {
        "invalid"
    }))
}

pub(super) const STUDIO_CONTEXT_PROTOCOL_VERSION: u64 = 1;
pub(super) const STUDIO_CONTEXT_MAX_FILES: usize = 24;
pub(super) const STUDIO_CONTEXT_MAX_FILE_BYTES: usize = 64 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_TOTAL_BYTES: usize = 256 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_SOURCE_READ_BYTES: usize = 2 * 1024 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_QUERY_BYTES: usize = 4096;
pub(super) const STUDIO_CONTEXT_MAX_IMAGE_BYTES: usize = 512 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_IMPORTS: usize = 64;
pub(super) const STUDIO_CONTEXT_MAX_DECLARATIONS: usize = 128;
pub(super) const STUDIO_CONTEXT_MAX_GRAPH_NODES: usize = 24;
pub(super) const STUDIO_CONTEXT_MAX_DIRECTORIES: usize = 4096;
pub(super) const STUDIO_CONTEXT_MAX_SOURCE_FILES: usize = 4096;
pub(super) const STUDIO_STAGE_MAX_FILES: usize = 24;
pub(super) const STUDIO_STAGE_MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
pub(super) const STUDIO_STAGE_MAX_TOTAL_BYTES: usize = 512 * 1024;
pub(super) const STUDIO_STAGE_MAX_PLAN_BYTES: usize = 768 * 1024;
pub(super) const STUDIO_STAGE_MAX_DIFF_BYTES: usize = 256 * 1024;
pub(super) const STUDIO_STAGE_MAX_PATCH_BYTES: usize = 16 * 1024;
pub(super) const STUDIO_STAGE_MAX_TEST_OUTPUT_BYTES: usize = 128 * 1024;
pub(super) const STUDIO_STAGE_DIRECTORY: &str = "studio-staging";

#[derive(Clone)]
pub(super) struct StudioSourceEntry {
    pub(super) relative: String,
    path: PathBuf,
    size: u64,
}

pub(super) struct StudioSourceContent {
    content: String,
    pub(super) digest: String,
}

pub(super) async fn full_studio_source_fingerprint(root: &Path) -> RuntimeResult<String> {
    let mut entries = Vec::new();
    collect_studio_sources(root, root, &mut entries).await?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    let mut digest = Sha256::new();
    for entry in entries {
        let source = read_studio_source(&entry).await?;
        digest.update(entry.relative.as_bytes());
        digest.update([0]);
        digest.update(source.digest.as_bytes());
    }
    Ok(hex_digest(&digest.finalize()))
}

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

pub(super) fn infer_studio_profile(
    query: &str,
    entries: &BTreeMap<String, StudioSourceEntry>,
    has_image: bool,
) -> &'static str {
    if has_image {
        return "viewReference";
    }
    let lower = query.to_ascii_lowercase();
    let views_request = contains_studio_term(
        &lower,
        &[
            "ui",
            "ux",
            "frontend",
            "dashboard",
            "view",
            "vista",
            "layout",
            "page",
            "screen",
            "pantalla",
            "component",
            "componente",
            "form",
            "formulario",
            "button",
            "responsive",
        ],
    );
    let server_request = contains_studio_term(
        &lower,
        &[
            "backend",
            "server",
            "servidor",
            "api",
            "endpoint",
            "handler",
            "middleware",
            "database",
            "persistencia",
            "cache",
            "vector",
            "websocket",
            "provider",
        ],
    );
    if contains_studio_term(
        &lower,
        &[
            "diagnostic",
            "diagnóstico",
            "compiler error",
            "compile error",
            "error de compilación",
        ],
    ) {
        return "diagnostics";
    }
    if contains_studio_term(
        &lower,
        &[
            "screenshot",
            "mockup",
            "reference",
            "referencia",
            "imagen",
            "captura",
        ],
    ) {
        return "viewReference";
    }
    if contains_studio_term(
        &lower,
        &["table", "tabla", "datagrid", "pagination", "paginación"],
    ) {
        return "viewTable";
    }
    if contains_studio_term(
        &lower,
        &["theme", "tema", "palette", "paleta", "colors", "colores"],
    ) {
        return "theme";
    }
    if contains_studio_term(
        &lower,
        &[
            "ipc",
            "filesystem",
            "file system",
            "native host",
            "staging",
            "preview host",
        ],
    ) {
        return "native";
    }
    if contains_studio_term(
        &lower,
        &[
            "pos",
            "point of sale",
            "inventory",
            "inventario",
            "checkout",
        ],
    ) {
        return "domainPos";
    }
    if contains_studio_term(
        &lower,
        &["crm", "pipeline", "lead", "opportunity", "cliente"],
    ) {
        return "domainCrm";
    }
    if contains_studio_term(
        &lower,
        &[
            "ecommerce",
            "e-commerce",
            "catalog",
            "catálogo",
            "cart",
            "carrito",
            "fulfillment",
        ],
    ) {
        return "domainEcommerce";
    }
    if contains_studio_term(
        &lower,
        &[
            "reservation",
            "reservations",
            "reserva",
            "reservas",
            "booking",
            "booking system",
        ],
    ) {
        return "domainReservations";
    }
    if contains_studio_term(
        &lower,
        &["domain", "dominio", "workflow", "workflow de negocio"],
    ) {
        return "domain";
    }
    if views_request && server_request {
        return "fullstack";
    }
    if server_request {
        return "server";
    }
    if views_request {
        return "views";
    }
    let has_views = entries.keys().any(|path| path.starts_with("views/"));
    let has_server = entries.keys().any(|path| path.starts_with("server/"));
    match (has_views, has_server) {
        (true, true) => "fullstack",
        (false, true) => "server",
        (true, false) => "views",
        (false, false) => "core",
    }
}

pub(super) fn infer_studio_request_type(query: &str, has_image: bool) -> &'static str {
    if has_image {
        return "vision_ui";
    }
    let lower = query.to_ascii_lowercase();
    if contains_studio_term(
        &lower,
        &[
            "read",
            "inspect",
            "explore",
            "context",
            "analiza",
            "analizar",
            "inspecciona",
        ],
    ) {
        "context_read"
    } else if contains_studio_term(&lower, &["review", "revisar", "diff", "audit", "auditar"]) {
        "review"
    } else if contains_studio_term(
        &lower,
        &[
            "validate",
            "validation",
            "validar",
            "test",
            "tests",
            "prueba",
        ],
    ) {
        "validation"
    } else if contains_studio_term(
        &lower,
        &["plan", "spec", "design", "diseña", "arquitectura"],
    ) {
        "spec_plan"
    } else if contains_studio_term(
        &lower,
        &[
            "implement",
            "implementation",
            "fix",
            "corrige",
            "arregla",
            "write",
            "escribe",
        ],
    ) {
        "implementation"
    } else if contains_studio_term(
        &lower,
        &[
            "screenshot",
            "mockup",
            "reference",
            "referencia",
            "imagen",
            "captura",
        ],
    ) {
        "vision_ui"
    } else if query.split_whitespace().count() <= 4 {
        "clarify"
    } else {
        "spec_plan"
    }
}

pub(super) fn contains_studio_term(value: &str, terms: &[&str]) -> bool {
    let words = value
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .collect::<BTreeSet<_>>();
    terms.iter().any(|term| {
        if term.contains(' ') || term.contains('-') {
            value.contains(term)
        } else {
            words.contains(term)
        }
    })
}

pub(super) fn parse_studio_imports(source: &str, importer: &str) -> Vec<String> {
    let parent = Path::new(importer)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let mut imports = BTreeSet::new();
    for line in source.lines() {
        let Some(index) = line.find(" from \"") else {
            continue;
        };
        let rest = &line[index + 7..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let specifier = &rest[..end];
        let candidate = if let Some(path) = specifier.strip_prefix("@/") {
            PathBuf::from(path)
        } else if let Some(path) = specifier.strip_prefix("./") {
            parent.join(path)
        } else {
            continue;
        };
        let Some(relative) = normalize_studio_source_path(&candidate) else {
            continue;
        };
        imports.insert(relative);
    }
    imports.into_iter().collect()
}

pub(super) fn normalize_studio_source_path(path: &Path) -> Option<String> {
    if path.is_absolute() {
        return None;
    }
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return None;
        };
        components.push(value.to_string_lossy().into_owned());
    }
    if components.is_empty() {
        return None;
    }
    let mut relative = components.join("/");
    if !relative.ends_with(".dowe") {
        relative.push_str(".dowe");
    }
    Some(relative)
}

pub(super) fn parse_studio_declarations(source: &str, path: &str) -> Vec<String> {
    const DECLARATIONS: &[&str] = &[
        "app",
        "component",
        "database",
        "desktop",
        "endpoints",
        "entity",
        "fn",
        "group",
        "handler",
        "layout",
        "main",
        "middleware",
        "page",
        "route",
        "server",
        "store",
        "theme",
        "type",
        "views",
        "websocket",
    ];
    source
        .lines()
        .filter_map(|line| {
            if line
                .chars()
                .next()
                .is_some_and(|character| character.is_whitespace())
            {
                return None;
            }
            let mut words = line.split_whitespace();
            let declaration = words.next()?;
            if !DECLARATIONS.contains(&declaration) {
                return None;
            }
            let name = words
                .next()
                .unwrap_or_default()
                .trim_end_matches(':')
                .trim_end_matches('"');
            Some(format!("{path}:{declaration}:{name}"))
        })
        .collect()
}

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

pub(super) async fn run_studio_blocking<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        std::thread::Builder::new()
            .name("dowe-studio-context".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn(work)
            .map_err(|error| error.to_string())?
            .join()
            .map_err(|_| "Studio context worker panicked".to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub(super) fn sanitize_studio_text(value: &str, root: &Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    value
        .replace(&root, "<workspace>")
        .replace(&root.replace('/', "\\"), "<workspace>")
}

pub(super) fn safe_studio_metadata_path(value: &str, root: &Path) -> Option<String> {
    let path = Path::new(value);
    let relative = if path.is_absolute() {
        path.strip_prefix(root).ok()?.to_path_buf()
    } else {
        path.to_path_buf()
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    validate_project_relative_path(&relative).ok()?;
    visible_studio_path(&relative).then_some(relative)
}

pub(super) fn visible_studio_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.components().any(|component| {
        let Component::Normal(name) = component else {
            return true;
        };
        matches!(
            name.to_string_lossy().as_ref(),
            ".env" | ".agents" | ".dowe" | ".git" | "AGENTS.md" | "CLAUDE.md"
        )
    })
}

pub(super) fn studio_workspace_id(root: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(b"dowe-studio-workspace:");
    digest.update(root.to_string_lossy().as_bytes());
    hex_digest(&digest.finalize())
}

pub(super) fn hex_digest(digest: &[u8]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
