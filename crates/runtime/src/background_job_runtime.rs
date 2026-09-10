use crate::error::{RuntimeError, RuntimeResult};
use crate::handlers::{CacheRuntimeMode, execute_background_action};
use crate::logging::{log_error, log_info};
use dowe_compiler::{
    CompiledProject, CronSchedule, ServerAction, ServerBackgroundJob, ServerConfig,
    ServerFunctionAction, ServerStatement, StoreLiteral, compile_dev,
};
#[cfg(test)]
use std::collections::HashMap;
use std::env;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(test)]
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;

const ROOT_ENV: &str = "DOWE_BACKGROUND_ROOT";
const JOB_ENV: &str = "DOWE_BACKGROUND_JOB";
const CACHE_MODE_ENV: &str = "DOWE_BACKGROUND_CACHE_MODE";
const ARGS_ENV: &str = "DOWE_BACKGROUND_ARGS";

pub fn start_init_background_jobs(
    root: &Path,
    action: &ServerAction,
    cache_mode: CacheRuntimeMode,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();
    for statement in &action.statements {
        match statement {
            ServerStatement::Task(job) => launch_task(root, job, cache_mode),
            ServerStatement::Cron(job) => handles.push(start_cron(root, job.clone(), cache_mode)),
            _ => {}
        }
    }
    handles
}

pub fn launch_task(root: &Path, job: &ServerBackgroundJob, cache_mode: CacheRuntimeMode) {
    let args = match static_json(&job.args) {
        Ok(args) => args,
        Err(error) => {
            log_error(format!(
                "Background task `{}` has invalid args: {error}",
                job.id
            ));
            return;
        }
    };
    launch_task_with_args(root, job, args, cache_mode);
}

pub fn launch_task_with_args(
    root: &Path,
    job: &ServerBackgroundJob,
    args: serde_json::Value,
    cache_mode: CacheRuntimeMode,
) {
    #[cfg(test)]
    if capture_task_launch(root, job, &args) {
        return;
    }
    if job.inline {
        launch_inline_task(root, job, args, cache_mode);
        return;
    }
    match spawn_worker(root, &job.id, &args, cache_mode) {
        Ok(child) => monitor_child(job.id.clone(), child, None),
        Err(error) => log_error(format!(
            "Background task `{}` failed to start: {error}",
            job.id
        )),
    }
}

fn launch_inline_task(
    root: &Path,
    job: &ServerBackgroundJob,
    args: serde_json::Value,
    cache_mode: CacheRuntimeMode,
) {
    let root = root.to_path_buf();
    let id = job.id.clone();
    let action = job.action.clone();
    tokio::spawn(async move {
        let project = match tokio::task::spawn_blocking({
            let root = root.clone();
            move || {
                std::thread::Builder::new()
                    .name("dowe-inline-task-compiler".to_string())
                    .stack_size(64 * 1024 * 1024)
                    .spawn(move || compile_dev(&root))
                    .map_err(|error| error.to_string())?
                    .join()
                    .map_err(|_| "inline task compiler panicked".to_string())
            }
        })
        .await
        {
            Ok(Ok(Ok(mut project))) => {
                project.local_databases = true;
                project
            }
            Ok(Ok(Err(error))) => {
                log_error(format!("Background task `{id}` failed to compile: {error}"));
                return;
            }
            Ok(Err(error)) => {
                log_error(format!(
                    "Background task `{id}` compiler worker failed: {error}"
                ));
                return;
            }
            Err(error) => {
                log_error(format!(
                    "Background task `{id}` compiler worker failed: {error}"
                ));
                return;
            }
        };
        if let Err(error) = execute_background_action(&project, &action, args, cache_mode).await {
            log_error(format!("Background task `{id}` failed: {error}"));
        }
    });
}

pub fn launch_task_statements(root: &Path, action: &ServerAction, cache_mode: CacheRuntimeMode) {
    for statement in &action.statements {
        if let ServerStatement::Task(job) = statement {
            launch_task(root, job, cache_mode);
        }
    }
}

pub async fn run_worker_from_env() -> RuntimeResult<bool> {
    let Ok(root) = env::var(ROOT_ENV) else {
        return Ok(false);
    };
    let job_id =
        env::var(JOB_ENV).map_err(|_| RuntimeError::new("background worker job id is missing"))?;
    let project = compile_dev(&root)?;
    let job = find_job(&project, &job_id)
        .ok_or_else(|| RuntimeError::new(format!("background job `{job_id}` was not found")))?;
    let args = match env::var(ARGS_ENV) {
        Ok(args) => serde_json::from_str(&args)
            .map_err(|_| RuntimeError::new("background worker args are invalid"))?,
        Err(_) => static_json(&job.args)?,
    };
    let cache_mode = match env::var(CACHE_MODE_ENV).as_deref() {
        Ok("production") => CacheRuntimeMode::Production,
        _ => CacheRuntimeMode::Local,
    };
    execute_background_action(&project, &job.action, args, cache_mode).await?;
    Ok(true)
}

fn start_cron(
    root: &Path,
    job: ServerBackgroundJob,
    cache_mode: CacheRuntimeMode,
) -> JoinHandle<()> {
    let root = root.to_path_buf();
    tokio::spawn(async move {
        let Some(expression) = job.schedule.as_deref() else {
            log_error(format!("Cron job `{}` has no schedule", job.id));
            return;
        };
        let schedule = match CronSchedule::parse(expression) {
            Ok(schedule) => schedule,
            Err(error) => {
                log_error(format!(
                    "Cron job `{}` has an invalid schedule: {error}",
                    job.id
                ));
                return;
            }
        };
        let running = Arc::new(AtomicBool::new(false));
        let mut last_minute = Some(unix_minute());
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let minute = unix_minute();
            if last_minute == Some(minute) || !schedule.matches_unix_minute(minute) {
                continue;
            }
            last_minute = Some(minute);
            if running.swap(true, Ordering::AcqRel) {
                log_info(format!(
                    "WARN Cron job `{}` skipped an overlapping run",
                    job.id
                ));
                continue;
            }
            let args = match static_json(&job.args) {
                Ok(args) => args,
                Err(error) => {
                    running.store(false, Ordering::Release);
                    log_error(format!("Cron job `{}` has invalid args: {error}", job.id));
                    continue;
                }
            };
            match spawn_worker(&root, &job.id, &args, cache_mode) {
                Ok(child) => monitor_child(job.id.clone(), child, Some(running.clone())),
                Err(error) => {
                    running.store(false, Ordering::Release);
                    log_error(format!("Cron job `{}` failed to start: {error}", job.id));
                }
            }
        }
    })
}

fn spawn_worker(
    root: &Path,
    job_id: &str,
    args: &serde_json::Value,
    cache_mode: CacheRuntimeMode,
) -> std::io::Result<Child> {
    Command::new(env::current_exe()?)
        .env(ROOT_ENV, root)
        .env(JOB_ENV, job_id)
        .env(ARGS_ENV, args.to_string())
        .env(
            CACHE_MODE_ENV,
            match cache_mode {
                CacheRuntimeMode::Local => "local",
                CacheRuntimeMode::Production => "production",
            },
        )
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn monitor_child(id: String, mut child: Child, running: Option<Arc<AtomicBool>>) {
    tokio::spawn(async move {
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(status)) => {
                    log_error(format!("Background job `{id}` exited with {status}"));
                    break;
                }
                Ok(None) => tokio::time::sleep(Duration::from_millis(100)).await,
                Err(error) => {
                    log_error(format!("Background job `{id}` wait failed: {error}"));
                    break;
                }
            }
        }
        if let Some(running) = running {
            running.store(false, Ordering::Release);
        }
    });
}

fn unix_minute() -> i64 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    (seconds / 60) as i64
}

fn find_job(project: &CompiledProject, id: &str) -> Option<ServerBackgroundJob> {
    find_server_job(&project.backend, id).or_else(|| {
        project
            .desktop_server
            .as_ref()
            .and_then(|server| find_server_job(server, id))
    })
}

fn find_server_job(server: &ServerConfig, id: &str) -> Option<ServerBackgroundJob> {
    find_action_job(&server.init_action, id)
        .or_else(|| {
            server.endpoints.iter().find_map(|endpoint| {
                find_action_job(&endpoint.action, id).or_else(|| {
                    endpoint.middlewares.iter().find_map(|middleware| {
                        find_middleware_job(&middleware.action.statements, id)
                    })
                })
            })
        })
        .or_else(|| {
            server.websockets.iter().find_map(|route| {
                find_action_job(&route.handlers.open, id)
                    .or_else(|| find_action_job(&route.handlers.message, id))
                    .or_else(|| find_action_job(&route.handlers.close, id))
                    .or_else(|| find_action_job(&route.handlers.drain, id))
                    .or_else(|| {
                        route.middlewares.iter().find_map(|middleware| {
                            find_middleware_job(&middleware.action.statements, id)
                        })
                    })
            })
        })
        .or_else(|| {
            server
                .transports
                .iter()
                .find_map(|transport| find_action_job(&transport.action, id))
        })
}

fn find_action_job(action: &ServerAction, id: &str) -> Option<ServerBackgroundJob> {
    for statement in &action.statements {
        match statement {
            ServerStatement::Task(job) | ServerStatement::Cron(job) => {
                if job.id == id {
                    return Some(job.clone());
                }
                if let Some(found) = find_server_function_job(&job.action, id) {
                    return Some(found);
                }
            }
            ServerStatement::Call(call) => {
                if let Some(found) = find_server_function_job(&call.action, id) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_middleware_job(
    statements: &[dowe_compiler::ServerMiddlewareStatement],
    id: &str,
) -> Option<ServerBackgroundJob> {
    statements.iter().find_map(|statement| match statement {
        dowe_compiler::ServerMiddlewareStatement::Call(call) => {
            find_server_function_job(&call.action, id)
        }
        dowe_compiler::ServerMiddlewareStatement::IfValid { statements, .. } => {
            find_middleware_job(statements, id)
        }
        _ => None,
    })
}

fn find_server_function_job(
    action: &ServerFunctionAction,
    id: &str,
) -> Option<ServerBackgroundJob> {
    find_action_job(
        &ServerAction {
            statements: action.statements.clone(),
        },
        id,
    )
}

fn static_json(value: &StoreLiteral) -> RuntimeResult<serde_json::Value> {
    Ok(match value {
        StoreLiteral::Null => serde_json::Value::Null,
        StoreLiteral::Bool(value) => serde_json::Value::Bool(*value),
        StoreLiteral::Number(value) => serde_json::from_str(value)
            .map_err(|_| RuntimeError::new("background numeric argument is invalid"))?,
        StoreLiteral::String(value) => serde_json::Value::String(value.clone()),
        StoreLiteral::Reference(value) => {
            return Err(RuntimeError::new(format!(
                "background argument reference `{value}` is not allowed"
            )));
        }
        StoreLiteral::Array(values) => serde_json::Value::Array(
            values
                .iter()
                .map(static_json)
                .collect::<RuntimeResult<Vec<_>>>()?,
        ),
        StoreLiteral::Object(entries) => serde_json::Value::Object(
            entries
                .iter()
                .map(|(key, value)| Ok((key.clone(), static_json(value)?)))
                .collect::<RuntimeResult<serde_json::Map<_, _>>>()?,
        ),
    })
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CapturedTaskLaunch {
    pub(crate) id: String,
    pub(crate) target: Option<String>,
    pub(crate) args: serde_json::Value,
}

#[cfg(test)]
fn task_launch_captures() -> &'static Mutex<HashMap<PathBuf, Vec<CapturedTaskLaunch>>> {
    static CAPTURES: OnceLock<Mutex<HashMap<PathBuf, Vec<CapturedTaskLaunch>>>> = OnceLock::new();
    CAPTURES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub(crate) fn start_task_launch_capture(root: &Path) {
    task_launch_captures()
        .lock()
        .expect("task launch capture")
        .insert(root.to_path_buf(), Vec::new());
}

#[cfg(test)]
pub(crate) fn take_task_launches(root: &Path) -> Vec<CapturedTaskLaunch> {
    task_launch_captures()
        .lock()
        .expect("task launch capture")
        .remove(root)
        .unwrap_or_default()
}

#[cfg(test)]
pub(crate) fn task_launches(root: &Path) -> Vec<CapturedTaskLaunch> {
    task_launch_captures()
        .lock()
        .expect("task launch capture")
        .get(root)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
fn capture_task_launch(root: &Path, job: &ServerBackgroundJob, args: &serde_json::Value) -> bool {
    let mut captures = task_launch_captures().lock().expect("task launch capture");
    let Some(launches) = captures.get_mut(root) else {
        return false;
    };
    launches.push(CapturedTaskLaunch {
        id: job.id.clone(),
        target: job.target.clone(),
        args: args.clone(),
    });
    true
}
