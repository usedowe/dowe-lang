use dowe_codegraph::clean::*;
use std::fs;

fn node(id: &str, namespace: Namespace, name: &str) -> CleanNode {
    CleanNode {
        id: id.into(),
        namespace,
        kind: "symbol".into(),
        name: name.into(),
        path: None,
        start_line: None,
        end_line: None,
        evidence: Evidence::Compiler,
    }
}

#[test]
fn one_graph_keeps_frontend_backend_edges() {
    let mut graph = CleanGraph::default();
    graph.insert_node(node("frontend:login", Namespace::Frontend, "Login"));
    graph.insert_node(node("backend:login", Namespace::Backend, "login"));
    graph
        .insert_edge(CleanEdge {
            source: "frontend:login".into(),
            relation: "calls".into(),
            target: "backend:login".into(),
            evidence: Evidence::Compiler,
        })
        .unwrap();
    let result = GraphQuery {
        kind: QueryKind::Dependencies,
        value: "frontend:login".into(),
        namespace: None,
        max_nodes: 10,
        max_depth: 4,
    }
    .execute(&graph);
    assert_eq!(result.nodes[0].namespace, Namespace::Backend);
}

#[test]
fn bounded_search_reports_incomplete_results() {
    let mut graph = CleanGraph::default();
    for index in 0..3 {
        graph.insert_node(node(
            &format!("frontend:{index}"),
            Namespace::Frontend,
            "Card",
        ));
    }
    let result = GraphQuery {
        kind: QueryKind::Search,
        value: "Card".into(),
        namespace: Some(Namespace::Frontend),
        max_nodes: 2,
        max_depth: 0,
    }
    .execute(&graph);
    assert_eq!(result.nodes.len(), 2);
    assert!(result.truncated);
}

#[test]
fn related_query_includes_incoming_and_outgoing_neighbors() {
    let mut graph = CleanGraph::default();
    for id in ["a", "b", "c"] {
        graph.insert_node(node(id, Namespace::Backend, id));
    }
    graph
        .insert_edge(CleanEdge {
            source: "a".into(),
            relation: "uses".into(),
            target: "b".into(),
            evidence: Evidence::Inferred,
        })
        .unwrap();
    graph
        .insert_edge(CleanEdge {
            source: "b".into(),
            relation: "uses".into(),
            target: "c".into(),
            evidence: Evidence::Inferred,
        })
        .unwrap();
    let result = GraphQuery {
        kind: QueryKind::Related,
        value: "b".into(),
        namespace: None,
        max_nodes: 8,
        max_depth: 1,
    }
    .execute(&graph);
    let mut ids = result
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(ids, ["a", "c"]);
    assert_eq!(result.edges.len(), 2);
}

#[test]
fn search_is_case_insensitive_for_dowe_names() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("main.dowe"), "main\n").unwrap();
    fs::create_dir_all(temp.path().join("views")).unwrap();
    fs::write(temp.path().join("views/login.dowe"), "page Login\n").unwrap();
    let graph = build_clean_graph(temp.path(), Default::default()).unwrap();
    let result = GraphQuery {
        kind: QueryKind::Search,
        value: "login".into(),
        namespace: None,
        max_nodes: 8,
        max_depth: 0,
    }
    .execute(&graph);
    assert!(result.nodes.iter().any(|node| node.name == "Login"));
}

#[test]
fn clean_extractor_classifies_dowe_assets_i18n_and_design_files() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("public")).unwrap();
    std::fs::create_dir_all(root.path().join("i18n")).unwrap();
    std::fs::create_dir_all(root.path().join("design")).unwrap();
    std::fs::create_dir_all(root.path().join("docs")).unwrap();
    std::fs::write(root.path().join("public/logo.svg"), "<svg/>").unwrap();
    std::fs::write(
        root.path().join("i18n/en.json"),
        "{\"home\":{\"title\":\"Home\"}}\n",
    )
    .unwrap();
    std::fs::write(root.path().join("design/tokens.dowe"), "type Color\n").unwrap();
    std::fs::write(
        root.path().join("docs/product.md"),
        "# Product\n## Capability: Login\n### Use case: Sign in\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert_eq!(
        graph.node("file:public/logo.svg").unwrap().namespace,
        Namespace::Asset
    );
    assert_eq!(
        graph.node("file:i18n/en.json").unwrap().namespace,
        Namespace::I18n
    );
    assert_eq!(
        graph
            .node("dowe:i18n:i18n/en.json:home.title")
            .unwrap()
            .kind,
        "translation_key"
    );
    assert_eq!(
        graph.node("file:design/tokens.dowe").unwrap().namespace,
        Namespace::Design
    );
    assert_eq!(
        graph.node("dowe:product:docs/product.md:2").unwrap().kind,
        "capability"
    );
    assert_eq!(
        graph.node("dowe:product:docs/product.md:3").unwrap().kind,
        "use_case"
    );
    assert!(graph.node("dowe:i18n-locale:en").is_some());
    assert!(graph.edges().any(|edge| {
        edge.relation == "translates" && edge.target == "dowe:i18n:i18n/en.json:home.title"
    }));
    assert!(graph.edges().any(|edge| {
        edge.relation == "contains"
            && edge.source == "dowe:product:docs/product.md:2"
            && edge.target == "dowe:product:docs/product.md:3"
    }));
    assert!(graph.edges().any(|edge| {
        edge.relation == "supports_use_case"
            && edge.source == "dowe:product:docs/product.md:2"
            && edge.target == "dowe:product:docs/product.md:3"
    }));
    assert!(graph.node("dowe:asset:public/logo.svg").is_some());
}

#[test]
fn clean_persistent_binding_round_trips_generation_identity() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.py"), "print(1)\n").unwrap();
    let snapshot = refresh_persistent_clean_codegraph(root.path()).unwrap();
    let binding = clean_binding(&snapshot);
    assert_eq!(binding.generation, snapshot.generation.unwrap());
    assert_eq!(binding.revision, snapshot.manifest.revision);
    assert_eq!(binding.root, snapshot.manifest.root);
    assert_eq!(binding.mode, snapshot.manifest.mode);
}

#[test]
fn clean_extractor_records_dowe_import_edges() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views/components")).unwrap();
    std::fs::write(
        root.path().join("views/page.dowe"),
        "import Card from \"@/views/components/card.dowe\"\npage Home\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("views/components/card.dowe"),
        "component Card\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    let result = GraphQuery {
        kind: QueryKind::Dependencies,
        value: "file:views/page.dowe".into(),
        namespace: None,
        max_nodes: 10,
        max_depth: 1,
    }
    .execute(&graph);
    assert!(
        result
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("views/components/card.dowe"))
    );
}

#[test]
fn clean_extractor_links_dowe_symbol_usage() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views/components")).unwrap();
    std::fs::write(
        root.path().join("views/home.dowe"),
        "import Card from \"@/views/components/card.dowe\"\npage Home\n  Card\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("views/components/card.dowe"),
        "component Card\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    let result = GraphQuery {
        kind: QueryKind::Consumers,
        value: "dowe:component:views/components/card.dowe:Card".into(),
        namespace: None,
        max_nodes: 10,
        max_depth: 1,
    }
    .execute(&graph);
    assert!(
        result
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("views/home.dowe"))
    );
}

#[test]
fn clean_extractor_preserves_dowe_reference_relations() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::create_dir_all(root.path().join("server")).unwrap();
    std::fs::write(
        root.path().join("views/home.dowe"),
        "page Home\n  route handler:health layout:AppLayout\n",
    )
    .unwrap();
    std::fs::write(root.path().join("views/layout.dowe"), "layout AppLayout\n").unwrap();
    std::fs::write(root.path().join("server/health.dowe"), "handler health\n").unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert!(graph.edges().any(|edge| {
        edge.relation == "uses_layout" && edge.target == "dowe:layout:views/layout.dowe:AppLayout"
    }));
    assert!(graph.edges().any(|edge| {
        edge.relation == "handled_by" && edge.target == "dowe:handler:server/health.dowe:health"
    }));
    assert_eq!(
        graph
            .edges()
            .filter(|edge| edge.relation == "handled_by")
            .count(),
        1
    );
}

#[test]
fn clean_extractor_does_not_promote_comment_text_to_compiler_edges() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::write(
        root.path().join("views/home.dowe"),
        "page Home\n// layout:AppLayout\n",
    )
    .unwrap();
    std::fs::write(root.path().join("views/layout.dowe"), "layout AppLayout\n").unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert!(!graph.edges().any(|edge| {
        edge.relation == "uses_layout" && edge.target == "dowe:layout:views/layout.dowe:AppLayout"
    }));
}

#[test]
fn clean_extractor_does_not_promote_declarations_inside_unknown_syntax() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("server")).unwrap();
    std::fs::write(
        root.path().join("server/unknown.dowe"),
        "unsupported_wrapper\n  handler forged\n",
    )
    .unwrap();

    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert!(
        !graph
            .nodes()
            .any(|node| { node.kind == "handler" && node.name == "forged" })
    );
}

#[test]
fn clean_extractor_links_translation_variants_across_locales() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("i18n")).unwrap();
    std::fs::write(
        root.path().join("i18n/en.json"),
        "{\"home\":{\"title\":\"Home\"}}\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("i18n/es.json"),
        "{\"home\":{\"title\":\"Inicio\"}}\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert!(
        graph
            .edges()
            .any(|edge| edge.relation == "translation_variant")
    );
}

#[test]
fn knowledge_registry_can_reference_extracted_nodes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::write(root.path().join("views/home.dowe"), "page Home\n").unwrap();
    std::fs::create_dir_all(root.path().join(".agent")).unwrap();
    std::fs::write(
        root.path().join(".agent/knowledge.json"),
        r#"{"schema":1,"nodes":[{"id":"knowledge:home","namespace":"frontend","kind":"capability","name":"Home capability","path":"views/home.dowe","summary":"The home page","evidence":"inferred"}],"edges":[{"from":"knowledge:home","relation":"references","to":"dowe:page:views/home.dowe:Home","evidence":"inferred"}]}"#,
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert_eq!(
        graph.node("knowledge:home").unwrap().namespace,
        Namespace::Frontend
    );
    assert!(graph.edges().any(|edge| {
        edge.source == "knowledge:home"
            && edge.target == "dowe:page:views/home.dowe:Home"
            && edge.evidence == Evidence::Inferred
    }));
}

#[test]
fn clean_extractor_links_dowe_middleware_and_schema_references() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("server")).unwrap();
    std::fs::write(
        root.path().join("server/api.dowe"),
        "type ApiResponse\ntype UserId\nmiddleware requireAuth\nroute middleware:requireAuth schema:ApiResponse type:UserId\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert!(graph.edges().any(|edge| edge.relation == "uses_middleware"));
    assert!(graph.edges().any(|edge| edge.relation == "uses_schema"));
    assert!(graph.edges().any(|edge| edge.relation == "uses_type"));
}

#[test]
fn clean_extractor_links_dowe_reference_lists() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("server")).unwrap();
    std::fs::write(
        root.path().join("server/api.dowe"),
        "middleware secure\nmiddleware audit\nroute middleware:[secure audit]\n",
    )
    .unwrap();
    let graph = build_clean_graph(root.path(), Default::default()).unwrap();
    assert_eq!(
        graph
            .edges()
            .filter(|edge| edge.relation == "uses_middleware")
            .count(),
        2
    );
}
