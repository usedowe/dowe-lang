#[path = "inspector/metadata.rs"]
mod metadata;
#[path = "inspector/resources.rs"]
mod resources;
#[path = "inspector/source_tree.rs"]
mod source_tree;

use resources::{
    action_uses_binding, collect_action_resources, collect_jobs_from_action,
    collect_middleware_resources, InspectorResourceAccumulator,
};
use source_tree::{
    collect_inspector_source_nodes, collect_inspector_source_tree, source_index_key, source_node_id,
};

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
            parameters: metadata::parameters(&endpoint.path, &endpoint.action),
            headers: inspector_headers(&endpoint.action, &endpoint.middlewares),
            body: metadata::body(&endpoint.action),
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
                    field_type: metadata::field_type(field.field_type).to_string(),
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
