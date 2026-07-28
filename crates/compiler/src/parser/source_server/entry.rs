#[cfg(test)]
pub fn parse_server_file(path: &Path, nodes: &[SourceNode]) -> DoweResult<ServerRoot> {
    let types = TypeRegistry::parse(path, nodes)?;
    parse_server_nodes(
        path,
        nodes,
        &ServerImports::default(),
        &types,
        &EnvironmentConfig::default(),
    )
}

pub fn parse_server_source(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerRoot> {
    let types = TypeRegistry::parse_file(root, file)?;
    let imports = server_imports(root, file, environment)?;
    parse_server_nodes(&file.path, &file.nodes, &imports, &types, environment)
}

pub(crate) fn validate_server_module_source(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    parse_server_module(root, file, environment, &mut Vec::new()).map(|_| ())
}

fn parse_server_nodes(
    path: &Path,
    nodes: &[SourceNode],
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
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
    let desktop_server = child_named(main, "desktop")
        .and_then(|desktop| child_named(desktop, "server"))
        .map(|node| parse_server_config(node, imports, types, environment, ServerTarget::Desktop))
        .transpose()?;

    let databases = database_bindings(imports, &backend, desktop_server.as_ref())?;
    Ok(ServerRoot {
        backend,
        desktop_server,
        databases,
    })
}

#[derive(Debug)]
pub struct ServerRoot {
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
    pub databases: Vec<DatabaseBinding>,
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

#[derive(Default)]
struct ServerImports {
    handlers: HashMap<String, ServerHandler>,
    middlewares: HashMap<String, ServerMiddleware>,
    callables: HashMap<String, ServerCallable>,
    config_bindings: HashMap<String, ServerConfigBinding>,
    endpoint_groups: HashMap<String, EndpointGroup>,
    entities: HashMap<String, DatabaseEntity>,
    seeders: HashMap<String, DatabaseSeeder>,
}

#[derive(Clone, Default)]
struct EndpointGroup {
    endpoints: Vec<Endpoint>,
    websockets: Vec<WebSocketRoute>,
}

