use crate::persistence::replace_file_contributions;
use crate::{CodeGraph, CodeGraphMode, Edge, EdgeKind, Node, NodeKind};

fn node(id: &str) -> Node {
    Node {
        id: id.into(),
        kind: NodeKind::File,
        path: Some(format!("{id}.rs")),
        name: id.into(),
        language: "rust".into(),
        owner: None,
        fingerprint: "hash".into(),
        metrics: None,
        source_range: None,
    }
}

#[test]
fn new_file_contributions_include_their_edges() {
    let base = CodeGraph {
        mode: CodeGraphMode::Project,
        root: ".".into(),
        nodes: vec![node("a")],
        edges: vec![],
    };
    let edge = Edge {
        from: "b".into(),
        to: "a".into(),
        kind: EdgeKind::DependsOn,
    };
    let mut replacement = base.clone();
    replacement.nodes.push(node("b"));
    replacement.edges.push(edge.clone());
    let result = replace_file_contributions(base, &replacement, &["b.rs".into()], &[]);
    assert_eq!(result.edges, [edge]);
    assert_eq!(result.nodes.len(), 2);
}

#[test]
fn renamed_symbols_keep_replacement_edges_and_drop_deleted_edges() {
    let mut base = CodeGraph {
        mode: CodeGraphMode::Project,
        root: ".".into(),
        nodes: vec![node("a"), node("b")],
        edges: vec![Edge {
            from: "b".into(),
            to: "a".into(),
            kind: EdgeKind::DependsOn,
        }],
    };
    let mut replacement = base.clone();
    replacement.nodes[1].id = "renamed".into();
    replacement.edges[0].from = "renamed".into();
    base = replace_file_contributions(base, &replacement, &["b.rs".into()], &[]);
    assert_eq!(base.edges, replacement.edges);
    replacement.nodes.retain(|node| node.id != "renamed");
    replacement.edges.clear();
    let result = replace_file_contributions(base, &replacement, &[], &["b.rs".into()]);
    assert!(result.edges.is_empty());
    assert_eq!(result.nodes.len(), 1);
}
