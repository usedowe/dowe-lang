use super::*;
use crate::{CodeGraphError, CodeGraphResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeRegistry {
    pub schema: u32,
    /// Locally generated registries may preserve verified workflow evidence.
    /// User-authored registries remain untrusted by default.
    #[serde(default)]
    pub trusted: bool,
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

impl KnowledgeRegistry {
    pub fn apply(&self, graph: &mut UnifiedCodeGraph) -> CodeGraphResult<()> {
        if self.schema != 1 || self.nodes.len() > 1024 || self.edges.len() > 4096 {
            return Err(CodeGraphError::new(
                "knowledge registry schema or bounds are invalid",
            ));
        }
        let mut ids: BTreeSet<_> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        for node in &self.nodes {
            bounded(&node.id, 256)?;
            bounded(&node.name, 1024)?;
            if !node.id.starts_with("knowledge:") || !ids.insert(node.id.clone()) {
                return Err(CodeGraphError::new(
                    "knowledge ids must be unique and start with knowledge:",
                ));
            }
            if let Some(summary) = &node.summary {
                bounded(summary, 4096)?;
            }
            if let Some(path) = &node.path {
                bounded(path, 512)?;
                if Path::new(path)
                    .components()
                    .any(|part| !matches!(part, Component::Normal(_)))
                {
                    return Err(CodeGraphError::new(
                        "knowledge source path must remain project-relative",
                    ));
                }
                if path.starts_with('.')
                    || path.starts_with("agents/")
                    || path
                        .split('/')
                        .any(|p| p == "credentials" || p == "secrets")
                {
                    return Err(CodeGraphError::new(
                        "private runtime and credential paths cannot be knowledge sources",
                    ));
                }
            }
        }
        for edge in &self.edges {
            if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
                return Err(CodeGraphError::new(
                    "knowledge edge references an unknown node",
                ));
            }
        }
        graph
            .nodes
            .extend(self.nodes.iter().cloned().map(|mut node| {
                if !self.trusted && node.evidence == EvidenceStatus::Verified {
                    node.evidence = EvidenceStatus::Assumed;
                }
                node
            }));
        graph
            .edges
            .extend(self.edges.iter().cloned().map(|mut edge| {
                if !self.trusted && edge.evidence == EvidenceStatus::Verified {
                    edge.evidence = EvidenceStatus::Assumed;
                }
                edge
            }));
        graph.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(())
    }

    pub fn apply_to_clean(&self, graph: &mut crate::clean::CleanGraph) -> CodeGraphResult<()> {
        let mut unified = UnifiedCodeGraph {
            version: 2,
            root: String::new(),
            nodes: graph
                .nodes()
                .map(|node| KnowledgeNode {
                    id: node.id.clone(),
                    namespace: knowledge_namespace(node.namespace),
                    kind: UnifiedNodeKind::Source,
                    name: node.name.clone(),
                    path: node.path.clone(),
                    summary: None,
                    evidence: EvidenceStatus::Verified,
                })
                .collect(),
            edges: Vec::new(),
        };
        self.apply(&mut unified)?;
        for node in &self.nodes {
            graph.insert_node(crate::clean::CleanNode {
                id: node.id.clone(),
                namespace: clean_namespace(node.namespace),
                kind: format!("{:?}", node.kind).to_ascii_lowercase(),
                name: node.name.clone(),
                path: node.path.clone(),
                start_line: None,
                end_line: None,
                evidence: clean_evidence(node.evidence),
            });
        }
        for edge in &self.edges {
            graph
                .insert_edge(crate::clean::CleanEdge {
                    source: edge.from.clone(),
                    relation: format!("{:?}", edge.relation).to_ascii_lowercase(),
                    target: edge.to.clone(),
                    evidence: clean_evidence(edge.evidence),
                })
                .map_err(CodeGraphError::new)?;
        }
        Ok(())
    }
}

pub fn load_knowledge_registry(root: &Path, graph: &mut UnifiedCodeGraph) -> CodeGraphResult<()> {
    if let Some(registry) = read_knowledge_registry(root)? {
        registry.apply(graph)?;
    }
    Ok(())
}

pub fn load_knowledge_registry_into_clean(
    root: &Path,
    graph: &mut crate::clean::CleanGraph,
) -> CodeGraphResult<()> {
    if let Some(registry) = read_knowledge_registry(root)? {
        registry.apply_to_clean(graph)?;
    }
    Ok(())
}

fn read_knowledge_registry(root: &Path) -> CodeGraphResult<Option<KnowledgeRegistry>> {
    let path = root.join(".agent/knowledge.json");
    for ancestor in path.ancestors() {
        if std::fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(CodeGraphError::new(
                "knowledge registry must not traverse symlinks",
            ));
        }
    }
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
        return Err(CodeGraphError::new(
            "knowledge registry must be a regular file of at most 2 MiB",
        ));
    }
    let registry: KnowledgeRegistry = serde_json::from_slice(&std::fs::read(path)?)
        .map_err(|error| CodeGraphError::new(format!("invalid knowledge registry: {error}")))?;
    Ok(Some(registry))
}

fn knowledge_namespace(namespace: crate::clean::Namespace) -> KnowledgeNamespace {
    match namespace {
        crate::clean::Namespace::Frontend => KnowledgeNamespace::Frontend,
        crate::clean::Namespace::Backend => KnowledgeNamespace::Backend,
        crate::clean::Namespace::Shared => KnowledgeNamespace::Shared,
        crate::clean::Namespace::Product => KnowledgeNamespace::Product,
        crate::clean::Namespace::Design => KnowledgeNamespace::Design,
        crate::clean::Namespace::Asset => KnowledgeNamespace::Asset,
        crate::clean::Namespace::I18n => KnowledgeNamespace::I18n,
        crate::clean::Namespace::Contract => KnowledgeNamespace::Contract,
        crate::clean::Namespace::Verification => KnowledgeNamespace::Verification,
    }
}

fn clean_namespace(namespace: KnowledgeNamespace) -> crate::clean::Namespace {
    match namespace {
        KnowledgeNamespace::Frontend => crate::clean::Namespace::Frontend,
        KnowledgeNamespace::Backend => crate::clean::Namespace::Backend,
        KnowledgeNamespace::Shared => crate::clean::Namespace::Shared,
        KnowledgeNamespace::Product => crate::clean::Namespace::Product,
        KnowledgeNamespace::Design => crate::clean::Namespace::Design,
        KnowledgeNamespace::Asset => crate::clean::Namespace::Asset,
        KnowledgeNamespace::I18n => crate::clean::Namespace::I18n,
        KnowledgeNamespace::Contract => crate::clean::Namespace::Contract,
        KnowledgeNamespace::Verification => crate::clean::Namespace::Verification,
    }
}

fn clean_evidence(evidence: EvidenceStatus) -> crate::clean::Evidence {
    match evidence {
        EvidenceStatus::Verified => crate::clean::Evidence::Assumed,
        EvidenceStatus::Inferred => crate::clean::Evidence::Inferred,
        EvidenceStatus::Assumed => crate::clean::Evidence::Assumed,
        EvidenceStatus::Unknown => crate::clean::Evidence::Unknown,
    }
}

fn bounded(value: &str, max: usize) -> CodeGraphResult<()> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        Err(CodeGraphError::new(
            "knowledge text is empty or exceeds its bounds",
        ))
    } else {
        Ok(())
    }
}
