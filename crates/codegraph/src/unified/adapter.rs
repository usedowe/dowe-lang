use super::*;
use crate::{CodeGraph, EdgeKind, Node, NodeKind};

pub fn unified_graph_from_codegraph(graph: &CodeGraph) -> UnifiedCodeGraph {
    let mut nodes = graph.nodes.iter().map(convert_node).collect::<Vec<_>>();
    let mut edges = graph
        .edges
        .iter()
        .filter_map(convert_edge)
        .collect::<Vec<_>>();
    declarations::augment_dowe_declarations(graph, &mut nodes, &mut edges);
    let mut namespaces = std::collections::BTreeMap::new();
    for node in nodes.iter().filter(|n| n.id.starts_with("dowe:")) {
        if let Some(path) = &node.path {
            namespaces
                .entry(path.clone())
                .and_modify(|current| {
                    if *current != node.namespace {
                        *current = KnowledgeNamespace::Shared;
                    }
                })
                .or_insert(node.namespace);
        }
    }
    for node in nodes
        .iter_mut()
        .filter(|n| n.kind == UnifiedNodeKind::Source)
    {
        if let Some(namespace) = node.path.as_ref().and_then(|p| namespaces.get(p)) {
            node.namespace = *namespace;
        }
    }
    UnifiedCodeGraph {
        version: 2,
        root: graph.root.clone(),
        nodes,
        edges,
    }
}

fn convert_node(node: &Node) -> KnowledgeNode {
    let (namespace, kind) = match node.kind {
        NodeKind::Spec => (KnowledgeNamespace::Product, UnifiedNodeKind::Requirement),
        NodeKind::Contract => (KnowledgeNamespace::Contract, UnifiedNodeKind::Contract),
        NodeKind::Acceptance | NodeKind::Test => {
            (KnowledgeNamespace::Verification, UnifiedNodeKind::Test)
        }
        NodeKind::Doc => (KnowledgeNamespace::Product, UnifiedNodeKind::Documentation),
        NodeKind::GeneratedArtifact => (KnowledgeNamespace::Shared, UnifiedNodeKind::Source),
        NodeKind::Symbol => (KnowledgeNamespace::Shared, UnifiedNodeKind::Source),
        NodeKind::Workspace => (KnowledgeNamespace::Shared, UnifiedNodeKind::Project),
        NodeKind::Crate
        | NodeKind::Module
        | NodeKind::File
        | NodeKind::AgentHarness
        | NodeKind::OwnershipArea => (
            namespace_for_path(node.path.as_deref()),
            UnifiedNodeKind::Source,
        ),
    };
    KnowledgeNode {
        id: node.id.clone(),
        namespace,
        kind,
        name: node.name.clone(),
        path: node.path.clone(),
        summary: None,
        evidence: if matches!(node.kind, NodeKind::Symbol | NodeKind::OwnershipArea) {
            EvidenceStatus::Inferred
        } else {
            EvidenceStatus::Verified
        },
    }
}

fn convert_edge(edge: &crate::Edge) -> Option<KnowledgeEdge> {
    let relation = match edge.kind {
        EdgeKind::Contains => KnowledgeRelation::Contains,
        EdgeKind::DependsOn => KnowledgeRelation::DependsOn,
        EdgeKind::Owns => KnowledgeRelation::Provides,
        EdgeKind::Implements => KnowledgeRelation::Implements,
        EdgeKind::Tests | EdgeKind::Validates => KnowledgeRelation::VerifiedBy,
        EdgeKind::Documents | EdgeKind::DeclaresContract => KnowledgeRelation::SpecifiedBy,
        EdgeKind::Generates => KnowledgeRelation::Provides,
        EdgeKind::Duplicates | EdgeKind::Reexports => return None,
    };
    Some(KnowledgeEdge {
        from: edge.from.clone(),
        relation,
        to: edge.to.clone(),
        evidence: if matches!(edge.kind, EdgeKind::Contains | EdgeKind::DependsOn) {
            EvidenceStatus::Verified
        } else {
            EvidenceStatus::Inferred
        },
    })
}

fn namespace_for_path(path: Option<&str>) -> KnowledgeNamespace {
    let path = path.unwrap_or_default();
    if path.starts_with("views/") {
        KnowledgeNamespace::Frontend
    } else if path.starts_with("server/") {
        KnowledgeNamespace::Backend
    } else if path.starts_with("docs") || path.starts_with("specs") {
        KnowledgeNamespace::Product
    } else {
        KnowledgeNamespace::Shared
    }
}
