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
    match spawn_worker(root, &job.id, &args, cache_mode) {
        Ok(child) => monitor_child(job.id.clone(), child, None),
        Err(error) => log_error(format!(
            "Background task `{}` failed to start: {error}",
            job.id
        )),
    }
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

#[cfg(test)]
mod tests {
    use super::find_job;
    use crate::handlers::{CacheRuntimeMode, execute_background_action};
    use dowe_compiler::{
        ServerBackgroundJob, ServerMiddlewareStatement, ServerStatement, compile_dev,
    };
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn worker_traverses_and_executes_named_and_inline_tasks() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("main.dowe"),
            r#"import dispatch from "@/server/tasks/dispatch"

main
  server port:0
    route "/tasks"
      handler
        task args:{ message:"inline" }
          log args.message
        dispatch result
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/tasks/dispatch.dowe"),
            r#"import recordTask from "./record"

fn dispatch
  task recordTask args:{ message:"named" }
  return value:{ ok:true }"#,
        )
        .expect("dispatch");
        fs::write(
            root.join("server/tasks/record.dowe"),
            r#"fn recordTask params:{ message:string }
  log args.message
  return value:null"#,
        )
        .expect("record");

        let project = compile_dev(root).expect("project");
        let action = &project.backend.endpoints[0].action;
        let ServerStatement::Task(inline) = &action.statements[0] else {
            panic!("inline task");
        };
        let ServerStatement::Call(dispatch) = &action.statements[1] else {
            panic!("dispatch call");
        };
        let ServerStatement::Task(named) = &dispatch.action.statements[0] else {
            panic!("named task");
        };

        assert!(find_job(&project, &inline.id).is_some_and(|job| job.target.is_none()));
        assert!(
            find_job(&project, &named.id)
                .and_then(|job| job.target)
                .is_some_and(|target| target == "recordTask")
        );
        execute_background_action(
            &project,
            &inline.action,
            json!({ "message": "inline" }),
            CacheRuntimeMode::Local,
        )
        .await
        .expect("inline worker execution");
        execute_background_action(
            &project,
            &named.action,
            json!({ "message": "named" }),
            CacheRuntimeMode::Local,
        )
        .await
        .expect("named worker execution");
    }

    #[tokio::test]
    async fn worker_finds_tasks_nested_in_endpoint_and_websocket_middlewares() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/functions")).expect("functions");
        fs::create_dir_all(root.join("server/middlewares")).expect("middlewares");
        fs::write(
            root.join("main.dowe"),
            r#"import endpointGuard from "@/server/middlewares/endpoint"
import websocketGuard from "@/server/middlewares/websocket"

main
  server port:0
    route "/endpoint" middleware:[endpointGuard]
      response text:"OK"
    websocket "/socket" middleware:[websocketGuard]
      open ws"#,
        )
        .expect("main");
        fs::write(
            root.join("server/functions/authorize.dowe"),
            r#"fn authorize
  return value:{ valid:true }"#,
        )
        .expect("authorize");
        fs::write(
            root.join("server/middlewares/endpoint.dowe"),
            r#"import authorize from "../functions/authorize"
import dispatchEndpoint from "../functions/dispatch-endpoint"

middleware endpointGuard
  authorize verification
  if verification.valid
    dispatchEndpoint result
  next"#,
        )
        .expect("endpoint middleware");
        fs::write(
            root.join("server/middlewares/websocket.dowe"),
            r#"import authorize from "../functions/authorize"
import dispatchWebsocket from "../functions/dispatch-websocket"

middleware websocketGuard
  authorize verification
  if verification.valid
    dispatchWebsocket result
  next"#,
        )
        .expect("websocket middleware");
        fs::write(
            root.join("server/functions/dispatch-endpoint.dowe"),
            r#"import enqueueEndpoint from "./enqueue-endpoint"

fn dispatchEndpoint
  enqueueEndpoint result
  return value:null"#,
        )
        .expect("endpoint dispatch");
        fs::write(
            root.join("server/functions/enqueue-endpoint.dowe"),
            r#"import recordEndpointTask from "./record-endpoint-task"

fn enqueueEndpoint
  task recordEndpointTask args:{ message:"endpoint" }
  return value:null"#,
        )
        .expect("endpoint enqueue");
        fs::write(
            root.join("server/functions/record-endpoint-task.dowe"),
            r#"fn recordEndpointTask params:{ message:string }
  log args.message
  return value:null"#,
        )
        .expect("endpoint task");
        fs::write(
            root.join("server/functions/dispatch-websocket.dowe"),
            r#"import enqueueWebsocket from "./enqueue-websocket"

fn dispatchWebsocket
  enqueueWebsocket result
  return value:null"#,
        )
        .expect("websocket dispatch");
        fs::write(
            root.join("server/functions/enqueue-websocket.dowe"),
            r#"import recordWebsocketTask from "./record-websocket-task"

fn enqueueWebsocket
  task recordWebsocketTask args:{ message:"websocket" }
  return value:null"#,
        )
        .expect("websocket enqueue");
        fs::write(
            root.join("server/functions/record-websocket-task.dowe"),
            r#"fn recordWebsocketTask params:{ message:string }
  log args.message
  return value:null"#,
        )
        .expect("websocket task");

        let project = compile_dev(root).expect("project");
        let endpoint_task = nested_middleware_task(&project.backend.endpoints[0].middlewares[0]);
        let websocket_task = nested_middleware_task(&project.backend.websockets[0].middlewares[0]);

        assert_eq!(
            find_job(&project, &endpoint_task.id),
            Some(endpoint_task.clone())
        );
        assert_eq!(
            find_job(&project, &websocket_task.id),
            Some(websocket_task.clone())
        );
        execute_background_action(
            &project,
            &endpoint_task.action,
            json!({ "message": "endpoint" }),
            CacheRuntimeMode::Local,
        )
        .await
        .expect("endpoint worker execution");
        execute_background_action(
            &project,
            &websocket_task.action,
            json!({ "message": "websocket" }),
            CacheRuntimeMode::Local,
        )
        .await
        .expect("websocket worker execution");
    }

    fn nested_middleware_task(middleware: &dowe_compiler::ServerMiddleware) -> ServerBackgroundJob {
        let ServerMiddlewareStatement::IfValid { statements, .. } =
            &middleware.action.statements[1]
        else {
            panic!("middleware validation");
        };
        let ServerMiddlewareStatement::Call(dispatch) = &statements[0] else {
            panic!("middleware dispatch");
        };
        let ServerStatement::Call(enqueue) = &dispatch.action.statements[0] else {
            panic!("function dispatch");
        };
        let ServerStatement::Task(task) = &enqueue.action.statements[0] else {
            panic!("task");
        };
        task.clone()
    }
}
