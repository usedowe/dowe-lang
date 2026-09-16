use crate::{HarnessError, HarnessResult};
use dowe_codegraph::{
    EvidenceStatus, KnowledgeEdge, KnowledgeNamespace, KnowledgeNode, KnowledgeRegistry,
    KnowledgeRelation, UnifiedNodeKind,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[path = "product_document.rs"]
mod product_document;
#[path = "product_validation.rs"]
mod product_validation;
use product_document::render_product_document;
use product_validation::{
    reject_symlinked_product_store, slug, validate_discovery, validate_update,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifiedProductUpdate {
    pub workflow_id: String,
    pub objective: String,
    #[serde(default)]
    pub specification_path: Option<String>,
    #[serde(default)]
    pub contract_paths: Vec<String>,
    #[serde(default)]
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub check_names: Vec<String>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub use_cases: Vec<VerifiedProductUseCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifiedProductUseCase {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductDiscovery {
    pub product_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub observed_concepts: Vec<String>,
    #[serde(default)]
    pub inferred_capabilities: Vec<String>,
    #[serde(default)]
    pub observed_use_cases: Vec<ProductDiscoveryUseCase>,
    #[serde(default)]
    pub inferred_use_cases: Vec<ProductDiscoveryUseCase>,
    #[serde(default)]
    pub source_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductDiscoveryUseCase {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

/// Persists screenshot/planner observations as non-verified product evidence.
/// These nodes are deliberately never promoted to implemented capabilities by
/// discovery alone; a verified workflow must publish that transition.
pub fn record_product_discovery(
    root: &Path,
    discovery: &ProductDiscovery,
) -> HarnessResult<PathBuf> {
    validate_discovery(discovery)?;
    let directory = root.join(".agent");
    reject_symlinked_product_store(root, &directory)?;
    std::fs::create_dir_all(&directory)?;
    let registry_path = directory.join("knowledge.json");
    let mut registry = if registry_path.is_file() {
        serde_json::from_slice::<KnowledgeRegistry>(&std::fs::read(&registry_path)?)?
    } else {
        KnowledgeRegistry {
            schema: 1,
            trusted: false,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    };
    let product_id = format!("knowledge:product:{}", slug(&discovery.product_name));
    upsert_node(
        &mut registry.nodes,
        KnowledgeNode {
            id: product_id.clone(),
            namespace: KnowledgeNamespace::Product,
            kind: UnifiedNodeKind::Project,
            name: discovery.product_name.clone(),
            path: None,
            summary: discovery.category.clone(),
            evidence: EvidenceStatus::Inferred,
        },
    );
    for (evidence, concepts) in [
        (EvidenceStatus::Inferred, &discovery.observed_concepts),
        (EvidenceStatus::Assumed, &discovery.inferred_capabilities),
    ] {
        for concept in concepts {
            let id = format!("knowledge:discovery:{}", slug(concept));
            upsert_node(
                &mut registry.nodes,
                KnowledgeNode {
                    id: id.clone(),
                    namespace: KnowledgeNamespace::Product,
                    kind: UnifiedNodeKind::Capability,
                    name: concept.clone(),
                    path: None,
                    summary: Some(
                        "Screenshot/planner discovery; not implementation evidence.".into(),
                    ),
                    evidence,
                },
            );
            upsert_edge(
                &mut registry.edges,
                KnowledgeEdge {
                    from: product_id.clone(),
                    relation: KnowledgeRelation::Provides,
                    to: id,
                    evidence,
                },
            );
        }
    }
    for (evidence, use_cases) in [
        (EvidenceStatus::Inferred, &discovery.observed_use_cases),
        (EvidenceStatus::Assumed, &discovery.inferred_use_cases),
    ] {
        for use_case in use_cases {
            let use_case_id = format!("knowledge:discovered-use-case:{}", slug(&use_case.id));
            upsert_node(
                &mut registry.nodes,
                KnowledgeNode {
                    id: use_case_id.clone(),
                    namespace: KnowledgeNamespace::Product,
                    kind: UnifiedNodeKind::UseCase,
                    name: use_case.name.clone(),
                    path: None,
                    summary: use_case.goal.clone(),
                    evidence,
                },
            );
            upsert_edge(
                &mut registry.edges,
                KnowledgeEdge {
                    from: product_id.clone(),
                    relation: KnowledgeRelation::Contains,
                    to: use_case_id.clone(),
                    evidence,
                },
            );
            if let Some(actor) = &use_case.actor {
                let actor_id = format!("knowledge:discovered-actor:{}", slug(actor));
                upsert_node(
                    &mut registry.nodes,
                    KnowledgeNode {
                        id: actor_id.clone(),
                        namespace: KnowledgeNamespace::Product,
                        kind: UnifiedNodeKind::Actor,
                        name: actor.clone(),
                        path: None,
                        summary: None,
                        evidence,
                    },
                );
                upsert_edge(
                    &mut registry.edges,
                    KnowledgeEdge {
                        from: use_case_id,
                        relation: KnowledgeRelation::Provides,
                        to: actor_id,
                        evidence,
                    },
                );
            }
        }
    }
    for image in &discovery.source_images {
        let image_id = add_source_node(&mut registry.nodes, image, UnifiedNodeKind::Asset);
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: product_id.clone(),
                relation: KnowledgeRelation::Uses,
                to: image_id,
                evidence: EvidenceStatus::Inferred,
            },
        );
    }
    write_product_registry(&directory, &registry)
}

/// Records only post-verification facts. Planning and failed workflows must not call this API.
pub fn record_verified_product_update(
    root: &Path,
    update: &VerifiedProductUpdate,
) -> HarnessResult<PathBuf> {
    validate_update(update)?;
    let directory = root.join(".agent");
    reject_symlinked_product_store(root, &directory)?;
    std::fs::create_dir_all(&directory)?;
    let registry_path = directory.join("knowledge.json");
    let mut registry = if registry_path.is_file() {
        serde_json::from_slice::<KnowledgeRegistry>(&std::fs::read(&registry_path)?)?
    } else {
        KnowledgeRegistry {
            schema: 1,
            trusted: true,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    };
    registry.trusted = true;
    let capability_id = format!("knowledge:capability:{}", slug(&update.objective));
    upsert_node(
        &mut registry.nodes,
        KnowledgeNode {
            id: capability_id.clone(),
            namespace: KnowledgeNamespace::Product,
            kind: UnifiedNodeKind::Capability,
            name: update.objective.trim().to_string(),
            path: None,
            summary: Some("Implemented and verified by a completed Dowe workflow.".into()),
            evidence: EvidenceStatus::Verified,
        },
    );
    let workflow_id = format!("knowledge:workflow:{}", slug(&update.workflow_id));
    upsert_node(
        &mut registry.nodes,
        KnowledgeNode {
            id: workflow_id.clone(),
            namespace: KnowledgeNamespace::Verification,
            kind: UnifiedNodeKind::Test,
            name: format!("Workflow {}", update.workflow_id),
            path: None,
            summary: Some("Workflow completed integration, verification and review.".into()),
            evidence: EvidenceStatus::Verified,
        },
    );
    upsert_edge(
        &mut registry.edges,
        KnowledgeEdge {
            from: capability_id.clone(),
            relation: KnowledgeRelation::VerifiedBy,
            to: workflow_id,
            evidence: EvidenceStatus::Verified,
        },
    );
    if let Some(path) = &update.specification_path {
        let id = add_source_node(&mut registry.nodes, path, UnifiedNodeKind::Documentation);
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: capability_id.clone(),
                relation: KnowledgeRelation::SpecifiedBy,
                to: id,
                evidence: EvidenceStatus::Verified,
            },
        );
    }
    for path in &update.contract_paths {
        let id = format!("knowledge:contract:{}", slug(path));
        upsert_node(
            &mut registry.nodes,
            KnowledgeNode {
                id: id.clone(),
                namespace: KnowledgeNamespace::Contract,
                kind: UnifiedNodeKind::Contract,
                name: path.clone(),
                path: None,
                summary: Some("Bound contract for the verified workflow.".into()),
                evidence: EvidenceStatus::Verified,
            },
        );
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: capability_id.clone(),
                relation: KnowledgeRelation::DependsOn,
                to: id,
                evidence: EvidenceStatus::Verified,
            },
        );
    }
    for id in update.task_ids.iter().chain(update.check_names.iter()) {
        let node_id = format!("knowledge:evidence:{}", slug(id));
        upsert_node(
            &mut registry.nodes,
            KnowledgeNode {
                id: node_id.clone(),
                namespace: KnowledgeNamespace::Verification,
                kind: UnifiedNodeKind::Test,
                name: id.clone(),
                path: None,
                summary: Some("Declared workflow evidence passed.".into()),
                evidence: EvidenceStatus::Verified,
            },
        );
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: capability_id.clone(),
                relation: KnowledgeRelation::VerifiedBy,
                to: node_id,
                evidence: EvidenceStatus::Verified,
            },
        );
    }
    for use_case in &update.use_cases {
        let use_case_id = format!("knowledge:use-case:{}", slug(&use_case.id));
        upsert_node(
            &mut registry.nodes,
            KnowledgeNode {
                id: use_case_id.clone(),
                namespace: KnowledgeNamespace::Product,
                kind: UnifiedNodeKind::UseCase,
                name: use_case.name.clone(),
                path: None,
                summary: use_case.goal.clone(),
                evidence: EvidenceStatus::Verified,
            },
        );
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: capability_id.clone(),
                relation: KnowledgeRelation::Contains,
                to: use_case_id.clone(),
                evidence: EvidenceStatus::Verified,
            },
        );
        if let Some(actor) = &use_case.actor {
            let actor_id = format!("knowledge:actor:{}", slug(actor));
            upsert_node(
                &mut registry.nodes,
                KnowledgeNode {
                    id: actor_id.clone(),
                    namespace: KnowledgeNamespace::Product,
                    kind: UnifiedNodeKind::Actor,
                    name: actor.clone(),
                    path: None,
                    summary: None,
                    evidence: EvidenceStatus::Verified,
                },
            );
            upsert_edge(
                &mut registry.edges,
                KnowledgeEdge {
                    from: use_case_id,
                    relation: KnowledgeRelation::Provides,
                    to: actor_id,
                    evidence: EvidenceStatus::Verified,
                },
            );
        }
    }
    for path in &update.changed_files {
        let id = add_source_node(&mut registry.nodes, path, UnifiedNodeKind::Source);
        upsert_edge(
            &mut registry.edges,
            KnowledgeEdge {
                from: capability_id.clone(),
                relation: KnowledgeRelation::Implements,
                to: id,
                evidence: EvidenceStatus::Verified,
            },
        );
    }
    if registry.nodes.len() > 1024 || registry.edges.len() > 4096 {
        return Err(HarnessError::new(
            "product knowledge registry exceeds its bounds",
        ));
    }
    write_product_registry(&directory, &registry)
}

fn write_product_registry(
    directory: &Path,
    registry: &KnowledgeRegistry,
) -> HarnessResult<PathBuf> {
    if registry.nodes.len() > 2048 || registry.edges.len() > 8192 {
        return Err(HarnessError::new(
            "product knowledge registry exceeds its bounds",
        ));
    }
    let bytes = serde_json::to_vec_pretty(registry)?;
    let temporary = directory.join(".knowledge.json.tmp");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, directory.join("knowledge.json"))?;
    let product_path = directory.join("product.md");
    let temporary = directory.join(".product.md.tmp");
    std::fs::write(&temporary, render_product_document(registry))?;
    std::fs::rename(&temporary, &product_path)?;
    Ok(product_path)
}

fn add_source_node(nodes: &mut Vec<KnowledgeNode>, path: &str, kind: UnifiedNodeKind) -> String {
    let id = format!("knowledge:source:{}", slug(path));
    upsert_node(
        nodes,
        KnowledgeNode {
            id: id.clone(),
            namespace: KnowledgeNamespace::Shared,
            kind,
            name: path.to_string(),
            path: Some(path.to_string()),
            summary: None,
            evidence: EvidenceStatus::Verified,
        },
    );
    id
}

fn upsert_node(nodes: &mut Vec<KnowledgeNode>, node: KnowledgeNode) {
    if let Some(existing) = nodes.iter_mut().find(|existing| existing.id == node.id) {
        *existing = node;
    } else {
        nodes.push(node);
    }
}

fn upsert_edge(edges: &mut Vec<KnowledgeEdge>, edge: KnowledgeEdge) {
    if let Some(existing) = edges.iter_mut().find(|existing| {
        existing.from == edge.from && existing.to == edge.to && existing.relation == edge.relation
    }) {
        *existing = edge;
    } else {
        edges.push(edge);
    }
}
