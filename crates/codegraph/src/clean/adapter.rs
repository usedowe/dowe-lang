use super::{CleanEdge, CleanGraph, CleanNode, Evidence, Namespace};
use crate::paths::{discover_files, slash_path};
use crate::{
    BuildOptions, CodeGraph, CodeGraphMode, CodeGraphResult, Edge, EdgeKind, KnowledgeNamespace,
    Node, NodeKind, UnifiedCodeGraph,
};
use crate::{CodeGraphError, detect_codegraph_mode, load_knowledge_registry_into_clean};
use dowe_compiler::{LanguageDocument, LanguageDocumentSymbol, document_symbols};
use std::fs;
use std::path::Path;
pub fn build_clean_graph(root: &Path, options: BuildOptions) -> CodeGraphResult<CleanGraph> {
    let root = root
        .canonicalize()
        .map_err(|error| CodeGraphError::at_path(root, error.to_string()))?;
    let mode = options
        .mode
        .map(Ok)
        .unwrap_or_else(|| detect_codegraph_mode(&root))?;
    let mut graph = extract_clean_graph(&root, mode)?;
    load_knowledge_registry_into_clean(&root, &mut graph)?;
    Ok(graph)
}
fn extract_clean_graph(root: &Path, mode: CodeGraphMode) -> CodeGraphResult<CleanGraph> {
    let mut graph = CleanGraph::default();
    graph.insert_node(CleanNode {
        id: "workspace:.".into(),
        namespace: Namespace::Shared,
        kind: "project".into(),
        name: "workspace".into(),
        path: Some(".".into()),
        start_line: None,
        end_line: None,
        evidence: Evidence::Compiler,
    });
    for file in discover_files(root, mode)? {
        let relative = slash_path(
            file.strip_prefix(root)
                .map_err(|_| CodeGraphError::new("CodeGraph file escaped the project root"))?,
        );
        let bytes =
            fs::read(&file).map_err(|error| CodeGraphError::at_path(&file, error.to_string()))?;
        let file_id = format!("file:{relative}");
        graph.insert_node(CleanNode {
            id: file_id.clone(),
            namespace: namespace_for_path(&relative),
            kind: "file".into(),
            name: file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .into(),
            path: Some(relative.clone()),
            start_line: None,
            end_line: None,
            evidence: Evidence::Compiler,
        });
        if namespace_for_path(&relative) == Namespace::Asset {
            let asset_id = format!("dowe:asset:{relative}");
            graph.insert_node(CleanNode {
                id: asset_id.clone(),
                namespace: Namespace::Asset,
                kind: "asset".into(),
                name: file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .into(),
                path: Some(relative.clone()),
                start_line: None,
                end_line: None,
                evidence: Evidence::Compiler,
            });
            graph
                .insert_edge(CleanEdge {
                    source: file_id.clone(),
                    relation: "describes_asset".into(),
                    target: asset_id,
                    evidence: Evidence::Compiler,
                })
                .map_err(CodeGraphError::new)?;
        }
        graph
            .insert_edge(CleanEdge {
                source: "workspace:.".into(),
                relation: "contains".into(),
                target: file_id.clone(),
                evidence: Evidence::Compiler,
            })
            .map_err(CodeGraphError::new)?;
        if namespace_for_path(&relative) == Namespace::I18n
            && relative.ends_with(".json")
            && bytes.len() <= 1024 * 1024
        {
            super::content::add_i18n_keys(&mut graph, &file_id, &relative, &bytes)?;
        }
        if namespace_for_path(&relative) == Namespace::Product
            && relative.ends_with(".md")
            && bytes.len() <= 1024 * 1024
        {
            super::content::add_product_sections(&mut graph, &file_id, &relative, &bytes)?;
        }
        if relative.ends_with(".dowe") && bytes.len() <= 1024 * 1024 {
            let source = String::from_utf8_lossy(&bytes).into_owned();
            let document = LanguageDocument {
                path: file.clone(),
                source,
            };
            for symbol in document_symbols(root, &document) {
                add_symbol(&mut graph, &symbol, &relative, &file_id)?;
            }
        }
    }
    add_import_edges(root, &mut graph)?;
    super::dowe_relations::add_dowe_symbol_usage_edges(root, &mut graph)?;
    super::dowe_relations::add_dowe_reference_edges(root, &mut graph)?;
    add_backend_edges(root, &mut graph)?;
    link_translation_variants(&mut graph)?;
    Ok(graph)
}

fn link_translation_variants(graph: &mut CleanGraph) -> CodeGraphResult<()> {
    let keys = graph
        .nodes()
        .filter(|node| node.kind == "translation_key")
        .cloned()
        .collect::<Vec<_>>();
    for (index, left) in keys.iter().enumerate() {
        for right in keys.iter().skip(index + 1) {
            if left.name == right.name && left.path != right.path {
                for (source, target) in [(&left.id, &right.id), (&right.id, &left.id)] {
                    graph
                        .insert_edge(CleanEdge {
                            source: source.clone(),
                            relation: "translation_variant".into(),
                            target: target.clone(),
                            evidence: Evidence::Compiler,
                        })
                        .map_err(CodeGraphError::new)?;
                }
            }
        }
    }
    Ok(())
}

fn add_import_edges(root: &Path, graph: &mut CleanGraph) -> CodeGraphResult<()> {
    let files = graph
        .nodes()
        .filter(|node| node.kind == "file" && node.path.is_some())
        .filter_map(|node| node.path.clone())
        .collect::<Vec<_>>();
    for source in files.iter().filter(|path| path.ends_with(".dowe")) {
        let source_path = root.join(source);
        let bytes = fs::read(&source_path)
            .map_err(|error| CodeGraphError::at_path(&source_path, error.to_string()))?;
        let content = String::from_utf8_lossy(&bytes);
        for line in content
            .lines()
            .filter(|line| line.trim_start().starts_with("import "))
        {
            let Some((_, quoted)) = line.split_once('"') else {
                continue;
            };
            let Some((target, _)) = quoted.split_once('"') else {
                continue;
            };
            let Some(target_path) = resolve_import(source, target, &files) else {
                continue;
            };
            let source_id = format!("file:{source}");
            let target_id = format!("file:{target_path}");
            let _ = graph.insert_edge(CleanEdge {
                source: source_id,
                relation: "imports".into(),
                target: target_id,
                evidence: Evidence::Inferred,
            });
        }
    }
    Ok(())
}
fn resolve_import(source: &str, target: &str, files: &[String]) -> Option<String> {
    let normalized = target
        .strip_prefix("@/")
        .or_else(|| target.strip_prefix("./"))
        .unwrap_or(target);
    let candidates = [normalized.to_string(), format!("{normalized}.dowe")];
    if target.starts_with("./") {
        let parent = source
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        let relative = format!("{parent}/{normalized}");
        return [relative.clone(), format!("{relative}.dowe")]
            .into_iter()
            .find(|candidate| files.iter().any(|file| file == candidate));
    }
    candidates
        .into_iter()
        .find(|candidate| files.iter().any(|file| file == candidate))
}
fn add_backend_edges(root: &Path, graph: &mut CleanGraph) -> CodeGraphResult<()> {
    let declarations = graph
        .nodes()
        .filter(|node| matches!(node.kind.as_str(), "route" | "handler" | "entity"))
        .cloned()
        .collect::<Vec<_>>();
    for route in declarations.iter().filter(|node| node.kind == "route") {
        let Some(path) = route.path.as_deref() else {
            continue;
        };
        let source = fs::read_to_string(root.join(path))?;
        let line = route
            .start_line
            .and_then(|line| source.lines().nth(line.saturating_sub(1) as usize))
            .unwrap_or_default();
        for handler in declarations.iter().filter(|node| {
            node.kind == "handler"
                && node.path.as_deref() == Some(path)
                && line
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|part| part == node.name)
        }) {
            let _ = graph.insert_edge(CleanEdge {
                source: route.id.clone(),
                relation: "handled_by".into(),
                target: handler.id.clone(),
                evidence: Evidence::Inferred,
            });
        }
    }
    for handler in declarations.iter().filter(|node| node.kind == "handler") {
        let Some(path) = handler.path.as_deref() else {
            continue;
        };
        let source = fs::read_to_string(root.join(path))?;
        let line = handler
            .start_line
            .and_then(|line| source.lines().nth(line.saturating_sub(1) as usize))
            .unwrap_or_default();
        for entity in declarations.iter().filter(|node| {
            node.kind == "entity"
                && node.path.as_deref() == Some(path)
                && line
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|part| part == node.name)
        }) {
            let _ = graph.insert_edge(CleanEdge {
                source: handler.id.clone(),
                relation: "reads".into(),
                target: entity.id.clone(),
                evidence: Evidence::Inferred,
            });
        }
    }
    Ok(())
}
fn add_symbol(
    graph: &mut CleanGraph,
    symbol: &LanguageDocumentSymbol,
    path: &str,
    parent: &str,
) -> CodeGraphResult<()> {
    let (keyword, name) = symbol
        .name
        .split_once(' ')
        .unwrap_or((&symbol.name, &symbol.name));
    let id = format!("dowe:{keyword}:{path}:{name}");
    let namespace = namespace_for_symbol(keyword, path);
    graph.insert_node(CleanNode {
        id: id.clone(),
        namespace,
        kind: keyword.into(),
        name: name.into(),
        path: Some(path.into()),
        start_line: Some(symbol.range.start.line as u32),
        end_line: Some(symbol.range.end.line as u32),
        evidence: Evidence::Compiler,
    });
    graph
        .insert_edge(CleanEdge {
            source: parent.into(),
            relation: "contains".into(),
            target: id.clone(),
            evidence: Evidence::Compiler,
        })
        .map_err(CodeGraphError::new)?;
    for child in &symbol.children {
        add_symbol(graph, child, path, &id)?;
    }
    Ok(())
}
fn namespace_for_path(path: &str) -> Namespace {
    if path.starts_with("public/")
        || path.starts_with("assets/")
        || path.contains("/assets/")
        || matches!(
            path.rsplit_once('.').map(|(_, ext)| ext),
            Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "avif" | "bmp" | "ico" | "svg")
        )
    {
        Namespace::Asset
    } else if path.starts_with("i18n/")
        || path.starts_with("locales/")
        || path.contains("/translations/")
    {
        Namespace::I18n
    } else if path == "theme.dowe" || path.starts_with("design/") || path.contains("/tokens/") {
        Namespace::Design
    } else if path.starts_with("server/")
        || path.contains("/handlers/")
        || path.contains("/routes/")
    {
        Namespace::Backend
    } else if path.starts_with("specs/") || path.starts_with("docs/") {
        Namespace::Product
    } else if path.starts_with("contracts/") {
        Namespace::Contract
    } else if path.starts_with("views/") || path.ends_with(".dowe") {
        Namespace::Frontend
    } else if path.contains("test") {
        Namespace::Verification
    } else {
        Namespace::Shared
    }
}
fn namespace_for_symbol(keyword: &str, path: &str) -> Namespace {
    match keyword {
        "entity" | "handler" | "middleware" | "route" if !path.starts_with("views/") => {
            Namespace::Backend
        }
        "schema" | "type" => Namespace::Shared,
        "test" => Namespace::Verification,
        "theme" | "design" => Namespace::Design,
        _ => Namespace::Frontend,
    }
}
pub fn clean_graph_from_unified(unified: &UnifiedCodeGraph) -> CleanGraph {
    let mut graph = CleanGraph::default();
    for node in &unified.nodes {
        graph.insert_node(CleanNode {
            id: node.id.clone(),
            namespace: namespace(node.namespace),
            kind: format!("{:?}", node.kind).to_ascii_lowercase(),
            name: node.name.clone(),
            path: node.path.clone(),
            start_line: None,
            end_line: None,
            evidence: evidence(node.evidence),
        });
    }
    for edge in &unified.edges {
        if graph.node(&edge.from).is_some() && graph.node(&edge.to).is_some() {
            let _ = graph.insert_edge(CleanEdge {
                source: edge.from.clone(),
                relation: format!("{:?}", edge.relation).to_ascii_lowercase(),
                target: edge.to.clone(),
                evidence: evidence(edge.evidence),
            });
        }
    }
    graph
}
pub fn legacy_codegraph_from_clean(
    graph: &CleanGraph,
    mode: CodeGraphMode,
    root: &Path,
) -> CodeGraph {
    let nodes = graph
        .nodes()
        .map(|node| Node {
            id: node.id.clone(),
            kind: legacy_node_kind(&node.kind),
            path: node.path.clone(),
            name: node.name.clone(),
            language: node
                .path
                .as_deref()
                .map(|path| crate::paths::language_for(path))
                .unwrap_or_else(|| "unknown".into()),
            owner: None,
            fingerprint: node
                .path
                .as_deref()
                .and_then(|path| fs::read(root.join(path)).ok())
                .map(|bytes| crate::metrics::fingerprint_bytes(&bytes))
                .unwrap_or_default(),
            metrics: None,
            source_range: node
                .start_line
                .zip(node.end_line)
                .map(|(start_line, end_line)| crate::SourceRange {
                    start_line: start_line as usize,
                    end_line: end_line as usize,
                }),
        })
        .collect();
    let edges = graph
        .edges()
        .filter_map(|edge| {
            Some(Edge {
                from: edge.source.clone(),
                to: edge.target.clone(),
                kind: legacy_edge_kind(&edge.relation)?,
            })
        })
        .collect();
    CodeGraph {
        mode,
        root: root.to_string_lossy().into_owned(),
        nodes,
        edges,
    }
}
fn legacy_node_kind(kind: &str) -> NodeKind {
    match kind {
        "project" => NodeKind::Workspace,
        "file" => NodeKind::File,
        "layout" | "page" | "component" | "entity" | "handler" | "route" | "middleware" => {
            NodeKind::Symbol
        }
        "contract" => NodeKind::Contract,
        "test" => NodeKind::Test,
        "documentation" | "requirement" => NodeKind::Doc,
        _ => NodeKind::Symbol,
    }
}
fn legacy_edge_kind(relation: &str) -> Option<EdgeKind> {
    Some(match relation {
        "contains" => EdgeKind::Contains,
        "depends_on" | "uses" | "calls" => EdgeKind::DependsOn,
        "implements" => EdgeKind::Implements,
        "verified_by" | "validates" => EdgeKind::Validates,
        "specified_by" => EdgeKind::Documents,
        "provides" => EdgeKind::Owns,
        _ => return None,
    })
}
fn namespace(value: KnowledgeNamespace) -> Namespace {
    match value {
        KnowledgeNamespace::Frontend => Namespace::Frontend,
        KnowledgeNamespace::Backend => Namespace::Backend,
        KnowledgeNamespace::Shared => Namespace::Shared,
        KnowledgeNamespace::Product => Namespace::Product,
        KnowledgeNamespace::Design => Namespace::Design,
        KnowledgeNamespace::Asset => Namespace::Asset,
        KnowledgeNamespace::I18n => Namespace::I18n,
        KnowledgeNamespace::Contract => Namespace::Contract,
        KnowledgeNamespace::Verification => Namespace::Verification,
    }
}
fn evidence(value: crate::EvidenceStatus) -> Evidence {
    match value {
        crate::EvidenceStatus::Verified => Evidence::Compiler,
        crate::EvidenceStatus::Inferred => Evidence::Inferred,
        crate::EvidenceStatus::Assumed => Evidence::Assumed,
        crate::EvidenceStatus::Unknown => Evidence::Unknown,
    }
}
