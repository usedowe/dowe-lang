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

mod routers;
mod transport;


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

fn clone_compiled_project(project: Arc<CompiledProject>) -> RuntimeResult<CompiledProject> {
    std::thread::Builder::new()
        .name("dowe-dev-project-clone".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || (*project).clone())
        .map_err(|error| RuntimeError::new(error.to_string()))?
        .join()
        .map_err(|_| RuntimeError::new("development project clone thread panicked"))
}

fn apply_development_url(project: &mut CompiledProject, name: &str, value: String) {
    if let Some(variable) = project
        .environment_config
        .variables
        .iter_mut()
        .find(|variable| variable.name == name)
    {
        variable.visibility = EnvironmentVisibility::Client;
        variable.resolved_source = EnvironmentValueSource::DotEnv;
        variable.resolved_value = Some(value);
        return;
    }
    project
        .environment_config
        .variables
        .push(EnvironmentVariable {
            name: name.to_string(),
            visibility: EnvironmentVisibility::Client,
            resolved_source: EnvironmentValueSource::DotEnv,
            resolved_value: Some(value),
        });
}

pub async fn serve_dev(project: CompiledProject) -> RuntimeResult<()> {
    let servers = start_dev(project).await?;
    servers.wait().await
}

pub async fn serve_production(project: CompiledProject, addr: SocketAddr) -> RuntimeResult<()> {
    serve_production_with_access(project, addr, None).await
}

pub async fn serve_production_with_access(
    project: CompiledProject,
    addr: SocketAddr,
    access: Option<ProductionAccess>,
) -> RuntimeResult<()> {
    let server = start_production_with_access(project, addr, access).await?;
    server.wait().await
}

pub async fn start_production(
    project: CompiledProject,
    addr: SocketAddr,
) -> RuntimeResult<RunningProductionServer> {
    start_production_with_access(project, addr, None).await
}

pub async fn start_production_with_access(
    mut project: CompiledProject,
    addr: SocketAddr,
    access: Option<ProductionAccess>,
) -> RuntimeResult<RunningProductionServer> {
    project.local_databases = false;
    crate::database_bootstrap::prepare_databases(&project).await?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| bind_error(addr, error))?;
    crate::server_actions::execute_server_action(&project.backend.init_action);
    let mut background_jobs = start_init_background_jobs(
        &project.root,
        &project.backend.init_action,
        crate::handlers::CacheRuntimeMode::Production,
    );
    background_jobs.push(transport::start_notification_dispatcher(
        project.root.clone(),
        transport::notification_namespace(&project.root),
    ));
    let addr = listener.local_addr()?;
    let backend_websocket_paths = project
        .backend
        .websockets
        .iter()
        .map(|route| route.path.clone())
        .collect::<Vec<_>>();
    let transport_configs = project.backend.transports.clone();
    let cache_service = project.backend.cache_service;
    let database_service = project.backend.database_service;
    let vector_service = project.backend.vector_service;
    let queue_service = project.backend.queue_service;
    let tls = project.backend.tls.clone();
    let tls_enabled = tls.is_some();
    let project_root = project.root.clone();
    let environment = project.environment_config.clone();
    let state = DevRuntimeState {
        project: Arc::new(RwLock::new(Arc::new(project))),
        events: DevEventBus::default(),
        dev_origins: Vec::new(),
        cache_mode: crate::handlers::CacheRuntimeMode::Production,
    };
    let router = routers::production_router(
        state,
        backend_websocket_paths,
        cache_service,
        database_service,
        vector_service,
        queue_service,
        project_root.clone(),
        access,
    );
    let (shutdown, signal) = oneshot::channel();
    let handle = match tls {
        Some(tls) => crate::tls::spawn_tls_server(
            listener,
            router,
            tls,
            environment,
            project_root.clone(),
            signal,
        ),
        None => spawn_server(listener, router, signal),
    };
    let listeners = transport::spawn_transport_listeners(
        &transport_configs,
        &project_root,
        crate::handlers::CacheRuntimeMode::Production,
    )
    .await?;
    let scheme = if tls_enabled { "https" } else { "http" };
    log_info(format!("Production server started at {scheme}://{addr}"));
    Ok(RunningProductionServer {
        addr,
        transport_addrs: listeners.addrs,
        shutdown: Some(shutdown),
        handle,
        transports: listeners.servers,
        background_jobs,
    })
}

impl RunningDevServers {
    pub fn events(&self) -> DevEventBus {
        self.state.events.clone()
    }

    pub fn runtime_state(&self) -> DevRuntimeState {
        self.state.clone()
    }

    pub async fn shutdown(mut self) -> RuntimeResult<()> {
        self.state.events.emit(
            DevEventType::Shutdown,
            None::<String>,
            None::<String>,
            Vec::new(),
        );
        self.request_shutdown();
        if let Some(server) = self.backend {
            server.handle.await??;
        }
        if let Some(server) = self.views {
            server.handle.await??;
        }
        if let Some(server) = self.desktop {
            server.handle.await??;
        }
        for server in self.backend_transports {
            server.handle.await??;
        }
        for server in self.desktop_transports {
            server.handle.await??;
        }
        Ok(())
    }

    pub async fn wait(mut self) -> RuntimeResult<()> {
        let outcome = match (
            self.backend.is_some(),
            self.views.is_some(),
            self.desktop.is_some(),
        ) {
            (false, false, false) => return Ok(()),
            (true, true, true) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let views = &mut self.views.as_mut().expect("views").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = views => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (true, true, false) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let views = &mut self.views.as_mut().expect("views").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = views => ServerWait::Finished(result),
                }
            }
            (true, false, true) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (false, true, true) => {
                let views = &mut self.views.as_mut().expect("views").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = views => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (true, false, false) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                }
            }
            (false, true, false) => {
                let views = &mut self.views.as_mut().expect("views").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = views => ServerWait::Finished(result),
                }
            }
            (false, false, true) => {
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = desktop => ServerWait::Finished(result),
                }
            }
        };
        self.handle_wait_outcome(outcome).await
    }

    pub fn has_any(&self) -> bool {
        self.backend.is_some() || self.views.is_some() || self.desktop.is_some()
    }

    fn request_shutdown(&mut self) {
        dowe_cache::close_remote_connections();
        for handle in self.background_jobs.drain(..) {
            handle.abort();
        }
        if let Some(server) = &mut self.backend
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        if let Some(server) = &mut self.views
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        if let Some(server) = &mut self.desktop
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        for server in &mut self.backend_transports {
            if let Some(sender) = server.shutdown.take() {
                let _ = sender.send(());
            }
        }
        for server in &mut self.desktop_transports {
            if let Some(sender) = server.shutdown.take() {
                let _ = sender.send(());
            }
        }
    }

    async fn handle_wait_outcome(mut self, outcome: ServerWait) -> RuntimeResult<()> {
        match outcome {
            ServerWait::Signal(signal) => {
                signal.map_err(RuntimeError::from)?;
                self.shutdown().await
            }
            ServerWait::Finished(result) => {
                self.request_shutdown();
                result??;
                Ok(())
            }
        }
    }
}

impl RunningProductionServer {
    pub async fn shutdown(mut self) -> RuntimeResult<()> {
        dowe_cache::close_remote_connections();
        for handle in self.background_jobs.drain(..) {
            handle.abort();
        }
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        for server in &mut self.transports {
            if let Some(shutdown) = server.shutdown.take() {
                let _ = shutdown.send(());
            }
        }
        self.handle.await??;
        for server in self.transports {
            server.handle.await??;
        }
        Ok(())
    }

    pub async fn wait(mut self) -> RuntimeResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(RuntimeError::from)?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                dowe_cache::close_remote_connections();
                result?
            },
        }
    }
}

enum ServerWait {
    Signal(std::io::Result<()>),
    Finished(Result<RuntimeResult<()>, tokio::task::JoinError>),
}

fn spawn_server(
    listener: TcpListener,
    router: Router,
    shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown.await;
            })
            .await
            .map_err(RuntimeError::from)
    })
}

fn bind_error(addr: SocketAddr, error: std::io::Error) -> RuntimeError {
    if addr.port() == 0 {
        RuntimeError::new(error.to_string())
    } else {
        RuntimeError::new(format!("Port {} is unavailable: {error}", addr.port()))
    }
}
