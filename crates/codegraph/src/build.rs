use crate::error::{CodeGraphError, CodeGraphResult};
use crate::metrics::{fingerprint_bytes, metrics_for};
use crate::mode::detect_codegraph_mode;
use crate::model::{BuildOptions, CodeGraph, Edge, EdgeKind, Node, NodeExplanation, NodeKind};
use crate::paths::{discover_files, slash_path};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn build_codegraph(root: &Path, options: BuildOptions) -> CodeGraphResult<CodeGraph> {
    let mode = options
        .mode
        .map(Ok)
        .unwrap_or_else(|| detect_codegraph_mode(root))?;
    let root = root
        .canonicalize()
        .map_err(|error| CodeGraphError::at_path(root, error.to_string()))?;
    let mut graph = CodeGraph {
        mode,
        root: ".".to_string(),
        nodes: Vec::new(),
        edges: Vec::new(),
    };
    let workspace_id = "workspace:.".to_string();
    graph.nodes.push(Node {
        id: workspace_id.clone(),
        kind: NodeKind::Workspace,
        path: Some(".".to_string()),
        name: "workspace".to_string(),
        owner: None,
        fingerprint: fingerprint_bytes(root.to_string_lossy().as_bytes()),
        metrics: None,
        source_range: None,
    });

    let files = discover_files(&root, mode)?;
    let crates = crate_nodes(&root, mode)?;
    let mut crate_ids = BTreeMap::new();
    let mut module_ids = BTreeSet::new();

    for (name, path) in crates {
        let relative = slash_path(&path);
        let id = format!("crate:{name}");
        crate_ids.insert(relative.clone(), id.clone());
        graph.nodes.push(Node {
            id: id.clone(),
            kind: NodeKind::Crate,
            path: Some(relative),
            name,
            owner: None,
            fingerprint: String::new(),
            metrics: None,
            source_range: None,
        });
        graph.edges.push(Edge {
            from: workspace_id.clone(),
            to: id,
            kind: EdgeKind::Contains,
        });
    }

    add_ownership_nodes(&mut graph, &workspace_id);

    for file in files {
        add_file(
            &root,
            &workspace_id,
            &crate_ids,
            &mut module_ids,
            &mut graph,
            &file,
        )?;
    }

    add_spec_edges(&mut graph);
    graph.nodes.sort_by(|a, b| a.id.cmp(&b.id));
    graph.edges.sort_by(|a, b| {
        (&a.from, &a.to, format!("{:?}", a.kind)).cmp(&(&b.from, &b.to, format!("{:?}", b.kind)))
    });
    Ok(graph)
}

pub(crate) fn explain_node(
    root: &Path,
    selector: &str,
    options: BuildOptions,
) -> CodeGraphResult<NodeExplanation> {
    let graph = build_codegraph(root, options)?;
    let node = graph
        .nodes
        .iter()
        .find(|node| {
            node.id == selector || node.name == selector || node.path.as_deref() == Some(selector)
        })
        .cloned()
        .ok_or_else(|| CodeGraphError::new(format!("CodeGraph node `{selector}` was not found")))?;
    let incoming = graph
        .edges
        .iter()
        .filter(|edge| edge.to == node.id)
        .cloned()
        .collect();
    let outgoing = graph
        .edges
        .iter()
        .filter(|edge| edge.from == node.id)
        .cloned()
        .collect();

    Ok(NodeExplanation {
        node,
        incoming,
        outgoing,
    })
}

fn add_file(
    root: &Path,
    _workspace_id: &str,
    crate_ids: &BTreeMap<String, String>,
    module_ids: &mut BTreeSet<String>,
    graph: &mut CodeGraph,
    file: &Path,
) -> CodeGraphResult<()> {
    let relative_path = file
        .strip_prefix(root)
        .map_err(|_| CodeGraphError::new("file must stay under CodeGraph root"))?;
    let relative = slash_path(relative_path);
    let bytes = fs::read(file).map_err(|error| CodeGraphError::at_path(file, error.to_string()))?;
    let content = String::from_utf8_lossy(&bytes);
    let kind = node_kind_for(&relative);
    let id = format!("{}:{relative}", node_kind_prefix(kind));
    let owner = owner_for(&relative);
    let metrics = if is_code_file(&relative) {
        Some(metrics_for(&content))
    } else {
        None
    };

    graph.nodes.push(Node {
        id: id.clone(),
        kind,
        path: Some(relative.clone()),
        name: file_name(&relative),
        owner,
        fingerprint: fingerprint_bytes(&bytes),
        metrics,
        source_range: None,
    });

    let parent = containing_node(root, crate_ids, module_ids, graph, file, &relative)?;
    graph.edges.push(Edge {
        from: parent,
        to: id.clone(),
        kind: EdgeKind::Contains,
    });

    if relative.ends_with(".rs") {
        add_symbols(graph, &id, &relative, &content);
    }

    Ok(())
}

fn containing_node(
    root: &Path,
    crate_ids: &BTreeMap<String, String>,
    module_ids: &mut BTreeSet<String>,
    graph: &mut CodeGraph,
    file: &Path,
    relative: &str,
) -> CodeGraphResult<String> {
    let mut crate_parent = None;
    for (crate_path, crate_id) in crate_ids {
        let prefix = crate_path
            .trim_end_matches("Cargo.toml")
            .trim_end_matches('/');
        if !prefix.is_empty() && relative.starts_with(prefix) {
            crate_parent = Some((prefix.to_string(), crate_id.clone()));
            break;
        }
    }

    let Some((crate_prefix, crate_id)) = crate_parent else {
        return Ok("workspace:.".to_string());
    };

    if !relative.ends_with(".rs") {
        return Ok(crate_id);
    }

    let parent = file.parent().unwrap_or(root);
    let module_path = slash_path(parent.strip_prefix(root).unwrap_or(parent));
    let module_id = format!("module:{module_path}");
    if module_ids.insert(module_id.clone()) {
        graph.nodes.push(Node {
            id: module_id.clone(),
            kind: NodeKind::Module,
            path: Some(module_path.clone()),
            name: module_path
                .trim_start_matches(&crate_prefix)
                .trim_matches('/')
                .replace('/', "::"),
            owner: Some(crate_id.trim_start_matches("crate:").to_string()),
            fingerprint: fingerprint_bytes(module_path.as_bytes()),
            metrics: None,
            source_range: None,
        });
        graph.edges.push(Edge {
            from: crate_id,
            to: module_id.clone(),
            kind: EdgeKind::Contains,
        });
    }

    Ok(module_id)
}

fn add_symbols(graph: &mut CodeGraph, file_id: &str, relative: &str, content: &str) {
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("pub const ")
            || trimmed.starts_with("pub type "))
        {
            continue;
        }

        let name = trimmed
            .split_whitespace()
            .nth(2)
            .unwrap_or("symbol")
            .trim_end_matches('{')
            .trim_end_matches(';')
            .split('(')
            .next()
            .unwrap_or("symbol")
            .to_string();
        let id = format!("symbol:{relative}:{}:{name}", index + 1);
        graph.nodes.push(Node {
            id: id.clone(),
            kind: NodeKind::Symbol,
            path: Some(relative.to_string()),
            name,
            owner: owner_for(relative),
            fingerprint: fingerprint_bytes(line.as_bytes()),
            metrics: None,
            source_range: Some(crate::model::SourceRange {
                start_line: index + 1,
                end_line: index + 1,
            }),
        });
        graph.edges.push(Edge {
            from: file_id.to_string(),
            to: id,
            kind: EdgeKind::Contains,
        });
    }
}

fn add_ownership_nodes(graph: &mut CodeGraph, workspace_id: &str) {
    for (name, owner) in crate::check::ownership_areas() {
        let id = format!("ownership:{owner}:{name}");
        graph.nodes.push(Node {
            id: id.clone(),
            kind: NodeKind::OwnershipArea,
            path: Some(owner.to_string()),
            name: name.to_string(),
            owner: Some(owner.to_string()),
            fingerprint: fingerprint_bytes(format!("{owner}:{name}").as_bytes()),
            metrics: None,
            source_range: None,
        });
        graph.edges.push(Edge {
            from: workspace_id.to_string(),
            to: id,
            kind: EdgeKind::Owns,
        });
    }
}

fn add_spec_edges(graph: &mut CodeGraph) {
    let specs = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Spec)
        .filter_map(|node| {
            node.path
                .as_ref()
                .map(|path| (node.id.clone(), spec_dir(path)))
        })
        .collect::<Vec<_>>();
    let contract_nodes = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::Contract | NodeKind::Acceptance))
        .filter_map(|node| {
            node.path
                .as_ref()
                .map(|path| (node.id.clone(), path.clone()))
        })
        .collect::<Vec<_>>();

    for (spec_id, dir) in specs {
        for (node_id, path) in &contract_nodes {
            if path.starts_with(&dir) {
                graph.edges.push(Edge {
                    from: spec_id.clone(),
                    to: node_id.clone(),
                    kind: if path.ends_with("acceptance.md") {
                        EdgeKind::Validates
                    } else {
                        EdgeKind::DeclaresContract
                    },
                });
            }
        }
    }
}

fn crate_nodes(
    root: &Path,
    mode: crate::model::CodeGraphMode,
) -> CodeGraphResult<Vec<(String, PathBuf)>> {
    let crate_roots = match mode {
        crate::model::CodeGraphMode::Dowe => vec![
            PathBuf::from("crates"),
            PathBuf::from("dowe-lang/crates"),
            PathBuf::from("dowe-lsp/crates"),
        ],
        crate::model::CodeGraphMode::Project => vec![PathBuf::from("crates")],
    };
    let mut crates = Vec::new();

    for crate_root in crate_roots {
        let crates_root = root.join(&crate_root);
        if !crates_root.exists() {
            continue;
        }

        for entry in fs::read_dir(&crates_root)
            .map_err(|error| CodeGraphError::at_path(&crates_root, error.to_string()))?
        {
            let entry = entry.map_err(CodeGraphError::from)?;
            let cargo = entry.path().join("Cargo.toml");
            if cargo.exists() {
                let name = package_name(&cargo)?.unwrap_or_else(|| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .replace('-', "_")
                        .to_string()
                });
                let relative = cargo
                    .strip_prefix(root)
                    .map_err(|_| CodeGraphError::new("crate path must stay under root"))?
                    .to_path_buf();
                crates.push((name, relative));
            }
        }
    }
    crates.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(crates)
}

fn package_name(path: &Path) -> CodeGraphResult<Option<String>> {
    let content = fs::read_to_string(path)
        .map_err(|error| CodeGraphError::at_path(path, error.to_string()))?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") && trimmed.contains('=') {
            return Ok(trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"').to_string()));
        }
    }
    Ok(None)
}

fn node_kind_for(relative: &str) -> NodeKind {
    if relative.starts_with(".dowe/") {
        NodeKind::GeneratedArtifact
    } else if relative.starts_with(".agents/") || relative.starts_with("agents/") {
        NodeKind::AgentHarness
    } else if relative.ends_with("/spec.md") && relative.contains("specs/features/") {
        NodeKind::Spec
    } else if relative.ends_with("/acceptance.md") && relative.contains("specs/features/") {
        NodeKind::Acceptance
    } else if relative.contains("specs/features/") && relative.ends_with(".md") {
        NodeKind::Contract
    } else if relative.starts_with("docs/") {
        NodeKind::Doc
    } else {
        NodeKind::File
    }
}

fn node_kind_prefix(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Spec => "spec",
        NodeKind::Contract => "contract",
        NodeKind::Acceptance => "acceptance",
        NodeKind::Doc => "doc",
        NodeKind::AgentHarness => "agent_harness",
        NodeKind::GeneratedArtifact => "generated",
        _ => "file",
    }
}

fn is_code_file(relative: &str) -> bool {
    relative.ends_with(".rs") || relative.ends_with(".dowe")
}

fn file_name(relative: &str) -> String {
    Path::new(relative)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| relative.to_string())
}

fn spec_dir(path: &str) -> String {
    Path::new(path).parent().map(slash_path).unwrap_or_default()
}

fn owner_for(relative: &str) -> Option<String> {
    if relative.starts_with("crates/") {
        let mut parts = relative.split('/');
        parts.next();
        return parts.next().map(|name| format!("crates/{name}"));
    }
    if relative.starts_with("dowe-lang/crates/") {
        let mut parts = relative.split('/');
        parts.next();
        parts.next();
        return parts.next().map(|name| format!("dowe-lang/crates/{name}"));
    }
    if relative.starts_with("dowe-lsp/crates/") {
        let mut parts = relative.split('/');
        parts.next();
        parts.next();
        return parts.next().map(|name| format!("dowe-lsp/crates/{name}"));
    }
    if relative.starts_with("docs/") {
        Some("docs".to_string())
    } else if relative.starts_with("specs/") {
        Some("specs".to_string())
    } else if relative.starts_with(".agents/") || relative.starts_with("agents/") {
        Some("agents".to_string())
    } else {
        None
    }
}
