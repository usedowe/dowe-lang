use dowe_codegraph::*;

fn node(id: &str) -> KnowledgeNode {
    KnowledgeNode {
        id: id.into(),
        namespace: KnowledgeNamespace::Backend,
        kind: UnifiedNodeKind::Entity,
        name: id.into(),
        path: None,
        summary: None,
        evidence: EvidenceStatus::Verified,
    }
}

fn graph() -> UnifiedCodeGraph {
    UnifiedCodeGraph {
        version: 2,
        root: ".".into(),
        nodes: vec![
            node("user"),
            node("user_profile"),
            node("login"),
            node("page"),
        ],
        edges: vec![
            KnowledgeEdge {
                from: "login".into(),
                to: "user".into(),
                relation: KnowledgeRelation::Reads,
                evidence: EvidenceStatus::Verified,
            },
            KnowledgeEdge {
                from: "page".into(),
                to: "login".into(),
                relation: KnowledgeRelation::Calls,
                evidence: EvidenceStatus::Verified,
            },
        ],
    }
}

#[test]
fn exact_lookup_does_not_expand_name_prefixes() {
    let result = graph().get_related("user", 0, 10);
    assert_eq!(
        result
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>(),
        ["user"]
    );
    assert!(graph().get_related("missing", 8, 10).nodes.is_empty());
}

#[test]
fn impact_follows_consumers_not_dependencies() {
    let graph = graph();
    assert_eq!(graph.get_impact("user", 0, 10).len(), 1);
    let ids = graph
        .get_impact("user", 8, 10)
        .into_iter()
        .map(|n| n.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, ["user", "login", "page"]);
    assert_eq!(graph.get_impact("page", 8, 10).len(), 1);
    assert_eq!(graph.get_dependencies("login", 10)[0].id, "user");
}

#[test]
fn truncation_counts_matches_and_edges() {
    let graph = graph();
    assert!(!graph.search("page", UnifiedScope::All, 1).truncated);
    assert!(graph.search("user", UnifiedScope::All, 1).truncated);
    let mut graph = graph;
    for index in 0..1000 {
        let id = format!("consumer{index}");
        graph.nodes.push(node(&id));
        graph.edges.push(KnowledgeEdge {
            from: id,
            to: "user".into(),
            relation: KnowledgeRelation::Uses,
            evidence: EvidenceStatus::Verified,
        });
    }
    let result = graph.get_related("user", 8, 1);
    assert!(result.edges.len() <= 512);
    assert!(result.impact.len() <= 128);
    assert!(result.truncated);
}

#[test]
fn cycles_and_container_edges_do_not_expand_impact_to_siblings() {
    let mut graph = graph();
    graph.edges.push(KnowledgeEdge {
        from: "user".into(),
        to: "login".into(),
        relation: KnowledgeRelation::Reads,
        evidence: EvidenceStatus::Verified,
    });
    graph.edges.push(KnowledgeEdge {
        from: "user_profile".into(),
        to: "user".into(),
        relation: KnowledgeRelation::Contains,
        evidence: EvidenceStatus::Verified,
    });
    assert_eq!(graph.get_impact("user", 8, 10).len(), 3);
}

#[test]
fn impact_reports_unexplored_frontiers_and_exact_capacity() {
    let graph = graph();
    assert!(graph.query_impact("user", 1, 10).truncated);
    assert!(graph.query_impact("user", 8, 2).truncated);
    assert!(!graph.query_impact("user", 2, 3).truncated);
    assert!(!graph.query_impact("page", 0, 1).truncated);
    assert!(graph.query_impact("missing", 8, 10).nodes.is_empty());
}

#[test]
fn neighbors_report_limits_without_dangling_or_missing_roots() {
    let mut graph = graph();
    graph.edges.push(KnowledgeEdge {
        from: "login".into(),
        to: "page".into(),
        relation: KnowledgeRelation::Uses,
        evidence: EvidenceStatus::Inferred,
    });
    graph.edges.push(KnowledgeEdge {
        from: "missing".into(),
        to: "user".into(),
        relation: KnowledgeRelation::Uses,
        evidence: EvidenceStatus::Inferred,
    });
    assert!(graph.query_dependencies("login", 1).truncated);
    assert!(!graph.query_dependencies("login", 2).truncated);
    assert!(!graph.query_consumers("user", 1).truncated);
    assert!(graph.query_dependencies("missing", 10).nodes.is_empty());
    assert_eq!(graph.query_consumers("user", 10).nodes.len(), 1);
}

#[test]
fn closed_cycles_do_not_report_unexplored_frontiers() {
    let mut graph = graph();
    graph.edges.retain(|edge| edge.from != "page");
    graph.edges.push(KnowledgeEdge {
        from: "user".into(),
        to: "login".into(),
        relation: KnowledgeRelation::Reads,
        evidence: EvidenceStatus::Verified,
    });
    assert!(!graph.query_impact("user", 1, 2).truncated);
}

#[test]
fn scoped_search_does_not_leak_cross_stack_impact_nodes() {
    let mut graph = graph();
    graph
        .nodes
        .iter_mut()
        .find(|node| node.id == "page")
        .expect("page node")
        .namespace = KnowledgeNamespace::Frontend;
    let result = graph.query(&UnifiedGraphQuery {
        query: "user".into(),
        scope: UnifiedScope::Backend,
        limit: 10,
        depth: 8,
    });
    assert!(
        result
            .nodes
            .iter()
            .all(|node| { matches!(node.namespace, KnowledgeNamespace::Backend) })
    );
    assert!(
        result
            .impact
            .iter()
            .all(|node| { matches!(node.namespace, KnowledgeNamespace::Backend) })
    );
    assert!(!result.impact.iter().any(|node| node.id == "page"));
}
