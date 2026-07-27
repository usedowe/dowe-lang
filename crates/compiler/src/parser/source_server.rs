use crate::CronSchedule;
use crate::error::{DoweError, DoweResult};
use crate::model::{
    AgentChatTransform, AgentResponseEndpoint, CacheConnection, CorsConfig, DatabaseBinding,
    DatabaseEntity, DatabaseSeeder, DoweType, DoweTypeField, Endpoint, EndpointBehavior,
    EnvironmentConfig, EnvironmentVisibility, HttpActionJsonEndpoint, HttpBytesEndpoint,
    HttpConnectionValue, HttpHeaderValue, HttpMethod, HttpProxyEndpoint, HttpRedirectPolicy,
    HttpResponseMode, OutboundHttpHeader, OutboundHttpRequest, ResponseCookie, ResponseHeader,
    RtpConfig, ServerAction, ServerBackgroundJob, ServerCallStatement, ServerConfig,
    ServerCryptoAesCtrStatement, ServerCryptoCencAesCtrStatement, ServerFunctionAction,
    ServerFunctionParameter, ServerFunctionReturn, ServerJwtStatement, ServerLog, ServerLogLevel,
    ServerLogValue, ServerMiddleware, ServerMiddlewareAction, ServerMiddlewareResponseBody,
    ServerMiddlewareStatement, ServerModel, ServerModelEngine, ServerModelFormat, ServerModelKind,
    ServerSecret, ServerSpawnStatement, ServerStatement, ServerStdlibStatement, ServerTransport,
    ServerTransportProtocol, ServerVectorStatement, StoreConnection, StoreLiteral, TlsConfig,
    TlsDomainsSource, TlsMode, WebSocketHandlers, WebSocketJsonStatement, WebSocketRoute,
    WebSocketSendJsonStatement, WebSocketSseBridgeStatement, normalize_http_header_name,
};
use crate::parser::source_ast::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
use crate::parser::source_config::{parse_desktop_cors_config, parse_server_cors_config};
use crate::parser::source_db::{
    database_action_endpoint_behavior, database_endpoint_behavior, parse_database_entity,
    parse_database_seeder, parse_database_statement, store_literal,
};
use crate::parser::source_imports::resolve_import;
use crate::parser::source_kv::{
    infer_kv_statement, kv_action_endpoint_behavior, parse_kv_statement, validate_kv_handles,
    validate_kv_statement_references,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_stdlib::{dowe_type_from_stdlib_return, parse_stdlib_call};
use crate::parser::source_types::{
    TypeRegistry, is_shared_type_path, type_from_store_literal, validate_reference_path,
};
use crate::parser::source_vector::{
    infer_vector_statement, parse_vector_statement, validate_vector_handles,
    validate_vector_statement_references, vector_action_endpoint_behavior,
};
use dowe_stdlib::StdlibSurface;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

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

fn server_imports(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerImports> {
    let mut imports = ServerImports::default();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if is_shared_type_path(root, &path) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let module_file = parse_source_file(root, &path, source)?;
        if module_has_views_export(&module_file, &import.local) {
            continue;
        }
        let module = parse_server_module(root, &module_file, environment, &mut Vec::new())?;
        if let Some(handler) = module.handlers.get(&import.local).cloned() {
            if imports
                .handlers
                .insert(import.local.clone(), handler)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate handler import `{}`", import.local),
                ));
            }
        } else if let Some(middleware) = module.middlewares.get(&import.local).cloned() {
            if imports
                .middlewares
                .insert(import.local.clone(), middleware)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate middleware import `{}`", import.local),
                ));
            }
        } else if let Some(binding) = module.config_bindings.get(&import.local).cloned() {
            if imports
                .config_bindings
                .insert(import.local.clone(), binding)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate config import `{}`", import.local),
                ));
            }
        } else if let Some(callable) = module.callables.get(&import.local).cloned() {
            if imports
                .callables
                .insert(import.local.clone(), callable)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate server function import `{}`", import.local),
                ));
            }
        } else if let Some(group) = module.endpoint_groups.get(&import.local).cloned() {
            if imports
                .endpoint_groups
                .insert(import.local.clone(), group)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate endpoints import `{}`", import.local),
                ));
            }
        } else if let Some(entity) = module.entities.get(&import.local).cloned() {
            if imports
                .entities
                .insert(import.local.clone(), entity)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate entity import `{}`", import.local),
                ));
            }
        } else if let Some(seeder) = module.seeders.get(&import.local).cloned() {
            if imports
                .seeders
                .insert(import.local.clone(), seeder)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate seeder import `{}`", import.local),
                ));
            }
        } else {
            return Err(DoweError::at_path(
                &import.location.path,
                format!("server module does not export `{}`", import.local),
            ));
        }
    }
    Ok(imports)
}

fn parse_server_module(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    stack: &mut Vec<std::path::PathBuf>,
) -> DoweResult<ServerImports> {
    if stack.iter().any(|path| path == &file.path) {
        return Err(DoweError::at_path(
            &file.path,
            "cyclic server module import detected",
        ));
    }
    stack.push(file.path.clone());
    let surface = server_module_surface(file);
    let imported = module_imports(root, file, environment, stack, surface)?;
    let types = TypeRegistry::parse_file(root, file)?;
    let mut imports = ServerImports::default();
    let mut available_entities = imported.entities.clone();
    for node in file.nodes.iter().filter(|node| node.name == "entity") {
        let entity = parse_database_entity(node)?;
        if available_entities
            .insert(entity.binding.clone(), entity.clone())
            .is_some()
        {
            return Err(node_error(
                node,
                format!("duplicate entity `{}`", entity.binding),
            ));
        }
        imports.entities.insert(entity.binding.clone(), entity);
    }
    let mut available_seeders = imported.seeders.clone();
    for node in file.nodes.iter().filter(|node| node.name == "seeder") {
        let seeder = parse_database_seeder(node, &available_entities)?;
        if available_seeders
            .insert(seeder.binding.clone(), seeder.clone())
            .is_some()
        {
            return Err(node_error(
                node,
                format!("duplicate seeder `{}`", seeder.binding),
            ));
        }
        imports.seeders.insert(seeder.binding.clone(), seeder);
    }
    for node in &file.nodes {
        match node.name.as_str() {
            "handler" => {
                let (name, handler) = parse_handler_node(node, &types, environment, &imported)?;
                if imports.handlers.insert(name.clone(), handler).is_some() {
                    return Err(node_error(node, format!("duplicate handler `{name}`")));
                }
            }
            "middleware" => {
                let (name, middleware) = parse_middleware_node(file, node, environment, &imported)?;
                if imports
                    .middlewares
                    .insert(name.clone(), middleware)
                    .is_some()
                {
                    return Err(node_error(node, format!("duplicate middleware `{name}`")));
                }
            }
            "fn" => {
                let (name, callable) =
                    parse_server_function_node(node, &types, environment, &imported)?;
                if imports.callables.insert(name.clone(), callable).is_some() {
                    return Err(node_error(node, format!("duplicate server fn `{name}`")));
                }
            }
            "service" => {
                return Err(node_error(
                    node,
                    "`service` was replaced by `fn` in `server/services`",
                ));
            }
            "repository" => {
                return Err(node_error(
                    node,
                    "`repository` was replaced by `fn` in `server/repositories`",
                ));
            }
            "database" | "db" | "cache" | "kv" | "let" | "query" | "vector" => {
                let binding = parse_config_binding_node(
                    node,
                    environment,
                    &available_entities,
                    &available_seeders,
                )?;
                if imports
                    .config_bindings
                    .insert(binding_name(&binding.statement).to_string(), binding)
                    .is_some()
                {
                    return Err(node_error(node, "duplicate config binding"));
                }
            }
            "entity" | "seeder" => {}
            "endpoints" => {
                let (name, group) =
                    parse_endpoint_group_node(node, &imported, &types, environment)?;
                if imports
                    .endpoint_groups
                    .insert(name.clone(), group)
                    .is_some()
                {
                    return Err(node_error(node, format!("duplicate endpoints `{name}`")));
                }
            }
            "type" => {}
            _ => {
                return Err(node_error(
                    node,
                    "server modules only accept `type`, `handler`, `middleware`, `fn`, `endpoints`, `entity`, `seeder`, `database`, `cache`, or `vector` declarations",
                ));
            }
        }
    }
    stack.pop();
    Ok(imports)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServerModuleSurface {
    Handler,
    Middleware,
    Function,
    Config,
    Other,
}

fn server_module_surface(file: &SourceFile) -> ServerModuleSurface {
    if file.nodes.iter().any(|node| {
        matches!(
            node.name.as_str(),
            "database" | "db" | "cache" | "kv" | "let" | "query" | "vector"
        )
    }) {
        ServerModuleSurface::Config
    } else if file.nodes.iter().any(|node| node.name == "handler") {
        ServerModuleSurface::Handler
    } else if file.nodes.iter().any(|node| node.name == "middleware") {
        ServerModuleSurface::Middleware
    } else if file.nodes.iter().any(|node| node.name == "fn") {
        ServerModuleSurface::Function
    } else {
        ServerModuleSurface::Other
    }
}

fn module_imports(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    stack: &mut Vec<std::path::PathBuf>,
    surface: ServerModuleSurface,
) -> DoweResult<ServerImports> {
    let mut imports = ServerImports::default();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if is_shared_type_path(root, &path) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let module_file = parse_source_file(root, &path, source)?;
        let module = parse_server_module(root, &module_file, environment, stack)?;
        if let Some(handler) = module.handlers.get(&import.local).cloned() {
            if imports
                .handlers
                .insert(import.local.clone(), handler)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate handler import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(middleware) = module.middlewares.get(&import.local).cloned() {
            if imports
                .middlewares
                .insert(import.local.clone(), middleware)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate middleware import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(callable) = module.callables.get(&import.local).cloned() {
            if surface == ServerModuleSurface::Config {
                return Err(DoweError::at_path(
                    &import.location.path,
                    "config modules cannot import server functions",
                ));
            }
            if imports
                .callables
                .insert(import.local.clone(), callable)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate server function import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(binding) = module.config_bindings.get(&import.local).cloned() {
            if imports
                .config_bindings
                .insert(import.local.clone(), binding)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate config import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(entity) = module.entities.get(&import.local).cloned() {
            if imports
                .entities
                .insert(import.local.clone(), entity)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate entity import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(seeder) = module.seeders.get(&import.local).cloned() {
            if imports
                .seeders
                .insert(import.local.clone(), seeder)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate seeder import `{}`", import.local),
                ));
            }
            continue;
        }
        return Err(DoweError::at_path(
            &import.location.path,
            format!("server module does not export `{}`", import.local),
        ));
    }
    Ok(imports)
}

fn module_has_views_export(file: &SourceFile, name: &str) -> bool {
    file.nodes.iter().any(|node| {
        node.name == "views"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|export| export == name)
    })
}

fn parse_config_binding_node(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    entities: &HashMap<String, DatabaseEntity>,
    seeders: &HashMap<String, DatabaseSeeder>,
) -> DoweResult<ServerConfigBinding> {
    if let Some(statement) = parse_database_statement(node, Some(environment), entities, seeders)? {
        if !matches!(statement, crate::model::ServerStoreStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support database handle bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_database(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Store(statement),
        });
    }
    if let Some(statement) = parse_kv_statement(node, Some(environment))? {
        if !matches!(statement, crate::model::ServerKvStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support Cache connection bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_kv(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Kv(statement),
        });
    }
    if let Some(statement) = parse_vector_statement(node, Some(environment))? {
        if !matches!(statement, ServerVectorStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support Vector connection bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_vector(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Vector(statement),
        });
    }
    Err(node_error(
        node,
        "config modules only support `database`, `cache`, and `vector` declarations",
    ))
}

fn binding_name(statement: &ServerStatement) -> &str {
    match statement {
        ServerStatement::Store(statement) => binding_name_from_database(statement),
        ServerStatement::Kv(statement) => binding_name_from_kv(statement),
        ServerStatement::Vector(statement) => binding_name_from_vector(statement),
        _ => "",
    }
}

fn binding_name_from_database(statement: &crate::model::ServerStoreStatement) -> &str {
    match statement {
        crate::model::ServerStoreStatement::Handle { connection } => &connection.binding,
        crate::model::ServerStoreStatement::Insert { binding, .. }
        | crate::model::ServerStoreStatement::List { binding, .. }
        | crate::model::ServerStoreStatement::Read { binding, .. }
        | crate::model::ServerStoreStatement::Update { binding, .. }
        | crate::model::ServerStoreStatement::Delete { binding, .. }
        | crate::model::ServerStoreStatement::Query { binding, .. }
        | crate::model::ServerStoreStatement::Transaction { binding, .. } => binding,
    }
}

fn binding_name_from_kv(statement: &crate::model::ServerKvStatement) -> &str {
    match statement {
        crate::model::ServerKvStatement::Handle { connection } => &connection.binding,
        crate::model::ServerKvStatement::Get { binding, .. }
        | crate::model::ServerKvStatement::Set { binding, .. }
        | crate::model::ServerKvStatement::Delete { binding, .. }
        | crate::model::ServerKvStatement::Keys { binding, .. }
        | crate::model::ServerKvStatement::Clear { binding, .. } => binding,
    }
}

fn binding_name_from_vector(statement: &ServerVectorStatement) -> &str {
    match statement {
        ServerVectorStatement::Handle { connection } => &connection.binding,
        ServerVectorStatement::Upsert { binding, .. }
        | ServerVectorStatement::Search { binding, .. }
        | ServerVectorStatement::Read { binding, .. }
        | ServerVectorStatement::Delete { binding, .. }
        | ServerVectorStatement::List { binding, .. } => binding,
    }
}

fn imported_config_statements(imports: &ServerImports) -> Vec<ServerStatement> {
    let mut names = imports.config_bindings.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
        .iter()
        .filter_map(|name| imports.config_bindings.get(name))
        .map(|binding| binding.statement.clone())
        .collect()
}

fn database_bindings(
    imports: &ServerImports,
    backend: &ServerConfig,
    desktop_server: Option<&ServerConfig>,
) -> DoweResult<Vec<DatabaseBinding>> {
    let mut connections = Vec::<StoreConnection>::new();
    for binding in imports.config_bindings.values() {
        collect_database_statement(&binding.statement, &mut connections)?;
    }
    collect_server_config_databases(backend, &mut connections)?;
    if let Some(desktop_server) = desktop_server {
        collect_server_config_databases(desktop_server, &mut connections)?;
    }
    let mut databases = connections
        .into_iter()
        .map(|connection| DatabaseBinding {
            binding: connection.binding.clone(),
            connection,
        })
        .collect::<Vec<_>>();
    databases.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.connection.database.cmp(&right.connection.database))
    });
    Ok(databases)
}

fn collect_server_config_databases(
    config: &ServerConfig,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    collect_database_action(&config.init_action, connections)?;
    for endpoint in &config.endpoints {
        collect_database_action(&endpoint.action, connections)?;
        for middleware in &endpoint.middlewares {
            collect_database_middleware(&middleware.action.statements, connections)?;
        }
    }
    for websocket in &config.websockets {
        collect_database_action(&websocket.handlers.open, connections)?;
        collect_database_action(&websocket.handlers.message, connections)?;
        collect_database_action(&websocket.handlers.close, connections)?;
        collect_database_action(&websocket.handlers.drain, connections)?;
        for middleware in &websocket.middlewares {
            collect_database_middleware(&middleware.action.statements, connections)?;
        }
    }
    for transport in &config.transports {
        collect_database_action(&transport.action, connections)?;
    }
    Ok(())
}

fn collect_database_action(
    action: &ServerAction,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    for statement in &action.statements {
        collect_database_statement(statement, connections)?;
    }
    Ok(())
}

fn collect_database_statement(
    statement: &ServerStatement,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    match statement {
        ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection }) => {
            if !connections.contains(connection) {
                connections.push(connection.clone());
            }
        }
        ServerStatement::Call(call) => {
            for statement in &call.action.statements {
                collect_database_statement(statement, connections)?;
            }
        }
        ServerStatement::Go(job) | ServerStatement::Cron(job) => {
            for statement in &job.action.statements {
                collect_database_statement(statement, connections)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_database_middleware(
    statements: &[ServerMiddlewareStatement],
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    for statement in statements {
        match statement {
            ServerMiddlewareStatement::Call(call) => {
                for statement in &call.action.statements {
                    collect_database_statement(statement, connections)?;
                }
            }
            ServerMiddlewareStatement::IfValid { statements, .. } => {
                collect_database_middleware(statements, connections)?;
            }
            ServerMiddlewareStatement::SessionVerify { database, .. } => {
                if !connections.contains(database) {
                    connections.push(database.clone());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn seed_config_bindings(statements: &[ServerStatement], bindings: &mut HashMap<String, DoweType>) {
    for statement in statements {
        match statement {
            ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            ServerStatement::Kv(crate::model::ServerKvStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            ServerStatement::Vector(ServerVectorStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            _ => {}
        }
    }
}

fn parse_handler_node(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerHandler)> {
    reject_explicit_handler_async(node)?;
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "handler must declare a name"))?;
    let action = parse_action(
        node,
        handler_action_context(node),
        types,
        environment,
        imports,
    )?;
    let behavior = exported_handler_behavior(node, &action)?;
    Ok((name, ServerHandler { action, behavior }))
}

fn parse_middleware_node(
    _file: &SourceFile,
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerMiddleware)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "middleware must declare a name"))?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "middleware declarations use `middleware <name> [params:{ ... }]`; `req` and `next` are implicit",
        ));
    }
    reject_unknown_props(node, &["params"])?;
    let params = middleware_params(node)?;
    let action = parse_middleware_action(node, environment, imports)?;
    Ok((
        name.clone(),
        ServerMiddleware {
            name,
            params,
            action,
        },
    ))
}

fn middleware_params(node: &SourceNode) -> DoweResult<StoreLiteral> {
    let Some(prop) = node.prop("params") else {
        return Ok(StoreLiteral::Object(Vec::new()));
    };
    let params = store_literal(&prop.value)?;
    if matches!(params, StoreLiteral::Object(_)) {
        Ok(params)
    } else {
        Err(prop_error(prop, "middleware params must be an object"))
    }
}

fn parse_server_function_node(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerCallable)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "`fn` must declare a name"))?;
    if node.args.len() != 1 {
        return Err(node_error(node, "`fn` must declare one name"));
    }
    validate_binding_name(node, &name)?;
    let action = parse_server_function_action(node, types, environment, imports)?;
    Ok((name.clone(), ServerCallable { name, action }))
}

fn parse_server_config(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    target: ServerTarget,
) -> DoweResult<ServerConfig> {
    let port = required_port(node)?;
    let mut endpoints = Vec::new();
    let mut websockets = Vec::new();
    let mut transports = Vec::new();
    let mut rtp = None;
    let mut models = Vec::new();
    let mut init_action = ServerAction::empty();
    let mut cors = CorsConfig::default();
    let mut cors_seen = false;
    let mut tls = None;
    let mut database_service = false;
    let mut cache_service = false;
    let mut vector_service = false;

    for child in &node.children {
        match child.name.as_str() {
            "endpoints" => {
                for name in endpoint_group_references(child)? {
                    let group = imports.endpoint_groups.get(&name).ok_or_else(|| {
                        node_error(child, format!("missing endpoints import `{name}`"))
                    })?;
                    endpoints.extend(group.endpoints.clone());
                    websockets.extend(group.websockets.clone());
                }
            }
            "route" => endpoints.extend(parse_route(child, imports, types, environment)?),
            "endpoint" => {
                return Err(node_error(child, "`endpoint` has been renamed to `route`"));
            }
            "websocket" => websockets.push(parse_websocket(child, environment, imports, "", &[])?),
            "udp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Udp,
                environment,
            )?),
            "tcp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Tcp,
                environment,
            )?),
            "rtp" => {
                if rtp.is_some() {
                    return Err(node_error(child, "duplicate `rtp` block"));
                }
                rtp = Some(parse_rtp_config(child)?);
            }
            "model" => models.push(parse_server_model(child)?),
            "database" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`database service` is only supported by `main.server`",
                    ));
                }
                if database_service {
                    return Err(node_error(child, "duplicate `database service` block"));
                }
                parse_database_service(child)?;
                database_service = true;
            }
            "cache" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`cache service` is only supported by `main.server`",
                    ));
                }
                if cache_service {
                    return Err(node_error(child, "duplicate `cache service` block"));
                }
                parse_cache_service(child)?;
                cache_service = true;
            }
            "vector" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`vector service` is only supported by `main.server`",
                    ));
                }
                if vector_service {
                    return Err(node_error(child, "duplicate `vector service` block"));
                }
                parse_vector_service(child)?;
                vector_service = true;
            }
            "cors" => {
                if cors_seen {
                    return Err(node_error(child, "duplicate `cors` block"));
                }
                cors_seen = true;
                cors = match target {
                    ServerTarget::Server => parse_server_cors_config(child)?,
                    ServerTarget::Desktop => parse_desktop_cors_config(child)?,
                };
            }
            "tls" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`tls` is only supported by `main.server`",
                    ));
                }
                if tls.is_some() {
                    return Err(node_error(child, "duplicate `tls` block"));
                }
                tls = Some(parse_tls_config(child)?);
            }
            "init" => {
                init_action = parse_action(child, ActionContext::Init, types, environment, imports)?
            }
            _ => return Err(node_error(child, "unsupported server block")),
        }
    }
    validate_unique_transport_names(node, &transports)?;
    validate_unique_model_names(node, &models)?;
    if database_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/databases/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/databases/:name"))
    {
        return Err(node_error(
            node,
            "`database service` reserves WebSocket path `/v1/databases/:name`",
        ));
    }
    if cache_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/caches/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/caches/:name"))
    {
        return Err(node_error(
            node,
            "`cache service` reserves WebSocket path `/v1/caches/:name`",
        ));
    }
    if vector_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/vectors/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/vectors/:name"))
    {
        return Err(node_error(
            node,
            "`vector service` reserves WebSocket path `/v1/vectors/:name`",
        ));
    }

    Ok(ServerConfig {
        port,
        tls,
        endpoints,
        websockets,
        transports,
        rtp,
        models,
        init_action,
        cors,
        database_service,
        cache_service,
        vector_service,
    })
}

fn parse_database_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Database server uses `database service`",
        ));
    }
    Ok(())
}

fn parse_cache_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Dowe Cache server uses `cache service`",
        ));
    }
    Ok(())
}

fn parse_vector_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Dowe Vector server uses `vector service`",
        ));
    }
    Ok(())
}

fn parse_tls_config(node: &SourceNode) -> DoweResult<TlsConfig> {
    reject_unknown_props(
        node,
        &[
            "mode",
            "domains",
            "email",
            "staging",
            "cache",
            "domainsFrom",
            "refreshSeconds",
        ],
    )?;
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`tls` accepts props only"));
    }
    let mode_prop = node
        .prop("mode")
        .ok_or_else(|| node_error(node, "missing `tls.mode`"))?;
    let mode = match required_static_string_prop(mode_prop)?.as_str() {
        "acme" => TlsMode::Acme,
        "local" => TlsMode::Local,
        _ => return Err(prop_error(mode_prop, "`mode` must be `acme` or `local`")),
    };
    let domains = node
        .prop("domains")
        .map(parse_tls_domains)
        .transpose()?
        .unwrap_or_default();
    let domains_from = node
        .prop("domainsFrom")
        .map(parse_tls_domains_source)
        .transpose()?;
    if domains.is_empty() && domains_from.is_none() {
        return Err(node_error(
            node,
            "`tls` requires `domains` or `domainsFrom`",
        ));
    }
    for domain in &domains {
        validate_tls_domain(node, mode, domain)?;
    }
    let email = node
        .prop("email")
        .map(required_static_string_prop)
        .transpose()?;
    if matches!(mode, TlsMode::Acme) && !email.as_deref().is_some_and(valid_tls_email) {
        return Err(node_error(node, "ACME TLS requires a valid `email`"));
    }
    if matches!(mode, TlsMode::Local) && email.is_some() {
        return Err(node_error(node, "`email` is only supported by ACME TLS"));
    }
    let staging = optional_bool_prop(node, "staging")?.unwrap_or(true);
    if matches!(mode, TlsMode::Local) && node.prop("staging").is_some() {
        return Err(node_error(node, "`staging` is only supported by ACME TLS"));
    }
    let cache = node
        .prop("cache")
        .map(required_static_string_prop)
        .transpose()?
        .unwrap_or_else(|| ".dowe/tls".to_string());
    validate_tls_cache(node, &cache)?;
    let refresh_seconds = match node.prop("refreshSeconds") {
        Some(prop) => {
            let value = required_u64_value(prop, &prop.value, "refreshSeconds")?;
            if !(30..=86_400).contains(&value) {
                return Err(prop_error(
                    prop,
                    "`refreshSeconds` must be between 30 and 86400",
                ));
            }
            value
        }
        None => 60,
    };
    if domains_from.is_none() && node.prop("refreshSeconds").is_some() {
        return Err(node_error(node, "`refreshSeconds` requires `domainsFrom`"));
    }
    Ok(TlsConfig {
        mode,
        domains,
        email,
        staging,
        cache,
        domains_from,
        refresh_seconds,
    })
}

fn parse_tls_domains(prop: &SourceProp) -> DoweResult<Vec<String>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`domains` must be an array of quoted strings",
        ));
    };
    let mut domains = Vec::new();
    for value in values {
        let SourceValue::String(domain) = value else {
            return Err(prop_error(
                prop,
                "`domains` must be an array of quoted strings",
            ));
        };
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            return Err(prop_error(prop, "TLS domains cannot be empty"));
        }
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    Ok(domains)
}

fn parse_tls_domains_source(prop: &SourceProp) -> DoweResult<TlsDomainsSource> {
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(
            prop,
            "`domainsFrom` must be a KV or Database object",
        ));
    };
    let mut values = HashMap::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`domainsFrom` does not support spread"));
        };
        let SourceValue::String(value) = value else {
            return Err(prop_error(
                prop,
                "`domainsFrom` values must be quoted strings",
            ));
        };
        if values.insert(key.as_str(), value.clone()).is_some() {
            return Err(prop_error(prop, format!("duplicate `domainsFrom.{key}`")));
        }
    }
    match (
        values.remove("kv"),
        values.remove("key"),
        values.remove("db"),
        values.remove("table"),
        values.remove("field"),
        values.is_empty(),
    ) {
        (Some(database), Some(key), None, None, None, true)
            if !database.is_empty() && !key.is_empty() =>
        {
            Ok(TlsDomainsSource::Kv { database, key })
        }
        (None, None, Some(database), Some(table), Some(field), true)
            if !database.is_empty() && !table.is_empty() && !field.is_empty() =>
        {
            Ok(TlsDomainsSource::Database {
                database,
                table,
                field,
            })
        }
        _ => Err(prop_error(
            prop,
            "`domainsFrom` must be `{ kv:\"name\" key:\"key\" }` or `{ db:\"name\" table:\"table\" field:\"field\" }`",
        )),
    }
}

fn validate_tls_domain(node: &SourceNode, mode: TlsMode, domain: &str) -> DoweResult<()> {
    let local = domain == "localhost"
        || domain.ends_with(".localhost")
        || matches!(domain, "127.0.0.1" | "::1");
    let valid_dns = domain.parse::<IpAddr>().is_err()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        });
    match mode {
        TlsMode::Acme if local || !valid_dns || !domain.contains('.') => Err(node_error(
            node,
            format!("invalid public ACME domain `{domain}`"),
        )),
        TlsMode::Local if !local => Err(node_error(
            node,
            format!("local TLS does not support public domain `{domain}`"),
        )),
        _ => Ok(()),
    }
}

fn valid_tls_email(value: &str) -> bool {
    let value = value.strip_prefix("mailto:").unwrap_or(value);
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
}

fn validate_tls_cache(node: &SourceNode, cache: &str) -> DoweResult<()> {
    let path = Path::new(cache);
    let inside_dowe = path
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == ".dowe");
    if path.is_absolute()
        || !inside_dowe
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(node_error(node, "`tls.cache` must stay inside `.dowe`"));
    }
    Ok(())
}

fn parse_endpoint_group_node(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<(String, EndpointGroup)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "`endpoints` must declare an export name"))?;
    validate_binding_name(node, &name)?;
    if node.children.is_empty() {
        return Err(node_error(
            node,
            "`endpoints` must declare at least one route",
        ));
    }
    let group = parse_endpoint_group_children(
        &node.children,
        EndpointScope::default(),
        false,
        imports,
        types,
        environment,
    )?;
    if group.endpoints.is_empty() && group.websockets.is_empty() {
        return Err(node_error(
            node,
            "`endpoints` must declare at least one HTTP method or WebSocket",
        ));
    }
    Ok((name, group))
}

#[derive(Clone, Default)]
struct EndpointScope {
    path: String,
    middlewares: Vec<ServerMiddleware>,
}

fn parse_endpoint_group_children(
    nodes: &[SourceNode],
    scope: EndpointScope,
    inside_group: bool,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<EndpointGroup> {
    let mut group = EndpointGroup::default();
    for node in nodes {
        match node.name.as_str() {
            "group" => {
                if inside_group {
                    return Err(node_error(
                        node,
                        "`endpoints` groups cannot contain another `group`; put middleware on the group or its HTTP method",
                    ));
                }
                let child_scope = endpoint_group_scope(node, &scope, imports)?;
                let child_group = parse_endpoint_group_children(
                    &node.children,
                    child_scope,
                    true,
                    imports,
                    types,
                    environment,
                )?;
                group.endpoints.extend(child_group.endpoints);
                group.websockets.extend(child_group.websockets);
            }
            "get" | "post" | "put" | "patch" | "delete" => {
                group.endpoints.push(parse_declared_endpoint_method(
                    node,
                    declared_http_method(node)?,
                    &scope,
                    imports,
                    types,
                    environment,
                )?)
            }
            "websocket" => group.websockets.push(parse_websocket(
                node,
                environment,
                imports,
                &scope.path,
                &scope.middlewares,
            )?),
            "route" | "method" => {
                return Err(node_error(
                    node,
                    "endpoint modules use `group` with lowercase HTTP declarations such as `get path:\"/status\" handler:status`",
                ));
            }
            _ => {
                return Err(node_error(
                    node,
                    "`endpoints` only accepts `group`, lowercase HTTP declarations, or `websocket` blocks",
                ));
            }
        }
    }
    Ok(group)
}

fn endpoint_group_scope(
    node: &SourceNode,
    parent: &EndpointScope,
    imports: &ServerImports,
) -> DoweResult<EndpointScope> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`group` does not accept positional arguments",
        ));
    }
    reject_unknown_props(node, &["path", "middleware"])?;
    let path = node
        .prop("path")
        .map(|prop| endpoint_path_value(prop, false))
        .transpose()?
        .unwrap_or_default();
    Ok(EndpointScope {
        path: join_endpoint_paths(&parent.path, &path),
        middlewares: merge_middlewares(&parent.middlewares, route_middlewares(node, imports)?),
    })
}

fn declared_http_method(node: &SourceNode) -> DoweResult<HttpMethod> {
    match node.name.as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "patch" => Ok(HttpMethod::Patch),
        "delete" => Ok(HttpMethod::Delete),
        _ => Err(node_error(node, "unsupported HTTP method declaration")),
    }
}

fn parse_declared_endpoint_method(
    node: &SourceNode,
    method: HttpMethod,
    scope: &EndpointScope,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "HTTP declarations use `path:\"/...\"` props",
        ));
    }
    reject_unknown_props(
        node,
        &["path", "handler", "middleware", "json", "status", "text"],
    )?;
    let path = node
        .prop("path")
        .ok_or_else(|| node_error(node, "HTTP declarations must declare `path`"))
        .and_then(|prop| endpoint_path_value(prop, true))?;
    let path = join_endpoint_paths(&scope.path, &path);
    if !path.starts_with('/') {
        return Err(node_error(
            node,
            "HTTP route path must resolve to a slash-prefixed path",
        ));
    }
    let middlewares = merge_middlewares(&scope.middlewares, route_middlewares(node, imports)?);
    parse_endpoint_method(
        node,
        method,
        &path,
        imports,
        &middlewares,
        types,
        environment,
    )
}

fn endpoint_path_value(prop: &SourceProp, allow_empty: bool) -> DoweResult<String> {
    let SourceValue::String(path) = &prop.value else {
        return Err(prop_error(prop, "`path` must be a quoted string"));
    };
    let path = path.clone();
    if path.is_empty() && allow_empty {
        return Ok(path);
    }
    if !path.starts_with('/') {
        return Err(prop_error(prop, "`path` must start with `/`"));
    }
    Ok(path)
}

fn join_endpoint_paths(parent: &str, child: &str) -> String {
    match (parent, child) {
        ("", "") => String::new(),
        ("", child) => child.to_string(),
        (parent, "") => parent.to_string(),
        (parent, child) => format!(
            "{}/{}",
            parent.trim_end_matches('/'),
            child.trim_start_matches('/')
        ),
    }
}

fn merge_middlewares(
    parent: &[ServerMiddleware],
    own: Vec<ServerMiddleware>,
) -> Vec<ServerMiddleware> {
    let mut middlewares = parent.to_vec();
    middlewares.extend(own);
    middlewares
}

fn endpoint_group_references(node: &SourceNode) -> DoweResult<Vec<String>> {
    let value = node
        .prop("endpoints")
        .map(|prop| &prop.value)
        .or_else(|| node.args.first())
        .ok_or_else(|| node_error(node, "`endpoints` must reference an imported route group"))?;
    let values = match value {
        SourceValue::Array(values) => {
            if values.is_empty() {
                return Err(node_error(
                    node,
                    "`endpoints` route module list must not be empty",
                ));
            }
            values
                .iter()
                .map(|value| {
                    let SourceValue::Bareword(value) = value else {
                        return Err(node_error(
                            node,
                            "`endpoints` list values must be imported symbols",
                        ));
                    };
                    (!value.is_empty()).then(|| value.clone()).ok_or_else(|| {
                        node_error(node, "`endpoints` list values must be imported symbols")
                    })
                })
                .collect::<DoweResult<Vec<_>>>()?
        }
        value => vec![value.as_required_string().ok_or_else(|| {
            node_error(node, "`endpoints` must reference an imported route group")
        })?],
    };
    let mut seen = HashSet::new();
    for reference in &values {
        if !seen.insert(reference.clone()) {
            return Err(node_error(
                node,
                format!("duplicate endpoints reference `{reference}`"),
            ));
        }
    }
    Ok(values)
}

fn parse_route(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Vec<Endpoint>> {
    let path = required_path_arg(node, "route")?;
    let middlewares = route_middlewares(node, imports)?;
    let mut endpoints = Vec::new();

    for child in &node.children {
        match child.name.as_str() {
            "response" => endpoints.push(Endpoint {
                method: HttpMethod::Get,
                path: path.clone(),
                behavior: EndpointBehavior::StaticText(required_text_prop(child)?),
                action: ServerAction::empty(),
                middlewares: middlewares.clone(),
            }),
            "handler" => {
                reject_explicit_handler_async(child)?;
                let action = parse_action(
                    child,
                    handler_action_context(child),
                    types,
                    environment,
                    imports,
                )?;
                endpoints.push(Endpoint {
                    method: HttpMethod::Get,
                    path: path.clone(),
                    behavior: handler_behavior(child, &path, &action)?,
                    action,
                    middlewares: middlewares.clone(),
                });
            }
            "method" => endpoints.push(parse_method(
                child,
                &path,
                imports,
                &middlewares,
                types,
                environment,
            )?),
            _ => return Err(node_error(child, "unsupported route block")),
        }
    }

    if endpoints.is_empty() {
        return Err(node_error(
            node,
            "route must declare a response, handler, or method",
        ));
    }

    Ok(endpoints)
}

fn route_middlewares(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<Vec<ServerMiddleware>> {
    let Some(prop) = node.prop("middleware") else {
        return Ok(Vec::new());
    };
    let names = match &prop.value {
        SourceValue::Bareword(value) => vec![value.clone()],
        SourceValue::Array(values) => {
            let mut names = Vec::new();
            for value in values {
                let SourceValue::Bareword(name) = value else {
                    return Err(prop_error(prop, "`middleware` values must be references"));
                };
                names.push(name.clone());
            }
            names
        }
        _ => {
            return Err(prop_error(
                prop,
                "`middleware` must be a reference or array",
            ));
        }
    };
    let mut middlewares = Vec::new();
    for name in names {
        let middleware = imports
            .middlewares
            .get(&name)
            .ok_or_else(|| prop_error(prop, format!("unknown middleware import `{name}`")))?;
        middlewares.push(middleware.clone());
    }
    Ok(middlewares)
}

fn parse_method(
    node: &SourceNode,
    path: &str,
    imports: &ServerImports,
    middlewares: &[ServerMiddleware],
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    let method_name = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "method must declare an HTTP method"))?;
    let method = HttpMethod::from_str(&method_name)
        .map_err(|_| node_error(node, format!("unsupported HTTP method `{method_name}`")))?;
    parse_endpoint_method(node, method, path, imports, middlewares, types, environment)
}

fn parse_endpoint_method(
    node: &SourceNode,
    method: HttpMethod,
    path: &str,
    imports: &ServerImports,
    middlewares: &[ServerMiddleware],
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    if let Some(handler_name) = optional_prop_string(node, "handler")? {
        let handler = imports
            .handlers
            .get(&handler_name)
            .ok_or_else(|| node_error(node, format!("unknown handler import `{handler_name}`")))?;
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior: handler.behavior.clone(),
            action: handler.action.clone(),
            middlewares: middlewares.to_vec(),
        });
    }
    let action = parse_action(
        node,
        handler_action_context(node),
        types,
        environment,
        imports,
    )?;
    if has_reference_log(&action)
        && let Some(behavior) = database_action_endpoint_behavior(
            &action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) = database_endpoint_behavior(&action, return_json_ref(node))? {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    let behavior = match method {
        HttpMethod::Get => EndpointBehavior::StaticText(
            return_text(node).unwrap_or_else(|| "List posts".to_string()),
        ),
        HttpMethod::Post => {
            if returns_created_json(node) {
                EndpointBehavior::CreatePostJson
            } else {
                return Err(node_error(
                    node,
                    "POST method must return supported JSON response",
                ));
            }
        }
        _ => return Err(node_error(node, "method behavior is not supported yet")),
    };

    Ok(Endpoint {
        method,
        path: path.to_string(),
        behavior,
        action,
        middlewares: middlewares.to_vec(),
    })
}

fn parse_transport(
    node: &SourceNode,
    protocol: ServerTransportProtocol,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerTransport> {
    let name = required_name_prop(node)?;
    let bind = optional_bind_prop(node)?;
    let port = required_transport_port(node)?;
    let expected = match protocol {
        ServerTransportProtocol::Udp => "packet",
        ServerTransportProtocol::Tcp => "connection",
    };
    let mut action = ServerAction::empty();
    let mut binding = expected.to_string();
    let mut seen = false;

    for child in &node.children {
        if child.name != expected {
            return Err(node_error(
                child,
                format!(
                    "{} transport only accepts `{expected}` block",
                    protocol.as_str()
                ),
            ));
        }
        if seen {
            return Err(node_error(child, format!("duplicate `{expected}` block")));
        }
        seen = true;
        binding = child
            .args
            .first()
            .and_then(SourceValue::as_string_like)
            .unwrap_or_else(|| expected.to_string());
        validate_binding_name(child, &binding)?;
        let imports = ServerImports::default();
        action = parse_action(
            child,
            ActionContext::Protocol { binding: &binding },
            &TypeRegistry::empty(),
            environment,
            &imports,
        )?;
    }

    Ok(ServerTransport {
        name,
        protocol,
        bind,
        port,
        action,
        binding,
    })
}

fn parse_rtp_config(node: &SourceNode) -> DoweResult<RtpConfig> {
    reject_unknown_props(node, &["bind", "min", "max"])?;
    let bind = optional_bind_prop(node)?;
    let min = required_port_prop(node, "min")?;
    let max = required_port_prop(node, "max")?;
    if min > max {
        return Err(node_error(
            node,
            "`rtp` min must be less than or equal to max",
        ));
    }
    Ok(RtpConfig { bind, min, max })
}

fn parse_server_model(node: &SourceNode) -> DoweResult<ServerModel> {
    reject_unknown_props(
        node,
        &["name", "kind", "engine", "format", "source", "sampleRates"],
    )?;
    let name = required_name_prop(node)?;
    let kind = required_model_kind_prop(node)?;
    let engine = required_model_engine_prop(node)?;
    let format = required_model_format_prop(node)?;
    let source = optional_model_source_prop(node, format)?;
    let sample_rates = optional_sample_rates_prop(node)?;
    match (engine, format) {
        (ServerModelEngine::Candle, ServerModelFormat::Onnx)
        | (ServerModelEngine::Energy, ServerModelFormat::Builtin) => {}
        (ServerModelEngine::Candle, _) => {
            return Err(node_error(node, "candle models must use format:\"onnx\""));
        }
        (ServerModelEngine::Energy, _) => {
            return Err(node_error(
                node,
                "energy models must use format:\"builtin\"",
            ));
        }
    }
    Ok(ServerModel {
        name,
        kind,
        engine,
        format,
        source,
        sample_rates,
    })
}

fn validate_unique_transport_names(
    node: &SourceNode,
    transports: &[ServerTransport],
) -> DoweResult<()> {
    let mut names = Vec::<&str>::new();
    for transport in transports {
        if names.iter().any(|name| *name == transport.name) {
            return Err(node_error(
                node,
                format!("duplicate transport `{}`", transport.name),
            ));
        }
        names.push(&transport.name);
    }
    Ok(())
}

fn validate_unique_model_names(node: &SourceNode, models: &[ServerModel]) -> DoweResult<()> {
    let mut names = Vec::<&str>::new();
    for model in models {
        if names.iter().any(|name| *name == model.name) {
            return Err(node_error(
                node,
                format!("duplicate model `{}`", model.name),
            ));
        }
        names.push(&model.name);
    }
    Ok(())
}

fn parse_websocket(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
    parent_path: &str,
    inherited_middlewares: &[ServerMiddleware],
) -> DoweResult<WebSocketRoute> {
    let path = if let Some(prop) = node.prop("path") {
        if !node.args.is_empty() {
            return Err(node_error(
                node,
                "`websocket` uses either `path:\"/...\"` or one path argument",
            ));
        }
        endpoint_path_value(prop, false)?
    } else {
        required_path_arg(node, "websocket")?
    };
    let path = join_endpoint_paths(parent_path, &path);
    let middlewares = merge_middlewares(inherited_middlewares, route_middlewares(node, imports)?);
    let mut handlers = WebSocketHandlers::default();

    for child in &node.children {
        let imports = ServerImports::default();
        let action = parse_action(
            child,
            ActionContext::WebSocket,
            &TypeRegistry::empty(),
            environment,
            &imports,
        )?;
        match child.name.as_str() {
            "open" => handlers.open = action,
            "message" => handlers.message = action,
            "close" => handlers.close = action,
            "drain" => handlers.drain = action,
            _ => return Err(node_error(child, "unsupported WebSocket handler")),
        }
    }

    Ok(WebSocketRoute {
        path,
        handlers,
        middlewares,
    })
}

fn parse_action(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerAction> {
    let mut statements = Vec::new();
    let mut returned = false;
    let mut inferred_bindings = HashMap::<String, DoweType>::new();
    let mut inferred_tables = HashMap::<String, DoweType>::new();
    let config_statements = imported_config_statements(imports);
    seed_config_bindings(&config_statements, &mut inferred_bindings);
    if let ActionContext::Protocol { binding } = context {
        inferred_bindings.insert(binding.to_string(), DoweType::Unknown);
    }

    for child in &node.children {
        match child.name.as_str() {
            "database" | "db" | "cache" | "kv" | "query" | "vector" | "emb" => {
                if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else if let Some(statement) = parse_vector_statement(child, Some(environment))? {
                    validate_vector_statement_references(child, &statement, &inferred_bindings)?;
                    infer_vector_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Vector(statement));
                } else {
                    return Err(node_error(
                        child,
                        "invalid Database, Cache, or Vector declaration",
                    ));
                }
            }
            "spawn" => {
                let statement = parse_spawn_declaration(child)?;
                validate_spawn_statement_references(child, &statement, &inferred_bindings)?;
                infer_spawn_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Spawn(statement));
            }
            "http" => {
                let statement = parse_http_declaration(child, context, environment)?;
                validate_http_statement_references(child, &statement, &inferred_bindings)?;
                infer_http_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Http(statement));
            }
            "crypto" => {
                let statement = parse_crypto_declaration(child)?;
                match &statement {
                    ServerStatement::CryptoAesCtr(statement) => {
                        validate_crypto_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    ServerStatement::CryptoCencAesCtr(statement) => {
                        validate_crypto_cenc_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_cenc_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    _ => unreachable!(),
                }
                statements.push(statement);
            }
            "request" => {
                let statement = parse_request_declaration(child, context)?;
                infer_request_metadata_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "ws" => {
                let statement = parse_websocket_json_declaration(child, context)?;
                infer_websocket_json_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::WebSocketJson(statement));
            }
            "agent" => {
                let statement = parse_agent_chat_declaration(child)?;
                validate_reference_path(child, &statement.source, &inferred_bindings)?;
                infer_agent_chat_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::AgentChat(statement));
            }
            name if dowe_stdlib::is_stdlib_namespace(name) => {
                let statement = parse_stdlib_declaration(child)?;
                validate_stdlib_statement_references(child, &statement, &inferred_bindings)?;
                infer_stdlib_statement(&statement, &mut inferred_bindings)?;
                statements.push(ServerStatement::Stdlib(statement));
            }
            "let" => {
                if legacy_request_json_let(child) {
                    return Err(node_error(
                        child,
                        "request JSON uses `const <binding[:Type]> value:req.json`",
                    ));
                } else if let Some(statement) = parse_request_metadata_let(child, context)? {
                    return Err(legacy_request_metadata_error(child, &statement));
                } else if let Some(statement) = parse_websocket_json_let(child, context)? {
                    return Err(node_error(
                        child,
                        format!(
                            "WebSocket JSON uses `ws {} source:\"json\"`",
                            statement.binding
                        ),
                    ));
                } else if let Some(statement) = parse_agent_chat_let(child)? {
                    return Err(node_error(
                        child,
                        format!(
                            "Agent chat uses `agent {} source:\"chat\" request:{}`",
                            statement.binding, statement.source
                        ),
                    ));
                } else if legacy_jwt_let(child) {
                    return Err(node_error(
                        child,
                        "JWT expressions use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>`",
                    ));
                } else if let Some(statement) = parse_stdlib_let(child)? {
                    return Err(legacy_stdlib_error(child, &statement));
                } else if legacy_http_let(child) {
                    return Err(node_error(
                        child,
                        "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
                    ));
                } else if legacy_spawn_let(child) {
                    return Err(node_error(
                        child,
                        "spawn uses `spawn <binding> command:<value> [args:<array>]`",
                    ));
                } else if legacy_crypto_let(child) {
                    return Err(node_error(
                        child,
                        "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
                    ));
                } else if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else {
                    reject_legacy_server_function_call(child, &imports.callables)?;
                    return Err(node_error(
                        child,
                        "`let` assignments are not supported; use `<capability> <binding> <props>`",
                    ));
                }
            }
            "jwt" => {
                let statement = parse_jwt_statement(child, environment)?;
                validate_jwt_statement_references(child, &statement, &inferred_bindings)?;
                infer_jwt_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Jwt(statement));
            }
            "const" => {
                let statement = parse_request_json_const(child, context, types)?;
                infer_request_json_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "return" => {
                validate_return(child, context)?;
                validate_return_references(child, &inferred_bindings)?;
                returned = true;
            }
            "log" | "info" | "warn" | "error" => {
                let log = parse_log(child)?;
                validate_log_references(child, &log, &inferred_bindings)?;
                statements.push(ServerStatement::Log(log));
            }
            "go" => statements.push(ServerStatement::Go(parse_background_job(
                child,
                context,
                &imports.callables,
                false,
            )?)),
            "cron" => statements.push(ServerStatement::Cron(parse_background_job(
                child,
                context,
                &imports.callables,
                true,
            )?)),
            "send" => {
                let statement = parse_websocket_send_json(child, context)?;
                validate_store_literal_references(child, &statement.value, &inferred_bindings)?;
                statements.push(ServerStatement::WebSocketSendJson(statement));
            }
            "bridge" => {
                let statement = parse_websocket_sse_bridge(child, context)?;
                validate_websocket_sse_bridge_references(child, &statement, &inferred_bindings)?;
                statements.push(ServerStatement::WebSocketSseBridge(statement));
            }
            "if" => {
                return Err(node_error(
                    child,
                    "server if is not supported by current contracts",
                ));
            }
            "commit" => return Err(node_error(child, "`commit` is only valid inside store tx")),
            "rollback" => {
                return Err(node_error(
                    child,
                    "`rollback` is only valid inside store tx",
                ));
            }
            _ => {
                if !push_server_function_call(
                    child,
                    context,
                    &imports.callables,
                    &mut inferred_bindings,
                    &mut statements,
                )? {
                    return Err(node_error(child, "unsupported server action"));
                }
            }
        }
    }

    if matches!(context, ActionContext::HttpHandler { .. }) && !returned {
        return Err(node_error(node, "handler must return a response"));
    }

    statements.splice(0..0, config_statements);
    validate_kv_handles(&statements)?;
    validate_vector_handles(&statements)?;
    Ok(ServerAction { statements })
}

fn parse_server_function_action(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerFunctionAction> {
    reject_unknown_props(node, &["params", "return"])?;
    let params = parse_server_function_params(node, types)?;
    let return_type = parse_server_function_return(node, types)?;
    let mut statements = Vec::new();
    let mut return_value = None;
    let mut inferred_bindings = HashMap::<String, DoweType>::new();
    let mut inferred_tables = HashMap::<String, DoweType>::new();
    inferred_bindings.insert(
        "args".to_string(),
        if params.is_empty() {
            DoweType::Unknown
        } else {
            DoweType::Object(
                params
                    .iter()
                    .map(|parameter| DoweTypeField {
                        name: parameter.name.clone(),
                        value: parameter.schema.clone(),
                        optional: false,
                    })
                    .collect(),
            )
        },
    );
    let context = ActionContext::Function;
    let config_statements = imported_config_statements(imports);
    seed_config_bindings(&config_statements, &mut inferred_bindings);

    for child in &node.children {
        match child.name.as_str() {
            "database" | "db" | "cache" | "kv" | "query" | "vector" | "emb" => {
                if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else if let Some(statement) = parse_vector_statement(child, Some(environment))? {
                    validate_vector_statement_references(child, &statement, &inferred_bindings)?;
                    infer_vector_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Vector(statement));
                } else {
                    return Err(node_error(
                        child,
                        "invalid Database, Cache, or Vector declaration",
                    ));
                }
            }
            "spawn" => {
                let statement = parse_spawn_declaration(child)?;
                validate_spawn_statement_references(child, &statement, &inferred_bindings)?;
                infer_spawn_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Spawn(statement));
            }
            "http" => {
                let statement = parse_http_declaration(child, context, environment)?;
                validate_http_statement_references(child, &statement, &inferred_bindings)?;
                infer_http_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Http(statement));
            }
            "crypto" => {
                let statement = parse_crypto_declaration(child)?;
                match &statement {
                    ServerStatement::CryptoAesCtr(statement) => {
                        validate_crypto_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    ServerStatement::CryptoCencAesCtr(statement) => {
                        validate_crypto_cenc_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_cenc_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    _ => unreachable!(),
                }
                statements.push(statement);
            }
            "request" => {
                let statement = parse_request_declaration(child, context)?;
                infer_request_metadata_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "agent" => {
                let statement = parse_agent_chat_declaration(child)?;
                validate_reference_path(child, &statement.source, &inferred_bindings)?;
                infer_agent_chat_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::AgentChat(statement));
            }
            name if dowe_stdlib::is_stdlib_namespace(name) => {
                let statement = parse_stdlib_declaration(child)?;
                validate_stdlib_statement_references(child, &statement, &inferred_bindings)?;
                infer_stdlib_statement(&statement, &mut inferred_bindings)?;
                statements.push(ServerStatement::Stdlib(statement));
            }
            "let" => {
                if legacy_request_json_let(child) {
                    return Err(node_error(
                        child,
                        "request JSON uses `const <binding[:Type]> value:req.json`",
                    ));
                } else if let Some(statement) = parse_request_metadata_let(child, context)? {
                    return Err(legacy_request_metadata_error(child, &statement));
                } else if let Some(statement) = parse_agent_chat_let(child)? {
                    return Err(node_error(
                        child,
                        format!(
                            "Agent chat uses `agent {} source:\"chat\" request:{}`",
                            statement.binding, statement.source
                        ),
                    ));
                } else if legacy_jwt_let(child) {
                    return Err(node_error(
                        child,
                        "JWT expressions use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>`",
                    ));
                } else if let Some(statement) = parse_stdlib_let(child)? {
                    return Err(legacy_stdlib_error(child, &statement));
                } else if legacy_http_let(child) {
                    return Err(node_error(
                        child,
                        "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
                    ));
                } else if legacy_spawn_let(child) {
                    return Err(node_error(
                        child,
                        "spawn uses `spawn <binding> command:<value> [args:<array>]`",
                    ));
                } else if legacy_crypto_let(child) {
                    return Err(node_error(
                        child,
                        "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
                    ));
                } else if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else {
                    reject_legacy_server_function_call(child, &imports.callables)?;
                    return Err(node_error(
                        child,
                        "`let` assignments are not supported; use `<capability> <binding> <props>`",
                    ));
                }
            }
            "jwt" => {
                let statement = parse_jwt_statement(child, environment)?;
                validate_jwt_statement_references(child, &statement, &inferred_bindings)?;
                infer_jwt_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Jwt(statement));
            }
            "const" => {
                let statement = parse_request_json_const(child, context, types)?;
                infer_request_json_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "return" => {
                if return_value.is_some() {
                    return Err(node_error(child, "server fn must return one value"));
                }
                let value = parse_server_function_return_value(child)?;
                validate_store_literal_references(child, &value, &inferred_bindings)?;
                if let Some(return_type) = &return_type {
                    let actual = server_literal_type(&value, &inferred_bindings);
                    if !server_type_assignable(&actual, &return_type.schema) {
                        return Err(node_error(
                            child,
                            format!(
                                "function return value is incompatible with declared return type `{}`",
                                return_type.type_name
                            ),
                        ));
                    }
                }
                return_value = Some(value);
            }
            "log" | "info" | "warn" | "error" => {
                let log = parse_log(child)?;
                validate_log_references(child, &log, &inferred_bindings)?;
                statements.push(ServerStatement::Log(log));
            }
            "go" => statements.push(ServerStatement::Go(parse_background_job(
                child,
                context,
                &imports.callables,
                false,
            )?)),
            "cron" => {
                return Err(node_error(child, "`cron` is only valid inside server init"));
            }
            "if" => {
                return Err(node_error(
                    child,
                    "server if is not supported by current contracts",
                ));
            }
            "send" | "bridge" => {
                return Err(node_error(
                    child,
                    "WebSocket actions are not valid inside server functions",
                ));
            }
            "commit" => return Err(node_error(child, "`commit` is only valid inside store tx")),
            "rollback" => {
                return Err(node_error(
                    child,
                    "`rollback` is only valid inside store tx",
                ));
            }
            _ => {
                if !push_server_function_call(
                    child,
                    context,
                    &imports.callables,
                    &mut inferred_bindings,
                    &mut statements,
                )? {
                    return Err(node_error(child, "unsupported server action"));
                }
            }
        }
    }

    let return_value =
        return_value.ok_or_else(|| node_error(node, "server fn must return value"))?;
    statements.splice(0..0, config_statements);
    validate_kv_handles(&statements)?;
    validate_vector_handles(&statements)?;
    Ok(ServerFunctionAction {
        params,
        return_type,
        statements,
        return_value,
    })
}

fn parse_middleware_action(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareAction> {
    let statements = parse_middleware_statements(&node.children, environment, imports)?;
    if !middleware_returns(&statements) {
        return Err(node_error(
            node,
            "middleware must call `next` or return a response",
        ));
    }
    Ok(ServerMiddlewareAction { statements })
}

fn parse_middleware_statements(
    nodes: &[SourceNode],
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<Vec<ServerMiddlewareStatement>> {
    let mut statements = Vec::new();
    for node in nodes {
        match node.name.as_str() {
            "let" => {
                reject_middleware_let(node, imports)?;
                unreachable!();
            }
            "request" => statements.push(parse_middleware_request_declaration(node)?),
            "bearer" => statements.push(parse_bearer_declaration(node)?),
            "session" => statements.push(parse_session_verify_declaration(node, imports)?),
            "jwt" => statements.push(ServerMiddlewareStatement::Jwt(parse_jwt_statement(
                node,
                environment,
            )?)),
            "const" => {
                return Err(node_error(
                    node,
                    "JWT results use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>` without `const`",
                ));
            }
            "if" => statements.push(parse_middleware_if(node, environment, imports)?),
            "return" => statements.push(parse_middleware_return(node)?),
            "log" | "info" | "warn" | "error" => {
                statements.push(ServerMiddlewareStatement::Log(parse_log(node)?));
            }
            "next" => statements.push(parse_middleware_next(node)?),
            "continue" => return Err(node_error(node, "middleware continuation uses `next`")),
            _ => {
                let statement = parse_server_function_call(
                    node,
                    ActionContext::Middleware,
                    &imports.callables,
                    &HashMap::new(),
                )?
                .ok_or_else(|| node_error(node, "unsupported middleware action"))?;
                statements.push(ServerMiddlewareStatement::Call(statement));
            }
        }
    }
    Ok(statements)
}

fn reject_middleware_let(node: &SourceNode, imports: &ServerImports) -> DoweResult<()> {
    let (binding, expression) =
        assignment(node).ok_or_else(|| node_error(node, "middleware let must assign a value"))?;
    match expression.as_str() {
        "req.header" => {
            let name = required_header_name_prop(node, "name")?;
            Err(node_error(
                node,
                format!(
                    "request headers use `request {binding} source:\"header\" name:\"{name}\"`"
                ),
            ))
        }
        "bearer" => Err(node_error(
            node,
            "bearer extraction uses `bearer <binding> value:req.header.Authorization`",
        )),
        "jwt.verify" | "jwt.decrypt" | "jwt.sign" | "jwt.encrypt" => Err(node_error(
            node,
            "JWT expressions use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>`",
        )),
        "session.verify" => Err(node_error(
            node,
            "session verification uses `session <binding> cache:<cache> database:<database> token:<token> [maxAge:<seconds>]`",
        )),
        _ => {
            reject_legacy_server_function_call(node, &imports.callables)?;
            Err(node_error(
                node,
                "`let` assignments are not supported; use `<capability> <binding> <props>`",
            ))
        }
    }
}

fn parse_middleware_request_declaration(
    node: &SourceNode,
) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "middleware request headers use `request <binding> source:\"header\" name:<header>`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "request requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_middleware_props(node, &["source", "name"])?;
    let source = required_source_selector(node, "request")?;
    if source != "header" {
        return Err(node_error(
            node,
            "middleware request only supports `source:\"header\"`",
        ));
    }
    Ok(ServerMiddlewareStatement::Header {
        binding,
        name: required_header_name_prop(node, "name")?,
    })
}

fn parse_bearer_declaration(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "bearer requires one result binding: `bearer <binding> value:req.header.Authorization`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "bearer requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["value"])?;
    let source = node
        .prop("value")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| node_error(node, "bearer requires `value:req.header.Authorization`"))?;
    let Some(header) = source.strip_prefix("req.header.") else {
        return Err(node_error(
            node,
            "bearer `value` must read a request header such as `req.header.Authorization`",
        ));
    };
    if normalize_http_header_name(header).is_none() {
        return Err(node_error(
            node,
            "bearer `value` uses an invalid header name",
        ));
    }
    Ok(ServerMiddlewareStatement::Bearer { binding, source })
}

fn parse_middleware_if(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareStatement> {
    let condition = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "middleware if must declare a condition"))?;
    let Some(binding) = condition.strip_suffix(".valid") else {
        return Err(node_error(
            node,
            "middleware if only supports validation checks",
        ));
    };
    let statements = parse_middleware_statements(&node.children, environment, imports)?;
    Ok(ServerMiddlewareStatement::IfValid {
        binding: binding.to_string(),
        statements,
    })
}

fn parse_session_verify_declaration(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "session verification uses `session <binding> cache:<cache> database:<database> token:<token> [maxAge:<seconds>]`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "session verification requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_middleware_props(node, &["cache", "database", "token", "maxAge"])?;
    let cache_name = required_middleware_reference(node, "cache")?;
    let database_name = required_middleware_reference(node, "database")?;
    let token = required_middleware_reference(node, "token")?;
    let max_age_seconds = node
        .prop("maxAge")
        .map(|prop| match &prop.value {
            SourceValue::Number(value) => value.parse::<u64>().map_err(|_| {
                node_error(node, "session verify `maxAge` must be a positive integer")
            }),
            _ => Err(node_error(
                node,
                "session verify `maxAge` must be a positive integer",
            )),
        })
        .transpose()?
        .unwrap_or(2_592_000);
    if max_age_seconds == 0 {
        return Err(node_error(
            node,
            "session verify `maxAge` must be a positive integer",
        ));
    }
    let cache = imported_cache_connection(node, imports, &cache_name)?;
    let database = imported_database_connection(node, imports, &database_name)?;
    Ok(ServerMiddlewareStatement::SessionVerify {
        binding,
        cache,
        database,
        token,
        max_age_seconds,
    })
}

fn required_middleware_reference(node: &SourceNode, name: &str) -> DoweResult<String> {
    let value = node
        .prop(name)
        .and_then(|prop| prop.value.as_string_like())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| node_error(node, format!("session verify must declare `{name}`")))?;
    Ok(value)
}

fn reject_unknown_middleware_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("session verify does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn imported_cache_connection(
    node: &SourceNode,
    imports: &ServerImports,
    name: &str,
) -> DoweResult<CacheConnection> {
    match imports
        .config_bindings
        .get(name)
        .map(|binding| &binding.statement)
    {
        Some(ServerStatement::Kv(crate::model::ServerKvStatement::Handle { connection })) => {
            Ok(connection.clone())
        }
        Some(_) => Err(node_error(
            node,
            format!("`{name}` must reference a Cache connection"),
        )),
        None => Err(node_error(
            node,
            format!("Cache connection `{name}` is not imported"),
        )),
    }
}

fn imported_database_connection(
    node: &SourceNode,
    imports: &ServerImports,
    name: &str,
) -> DoweResult<StoreConnection> {
    match imports
        .config_bindings
        .get(name)
        .map(|binding| &binding.statement)
    {
        Some(ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection })) => {
            Ok(connection.clone())
        }
        Some(_) => Err(node_error(
            node,
            format!("`{name}` must reference a Database connection"),
        )),
        None => Err(node_error(
            node,
            format!("Database connection `{name}` is not imported"),
        )),
    }
}

fn parse_middleware_return(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("continue")
    {
        return Err(node_error(node, "middleware continuation uses `next`"));
    }
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("response")
    {
        return Err(node_error(
            node,
            "middleware HTTP returns use `return <props>`; remove `response`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "middleware HTTP returns do not accept positional values",
        ));
    }
    reject_unknown_props(node, &["status", "text", "json"])?;
    let status = return_status_from_node(node)?;
    match (node.prop("text"), node.prop("json")) {
        (Some(prop), None) => Ok(ServerMiddlewareStatement::Response {
            status,
            body: ServerMiddlewareResponseBody::Text(required_static_string_prop(prop)?),
        }),
        (None, Some(prop)) => Ok(ServerMiddlewareStatement::Response {
            status,
            body: ServerMiddlewareResponseBody::Json(store_literal(&prop.value)?),
        }),
        (None, None) => Err(node_error(node, "return must declare text or json")),
        (Some(_), Some(_)) => Err(node_error(
            node,
            "return must declare exactly one of text or json",
        )),
    }
}

fn middleware_returns(statements: &[ServerMiddlewareStatement]) -> bool {
    statements.iter().any(|statement| {
        matches!(
            statement,
            ServerMiddlewareStatement::Next { .. } | ServerMiddlewareStatement::Response { .. }
        )
    })
}

fn parse_middleware_next(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`next` does not accept positional arguments",
        ));
    }
    reject_unknown_props(node, &["context"])?;
    Ok(ServerMiddlewareStatement::Next {
        context: node
            .prop("context")
            .map(|prop| store_literal(&prop.value))
            .transpose()?,
    })
}

fn infer_store_statement(
    statement: &crate::model::ServerStoreStatement,
    bindings: &mut HashMap<String, DoweType>,
    tables: &mut HashMap<String, DoweType>,
) {
    match statement {
        crate::model::ServerStoreStatement::Insert {
            binding,
            table,
            value,
            ..
        } => {
            let mut value_type = type_from_store_literal(value);
            if let DoweType::Object(fields) = &mut value_type
                && !fields.iter().any(|field| field.name == "id")
            {
                fields.push(DoweTypeField {
                    name: "id".to_string(),
                    value: DoweType::String,
                    optional: false,
                });
            }
            tables.insert(table.clone(), value_type.clone());
            bindings.insert(binding.clone(), value_type);
        }
        crate::model::ServerStoreStatement::Read { binding, table, .. } => {
            if let Some(value) = tables.get(table) {
                bindings.insert(binding.clone(), value.clone());
            }
        }
        crate::model::ServerStoreStatement::Update { binding, .. }
        | crate::model::ServerStoreStatement::Delete { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![DoweTypeField {
                    name: "changed".to_string(),
                    value: DoweType::Number,
                    optional: false,
                }]),
            );
        }
        _ => {}
    }
}

fn infer_request_json_statement(
    statement: &ServerStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    if let ServerStatement::RequestJson { binding, schema } = statement {
        bindings.insert(binding.clone(), schema.clone().unwrap_or(DoweType::Unknown));
    }
}

fn infer_request_metadata_statement(
    statement: &ServerStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    match statement {
        ServerStatement::RequestQuery { binding } => {
            bindings.insert(binding.clone(), DoweType::Unknown);
        }
        ServerStatement::RequestRawQuery { binding }
        | ServerStatement::RequestHeader { binding, .. }
        | ServerStatement::RequestCookie { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::String);
        }
        _ => {}
    }
}

fn infer_websocket_json_statement(
    statement: &WebSocketJsonStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_agent_chat_statement(
    statement: &AgentChatTransform,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn validate_jwt_statement_references(
    node: &SourceNode,
    statement: &ServerJwtStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        ServerJwtStatement::Verify { token, .. } | ServerJwtStatement::Decrypt { token, .. } => {
            validate_reference_path(node, token, bindings)
        }
        ServerJwtStatement::Sign { claims, .. } | ServerJwtStatement::Encrypt { claims, .. } => {
            validate_store_literal_references(node, claims, bindings)
        }
    }
}

fn infer_jwt_statement(statement: &ServerJwtStatement, bindings: &mut HashMap<String, DoweType>) {
    match statement {
        ServerJwtStatement::Verify { binding, .. }
        | ServerJwtStatement::Decrypt { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![
                    DoweTypeField {
                        name: "valid".to_string(),
                        value: DoweType::Bool,
                        optional: false,
                    },
                    DoweTypeField {
                        name: "claims".to_string(),
                        value: DoweType::Unknown,
                        optional: false,
                    },
                ]),
            );
        }
        ServerJwtStatement::Sign { binding, .. } | ServerJwtStatement::Encrypt { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::String);
        }
    }
}

fn infer_stdlib_statement(
    statement: &ServerStdlibStatement,
    bindings: &mut HashMap<String, DoweType>,
) -> DoweResult<()> {
    let kind = dowe_stdlib::validate_call(&statement.call, StdlibSurface::Server)
        .map_err(|error| DoweError::new(error.to_string()))?;
    bindings.insert(
        statement.binding.clone(),
        dowe_type_from_stdlib_return(kind),
    );
    Ok(())
}

fn infer_http_statement(statement: &OutboundHttpRequest, bindings: &mut HashMap<String, DoweType>) {
    bindings.insert(
        statement.binding.clone(),
        DoweType::Object(vec![
            DoweTypeField {
                name: "status".to_string(),
                value: DoweType::Number,
                optional: false,
            },
            DoweTypeField {
                name: "ok".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
            DoweTypeField {
                name: "url".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "redirected".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
            DoweTypeField {
                name: "contentType".to_string(),
                value: DoweType::String,
                optional: true,
            },
            DoweTypeField {
                name: "headers".to_string(),
                value: DoweType::Unknown,
                optional: false,
            },
            DoweTypeField {
                name: "location".to_string(),
                value: DoweType::String,
                optional: true,
            },
            DoweTypeField {
                name: "json".to_string(),
                value: DoweType::Unknown,
                optional: true,
            },
        ]),
    );
}

fn infer_spawn_statement(
    statement: &ServerSpawnStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_crypto_aes_ctr_statement(
    statement: &ServerCryptoAesCtrStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_crypto_cenc_aes_ctr_statement(
    statement: &ServerCryptoCencAesCtrStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn validate_http_statement_references(
    node: &SourceNode,
    statement: &OutboundHttpRequest,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    if let Some(json) = &statement.json {
        validate_store_literal_references(node, json, bindings)?;
    }
    Ok(())
}

fn validate_spawn_statement_references(
    node: &SourceNode,
    statement: &ServerSpawnStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_store_literal_references(node, &statement.command, bindings)?;
    validate_store_literal_references(node, &statement.args, bindings)?;
    if let Some(cwd) = &statement.cwd {
        validate_store_literal_references(node, cwd, bindings)?;
    }
    Ok(())
}

fn validate_crypto_aes_ctr_statement_references(
    node: &SourceNode,
    statement: &ServerCryptoAesCtrStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.data, bindings)?;
    validate_store_literal_references(node, &statement.key, bindings)?;
    validate_store_literal_references(node, &statement.iv, bindings)
}

fn validate_crypto_cenc_aes_ctr_statement_references(
    node: &SourceNode,
    statement: &ServerCryptoCencAesCtrStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.data, bindings)?;
    validate_store_literal_references(node, &statement.key, bindings)?;
    validate_store_literal_references(node, &statement.iv, bindings)?;
    if let Some(subsamples) = &statement.subsamples {
        validate_store_literal_references(node, subsamples, bindings)?;
    }
    Ok(())
}

fn validate_websocket_sse_bridge_references(
    node: &SourceNode,
    statement: &WebSocketSseBridgeStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.upstream, bindings)?;
    validate_reference_path(node, &statement.request_id, bindings)?;
    validate_reference_path(node, &statement.request_type, bindings)?;
    validate_reference_path(node, &statement.model, bindings)
}

fn validate_store_statement_references(
    node: &SourceNode,
    statement: &crate::model::ServerStoreStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        crate::model::ServerStoreStatement::Insert { value, .. } => {
            validate_store_literal_references(node, value, bindings)
        }
        crate::model::ServerStoreStatement::Update {
            filter,
            value,
            matches,
            ..
        } => {
            validate_store_literal_references(node, &filter.value, bindings)?;
            for field in &filter.additional {
                validate_store_literal_references(node, &field.value, bindings)?;
            }
            validate_store_literal_references(node, value, bindings)?;
            for expected in matches {
                validate_store_literal_references(node, &expected.value, bindings)?;
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Read { filter, .. }
        | crate::model::ServerStoreStatement::Delete { filter, .. } => {
            validate_store_literal_references(node, &filter.value, bindings)?;
            for field in &filter.additional {
                validate_store_literal_references(node, &field.value, bindings)?;
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Transaction { operations, .. } => {
            for operation in operations {
                match operation {
                    crate::model::StoreTransactionOperation::Insert { value, .. } => {
                        validate_store_literal_references(node, value, bindings)?;
                    }
                }
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Handle { .. }
        | crate::model::ServerStoreStatement::List { .. }
        | crate::model::ServerStoreStatement::Query { .. } => Ok(()),
    }
}

fn validate_store_literal_references(
    node: &SourceNode,
    value: &StoreLiteral,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => validate_reference_path(node, reference, bindings),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_store_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_store_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn validate_stdlib_statement_references(
    node: &SourceNode,
    statement: &ServerStdlibStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    for reference in dowe_stdlib::reference_paths(&statement.call) {
        validate_reference_path(node, &reference, bindings)?;
    }
    Ok(())
}

fn validate_log_references(
    node: &SourceNode,
    log: &ServerLog,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    for value in &log.values {
        let ServerLogValue::Reference(reference) = value else {
            continue;
        };
        validate_reference_path(node, reference, bindings)?;
    }
    Ok(())
}

fn validate_return_references(
    node: &SourceNode,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    if let Some(json) = node.prop("json") {
        validate_source_value_references(node, &json.value, bindings)?;
    }
    if let Some(proxy) = node.prop("proxy") {
        let reference = proxy
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(proxy, "`proxy` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(agent) = node.prop("agent") {
        let reference = agent
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(agent, "`agent` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(request) = node.prop("request") {
        let reference = request
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(request, "`request` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(bytes) = node.prop("bytes") {
        let reference = bytes
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(bytes, "`bytes` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(headers) = node.prop("headers") {
        validate_source_value_references(node, &headers.value, bindings)?;
    }
    if let Some(cookies) = node.prop("cookies") {
        validate_source_value_references(node, &cookies.value, bindings)?;
    }
    Ok(())
}

fn validate_source_value_references(
    node: &SourceNode,
    value: &SourceValue,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        SourceValue::Bareword(reference) => validate_reference_path(node, reference, bindings),
        SourceValue::Array(values) => {
            for value in values {
                validate_source_value_references(node, value, bindings)?;
            }
            Ok(())
        }
        SourceValue::Object(entries) => {
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { value, .. } => {
                        validate_source_value_references(node, value, bindings)?;
                    }
                    SourceObjectEntry::Spread(reference) => {
                        validate_reference_path(node, reference, bindings)?;
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[derive(Clone, Copy)]
enum ActionContext<'a> {
    Init,
    HttpHandler {
        async_handler: bool,
        request: Option<&'a str>,
    },
    Middleware,
    Function,
    WebSocket,
    Protocol {
        binding: &'a str,
    },
}

fn parse_request_json_const(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        ));
    }
    reject_unknown_props(node, &["value"])?;
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        )
    })?;
    let value = node
        .prop("value")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| {
            node_error(
                node,
                "request JSON uses `const <binding[:Type]> value:req.json`",
            )
        })?;
    if value != "req.json" {
        return Err(node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        ));
    }
    validate_request_usage(node, context, "req.json")?;
    let (binding, schema) = parse_binding_type(node, &binding, types)?;
    Ok(ServerStatement::RequestJson { binding, schema })
}

fn legacy_request_json_let(node: &SourceNode) -> bool {
    node.args.len() == 4
        && node.args[1].as_string_like().as_deref() == Some("=")
        && node.args[2].as_string_like().as_deref() == Some("await")
        && node.args[3].as_string_like().as_deref() == Some("req.json()")
}

fn legacy_spawn_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| expression == "dowe.spawn")
}

fn legacy_http_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(
            expression.as_str(),
            "http.request" | "http.get" | "http.post"
        )
    })
}

fn legacy_crypto_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(expression.as_str(), "crypto.aesCtr" | "crypto.cencAesCtr")
    })
}

fn parse_request_metadata_let(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<Option<ServerStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    let statement = match expression.as_str() {
        "req.query" => {
            reject_unknown_props(node, &[])?;
            ServerStatement::RequestQuery { binding }
        }
        "req.rawQuery" => {
            reject_unknown_props(node, &[])?;
            ServerStatement::RequestRawQuery { binding }
        }
        "req.header" => {
            reject_unknown_props(node, &["name"])?;
            ServerStatement::RequestHeader {
                binding,
                name: required_header_name_prop(node, "name")?,
            }
        }
        "req.cookie" => {
            reject_unknown_props(node, &["name"])?;
            ServerStatement::RequestCookie {
                binding,
                name: required_cookie_name_prop(node, "name")?,
            }
        }
        _ => return Ok(None),
    };
    validate_request_usage(node, context, expression.as_str())?;
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` only accept named props",
        ));
    }
    match context {
        ActionContext::HttpHandler {
            request: Some("req"),
            ..
        } => {}
        _ => {
            return Err(node_error(
                node,
                "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` are only valid in HTTP handlers",
            ));
        }
    }
    Ok(Some(statement))
}

fn parse_request_declaration(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "request uses `request <binding> source:\"query|rawQuery|header|cookie\" [name:<name>]`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "request requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, "request")?;
    let statement = match source.as_str() {
        "query" => {
            reject_unknown_props(node, &["source"])?;
            ServerStatement::RequestQuery { binding }
        }
        "rawQuery" => {
            reject_unknown_props(node, &["source"])?;
            ServerStatement::RequestRawQuery { binding }
        }
        "header" => {
            reject_unknown_props(node, &["source", "name"])?;
            ServerStatement::RequestHeader {
                binding,
                name: required_header_name_prop(node, "name")?,
            }
        }
        "cookie" => {
            reject_unknown_props(node, &["source", "name"])?;
            ServerStatement::RequestCookie {
                binding,
                name: required_cookie_name_prop(node, "name")?,
            }
        }
        _ => {
            return Err(node_error(
                node,
                "request `source` must be `query`, `rawQuery`, `header`, or `cookie`",
            ));
        }
    };
    if !matches!(
        context,
        ActionContext::HttpHandler {
            request: Some("req"),
            ..
        }
    ) {
        return Err(node_error(
            node,
            "`request` declarations are only valid in HTTP handlers",
        ));
    }
    Ok(statement)
}

fn legacy_request_metadata_error(node: &SourceNode, statement: &ServerStatement) -> DoweError {
    let replacement = match statement {
        ServerStatement::RequestQuery { binding } => {
            format!("request {binding} source:\"query\"")
        }
        ServerStatement::RequestRawQuery { binding } => {
            format!("request {binding} source:\"rawQuery\"")
        }
        ServerStatement::RequestHeader { binding, name } => {
            format!("request {binding} source:\"header\" name:\"{name}\"")
        }
        ServerStatement::RequestCookie { binding, name } => {
            format!("request {binding} source:\"cookie\" name:\"{name}\"")
        }
        _ => unreachable!(),
    };
    node_error(
        node,
        format!("request metadata uses `{replacement}`; `let` is not supported"),
    )
}

fn parse_websocket_json_let(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<Option<WebSocketJsonStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    if expression != "ws.json" {
        return Ok(None);
    }
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            "`ws.json` does not accept positional values",
        ));
    }
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`ws.json` is only valid in WebSocket handlers",
        ));
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &[])?;
    Ok(Some(WebSocketJsonStatement { binding }))
}

fn parse_websocket_json_declaration(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketJsonStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "WebSocket JSON uses `ws <binding> source:\"json\"`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "WebSocket JSON requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["source"])?;
    if required_source_selector(node, "ws")? != "json" {
        return Err(node_error(node, "ws only supports `source:\"json\"`"));
    }
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`ws ... source:\"json\"` is only valid in WebSocket handlers",
        ));
    }
    Ok(WebSocketJsonStatement { binding })
}

fn parse_agent_chat_let(node: &SourceNode) -> DoweResult<Option<AgentChatTransform>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    if expression != "agent.chat" {
        return Ok(None);
    }
    if node.args.len() != 4 {
        return Err(node_error(
            node,
            "`agent.chat` requires a source request binding",
        ));
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &[])?;
    let source = node.args[3]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`agent.chat` source must be a reference"))?;
    Ok(Some(AgentChatTransform { binding, source }))
}

fn parse_agent_chat_declaration(node: &SourceNode) -> DoweResult<AgentChatTransform> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "Agent chat uses `agent <binding> source:\"chat\" request:<request>`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "Agent chat requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["source", "request"])?;
    if required_source_selector(node, "agent")? != "chat" {
        return Err(node_error(node, "agent only supports `source:\"chat\"`"));
    }
    Ok(AgentChatTransform {
        binding,
        source: required_reference_prop(node, "request")?,
    })
}

fn parse_jwt_statement(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerJwtStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "JWT requires one result binding: `jwt <binding> ...`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "JWT requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "secret",
            "key",
            "algorithm",
            "encryption",
            "token",
            "claims",
        ],
    )?;
    let token = node
        .prop("token")
        .map(|prop| {
            prop.value
                .as_string_like()
                .ok_or_else(|| prop_error(prop, "`token` must be a binding reference"))
        })
        .transpose()?;
    let claims = node
        .prop("claims")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    if token.is_some() == claims.is_some() {
        return Err(node_error(
            node,
            "JWT requires exactly one of `token` or `claims`",
        ));
    }
    if node.prop("secret").is_some() {
        if node.prop("key").is_some() || node.prop("encryption").is_some() {
            return Err(node_error(
                node,
                "JWS JWT uses `secret` and `algorithm:\"HS256\"`",
            ));
        }
        let secret = required_secret_prop(node, "secret", environment)?;
        let algorithm = required_algorithm_prop(node, "algorithm", &["HS256"])?;
        return match (token, claims) {
            (Some(token), None) => Ok(ServerJwtStatement::Verify {
                binding,
                token,
                secret,
                algorithm,
            }),
            (None, Some(claims)) => Ok(ServerJwtStatement::Sign {
                binding,
                claims,
                secret,
                algorithm,
            }),
            _ => unreachable!(),
        };
    }
    if node.prop("key").is_some() {
        let key = required_secret_prop(node, "key", environment)?;
        let algorithm = required_algorithm_prop(node, "algorithm", &["dir"])?;
        let encryption = required_algorithm_prop(node, "encryption", &["A256GCM"])?;
        return match (token, claims) {
            (Some(token), None) => Ok(ServerJwtStatement::Decrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            }),
            (None, Some(claims)) => Ok(ServerJwtStatement::Encrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            }),
            _ => unreachable!(),
        };
    }
    Err(node_error(
        node,
        "JWT requires server-only `secret` for JWS or `key` for JWE",
    ))
}

fn legacy_jwt_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(
            expression.as_str(),
            "jwt.verify" | "jwt.sign" | "jwt.decrypt" | "jwt.encrypt"
        )
    })
}

fn parse_stdlib_let(node: &SourceNode) -> DoweResult<Option<ServerStdlibStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    let Some(call) = parse_stdlib_call(node, &expression, StdlibSurface::Server, &[])? else {
        return Ok(None);
    };
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            format!("`{expression}` only accepts named arguments"),
        ));
    }
    validate_binding_name(node, &binding)?;
    Ok(Some(ServerStdlibStatement { binding, call }))
}

fn parse_stdlib_declaration(node: &SourceNode) -> DoweResult<ServerStdlibStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            format!(
                "{} uses `{} <binding> source:\"<function>\" <props>`",
                node.name, node.name
            ),
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "stdlib requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, &node.name)?;
    let expression = format!("{}.{}", node.name, source);
    let call = parse_stdlib_call(node, &expression, StdlibSurface::Server, &["source"])?
        .ok_or_else(|| {
            node_error(
                node,
                format!("unsupported stdlib namespace `{}`", node.name),
            )
        })?;
    Ok(ServerStdlibStatement { binding, call })
}

fn legacy_stdlib_error(node: &SourceNode, statement: &ServerStdlibStatement) -> DoweError {
    node_error(
        node,
        format!(
            "stdlib uses `{} {} source:\"{}\" <props>`; `let` is not supported",
            statement.call.namespace, statement.binding, statement.call.function
        ),
    )
}

fn required_source_selector(node: &SourceNode, capability: &str) -> DoweResult<String> {
    let prop = node
        .prop("source")
        .ok_or_else(|| node_error(node, format!("`{capability}` requires `source:\"...\"`")))?;
    required_static_string_prop(prop)
}

fn parse_server_function_call(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<Option<ServerCallStatement>> {
    let Some(callable) = callables.get(&node.name) else {
        return Ok(None);
    };
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            "server function calls do not accept child blocks",
        ));
    }
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            format!(
                "server function call requires one result binding: `{} <binding> [args:{{ ... }}]`",
                callable.name
            ),
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "server function call requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["args"])?;
    let source = node
        .args
        .iter()
        .map(SourceValue::to_source)
        .chain(
            node.props
                .iter()
                .map(|prop| format!("{}:{}", prop.name, prop.value.to_source())),
        )
        .collect::<Vec<_>>()
        .join(" ");
    validate_request_usage(node, context, &source)?;
    let args = if let Some(prop) = node.prop("args") {
        match &prop.value {
            SourceValue::Object(_) => store_literal(&prop.value)?,
            _ => return Err(prop_error(prop, "`args` must be an object")),
        }
    } else {
        StoreLiteral::Object(Vec::new())
    };
    validate_server_function_args(node, &args, &callable.action.params, bindings)?;
    Ok(Some(ServerCallStatement {
        binding,
        target: callable.name.clone(),
        args,
        action: Box::new(callable.action.clone()),
    }))
}

fn push_server_function_call(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &mut HashMap<String, DoweType>,
    statements: &mut Vec<ServerStatement>,
) -> DoweResult<bool> {
    let Some(statement) = parse_server_function_call(node, context, callables, bindings)? else {
        return Ok(false);
    };
    validate_store_literal_references(node, &statement.args, bindings)?;
    bindings.insert(
        statement.binding.clone(),
        statement
            .action
            .return_type
            .as_ref()
            .map(|return_type| return_type.schema.clone())
            .unwrap_or(DoweType::Unknown),
    );
    statements.push(ServerStatement::Call(statement));
    Ok(true)
}

fn reject_legacy_server_function_call(
    node: &SourceNode,
    callables: &HashMap<String, ServerCallable>,
) -> DoweResult<()> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(());
    };
    let Some(callable) = callables.get(&expression) else {
        return Ok(());
    };
    let args = if callable.action.params.is_empty() {
        ""
    } else {
        " args:{ ... }"
    };
    Err(node_error(
        node,
        format!(
            "server function calls use `{} {}{args}`; `let {} = {}` is not supported",
            callable.name, binding, binding, callable.name
        ),
    ))
}

fn parse_background_job(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    cron: bool,
) -> DoweResult<ServerBackgroundJob> {
    if cron && !matches!(context, ActionContext::Init) {
        return Err(node_error(node, "`cron` is only valid inside server init"));
    }
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            "background jobs do not accept child blocks",
        ));
    }
    let target = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "background job must declare a target"))?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "background jobs accept one target and named props",
        ));
    }
    let callable = callables
        .get(&target)
        .ok_or_else(|| node_error(node, format!("missing server function import `{target}`")))?;
    reject_unknown_props(
        node,
        if cron {
            &["args", "schedule"]
        } else {
            &["args"]
        },
    )?;
    let args = if let Some(prop) = node.prop("args") {
        let SourceValue::Object(_) = &prop.value else {
            return Err(prop_error(prop, "`args` must be an object"));
        };
        let value = store_literal(&prop.value)?;
        reject_background_references(node, &value)?;
        value
    } else {
        StoreLiteral::Object(Vec::new())
    };
    validate_server_function_args(node, &args, &callable.action.params, &HashMap::new())?;
    let schedule = if cron {
        let prop = node
            .prop("schedule")
            .ok_or_else(|| node_error(node, "cron must declare `schedule`"))?;
        let SourceValue::String(value) = &prop.value else {
            return Err(prop_error(prop, "`schedule` must be a quoted string"));
        };
        CronSchedule::parse(value).map_err(|error| prop_error(prop, error.to_string()))?;
        Some(value.clone())
    } else {
        None
    };
    Ok(ServerBackgroundJob {
        id: format!(
            "{}:{}:{}:{}",
            node.location.relative_path.display(),
            node.location.line,
            node.name,
            target
        ),
        target: callable.name.clone(),
        args,
        action: Box::new(callable.action.clone()),
        schedule,
    })
}

fn reject_background_references(node: &SourceNode, value: &StoreLiteral) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => Err(node_error(
            node,
            format!("background args must be static JSON; found reference `{reference}`"),
        )),
        StoreLiteral::Array(values) => {
            for value in values {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn parse_server_function_params(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Vec<ServerFunctionParameter>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(prop, "fn params must be an object"));
    };
    if entries.is_empty() {
        return Err(prop_error(prop, "fn params must be a non-empty object"));
    }
    let mut params = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "fn params does not support object spread"));
        };
        if !names.insert(key.clone()) {
            return Err(prop_error(prop, format!("duplicate fn parameter `{key}`")));
        }
        validate_binding_name(node, key)?;
        let type_name = value
            .as_required_string()
            .ok_or_else(|| prop_error(prop, "fn params values must be type names"))?;
        let schema = types.resolve(node, &type_name)?;
        params.push(ServerFunctionParameter {
            name: key.clone(),
            type_name,
            schema,
        });
    }
    Ok(params)
}

fn parse_server_function_return(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Option<ServerFunctionReturn>> {
    let Some(prop) = node.prop("return") else {
        return Ok(None);
    };
    let type_name = prop
        .value
        .as_required_string()
        .ok_or_else(|| prop_error(prop, "fn return must be a quoted type name"))?;
    let schema = types.resolve(node, &type_name)?;
    Ok(Some(ServerFunctionReturn { type_name, schema }))
}

fn validate_server_function_args(
    node: &SourceNode,
    args: &StoreLiteral,
    params: &[ServerFunctionParameter],
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    let StoreLiteral::Object(entries) = args else {
        return Err(node_error(node, "function args must be an object"));
    };
    if params.is_empty() {
        if entries.is_empty() {
            return Ok(());
        }
        return Err(node_error(node, "function does not declare params"));
    }
    for parameter in params {
        let value = entries
            .iter()
            .find(|(name, _)| name == &parameter.name)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                node_error(
                    node,
                    format!(
                        "function call is missing required argument `{}`",
                        parameter.name
                    ),
                )
            })?;
        let actual = server_literal_type(value, bindings);
        if !server_type_assignable(&actual, &parameter.schema) {
            return Err(node_error(
                node,
                format!(
                    "argument `{}` is incompatible with function parameter type `{}`",
                    parameter.name, parameter.type_name
                ),
            ));
        }
    }
    for (name, _) in entries {
        if !params.iter().any(|parameter| parameter.name == *name) {
            return Err(node_error(
                node,
                format!("function call does not declare argument `{name}`"),
            ));
        }
    }
    Ok(())
}

fn server_literal_type(value: &StoreLiteral, bindings: &HashMap<String, DoweType>) -> DoweType {
    match value {
        StoreLiteral::Reference(reference) => {
            server_reference_type(reference, bindings).unwrap_or(DoweType::Unknown)
        }
        StoreLiteral::Array(values) => DoweType::Array(Box::new(
            values
                .first()
                .map(|value| server_literal_type(value, bindings))
                .unwrap_or(DoweType::Unknown),
        )),
        StoreLiteral::Object(entries) => DoweType::Object(
            entries
                .iter()
                .map(|(name, value)| DoweTypeField {
                    name: name.clone(),
                    value: server_literal_type(value, bindings),
                    optional: false,
                })
                .collect(),
        ),
        _ => type_from_store_literal(value),
    }
}

fn server_reference_type(
    reference: &str,
    bindings: &HashMap<String, DoweType>,
) -> Option<DoweType> {
    let (binding, path) = reference
        .split_once('.')
        .map_or((reference, ""), |(binding, path)| (binding, path));
    let mut value = bindings.get(binding)?.clone();
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        value = match value {
            DoweType::Unknown => return Some(DoweType::Unknown),
            DoweType::Object(fields) => fields
                .into_iter()
                .find(|field| field.name == segment)
                .map(|field| field.value)?,
            _ => return None,
        };
    }
    Some(value)
}

fn server_type_assignable(actual: &DoweType, expected: &DoweType) -> bool {
    match (actual, expected) {
        (_, DoweType::Unknown) | (DoweType::Unknown, _) => true,
        (DoweType::Null, DoweType::Null)
        | (DoweType::Bool, DoweType::Bool)
        | (DoweType::Number, DoweType::Number)
        | (DoweType::String, DoweType::String) => true,
        (DoweType::Array(actual), DoweType::Array(expected)) => {
            server_type_assignable(actual, expected)
        }
        (DoweType::Object(actual), DoweType::Object(expected)) => expected.iter().all(|field| {
            actual
                .iter()
                .find(|candidate| candidate.name == field.name)
                .is_some_and(|candidate| server_type_assignable(&candidate.value, &field.value))
                || field.optional
        }),
        _ => false,
    }
}

fn parse_server_function_return_value(node: &SourceNode) -> DoweResult<StoreLiteral> {
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("response")
    {
        return Err(node_error(
            node,
            "server fn return must use `return value:<value>`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(node, "return value must use `value:<value>`"));
    }
    if node.props.iter().any(|prop| prop.name != "value") {
        return Err(node_error(
            node,
            "server fn return must use `return value:<value>`",
        ));
    }
    reject_unknown_props(node, &["value"])?;
    required_store_literal_prop(node, "value")
}

fn parse_http_declaration(
    node: &SourceNode,
    context: ActionContext,
    environment: &EnvironmentConfig,
) -> DoweResult<OutboundHttpRequest> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
        )
    })?;
    let method = required_http_method_prop(node)?;
    match context {
        ActionContext::HttpHandler {
            async_handler: true,
            ..
        }
        | ActionContext::WebSocket
        | ActionContext::Function => {}
        ActionContext::HttpHandler { .. } => {
            return Err(node_error(node, "http requires an async request handler"));
        }
        ActionContext::Init => {
            return Err(node_error(node, "http is not valid in server init"));
        }
        ActionContext::Middleware => {
            return Err(node_error(
                node,
                "http is only valid in async handlers, server functions, and WebSocket handlers",
            ));
        }
        ActionContext::Protocol { .. } => {
            return Err(node_error(node, "http is not valid in protocol handlers"));
        }
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "method",
            "base",
            "path",
            "bearer",
            "headers",
            "json",
            "mode",
            "redirect",
            "maxRedirects",
            "timeoutMs",
        ],
    )?;
    let base = required_http_base_prop(node, environment)?;
    let path = required_http_path_prop(node)?;
    let bearer = if node.prop("bearer").is_some() {
        Some(required_secret_prop(node, "bearer", environment)?)
    } else {
        None
    };
    let headers = optional_http_headers_prop(node, environment)?;
    let json = node
        .prop("json")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    let mode = optional_http_mode_prop(node)?;
    let redirect = optional_http_redirect_prop(node)?;
    let max_redirects = optional_positive_u32_prop(node, "maxRedirects")?;
    if max_redirects.is_some() && redirect != HttpRedirectPolicy::Follow {
        return Err(node_error(
            node,
            "`maxRedirects` is only valid with redirect:\"follow\"",
        ));
    }
    let timeout_ms = optional_positive_u64_prop(node, "timeoutMs")?;
    Ok(OutboundHttpRequest {
        binding,
        method,
        base,
        path,
        bearer,
        headers,
        json,
        mode,
        redirect,
        max_redirects,
        timeout_ms,
    })
}

fn parse_spawn_declaration(node: &SourceNode) -> DoweResult<ServerSpawnStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "spawn uses `spawn <binding> command:<value> [args:<array>]`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "spawn uses `spawn <binding> command:<value> [args:<array>]`",
        )
    })?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "command",
            "args",
            "cwd",
            "timeoutMs",
            "maxOutputBytes",
            "background",
        ],
    )?;
    let command = required_store_literal_prop(node, "command")?;
    let args = node
        .prop("args")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or_else(|| StoreLiteral::Array(Vec::new()));
    let cwd = node
        .prop("cwd")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    let timeout_ms = optional_positive_u64_prop(node, "timeoutMs")?;
    let max_output_bytes = optional_positive_usize_prop(node, "maxOutputBytes")?;
    let background = optional_bool_prop(node, "background")?.unwrap_or(false);
    Ok(ServerSpawnStatement {
        binding,
        command,
        args,
        cwd,
        timeout_ms,
        max_output_bytes,
        background,
    })
}

fn parse_crypto_declaration(node: &SourceNode) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
        )
    })?;
    validate_binding_name(node, &binding)?;
    let encryption_prop = node
        .prop("encryption")
        .ok_or_else(|| node_error(node, "missing `encryption`"))?;
    let encryption = required_static_string_prop(encryption_prop)?;
    match encryption.as_str() {
        "aesCtr" => {
            reject_unknown_props(node, &["encryption", "data", "key", "iv"])?;
            Ok(ServerStatement::CryptoAesCtr(ServerCryptoAesCtrStatement {
                binding,
                data: required_reference_prop(node, "data")?,
                key: required_store_literal_prop(node, "key")?,
                iv: required_store_literal_prop(node, "iv")?,
            }))
        }
        "cencAesCtr" => {
            reject_unknown_props(node, &["encryption", "data", "key", "iv", "subsamples"])?;
            let subsamples = node
                .prop("subsamples")
                .map(|prop| store_literal(&prop.value))
                .transpose()?;
            Ok(ServerStatement::CryptoCencAesCtr(
                ServerCryptoCencAesCtrStatement {
                    binding,
                    data: required_reference_prop(node, "data")?,
                    key: required_store_literal_prop(node, "key")?,
                    iv: required_store_literal_prop(node, "iv")?,
                    subsamples,
                },
            ))
        }
        _ => Err(prop_error(
            encryption_prop,
            "`encryption` must be aesCtr or cencAesCtr",
        )),
    }
}

fn parse_websocket_send_json(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketSendJsonStatement> {
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`send ws` is only valid in WebSocket handlers",
        ));
    }
    if node.args.len() != 1 || node.args[0].as_string_like().as_deref() != Some("ws") {
        return Err(node_error(node, "`send` must target `ws`"));
    }
    reject_unknown_props(node, &["json"])?;
    let value = required_store_literal_prop(node, "json")?;
    Ok(WebSocketSendJsonStatement { value })
}

fn parse_websocket_sse_bridge(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketSseBridgeStatement> {
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`bridge sse` is only valid in WebSocket handlers",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`bridge` does not accept positional values",
        ));
    }
    reject_unknown_props(node, &["sse", "to", "requestId", "requestType", "model"])?;
    let upstream = required_reference_prop(node, "sse")?;
    let target = node
        .prop("to")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| node_error(node, "`bridge` must declare `to:ws`"))?;
    if target != "ws" {
        return Err(node_error(node, "`bridge` only supports `to:ws`"));
    }
    Ok(WebSocketSseBridgeStatement {
        upstream,
        request_id: required_reference_prop(node, "requestId")?,
        request_type: required_reference_prop(node, "requestType")?,
        model: required_reference_prop(node, "model")?,
    })
}

fn parse_binding_type(
    node: &SourceNode,
    value: &str,
    types: &TypeRegistry,
) -> DoweResult<(String, Option<DoweType>)> {
    let Some((binding, type_name)) = value.split_once(':') else {
        validate_binding_name(node, value)?;
        return Ok((value.to_string(), None));
    };
    if binding.is_empty() || type_name.is_empty() {
        return Err(node_error(node, "typed binding must use `name:Type`"));
    }
    validate_binding_name(node, binding)?;
    let schema = types.resolve(node, type_name)?;
    Ok((binding.to_string(), Some(schema)))
}

fn validate_binding_name(node: &SourceNode, value: &str) -> DoweResult<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(node_error(node, "binding name must not be empty"));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
    {
        return Err(node_error(
            node,
            format!("binding `{value}` must be an ASCII identifier"),
        ));
    }
    Ok(())
}

fn validate_return(node: &SourceNode, context: ActionContext) -> DoweResult<()> {
    let source = node
        .args
        .iter()
        .map(SourceValue::to_source)
        .chain(
            node.props
                .iter()
                .map(|prop| format!("{}:{}", prop.name, prop.value.to_source())),
        )
        .collect::<Vec<_>>()
        .join(" ");
    validate_request_usage(node, context, &source)?;
    if source.contains("await") && !context_allows_await(context) {
        return Err(node_error(
            node,
            "`await` is only valid inside async handlers",
        ));
    }
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("response")
    {
        return Err(node_error(
            node,
            "HTTP returns use `return <props>`; remove `response`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "HTTP returns do not accept positional values",
        ));
    }
    reject_unknown_props(
        node,
        &[
            "status",
            "text",
            "json",
            "proxy",
            "agent",
            "bytes",
            "contentType",
            "headers",
            "cookies",
            "request",
        ],
    )?;
    let body_count = ["text", "json", "proxy", "agent", "bytes"]
        .iter()
        .filter(|name| node.prop(name).is_some())
        .count();
    if body_count == 0 {
        return Err(node_error(
            node,
            "return must declare text, json, proxy, agent, or bytes",
        ));
    }
    if body_count > 1 {
        return Err(node_error(
            node,
            "return must declare exactly one of text, json, proxy, agent, or bytes",
        ));
    }
    if let Some(prop) = node.prop("text") {
        required_static_string_prop(prop)?;
    }
    if node.prop("agent").is_some() && node.prop("request").is_none() {
        return Err(node_error(
            node,
            "agent response must declare `request` binding",
        ));
    }
    Ok(())
}

fn validate_request_usage(
    node: &SourceNode,
    context: ActionContext,
    source: &str,
) -> DoweResult<()> {
    if source.contains("req.params")
        && !matches!(
            context,
            ActionContext::HttpHandler {
                request: Some("req"),
                ..
            }
        )
    {
        return Err(node_error(
            node,
            "`req.params` is only valid in HTTP handlers",
        ));
    }
    let uses_request_metadata = source.contains("req.query")
        || source.contains("req.rawQuery")
        || source.contains("req.header")
        || source.contains("req.cookie");
    if uses_request_metadata
        && !matches!(
            context,
            ActionContext::HttpHandler {
                request: Some("req"),
                ..
            }
        )
    {
        return Err(node_error(
            node,
            "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` are only valid in HTTP handlers",
        ));
    }
    if source.contains("req.json") {
        match context {
            ActionContext::HttpHandler {
                async_handler: true,
                request: Some("req"),
            } => {}
            ActionContext::HttpHandler { .. } => {
                return Err(node_error(
                    node,
                    "`req.json` requires an async request handler",
                ));
            }
            ActionContext::Init
            | ActionContext::Middleware
            | ActionContext::Function
            | ActionContext::WebSocket
            | ActionContext::Protocol { .. } => {
                return Err(node_error(
                    node,
                    "`req.json` is only valid in HTTP handlers",
                ));
            }
        }
    }
    Ok(())
}

fn context_allows_await(context: ActionContext) -> bool {
    matches!(
        context,
        ActionContext::HttpHandler {
            async_handler: true,
            ..
        }
    )
}

fn parse_log(node: &SourceNode) -> DoweResult<ServerLog> {
    let level = match node.name.as_str() {
        "log" => ServerLogLevel::Log,
        "info" => ServerLogLevel::Info,
        "warn" => ServerLogLevel::Warn,
        "error" => ServerLogLevel::Error,
        _ => return Err(node_error(node, "unsupported log action")),
    };
    let values = node
        .args
        .iter()
        .map(log_value)
        .collect::<DoweResult<Vec<_>>>()?;
    Ok(ServerLog { level, values })
}

fn log_value(value: &SourceValue) -> DoweResult<ServerLogValue> {
    match value {
        SourceValue::String(value) => Ok(ServerLogValue::String(value.clone())),
        SourceValue::Bareword(value) => Ok(ServerLogValue::Reference(value.clone())),
        SourceValue::Number(value) => Ok(ServerLogValue::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(ServerLogValue::Boolean(*value)),
        SourceValue::Null => Ok(ServerLogValue::Null),
        SourceValue::Array(_) | SourceValue::Object(_) => {
            Ok(ServerLogValue::JsonLiteral(value.to_source()))
        }
    }
}

fn required_port(node: &SourceNode) -> DoweResult<u16> {
    let prop = node
        .prop("port")
        .ok_or_else(|| node_error(node, "missing server port"))?;
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "invalid server port"))?;
    value
        .parse::<u16>()
        .map_err(|_| node_error(node, "invalid server port"))
}

fn required_name_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "missing `name`"))?;
    let value = required_static_string_prop(prop)?;
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(prop_error(prop, "`name` must be a stable ASCII name"));
    }
    Ok(value)
}

fn optional_bind_prop(node: &SourceNode) -> DoweResult<String> {
    let Some(prop) = node.prop("bind") else {
        return Ok("127.0.0.1".to_string());
    };
    let value = required_static_string_prop(prop)?;
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(prop_error(prop, "`bind` must be an IP address or host"));
    }
    Ok(value)
}

fn required_transport_port(node: &SourceNode) -> DoweResult<u16> {
    reject_unknown_props(node, &["name", "bind", "port"])?;
    required_port_prop(node, "port")
}

fn required_port_prop(node: &SourceNode, name: &str) -> DoweResult<u16> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a port")))?;
    value
        .parse::<u16>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a port")))
}

fn required_model_kind_prop(node: &SourceNode) -> DoweResult<ServerModelKind> {
    let prop = node
        .prop("kind")
        .ok_or_else(|| node_error(node, "missing `kind`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "vad.silero" => Ok(ServerModelKind::VadSilero),
        _ => Err(prop_error(prop, "unsupported model kind")),
    }
}

fn required_model_engine_prop(node: &SourceNode) -> DoweResult<ServerModelEngine> {
    let prop = node
        .prop("engine")
        .ok_or_else(|| node_error(node, "missing `engine`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "candle" => Ok(ServerModelEngine::Candle),
        "energy" => Ok(ServerModelEngine::Energy),
        _ => Err(prop_error(prop, "`engine` must be `candle` or `energy`")),
    }
}

fn required_model_format_prop(node: &SourceNode) -> DoweResult<ServerModelFormat> {
    let prop = node
        .prop("format")
        .ok_or_else(|| node_error(node, "missing `format`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "onnx" => Ok(ServerModelFormat::Onnx),
        "builtin" => Ok(ServerModelFormat::Builtin),
        _ => Err(prop_error(prop, "`format` must be `onnx` or `builtin`")),
    }
}

fn optional_model_source_prop(
    node: &SourceNode,
    format: ServerModelFormat,
) -> DoweResult<Option<std::path::PathBuf>> {
    let Some(prop) = node.prop("source") else {
        return match format {
            ServerModelFormat::Builtin => Ok(None),
            ServerModelFormat::Onnx => Err(node_error(node, "missing `source`")),
        };
    };
    let value = required_static_string_prop(prop)?;
    if !value.starts_with("assets/")
        || value.is_empty()
        || value.starts_with('/')
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
    {
        return Err(prop_error(prop, "`source` must be under `assets/`"));
    }
    Ok(Some(std::path::PathBuf::from(value)))
}

fn optional_sample_rates_prop(node: &SourceNode) -> DoweResult<Vec<u32>> {
    let Some(prop) = node.prop("sampleRates") else {
        return Ok(vec![8_000, 16_000]);
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`sampleRates` must be an array"));
    };
    let mut rates = Vec::new();
    for value in values {
        let Some(rate) = value.as_string_like() else {
            return Err(prop_error(prop, "`sampleRates` entries must be numbers"));
        };
        let rate = rate
            .parse::<u32>()
            .map_err(|_| prop_error(prop, "`sampleRates` entries must be numbers"))?;
        match rate {
            8_000 | 16_000 => rates.push(rate),
            _ => return Err(prop_error(prop, "Silero VAD supports 8000 and 16000 Hz")),
        }
    }
    if rates.is_empty() {
        return Err(prop_error(prop, "`sampleRates` cannot be empty"));
    }
    rates.sort_unstable();
    rates.dedup();
    Ok(rates)
}

fn required_path_arg(node: &SourceNode, label: &str) -> DoweResult<String> {
    let path = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, format!("{label} must declare a path string")))?;
    if !path.starts_with('/') {
        return Err(node_error(
            node,
            format!("{label} path must start with `/`"),
        ));
    }
    Ok(path)
}

fn required_text_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("text")
        .ok_or_else(|| node_error(node, "response must declare text"))?;
    required_static_string_prop(prop)
}

fn http_endpoint_behavior(node: &SourceNode) -> DoweResult<Option<EndpointBehavior>> {
    if let Some(binding) = return_reference_prop(node, "proxy")? {
        return Ok(Some(EndpointBehavior::HttpProxy(HttpProxyEndpoint {
            binding,
        })));
    }
    if let Some(binding) = return_reference_prop(node, "bytes")? {
        return Ok(Some(EndpointBehavior::HttpBytes(HttpBytesEndpoint {
            status: return_status(node)?,
            binding,
            content_type: return_content_type(node)?,
            headers: return_headers(node)?,
            cookies: return_cookies(node)?,
        })));
    }
    if let Some(upstream) = return_reference_prop(node, "agent")? {
        let request = return_reference_prop(node, "request")?
            .ok_or_else(|| node_error(node, "agent response must declare `request` binding"))?;
        return Ok(Some(EndpointBehavior::AgentResponse(
            AgentResponseEndpoint { upstream, request },
        )));
    }
    if let Some(value) = return_json_value(node) {
        if !returns_created_json(node) {
            return Ok(Some(EndpointBehavior::HttpActionJson(
                HttpActionJsonEndpoint {
                    status: return_status(node)?,
                    value: store_literal(value)?,
                },
            )));
        }
    }
    Ok(None)
}

fn handler_behavior(
    node: &SourceNode,
    path: &str,
    action: &ServerAction,
) -> DoweResult<EndpointBehavior> {
    if has_reference_log(action)
        && let Some(behavior) = database_action_endpoint_behavior(
            action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = database_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(behavior);
    }
    if return_text(node).is_some_and(|value| value.contains("req.context")) {
        Ok(EndpointBehavior::TextTemplate(return_text(node).unwrap()))
    } else if path.contains("/:")
        && return_text(node).is_some_and(|value| value.contains("req.params"))
    {
        Ok(EndpointBehavior::UserGreeting)
    } else if let Some(text) = return_text(node) {
        Ok(EndpointBehavior::StaticText(text))
    } else {
        Err(node_error(
            node,
            "handler must return supported text response",
        ))
    }
}

fn exported_handler_behavior(
    node: &SourceNode,
    action: &ServerAction,
) -> DoweResult<EndpointBehavior> {
    if has_reference_log(action)
        && let Some(behavior) = database_action_endpoint_behavior(
            action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = database_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(behavior);
    }
    if let Some(text) = return_text(node)
        && text.contains("req.context")
    {
        return Ok(EndpointBehavior::TextTemplate(text));
    }
    if let Some(text) = return_text(node) {
        return Ok(EndpointBehavior::StaticText(text));
    }
    if returns_created_json(node) {
        return Ok(EndpointBehavior::CreatePostJson);
    }
    Err(node_error(
        node,
        "external handler must return supported response behavior",
    ))
}

fn has_reference_log(action: &ServerAction) -> bool {
    action.statements.iter().any(|statement| {
        matches!(
            statement,
            ServerStatement::Log(ServerLog { values, .. })
                if values
                    .iter()
                    .any(|value| matches!(value, ServerLogValue::Reference(_)))
        )
    })
}

fn return_text(node: &SourceNode) -> Option<String> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("text"))
        .and_then(|prop| match &prop.value {
            SourceValue::String(value) => Some(value.clone()),
            _ => None,
        })
}

fn returns_created_json(node: &SourceNode) -> bool {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .is_some_and(|prop| match &prop.value {
            SourceValue::Object(entries) => entries.iter().any(|entry| {
                matches!(
                    entry,
                    SourceObjectEntry::KeyValue {
                        key,
                        value: SourceValue::Boolean(true)
                    } if key == "created"
                )
            }),
            _ => false,
        })
}

fn return_json_ref(node: &SourceNode) -> Option<String> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .and_then(|prop| match &prop.value {
            SourceValue::Bareword(value) => Some(value.clone()),
            _ => None,
        })
}

fn return_json_value(node: &SourceNode) -> Option<&SourceValue> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .map(|prop| &prop.value)
}

fn return_reference_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop(name))
        .map(|prop| {
            prop.value
                .as_string_like()
                .ok_or_else(|| prop_error(prop, format!("`{name}` must be a binding reference")))
        })
        .transpose()
}

fn return_content_type(node: &SourceNode) -> DoweResult<Option<String>> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("contentType"))
        .map(required_static_string_prop)
        .transpose()
}

fn return_headers(node: &SourceNode) -> DoweResult<Vec<ResponseHeader>> {
    let Some(prop) = node
        .children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("headers"))
    else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`headers` must be an array"));
    };
    values
        .iter()
        .map(|value| parse_response_header(prop, value))
        .collect()
}

fn parse_response_header(prop: &SourceProp, value: &SourceValue) -> DoweResult<ResponseHeader> {
    let SourceValue::Object(entries) = value else {
        return Err(prop_error(prop, "`headers` entries must be objects"));
    };
    let mut name = None;
    let mut header_value = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`headers` entries do not support spread"));
        };
        match key.as_str() {
            "name" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "header `name` must be a quoted string"));
                };
                name = Some(value.clone());
            }
            "value" => header_value = Some(store_literal(value)?),
            _ => return Err(prop_error(prop, format!("unknown header field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "header entry missing `name`"))?;
    let name =
        normalize_http_header_name(&name).ok_or_else(|| prop_error(prop, "invalid header name"))?;
    let value = header_value.ok_or_else(|| prop_error(prop, "header entry missing `value`"))?;
    Ok(ResponseHeader { name, value })
}

fn return_cookies(node: &SourceNode) -> DoweResult<Vec<ResponseCookie>> {
    let Some(prop) = node
        .children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("cookies"))
    else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`cookies` must be an array"));
    };
    values
        .iter()
        .map(|value| parse_response_cookie(prop, value))
        .collect()
}

fn parse_response_cookie(prop: &SourceProp, value: &SourceValue) -> DoweResult<ResponseCookie> {
    let SourceValue::Object(entries) = value else {
        return Err(prop_error(prop, "`cookies` entries must be objects"));
    };
    let mut name = None;
    let mut cookie_value = None;
    let mut path = None;
    let mut http_only = false;
    let mut secure = false;
    let mut same_site = None;
    let mut max_age = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`cookies` entries do not support spread"));
        };
        match key.as_str() {
            "name" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "cookie `name` must be a quoted string"));
                };
                name = Some(value.clone());
            }
            "value" => cookie_value = Some(store_literal(value)?),
            "path" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "cookie `path` must be a quoted string"));
                };
                path = Some(value.clone());
            }
            "httpOnly" => http_only = required_bool_value(prop, value, "httpOnly")?,
            "secure" => secure = required_bool_value(prop, value, "secure")?,
            "sameSite" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(
                        prop,
                        "cookie `sameSite` must be a quoted string",
                    ));
                };
                if !matches!(value.as_str(), "Lax" | "Strict" | "None") {
                    return Err(prop_error(
                        prop,
                        "cookie `sameSite` must be `Lax`, `Strict`, or `None`",
                    ));
                }
                same_site = Some(value.clone());
            }
            "maxAge" => max_age = Some(required_u64_value(prop, value, "maxAge")?),
            _ => return Err(prop_error(prop, format!("unknown cookie field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "cookie entry missing `name`"))?;
    let value = cookie_value.ok_or_else(|| prop_error(prop, "cookie entry missing `value`"))?;
    Ok(ResponseCookie {
        name,
        value,
        path,
        http_only,
        secure,
        same_site,
        max_age,
    })
}

fn return_status(node: &SourceNode) -> DoweResult<u16> {
    let Some(return_node) = node.children.iter().find(|child| child.name == "return") else {
        return Ok(200);
    };
    return_status_from_node(return_node)
}

fn return_status_from_node(node: &SourceNode) -> DoweResult<u16> {
    let Some(prop) = node.prop("status") else {
        return Ok(200);
    };
    let Some(value) = prop.value.as_string_like() else {
        return Err(node_error(node, "`status` must be a number"));
    };
    value
        .parse::<u16>()
        .map_err(|_| node_error(node, "`status` must be a valid HTTP status"))
}

fn optional_prop_string(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| {
            prop.value
                .as_required_string()
                .ok_or_else(|| node_error(node, format!("`{name}` must be a string")))
        })
        .transpose()
}

fn required_http_base_prop(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<HttpConnectionValue> {
    let prop = node
        .prop("base")
        .ok_or_else(|| node_error(node, "missing `base`"))?;
    match &prop.value {
        SourceValue::String(value) => {
            if !value.starts_with("https://") && !value.starts_with("http://") {
                return Err(prop_error(prop, "`base` must be an http or https URL"));
            }
            Ok(HttpConnectionValue::Static(value.clone()))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(prop_error(
                    prop,
                    "`base` must be a quoted URL or server env reference",
                ));
            };
            let variable = environment.variable(env_name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{env_name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(
                    prop,
                    format!("environment variable `{env_name}` must be server-only"),
                ));
            }
            Ok(HttpConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(prop_error(
            prop,
            "`base` must be a quoted URL or server env reference",
        )),
    }
}

fn required_http_path_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("path")
        .ok_or_else(|| node_error(node, "missing `path`"))?;
    let value = required_static_string_prop(prop)?;
    if !value.starts_with('/') {
        return Err(prop_error(prop, "`path` must start with `/`"));
    }
    Ok(value)
}

fn optional_http_mode_prop(node: &SourceNode) -> DoweResult<HttpResponseMode> {
    let Some(prop) = node.prop("mode") else {
        return Ok(HttpResponseMode::Json);
    };
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "json" => Ok(HttpResponseMode::Json),
        "proxy" => Ok(HttpResponseMode::Proxy),
        "bytes" => Ok(HttpResponseMode::Bytes),
        _ => Err(prop_error(
            prop,
            "`mode` must be `json`, `proxy`, or `bytes`",
        )),
    }
}

fn optional_http_redirect_prop(node: &SourceNode) -> DoweResult<HttpRedirectPolicy> {
    let Some(prop) = node.prop("redirect") else {
        return Ok(HttpRedirectPolicy::Follow);
    };
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "follow" => Ok(HttpRedirectPolicy::Follow),
        "manual" => Ok(HttpRedirectPolicy::Manual),
        "error" => Ok(HttpRedirectPolicy::Error),
        _ => Err(prop_error(
            prop,
            "`redirect` must be `follow`, `manual`, or `error`",
        )),
    }
}

fn optional_positive_u32_prop(node: &SourceNode, name: &str) -> DoweResult<Option<u32>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<u32>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_positive_u64_prop(node: &SourceNode, name: &str) -> DoweResult<Option<u64>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<u64>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_positive_usize_prop(node: &SourceNode, name: &str) -> DoweResult<Option<usize>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<usize>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::Boolean(value) => Ok(Some(*value)),
        _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
    }
}

fn required_bool_value(prop: &SourceProp, value: &SourceValue, name: &str) -> DoweResult<bool> {
    match value {
        SourceValue::Boolean(value) => Ok(*value),
        _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
    }
}

fn required_u64_value(prop: &SourceProp, value: &SourceValue, name: &str) -> DoweResult<u64> {
    let Some(value) = value.as_string_like() else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    };
    value
        .parse::<u64>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))
}

fn required_reference_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    prop.value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a binding reference")))
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|name| *name == prop.name) {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `{}`", prop.name, node.name),
            ));
        }
    }
    Ok(())
}

fn assignment(node: &SourceNode) -> Option<(String, String)> {
    if node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let equals = node.args[1].as_string_like()?;
    let expression = node.args[2].as_string_like()?;
    (equals == "=").then_some((binding, expression))
}

fn required_header_name_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = match &prop.value {
        SourceValue::String(value) => value.clone(),
        _ => {
            return Err(prop_error(
                prop,
                format!("`{name}` must be a quoted static string literal"),
            ));
        }
    };
    normalize_http_header_name(&value).ok_or_else(|| prop_error(prop, "invalid header name"))
}

fn required_cookie_name_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let SourceValue::String(value) = &prop.value else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a quoted static string literal"),
        ));
    };
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(prop_error(prop, "invalid cookie name"));
    }
    Ok(value.clone())
}

fn required_http_method_prop(node: &SourceNode) -> DoweResult<HttpMethod> {
    let prop = node
        .prop("method")
        .ok_or_else(|| node_error(node, "missing `method`"))?;
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "patch" => Ok(HttpMethod::Patch),
        "delete" => Ok(HttpMethod::Delete),
        _ => Err(prop_error(
            prop,
            "`method` must be get, post, put, patch, or delete",
        )),
    }
}

fn optional_http_headers_prop(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<Vec<OutboundHttpHeader>> {
    let Some(prop) = node.prop("headers") else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`headers` must be an array of { name:\"Header\" value:\"literal\" } objects",
        ));
    };
    values
        .iter()
        .map(|value| parse_http_header_value(prop, value, environment))
        .collect()
}

fn parse_http_header_value(
    prop: &SourceProp,
    value: &SourceValue,
    environment: &EnvironmentConfig,
) -> DoweResult<OutboundHttpHeader> {
    let SourceValue::Object(entries) = value else {
        return Err(prop_error(prop, "`headers` entries must be objects"));
    };
    let mut name = None;
    let mut header_value = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`headers` entries do not support spread"));
        };
        match key.as_str() {
            "name" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "header `name` must be a quoted string"));
                };
                name = Some(value.clone());
            }
            "value" => header_value = Some(parse_http_header_binding(prop, value, environment)?),
            _ => return Err(prop_error(prop, format!("unknown header field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "header entry missing `name`"))?;
    let name =
        normalize_http_header_name(&name).ok_or_else(|| prop_error(prop, "invalid header name"))?;
    if is_restricted_outbound_header(&name) {
        return Err(prop_error(
            prop,
            format!("header `{name}` is not allowed in outbound request headers"),
        ));
    }
    let value = header_value.ok_or_else(|| prop_error(prop, "header entry missing `value`"))?;
    Ok(OutboundHttpHeader { name, value })
}

fn parse_http_header_binding(
    prop: &SourceProp,
    value: &SourceValue,
    environment: &EnvironmentConfig,
) -> DoweResult<HttpHeaderValue> {
    match value {
        SourceValue::String(value) => Ok(HttpHeaderValue::Static(value.clone())),
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(prop_error(
                    prop,
                    "header `value` must be a quoted string or server env reference",
                ));
            };
            let variable = environment.variable(env_name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{env_name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(
                    prop,
                    format!("environment variable `{env_name}` must be server-only"),
                ));
            }
            Ok(HttpHeaderValue::Environment(env_name.to_string()))
        }
        _ => Err(prop_error(
            prop,
            "header `value` must be a quoted string or server env reference",
        )),
    }
}

fn is_restricted_outbound_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "set-cookie"
            | "host"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "upgrade"
            | "proxy-authorization"
            | "proxy-authenticate"
            | "te"
            | "trailer"
    )
}

fn required_secret_prop(
    node: &SourceNode,
    name: &str,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerSecret> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let Some(value) = prop.value.as_string_like() else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be an env reference"),
        ));
    };
    let Some(env_name) = value.strip_prefix("env.") else {
        return Err(prop_error(
            prop,
            format!("`{name}` must use a server env variable"),
        ));
    };
    let variable = environment
        .variable(env_name)
        .ok_or_else(|| prop_error(prop, format!("unknown environment variable `{env_name}`")))?;
    if variable.visibility != EnvironmentVisibility::Server {
        return Err(prop_error(
            prop,
            format!("environment variable `{env_name}` must be server-only"),
        ));
    }
    Ok(ServerSecret::Environment(env_name.to_string()))
}

fn required_algorithm_prop(node: &SourceNode, name: &str, allowed: &[&str]) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = required_static_string_prop(prop)?;
    if value == "none" {
        return Err(prop_error(prop, "`alg:\"none\"` is not supported"));
    }
    if allowed.iter().any(|allowed| *allowed == value) {
        Ok(value)
    } else {
        Err(prop_error(prop, format!("unsupported algorithm `{value}`")))
    }
}

fn required_store_literal_prop(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    store_literal(&prop.value)
}

fn handler_request_name(_node: &SourceNode) -> Option<&'static str> {
    Some("req")
}

fn handler_action_context(node: &SourceNode) -> ActionContext<'_> {
    ActionContext::HttpHandler {
        async_handler: true,
        request: handler_request_name(node),
    }
}

fn reject_explicit_handler_async(node: &SourceNode) -> DoweResult<()> {
    if node
        .args
        .iter()
        .any(|arg| arg.as_string_like().as_deref() == Some("async"))
    {
        return Err(node_error(
            node,
            "handlers are asynchronous by default; remove `async`",
        ));
    }
    Ok(())
}

fn child_named<'a>(node: &'a SourceNode, name: &str) -> Option<&'a SourceNode> {
    node.children.iter().find(|child| child.name == name)
}

fn single_root<'a>(
    path: &Path,
    nodes: &'a [SourceNode],
    expected: &str,
) -> DoweResult<&'a SourceNode> {
    let mut roots = nodes.iter().filter(|node| node.name == expected);
    let root = roots
        .next()
        .ok_or_else(|| DoweError::at_path(path, format!("missing `{expected}` block")))?;
    if roots.next().is_some() {
        return Err(DoweError::at_path(
            path,
            format!("multiple `{expected}` blocks are not supported"),
        ));
    }
    Ok(root)
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

fn required_static_string_prop(prop: &SourceProp) -> DoweResult<String> {
    match &prop.value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(DoweError::at_path(
            &prop.location.path,
            format!(
                "{}:{}: invalid value for prop `{}`: expected quoted static string literal",
                prop.location.line, prop.location.column, prop.name
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_server_file, parse_server_source};
    use crate::model::{
        EndpointBehavior, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
        EnvironmentVisibility, HttpConnectionValue, HttpHeaderValue, HttpMethod,
        HttpRedirectPolicy, HttpResponseMode, ServerJwtStatement, ServerLogValue,
        ServerMiddlewareStatement, ServerModelEngine, ServerModelFormat, ServerModelKind,
        ServerSecret, ServerStatement, ServerTransportProtocol, TlsDomainsSource, TlsMode,
    };
    use crate::parser::source_parser::parse_source_file;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn parses_main_server_route() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/status")
            .expect("route");

        assert_eq!(
            endpoint.endpoint.behavior,
            EndpointBehavior::StaticText("OK".to_string())
        );
    }

    #[test]
    fn parses_acme_tls_with_managed_kv_domains() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:443
    tls:
      mode:"acme"
      domains:["example.com", "www.example.com"]
      email:"admin@example.com"
      staging:false
      domainsFrom:{ kv:"domains" key:"tls" }
      refreshSeconds:90"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let tls = server.backend.tls.expect("tls");

        assert_eq!(tls.mode, TlsMode::Acme);
        assert_eq!(tls.domains, ["example.com", "www.example.com"]);
        assert_eq!(tls.email.as_deref(), Some("admin@example.com"));
        assert!(!tls.staging);
        assert_eq!(tls.refresh_seconds, 90);
        assert_eq!(
            tls.domains_from,
            Some(TlsDomainsSource::Kv {
                database: "domains".to_string(),
                key: "tls".to_string(),
            })
        );
    }

    #[test]
    fn parses_local_tls_and_database_domain_source() {
        let local = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:8443\n    tls mode:\"local\" domains:[\"localhost\", \"app.localhost\"]\n"
                .to_string(),
        )
        .expect("source");
        let local =
            parse_server_file(Path::new("/project/main.dowe"), &local.nodes).expect("local server");
        assert_eq!(local.backend.tls.expect("tls").mode, TlsMode::Local);

        let database = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:443\n    tls mode:\"acme\" email:\"admin@example.com\" domainsFrom:{ db:\"admin\" table:\"domains\" field:\"hostname\" }\n"
                .to_string(),
        )
        .expect("source");
        let database = parse_server_file(Path::new("/project/main.dowe"), &database.nodes)
            .expect("database server");
        assert!(matches!(
            database.backend.tls.expect("tls").domains_from,
            Some(TlsDomainsSource::Database { .. })
        ));
    }

    #[test]
    fn rejects_invalid_tls_contracts() {
        for (tls, message) in [
            (
                "tls mode:\"acme\" domains:[\"localhost\"] email:\"admin@example.com\"",
                "invalid public ACME domain",
            ),
            (
                "tls mode:\"acme\" domains:[\"192.0.2.1\"] email:\"admin@example.com\"",
                "invalid public ACME domain",
            ),
            (
                "tls mode:\"local\" domains:[\"example.com\"]",
                "local TLS does not support public domain",
            ),
            (
                "tls mode:\"acme\" domains:[\"example.com\"]",
                "requires a valid `email`",
            ),
            (
                "tls mode:\"acme\" domains:[\"example.com\"] email:\"admin@example.com\" cache:\"../tls\"",
                "must stay inside `.dowe`",
            ),
            (
                "tls mode:\"acme\" email:\"admin@example.com\" domainsFrom:{ kv:\"domains\" table:\"domains\" }",
                "must be `{ kv:",
            ),
        ] {
            let file = parse_source_file(
                Path::new("/project"),
                Path::new("/project/main.dowe"),
                format!("main\n  server port:443\n    {tls}\n"),
            )
            .expect("source");
            let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
                .expect_err("invalid tls");
            assert!(error.to_string().contains(message), "{error}");
        }
    }

    #[test]
    fn parses_protocol_transports_and_rtp_pool() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    udp name:"sip-udp" bind:"0.0.0.0" port:5060
      packet pkt
        log "udp" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" bind:"0.0.0.0" port:5060
      connection conn
        log "tcp" conn.addr conn.text conn.bytes
    rtp bind:"0.0.0.0" min:40000 max:40100
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert_eq!(server.backend.transports.len(), 2);
        assert_eq!(server.backend.transports[0].name, "sip-udp");
        assert_eq!(
            server.backend.transports[0].protocol,
            ServerTransportProtocol::Udp
        );
        assert_eq!(server.backend.transports[0].binding, "pkt");
        assert_eq!(
            server.backend.transports[1].protocol,
            ServerTransportProtocol::Tcp
        );
        assert_eq!(server.backend.transports[1].binding, "conn");
        let rtp = server.backend.rtp.expect("rtp");
        assert_eq!(rtp.bind, "0.0.0.0");
        assert!(rtp.contains(40000));
        assert!(rtp.contains(40100));
        assert!(!rtp.contains(40101));
    }

    #[test]
    fn parses_server_model_declarations() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    model name:"voice-vad" kind:"vad.silero" engine:"candle" format:"onnx" source:"assets/silero_vad.onnx" sampleRates:[8000,16000]
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let model = server.backend.models.first().expect("model");

        assert_eq!(model.name, "voice-vad");
        assert_eq!(model.kind, ServerModelKind::VadSilero);
        assert_eq!(model.engine, ServerModelEngine::Candle);
        assert_eq!(model.format, ServerModelFormat::Onnx);
        assert_eq!(model.sample_rates, vec![8_000, 16_000]);
    }

    #[test]
    fn parses_media_proxy_primitives() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/dash/:name/*segment"
      method GET async req
        request query source:"query"
        request raw source:"rawQuery"
        request range source:"header" name:"Range"
        request session source:"cookie" name:"session"
        http upstream method:"get" base:"https://media.example" path:"/segment.m4s" mode:"bytes" headers:[{ name:"Accept" value:"*/*" }]
        crypto decrypted encryption:"aesCtr" data:upstream key:"00000000000000000000000000000000" iv:"00000000000000000000000000000000"
        crypto cenc encryption:"cencAesCtr" data:decrypted key:"00000000000000000000000000000000" iv:"0000000000000000" subsamples:[{ clear:5 encrypted:10 }]
        spawn ffmpeg command:"ffmpeg" args:["-version"] timeoutMs:1000 maxOutputBytes:4096
        return bytes:cenc contentType:"video/mp4" headers:[{ name:"Cache-Control" value:"no-store" }] cookies:[{ name:"session" value:session path:"/" httpOnly:true sameSite:"Lax" maxAge:60 }]"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/dash/news/video/1.m4s")
            .expect("route");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpBytes(_)
        ));
        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::RequestQuery { .. }
        ));
        assert!(matches!(
            endpoint.endpoint.action.statements[4],
            ServerStatement::Http(_)
        ));
        assert!(matches!(
            endpoint.endpoint.action.statements[5],
            ServerStatement::CryptoAesCtr(_)
        ));
        assert!(matches!(
            endpoint.endpoint.action.statements[6],
            ServerStatement::CryptoCencAesCtr(_)
        ));
        assert!(matches!(
            endpoint.endpoint.action.statements[7],
            ServerStatement::Spawn(_)
        ));
    }

    #[test]
    fn rejects_legacy_spawn_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/inspect"
      method GET async req
        let ffmpeg = dowe.spawn command:"ffmpeg" args:["-version"]
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");

        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy spawn assignment must fail");

        assert!(
            error
                .to_string()
                .contains("spawn uses `spawn <binding> command:<value> [args:<array>]`")
        );
    }

    #[test]
    fn rejects_legacy_http_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/status"
      method GET async req
        let upstream = http.get base:"https://media.example" path:"/api/status"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");

        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy HTTP assignment must fail");

        assert!(
            error
                .to_string()
                .contains("http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`")
        );
    }

    #[test]
    fn rejects_legacy_crypto_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/segment"
      method GET async req
        let encrypted = crypto.cencAesCtr data:req.body key:"00000000000000000000000000000000" iv:"0000000000000000"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");

        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy crypto assignment must fail");

        assert!(
            error.to_string().contains(
                "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`"
            )
        );
    }

    #[test]
    fn parses_cenc_crypto_declaration_without_subsamples() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/segment"
      method GET async req
        http upstream method:"get" base:"https://media.example" path:"/segment.m4s" mode:"bytes"
        crypto encrypted encryption:"cencAesCtr" data:upstream key:"00000000000000000000000000000000" iv:"0000000000000000"
        return bytes:encrypted contentType:"video/mp4""#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/segment")
            .expect("route");

        assert!(matches!(
            endpoint.endpoint.action.statements[1],
            ServerStatement::CryptoCencAesCtr(_)
        ));
    }

    #[test]
    fn parses_route_middlewares_from_imports() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("features/blogs")).expect("api");
        fs::create_dir_all(root.join("shared/authentication")).expect("middleware");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/features/blogs/api"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");
        fs::write(
            root.join("features/blogs/api.dowe"),
            r#"import requireBearer from "../../shared/authentication/bearer"

endpoints apiRoutes
  get path:"/users/:id" middleware:[requireBearer]
    return text:"Hello""#,
        )
        .expect("api");
        fs::write(
            root.join("shared/authentication/bearer.dowe"),
            r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub claims:verified.claims } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
        )
        .expect("middleware");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "JWT_SECRET".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/users/123")
            .expect("endpoint");

        assert_eq!(endpoint.endpoint.middlewares.len(), 1);
        assert_eq!(endpoint.endpoint.middlewares[0].name, "requireBearer");
        assert!(matches!(
            &endpoint.endpoint.middlewares[0].action.statements[1],
            ServerMiddlewareStatement::Jwt(ServerJwtStatement::Verify {
                secret: ServerSecret::Environment(name),
                algorithm,
                ..
            }) if name == "JWT_SECRET" && algorithm == "HS256"
        ));
    }

    #[test]
    fn parses_capability_first_session_verification() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        write_session_middleware_project(
            root,
            "session verified cache:appCache database:appDb token:token maxAge:2592000",
        );

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/private")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.middlewares[0].action.statements[1],
            ServerMiddlewareStatement::SessionVerify {
                binding,
                max_age_seconds: 2_592_000,
                ..
            } if binding == "verified"
        ));
    }

    #[test]
    fn rejects_legacy_session_verification_assignment() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        write_session_middleware_project(
            root,
            "let verified = session.verify cache:appCache database:appDb token:token maxAge:2592000",
        );

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("legacy session expression");

        assert!(
            error
                .to_string()
                .contains("session verification uses `session <binding>")
        );
    }

    #[test]
    fn parses_server_function_call_from_middleware() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/middlewares")).expect("middleware");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import requireAccess from "@/server/middlewares/access"

main
  server port:8080
    route "/api/private" middleware:[requireAccess]
      response text:"Private""#,
        )
        .expect("main");
        fs::write(
            root.join("server/middlewares/access.dowe"),
            r#"import authorizeRequest from "../services/access"

middleware requireAccess
  bearer token value:req.header.Authorization
  authorizeRequest verification args:{ authorization:token }
  if verification.valid
    next
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
        )
        .expect("middleware");
        fs::write(
            root.join("server/services/access.dowe"),
            r#"fn authorizeRequest params:{ authorization:string }
  return value:{ valid:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/private")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.middlewares[0].action.statements[1],
            ServerMiddlewareStatement::Call(call)
                if call.binding == "verification" && call.target == "authorizeRequest"
        ));
    }

    #[test]
    fn parses_multiple_handlers_imported_from_one_module() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server/api"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");
        fs::write(
            root.join("server/api.dowe"),
            r#"import { listBlogs, createBlog } from "./handlers/blogs"

endpoints apiRoutes
  group path:"/api/blogs"
    get path:"" handler:listBlogs
    post path:"" handler:createBlog"#,
        )
        .expect("api");
        fs::write(
            root.join("server/handlers/blogs.dowe"),
            r#"handler listBlogs
  return text:"List"

handler createBlog
  return text:"Created""#,
        )
        .expect("handlers");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Get, "/api/blogs")
                .is_some()
        );
        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Post, "/api/blogs")
                .is_some()
        );
    }

    #[test]
    fn parses_jwt_result_binding_in_handler() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(
            root.join("main.dowe"),
            r#"main
  server port:8080
    route "/login"
      handler
        jwt token secret:env.JWT_SECRET algorithm:"HS256" claims:{ sub:"user-1" }
        return json:{ ok:true data:{ token:token } }"#,
        )
        .expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "JWT_SECRET".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/login")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Jwt(ServerJwtStatement::Sign { .. })
        ));
    }

    #[test]
    fn rejects_middleware_without_next_or_response() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("middlewares")).expect("middlewares");
        fs::write(
            root.join("main.dowe"),
            r#"import requireBearer from "@/middlewares/auth"

main
  server port:8080
    route "/users" middleware:[requireBearer]
      handler
        return json:{ ok:true }"#,
        )
        .expect("main");
        fs::write(
            root.join("middlewares/auth.dowe"),
            r#"middleware requireBearer
  bearer token value:req.header.Authorization"#,
        )
        .expect("middleware");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("middleware must not fall through");

        assert!(
            error
                .to_string()
                .contains("must call `next` or return a response")
        );
    }

    #[test]
    fn parses_implicit_handler_request_binding() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import blogDetail from "@/handlers/blogs"

main
  server port:0
    route "/blogs/:id"
      method GET handler:blogDetail"#,
        )
        .expect("main");
        fs::write(
            root.join("handlers/blogs.dowe"),
            r#"handler blogDetail
  return json:{ id:req.params.id }"#,
        )
        .expect("handler");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/blogs/:id")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpActionJson(_)
        ));
    }

    #[test]
    fn parses_implicit_async_handler_operations() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/products"
      handler fetchProducts
        http upstream method:"get" base:"https://example.com" path:"/products"
        return json:{ ok:upstream.ok data:upstream.json }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/products")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Http(_)
        ));
    }

    #[test]
    fn rejects_explicit_async_handler_marker() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/products"
      handler fetchProducts async
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("explicit async handler");

        assert!(
            error
                .to_string()
                .contains("handlers are asynchronous by default; remove `async`")
        );
    }

    #[test]
    fn preserves_explicit_handler_request_compatibility() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/blogs/:id"
      handler blogDetail req
        return json:{ id:req.params.id }"#
                .to_string(),
        )
        .expect("source");

        parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect("explicit request alias");
    }

    #[test]
    fn parses_encrypted_jwt_handler_without_request_binding() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/session"
      handler createEncryptedSession
        jwt token key:env.JWT_KEY algorithm:"dir" encryption:"A256GCM" claims:{ sub:"user-1" }
        return json:{ token:token }"#
                .to_string(),
        )
        .expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "JWT_KEY".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let server =
            parse_server_source(Path::new("/project"), &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/session")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Jwt(ServerJwtStatement::Encrypt { .. })
        ));
    }

    #[test]
    fn rejects_legacy_jwt_let_expression() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/login"
      handler req
        let token = jwt.sign claims:{ sub:"user-1" } secret:env.JWT_SECRET algorithm:"HS256"
        return json:{ token:token }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy JWT let must fail");

        assert!(error.to_string().contains("JWT expressions use"));
    }

    #[test]
    fn parses_typed_server_function_call_chain() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::create_dir_all(root.join("server/repositories")).expect("repositories");
        fs::write(
            root.join("main.dowe"),
            r#"import listTickets from "@/server/handlers/tickets"

main
  server port:0
    route "/api/tickets"
      method GET handler:listTickets"#,
        )
        .expect("main");
        fs::write(
            root.join("server/handlers/tickets.dowe"),
            r#"import listTicketsService from "../services/tickets"

handler listTickets req
  listTicketsService result args:{ status:"open" }
  return json:result"#,
        )
        .expect("handler");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"import listTicketsRepository from "../repositories/tickets"

fn listTicketsService params:{ status:string }
  listTicketsRepository result args:{ status:args.status }
  return value:{ ok:true data:result.rows cache:result.cache }"#,
        )
        .expect("function");
        fs::write(
            root.join("server/repositories/tickets.dowe"),
            r#"fn listTicketsRepository params:{ status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query rows db:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-list" value:{ status:args.status }
  return value:{ rows:rows cache:saved }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/tickets")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::Call(call) if call.binding == "result" && call.target == "listTicketsService"
        ));
        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpActionJson(_)
        ));
    }

    #[test]
    fn rejects_legacy_server_function_assignment() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler
        let result = saveTicket args:{ title:"Open" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn saveTicket params:{ title:string }
  return value:{ title:args.title }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("legacy function call");

        assert!(
            error
                .to_string()
                .contains("server function calls use `saveTicket result args:{ ... }`")
        );
    }

    #[test]
    fn rejects_invalid_server_function_call_shape() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn saveTicket
  return value:{ ok:true }"#,
        )
        .expect("function");

        for (call, expected) in [
            (
                "saveTicket",
                "server function call requires one result binding",
            ),
            (
                "saveTicket first second",
                "server function call requires one result binding",
            ),
            (
                "saveTicket result unsupported:true",
                "unknown prop `unsupported`",
            ),
        ] {
            fs::write(
                root.join("main.dowe"),
                format!(
                    "import saveTicket from \"@/server/services/tickets\"\n\nmain\n  server port:0\n    route \"/api/tickets\"\n      handler\n        {call}\n        return json:{{ ok:true }}"
                ),
            )
            .expect("main");
            let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
            let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
            let error = parse_server_source(root, &file, &EnvironmentConfig::default())
                .expect_err("invalid function call");

            assert!(error.to_string().contains(expected), "{call}: {error}");
        }
    }

    #[test]
    fn validates_server_function_params_and_return_contracts() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:{ title:"Open" } }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/tickets")
            .expect("endpoint");
        let ServerStatement::Call(call) = &endpoint.endpoint.action.statements[0] else {
            panic!("server function call");
        };
        assert_eq!(call.action.params[0].name, "ticket");
        assert_eq!(call.action.params[0].type_name, "TicketInput");
        assert_eq!(
            call.action
                .return_type
                .as_ref()
                .map(|value| value.type_name.as_str()),
            Some("TicketOutput")
        );
    }

    #[test]
    fn rejects_incompatible_server_function_return() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:"invalid" }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function return error");

        assert!(
            error
                .to_string()
                .contains("function return value is incompatible")
        );
    }

    #[test]
    fn rejects_incompatible_server_function_args() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

fn saveTicket params:{ ticket:TicketInput }
  return value:{ ok:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function argument error");

        assert!(
            error
                .to_string()
                .contains("argument `ticket` is incompatible")
        );
    }

    #[test]
    fn parses_go_and_cron_jobs_from_server_init() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("main.dowe"),
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      runCleanup startupResult args:{ source:"direct" }
      go runCleanup args:{ source:"startup" }
      cron runCleanup schedule:"*/15 * * * *" args:{ source:"cron" }"#,
        )
        .expect("main");
        fs::write(
            root.join("server/tasks/cleanup.dowe"),
            r#"fn runCleanup params:{ source:string }
  log args.source
  return value:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(matches!(
            &server.backend.init_action.statements[0],
            ServerStatement::Call(call)
                if call.binding == "startupResult" && call.target == "runCleanup"
        ));
        assert!(matches!(
            &server.backend.init_action.statements[1],
            ServerStatement::Go(job) if job.target == "runCleanup" && job.schedule.is_none()
        ));
        assert!(matches!(
            &server.backend.init_action.statements[2],
            ServerStatement::Cron(job)
                if job.target == "runCleanup" && job.schedule.as_deref() == Some("*/15 * * * *")
        ));
    }

    #[test]
    fn rejects_invalid_background_jobs() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("main.dowe"),
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      cron runCleanup schedule:"60 * * * *"
    route "/run"
      handler req
        go runCleanup args:{ source:req.params.id }
        return text:"OK""#,
        )
        .expect("main");
        fs::write(
            root.join("server/tasks/cleanup.dowe"),
            r#"fn runCleanup
  return value:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid cron");

        assert!(error.to_string().contains("cron value `60`"));
    }

    #[test]
    fn functions_import_store_handles_from_config_modules() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::create_dir_all(root.join("server/repositories")).expect("repositories");
        fs::create_dir_all(root.join("server/config")).expect("config");
        fs::write(
            root.join("main.dowe"),
            r#"import listAccounts from "@/server/handlers/accounts"

main
  server port:0
    route "/api/accounts"
      method GET handler:listAccounts"#,
        )
        .expect("main");
        fs::write(
            root.join("server/handlers/accounts.dowe"),
            r#"import listAccountsService from "../services/accounts"

handler listAccounts req
  listAccountsService result
  return json:result"#,
        )
        .expect("handler");
        fs::write(
            root.join("server/services/accounts.dowe"),
            r#"import listAccountsRepository from "../repositories/accounts"

fn listAccountsService
  listAccountsRepository result
  return value:{ rows:result.rows }"#,
        )
        .expect("function");
        fs::write(
            root.join("server/config/db.dowe"),
            r#"entity Accounts
  id:string primary:true
  name:string required:true index:true

seeder Bootstrap
  insert entity:Accounts value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Primary" }

database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"iptv" entities:[Accounts] seeders:[Bootstrap]"#,
        )
        .expect("config");
        fs::write(
            root.join("server/repositories/accounts.dowe"),
            r#"import db from "../config/db"

fn listAccountsRepository
  query rows db:db.list table:"directvAccounts"
  return value:{ rows:rows }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/accounts")
            .expect("endpoint");

        let ServerStatement::Call(service_call) = &endpoint.endpoint.action.statements[0] else {
            panic!("function call");
        };
        let ServerStatement::Call(repository_call) = &service_call.action.statements[0] else {
            panic!("nested function call");
        };
        assert!(matches!(
            &repository_call.action.statements[0],
            ServerStatement::Store(crate::model::ServerStoreStatement::Handle {
                connection
            }) if connection.binding == "db"
                && connection.database == "iptv"
                && connection.entities.len() == 1
                && connection.entities[0].binding == "Accounts"
                && connection.seeders.len() == 1
                && connection.seeders[0].binding == "Bootstrap"
        ));
        assert!(matches!(
            &repository_call.action.statements[1],
            ServerStatement::Store(crate::model::ServerStoreStatement::List {
                handle,
                table,
                ..
            }) if handle == "db" && table == "directvAccounts"
        ));
    }

    #[test]
    fn accepts_server_functions_from_arbitrary_module_paths() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();

        for relative in [
            "server/services/example.dowe",
            "domains/accounts/application/example.dowe",
            "shared/example.dowe",
            "example.dowe",
        ] {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("directory");
            fs::write(&path, "fn example\n  return value:{ ok:true }\n").expect("function");
            let source = fs::read_to_string(&path).expect("source");
            let file = parse_source_file(root, &path, source).expect("file");

            super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
                .expect("declaration-based function module");
        }
    }

    #[test]
    fn accepts_server_config_from_arbitrary_module_path() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        let path = root.join("domains/accounts/storage.dowe");
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        fs::write(
            &path,
            "database accountsDb provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"accounts\"\n",
        )
        .expect("config");
        let source = fs::read_to_string(&path).expect("source");
        let file = parse_source_file(root, &path, source).expect("file");

        super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
            .expect("declaration-based config module");
    }

    #[test]
    fn rejects_store_operations_inside_config_modules() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("config")).expect("config");
        fs::write(
            root.join("config/db.dowe"),
            r#"query rows db:db.list table:"directvAccounts""#,
        )
        .expect("config");
        let source = fs::read_to_string(root.join("config/db.dowe")).expect("config source");
        let file = parse_source_file(root, &root.join("config/db.dowe"), source).expect("source");
        let error =
            super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
                .expect_err("config operation error");

        assert!(
            error
                .to_string()
                .contains("config modules only support database handle bindings")
        );
    }

    #[test]
    fn rejects_legacy_service_declarations() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import listTickets from "@/handlers/tickets"

main
  server port:0
    route "/api/tickets"
      method GET handler:listTickets"#,
        )
        .expect("main");
        fs::write(
            root.join("handlers/tickets.dowe"),
            r#"service listTickets
  return value:{ ok:true }"#,
        )
        .expect("legacy service");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("legacy service error");

        assert!(error.to_string().contains("replaced by `fn`"));
    }

    #[test]
    fn rejects_response_return_inside_server_function() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import listTickets from "@/server/handlers/tickets"

main
  server port:0
    route "/api/tickets"
      method GET handler:listTickets"#,
        )
        .expect("main");
        fs::write(
            root.join("server/handlers/tickets.dowe"),
            r#"import listTicketsService from "../services/tickets"

handler listTickets req
  listTicketsService result
  return json:result"#,
        )
        .expect("handler");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn listTicketsService
  return json:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function return error");

        assert!(error.to_string().contains("return value"));
    }

    #[test]
    fn rejects_client_environment_for_remote_store_credentials() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:env.DB_TOKEN name:"db1"
        query users db:db.list table:"users"
        return json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "DB_TOKEN".to_string(),
                visibility: EnvironmentVisibility::Client,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn parses_outbound_http_proxy_response() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/v1/chat/completions"
      method POST async req
        const body value:req.json
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:body mode:"proxy"
        return proxy:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/v1/chat/completions")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpProxy(_)
        ));
        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Http(request)
                if request.mode == HttpResponseMode::Proxy
                    && request.base == HttpConnectionValue::Environment("OPENROUTER_BASE_URL".to_string())
        ));
    }

    #[test]
    fn parses_general_outbound_http_request_options() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method PATCH async req
        const body value:req.json
        http upstream method:"patch" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"Accept" value:"application/json" }, { name:"X-Api-Key" value:env.OPENROUTER_API_KEY }] json:body redirect:"manual" timeoutMs:5000 mode:"json"
        return json:{ ok:upstream.ok status:upstream.status location:upstream.location data:upstream.json }"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Patch, "/api/products")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Http(request)
                if request.method == HttpMethod::Patch
                    && request.redirect == HttpRedirectPolicy::Manual
                    && request.timeout_ms == Some(5000)
                    && request.headers.len() == 2
                    && request.headers[0].name == "Accept"
                    && request.headers[0].value == HttpHeaderValue::Static("application/json".to_string())
                    && request.headers[1].name == "X-Api-Key"
                    && request.headers[1].value == HttpHeaderValue::Environment("OPENROUTER_API_KEY".to_string())
        ));
    }

    #[test]
    fn parses_stdlib_capability_statement() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string

main
  server port:8080
    route "/api/normalize"
      method POST async req
        const body:User value:req.json
        str normalized source:"trim" value:body.name
        return json:{ name:normalized }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/normalize")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Stdlib(statement)
                if statement.binding == "normalized"
                    && statement.call.namespace == "str"
                    && statement.call.function == "trim"
                    && statement.call.args[0].name == "value"
        ));
    }

    #[test]
    fn rejects_legacy_stdlib_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/session"
      handler
        let sessionKey = str.join values:["session", req.params.id] delimiter:":"
        return json:{ key:sessionKey }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy stdlib assignment");

        assert!(error.to_string().contains("str sessionKey source:\"join\""));
    }

    #[test]
    fn parses_request_websocket_and_agent_capabilities() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/inspect"
      handler
        request query source:"query"
        request range source:"header" name:"Range"
        return json:{ query:query range:range }
    websocket "/agent"
      message ws
        ws request source:"json"
        agent chat source:"chat" request:request
        send ws json:chat"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/inspect")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestQuery { binding } if binding == "query"
        ));
        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::RequestHeader { binding, name }
                if binding == "range" && name == "Range"
        ));
        assert!(matches!(
            &server.backend.websockets[0].handlers.message.statements[0],
            ServerStatement::WebSocketJson(statement) if statement.binding == "request"
        ));
        assert!(matches!(
            &server.backend.websockets[0].handlers.message.statements[1],
            ServerStatement::AgentChat(statement)
                if statement.binding == "chat" && statement.source == "request"
        ));
    }

    #[test]
    fn rejects_legacy_response_selector() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/health"
      handler
        return response json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy response selector");

        assert!(
            error
                .to_string()
                .contains("HTTP returns use `return <props>`; remove `response`")
        );
    }

    #[test]
    fn rejects_authorization_header_on_outbound_http_request() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream method:"get" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"Authorization" value:"Bearer token" }]
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("not allowed"));
    }

    #[test]
    fn rejects_client_environment_for_outbound_http_header() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream method:"get" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"X-Api-Key" value:env.OPENROUTER_API_KEY }]
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Client);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn rejects_outbound_http_request_without_method() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream base:env.OPENROUTER_BASE_URL path:"/v1/products"
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("missing `method`"));
    }

    #[test]
    fn rejects_client_environment_for_outbound_http_bearer() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/v1/chat/completions"
      method POST async req
        const body value:req.json
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:body mode:"proxy"
        return proxy:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Client);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn parses_static_json_response() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/health"
      handler
        return json:{ ok:true service:"dowe-llm-server" }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/health")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpActionJson(_)
        ));
    }

    #[test]
    fn parses_declared_websocket_http_bridge() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    websocket "/api/v1/agent/ws"
      message ws
        ws request source:"json"
        send ws json:{ event:"started" requestId:request.requestId requestType:request.requestType model:request.model payload:{ stream:request.stream } }
        agent chat source:"chat" request:request
        http upstream method:"post" base:"https://openrouter.ai" path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"proxy"
        bridge sse:upstream to:ws requestId:request.requestId requestType:request.requestType model:request.model"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let server = parse_server_source(root, &file, &environment).expect("server");
        let route = server
            .backend
            .find_websocket("/api/v1/agent/ws")
            .expect("websocket");
        let statements = &route.handlers.message.statements;

        assert!(matches!(&statements[0], ServerStatement::WebSocketJson(_)));
        assert!(matches!(
            &statements[1],
            ServerStatement::WebSocketSendJson(_)
        ));
        assert!(matches!(&statements[2], ServerStatement::AgentChat(_)));
        assert!(matches!(&statements[3], ServerStatement::Http(_)));
        assert!(matches!(
            &statements[4],
            ServerStatement::WebSocketSseBridge(_)
        ));
    }

    #[test]
    fn rejects_legacy_app_root() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"app
  server port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `main`"));
    }

    #[test]
    fn rejects_legacy_backend_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  backend port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `server`"));
    }

    #[test]
    fn rejects_legacy_endpoint_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    endpoint "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `route`"));
    }

    #[test]
    fn infers_store_insert_fields_for_log_references() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"blogs" value:{ title:"First" }
        log created.title
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/blogs")
            .expect("route");

        assert!(endpoint.endpoint.action.statements.iter().any(|statement| matches!(
            statement,
            ServerStatement::Log(log)
                if log.values == vec![ServerLogValue::Reference("created.title".to_string())]
        )));
        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::StoreActionJson(_)
        ));
    }

    #[test]
    fn rejects_unknown_store_insert_fields_in_logs() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"blogs" value:{ title:"First" }
        log created.missing
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unknown field `created.missing`")
        );
    }

    #[test]
    fn rejects_unknown_store_insert_fields_in_json_responses() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"blogs" value:{ title:"First" }
        return json:{ data:created.missing }"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unknown field `created.missing`")
        );
    }

    #[test]
    fn validates_typed_request_body_references() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:User value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.name age:body.age }
        return json:{ ok:true user:created }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/users")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestJson {
                binding,
                schema: Some(_)
            } if binding == "body"
        ));
    }

    #[test]
    fn rejects_legacy_request_json_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/users"
      method POST async req
        let body = await req.json()
        return json:body"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("const <binding[:Type]> value:req.json")
        );
    }

    #[test]
    fn validates_shared_type_imported_by_request_body() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("types")).expect("types");
        fs::write(
            root.join("types/users.dowe"),
            r#"type UserInput
  name:string
  age:number"#,
        )
        .expect("type source");
        let file = parse_source_file(
            root,
            &root.join("main.dowe"),
            r#"import UserInput from "@/types/users"

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:UserInput value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.name age:body.age }
        return json:{ ok:true user:created }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/users")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestJson {
                binding,
                schema: Some(_)
            } if binding == "body"
        ));
    }

    #[test]
    fn rejects_unknown_typed_request_body_fields_in_store_literals() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:User value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.email }
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("unknown field `body.email`"));
    }

    #[test]
    fn expands_grouped_endpoint_methods_and_websocket_middlewares() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::create_dir_all(root.join("middlewares")).expect("middlewares");
        fs::write(
            root.join("handlers/blogs.dowe"),
            r#"handler listBlogs
  return text:"List"

handler createBlog
  return text:"Created""#,
        )
        .expect("handlers");
        fs::write(
            root.join("middlewares/auth.dowe"),
            r#"middleware requireBearer
  next"#,
        )
        .expect("middleware");
        fs::write(
            root.join("server.dowe"),
            r#"import { listBlogs, createBlog } from "@/handlers/blogs"
import requireBearer from "@/middlewares/auth"

endpoints apiRoutes
  group path:"/api/blogs" middleware:[requireBearer]
    get path:"" handler:listBlogs
    post path:"/create" handler:createBlog middleware:[requireBearer]
    websocket path:"/events" middleware:[requireBearer]
      open ws
        log "open""#,
        )
        .expect("endpoints");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");

        let source = fs::read_to_string(root.join("main.dowe")).expect("source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("main file");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Get, "/api/blogs")
                .is_some()
        );
        let created = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/blogs/create")
            .expect("created endpoint");
        assert_eq!(created.endpoint.middlewares.len(), 2);
        let websocket = server
            .backend
            .find_websocket("/api/blogs/events")
            .expect("websocket");
        assert_eq!(websocket.middlewares.len(), 2);
    }

    #[test]
    fn rejects_nested_endpoint_groups() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server")).expect("server");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server/endpoints"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");
        fs::write(
            root.join("server/endpoints.dowe"),
            r#"endpoints apiRoutes
  group path:"/api"
    group middleware:[requireBearer]
      get path:"/blogs" handler:listBlogs"#,
        )
        .expect("endpoints");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");

        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("nested endpoint group");

        assert!(error.to_string().contains(
            "`endpoints` groups cannot contain another `group`; put middleware on the group or its HTTP method"
        ), "{error}");
    }

    #[test]
    fn parses_database_service_and_reserves_its_websocket_path() {
        let source = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:4147\n    database service\n".to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &source.nodes).expect("server");
        assert!(server.backend.database_service);

        let source = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:4147
    database service
    websocket path:"/v1/databases/:name"
      open ws
        log "open""#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &source.nodes)
            .expect_err("reserved route");
        assert!(
            error
                .to_string()
                .contains("reserves WebSocket path `/v1/databases/:name`"),
            "{error}"
        );
    }

    fn openrouter_environment(visibility: EnvironmentVisibility) -> EnvironmentConfig {
        EnvironmentConfig {
            variables: vec![
                EnvironmentVariable {
                    name: "OPENROUTER_BASE_URL".to_string(),
                    visibility: EnvironmentVisibility::Server,
                    resolved_source: EnvironmentValueSource::Missing,
                    resolved_value: None,
                },
                EnvironmentVariable {
                    name: "OPENROUTER_API_KEY".to_string(),
                    visibility,
                    resolved_source: EnvironmentValueSource::Missing,
                    resolved_value: None,
                },
            ],
        }
    }

    fn write_session_middleware_project(root: &Path, verification: &str) {
        fs::create_dir_all(root.join("server/config")).expect("config");
        fs::create_dir_all(root.join("server/middlewares")).expect("middlewares");
        fs::write(
            root.join("main.dowe"),
            r#"import requireBearer from "@/server/middlewares/auth"

main
  server port:8080
    route "/api/private" middleware:[requireBearer]
      handler
        return text:"Private""#,
        )
        .expect("main");
        fs::write(
            root.join("server/config/database.dowe"),
            r#"database appDb provider:"dowe" host:"127.0.0.1" port:4147 account:"app" secret:"secret" name:"app" entities:[] seeders:[]
cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"app""#,
        )
        .expect("config");
        fs::write(
            root.join("server/middlewares/auth.dowe"),
            format!(
                r#"import {{ appDb, appCache }} from "@/server/config/database"

middleware requireBearer
  bearer token value:req.header.Authorization
  {verification}
  if verified.valid
    next context:{{ auth:{{ subject:verified.userId session:verified.id }} }}
  return status:401 json:{{ ok:false error:"Unauthorized" }}"#
            ),
        )
        .expect("middleware");
    }
}
