use crate::error::{RuntimeError, RuntimeResult};
use crate::handlers::{
    backend_handler, desktop_handler, desktop_websocket_handler, dev_websocket_handler,
    views_handler, websocket_handler,
};
use crate::logging::log_info;
use crate::production_handlers::production_handler;
use crate::server_actions::execute_server_action;
use crate::{DevEventBus, DevEventType};
use axum::Router;
use axum::routing::get;
use dowe_compiler::CompiledProject;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

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
    state: DevRuntimeState,
    backend: Option<RunningServer>,
    views: Option<RunningServer>,
    desktop: Option<RunningServer>,
}

pub struct RunningProductionServer {
    pub addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<RuntimeResult<()>>,
}

#[derive(Clone)]
pub struct DevRuntimeState {
    pub project: Arc<RwLock<CompiledProject>>,
    pub events: DevEventBus,
    pub dev_origins: Vec<String>,
}

struct RunningServer {
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<RuntimeResult<()>>,
}

pub async fn start_dev(project: CompiledProject) -> RuntimeResult<RunningDevServers> {
    start_dev_servers(project, DevServerTargets::all()).await
}

pub async fn start_dev_servers(
    project: CompiledProject,
    targets: DevServerTargets,
) -> RuntimeResult<RunningDevServers> {
    if targets.backend {
        log_info("Backend server starting");
    }
    if targets.views {
        log_info("Views server starting");
    }
    if targets.desktop {
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
        Some(TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?)
    } else {
        None
    };
    let desktop_listener = match (targets.desktop, project.desktop_server.as_ref()) {
        (true, Some(server)) => {
            let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, server.port));
            Some(
                TcpListener::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?,
            )
        }
        _ => None,
    };

    if targets.backend {
        execute_server_action(&project.backend.init_action);
    }
    if targets.desktop
        && let Some(server) = &project.desktop_server
    {
        execute_server_action(&server.init_action);
    }

    let mut backend = None;
    let mut backend_addr = None;
    let mut views = None;
    let mut views_addr = None;
    let mut desktop = None;
    let mut desktop_addr = None;
    let mut dev_origins = Vec::new();

    if let Some(listener) = &views_listener {
        let addr = listener.local_addr()?;
        dev_origins.push(format!("http://{addr}"));
    }
    if let Some(listener) = &desktop_listener {
        let addr = listener.local_addr()?;
        dev_origins.push(format!("http://{addr}"));
    }

    let state = DevRuntimeState {
        project: Arc::new(RwLock::new(project)),
        events: DevEventBus::default(),
        dev_origins,
    };

    if let Some(listener) = backend_listener {
        let addr = listener.local_addr()?;
        let router = backend_router(state.clone());
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_info(format!("Backend server started at http://{addr}"));
        backend_addr = Some(addr);
        backend = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
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
        let router = desktop_router(state.clone());
        let (shutdown, signal) = oneshot::channel();
        let handle = spawn_server(listener, router, signal);
        log_info(format!("Desktop server started at http://{addr}"));
        desktop_addr = Some(addr);
        desktop = Some(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
    }

    Ok(RunningDevServers {
        backend_addr,
        views_addr,
        desktop_addr,
        state,
        backend,
        views,
        desktop,
    })
}

pub async fn serve_dev(project: CompiledProject) -> RuntimeResult<()> {
    let servers = start_dev(project).await?;
    servers.wait().await
}

pub async fn serve_production(project: CompiledProject, addr: SocketAddr) -> RuntimeResult<()> {
    let server = start_production(project, addr).await?;
    server.wait().await
}

pub async fn start_production(
    project: CompiledProject,
    addr: SocketAddr,
) -> RuntimeResult<RunningProductionServer> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| bind_error(addr, error))?;
    execute_server_action(&project.backend.init_action);
    let addr = listener.local_addr()?;
    let state = DevRuntimeState {
        project: Arc::new(RwLock::new(project)),
        events: DevEventBus::default(),
        dev_origins: Vec::new(),
    };
    let router = production_router(state);
    let (shutdown, signal) = oneshot::channel();
    let handle = spawn_server(listener, router, signal);
    log_info(format!("Production server started at http://{addr}"));
    Ok(RunningProductionServer {
        addr,
        shutdown: Some(shutdown),
        handle,
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
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle.await?
    }

    pub async fn wait(mut self) -> RuntimeResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(RuntimeError::from)?;
                self.shutdown().await
            }
            result = &mut self.handle => result?,
        }
    }
}

enum ServerWait {
    Signal(std::io::Result<()>),
    Finished(Result<RuntimeResult<()>, tokio::task::JoinError>),
}

fn backend_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .fallback(backend_handler)
        .with_state(state)
}

fn views_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .fallback(views_handler)
        .with_state(state)
}

fn desktop_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/ws", get(desktop_websocket_handler))
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .fallback(desktop_handler)
        .with_state(state)
}

fn production_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/ws", get(websocket_handler))
        .fallback(production_handler)
        .with_state(state)
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
