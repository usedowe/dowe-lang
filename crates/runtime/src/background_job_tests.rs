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
  task fn:recordTask args:{ message:"named" }
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
  task fn:recordEndpointTask args:{ message:"endpoint" }
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
  task fn:recordWebsocketTask args:{ message:"websocket" }
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
