#[derive(Clone)]
struct InspectorRouteSource {
    method: String,
    path: String,
    source: ServerInspectorSource,
    handler: Option<String>,
}

#[derive(Default)]
struct InspectorResourceAccumulator {
    kind: String,
    binding: String,
    provider: String,
    operations: HashSet<String>,
}

fn build_server_inspector(
    path: &Path,
    nodes: &[SourceNode],
    backend: &ServerConfig,
    databases: &[DatabaseBinding],
    excluded_seeder_paths: &HashSet<std::path::PathBuf>,
    include_seeders: bool,
) -> DoweResult<ServerInspectorManifest> {
    let mut source_nodes = Vec::new();
    let mut route_sources = Vec::new();
    let mut websocket_sources = Vec::new();
    let main = nodes.iter().find(|node| node.name == "main");
    let server = main.and_then(|node| node.children.iter().find(|child| child.name == "server"));
    if let Some(server) = server {
        collect_inspector_source_nodes(
            server,
            &mut source_nodes,
            &mut route_sources,
            &mut websocket_sources,
        );
    }
    collect_inspector_source_tree(
        path.parent().unwrap_or(path),
        path.parent().unwrap_or(path),
        &mut source_nodes,
        &mut route_sources,
        &mut websocket_sources,
        excluded_seeder_paths,
        include_seeders,
    );
    route_sources.sort_by(|left, right| {
        (
            left.method.as_str(),
            left.path.as_str(),
            left.source.path.as_str(),
            left.source.line,
        )
            .cmp(&(
                right.method.as_str(),
                right.path.as_str(),
                right.source.path.as_str(),
                right.source.line,
            ))
    });
    websocket_sources.sort_by(|left, right| {
        (
            left.path.as_str(),
            left.source.path.as_str(),
            left.source.line,
        )
            .cmp(&(
                right.path.as_str(),
                right.source.path.as_str(),
                right.source.line,
            ))
    });

    let mut routes = Vec::new();
    let mut nodes_out = Vec::new();
    let mut edges = Vec::new();
    let mut used_sources = HashSet::new();
    for (index, endpoint) in backend.endpoints.iter().enumerate() {
        let method = endpoint.method.as_str().to_string();
        let source_index = route_sources.iter().position(|source| {
            source.method == method
                && source.path == endpoint.path
                && !used_sources.contains(&source_index_key(source))
        });
        let source = source_index.and_then(|index| {
            let key = source_index_key(&route_sources[index]);
            used_sources.insert(key);
            Some(route_sources[index].clone())
        });
        let id = inspector_id("route", &format!("{method}:{}:{index}", endpoint.path));
        let source_location = source.as_ref().map(|value| value.source.clone());
        let handler = source.as_ref().and_then(|value| value.handler.clone());
        routes.push(ServerInspectorRoute {
            id: id.clone(),
            method: method.clone(),
            path: endpoint.path.clone(),
            behavior: behavior_label(&endpoint.behavior),
            source: source_location.clone(),
            handler: handler.clone(),
            parameters: inspector_parameters(&endpoint.path, &endpoint.action),
            headers: inspector_headers(&endpoint.action, &endpoint.middlewares),
            body: inspector_body(&endpoint.action),
            middleware: endpoint
                .middlewares
                .iter()
                .map(|middleware| middleware.name.clone())
                .collect(),
        });
        nodes_out.push(ServerInspectorNode {
            id: id.clone(),
            kind: "route".to_string(),
            label: format!("{method} {}", endpoint.path),
            source: source_location,
        });
        if let Some(handler) = handler {
            let handler_id = source_nodes
                .iter()
                .find(|node| node.kind == "handler" && node.label == handler)
                .map(|node| source_node_id(node))
                .unwrap_or_else(|| inspector_id("handler", &handler));
            edges.push(ServerInspectorEdge {
                from: id,
                to: handler_id,
                relation: "handler".to_string(),
            });
        }
    }

    let mut websockets = Vec::new();
    for (index, websocket) in backend.websockets.iter().enumerate() {
        let source = websocket_sources
            .iter()
            .find(|source| source.path == websocket.path)
            .map(|source| source.source.clone());
        let id = inspector_id("websocket", &format!("{}:{index}", websocket.path));
        websockets.push(ServerInspectorWebSocket {
            id: id.clone(),
            path: websocket.path.clone(),
            source: source.clone(),
            middleware: websocket
                .middlewares
                .iter()
                .map(|middleware| middleware.name.clone())
                .collect(),
            message_format: if websocket_action_uses_json(&websocket.handlers.message) {
                "json".to_string()
            } else {
                "text".to_string()
            },
        });
        nodes_out.push(ServerInspectorNode {
            id,
            kind: "websocket".to_string(),
            label: websocket.path.clone(),
            source,
        });
    }

    for source in source_nodes {
        let id = source_node_id(&source);
        if nodes_out.iter().any(|node| node.id == id) {
            continue;
        }
        nodes_out.push(ServerInspectorNode {
            id,
            kind: source.kind,
            label: source.label,
            source: Some(source.source),
        });
    }

    let mut resource_map = HashMap::<String, InspectorResourceAccumulator>::new();
    for endpoint in &backend.endpoints {
        collect_action_resources(&endpoint.action, &mut resource_map);
        for middleware in &endpoint.middlewares {
            collect_middleware_resources(middleware, &mut resource_map);
        }
    }
    collect_action_resources(&backend.init_action, &mut resource_map);
    let mut resources = resource_map
        .into_values()
        .map(|value| {
            let id = inspector_id(&value.kind, &value.binding);
            let mut operations = value.operations.into_iter().collect::<Vec<_>>();
            operations.sort();
            nodes_out.push(ServerInspectorNode {
                id: id.clone(),
                kind: "resource".to_string(),
                label: value.binding.clone(),
                source: None,
            });
            ServerInspectorResource {
                id,
                kind: value.kind,
                binding: value.binding,
                provider: value.provider,
                operations,
            }
        })
        .collect::<Vec<_>>();
    resources.sort_by(|left, right| left.id.cmp(&right.id));

    let mut entities = Vec::new();
    for database in databases {
        for entity in &database.connection.entities {
            let id = inspector_id("entity", &format!("{}:{}", entity.binding, entity.table));
            let fields = entity
                .fields
                .iter()
                .map(|field| field.name.clone())
                .collect::<Vec<_>>();
            let field_details = entity
                .fields
                .iter()
                .map(|field| ServerInspectorEntityField {
                    name: field.name.clone(),
                    field_type: inspector_field_type(field.field_type).to_string(),
                    primary: field.primary,
                    required: field.required,
                    unique: field.unique,
                    index: field.index,
                })
                .collect::<Vec<_>>();
            entities.push(ServerInspectorEntity {
                id: id.clone(),
                binding: entity.binding.clone(),
                database: database.connection.database.clone(),
                table: entity.table.clone(),
                fields,
                field_details,
                provider: format!("{:?}", database.connection.provider),
            });
            nodes_out.push(ServerInspectorNode {
                id,
                kind: "entity".to_string(),
                label: format!("{}.{}", entity.binding, entity.table),
                source: None,
            });
        }
    }
    entities.sort_by(|left, right| left.id.cmp(&right.id));

    let mut jobs = Vec::new();
    collect_jobs_from_action(
        &backend.init_action,
        path.parent().unwrap_or(path),
        &mut jobs,
    );
    for endpoint in &backend.endpoints {
        collect_jobs_from_action(&endpoint.action, path.parent().unwrap_or(path), &mut jobs);
    }
    jobs.sort_by(|left, right| left.id.cmp(&right.id));
    for job in &jobs {
        nodes_out.push(ServerInspectorNode {
            id: job.id.clone(),
            kind: job.kind.clone(),
            label: job.target.clone().unwrap_or_else(|| job.kind.clone()),
            source: job.source.clone(),
        });
    }

    let services = [
        ("database", backend.database_service, "/_dowe/database"),
        ("cache", backend.cache_service, "/v1/caches/:name"),
        ("vector", backend.vector_service, "/v1/vectors/:name"),
        ("queue", backend.queue_service, "/v1/queues/:name"),
    ]
    .into_iter()
    .map(|(kind, enabled, endpoint)| {
        if enabled {
            nodes_out.push(ServerInspectorNode {
                id: inspector_id("service", kind),
                kind: "service".to_string(),
                label: kind.to_string(),
                source: None,
            });
        }
        ServerInspectorService {
            kind: kind.to_string(),
            enabled,
            endpoint: endpoint.to_string(),
        }
    })
    .collect::<Vec<_>>();

    for route in &routes {
        for resource in &resources {
            if route.action_uses_resource(backend, resource) {
                edges.push(ServerInspectorEdge {
                    from: route.id.clone(),
                    to: resource.id.clone(),
                    relation: "uses".to_string(),
                });
            }
        }
    }
    nodes_out.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| {
        (left.from.as_str(), left.to.as_str()).cmp(&(right.from.as_str(), right.to.as_str()))
    });

    Ok(ServerInspectorManifest {
        schema_version: 2,
        port: backend.port,
        routes,
        websockets,
        nodes: nodes_out,
        edges,
        resources,
        entities,
        jobs,
        services,
    })
}

fn inspector_field_type(field_type: DatabaseFieldType) -> &'static str {
    match field_type {
        DatabaseFieldType::String => "string",
        DatabaseFieldType::Bool => "bool",
        DatabaseFieldType::Int => "int",
        DatabaseFieldType::Number => "number",
        DatabaseFieldType::Decimal => "decimal",
        DatabaseFieldType::Timestamp => "timestamp",
        DatabaseFieldType::Json => "json",
    }
}

fn inspector_parameters(path: &str, action: &ServerAction) -> Vec<ServerInspectorParameter> {
    let mut parameters = Vec::new();
    for segment in path.trim_matches('/').split('/') {
        let Some(name) = segment
            .strip_prefix(':')
            .or_else(|| segment.strip_prefix('*'))
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        parameters.push(ServerInspectorParameter {
            name: name.to_string(),
            location: "path".to_string(),
            required: true,
            field_type: "string".to_string(),
        });
    }
    for statement in &action.statements {
        let (name, location, field_type) = match statement {
            ServerStatement::RequestQuery { .. } => ("query", "query", "object"),
            ServerStatement::RequestRawQuery { .. } => ("rawQuery", "query", "string"),
            ServerStatement::RequestCookie { name, .. } => (name.as_str(), "cookie", "string"),
            _ => continue,
        };
        if parameters
            .iter()
            .any(|parameter| parameter.name == name && parameter.location == location)
        {
            continue;
        }
        parameters.push(ServerInspectorParameter {
            name: name.to_string(),
            location: location.to_string(),
            required: false,
            field_type: field_type.to_string(),
        });
    }
    parameters
}

fn inspector_headers(
    action: &ServerAction,
    middlewares: &[ServerMiddleware],
) -> Vec<ServerInspectorHeader> {
    let mut headers = Vec::new();
    for statement in &action.statements {
        if let ServerStatement::RequestHeader { name, .. } = statement {
            add_inspector_header(&mut headers, name, false, false);
        }
    }
    for middleware in middlewares {
        collect_middleware_headers(&middleware.action.statements, &mut headers);
    }
    headers
}

fn collect_middleware_headers(
    statements: &[ServerMiddlewareStatement],
    headers: &mut Vec<ServerInspectorHeader>,
) {
    for statement in statements {
        match statement {
            ServerMiddlewareStatement::Header { name, .. } => {
                add_inspector_header(headers, name, false, false)
            }
            ServerMiddlewareStatement::Bearer { source, .. } => {
                if let Some(name) = source.strip_prefix("req.header.") {
                    add_inspector_header(headers, name, true, true);
                }
            }
            ServerMiddlewareStatement::Jwt(statement) => match statement {
                ServerJwtStatement::Verify { token, .. }
                | ServerJwtStatement::Decrypt { token, .. } => {
                    if let Some(name) = token.strip_prefix("req.header.") {
                        add_inspector_header(headers, name, true, true);
                    }
                }
                ServerJwtStatement::Sign { .. } | ServerJwtStatement::Encrypt { .. } => {}
            },
            ServerMiddlewareStatement::SessionVerify { .. } => {
                add_inspector_header(headers, "Authorization", true, true);
            }
            ServerMiddlewareStatement::IfValid { statements, .. } => {
                collect_middleware_headers(statements, headers)
            }
            _ => {}
        }
    }
}

fn add_inspector_header(
    headers: &mut Vec<ServerInspectorHeader>,
    name: &str,
    required: bool,
    sensitive: bool,
) {
    if let Some(existing) = headers
        .iter_mut()
        .find(|header| header.name.eq_ignore_ascii_case(name))
    {
        existing.required |= required;
        existing.sensitive |= sensitive;
        return;
    }
    headers.push(ServerInspectorHeader {
        name: name.to_string(),
        required,
        sensitive,
    });
}

fn inspector_body(action: &ServerAction) -> Option<ServerInspectorBody> {
    for statement in &action.statements {
        match statement {
            ServerStatement::RequestJson { schema, .. } => {
                return Some(ServerInspectorBody {
                    content_type: "application/json".to_string(),
                    required: true,
                    fields: schema
                        .as_ref()
                        .map(inspector_body_fields)
                        .unwrap_or_default(),
                });
            }
            ServerStatement::RequestBytes { .. } => {
                return Some(ServerInspectorBody {
                    content_type: "application/octet-stream".to_string(),
                    required: true,
                    fields: Vec::new(),
                });
            }
            _ => {}
        }
    }
    None
}

fn inspector_body_fields(schema: &DoweType) -> Vec<ServerInspectorBodyField> {
    let DoweType::Object(fields) = schema else {
        return Vec::new();
    };
    fields
        .iter()
        .map(|field| ServerInspectorBodyField {
            name: field.name.clone(),
            field_type: inspector_dowe_type(&field.value),
            required: !field.optional,
        })
        .collect()
}

fn inspector_dowe_type(value: &DoweType) -> String {
    match value {
        DoweType::Unknown => "unknown".to_string(),
        DoweType::Null => "null".to_string(),
        DoweType::Bool => "boolean".to_string(),
        DoweType::Number => "number".to_string(),
        DoweType::String => "string".to_string(),
        DoweType::Array(_) => "array".to_string(),
        DoweType::Object(_) => "object".to_string(),
    }
}

fn websocket_action_uses_json(action: &ServerAction) -> bool {
    action.statements.iter().any(|statement| match statement {
        ServerStatement::WebSocketJson(_) => true,
        ServerStatement::Call(call) => websocket_action_uses_json(&ServerAction {
            statements: call.action.statements.clone(),
        }),
        ServerStatement::Task(job) | ServerStatement::Cron(job) => {
            websocket_action_uses_json(&ServerAction {
                statements: job.action.statements.clone(),
            })
        }
        _ => false,
    })
}

impl ServerInspectorRoute {
    fn action_uses_resource(
        &self,
        backend: &ServerConfig,
        resource: &ServerInspectorResource,
    ) -> bool {
        backend
            .endpoints
            .iter()
            .find(|endpoint| endpoint.method.as_str() == self.method && endpoint.path == self.path)
            .is_some_and(|endpoint| action_uses_binding(&endpoint.action, &resource.binding))
    }
}

fn collect_inspector_source_nodes(
    server: &SourceNode,
    nodes: &mut Vec<InspectorSourceNode>,
    routes: &mut Vec<InspectorRouteSource>,
    websockets: &mut Vec<InspectorRouteSource>,
) {
    for node in &server.children {
        let source = source_for_node(node);
        match node.name.as_str() {
            "route" => collect_route_source(node, routes),
            "websocket" => {
                if let Some(path) = node.args.first().and_then(SourceValue::as_string_like) {
                    websockets.push(InspectorRouteSource {
                        method: "WS".to_string(),
                        path,
                        source,
                        handler: None,
                    });
                }
            }
            "endpoints" | "handler" | "middleware" | "fn" | "entity" | "seeder" | "database"
            | "cache" | "vector" | "queue" => {
                let label = node
                    .args
                    .first()
                    .and_then(SourceValue::as_string_like)
                    .unwrap_or_else(|| node.name.clone());
                nodes.push(InspectorSourceNode {
                    kind: node.name.clone(),
                    label,
                    source,
                });
            }
            _ => {}
        }
    }
}

fn collect_inspector_source_tree(
    root: &Path,
    directory: &Path,
    nodes: &mut Vec<InspectorSourceNode>,
    routes: &mut Vec<InspectorRouteSource>,
    websockets: &mut Vec<InspectorRouteSource>,
    excluded_seeder_paths: &HashSet<std::path::PathBuf>,
    include_seeders: bool,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.file_name().and_then(|name| name.to_str()) == Some(".dowe") {
            continue;
        }
        if file_path.is_dir() {
            collect_inspector_source_tree(
                root,
                &file_path,
                nodes,
                routes,
                websockets,
                excluded_seeder_paths,
                include_seeders,
            );
            continue;
        }
        if file_path.extension().and_then(|value| value.to_str()) != Some("dowe") {
            continue;
        }
        if !include_seeders && excluded_seeder_paths.contains(&file_path) {
            continue;
        }
        let Ok(source) = fs::read_to_string(&file_path) else {
            continue;
        };
        let Ok(file) = parse_source_file(root, &file_path, source) else {
            continue;
        };
        for node in &file.nodes {
            if node.name == "main" {
                if let Some(server) = node.children.iter().find(|child| child.name == "server") {
                    collect_inspector_source_nodes(server, nodes, routes, websockets);
                }
            } else {
                collect_module_inspector_node(node, nodes, routes, websockets, "");
            }
        }
    }
}

fn collect_module_inspector_node(
    node: &SourceNode,
    nodes: &mut Vec<InspectorSourceNode>,
    routes: &mut Vec<InspectorRouteSource>,
    websockets: &mut Vec<InspectorRouteSource>,
    scope: &str,
) {
    match node.name.as_str() {
        "endpoints" => collect_endpoint_group_sources(node, scope, routes, websockets),
        "route" => collect_route_source(node, routes),
        "websocket" => {
            if let Some(path) = node.args.first().and_then(SourceValue::as_string_like) {
                websockets.push(InspectorRouteSource {
                    method: "WS".to_string(),
                    path,
                    source: source_for_node(node),
                    handler: None,
                });
            }
        }
        "handler" | "middleware" | "fn" | "entity" | "seeder" | "database" | "cache" | "vector"
        | "queue" => {
            let label = node
                .args
                .first()
                .and_then(SourceValue::as_string_like)
                .unwrap_or_else(|| node.name.clone());
            nodes.push(InspectorSourceNode {
                kind: node.name.clone(),
                label,
                source: source_for_node(node),
            });
        }
        _ => {}
    }
}

fn collect_endpoint_group_sources(
    node: &SourceNode,
    scope: &str,
    routes: &mut Vec<InspectorRouteSource>,
    websockets: &mut Vec<InspectorRouteSource>,
) {
    for child in &node.children {
        match child.name.as_str() {
            "group" => {
                let child_scope = child
                    .prop("path")
                    .and_then(|prop| prop.value.as_string_like())
                    .map(|path| join_inspector_paths(scope, &path))
                    .unwrap_or_else(|| scope.to_string());
                collect_endpoint_group_sources(child, &child_scope, routes, websockets);
            }
            "get" | "post" | "put" | "patch" | "delete" => {
                if let Some(path) = child
                    .prop("path")
                    .and_then(|prop| prop.value.as_string_like())
                {
                    routes.push(InspectorRouteSource {
                        method: child.name.to_ascii_uppercase(),
                        path: join_inspector_paths(scope, &path),
                        source: source_for_node(child),
                        handler: child
                            .prop("handler")
                            .and_then(|prop| prop.value.as_string_like()),
                    });
                }
            }
            "websocket" => {
                if let Some(path) = child.args.first().and_then(SourceValue::as_string_like) {
                    websockets.push(InspectorRouteSource {
                        method: "WS".to_string(),
                        path: join_inspector_paths(scope, &path),
                        source: source_for_node(child),
                        handler: None,
                    });
                }
            }
            _ => {}
        }
    }
}

fn join_inspector_paths(parent: &str, child: &str) -> String {
    match (parent, child) {
        ("", value) => value.to_string(),
        (parent, "") => parent.to_string(),
        (parent, child) => format!(
            "{}/{}",
            parent.trim_end_matches('/'),
            child.trim_start_matches('/')
        ),
    }
}

#[derive(Clone)]
struct InspectorSourceNode {
    kind: String,
    label: String,
    source: ServerInspectorSource,
}

fn collect_route_source(node: &SourceNode, routes: &mut Vec<InspectorRouteSource>) {
    let Some(path) = node.args.first().and_then(SourceValue::as_string_like) else {
        return;
    };
    for child in &node.children {
        let method = match child.name.as_str() {
            "response" | "handler" => "GET".to_string(),
            "method" => child
                .args
                .first()
                .and_then(SourceValue::as_string_like)
                .unwrap_or_else(|| "GET".to_string()),
            _ => continue,
        };
        routes.push(InspectorRouteSource {
            method,
            path: path.clone(),
            source: source_for_node(child),
            handler: child
                .prop("handler")
                .and_then(|prop| prop.value.as_string_like()),
        });
    }
}

fn source_for_node(node: &SourceNode) -> ServerInspectorSource {
    ServerInspectorSource {
        path: node
            .location
            .relative_path
            .to_string_lossy()
            .replace('\\', "/"),
        line: node.location.line,
        end_line: source_end_line(node),
    }
}

fn source_end_line(node: &SourceNode) -> usize {
    node.children
        .iter()
        .map(source_end_line)
        .chain(node.props.iter().map(|prop| prop.location.line))
        .fold(node.location.line, usize::max)
}

fn source_index_key(source: &InspectorRouteSource) -> String {
    format!("{}:{}:{}", source.method, source.path, source.source.line)
}

fn source_node_id(source: &InspectorSourceNode) -> String {
    inspector_id(
        &source.kind,
        &format!(
            "{}:{}:{}",
            source.label, source.source.path, source.source.line
        ),
    )
}

fn behavior_label(behavior: &EndpointBehavior) -> String {
    format!("{behavior:?}")
        .split(['(', '{'])
        .next()
        .unwrap_or("action")
        .to_string()
}

fn inspector_id(kind: &str, label: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!("{kind}:{label}").as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("si_{hash:016x}")
}

fn collect_middleware_resources(
    middleware: &ServerMiddleware,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    for statement in &middleware.action.statements {
        if let ServerMiddlewareStatement::SessionVerify {
            cache, database, ..
        } = statement
        {
            add_resource(
                resources,
                "cache",
                &cache.binding,
                &format!("{:?}", cache.provider),
                "verify",
            );
            add_resource(
                resources,
                "database",
                &database.binding,
                &format!("{:?}", database.provider),
                "verify",
            );
        }
    }
}

fn collect_action_resources(
    action: &ServerAction,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    for statement in &action.statements {
        match statement {
            ServerStatement::Store(statement) => match statement {
                ServerStoreStatement::Handle { connection } => add_resource(
                    resources,
                    "database",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerStoreStatement::Insert { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("insert:{table}"),
                ),
                ServerStoreStatement::List { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("list:{table}"),
                ),
                ServerStoreStatement::Read { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("read:{table}"),
                ),
                ServerStoreStatement::Update { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("update:{table}"),
                ),
                ServerStoreStatement::Delete { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("delete:{table}"),
                ),
                ServerStoreStatement::Query { binding, .. } => {
                    add_resource(resources, "database", binding, "configured", "query")
                }
                ServerStoreStatement::Transaction { binding, .. } => {
                    add_resource(resources, "database", binding, "configured", "transaction")
                }
            },
            ServerStatement::Kv(statement) => match statement {
                ServerKvStatement::Handle { connection } => add_resource(
                    resources,
                    "cache",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerKvStatement::Get { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "get")
                }
                ServerKvStatement::Set { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "set")
                }
                ServerKvStatement::Delete { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "delete")
                }
                ServerKvStatement::Keys { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "keys")
                }
                ServerKvStatement::Clear { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "clear")
                }
            },
            ServerStatement::Vector(statement) => match statement {
                ServerVectorStatement::Handle { connection } => add_resource(
                    resources,
                    "vector",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerVectorStatement::Upsert { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "upsert")
                }
                ServerVectorStatement::Search { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "search")
                }
                ServerVectorStatement::Read { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "read")
                }
                ServerVectorStatement::Delete { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "delete")
                }
                ServerVectorStatement::List { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "list")
                }
            },
            ServerStatement::Queue(statement) => match statement {
                ServerQueueStatement::Handle { connection } => add_resource(
                    resources,
                    "queue",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerQueueStatement::Publish { binding, .. } => {
                    add_resource(resources, "queue", binding, "configured", "publish")
                }
            },
            ServerStatement::Call(call) => collect_function_resources(&call.action, resources),
            ServerStatement::Task(job) | ServerStatement::Cron(job) => {
                collect_function_resources(&job.action, resources)
            }
            _ => {}
        }
    }
}

fn collect_function_resources(
    action: &ServerFunctionAction,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    collect_action_resources(
        &ServerAction {
            statements: action.statements.clone(),
        },
        resources,
    );
}

fn add_resource(
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
    kind: &str,
    binding: &str,
    provider: &str,
    operation: &str,
) {
    let key = format!("{kind}:{binding}");
    let entry = resources
        .entry(key)
        .or_insert_with(|| InspectorResourceAccumulator {
            kind: kind.to_string(),
            binding: binding.to_string(),
            provider: provider.to_string(),
            operations: HashSet::new(),
        });
    if entry.provider == "configured" && provider != "configured" {
        entry.provider = provider.to_string();
    }
    entry.operations.insert(operation.to_string());
}

fn collect_jobs_from_action(
    action: &ServerAction,
    root: &Path,
    jobs: &mut Vec<ServerInspectorJob>,
) {
    for statement in &action.statements {
        match statement {
            ServerStatement::Task(job) | ServerStatement::Cron(job) => {
                let path = job
                    .source_path
                    .strip_prefix(root)
                    .unwrap_or(&job.source_path)
                    .to_string_lossy()
                    .replace('\\', "/");
                jobs.push(ServerInspectorJob {
                    id: inspector_id("job", &job.id),
                    kind: if matches!(statement, ServerStatement::Cron(_)) {
                        "cron".to_string()
                    } else {
                        "task".to_string()
                    },
                    target: job.target.clone(),
                    schedule: job.schedule.clone(),
                    source: Some(ServerInspectorSource {
                        path,
                        line: job.source_line,
                        end_line: job.source_line,
                    }),
                });
            }
            ServerStatement::Call(call) => collect_jobs_from_function(&call.action, root, jobs),
            _ => {}
        }
    }
}

fn collect_jobs_from_function(
    action: &ServerFunctionAction,
    root: &Path,
    jobs: &mut Vec<ServerInspectorJob>,
) {
    collect_jobs_from_action(
        &ServerAction {
            statements: action.statements.clone(),
        },
        root,
        jobs,
    );
}

fn action_uses_binding(action: &ServerAction, binding: &str) -> bool {
    action.statements.iter().any(|statement| match statement {
        ServerStatement::Store(statement) => match statement {
            ServerStoreStatement::Handle { connection } => connection.binding == binding,
            ServerStoreStatement::Insert { binding: value, .. }
            | ServerStoreStatement::List { binding: value, .. }
            | ServerStoreStatement::Read { binding: value, .. }
            | ServerStoreStatement::Update { binding: value, .. }
            | ServerStoreStatement::Delete { binding: value, .. }
            | ServerStoreStatement::Query { binding: value, .. }
            | ServerStoreStatement::Transaction { binding: value, .. } => value == binding,
        },
        ServerStatement::Kv(statement) => match statement {
            ServerKvStatement::Handle { connection } => connection.binding == binding,
            ServerKvStatement::Get { binding: value, .. }
            | ServerKvStatement::Set { binding: value, .. }
            | ServerKvStatement::Delete { binding: value, .. }
            | ServerKvStatement::Keys { binding: value, .. }
            | ServerKvStatement::Clear { binding: value, .. } => value == binding,
        },
        ServerStatement::Vector(statement) => match statement {
            ServerVectorStatement::Handle { connection } => connection.binding == binding,
            ServerVectorStatement::Upsert { binding: value, .. }
            | ServerVectorStatement::Search { binding: value, .. }
            | ServerVectorStatement::Read { binding: value, .. }
            | ServerVectorStatement::Delete { binding: value, .. }
            | ServerVectorStatement::List { binding: value, .. } => value == binding,
        },
        ServerStatement::Queue(statement) => match statement {
            ServerQueueStatement::Handle { connection } => connection.binding == binding,
            ServerQueueStatement::Publish { binding: value, .. } => value == binding,
        },
        ServerStatement::Call(call) => action_uses_binding(
            &ServerAction {
                statements: call.action.statements.clone(),
            },
            binding,
        ),
        ServerStatement::Task(job) | ServerStatement::Cron(job) => action_uses_binding(
            &ServerAction {
                statements: job.action.statements.clone(),
            },
            binding,
        ),
        _ => false,
    })
}
