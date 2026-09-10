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

