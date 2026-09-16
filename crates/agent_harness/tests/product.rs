use dowe_agent_harness::{
    ProductDiscovery, VerifiedProductUpdate, VerifiedProductUseCase, record_product_discovery,
    record_verified_product_update,
};
use dowe_codegraph::{EvidenceStatus, KnowledgeRegistry, KnowledgeRelation};

#[test]
fn verified_workflow_publishes_product_use_cases() {
    let root = tempfile::tempdir().unwrap();
    let path = record_verified_product_update(
        root.path(),
        &VerifiedProductUpdate {
            workflow_id: "wf-1".into(),
            objective: "P2P exchange".into(),
            specification_path: Some("specs/p2p.md".into()),
            contract_paths: vec![".agent/contracts.json".into()],
            task_ids: vec!["implement-p2p".into()],
            check_names: vec!["integration".into()],
            changed_files: vec!["views/p2p.dowe".into()],
            use_cases: vec![VerifiedProductUseCase {
                id: "create-offer".into(),
                name: "Create an offer".into(),
                actor: Some("User".into()),
                goal: Some("Offer the currency to another user.".into()),
            }],
        },
    )
    .unwrap();
    let document = std::fs::read_to_string(path).unwrap();
    assert!(document.contains("P2P exchange"));
    assert!(document.contains("Create an offer"));
    let registry: KnowledgeRegistry =
        serde_json::from_slice(&std::fs::read(root.path().join(".agent/knowledge.json")).unwrap())
            .unwrap();
    assert!(
        registry
            .edges
            .iter()
            .any(|edge| edge.relation == KnowledgeRelation::VerifiedBy)
    );
}

#[test]
fn screenshot_discovery_remains_non_verified_evidence() {
    let root = tempfile::tempdir().unwrap();
    let document_path = record_product_discovery(
        root.path(),
        &ProductDiscovery {
            product_name: "Dowe Coin".into(),
            category: Some("digital currency".into()),
            observed_concepts: vec!["wallet".into()],
            inferred_capabilities: vec!["purchase".into()],
            observed_use_cases: Vec::new(),
            inferred_use_cases: Vec::new(),
            source_images: vec!["references/home.png".into()],
        },
    )
    .unwrap();
    let document = std::fs::read_to_string(document_path).unwrap();
    assert!(document.contains("Product: Dowe Coin"));
    assert!(document.contains("wallet"));
    let registry: KnowledgeRegistry =
        serde_json::from_slice(&std::fs::read(root.path().join(".agent/knowledge.json")).unwrap())
            .unwrap();
    assert!(
        registry
            .nodes
            .iter()
            .any(|node| { node.name == "wallet" && node.evidence == EvidenceStatus::Inferred })
    );
    assert!(
        registry
            .nodes
            .iter()
            .any(|node| { node.name == "purchase" && node.evidence == EvidenceStatus::Assumed })
    );
}
