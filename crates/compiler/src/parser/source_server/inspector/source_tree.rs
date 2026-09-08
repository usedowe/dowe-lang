use super::{parse_source_file, ServerInspectorSource, SourceNode, SourceValue};
use std::{collections::HashSet, fs, path::Path};

#[derive(Clone)]
pub(super) struct InspectorRouteSource {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) source: ServerInspectorSource,
    pub(super) handler: Option<String>,
}
pub(super) fn collect_inspector_source_nodes(
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

pub(super) fn collect_inspector_source_tree(
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
pub(super) struct InspectorSourceNode {
    pub(super) kind: String,
    pub(super) label: String,
    pub(super) source: ServerInspectorSource,
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

pub(super) fn source_index_key(source: &InspectorRouteSource) -> String {
    format!("{}:{}:{}", source.method, source.path, source.source.line)
}

pub(super) fn source_node_id(source: &InspectorSourceNode) -> String {
    super::inspector_id(
        &source.kind,
        &format!(
            "{}:{}:{}",
            source.label, source.source.path, source.source.line
        ),
    )
}
