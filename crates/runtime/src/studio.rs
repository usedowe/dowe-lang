use crate::dev::{RunningDevSession, start_studio_session};
use crate::{RuntimeError, RuntimeResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

struct ManagedPreview {
    root: std::path::PathBuf,
    session: RunningDevSession,
    stop_watch: oneshot::Sender<()>,
    watch_task: JoinHandle<RuntimeResult<()>>,
}

static PREVIEWS: OnceLock<Mutex<HashMap<String, ManagedPreview>>> = OnceLock::new();

fn previews() -> &'static Mutex<HashMap<String, ManagedPreview>> {
    PREVIEWS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn start_preview(root: impl AsRef<Path>) -> RuntimeResult<Value> {
    let root = root.as_ref().canonicalize().map_err(|error| {
        RuntimeError::new(format!("studio preview root is unavailable: {error}"))
    })?;
    if !root.is_dir() {
        return Err(RuntimeError::new("studio preview root must be a directory"));
    }

    if let Some(preview) = previews()
        .lock()
        .map_err(|_| RuntimeError::new("studio preview manager is unavailable"))?
        .values()
        .find(|managed| managed.root == root)
    {
        let views_addr = preview
            .session
            .servers
            .views_addr
            .ok_or_else(|| RuntimeError::new("studio preview did not start a Views server"))?;
        return Ok(json!(format!("http://{views_addr}")));
    }

    let session = start_studio_session(&root).await?;
    let views_addr = session
        .servers
        .views_addr
        .ok_or_else(|| RuntimeError::new("studio preview did not start a Views server"))?;
    let preview = format!("http://{views_addr}");
    let (stop_watch, watch_task) = session.spawn_watch();
    previews()
        .lock()
        .map_err(|_| RuntimeError::new("studio preview manager is unavailable"))?
        .insert(
            preview.clone(),
            ManagedPreview {
                root,
                session,
                stop_watch,
                watch_task,
            },
        );

    Ok(json!(preview))
}

pub async fn stop_preview(preview: &str) -> RuntimeResult<Value> {
    let managed = previews()
        .lock()
        .map_err(|_| RuntimeError::new("studio preview manager is unavailable"))?
        .remove(preview);
    let Some(managed) = managed else {
        return Ok(json!(false));
    };
    let _ = managed.stop_watch.send(());
    let _ = managed.watch_task.await;
    managed.session.shutdown().await?;
    Ok(json!(true))
}
