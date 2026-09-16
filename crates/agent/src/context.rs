use crate::error::AgentResult;
use dowe_codegraph::clean::{CleanGraph, CleanNode, GraphQuery, QueryKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCodeGraphSummary {
    pub mode: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub relevant_nodes: Vec<AgentCodeGraphNodeSummary>,
    pub navigation: Vec<AgentCodeGraphNodeSummary>,
    pub impact: Vec<AgentCodeGraphNodeSummary>,
    pub navigation_truncated: bool,
    pub impact_truncated: bool,
    pub navigation_edges: Vec<String>,
    pub edge_policy: String,
    pub error: Option<String>,
    pub freshness: String,
    pub revision: u64,
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCodeGraphNodeSummary {
    pub kind: String,
    pub language: String,
    pub path: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    pub total_lines: Option<usize>,
    pub evidence: String,
}

pub fn summarize_codegraph(
    root: impl AsRef<Path>,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    summarize_clean_codegraph(root.as_ref(), None, max_nodes)
}

pub fn summarize_codegraph_for(
    root: impl AsRef<Path>,
    query: &str,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    summarize_clean_codegraph(root.as_ref(), Some(query), max_nodes)
}

fn summarize_clean_codegraph(
    root: &Path,
    query: Option<&str>,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    let (graph, freshness, revision, stale) =
        match dowe_codegraph::clean::read_persistent_clean_codegraph(root) {
            Ok(snapshot) => {
                let stale = matches!(snapshot.freshness, dowe_codegraph::GraphFreshness::Stale);
                let graph = if stale {
                    dowe_codegraph::clean::build_clean_graph(root, Default::default())
                        .map_err(|error| crate::AgentError::new(error.to_string()))?
                } else {
                    snapshot.graph
                };
                (
                    graph,
                    format!("{:?}", snapshot.freshness).to_lowercase(),
                    snapshot.manifest.revision,
                    stale,
                )
            }
            Err(_) => (
                dowe_codegraph::clean::build_clean_graph(root, Default::default())
                    .map_err(|error| crate::AgentError::new(error.to_string()))?,
                "ephemeral".into(),
                0,
                false,
            ),
        };
    let limit = max_nodes.max(1);
    let navigation_result = query_nodes(&graph, query, limit);
    let navigation_truncated = navigation_result.truncated;
    let navigation = navigation_result.nodes;
    let impact_result = navigation
        .first()
        .map(|node| {
            GraphQuery {
                kind: QueryKind::Impact,
                value: node.id.clone(),
                namespace: None,
                max_nodes: limit,
                max_depth: 2,
            }
            .execute(&graph)
        })
        .unwrap_or_else(|| dowe_codegraph::clean::GraphResult {
            nodes: Vec::new(),
            edges: Vec::new(),
            truncated: false,
        });
    let navigation_ids = navigation
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let navigation_edges = graph
        .edges()
        .filter(|edge| {
            navigation_ids.contains(edge.source.as_str())
                || navigation_ids.contains(edge.target.as_str())
        })
        .take(limit * 2)
        .map(|edge| format!("{} -> {} ({})", edge.source, edge.target, edge.relation))
        .collect();
    Ok(AgentCodeGraphSummary {
        mode: "clean".into(),
        node_count: graph.nodes().count(),
        edge_count: graph.edges().count(),
        relevant_nodes: navigation.iter().map(node_summary).collect(),
        navigation: navigation.iter().map(node_summary).collect(),
        impact: impact_result.nodes.iter().map(node_summary).collect(),
        navigation_truncated,
        impact_truncated: impact_result.truncated,
        navigation_edges,
        edge_policy: "clean_bounded_graph".into(),
        error: None,
        freshness,
        revision,
        stale,
    })
}

fn query_nodes(
    graph: &CleanGraph,
    query: Option<&str>,
    limit: usize,
) -> dowe_codegraph::clean::GraphResult {
    let Some(query) = query else {
        let nodes = graph.nodes().take(limit).cloned().collect::<Vec<_>>();
        return dowe_codegraph::clean::GraphResult {
            truncated: graph.nodes().nth(limit).is_some(),
            nodes,
            edges: Vec::new(),
        };
    };

    // Prompts are intent descriptions, not exact node selectors. Use the
    // deterministic graph search for each meaningful term and rank nodes by
    // the number of matching terms so the agent receives focused context.
    let terms = query
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .map(str::trim)
        .filter(|term| term.chars().count() >= 3)
        .map(str::to_lowercase)
        .collect::<BTreeSet<_>>();
    if terms.is_empty() {
        return dowe_codegraph::clean::GraphResult {
            nodes: graph.nodes().take(limit).cloned().collect(),
            edges: Vec::new(),
            truncated: false,
        };
    }

    let mut scored = graph
        .nodes()
        .filter_map(|node| {
            let haystack = format!("{} {} {}", node.id, node.name, node.kind).to_lowercase();
            let score = terms
                .iter()
                .filter(|term| haystack.contains(term.as_str()))
                .count();
            (score > 0).then_some((score, node))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.id.cmp(&right.id))
    });
    let total_matches = scored.len();
    let nodes = scored
        .into_iter()
        .take(limit)
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();

    // Preserve the old exact-search behavior for unusual selectors that do
    // not tokenize cleanly, while still keeping the result bounded.
    if nodes.is_empty() {
        return GraphQuery {
            kind: QueryKind::Search,
            value: query.into(),
            namespace: None,
            max_nodes: limit,
            max_depth: 2,
        }
        .execute(graph);
    }
    dowe_codegraph::clean::GraphResult {
        nodes,
        edges: Vec::new(),
        truncated: total_matches > limit,
    }
}

fn node_summary(node: &CleanNode) -> AgentCodeGraphNodeSummary {
    AgentCodeGraphNodeSummary {
        kind: node.kind.clone(),
        language: node
            .path
            .as_deref()
            .and_then(|path| path.rsplit_once('.').map(|(_, extension)| extension.into()))
            .unwrap_or_else(|| "unknown".into()),
        path: node.path.clone(),
        name: node.name.clone(),
        owner: None,
        total_lines: node
            .start_line
            .zip(node.end_line)
            .map(|(start, end)| end.saturating_sub(start) as usize + 1),
        evidence: serde_json::to_string(&node.evidence)
            .unwrap_or_else(|_| "unknown".into())
            .trim_matches('"')
            .to_string(),
    }
}
