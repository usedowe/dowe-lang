use crate::auth::verify_user;
use crate::engine::{inspection_json, open_database};
use crate::error::{KvError, KvResult};
use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteKvConfig {
    pub host: String,
    pub database: String,
    pub user: String,
    pub credential: String,
    pub persist: bool,
}

#[derive(Clone)]
pub struct RemoteKvClient {
    config: RemoteKvConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvServerConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningKvServer {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<KvResult<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteKvRequest {
    pub operation: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub persist: bool,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteKvResponse {
    ok: bool,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    error: Option<RemoteKvError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteKvError {
    category: String,
    message: String,
}

#[derive(Clone)]
struct KvServerState {
    root: PathBuf,
}

impl Default for KvServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4148,
        }
    }
}

impl RemoteKvClient {
    pub fn new(config: RemoteKvConfig) -> KvResult<Self> {
        if config.host.trim().is_empty() {
            return Err(KvError::Remote("remote KV host is empty".to_string()));
        }
        if config.user.trim().is_empty() {
            return Err(KvError::Authentication(
                "remote KV user is empty".to_string(),
            ));
        }
        if config.credential.is_empty() {
            return Err(KvError::Authentication(
                "remote KV credential is empty".to_string(),
            ));
        }
        Ok(Self {
            config,
            client: reqwest::Client::new(),
        })
    }

    pub async fn get(&self, key: &str, required: bool) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "get".to_string(),
            key: Some(key.to_string()),
            value: None,
            prefix: None,
            persist: self.config.persist,
            required,
        })
        .await
    }

    pub async fn set(&self, key: &str, value: Value) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "set".to_string(),
            key: Some(key.to_string()),
            value: Some(value),
            prefix: None,
            persist: self.config.persist,
            required: false,
        })
        .await
    }

    pub async fn delete(&self, key: &str) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "delete".to_string(),
            key: Some(key.to_string()),
            value: None,
            prefix: None,
            persist: self.config.persist,
            required: false,
        })
        .await
    }

    pub async fn keys(&self, prefix: Option<&str>) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "keys".to_string(),
            key: None,
            value: None,
            prefix: prefix.map(str::to_string),
            persist: self.config.persist,
            required: false,
        })
        .await
    }

    pub async fn clear(&self) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "clear".to_string(),
            key: None,
            value: None,
            prefix: None,
            persist: self.config.persist,
            required: false,
        })
        .await
    }

    pub async fn inspect(&self) -> KvResult<Value> {
        self.send(RemoteKvRequest {
            operation: "inspect".to_string(),
            key: None,
            value: None,
            prefix: None,
            persist: self.config.persist,
            required: false,
        })
        .await
    }

    async fn send(&self, request: RemoteKvRequest) -> KvResult<Value> {
        let url = format!(
            "{}/v1/databases/{}/operation",
            self.config.host.trim_end_matches('/'),
            self.config.database
        );
        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(30))
            .header("X-Dowe-Kv-User", &self.config.user)
            .bearer_auth(&self.config.credential)
            .json(&request)
            .send()
            .await
            .map_err(|error| KvError::Remote(error.to_string()))?;
        let status = response.status();
        let body = response
            .json::<RemoteKvResponse>()
            .await
            .map_err(|error| KvError::Remote(error.to_string()))?;
        if body.ok {
            return Ok(body.data.unwrap_or(Value::Null));
        }
        let Some(error) = body.error else {
            return Err(KvError::Remote(format!(
                "remote KV returned HTTP {status} without error"
            )));
        };
        Err(remote_error(&error.category, error.message))
    }
}

pub async fn start_kv_server(config: KvServerConfig) -> KvResult<RunningKvServer> {
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|error| KvError::Remote(error.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|error| KvError::Remote(error.to_string()))?;
    let router = Router::new()
        .route(
            "/v1/databases/{database}/operation",
            post(operation_handler),
        )
        .with_state(KvServerState { root: config.root });
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|error| KvError::Remote(error.to_string()))
    });
    Ok(RunningKvServer {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_kv_server(config: KvServerConfig) -> KvResult<()> {
    let server = start_kv_server(config).await?;
    server.wait().await
}

impl RunningKvServer {
    pub async fn shutdown(mut self) -> KvResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle
            .await
            .map_err(|error| KvError::Remote(error.to_string()))?
    }

    pub async fn wait(mut self) -> KvResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| KvError::Remote(error.to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => result.map_err(|error| KvError::Remote(error.to_string()))?,
        }
    }
}

async fn operation_handler(
    State(state): State<KvServerState>,
    AxumPath(database): AxumPath<String>,
    headers: HeaderMap,
    Json(request): Json<RemoteKvRequest>,
) -> Response {
    match authorize(&state.root, &database, &headers)
        .and_then(|_| execute(&state.root, &database, request))
    {
        Ok(value) => json_status(StatusCode::OK, success(value)),
        Err(error) => kv_error_response(error),
    }
}

fn authorize(root: &Path, database: &str, headers: &HeaderMap) -> KvResult<()> {
    let user = headers
        .get("X-Dowe-Kv-User")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| KvError::Authentication("KV user is required".to_string()))?;
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_bearer)
        .ok_or_else(|| KvError::Authentication("KV bearer token is required".to_string()))?;
    verify_user(root, database, user, &token)
}

fn execute(root: &Path, database: &str, request: RemoteKvRequest) -> KvResult<Value> {
    let db = open_database(root, database, request.persist)?;
    match request.operation.as_str() {
        "get" => {
            let key = required_text(request.key, "key")?;
            let value = db.get(&key)?;
            if value.is_none() && request.required {
                return Err(KvError::NotFound("KV key not found".to_string()));
            }
            Ok(value.unwrap_or(Value::Null))
        }
        "set" => {
            let key = required_text(request.key, "key")?;
            let value = request.value.ok_or_else(|| {
                KvError::InvalidRequest("remote KV request missing `value`".to_string())
            })?;
            db.set(&key, value)?;
            Ok(set_json(&key))
        }
        "delete" => {
            let key = required_text(request.key, "key")?;
            Ok(delete_json(db.delete(&key)?))
        }
        "keys" => Ok(Value::Array(
            db.keys(request.prefix.as_deref())?
                .into_iter()
                .map(Value::String)
                .collect(),
        )),
        "clear" => Ok(clear_json(db.clear()?)),
        "inspect" => Ok(inspection_json(&db.inspect()?)),
        value => Err(KvError::InvalidRequest(format!(
            "unsupported remote KV operation `{value}`"
        ))),
    }
}

fn parse_bearer(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || token.is_empty() || !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    Some(token.to_string())
}

fn required_text(value: Option<String>, name: &str) -> KvResult<String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| KvError::InvalidRequest(format!("remote KV request missing `{name}`")))
}

fn set_json(key: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("key".to_string(), Value::String(key.to_string()));
    Value::Object(output)
}

fn delete_json(deleted: bool) -> Value {
    let mut output = Map::new();
    output.insert("deleted".to_string(), Value::Bool(deleted));
    Value::Object(output)
}

fn clear_json(cleared: usize) -> Value {
    let mut output = Map::new();
    output.insert("cleared".to_string(), Value::Number(cleared.into()));
    Value::Object(output)
}

fn json_status(status: StatusCode, value: Value) -> Response {
    (status, Json(value)).into_response()
}

fn success(value: Value) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("data".to_string(), value);
    Value::Object(output)
}

fn kv_error_response(error: KvError) -> Response {
    let status = match error {
        KvError::Authentication(_) => StatusCode::UNAUTHORIZED,
        KvError::Authorization(_) => StatusCode::FORBIDDEN,
        KvError::InvalidName(_) | KvError::InvalidRequest(_) | KvError::Corruption(_) => {
            StatusCode::BAD_REQUEST
        }
        KvError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let mut error_json = Map::new();
    error_json.insert(
        "category".to_string(),
        Value::String(error.category().to_string()),
    );
    error_json.insert(
        "message".to_string(),
        Value::String(error.message().to_string()),
    );
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(false));
    output.insert("error".to_string(), Value::Object(error_json));
    json_status(status, Value::Object(output))
}

fn remote_error(category: &str, message: String) -> KvError {
    match category {
        "NotFound" => KvError::NotFound(message),
        "InvalidName" => KvError::InvalidName(message),
        "InvalidRequest" => KvError::InvalidRequest(message),
        "Authentication" => KvError::Authentication(message),
        "Authorization" => KvError::Authorization(message),
        "Remote" => KvError::Remote(message),
        "DurabilityError" => KvError::DurabilityError(message),
        "Corruption" => KvError::Corruption(message),
        "Io" => KvError::Io(message),
        _ => KvError::Remote(format!("remote KV returned unknown category `{category}`")),
    }
}
