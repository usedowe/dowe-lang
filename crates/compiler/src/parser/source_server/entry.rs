#[cfg(test)]
pub fn parse_server_file(path: &Path, nodes: &[SourceNode]) -> DoweResult<ServerRoot> {
    let types = TypeRegistry::parse(path, nodes)?;
    parse_server_nodes(
        path,
        nodes,
        &ServerImports::default(),
        &types,
        &EnvironmentConfig::default(),
        true,
    )
}

pub fn parse_server_source(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerRoot> {
    parse_server_source_for(root, file, environment, true)
}

pub fn parse_server_source_without_seeders(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerRoot> {
    parse_server_source_for(root, file, environment, false)
}

fn parse_server_source_for(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    include_seeders: bool,
) -> DoweResult<ServerRoot> {
    let types = TypeRegistry::parse_file_with_import_filter(
        root,
        file,
        &|file, import| !include_seeders && seeder_import_names(file).contains(&import.local),
    )?;
    let imports = server_imports(root, file, environment, include_seeders)?;
    parse_server_nodes(
        &file.path,
        &file.nodes,
        &imports,
        &types,
        environment,
        include_seeders,
    )
}

pub(crate) fn validate_server_module_source(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    parse_server_module(
        root,
        file,
        environment,
        &mut Vec::new(),
        true,
        &mut HashMap::new(),
    )
    .map(|_| ())
}

fn parse_server_nodes(
    path: &Path,
    nodes: &[SourceNode],
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    include_seeders: bool,
) -> DoweResult<ServerRoot> {
    if let Some(app) = nodes.iter().find(|node| node.name == "app") {
        return Err(node_error(app, "`app` has been renamed to `main`"));
    }
    let main = single_root(path, nodes, "main")?;
    if let Some(backend) = child_named(main, "backend") {
        return Err(node_error(
            backend,
            "`backend` has been renamed to `server`",
        ));
    }
    let backend = child_named(main, "server")
        .map(|node| parse_server_config(node, imports, types, environment, ServerTarget::Server))
        .transpose()?
        .unwrap_or_default();
    let desktop = child_named(main, "desktop");
    let desktop_server = desktop
        .and_then(|desktop| child_named(desktop, "server"))
        .map(|node| parse_server_config(node, imports, types, environment, ServerTarget::Desktop))
        .transpose()?;
    let (ipc_functions, ipc_databases) = child_named(main, "ipc")
        .map(|node| parse_native_ipc_functions(node, imports))
        .transpose()?
        .unwrap_or_default();
    let native_ipc = NativeIpcConfig {
        functions: ipc_functions,
        databases: ipc_databases,
    };

    let databases = database_bindings(imports, &backend, desktop_server.as_ref(), &native_ipc)?;
    let inspector = build_server_inspector(
        path,
        nodes,
        &backend,
        &databases,
        &imports.excluded_seeder_paths,
        include_seeders,
    )?;
    Ok(ServerRoot {
        backend,
        desktop_server,
        native_ipc,
        databases,
        inspector,
    })
}

#[derive(Debug)]
pub struct ServerRoot {
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
    pub native_ipc: NativeIpcConfig,
    pub databases: Vec<DatabaseBinding>,
    pub inspector: ServerInspectorManifest,
}

#[derive(Debug, Clone)]
struct ServerHandler {
    action: ServerAction,
    behavior: EndpointBehavior,
}

#[derive(Debug, Clone)]
struct ServerCallable {
    name: String,
    action: ServerFunctionAction,
}

#[derive(Debug, Clone)]
struct ServerConfigBinding {
    statement: ServerStatement,
}

#[derive(Clone, Copy)]
enum ServerTarget {
    Server,
    Desktop,
}

#[derive(Clone, Default)]
struct ServerImports {
    handlers: HashMap<String, ServerHandler>,
    middlewares: HashMap<String, ServerMiddleware>,
    callables: HashMap<String, ServerCallable>,
    config_bindings: HashMap<String, ServerConfigBinding>,
    endpoint_groups: HashMap<String, EndpointGroup>,
    entities: HashMap<String, DatabaseEntity>,
    seeders: HashMap<String, DatabaseSeeder>,
    excluded_seeder_paths: HashSet<std::path::PathBuf>,
}

#[derive(Clone, Default)]
struct EndpointGroup {
    endpoints: Vec<Endpoint>,
    websockets: Vec<WebSocketRoute>,
}
