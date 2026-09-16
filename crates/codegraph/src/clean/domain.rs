use super::{CleanGraph, Namespace, build_clean_graph};
use crate::{BuildOptions, CodeGraphResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// The named domain projections used by planning and impact queries.
/// Extraction still happens once in CleanGraph; these projections preserve the
/// compiler/durable/inferred evidence attached to every node and edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainGraphs {
    pub design: CleanGraph,
    pub assets: CleanGraph,
    pub product: CleanGraph,
    pub i18n: CleanGraph,
}

pub fn build_domain_graphs(root: &Path, options: BuildOptions) -> CodeGraphResult<DomainGraphs> {
    let graph = build_clean_graph(root, options)?;
    Ok(DomainGraphs {
        design: project(&graph, Namespace::Design),
        assets: project(&graph, Namespace::Asset),
        product: project(&graph, Namespace::Product),
        i18n: project(&graph, Namespace::I18n),
    })
}

fn project(graph: &CleanGraph, namespace: Namespace) -> CleanGraph {
    let domain_ids = graph
        .nodes()
        .filter(|node| node.namespace == namespace)
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let boundary_ids = graph
        .edges()
        .filter(|edge| domain_ids.contains(&edge.source) || domain_ids.contains(&edge.target))
        .flat_map(|edge| [edge.source.clone(), edge.target.clone()])
        .filter(|id| graph.node(id).is_some_and(|node| node.kind == "file"))
        .collect::<BTreeSet<_>>();
    let mut projected = CleanGraph::default();
    for node in graph
        .nodes()
        .filter(|node| domain_ids.contains(&node.id) || boundary_ids.contains(&node.id))
    {
        projected.insert_node(node.clone());
    }
    let edges = graph
        .edges()
        .filter(|edge| {
            projected.node(&edge.source).is_some() && projected.node(&edge.target).is_some()
        })
        .cloned()
        .collect::<Vec<_>>();
    for edge in edges {
        let _ = projected.insert_edge(edge);
    }
    projected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::{GraphQuery, QueryKind};
    use std::fs;

    #[test]
    fn named_domain_projections_keep_source_boundaries_and_evidence() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("design")).unwrap();
        fs::create_dir_all(root.path().join("docs")).unwrap();
        fs::create_dir_all(root.path().join("i18n")).unwrap();
        fs::write(root.path().join("design/tokens.dowe"), "type Color\n").unwrap();
        fs::write(
            root.path().join("docs/product.md"),
            "# Product\n## Capability: Login\n### Use case: Sign in\n",
        )
        .unwrap();
        fs::write(
            root.path().join("i18n/en.json"),
            "{\"login\":{\"title\":\"Log in\"}}\n",
        )
        .unwrap();

        let domains = build_domain_graphs(root.path(), BuildOptions::default()).unwrap();
        assert!(
            domains
                .design
                .nodes()
                .any(|node| node.namespace == Namespace::Design)
        );
        assert!(domains.design.nodes().any(|node| node.kind == "file"));
        assert!(domains.assets.nodes().next().is_none());
        let use_cases = GraphQuery {
            kind: QueryKind::Search,
            value: "sign in".into(),
            namespace: Some(Namespace::Product),
            max_nodes: 8,
            max_depth: 0,
        }
        .execute(&domains.product);
        assert_eq!(use_cases.nodes.len(), 1);
        assert!(
            domains
                .i18n
                .nodes()
                .any(|node| node.kind == "translation_key")
        );
    }
}
