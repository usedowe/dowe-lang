use crate::error::{DoweError, DoweResult};
use crate::model::{
    CorsConfig, DoweType, DoweTypeField, Endpoint, EndpointBehavior, EnvironmentConfig,
    EnvironmentVisibility, HttpMethod, ServerAction, ServerConfig, ServerLog, ServerLogLevel,
    ServerLogValue, ServerMiddleware, ServerMiddlewareAction, ServerMiddlewareResponseBody,
    ServerMiddlewareStatement, ServerSecret, ServerStatement, StoreLiteral, WebSocketHandlers,
    WebSocketRoute, normalize_http_header_name,
};
use crate::parser::source_ast::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
use crate::parser::source_imports::resolve_import;
use crate::parser::source_kv::{
    infer_kv_statement, kv_action_endpoint_behavior, parse_kv_let, validate_kv_statement_references,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_store::{
    parse_store_let, store_action_endpoint_behavior, store_endpoint_behavior, store_literal,
};
use crate::parser::source_types::{TypeRegistry, type_from_store_literal, validate_reference_path};
use std::collections::HashMap;
use std::fs;
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
    let types = TypeRegistry::parse(&file.path, &file.nodes)?;
    let imports = server_imports(root, file, environment)?;
    parse_server_nodes(&file.path, &file.nodes, &imports, &types, environment)
}

pub(crate) fn validate_server_module_source(
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    parse_server_module(file, environment).map(|_| ())
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
    let server_node =
        child_named(main, "server").ok_or_else(|| node_error(main, "missing main.server"))?;
    let backend = parse_server_config(server_node, imports, types, environment)?;
    let desktop_server = child_named(main, "desktop")
        .and_then(|desktop| child_named(desktop, "server"))
        .map(|node| parse_server_config(node, imports, types, environment))
        .transpose()?;

    Ok(ServerRoot {
        backend,
        desktop_server,
    })
}

#[derive(Debug)]
pub struct ServerRoot {
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
}

#[derive(Debug, Clone)]
struct ServerHandler {
    action: ServerAction,
    behavior: EndpointBehavior,
}

#[derive(Default)]
struct ServerImports {
    handlers: HashMap<String, ServerHandler>,
    middlewares: HashMap<String, ServerMiddleware>,
}

fn server_imports(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerImports> {
    let mut imports = ServerImports::default();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let module_file = parse_source_file(root, &path, source)?;
        let module = parse_server_module(&module_file, environment)?;
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
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerImports> {
    let types = TypeRegistry::parse(&file.path, &file.nodes)?;
    let mut imports = ServerImports::default();
    for node in &file.nodes {
        match node.name.as_str() {
            "handler" => {
                let (name, handler) = parse_handler_node(node, &types, environment)?;
                if imports.handlers.insert(name.clone(), handler).is_some() {
                    return Err(node_error(node, format!("duplicate handler `{name}`")));
                }
            }
            "middleware" => {
                let (name, middleware) = parse_middleware_node(file, node, environment)?;
                if imports
                    .middlewares
                    .insert(name.clone(), middleware)
                    .is_some()
                {
                    return Err(node_error(node, format!("duplicate middleware `{name}`")));
                }
            }
            "type" => {}
            _ => {
                return Err(node_error(
                    node,
                    "server modules only accept `type`, `handler` or `middleware`",
                ));
            }
        }
    }
    Ok(imports)
}

fn parse_handler_node(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<(String, ServerHandler)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "handler must declare a name"))?;
    let action = parse_action(
        node,
        ActionContext::HttpHandler {
            async_handler: has_arg(node, "async"),
            request: handler_request_name(node),
        },
        types,
        environment,
    )?;
    let behavior = exported_handler_behavior(node, &action)?;
    Ok((name, ServerHandler { action, behavior }))
}

fn parse_middleware_node(
    file: &SourceFile,
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<(String, ServerMiddleware)> {
    let relative = file.relative_path.to_string_lossy().replace('\\', "/");
    if !relative.starts_with("src/middlewares/") {
        return Err(node_error(
            node,
            "`middleware` declarations must live under `src/middlewares`",
        ));
    }
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "middleware must declare a name"))?;
    if handler_request_name(node) != Some("req") {
        return Err(node_error(
            node,
            "middleware must declare request binding `req`",
        ));
    }
    let action = parse_middleware_action(node, environment)?;
    Ok((name.clone(), ServerMiddleware { name, action }))
}

fn parse_server_config(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerConfig> {
    let port = required_port(node)?;
    let mut endpoints = Vec::new();
    let mut websockets = Vec::new();
    let mut init_action = ServerAction::empty();

    for child in &node.children {
        match child.name.as_str() {
            "route" => endpoints.extend(parse_route(child, imports, types, environment)?),
            "endpoint" => {
                return Err(node_error(child, "`endpoint` has been renamed to `route`"));
            }
            "websocket" => websockets.push(parse_websocket(child, environment)?),
            "init" => init_action = parse_action(child, ActionContext::Init, types, environment)?,
            _ => return Err(node_error(child, "unsupported server block")),
        }
    }

    Ok(ServerConfig {
        port,
        endpoints,
        websockets,
        init_action,
        cors: CorsConfig::default(),
    })
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
                let action = parse_action(
                    child,
                    ActionContext::HttpHandler {
                        async_handler: has_arg(child, "async"),
                        request: handler_request_name(child),
                    },
                    types,
                    environment,
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
    let async_handler = has_arg(node, "async");
    let request = handler_request_name(node);
    let action = parse_action(
        node,
        ActionContext::HttpHandler {
            async_handler,
            request,
        },
        types,
        environment,
    )?;
    if has_reference_log(&action)
        && let Some(behavior) =
            store_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) = store_endpoint_behavior(&action, return_json_ref(node))? {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        store_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
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

fn parse_websocket(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<WebSocketRoute> {
    let path = required_path_arg(node, "websocket")?;
    let mut handlers = WebSocketHandlers::default();

    for child in &node.children {
        let action = parse_action(
            child,
            ActionContext::WebSocket,
            &TypeRegistry::empty(),
            environment,
        )?;
        match child.name.as_str() {
            "open" => handlers.open = action,
            "message" => handlers.message = action,
            "close" => handlers.close = action,
            "drain" => handlers.drain = action,
            _ => return Err(node_error(child, "unsupported WebSocket handler")),
        }
    }

    Ok(WebSocketRoute { path, handlers })
}

fn parse_action(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerAction> {
    let mut statements = Vec::new();
    let mut returned = false;
    let mut inferred_bindings = HashMap::<String, DoweType>::new();
    let mut inferred_tables = HashMap::<String, DoweType>::new();

    for child in &node.children {
        match child.name.as_str() {
            "let" => {
                if let Some(statement) = parse_request_json_let(child, context, types)? {
                    infer_request_json_statement(&statement, &mut inferred_bindings);
                    statements.push(statement);
                } else if let Some(statement) = parse_store_let(child, Some(environment))? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_let(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else {
                    validate_let(child, context)?;
                }
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
            _ => return Err(node_error(child, "unsupported server action")),
        }
    }

    if matches!(context, ActionContext::HttpHandler { .. }) && !returned {
        return Err(node_error(node, "handler must return a response"));
    }

    Ok(ServerAction { statements })
}

fn parse_middleware_action(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerMiddlewareAction> {
    let statements = parse_middleware_statements(&node.children, environment)?;
    if !middleware_returns(&statements) {
        return Err(node_error(
            node,
            "middleware must return continue or response on every path",
        ));
    }
    Ok(ServerMiddlewareAction { statements })
}

fn parse_middleware_statements(
    nodes: &[SourceNode],
    environment: &EnvironmentConfig,
) -> DoweResult<Vec<ServerMiddlewareStatement>> {
    let mut statements = Vec::new();
    for node in nodes {
        match node.name.as_str() {
            "let" => statements.push(parse_middleware_let(node, environment)?),
            "if" => statements.push(parse_middleware_if(node, environment)?),
            "return" => statements.push(parse_middleware_return(node)?),
            "log" | "info" | "warn" | "error" => {
                statements.push(ServerMiddlewareStatement::Log(parse_log(node)?));
            }
            "continue" => return Err(node_error(node, "`continue` must be returned")),
            _ => return Err(node_error(node, "unsupported middleware action")),
        }
    }
    Ok(statements)
}

fn parse_middleware_let(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerMiddlewareStatement> {
    let (binding, expression) =
        assignment(node).ok_or_else(|| node_error(node, "middleware let must assign a value"))?;
    match expression.as_str() {
        "req.header" => {
            let name = required_header_name_prop(node, "name")?;
            Ok(ServerMiddlewareStatement::Header { binding, name })
        }
        "bearer" => {
            let source = node
                .args
                .get(3)
                .and_then(SourceValue::as_string_like)
                .ok_or_else(|| node_error(node, "`bearer` requires a source value"))?;
            Ok(ServerMiddlewareStatement::Bearer { binding, source })
        }
        "jwt.verify" => {
            let token = node
                .args
                .get(3)
                .and_then(SourceValue::as_string_like)
                .ok_or_else(|| node_error(node, "`jwt.verify` requires a token value"))?;
            let secret = required_secret_prop(node, "secret", environment)?;
            let algorithm = required_algorithm_prop(node, "algorithm", &["HS256"])?;
            Ok(ServerMiddlewareStatement::JwtVerify {
                binding,
                token,
                secret,
                algorithm,
            })
        }
        "jwt.decrypt" => {
            let token = node
                .args
                .get(3)
                .and_then(SourceValue::as_string_like)
                .ok_or_else(|| node_error(node, "`jwt.decrypt` requires a token value"))?;
            let key = required_secret_prop(node, "key", environment)?;
            let algorithm = required_algorithm_prop(node, "algorithm", &["dir"])?;
            let encryption = required_algorithm_prop(node, "encryption", &["A256GCM"])?;
            Ok(ServerMiddlewareStatement::JwtDecrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            })
        }
        "jwt.sign" => {
            let claims = required_store_literal_prop(node, "claims")?;
            let secret = required_secret_prop(node, "secret", environment)?;
            let algorithm = required_algorithm_prop(node, "algorithm", &["HS256"])?;
            Ok(ServerMiddlewareStatement::JwtSign {
                binding,
                claims,
                secret,
                algorithm,
            })
        }
        "jwt.encrypt" => {
            let claims = required_store_literal_prop(node, "claims")?;
            let key = required_secret_prop(node, "key", environment)?;
            let algorithm = required_algorithm_prop(node, "algorithm", &["dir"])?;
            let encryption = required_algorithm_prop(node, "encryption", &["A256GCM"])?;
            Ok(ServerMiddlewareStatement::JwtEncrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            })
        }
        _ => Err(node_error(node, "unsupported middleware let expression")),
    }
}

fn parse_middleware_if(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerMiddlewareStatement> {
    let condition = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "middleware if must declare a condition"))?;
    let Some(binding) = condition.strip_suffix(".valid") else {
        return Err(node_error(
            node,
            "middleware if only supports JWT validation checks",
        ));
    };
    let statements = parse_middleware_statements(&node.children, environment)?;
    Ok(ServerMiddlewareStatement::IfValid {
        binding: binding.to_string(),
        statements,
    })
}

fn parse_middleware_return(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    match node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
    {
        Some("continue") => Ok(ServerMiddlewareStatement::Continue {
            context: node
                .prop("context")
                .map(|prop| store_literal(&prop.value))
                .transpose()?,
        }),
        Some("response") => {
            let status = return_status_from_node(node)?;
            if let Some(prop) = node.prop("text") {
                return Ok(ServerMiddlewareStatement::Response {
                    status,
                    body: ServerMiddlewareResponseBody::Text(required_static_string_prop(prop)?),
                });
            }
            if let Some(prop) = node.prop("json") {
                return Ok(ServerMiddlewareStatement::Response {
                    status,
                    body: ServerMiddlewareResponseBody::Json(store_literal(&prop.value)?),
                });
            }
            Err(node_error(node, "response must declare text or json"))
        }
        _ => Err(node_error(
            node,
            "middleware return must produce continue or response",
        )),
    }
}

fn middleware_returns(statements: &[ServerMiddlewareStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        ServerMiddlewareStatement::Continue { .. } | ServerMiddlewareStatement::Response { .. } => {
            true
        }
        ServerMiddlewareStatement::IfValid { statements, .. } => middleware_returns(statements),
        _ => false,
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
    if let ServerStatement::RequestJson {
        binding,
        schema: Some(schema),
    } = statement
    {
        bindings.insert(binding.clone(), schema.clone());
    }
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
            validate_store_literal_references(node, value, bindings)?;
            for expected in matches {
                validate_store_literal_references(node, &expected.value, bindings)?;
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Read { filter, .. }
        | crate::model::ServerStoreStatement::Delete { filter, .. } => {
            validate_store_literal_references(node, &filter.value, bindings)
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
    WebSocket,
}

fn validate_let(node: &SourceNode, context: ActionContext) -> DoweResult<()> {
    let source = node
        .args
        .iter()
        .map(SourceValue::to_source)
        .collect::<Vec<_>>()
        .join(" ");
    validate_request_usage(node, context, &source)?;
    if source.contains("await") && !context_allows_await(context) {
        return Err(node_error(
            node,
            "`await` is only valid inside async handlers",
        ));
    }
    Ok(())
}

fn parse_request_json_let(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
) -> DoweResult<Option<ServerStatement>> {
    if node.args.len() != 4 {
        return Ok(None);
    }
    let Some(binding) = node.args[0].as_string_like() else {
        return Ok(None);
    };
    let Some(equals) = node.args[1].as_string_like() else {
        return Ok(None);
    };
    let Some(await_token) = node.args[2].as_string_like() else {
        return Ok(None);
    };
    let Some(json_call) = node.args[3].as_string_like() else {
        return Ok(None);
    };
    if equals != "=" || await_token != "await" || json_call != "req.json()" {
        return Ok(None);
    }
    validate_request_usage(node, context, "await req.json()")?;
    let (binding, schema) = parse_binding_type(node, &binding, types)?;
    Ok(Some(ServerStatement::RequestJson { binding, schema }))
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
        != Some("response")
    {
        return Err(node_error(node, "return must produce a response"));
    }
    if node.prop("text").is_none() && node.prop("json").is_none() {
        return Err(node_error(node, "response must declare text or json"));
    }
    if let Some(prop) = node.prop("text") {
        required_static_string_prop(prop)?;
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
    if source.contains("req.json()") {
        match context {
            ActionContext::HttpHandler {
                async_handler: true,
                request: Some("req"),
            } => {}
            ActionContext::HttpHandler { .. } => {
                return Err(node_error(
                    node,
                    "`req.json()` requires an async request handler",
                ));
            }
            ActionContext::Init | ActionContext::WebSocket => {
                return Err(node_error(
                    node,
                    "`req.json()` is only valid in HTTP handlers",
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

fn handler_behavior(
    node: &SourceNode,
    path: &str,
    action: &ServerAction,
) -> DoweResult<EndpointBehavior> {
    if has_reference_log(action)
        && let Some(behavior) =
            store_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = store_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        store_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
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
        && let Some(behavior) =
            store_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = store_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        store_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
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

fn handler_request_name(node: &SourceNode) -> Option<&str> {
    node.args
        .iter()
        .filter_map(SourceValue::as_string_like)
        .find_map(|value| if value == "req" { Some("req") } else { None })
}

fn has_arg(node: &SourceNode, expected: &str) -> bool {
    node.args
        .iter()
        .any(|arg| arg.as_string_like().as_deref() == Some(expected))
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
        EnvironmentVisibility, HttpMethod, ServerLogValue, ServerMiddlewareStatement, ServerSecret,
        ServerStatement,
    };
    use crate::parser::source_parser::parse_source_file;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn parses_main_server_route() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:8080
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");
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
    fn parses_route_middlewares_from_imports() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("src/middlewares")).expect("middlewares");
        fs::write(
            root.join("src/main.dowe"),
            r#"import requireBearer from "./middlewares/auth"

main
  server port:8080
    route "/users/:id" middleware:[requireBearer]
      handler req
        return response text:"Hello {req.context.auth.subject}""#,
        )
        .expect("main");
        fs::write(
            root.join("src/middlewares/auth.dowe"),
            r#"middleware requireBearer async req
  let authorization = req.header name:"Authorization"
  let token = bearer authorization
  let verified = jwt.verify token secret:env.JWT_SECRET algorithm:"HS256"
  if verified.valid
    return continue context:{ auth:{ subject:verified.claims.sub claims:verified.claims } }
  return response status:401 json:{ ok:false error:"Unauthorized" }"#,
        )
        .expect("middleware");
        let source = fs::read_to_string(root.join("src/main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("src/main.dowe"), source).expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "JWT_SECRET".to_string(),
                visibility: EnvironmentVisibility::Server,
                required: true,
                default_value: None,
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
            &endpoint.endpoint.middlewares[0].action.statements[2],
            ServerMiddlewareStatement::JwtVerify {
                secret: ServerSecret::Environment(name),
                algorithm,
                ..
            } if name == "JWT_SECRET" && algorithm == "HS256"
        ));
        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::TextTemplate(_)
        ));
    }

    #[test]
    fn rejects_client_environment_for_remote_store_credentials() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1" host:"http://127.0.0.1:4147" user:"api-user" token:env.STORE_TOKEN
        let users = db.list table:"users"
        return response json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "STORE_TOKEN".to_string(),
                visibility: EnvironmentVisibility::Client,
                required: true,
                default_value: None,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn rejects_legacy_app_root() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"app
  server port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `main`"));
    }

    #[test]
    fn rejects_legacy_backend_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  backend port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `server`"));
    }

    #[test]
    fn rejects_legacy_endpoint_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:8080
    endpoint "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `route`"));
    }

    #[test]
    fn infers_store_insert_fields_for_log_references() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        let db = store database:"app"
        let created = db.insert table:"blogs" value:{ title:"First" }
        log created.title
        return response json:created"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");
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
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        let db = store database:"app"
        let created = db.insert table:"blogs" value:{ title:"First" }
        log created.missing
        return response json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

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
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        let db = store database:"app"
        let created = db.insert table:"blogs" value:{ title:"First" }
        return response json:{ data:created.missing }"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

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
            Path::new("/project/src/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        let body:User = await req.json()
        let db = store database:"app"
        let created = db.insert table:"users" value:{ name:body.name age:body.age }
        return response json:{ ok:true user:created }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");
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
            Path::new("/project/src/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        let body:User = await req.json()
        let db = store database:"app"
        let created = db.insert table:"users" value:{ name:body.email }
        return response json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("unknown field `body.email`"));
    }
}
