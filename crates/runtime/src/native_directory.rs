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
        "createFolder" => project_files::create_folder(args).await,
        "listFolders" => project_files::list_folders(args).await,
        "listProjectFiles" => project_files::list_project_files(args).await,
        "readProjectFile" => project_files::read_project_file(args).await,
        "writeProjectFile" => project_files::write_project_file(args).await,
        "startStudioPreview" => start_studio_preview(args).await,
        "stopStudioPreview" => stop_studio_preview(args).await,
        "studioEvent" => studio_event(args).await,
        "studioAgentStream" => studio_agent::studio_agent_stream(args).await,
        "cancelStudioAgent" => studio_agent::cancel_studio_agent(args).await,
        "studioSketch" => project_files::studio_sketch(root, args).await,
        "listStudioApps" => studio_apps::list_studio_apps(root).await,
        "saveStudioApp" => studio_apps::save_studio_app(root, args).await,
        "registerStudioApp" => studio_apps::register_studio_app(root, args).await,
        "deleteStudioApp" => studio_apps::delete_studio_app(root, args).await,
        "inspectDoweProject" => studio_context::inspect_dowe_project(args).await,
        "prepareStudioContext" => studio_context::prepare_studio_context(args).await,
        "validDoweProject" => project_files::valid_dowe_project(args).await,
        "emptyDoweFolder" => project_files::empty_dowe_folder(args).await,
        "initializeDoweProject" => project_files::initialize_dowe_project(args).await,
        "cloneDoweRepository" => project_files::clone_dowe_repository(args).await,
        "stageStudioChanges" => studio_changes::stage_studio_changes(args).await,
        "inspectStudioChanges" => studio_changes::inspect_studio_changes(args).await,
        "applyStudioChanges" => studio_changes::apply_studio_changes(args).await,
        "rejectStudioChanges" => studio_changes::reject_studio_changes(args).await,
        "rollbackStudioChanges" => studio_changes::rollback_studio_changes(args).await,
        _ => return None,
    };
    Some(result)
}

mod project_files;
mod studio_agent;
mod studio_apps;
mod studio_changes;
mod studio_context;

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
mod tests;
