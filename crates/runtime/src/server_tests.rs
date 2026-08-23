use crate::DevEventType;
use crate::ProductionAccess;
use crate::server::{
    DevServerTargets, start_dev, start_dev_servers, start_production, start_production_with_access,
};
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use dowe_cache::{
    CacheServerConfig, clear_memory as clear_kv_memory, create_account as create_cache_account,
    start_cache_server,
};
use dowe_compiler::{compile_dev, compile_dev_web, compile_dev_with_seeders};
use dowe_crypto::sign_jws_hs256;
use dowe_database::{
    DatabaseServiceConfig, DoweDatabaseClient, DoweDatabaseConfig, create_account,
    start_database_service,
};
use dowe_queue::{
    QueueClient, QueueConfig, QueueProvider, create_account as create_queue_account, open_namespace,
};
use dowe_vector::{
    DoweVectorClient, DoweVectorConfig, VectorServerConfig,
    close_remote_connections as close_vector_connections, create_account as create_vector_account,
    start_vector_server,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

mod database_cache;
mod development;
mod http_routes;
mod providers;
mod server_selection;

#[derive(Clone, Debug)]
struct MockOpenRouterRequest {
    authorization: Option<String>,
    body: serde_json::Value,
}

#[derive(Clone, Debug)]
struct MockExternalRequest {
    accept: Option<String>,
    api_key: Option<String>,
}

struct MockExternalApi {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<MockExternalRequest>>>,
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl MockExternalApi {
    async fn start() -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new()
            .route("/v1/products", get(mock_external_products))
            .route("/redirect", get(mock_external_redirect))
            .route("/slow", get(mock_external_slow))
            .with_state(requests.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener");
        let addr = listener.local_addr().expect("mock addr");
        let (shutdown, receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
        });
        Self {
            addr,
            requests,
            shutdown,
            handle,
        }
    }

    async fn requests(&self) -> Vec<MockExternalRequest> {
        self.requests.lock().await.clone()
    }

    async fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.handle.await;
    }
}

async fn mock_external_products(
    State(requests): State<Arc<Mutex<Vec<MockExternalRequest>>>>,
    headers: HeaderMap,
) -> Response {
    let accept = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    requests
        .lock()
        .await
        .push(MockExternalRequest { accept, api_key });
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        json!({"items":[{"name":"Dowe Kit"}]}).to_string(),
    )
        .into_response()
}

async fn mock_external_redirect() -> Response {
    (StatusCode::FOUND, [(LOCATION, "/v1/products")], "").into_response()
}

async fn mock_external_slow() -> Response {
    sleep(Duration::from_millis(50)).await;
    (StatusCode::OK, "slow").into_response()
}

struct MockOpenRouter {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<MockOpenRouterRequest>>>,
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl MockOpenRouter {
    async fn start() -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new()
            .route("/api/v1/chat/completions", post(mock_openrouter_chat))
            .with_state(requests.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener");
        let addr = listener.local_addr().expect("mock addr");
        let (shutdown, receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
        });
        Self {
            addr,
            requests,
            shutdown,
            handle,
        }
    }

    async fn requests(&self) -> Vec<MockOpenRouterRequest> {
        self.requests.lock().await.clone()
    }

    async fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.handle.await;
    }
}

async fn mock_openrouter_chat(
    State(requests): State<Arc<Mutex<Vec<MockOpenRouterRequest>>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let value = serde_json::from_slice::<serde_json::Value>(&body).expect("mock request json");
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    requests.lock().await.push(MockOpenRouterRequest {
        authorization,
        body: value.clone(),
    });
    if value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/event-stream")],
            "data: {\"choices\":[{\"delta\":{\"content\":\"mock delta\"}}]}\ndata: [DONE]\n",
        )
            .into_response();
    }
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        json!({"choices":[{"message":{"content":"mock message"}}]}).to_string(),
    )
        .into_response()
}

async fn websocket_json(
    websocket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> serde_json::Value {
    let message = websocket
        .next()
        .await
        .expect("websocket message")
        .expect("websocket result");
    serde_json::from_str(message.to_text().expect("websocket text")).expect("websocket json")
}

fn write_fixture(root: &Path, port: u16) {
    fs::create_dir_all(root.join("layouts")).expect("layouts");
    fs::create_dir_all(root.join("pages")).expect("pages");
    fs::create_dir_all(root.join("routes")).expect("routes");
    fs::write(
        root.join("main.dowe"),
        format!(
            r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:{port}
    route "/api/status"
      response text:"OK"
    route "/users/:id"
      handler req
        return text:"Hello User {{req.params.id}}!"
    route "/api/posts"
      method GET
        return text:"List posts"
      method POST async req
        const body value:req.json
        return json:{{ created:true ...body }}
    websocket "/ws"
      open ws
      message ws data
      close ws code reason
      drain ws
    init
      log "Server inicializado""#
        ),
    )
    .expect("server");
    fs::write(
        root.join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage"#,
    )
    .expect("views");
    fs::write(
        root.join("layouts/auth.dowe"),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
    )
    .expect("layout");
    fs::write(
        root.join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login""#,
    )
    .expect("page");
}

fn write_blog_server_fixture(root: &Path) {
    fs::create_dir_all(root.join("handlers")).expect("handlers");
    fs::write(
        root.join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:0
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#,
    )
    .expect("server");
    fs::write(
        root.join("handlers/blogs.dowe"),
r#"type BlogInput
  title:string
  content:string

type BlogPatch
  id?:string
  title?:string
  content?:string

handler listBlogs req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler createBlog
  const body:BlogInput value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query created conn:db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  log created.title
  query blogs conn:db.list table:"blogs"
  return status:201 json:{ ok:true data:blogs }

handler readBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blog conn:db.read table:"blogs" where:{ id:req.params.id } required:true
  return json:{ ok:true data:blog }

handler updateBlog
  const body:BlogPatch value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query updated conn:db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler deleteBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query deleted conn:db.delete table:"blogs" where:{ id:req.params.id } required:true
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }"#,
    )
    .expect("blogs handler");
}

fn write_blog_server_fixture_with_cors(root: &Path, cors: &str) {
    write_blog_server_fixture(root);
    fs::write(
        root.join("main.dowe"),
        format!(
            r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:0
    {cors}
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#
        ),
    )
    .expect("server");
}
