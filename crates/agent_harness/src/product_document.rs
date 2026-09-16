use dowe_codegraph::{
    EvidenceStatus, KnowledgeNamespace, KnowledgeRegistry, KnowledgeRelation, UnifiedNodeKind,
};
use std::collections::BTreeMap;

pub(super) fn render_product_document(registry: &KnowledgeRegistry) -> String {
    let capabilities = registry
        .nodes
        .iter()
        .filter(|node| {
            node.namespace == KnowledgeNamespace::Product
                && node.kind == UnifiedNodeKind::Capability
        })
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let products = registry
        .nodes
        .iter()
        .filter(|node| {
            node.namespace == KnowledgeNamespace::Product && node.kind == UnifiedNodeKind::Project
        })
        .collect::<Vec<_>>();
    let mut output = String::from(
        "# Product knowledge\n\nThis document is generated from verified workflow evidence and clearly labeled discovery evidence.\n\n",
    );
    if capabilities.is_empty() {
        output.push_str("No verified capabilities recorded yet.\n");
    }
    for product in products {
        output.push_str(&format!(
            "## Product: {}\n\n- Evidence: {:?}\n- {}\n",
            product.name,
            product.evidence,
            product
                .summary
                .as_deref()
                .unwrap_or("No category recorded."),
        ));
        for edge in registry
            .edges
            .iter()
            .filter(|edge| edge.from == product.id && edge.relation == KnowledgeRelation::Contains)
        {
            if let Some(use_case) = registry.nodes.iter().find(|candidate| {
                candidate.id == edge.to && candidate.kind == UnifiedNodeKind::UseCase
            }) {
                output.push_str(&format!(
                    "- Discovered use case: {} (evidence: {:?})\n",
                    use_case.name, use_case.evidence
                ));
            }
        }
        output.push('\n');
    }
    for node in capabilities.values() {
        output.push_str(&format!(
            "## {}\n\n- Status: {}\n- Evidence: {:?}\n- {}\n\n",
            node.name,
            if node.evidence == EvidenceStatus::Verified {
                "implemented"
            } else {
                "discovered"
            },
            node.evidence,
            node.summary.as_deref().unwrap_or("No summary.")
        ));
        for edge in registry
            .edges
            .iter()
            .filter(|edge| edge.from == node.id && edge.relation == KnowledgeRelation::Contains)
        {
            let Some(use_case) = registry.nodes.iter().find(|candidate| {
                candidate.id == edge.to && candidate.kind == UnifiedNodeKind::UseCase
            }) else {
                continue;
            };
            output.push_str(&format!("- Use case: {}", use_case.name));
            if let Some(goal) = use_case.summary.as_deref() {
                output.push_str(&format!(" — {goal}"));
            }
            output.push('\n');
        }
        output.push('\n');
    }
    output
}
