use crate::background_jobs::start_init_background_jobs;
use crate::error::{RuntimeError, RuntimeResult};
use crate::handlers::{
    backend_declared_websocket_handler, backend_handler, desktop_declared_websocket_handler,
    desktop_handler, dev_websocket_handler, server_inspector_data, server_inspector_execute,
    server_inspector_index, server_inspector_manifest, server_inspector_selection,
    server_inspector_source, views_handler,
};
use crate::logging::{log_dev_info, log_info};
use crate::production_handlers::{production_declared_websocket_handler, production_handler};
use crate::{DevEventBus, DevEventType, ProductionAccess};
use axum::extract::{Json, Path as AxumPath, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Router, middleware};
use dowe_compiler::{
    CompiledProject, EnvironmentValueSource, EnvironmentVariable, EnvironmentVisibility,
    NativeIpcTarget,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::compression::CompressionLayer;

const VIEWS_DEV_PORT: u16 = 7654;
const BACKEND_DEV_PORT: u16 = 7754;
const DESKTOP_DEV_PORT: u16 = 7854;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevServerTargets {
    pub backend: bool,
    pub views: bool,
    pub desktop: bool,
}

impl DevServerTargets {
    pub fn all() -> Self {
        Self {
            backend: true,
            views: true,
            desktop: false,
        }
    }
}

pub struct RunningDevServers {
    pub backend_addr: Option<SocketAddr>,
    pub views_addr: Option<SocketAddr>,
    pub desktop_addr: Option<SocketAddr>,
    pub backend_transport_addrs: Vec<(String, SocketAddr)>,
    pub desktop_transport_addrs: Vec<(String, SocketAddr)>,
    state: DevRuntimeState,
    backend: Option<RunningServer>,
    views: Option<RunningServer>,
    desktop: Option<RunningServer>,
    backend_transports: Vec<RunningServer>,
    desktop_transports: Vec<RunningServer>,
    background_jobs: Vec<JoinHandle<()>>,
}

pub struct RunningProductionServer {
    pub addr: SocketAddr,
    pub transport_addrs: Vec<(String, SocketAddr)>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<RuntimeResult<()>>,
    transports: Vec<RunningServer>,
    background_jobs: Vec<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct DevRuntimeState {
    pub project: Arc<RwLock<Arc<CompiledProject>>>,
    pub events: DevEventBus,
    pub dev_origins: Vec<String>,
    pub(crate) cache_mode: crate::handlers::CacheRuntimeMode,
}

pub(crate) struct RunningServer {
    pub(crate) shutdown: Option<oneshot::Sender<()>>,
    pub(crate) handle: JoinHandle<RuntimeResult<()>>,
}

pub async fn start_dev(project: CompiledProject) -> RuntimeResult<RunningDevServers> {
    start_dev_servers(project, DevServerTargets::all()).await
}

pub async fn start_dev_servers(
    mut project: CompiledProject,
    targets: DevServerTargets,
) -> RuntimeResult<RunningDevServers> {
    project.local_databases = true;
    start_dev_servers_shared(Arc::new(project), targets).await
}

pub(crate) async fn start_dev_servers_shared(
    project: Arc<CompiledProject>,
    targets: DevServerTargets,
) -> RuntimeResult<RunningDevServers> {
    let mut project = match Arc::try_unwrap(project) {
        Ok(project) => project,
        Err(project) => clone_compiled_project(project)?,
    };
    project.local_databases = true;
    if targets.backend {
        log_dev_info("Backend server starting");
    }
    if targets.views {
        log_dev_info("Views server starting");
    }
    if targets.desktop && project.desktop_server.is_some() {
        log_dev_info("Desktop server starting");
    }

    let backend_listener = if targets.backend {
        Some(bind_development_listener(BACKEND_DEV_PORT, "backend").await?)
    } else {
        None
    };
    let views_listener = if targets.views {
        Some(bind_development_listener(VIEWS_DEV_PORT, "views").await?)
    } else {
        None
    };
    let desktop_listener = if targets.desktop && project.desktop_server.is_some() {
        Some(bind_development_listener(DESKTOP_DEV_PORT, "desktop").await?)
    } else {
        None
    };

    if let Some(listener) = &backend_listener {
        let addr = listener.local_addr()?;
        project.backend.port = addr.port();
        let scheme = if project.backend.tls.is_some() {
            "https"
        } else {
            "http"
        };
        apply_development_url(&mut project, "BACKEND_URL", format!("{scheme}://{addr}"));
        apply_development_url(&mut project, "SERVER_URL", format!("{scheme}://{addr}"));
        if let Some(inspector) = &mut project.server_inspector {
            inspector.port = addr.port();
        }
    }
    if let Some(listener) = &desktop_listener {
        let addr = listener.local_addr()?;
        if let Some(server) = &mut project.desktop_server {
            server.port = addr.port();
        }
        apply_development_url(
            &mut project,
            "BACKEND_DESKTOP_URL",
            format!("http://{addr}"),
        );
        apply_development_url(&mut project, "SERVER_DESKTOP_URL", format!("http://{addr}"));
    }
    let project = Arc::new(project);
    crate::database_bootstrap::prepare_databases(&project).await?;

    let mut background_jobs = Vec::new();
    if targets.backend {
        crate::server_actions::execute_server_action(&project.backend.init_action);
        background_jobs.extend(start_init_background_jobs(
            &project.root,
            &project.backend.init_action,
            crate::handlers::CacheRuntimeMode::Local,
        ));
    }
    if targets.desktop
        && let Some(server) = &project.desktop_server
    {
        crate::server_actions::execute_server_action(&server.init_action);
        background_jobs.extend(start_init_background_jobs(
            &project.root,
            &server.init_action,
            crate::handlers::CacheRuntimeMode::Local,
        ));
    }
    background_jobs.push(transport::start_notification_dispatcher(
        project.root.clone(),
        transport::notification_namespace(&project.root),
    ));

    let mut backend = None;
    let mut backend_addr = None;
    let mut views = None;
    let mut views_addr = None;
    let mut desktop = None;
    let mut desktop_addr = None;
    let mut backend_transports = Vec::new();
    let mut backend_transport_addrs = Vec::new();
    let mut desktop_transports = Vec::new();
    let mut desktop_transport_addrs = Vec::new();
    let mut dev_origins = Vec::new();

    if let Some(listener) = &views_listener {
        let addr = listener.local_addr()?;
        dev_origins.push(format!("http://{addr}"));
    }
    let backend_websocket_paths = project
        .backend
        .websockets
        .iter()
        .map(|route| route.path.clone())
        .collect::<Vec<_>>();
    let backend_transport_configs = project.backend.transports.clone();
    let backend_cache_service = project.backend.cache_service;
    let backend_database_service = project.backend.database_service;
    let backend_vector_service = project.backend.vector_service;
    let backend_queue_service = project.backend.queue_service;
    let backend_tls = project.backend.tls.clone();
    let backend_environment = project.environment_config.clone();
    let desktop_websocket_paths = project
        .desktop_server
        .as_ref()
        .map(|server| {
            server
                .websockets
                .iter()
                .map(|route| route.path.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let desktop_transport_configs = project
        .desktop_server
        .as_ref()
        .map(|server| server.transports.clone())
        .unwrap_or_default();
    let project_root = project.root.clone();

    let mut state = DevRuntimeState {
        project: Arc::new(RwLock::new(project)),
        events: DevEventBus::default(),
        dev_origins,
        cache_mode: crate::handlers::CacheRuntimeMode::Local,
    };

    if let Some(listener) = backend_listener {
        let addr = listener.local_addr()?;
        let router = routers::backend_router(
            state.clone(),
            backend_websocket_paths,
            backend_cache_service,
            backend_database_service,
            backend_vector_service,
            backend_queue_service,
            project_root.clone(),
        );
        let (shutdown, signal) = oneshot::channel();
        let handle = match backend_tls.clone() {
            Some(tls) => crate::tls::spawn_tls_server(
                listener,
                router,
                tls,
                backend_environment,
                project_root.clone(),
                signal,
            ),
            None => spawn_server(listener, router, signal),
        };
        let scheme = if backend_tls.is_some() {
            "https"
        } else {
            "http"
        };
        state.dev_origins.push(format!("{scheme}://{addr}"));
        log_dev_info(format!("Backend server started at {scheme}://{addr}"));
        if state.project.read().await.server_inspector.is_some() {
            log_info(format!(
                "Server inspector available at {scheme}://{addr}/_dowe/dev/server/"
            ));
        }
        backend_addr = Some(addr);
        backend = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
        let listeners = transport::spawn_transport_listeners(
            &backend_transport_configs,
            &project_root,
            crate::handlers::CacheRuntimeMode::Local,
        )
        .await?;
        backend_transport_addrs = listeners.addrs;
        backend_transports = listeners.servers;
    }

    if let Some(listener) = views_listener {
        let addr = listener.local_addr()?;
        let router = routers::views_router(state.clone());
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_dev_info(format!("Views server started at http://{addr}"));
        views_addr = Some(addr);
        views = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
    }
    if let Some(listener) = desktop_listener {
        let addr = listener.local_addr()?;
        let router = routers::desktop_router(state.clone(), desktop_websocket_paths);
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_dev_info(format!("Desktop server started at http://{addr}"));
        desktop_addr = Some(addr);
        desktop = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
        let listeners = transport::spawn_transport_listeners(
            &desktop_transport_configs,
            &project_root,
            crate::handlers::CacheRuntimeMode::Local,
        )
        .await?;
        desktop_transport_addrs = listeners.addrs;
        desktop_transports = listeners.servers;
    }

    Ok(RunningDevServers {
        backend_addr,
        views_addr,
        desktop_addr,
        backend_transport_addrs,
        desktop_transport_addrs,
        state,
        backend,
        views,
        desktop,
        backend_transports,
        desktop_transports,
        background_jobs,
    })
}

async fn bind_development_listener(start_port: u16, label: &str) -> RuntimeResult<TcpListener> {
    for port in start_port..=u16::MAX {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        match TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(error) => return Err(bind_error(addr, error)),
        }
    }
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, start_port));
    Err(RuntimeError::new(format!(
        "no available {label} development port at or above {start_port} on {addr}"
    )))
}
