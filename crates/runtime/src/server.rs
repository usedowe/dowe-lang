use crate::background_jobs::start_init_background_jobs;
use crate::error::{RuntimeError, RuntimeResult};
use crate::handlers::{
    backend_declared_websocket_handler, backend_handler, desktop_declared_websocket_handler,
    desktop_handler, dev_websocket_handler, server_inspector_data, server_inspector_execute,
    server_inspector_index, server_inspector_manifest, server_inspector_selection,
    server_inspector_source, views_handler,
};
use crate::logging::log_info;
use crate::production_handlers::{production_declared_websocket_handler, production_handler};
use crate::{DevEventBus, DevEventType, ProductionAccess};
use axum::extract::{Path as AxumPath, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use axum::{Router, middleware};
use dowe_compiler::{CompiledProject, ServerAction, ServerTransport, ServerTransportProtocol};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::compression::CompressionLayer;

const VIEWS_DEV_PORT: u16 = 7654;

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

struct RunningServer {
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<RuntimeResult<()>>,
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
    crate::database_bootstrap::prepare_databases(&project).await?;
    if targets.backend {
        log_info("Backend server starting");
    }
    if targets.views {
        log_info("Views server starting");
    }
    if targets.desktop && project.desktop_server.is_some() {
        log_info("Desktop server starting");
    }

    let backend_listener = if targets.backend {
        let backend_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, project.backend.port));
        Some(
            TcpListener::bind(backend_addr)
                .await
                .map_err(|error| bind_error(backend_addr, error))?,
        )
    } else {
        None
    };
    let views_listener = if targets.views {
        Some(bind_views_listener().await?)
    } else {
        None
    };
    let desktop_listener = if targets.desktop {
        if let Some(server) = project.desktop_server.as_ref() {
            let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, server.port));
            Some(
                TcpListener::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?,
            )
        } else {
            None
        }
    } else {
        None
    };

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
        let router = backend_router(
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
        log_info(format!("Backend server started at {scheme}://{addr}"));
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
        let listeners = spawn_transport_listeners(
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
        let router = views_router(state.clone());
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_info(format!("Views server started at http://{addr}"));
        views_addr = Some(addr);
        views = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
    }
    if let Some(listener) = desktop_listener {
        let addr = listener.local_addr()?;
        let router = desktop_router(state.clone(), desktop_websocket_paths);
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_info(format!("Desktop server started at http://{addr}"));
        desktop_addr = Some(addr);
        desktop = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
        let listeners = spawn_transport_listeners(
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

async fn bind_views_listener() -> RuntimeResult<TcpListener> {
    for port in VIEWS_DEV_PORT..=u16::MAX {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        match TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(error) => return Err(bind_error(addr, error)),
        }
    }
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, VIEWS_DEV_PORT));
    Err(RuntimeError::new(format!(
        "no available views development port at or above {VIEWS_DEV_PORT} on {addr}"
    )))
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
    let background_jobs = start_init_background_jobs(
        &project.root,
        &project.backend.init_action,
        crate::handlers::CacheRuntimeMode::Production,
    );
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
    let router = production_router(
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
    let listeners = spawn_transport_listeners(
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

struct RunningTransportListeners {
    addrs: Vec<(String, SocketAddr)>,
    servers: Vec<RunningServer>,
}

async fn spawn_transport_listeners(
    transports: &[ServerTransport],
    root: &std::path::Path,
    cache_mode: crate::handlers::CacheRuntimeMode,
) -> RuntimeResult<RunningTransportListeners> {
    let mut addrs = Vec::new();
    let mut servers = Vec::new();
    for transport in transports {
        let addr = transport_addr(transport)?;
        let (shutdown, signal) = oneshot::channel();
        let (actual_addr, handle) = match transport.protocol {
            ServerTransportProtocol::Udp => {
                let socket = UdpSocket::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?;
                let actual_addr = socket.local_addr()?;
                (
                    actual_addr,
                    spawn_udp_transport(
                        socket,
                        transport.clone(),
                        root.to_path_buf(),
                        cache_mode,
                        signal,
                    ),
                )
            }
            ServerTransportProtocol::Tcp => {
                let listener = TcpListener::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?;
                let actual_addr = listener.local_addr()?;
                (
                    actual_addr,
                    spawn_tcp_transport(
                        listener,
                        transport.clone(),
                        root.to_path_buf(),
                        cache_mode,
                        signal,
                    ),
                )
            }
        };
        log_info(format!(
            "{} transport `{}` started at {}",
            transport.protocol.as_str(),
            transport.name,
            actual_addr
        ));
        addrs.push((transport.name.clone(), actual_addr));
        servers.push(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
    }
    Ok(RunningTransportListeners { addrs, servers })
}

fn transport_addr(transport: &ServerTransport) -> RuntimeResult<SocketAddr> {
    format!("{}:{}", transport.bind, transport.port)
        .parse::<SocketAddr>()
        .map_err(|error| RuntimeError::new(format!("invalid transport bind address: {error}")))
}

fn spawn_udp_transport(
    socket: UdpSocket,
    transport: ServerTransport,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    mut shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        let mut buffer = vec![0_u8; 65_535];
        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                received = socket.recv_from(&mut buffer) => {
                    let (len, addr) = received?;
                    crate::background_jobs::launch_task_statements(
                        &root,
                        &transport.action,
                        cache_mode,
                    );
                    execute_transport_action(&transport.action, &transport.binding, &buffer[..len], addr);
                }
            }
        }
    })
}

fn spawn_tcp_transport(
    listener: TcpListener,
    transport: ServerTransport,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    mut shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                accepted = listener.accept() => {
                    let (stream, addr) = accepted?;
                    let action = transport.action.clone();
                    let binding = transport.binding.clone();
                    let root = root.clone();
                    tokio::spawn(async move {
                        let _ = handle_tcp_connection(
                            stream,
                            action,
                            binding,
                            root,
                            cache_mode,
                            addr,
                        )
                        .await;
                    });
                }
            }
        }
    })
}

async fn handle_tcp_connection(
    mut stream: tokio::net::TcpStream,
    action: ServerAction,
    binding: String,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    addr: SocketAddr,
) -> RuntimeResult<()> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    crate::background_jobs::launch_task_statements(&root, &action, cache_mode);
    execute_transport_action(&action, &binding, &buffer, addr);
    Ok(())
}

fn execute_transport_action(action: &ServerAction, binding: &str, bytes: &[u8], addr: SocketAddr) {
    let text = String::from_utf8_lossy(bytes).to_string();
    let byte_len = bytes.len().to_string();
    let addr = addr.to_string();
    execute_server_action_with_transport_resolver(action, binding, &text, &byte_len, &addr);
}

fn execute_server_action_with_transport_resolver(
    action: &ServerAction,
    binding: &str,
    text: &str,
    bytes: &str,
    addr: &str,
) {
    crate::server_actions::execute_server_action_with_resolver(action, |reference| {
        resolve_transport_reference(reference, binding, text, bytes, addr)
    });
}

fn resolve_transport_reference(
    reference: &str,
    binding: &str,
    text: &str,
    bytes: &str,
    addr: &str,
) -> Option<String> {
    if reference == binding || reference == format!("{binding}.text") {
        Some(text.to_string())
    } else if reference == format!("{binding}.bytes") {
        Some(bytes.to_string())
    } else if reference == format!("{binding}.addr") {
        Some(addr.to_string())
    } else {
        None
    }
}

fn backend_router(
    state: DevRuntimeState,
    websocket_paths: Vec<String>,
    cache_service: bool,
    database_service: bool,
    vector_service: bool,
    queue_service: bool,
    project_root: std::path::PathBuf,
) -> Router {
    let mut router = Router::new()
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .route("/_dowe/dev/server", get(server_inspector_index))
        .route("/_dowe/dev/server/", get(server_inspector_index))
        .route(
            "/_dowe/dev/server/manifest.json",
            get(server_inspector_manifest),
        )
        .route(
            "/_dowe/dev/server/source/{id}",
            get(server_inspector_source),
        )
        .route("/_dowe/dev/server/data/{kind}", get(server_inspector_data))
        .route(
            "/_dowe/dev/server/execute",
            axum::routing::post(server_inspector_execute),
        )
        .route("/_dowe/dev/server/events", get(dev_websocket_handler))
        .route(
            "/_dowe/dev/server/selection",
            axum::routing::post(server_inspector_selection),
        );
    if cache_service {
        router = router.route("/v1/caches/{name}", get(cache_service_handler));
    }
    if vector_service {
        router = router.route("/v1/vectors/{name}", get(vector_service_handler));
    }
    if queue_service {
        router = router.route("/v1/queues/{name}", get(queue_service_handler));
    }
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        backend_declared_websocket_handler(state, upgrade, uri, headers, path).await
                    }
                },
            ),
        );
    }
    let router = router.fallback(backend_handler).with_state(state);
    if database_service {
        router.merge(dowe_database::database_service_router(project_root))
    } else {
        router
    }
}

fn views_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .fallback(views_handler)
        .with_state(state)
}

fn desktop_router(state: DevRuntimeState, websocket_paths: Vec<String>) -> Router {
    let mut router = Router::new().route("/_dowe/dev/ws", get(dev_websocket_handler));
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        desktop_declared_websocket_handler(state, upgrade, uri, headers, path).await
                    }
                },
            ),
        );
    }
    router.fallback(desktop_handler).with_state(state)
}

fn production_router(
    state: DevRuntimeState,
    websocket_paths: Vec<String>,
    cache_service: bool,
    database_service: bool,
    vector_service: bool,
    queue_service: bool,
    project_root: std::path::PathBuf,
    access: Option<ProductionAccess>,
) -> Router {
    let mut router = Router::new();
    if cache_service {
        router = router.route("/v1/caches/{name}", get(cache_service_handler));
    }
    if vector_service {
        router = router.route("/v1/vectors/{name}", get(vector_service_handler));
    }
    if queue_service {
        router = router.route("/v1/queues/{name}", get(queue_service_handler));
    }
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        production_declared_websocket_handler(state, upgrade, uri, headers, path)
                            .await
                    }
                },
            ),
        );
    }
    let router = router.fallback(production_handler).with_state(state);
    let router = if database_service {
        router.merge(dowe_database::database_service_router(project_root))
    } else {
        router
    };
    let router = router.layer(CompressionLayer::new().br(true).gzip(true));
    if let Some(access) = access {
        router.layer(middleware::from_fn_with_state(
            access,
            crate::production_access::require_production_access,
        ))
    } else {
        router
    }
}

async fn cache_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_cache::cache_service_upgrade(project.root.clone(), name, headers, upgrade).await
}

async fn vector_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_vector::vector_service_upgrade(project.root.clone(), name, headers, upgrade).await
}

async fn queue_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_queue::queue_service_upgrade(project.root.clone(), name, headers, upgrade).await
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
