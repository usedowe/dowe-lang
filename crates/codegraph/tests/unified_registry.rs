use dowe_codegraph::*;

fn graph() -> UnifiedCodeGraph {
    UnifiedCodeGraph {
        version: 2,
        root: ".".into(),
        nodes: vec![],
        edges: vec![],
    }
}

fn registry() -> KnowledgeRegistry {
    KnowledgeRegistry {
        schema: 1,
        trusted: false,
        nodes: vec![KnowledgeNode {
            id: "knowledge:login".into(),
            namespace: KnowledgeNamespace::Product,
            kind: UnifiedNodeKind::Capability,
            name: "Login".into(),
            path: Some("docs/login.md".into()),
            summary: None,
            evidence: EvidenceStatus::Verified,
        }],
        edges: vec![],
    }
}

#[test]
fn registry_never_promotes_user_claims_to_verified_evidence() {
    let mut graph = graph();
    registry().apply(&mut graph).unwrap();
    assert_eq!(graph.nodes[0].evidence, EvidenceStatus::Assumed);
    assert_eq!(
        graph.search("login", UnifiedScope::Product, 10).nodes.len(),
        1
    );
}

#[test]
fn invalid_registry_is_atomic() {
    let mut graph = graph();
    let mut registry = registry();
    registry.edges.push(KnowledgeEdge {
        from: "knowledge:login".into(),
        to: "missing".into(),
        relation: KnowledgeRelation::Implements,
        evidence: EvidenceStatus::Verified,
    });
    assert!(registry.apply(&mut graph).is_err());
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}

#[test]
fn private_paths_and_duplicate_ids_are_rejected() {
    for path in [
        "../secret",
        "/secret",
        ".dowe/auth.json",
        "agents/instructions.md",
        "secrets/token.json",
    ] {
        let mut registry = registry();
        registry.nodes[0].path = Some(path.into());
        assert!(registry.apply(&mut graph()).is_err(), "{path}");
    }
    let mut registry = registry();
    registry.nodes.push(registry.nodes[0].clone());
    assert!(registry.apply(&mut graph()).is_err());
}

#[test]
fn compiler_extracts_real_declarations_and_classifies_root_level_files() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("home.dowe"),
        "page Home\n  Text\n    \"entity Fake\"\n",
    )
    .unwrap();
    let mut source = build_codegraph(root.path(), BuildOptions::default()).unwrap();
    source.root = root
        .path()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let graph = unified_graph_from_codegraph(&source);
    let page = graph
        .nodes
        .iter()
        .find(|n| n.kind == UnifiedNodeKind::Page)
        .unwrap();
    assert_eq!(page.name, "Home");
    assert_eq!(page.namespace, KnowledgeNamespace::Frontend);
    assert!(
        !graph
            .nodes
            .iter()
            .any(|n| n.kind == UnifiedNodeKind::Entity)
    );
    assert!(
        graph
            .edges
            .iter()
            .any(|e| e.to == page.id && e.relation == KnowledgeRelation::Contains)
    );
}

#[cfg(unix)]
#[test]
fn registry_symlinks_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join(".agent")).unwrap();
    assert!(load_knowledge_registry(root.path(), &mut graph()).is_err());
}
