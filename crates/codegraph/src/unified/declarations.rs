use super::{
    EvidenceStatus, KnowledgeEdge, KnowledgeNamespace, KnowledgeNode, KnowledgeRelation,
    UnifiedNodeKind,
};
use crate::CodeGraph;
use dowe_compiler::{LanguageDocument, LanguageDocumentSymbol, document_symbols};
use std::path::{Component, Path};

pub(super) fn augment_dowe_declarations(
    graph: &CodeGraph,
    nodes: &mut Vec<KnowledgeNode>,
    edges: &mut Vec<KnowledgeEdge>,
) {
    let root = Path::new(&graph.root);
    for source in &graph.nodes {
        let Some(relative) = source.path.as_deref().filter(|p| p.ends_with(".dowe")) else {
            continue;
        };
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            continue;
        }
        let path = root.join(path);
        if path
            .ancestors()
            .any(|p| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink()))
        {
            continue;
        }
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        if !metadata.is_file() || metadata.len() > 1024 * 1024 {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let document = LanguageDocument {
            path,
            source: content,
        };
        for symbol in document_symbols(root, &document) {
            collect(&symbol, relative, &source.id, nodes, edges);
        }
    }
}

fn collect(
    symbol: &LanguageDocumentSymbol,
    path: &str,
    parent: &str,
    nodes: &mut Vec<KnowledgeNode>,
    edges: &mut Vec<KnowledgeEdge>,
) {
    let (keyword, name) = symbol
        .name
        .split_once(' ')
        .unwrap_or((&symbol.name, &symbol.name));
    let mut owner = parent.to_string();
    if let Some((namespace, kind)) = shape(keyword, path) {
        let id = format!("dowe:{keyword}:{path}:{name}");
        if !nodes.iter().any(|node| node.id == id) {
            nodes.push(KnowledgeNode {
                id: id.clone(),
                namespace,
                kind,
                name: name.into(),
                path: Some(path.into()),
                summary: Some(format!("{keyword} at line {}", symbol.range.start.line)),
                evidence: EvidenceStatus::Verified,
            });
            edges.push(KnowledgeEdge {
                from: parent.into(),
                to: id.clone(),
                relation: KnowledgeRelation::Contains,
                evidence: EvidenceStatus::Verified,
            });
        }
        owner = id;
    }
    for child in &symbol.children {
        collect(child, path, &owner, nodes, edges);
    }
}

fn shape(keyword: &str, path: &str) -> Option<(KnowledgeNamespace, UnifiedNodeKind)> {
    use KnowledgeNamespace as N;
    use UnifiedNodeKind as K;
    Some(match keyword {
        "layout" => (N::Frontend, K::Layout),
        "page" => (N::Frontend, K::Page),
        "component" => (N::Frontend, K::Component),
        "entity" => (N::Backend, K::Entity),
        "handler" => (N::Backend, K::Handler),
        "middleware" => (N::Backend, K::Middleware),
        "route" => (
            if path.starts_with("views/") {
                N::Frontend
            } else {
                N::Backend
            },
            K::Route,
        ),
        "type" | "schema" => (N::Shared, K::Schema),
        "theme" | "design" => (N::Design, K::DesignPattern),
        "test" => (N::Verification, K::Test),
        _ => return None,
    })
}
