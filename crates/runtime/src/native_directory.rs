use crate::init::{InitProjectOptions, ProjectTemplate, init_project};
use crate::{RuntimeError, RuntimeResult};
use dowe_codegraph::{BuildOptions, build_codegraph};
use dowe_database::{StoreRecord, StoreValue, open_database};
use dowe_id::generate_ulid;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};

pub async fn invoke(root: &Path, function: &str, args: &Value) -> Option<RuntimeResult<Value>> {
    let result = match function {
        "pickDirectory" => pick_directory().await,
        "createFolder" => create_folder(args).await,
        "listFolders" => list_folders(args).await,
        "listProjectFiles" => list_project_files(args).await,
        "readProjectFile" => read_project_file(args).await,
        "writeProjectFile" => write_project_file(args).await,
        "startStudioPreview" => start_studio_preview(args).await,
        "stopStudioPreview" => stop_studio_preview(args).await,
        "studioEvent" => studio_event(args).await,
        "studioAgentStream" => studio_agent_stream(args).await,
        "cancelStudioAgent" => cancel_studio_agent(args).await,
        "studioSketch" => studio_sketch(root, args).await,
        "listStudioApps" => list_studio_apps(root).await,
        "saveStudioApp" => save_studio_app(root, args).await,
        "registerStudioApp" => register_studio_app(root, args).await,
        "deleteStudioApp" => delete_studio_app(root, args).await,
        "inspectDoweProject" => inspect_dowe_project(args).await,
        "prepareStudioContext" => prepare_studio_context(args).await,
        "validDoweProject" => valid_dowe_project(args).await,
        "emptyDoweFolder" => empty_dowe_folder(args).await,
        "initializeDoweProject" => initialize_dowe_project(args).await,
        "cloneDoweRepository" => clone_dowe_repository(args).await,
        "stageStudioChanges" => stage_studio_changes(args).await,
        "inspectStudioChanges" => inspect_studio_changes(args).await,
        "applyStudioChanges" => apply_studio_changes(args).await,
        "rejectStudioChanges" => reject_studio_changes(args).await,
        "rollbackStudioChanges" => rollback_studio_changes(args).await,
        _ => return None,
    };
    Some(result)
}

async fn list_studio_apps(root: &Path) -> RuntimeResult<Value> {
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let records = database
        .records("workspace_apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(Value::Array(
        records
            .into_iter()
            .map(|record| {
                let mut object = serde_json::Map::new();
                for (key, value) in record {
                    object.insert(key, value.to_json());
                }
                Value::Object(object)
            })
            .collect(),
    ))
}

async fn save_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let path = required_string(args, "path")?;
    let name = required_string(args, "name")?;
    register_studio_app_record(root, path, name, "local").await
}

async fn register_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let path = required_string(args, "path")?;
    let name = required_string(args, "name")?;
    let source = required_string(args, "source")?;
    register_studio_app_record(root, path, name, &source).await
}

async fn register_studio_app_record(
    root: &Path,
    path: String,
    name: String,
    source: &str,
) -> RuntimeResult<Value> {
    if !matches!(source, "local" | "github" | "dowe") {
        return Err(RuntimeError::new(
            "application source must be local, github, or dowe",
        ));
    }
    let path = validated_dowe_project_root(&path).await?;
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut record = StoreRecord::new();
    record.insert("id".to_string(), StoreValue::String(generate_ulid()));
    record.insert(
        "path".to_string(),
        StoreValue::String(path.to_string_lossy().into_owned()),
    );
    record.insert("name".to_string(), StoreValue::String(name));
    record.insert("source".to_string(), StoreValue::String(source.to_string()));
    let icon_path = path.join("icons/web/favicon-32x32.png");
    let has_icon = tokio::fs::metadata(&icon_path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false);
    record.insert(
        "icon".to_string(),
        StoreValue::String(icon_path.to_string_lossy().into_owned()),
    );
    record.insert("hasIcon".to_string(), StoreValue::Bool(has_icon));
    let now = StoreValue::Timestamp(format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| RuntimeError::new("system clock is invalid"))?
            .as_secs()
    ));
    record.insert("createdAt".to_string(), now.clone());
    record.insert("updatedAt".to_string(), now);
    let inserted = database
        .insert("workspace_apps", record)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let mut object = serde_json::Map::new();
    for (key, value) in inserted {
        object.insert(key, value.to_json());
    }
    Ok(Value::Object(object))
}

async fn delete_studio_app(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let id = required_string(args, "id")?;
    let database = open_database(root, "dowe-studio-apps")
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let changed = database
        .delete("workspace_apps", "id", &StoreValue::String(id))
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(json!({ "changed": changed }))
}

async fn start_studio_preview(args: &Value) -> RuntimeResult<Value> {
    let root = required_string(args, "root")?;
    crate::start_preview(root).await
}

async fn stop_studio_preview(args: &Value) -> RuntimeResult<Value> {
    let preview = required_string(args, "preview")?;
    crate::stop_preview(&preview).await
}

async fn studio_event(args: &Value) -> RuntimeResult<Value> {
    let _event = required_string(args, "event")?;
    Ok(json!(true))
}

const STUDIO_AGENT_TIMEOUT: Duration = Duration::from_secs(125);
const STUDIO_AGENT_MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
const STUDIO_AGENT_MAX_EVENTS: usize = 256;
const STUDIO_AGENT_MAX_EVENT_BYTES: usize = 128 * 1024;
const STUDIO_AGENT_MAX_TOTAL_EVENT_BYTES: usize = 512 * 1024;
const STUDIO_AGENT_MAX_ANSWER_BYTES: usize = 256 * 1024;

fn active_studio_agents() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static ACTIVE: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn studio_agent_stream(args: &Value) -> RuntimeResult<Value> {
    let backend = required_string(args, "backend")?;
    let authorization = required_string(args, "authorization")?;
    let run_id = required_string(args, "runId")?;
    let payload = required_string(args, "payload")?;
    if payload.len() > STUDIO_AGENT_MAX_PAYLOAD_BYTES {
        return Err(RuntimeError::new(
            "Studio agent payload exceeds its size limit",
        ));
    }
    validate_studio_authorization(&authorization)?;
    validate_studio_run_id(&run_id)?;
    let websocket_url = studio_agent_websocket_url(&backend, &run_id)?;
    let cancellation = Arc::new(AtomicBool::new(false));
    if let Ok(mut active) = active_studio_agents().lock() {
        if let Some(previous) = active.insert(run_id.clone(), cancellation.clone()) {
            previous.store(true, Ordering::Release);
        }
    }
    let result = tokio::time::timeout(
        STUDIO_AGENT_TIMEOUT,
        studio_agent_stream_inner(
            websocket_url,
            authorization,
            run_id.clone(),
            payload,
            cancellation,
        ),
    )
    .await
    .map_err(|_| RuntimeError::new("Studio agent request timed out"))?;
    if let Ok(mut active) = active_studio_agents().lock() {
        active.remove(&run_id);
    }
    result
}

async fn cancel_studio_agent(args: &Value) -> RuntimeResult<Value> {
    let run_id = required_string(args, "runId")?;
    validate_studio_run_id(&run_id)?;
    let active = active_studio_agents()
        .lock()
        .ok()
        .and_then(|agents| agents.get(&run_id).cloned());
    let was_active = active.is_some();
    if let Some(active) = active {
        active.store(true, Ordering::Release);
    }
    Ok(json!({ "ok": true, "runId": run_id, "active": was_active, "status": "cancelled" }))
}

async fn studio_agent_stream_inner(
    websocket_url: String,
    authorization: String,
    run_id: String,
    payload: String,
    cancellation: Arc<AtomicBool>,
) -> RuntimeResult<Value> {
    let mut request = websocket_url.into_client_request().map_err(|error| {
        RuntimeError::new(format!("could not create Studio agent request: {error}"))
    })?;
    let header = HeaderValue::from_str(&authorization)
        .map_err(|_| RuntimeError::new("Studio agent authorization is invalid"))?;
    request.headers_mut().insert("Authorization", header);
    let request_value = serde_json::from_str::<Value>(&payload).unwrap_or_else(|_| json!({}));
    let request_id = request_value
        .get("requestId")
        .or_else(|| request_value.get("request_id"))
        .and_then(Value::as_str)
        .unwrap_or(&run_id)
        .to_string();
    let (mut socket, _) = connect_async(request).await.map_err(|error| {
        RuntimeError::new(format!("could not connect to Studio agent: {error}"))
    })?;
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|error| {
            RuntimeError::new(format!("could not send Studio agent request: {error}"))
        })?;
    let mut events = Vec::new();
    let mut total_event_bytes = 0usize;
    let mut answer = String::new();
    loop {
        let message = tokio::select! {
            _ = wait_for_studio_agent_cancellation(cancellation.clone()) => {
                let _ = socket.close(None).await;
                return Ok(studio_agent_result(false, &request_id, &answer, "studio_agent_cancelled", events));
            }
            message = socket.next() => message,
        };
        let Some(message) = message else {
            return Ok(studio_agent_result(
                false,
                &request_id,
                &answer,
                "studio_agent_closed",
                events,
            ));
        };
        let message = message.map_err(|error| {
            RuntimeError::new(format!("Studio agent connection failed: {error}"))
        })?;
        let text = match message {
            Message::Text(text) => text.to_string(),
            Message::Binary(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).await.map_err(|error| {
                    RuntimeError::new(format!("Studio agent ping failed: {error}"))
                })?;
                continue;
            }
            Message::Close(_) => {
                return Ok(studio_agent_result(
                    false,
                    &request_id,
                    &answer,
                    "studio_agent_closed",
                    events,
                ));
            }
            Message::Pong(_) | Message::Frame(_) => continue,
        };
        if text.len() > STUDIO_AGENT_MAX_EVENT_BYTES
            || total_event_bytes.saturating_add(text.len()) > STUDIO_AGENT_MAX_TOTAL_EVENT_BYTES
        {
            return Err(RuntimeError::new(
                "Studio agent event exceeds its size limit",
            ));
        }
        total_event_bytes = total_event_bytes.saturating_add(text.len());
        let value = serde_json::from_str::<Value>(&text)
            .map_err(|_| RuntimeError::new("Studio agent returned invalid JSON"))?;
        let event = value
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let content = value
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if answer.len().saturating_add(content.len()) > STUDIO_AGENT_MAX_ANSWER_BYTES {
            return Err(RuntimeError::new(
                "Studio agent answer exceeds its size limit",
            ));
        }
        answer.push_str(&content);
        if events.len() >= STUDIO_AGENT_MAX_EVENTS {
            return Err(RuntimeError::new(
                "Studio agent event count exceeds its limit",
            ));
        }
        events.push(json!({
            "event": event,
            "requestId": value.get("requestId").and_then(Value::as_str).unwrap_or(&request_id),
            "content": content,
            "payload": "",
        }));
        if event == "error" {
            let error = value
                .get("payload")
                .and_then(|payload| payload.get("error"))
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str)
                .unwrap_or("studio_agent_error")
                .to_string();
            return Ok(studio_agent_result(
                false,
                &request_id,
                &answer,
                &error,
                events,
            ));
        }
        if event == "done" {
            return Ok(studio_agent_result(true, &request_id, &answer, "", events));
        }
    }
}

async fn wait_for_studio_agent_cancellation(cancellation: Arc<AtomicBool>) {
    while !cancellation.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
}

fn studio_agent_result(
    ok: bool,
    request_id: &str,
    answer: &str,
    error: &str,
    events: Vec<Value>,
) -> Value {
    json!({
        "ok": ok,
        "requestId": request_id,
        "answer": answer,
        "error": error,
        "events": events,
    })
}

fn studio_agent_websocket_url(backend: &str, run_id: &str) -> RuntimeResult<String> {
    let mut url = reqwest::Url::parse(backend)
        .map_err(|_| RuntimeError::new("Studio backend URL is invalid"))?;
    if url.username() != "" || url.password().is_some() {
        return Err(RuntimeError::new(
            "Studio backend URL cannot contain credentials",
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| RuntimeError::new("Studio backend URL has no host"))?;
    if url.scheme() != "https" && !(url.scheme() == "http" && is_loopback_host(host)) {
        return Err(RuntimeError::new(
            "Studio agent requires HTTPS unless the backend is loopback",
        ));
    }
    let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    url.set_scheme(scheme)
        .map_err(|_| RuntimeError::new("Studio backend URL scheme is invalid"))?;
    let base_path = url.path().trim_end_matches('/');
    url.set_path(&format!("{base_path}/api/studio/runs/{run_id}/stream"));
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn validate_studio_authorization(value: &str) -> RuntimeResult<()> {
    if value.len() > 512 || !value.starts_with("Bearer ") || value[7..].trim().is_empty() {
        return Err(RuntimeError::new("Studio agent authorization is invalid"));
    }
    Ok(())
}

fn validate_studio_run_id(value: &str) -> RuntimeResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(RuntimeError::new("Studio agent run id is invalid"));
    }
    Ok(())
}

async fn studio_sketch(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let image = required_string(args, "image")?;
    let encoded = image
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(|| RuntimeError::new("sketch must be a PNG data URL"))?;
    if image.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new("sketch image exceeds its size limit"));
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|error| RuntimeError::new(format!("invalid sketch image: {error}")))?;
    if bytes.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new("sketch image exceeds its size limit"));
    }
    let directory = root.join(".dowe");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    tokio::fs::write(directory.join("latest-sketch.png"), bytes)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(json!(true))
}

async fn valid_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    Ok(json!(is_valid_dowe_project(&path).await))
}

async fn is_valid_dowe_project(path: &Path) -> bool {
    let Ok(metadata) = tokio::fs::symlink_metadata(path).await else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return false;
    }
    let main = path.join("main.dowe");
    let Ok(main_metadata) = tokio::fs::symlink_metadata(&main).await else {
        return false;
    };
    if main_metadata.file_type().is_symlink() || !main_metadata.is_file() {
        return false;
    }
    let Ok(root) = tokio::fs::canonicalize(path).await else {
        return false;
    };
    run_studio_blocking(move || dowe_compiler::inspect_project_capabilities(&root).is_ok())
        .await
        .unwrap_or(false)
}

async fn validated_dowe_project_root(value: &str) -> RuntimeResult<PathBuf> {
    let path = PathBuf::from(value);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeError::new(
            "selected application path must be a regular directory",
        ));
    }
    let root = tokio::fs::canonicalize(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if !is_valid_dowe_project(&root).await {
        return Err(RuntimeError::new(
            "selected folder does not contain a valid Dowe project",
        ));
    }
    Ok(root)
}

async fn empty_dowe_folder(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    Ok(json!(is_empty_directory(&path).await?))
}

async fn is_empty_directory(path: &Path) -> RuntimeResult<bool> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeError::new(
            "project path must be a regular directory",
        ));
    }
    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    Ok(entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
        .is_none())
}

async fn initialize_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    let template_name = args
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("blank");
    let template = ProjectTemplate::from_str(template_name)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(RuntimeError::new(
                "project path must not be a symbolic link",
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(RuntimeError::new("project path must be a directory"));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tokio::fs::create_dir_all(&path)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
        Err(error) => return Err(RuntimeError::new(error.to_string())),
    }
    let root = tokio::fs::canonicalize(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if !is_empty_directory(&root).await? {
        return Err(RuntimeError::new(
            "new Dowe projects must use an empty folder",
        ));
    }
    let initialized_root = root.clone();
    run_studio_blocking(move || init_project(&initialized_root, InitProjectOptions::new(template)))
        .await
        .map_err(|error| RuntimeError::new(format!("could not initialize Dowe project: {error}")))?
        .map_err(|error| {
            RuntimeError::new(format!("could not initialize Dowe project: {error}"))
        })?;
    Ok(json!(root.to_string_lossy().into_owned()))
}

async fn clone_dowe_repository(args: &Value) -> RuntimeResult<Value> {
    let url = normalized_github_repository_url(&required_string(args, "url")?)?;
    let requested_destination = PathBuf::from(required_string(args, "destination")?);
    if !is_empty_directory(&requested_destination).await? {
        return Err(RuntimeError::new(
            "repository imports require an empty destination folder",
        ));
    }
    let destination = tokio::fs::canonicalize(&requested_destination)
        .await
        .map_err(|error| {
            RuntimeError::new(format!("could not inspect repository folder: {error}"))
        })?;
    let destination_arg = destination.to_string_lossy().into_owned();
    let output = tokio::time::timeout(
        Duration::from_secs(120),
        tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--no-tags",
                "--single-branch",
                &url,
                &destination_arg,
            ])
            .env("GIT_TERMINAL_PROMPT", "0")
            .output(),
    )
    .await
    .map_err(|_| RuntimeError::new("GitHub repository clone timed out"))?
    .map_err(|error| RuntimeError::new(format!("could not start git clone: {error}")))?;
    if !output.status.success() {
        let details = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        let details =
            truncate_studio_text(&sanitize_studio_text(details.trim(), &destination), 512);
        let details = if details.is_empty() {
            "git clone returned a failure".to_string()
        } else {
            details
        };
        return Err(RuntimeError::new(format!(
            "could not clone GitHub repository: {details}"
        )));
    }
    if !is_valid_dowe_project(&destination).await {
        return Err(RuntimeError::new(
            "the GitHub repository does not contain a valid Dowe project",
        ));
    }
    Ok(json!(destination.to_string_lossy().into_owned()))
}

fn normalized_github_repository_url(value: &str) -> RuntimeResult<String> {
    let parsed = reqwest::Url::parse(value.trim())
        .map_err(|_| RuntimeError::new("GitHub repository URL is invalid"))?;
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if parsed.scheme() != "https"
        || !matches!(host.as_str(), "github.com" | "www.github.com")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(RuntimeError::new(
            "repository imports accept public HTTPS GitHub URLs only",
        ));
    }
    let parts = parsed
        .path_segments()
        .ok_or_else(|| RuntimeError::new("GitHub repository URL has no repository path"))?
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(RuntimeError::new(
            "GitHub repository URL must contain an owner and repository",
        ));
    }
    let repository = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    if !valid_github_segment(parts[0]) || !valid_github_segment(repository) {
        return Err(RuntimeError::new(
            "GitHub repository URL contains invalid names",
        ));
    }
    Ok(format!(
        "https://github.com/{}/{}.git",
        parts[0], repository
    ))
}

fn valid_github_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Clone)]
struct StudioChangeSpec {
    path: String,
    operation: String,
    owner: String,
    content: String,
}

async fn stage_studio_changes(args: &Value) -> RuntimeResult<Value> {
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

async fn inspect_studio_changes(args: &Value) -> RuntimeResult<Value> {
    let root = canonical_studio_root(&required_string(args, "root")?).await?;
    let stage_id = required_stage_id(args)?;
    ensure_studio_stage_directory(&root).await?;
    let stage_root = studio_stage_root(&root, &stage_id)?;
    let metadata = read_stage_metadata(&stage_root).await?;
    Ok(safe_stage_metadata(metadata))
}

async fn apply_studio_changes(args: &Value) -> RuntimeResult<Value> {
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

async fn reject_studio_changes(args: &Value) -> RuntimeResult<Value> {
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

async fn rollback_studio_changes(args: &Value) -> RuntimeResult<Value> {
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

async fn inspect_dowe_project(args: &Value) -> RuntimeResult<Value> {
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

const STUDIO_CONTEXT_PROTOCOL_VERSION: u64 = 1;
const STUDIO_CONTEXT_MAX_FILES: usize = 24;
const STUDIO_CONTEXT_MAX_FILE_BYTES: usize = 64 * 1024;
const STUDIO_CONTEXT_MAX_TOTAL_BYTES: usize = 256 * 1024;
const STUDIO_CONTEXT_MAX_SOURCE_READ_BYTES: usize = 2 * 1024 * 1024;
const STUDIO_CONTEXT_MAX_QUERY_BYTES: usize = 4096;
const STUDIO_CONTEXT_MAX_IMAGE_BYTES: usize = 512 * 1024;
const STUDIO_CONTEXT_MAX_IMPORTS: usize = 64;
const STUDIO_CONTEXT_MAX_DECLARATIONS: usize = 128;
const STUDIO_CONTEXT_MAX_GRAPH_NODES: usize = 24;
const STUDIO_CONTEXT_MAX_DIRECTORIES: usize = 4096;
const STUDIO_CONTEXT_MAX_SOURCE_FILES: usize = 4096;
const STUDIO_STAGE_MAX_FILES: usize = 24;
const STUDIO_STAGE_MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const STUDIO_STAGE_MAX_TOTAL_BYTES: usize = 512 * 1024;
const STUDIO_STAGE_MAX_PLAN_BYTES: usize = 768 * 1024;
const STUDIO_STAGE_MAX_DIFF_BYTES: usize = 256 * 1024;
const STUDIO_STAGE_MAX_PATCH_BYTES: usize = 16 * 1024;
const STUDIO_STAGE_MAX_TEST_OUTPUT_BYTES: usize = 128 * 1024;
const STUDIO_STAGE_DIRECTORY: &str = "studio-staging";

#[derive(Clone)]
struct StudioSourceEntry {
    relative: String,
    path: PathBuf,
    size: u64,
}

struct StudioSourceContent {
    content: String,
    digest: String,
}

async fn prepare_studio_context(args: &Value) -> RuntimeResult<Value> {
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

async fn collect_studio_sources(
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

async fn read_studio_source(entry: &StudioSourceEntry) -> RuntimeResult<StudioSourceContent> {
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

fn validate_studio_image(image: &str) -> RuntimeResult<()> {
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

fn context_selected_paths(args: &Value) -> RuntimeResult<Vec<String>> {
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

fn studio_context_query_terms(query: &str) -> Vec<String> {
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

fn studio_context_path_score(path: &str, terms: &[String]) -> usize {
    let lower = path.to_ascii_lowercase();
    terms
        .iter()
        .map(|term| usize::from(lower.contains(term)) * 100)
        .sum()
}

fn infer_studio_profile(
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

fn infer_studio_request_type(query: &str, has_image: bool) -> &'static str {
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

fn contains_studio_term(value: &str, terms: &[&str]) -> bool {
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

fn parse_studio_imports(source: &str, importer: &str) -> Vec<String> {
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

fn normalize_studio_source_path(path: &Path) -> Option<String> {
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

fn parse_studio_declarations(source: &str, path: &str) -> Vec<String> {
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

fn truncate_studio_text(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

async fn cached_studio_analysis(root: &Path, workspace_fingerprint: &str) -> (Vec<Value>, Value) {
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

async fn studio_compile_diagnostics(root: PathBuf) -> Vec<Value> {
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

async fn studio_codegraph_summary(root: PathBuf) -> Value {
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

async fn run_studio_blocking<T, F>(work: F) -> Result<T, String>
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

fn sanitize_studio_text(value: &str, root: &Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    value
        .replace(&root, "<workspace>")
        .replace(&root.replace('/', "\\"), "<workspace>")
}

fn safe_studio_metadata_path(value: &str, root: &Path) -> Option<String> {
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

fn visible_studio_path(path: &str) -> bool {
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

fn studio_workspace_id(root: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(b"dowe-studio-workspace:");
    digest.update(root.to_string_lossy().as_bytes());
    hex_digest(&digest.finalize())
}

fn hex_digest(digest: &[u8]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

async fn pick_directory() -> RuntimeResult<Value> {
    let path = tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            command_output("osascript", &["-e", "POSIX path of (choose folder)"])
        }
        #[cfg(target_os = "windows")]
        {
            command_output(
                "powershell",
                &[
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.FolderBrowserDialog; if($d.ShowDialog() -eq 'OK'){ $d.SelectedPath }",
                ],
            )
        }
        #[cfg(target_os = "linux")]
        {
            command_output("zenity", &["--file-selection", "--directory"])
                .or_else(|| command_output("kdialog", &["--getexistingdirectory", "."]))
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            None
        }
    })
    .await
    .map_err(|_| RuntimeError::new("directory picker task failed"))?;
    Ok(json!(path.unwrap_or_default()))
}

async fn create_folder(args: &Value) -> RuntimeResult<Value> {
    let parent = required_string(args, "parent")?;
    let name = required_string(args, "name")?;
    validate_folder_name(&name)?;
    let parent = PathBuf::from(parent);
    let destination = parent.join(&name);
    tokio::fs::create_dir(&destination)
        .await
        .map_err(|error| RuntimeError::new(format!("could not create folder: {error}")))?;
    Ok(json!(destination.to_string_lossy().into_owned()))
}

async fn list_project_files(args: &Value) -> RuntimeResult<Value> {
    let root = PathBuf::from(required_string(args, "root")?);
    let root = tokio::fs::canonicalize(&root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    collect_project_tree(&root, &root).await
}

async fn collect_project_tree(root: &Path, directory: &Path) -> RuntimeResult<Value> {
    let mut entries = tokio::fs::read_dir(directory)
        .await
        .map_err(|error| RuntimeError::new(format!("could not list project files: {error}")))?;
    let mut files = Vec::new();
    let mut folders = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project entry: {error}")))?
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.')
            || matches!(name.as_str(), "AGENTS.md" | "CLAUDE.md")
            || name == ".dowe"
        {
            continue;
        }
        let path = entry.path();
        let metadata = tokio::fs::symlink_metadata(&path).await.map_err(|error| {
            RuntimeError::new(format!("could not read project metadata: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if metadata.is_dir() {
            if should_skip_project_directory(&name) {
                continue;
            }
            let tree = Box::pin(collect_project_tree(root, &path)).await?;
            folders.push(json!({
                "id": relative,
                "name": name,
                "path": relative,
                "files": tree.get("files").cloned().unwrap_or_else(|| json!([])),
                "folders": tree.get("folders").cloned().unwrap_or_else(|| json!([])),
            }));
        } else if metadata.is_file() {
            files.push(json!({
                "id": relative,
                "name": name,
                "path": relative,
                "icon": project_file_icon(&name),
            }));
        }
    }
    files.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    folders.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(json!({ "files": files, "folders": folders }))
}

fn should_skip_project_directory(name: &str) -> bool {
    matches!(
        name,
        "build" | "coverage" | "dist" | "node_modules" | "out" | "target"
    )
}

fn project_file_icon(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("dowe") | Some("rs") | Some("ts") | Some("js") | Some("swift") | Some("kt") => {
            "code-file"
        }
        Some("json") | Some("toml") | Some("yaml") | Some("yml") | Some("md") | Some("txt") => {
            "file-text"
        }
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("svg") => "gallery",
        _ => "file",
    }
}

async fn read_project_file(args: &Value) -> RuntimeResult<Value> {
    let root = tokio::fs::canonicalize(PathBuf::from(required_string(args, "root")?))
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    let relative = required_string(args, "path")?;
    validate_project_relative_path(&relative)?;
    let path = root.join(&relative);
    let mut current = root.clone();
    for component in Path::new(&relative).components() {
        let Component::Normal(name) = component else {
            return Err(RuntimeError::new("project file path must be relative"));
        };
        current.push(name);
        let metadata = tokio::fs::symlink_metadata(&current)
            .await
            .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(RuntimeError::new(
                "project file path cannot contain symlinks",
            ));
        }
    }
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
    if !metadata.is_file() {
        return Err(RuntimeError::new("project path is not a file"));
    }
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(RuntimeError::new("project file exceeds the 2 MiB limit"));
    }
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
    Ok(json!(content))
}

const MAX_PROJECT_FILE_BYTES: u64 = 2 * 1024 * 1024;

fn validate_project_relative_path(value: &str) -> RuntimeResult<()> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(RuntimeError::new("project file path must be relative"));
    }
    Ok(())
}

async fn write_project_file(args: &Value) -> RuntimeResult<Value> {
    let root = tokio::fs::canonicalize(PathBuf::from(required_string(args, "root")?))
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    let relative = required_string(args, "path")?;
    validate_project_relative_path(&relative)?;
    let content = args
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| RuntimeError::new("native directory argument `content` is required"))?;
    if content.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(RuntimeError::new("project file exceeds the 2 MiB limit"));
    }
    let path = root.join(&relative);
    let mut current = root.clone();
    for component in Path::new(&relative).components() {
        let Component::Normal(name) = component else {
            return Err(RuntimeError::new("project file path must be relative"));
        };
        current.push(name);
        let metadata = tokio::fs::symlink_metadata(&current)
            .await
            .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(RuntimeError::new(
                "project file path cannot contain symlinks",
            ));
        }
    }
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
    if !metadata.is_file() {
        return Err(RuntimeError::new("project path is not a file"));
    }
    tokio::fs::write(path, content)
        .await
        .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
    Ok(json!(true))
}

fn parse_studio_change_plan(value: &str) -> RuntimeResult<Vec<StudioChangeSpec>> {
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

fn studio_change_owner(path: &str) -> Option<&'static str> {
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

async fn canonical_studio_root(value: &str) -> RuntimeResult<PathBuf> {
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

async fn ensure_studio_stage_directory(root: &Path) -> RuntimeResult<()> {
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

fn studio_stage_root(root: &Path, stage_id: &str) -> RuntimeResult<PathBuf> {
    validate_studio_run_id(stage_id)?;
    Ok(root
        .join(".dowe")
        .join(STUDIO_STAGE_DIRECTORY)
        .join(stage_id))
}

fn required_stage_id(args: &Value) -> RuntimeResult<String> {
    let value = required_string(args, "stageId")?;
    validate_studio_run_id(&value)?;
    Ok(value)
}

async fn copy_studio_workspace(root: &Path, destination: &Path) -> RuntimeResult<()> {
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

async fn copy_studio_directory(source: &Path, destination: PathBuf) -> RuntimeResult<()> {
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

async fn snapshot_studio_changes(
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

async fn apply_changes_to_stage(
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

fn changes_from_metadata(metadata: &Value) -> RuntimeResult<Vec<StudioChangeSpec>> {
    let value = metadata
        .get("changePlan")
        .ok_or_else(|| RuntimeError::new("Studio stage change plan is missing"))?;
    let serialized = serde_json::to_string(value)
        .map_err(|_| RuntimeError::new("Studio stage change plan cannot be serialized"))?;
    parse_studio_change_plan(&serialized)
}

async fn studio_target_path(
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

async fn write_studio_file_atomic(path: &Path, content: &[u8]) -> RuntimeResult<()> {
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

async fn write_stage_metadata(stage_root: &Path, metadata: &Value) -> RuntimeResult<()> {
    let serialized = serde_json::to_vec(metadata)
        .map_err(|_| RuntimeError::new("Studio stage metadata cannot be serialized"))?;
    write_studio_metadata_atomic(&stage_root.join("metadata.json"), &serialized).await
}

async fn write_studio_metadata_atomic(path: &Path, content: &[u8]) -> RuntimeResult<()> {
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

async fn read_stage_metadata(stage_root: &Path) -> RuntimeResult<Value> {
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

fn safe_stage_metadata(mut metadata: Value) -> Value {
    if let Some(object) = metadata.as_object_mut() {
        object.remove("workspaceRoot");
        object.remove("stagedRoot");
    }
    metadata
}

async fn studio_context_fingerprint(
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

async fn studio_context_fingerprint_from_metadata(
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

async fn full_studio_source_fingerprint(root: &Path) -> RuntimeResult<String> {
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

async fn build_studio_diff(
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

async fn apply_one_studio_change(
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

async fn restore_studio_changes(
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

async fn read_optional_studio_file(root: &Path, relative: &str) -> RuntimeResult<Option<Vec<u8>>> {
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

fn studio_unified_diff(path: &str, before: &[u8], after: &[u8]) -> (String, usize, usize) {
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

async fn run_studio_tests(root: PathBuf) -> Value {
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

async fn studio_visual_validation(root: &Path, changes: &[StudioChangeSpec]) -> Value {
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

async fn list_folders(args: &Value) -> RuntimeResult<Value> {
    let parent = PathBuf::from(required_string(args, "parent")?);
    let mut entries = tokio::fs::read_dir(&parent)
        .await
        .map_err(|error| RuntimeError::new(format!("could not list folders: {error}")))?;
    let mut folders = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(format!("could not read folder entry: {error}")))?
    {
        let metadata = entry.metadata().await.map_err(|error| {
            RuntimeError::new(format!("could not read folder metadata: {error}"))
        })?;
        if metadata.is_dir() {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    folders.sort_unstable();
    Ok(json!(folders))
}

fn required_string(args: &Value, name: &str) -> RuntimeResult<String> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| RuntimeError::new(format!("native directory argument `{name}` is required")))
}

fn validate_folder_name(name: &str) -> RuntimeResult<()> {
    let path = Path::new(name);
    if name == "." || name == ".." || path.is_absolute() || path.components().count() != 1 {
        return Err(RuntimeError::new(
            "folder name must be a single relative directory name",
        ));
    }
    Ok(())
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_studio_changes, cancel_studio_agent, create_folder, delete_studio_app,
        full_studio_source_fingerprint, initialize_dowe_project, inspect_dowe_project,
        list_folders, list_project_files, normalized_github_repository_url, prepare_studio_context,
        read_project_file, register_studio_app, rollback_studio_changes, save_studio_app,
        stage_studio_changes, studio_agent_stream, studio_agent_websocket_url, studio_unified_diff,
        validate_folder_name, validate_studio_run_id, write_project_file,
    };
    use dowe_database::{init_database, open_database};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn creates_and_lists_folders() {
        let temp = TempDir::new().expect("tempdir");
        let created = create_folder(&json!({ "parent": temp.path(), "name": "workspace" }))
            .await
            .expect("create");
        assert_eq!(
            created.as_str().map(|value| value.ends_with("workspace")),
            Some(true)
        );
        let folders = list_folders(&json!({ "parent": temp.path() }))
            .await
            .expect("list");
        assert_eq!(folders, json!(["workspace"]));
    }

    #[test]
    fn accepts_only_public_github_repository_urls() {
        assert_eq!(
            normalized_github_repository_url("https://github.com/dowe-lang/example")
                .expect("github url"),
            "https://github.com/dowe-lang/example.git"
        );
        assert_eq!(
            normalized_github_repository_url("https://www.github.com/dowe-lang/example.git")
                .expect("www github url"),
            "https://github.com/dowe-lang/example.git"
        );
        assert!(normalized_github_repository_url("git@github.com:dowe-lang/example.git").is_err());
        assert!(normalized_github_repository_url("https://example.com/dowe-lang/example").is_err());
        assert!(
            normalized_github_repository_url("https://github.com/dowe-lang/example?token=secret")
                .is_err()
        );
    }

    #[tokio::test]
    async fn classifies_only_structurally_valid_dowe_projects() {
        let temp = TempDir::new().expect("tempdir");
        assert_eq!(
            inspect_dowe_project(&json!({ "path": temp.path() }))
                .await
                .expect("empty"),
            "empty"
        );
        fs::write(temp.path().join("main.dowe"), "not a main block\n").expect("invalid main");
        assert_eq!(
            inspect_dowe_project(&json!({ "path": temp.path() }))
                .await
                .expect("invalid"),
            "invalid"
        );
    }

    #[tokio::test]
    async fn initializes_a_blank_project_without_agents() {
        let temp = TempDir::new().expect("project");
        let result = initialize_dowe_project(&json!({ "path": temp.path(), "template": "blank" }))
            .await
            .expect("initialize");
        assert_eq!(
            result,
            json!(
                temp.path()
                    .canonicalize()
                    .expect("canonical project")
                    .to_string_lossy()
                    .to_string()
            )
        );
        assert!(temp.path().join("main.dowe").is_file());
        assert!(temp.path().join("views/pages/home.dowe").is_file());
        assert!(!temp.path().join("AGENTS.md").exists());
        assert!(!temp.path().join(".agents").exists());
    }

    #[tokio::test]
    async fn registers_an_app_with_its_source() {
        let studio = TempDir::new().expect("studio root");
        let app = TempDir::new().expect("app root");
        fs::write(app.path().join("main.dowe"), "main\n").expect("main");
        init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

        let inserted = register_studio_app(
            studio.path(),
            &json!({ "path": app.path(), "name": "Imported app", "source": "github" }),
        )
        .await
        .expect("register");

        assert_eq!(inserted["source"], "github");
        assert_eq!(
            open_database(studio.path(), "dowe-studio-apps")
                .expect("database")
                .records("workspace_apps")
                .expect("records")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn deletes_only_the_local_app_record() {
        let studio = TempDir::new().expect("studio root");
        let app = TempDir::new().expect("app root");
        fs::write(app.path().join("main.dowe"), "main\n").expect("main");
        init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

        let inserted = save_studio_app(
            studio.path(),
            &json!({ "path": app.path(), "name": "Test app" }),
        )
        .await
        .expect("save");
        let id = inserted
            .get("id")
            .and_then(|value| value.as_str())
            .expect("id");

        let deleted = delete_studio_app(studio.path(), &json!({ "id": id }))
            .await
            .expect("delete");

        assert_eq!(deleted, json!({ "changed": 1 }));
        assert!(app.path().join("main.dowe").is_file());
        let database = open_database(studio.path(), "dowe-studio-apps").expect("database");
        assert!(
            database
                .records("workspace_apps")
                .expect("records")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn lists_and_reads_project_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir(temp.path().join("views")).expect("views");
        fs::create_dir(temp.path().join("target")).expect("target");
        fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
        fs::write(temp.path().join("views/editor.dowe"), "page editor\n").expect("editor");
        fs::write(temp.path().join("target/generated"), "generated\n").expect("generated");
        fs::write(temp.path().join(".hidden"), "hidden\n").expect("hidden");
        fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");

        let files = list_project_files(&json!({ "root": temp.path() }))
            .await
            .expect("list");
        assert_eq!(
            files,
            json!({
                "files": [{ "id":"main.dowe", "name":"main.dowe", "path":"main.dowe", "icon":"code-file" }],
                "folders": [{
                    "id":"views",
                    "name":"views",
                    "path":"views",
                    "files": [{ "id":"views/editor.dowe", "name":"editor.dowe", "path":"views/editor.dowe", "icon":"code-file" }],
                    "folders": []
                }]
            })
        );

        let content = read_project_file(&json!({
            "root": temp.path(),
            "path": "views/editor.dowe"
        }))
        .await
        .expect("read");
        assert_eq!(content, json!("page editor\n"));
        assert!(
            read_project_file(&json!({
                "root": temp.path(),
                "path": "../outside"
            }))
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn prepares_bounded_context_without_private_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("views/components")).expect("components");
        fs::create_dir_all(temp.path().join("server")).expect("server");
        fs::create_dir(temp.path().join("target")).expect("target");
        fs::write(
            temp.path().join("main.dowe"),
            "import viewRoutes from \"@/views/routes\"\n\nmain\n  views:viewRoutes\n",
        )
        .expect("main");
        fs::write(
            temp.path().join("views/routes.dowe"),
            "import Card from \"@/views/components/card\"\n\nviews viewRoutes\n  group path:\"/\" layout:Card\n",
        )
        .expect("routes");
        fs::write(
            temp.path().join("views/components/card.dowe"),
            "component Card\n  Text\n    \"Card\"\n",
        )
        .expect("component");
        fs::write(temp.path().join("server/private.dowe"), "fn private\n").expect("server");
        fs::write(temp.path().join(".env"), "OPENROUTER_API_KEY=secret\n").expect("env");
        fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");
        fs::write(temp.path().join("target/generated.dowe"), "generated\n").expect("generated");

        let context = prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "component",
            "selectedPaths": ["views/routes.dowe"]
        }))
        .await
        .expect("context");
        assert_eq!(context["protocolVersion"], 1);
        assert_eq!(context["profile"], "views");
        assert_eq!(context["requestType"], "clarify");
        assert_eq!(context["mode"], "dowe");
        assert_eq!(context["compilerVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(context["workspaceId"].as_str().map(str::len), Some(64));
        let files = context["files"].as_array().expect("files");
        let paths = files
            .iter()
            .filter_map(|file| file["path"].as_str())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"main.dowe"));
        assert!(paths.contains(&"views/routes.dowe"));
        assert!(paths.contains(&"views/components/card.dowe"));
        assert!(!paths.iter().any(|path| path.contains("target")));
        assert!(!paths.iter().any(|path| path.contains("AGENTS")));
        assert!(context["imports"].as_array().is_some_and(|imports| {
            imports
                .iter()
                .any(|value| value == "views/components/card.dowe")
        }));
        assert!(
            context["sourceFingerprint"]
                .as_str()
                .is_some_and(|value| value.len() == 64)
        );
        assert!(
            context["workspaceFingerprint"]
                .as_str()
                .is_some_and(|value| value.len() == 64)
        );
        assert!(temp.path().join(".dowe/studio-context-cache").is_dir());
        assert!(
            !context
                .to_string()
                .contains(&temp.path().to_string_lossy().to_string())
        );
        let compact_context = prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "component",
            "selectedPaths": [],
            "detail": "compact"
        }))
        .await
        .expect("compact context");
        assert_eq!(compact_context["detail"], "compact");
        assert_eq!(compact_context["image"], "");
        assert!(
            compact_context["files"]
                .as_array()
                .is_some_and(|files| { files.iter().all(|file| file["content"] == "") })
        );
        let read_context = prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "inspect context",
            "selectedPaths": [],
            "detail": "compact"
        }))
        .await
        .expect("read context");
        assert_eq!(read_context["requestType"], "context_read");
        let image_context = prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": [],
            "image": "data:image/png;base64,AA=="
        }))
        .await
        .expect("image context");
        assert_eq!(image_context["profile"], "viewReference");
        assert_eq!(image_context["requestType"], "vision_ui");
        assert_eq!(image_context["image"], "data:image/png;base64,AA==");
        assert!(
            prepare_studio_context(&json!({
                "path": temp.path(),
                "query": "",
                "selectedPaths": [],
                "image": "data:image/png;base64:not-base64"
            }))
            .await
            .is_err()
        );
        assert!(
            prepare_studio_context(&json!({
                "path": temp.path(),
                "query": "",
                "selectedPaths": ["../outside.dowe"]
            }))
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn fingerprints_deep_source_tree_without_recursive_directory_calls() {
        let temp = TempDir::new().expect("tempdir");
        let mut directory = temp.path().to_path_buf();
        for _ in 0..256 {
            directory = directory.join("d");
            fs::create_dir(&directory).expect("directory");
        }
        fs::write(directory.join("deep.dowe"), "page deep\n").expect("source");

        let fingerprint = full_studio_source_fingerprint(temp.path())
            .await
            .expect("fingerprint");

        assert_eq!(fingerprint.len(), 64);
    }

    #[tokio::test]
    async fn writes_project_file_with_relative_path() {
        let temp = TempDir::new().expect("tempdir");
        fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
        fs::write(temp.path().join("views.dowe"), "old\n").expect("file");

        let saved = write_project_file(&json!({
            "root": temp.path(),
            "path": "views.dowe",
            "content": "new source\n"
        }))
        .await
        .expect("write");

        assert_eq!(saved, json!(true));
        assert_eq!(
            fs::read_to_string(temp.path().join("views.dowe")).expect("content"),
            "new source\n"
        );
        assert!(
            write_project_file(&json!({
                "root": temp.path(),
                "path": "../outside",
                "content": "unsafe"
            }))
            .await
            .is_err()
        );
        #[cfg(unix)]
        {
            let outside = TempDir::new().expect("outside tempdir");
            fs::write(outside.path().join("source.dowe"), "outside\n").expect("outside file");
            std::os::unix::fs::symlink(
                outside.path().join("source.dowe"),
                temp.path().join("link.dowe"),
            )
            .expect("symlink");
            assert!(
                write_project_file(&json!({
                    "root": temp.path(),
                    "path": "link.dowe",
                    "content": "unsafe"
                }))
                .await
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn streams_studio_agent_events_through_native_host() {
        use axum::Router;
        use axum::extract::ws::{Message, WebSocketUpgrade};
        use axum::routing::get;
        use futures_util::StreamExt;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let app = Router::new().route(
            "/api/studio/runs/run_1/stream",
            get(|upgrade: WebSocketUpgrade| async move {
                upgrade.on_upgrade(|mut socket| async move {
                    let Some(Ok(Message::Text(_))) = socket.next().await else {
                        return;
                    };
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"started", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"delta", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "content":"done", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                    let _ = socket
                        .send(Message::Text(
                            json!({ "event":"done", "requestId":"request-1", "requestType":"implementation", "model":"minimax/minimax-m3", "payload":{} })
                                .to_string()
                                .into(),
                        ))
                        .await;
                })
            }),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });

        let result = studio_agent_stream(&json!({
            "backend": format!("http://{address}"),
            "authorization": "Bearer test-token",
            "runId": "run_1",
            "payload": "{\"requestId\":\"request-1\"}",
        }))
        .await
        .expect("agent stream");
        server.abort();

        assert_eq!(result["ok"], true);
        assert_eq!(result["requestId"], "request-1");
        assert_eq!(result["answer"], "done");
        assert_eq!(result["events"].as_array().map(Vec::len), Some(3));
        assert!(result["events"][0].get("model").is_none());
        assert!(result["events"][0].get("requestType").is_none());
        assert_eq!(result["events"][1]["payload"], "");
    }

    #[tokio::test]
    async fn cancellation_is_safe_when_no_agent_is_active() {
        let result = cancel_studio_agent(&json!({ "runId": "inactive-run" }))
            .await
            .expect("cancel");
        assert_eq!(result["ok"], true);
        assert_eq!(result["active"], false);
        assert_eq!(result["status"], "cancelled");
    }

    #[tokio::test]
    async fn cancellation_closes_an_active_studio_agent_socket() {
        use axum::Router;
        use axum::extract::ws::{Message, WebSocketUpgrade};
        use axum::routing::get;
        use futures_util::StreamExt;
        use std::time::Duration;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let app = Router::new().route(
            "/api/studio/runs/run_cancel/stream",
            get(|upgrade: WebSocketUpgrade| async move {
                upgrade.on_upgrade(|mut socket| async move {
                    let _ = socket.next().await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let _ = socket.send(Message::Close(None)).await;
                })
            }),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });
        let request_args = json!({
            "backend": format!("http://{address}"),
            "authorization": "Bearer test-token",
            "runId": "run_cancel",
            "payload": "{\"requestId\":\"request-cancel\"}",
        });
        let request = tokio::spawn(async move { studio_agent_stream(&request_args).await });
        tokio::time::sleep(Duration::from_millis(100)).await;
        let cancellation = cancel_studio_agent(&json!({ "runId": "run_cancel" }))
            .await
            .expect("cancel");
        let result = tokio::time::timeout(Duration::from_secs(2), request)
            .await
            .expect("cancelled agent")
            .expect("agent task")
            .expect("agent result");
        server.abort();

        assert_eq!(cancellation["active"], true);
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"], "studio_agent_cancelled");
    }

    #[test]
    fn validates_studio_agent_urls_and_run_ids() {
        assert_eq!(
            studio_agent_websocket_url("http://127.0.0.1:7755", "run_1").expect("loopback"),
            "ws://127.0.0.1:7755/api/studio/runs/run_1/stream"
        );
        assert_eq!(
            studio_agent_websocket_url("https://studio.example.com/base", "run-1")
                .expect("secure backend"),
            "wss://studio.example.com/base/api/studio/runs/run-1/stream"
        );
        assert!(studio_agent_websocket_url("http://studio.example.com", "run-1").is_err());
        assert!(validate_studio_run_id("run/1").is_err());
        assert!(validate_studio_run_id("run-1").is_ok());
    }

    #[test]
    fn rejects_nested_folder_names() {
        assert!(validate_folder_name("workspace").is_ok());
        assert!(validate_folder_name("../workspace").is_err());
        assert!(validate_folder_name("workspace/child").is_err());
    }

    #[test]
    fn validates_change_ownership_and_bounds_diff() {
        let plan = r#"{"files":[{"path":"views/pages/home.dowe","owner":"views","operation":"upsert","content":"page Home\n"}]}"#;
        assert_eq!(
            super::parse_studio_change_plan(plan).expect("plan").len(),
            1
        );
        let invalid = r#"{"files":[{"path":"server/handlers/home.dowe","owner":"views","content":"handler home\n"}]}"#;
        assert!(super::parse_studio_change_plan(invalid).is_err());
        let (patch, added, removed) = studio_unified_diff("main.dowe", b"old\n", b"new\n");
        assert!(patch.contains("--- a/main.dowe"));
        assert_eq!((added, removed), (1, 1));
    }

    #[tokio::test]
    async fn stages_applies_and_rolls_back_an_authorized_change() {
        let temp = TempDir::new().expect("workspace");
        fs::write(
            temp.path().join("main.dowe"),
            "main\n  app name:\"Before\" bundle:\"dev.dowe.before\"\n",
        )
        .expect("main");
        let context = prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": [],
            "detail": "compact"
        }))
        .await
        .expect("context");
        let fingerprint = context["sourceFingerprint"].as_str().expect("fingerprint");
        let plan = json!({
            "files": [{
                "path": "main.dowe",
                "owner": "core",
                "operation": "upsert",
                "content": "main\n  app name:\"After\" bundle:\"dev.dowe.after\"\n"
            }]
        });
        assert!(
            stage_studio_changes(&json!({
                "root": temp.path(),
                "runId": "run-stage-1",
                "sourceFingerprint": "wrong",
                "query": "",
                "selectedPaths": [],
                "changePlan": plan.to_string()
            }))
            .await
            .is_err()
        );
        let staged = stage_studio_changes(&json!({
            "root": temp.path(),
            "runId": "run-stage-1",
            "sourceFingerprint": fingerprint,
            "query": "",
            "selectedPaths": [],
            "changePlan": plan.to_string()
        }))
        .await
        .expect("stage");
        assert_eq!(staged["ok"], true);
        assert_eq!(staged["status"], "staged");
        assert!(
            staged["validation"]["compiler"]["ok"]
                .as_bool()
                .unwrap_or(false)
        );
        let stage_id = staged["stageId"].as_str().expect("stage id");
        let applied = apply_studio_changes(&json!({
            "root": temp.path(),
            "stageId": stage_id,
            "sourceFingerprint": fingerprint
        }))
        .await
        .expect("apply");
        assert_eq!(applied["status"], "applied");
        assert_eq!(
            apply_studio_changes(&json!({
                "root": temp.path(),
                "stageId": stage_id,
                "sourceFingerprint": fingerprint
            }))
            .await
            .expect("idempotent apply")["status"],
            "applied"
        );
        assert!(
            fs::read_to_string(temp.path().join("main.dowe"))
                .expect("updated")
                .contains("After")
        );
        let rolled_back = rollback_studio_changes(&json!({
            "root": temp.path(),
            "stageId": stage_id
        }))
        .await
        .expect("rollback");
        assert_eq!(rolled_back["status"], "rolled_back");
        assert!(
            fs::read_to_string(temp.path().join("main.dowe"))
                .expect("restored")
                .contains("Before")
        );
        assert_eq!(
            rollback_studio_changes(&json!({
                "root": temp.path(),
                "stageId": stage_id
            }))
            .await
            .expect("idempotent rollback")["status"],
            "rolled_back"
        );
    }
}
