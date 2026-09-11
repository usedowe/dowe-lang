use crate::error::AgentResult;
use dowe_codegraph::{
    CodeGraphMode, CodeGraphQuery, GraphFreshness, NodeKind, ensure_persistent_codegraph,
    query_persistent_codegraph,
};
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
}

pub fn summarize_codegraph(
    root: impl AsRef<Path>,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    summarize_codegraph_inner(root.as_ref(), None, max_nodes)
}

pub fn summarize_codegraph_for(
    root: impl AsRef<Path>,
    query: &str,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    summarize_codegraph_inner(root.as_ref(), Some(query), max_nodes)
}

fn summarize_codegraph_inner(
    root: &Path,
    query: Option<&str>,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    match ensure_persistent_codegraph(root) {
        Ok(snapshot) => {
            let freshness = format!("{:?}", snapshot.freshness).to_ascii_lowercase();
            let stale = matches!(
                snapshot.freshness,
                GraphFreshness::Stale | GraphFreshness::Error
            );
            let graph = snapshot.graph;
            let visible_node_ids = graph
                .nodes
                .iter()
                .filter(|node| visible_in_agent_context(node.path.as_deref()))
                .map(|node| node.id.clone())
                .collect::<BTreeSet<_>>();
            let navigation = query_persistent_codegraph(
                root,
                CodeGraphQuery {
                    text: query.unwrap_or_default().to_string(),
                    limit: max_nodes,
                    depth: 2,
                },
            )
            .ok();
            let navigation_nodes = navigation
                .as_ref()
                .map(|r| {
                    r.nodes
                        .iter()
                        .filter(|node| visible_in_agent_context(node.path.as_deref()))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let navigation_edges = navigation
                .as_ref()
                .map(|r| {
                    r.incoming
                        .iter()
                        .chain(r.outgoing.iter())
                        .filter(|edge| {
                            visible_node_ids.contains(&edge.from)
                                && visible_node_ids.contains(&edge.to)
                        })
                        .map(|e| format!("{} -> {} ({:?})", e.from, e.to, e.kind))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let impact_nodes = navigation
                .as_ref()
                .map(|r| {
                    r.impact
                        .iter()
                        .filter(|node| visible_in_agent_context(node.path.as_deref()))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let mode = match graph.mode {
                CodeGraphMode::Dowe => "dowe",
                CodeGraphMode::Project => "project",
            }
            .to_string();
            let terms = query.map(search_terms).unwrap_or_default();
            let mut scored = graph
                .nodes
                .iter()
                .filter_map(|node| {
                    let score = if query.is_some() {
                        node_score(
                            &terms,
                            &node.kind,
                            node.path.as_deref(),
                            &node.name,
                            node.owner.as_deref(),
                        )
                    } else {
                        usize::from(relevant_node(&node.kind, node.path.as_deref()))
                    };
                    (score > 0).then(|| {
                        (
                            score,
                            AgentCodeGraphNodeSummary {
                                kind: format!("{:?}", node.kind).to_ascii_lowercase(),
                                language: node.language.clone(),
                                path: node.path.clone(),
                                name: node.name.clone(),
                                owner: node.owner.clone(),
                                total_lines: node
                                    .metrics
                                    .as_ref()
                                    .map(|metrics| metrics.total_lines),
                            },
                        )
                    })
                })
                .collect::<Vec<_>>();
            scored.sort_by(|(left_score, left), (right_score, right)| {
                right_score.cmp(left_score).then_with(|| {
                    (
                        left.path.as_deref().unwrap_or_default(),
                        &left.name,
                        left.owner.as_deref().unwrap_or_default(),
                    )
                        .cmp(&(
                            right.path.as_deref().unwrap_or_default(),
                            &right.name,
                            right.owner.as_deref().unwrap_or_default(),
                        ))
                })
            });
            let nodes = scored
                .into_iter()
                .take(max_nodes)
                .map(|(_, node)| node)
                .collect();

            Ok(AgentCodeGraphSummary {
                mode,
                node_count: graph.nodes.len(),
                edge_count: graph.edges.len(),
                relevant_nodes: nodes,
                navigation: navigation_nodes
                    .into_iter()
                    .map(node_summary)
                    .take(max_nodes)
                    .collect(),
                impact: impact_nodes
                    .into_iter()
                    .map(node_summary)
                    .take(max_nodes)
                    .collect(),
                navigation_edges: navigation_edges.into_iter().take(max_nodes * 2).collect(),
                edge_policy: "derived_only_existing_extractors".to_string(),
                error: snapshot.error,
                freshness,
                revision: snapshot.manifest.revision,
                stale,
            })
        }
        Err(error) => Ok(AgentCodeGraphSummary {
            mode: "unknown".to_string(),
            node_count: 0,
            edge_count: 0,
            relevant_nodes: Vec::new(),
            navigation: Vec::new(),
            impact: Vec::new(),
            navigation_edges: Vec::new(),
            edge_policy: "derived_only_existing_extractors".to_string(),
            error: Some(error.to_string()),
            freshness: "error".to_string(),
            revision: 0,
            stale: true,
        }),
    }
}

fn visible_in_agent_context(path: Option<&str>) -> bool {
    path.is_none_or(|path| {
        !matches!(path, ".agents" | "agents" | ".dowe" | "dowe")
            && !path.starts_with(".agents/")
            && !path.starts_with("agents/")
            && !path.starts_with(".dowe/")
            && !path.starts_with("dowe/")
    })
}

fn node_summary(node: dowe_codegraph::Node) -> AgentCodeGraphNodeSummary {
    AgentCodeGraphNodeSummary {
        kind: format!("{:?}", node.kind).to_ascii_lowercase(),
        language: node.language,
        path: node.path,
        name: node.name,
        owner: node.owner,
        total_lines: node.metrics.map(|m| m.total_lines),
    }
}

fn search_terms(query: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "and",
        "the",
        "for",
        "with",
        "update",
        "implement",
        "create",
        "change",
        "add",
        "una",
        "para",
        "con",
        "actualiza",
        "implementa",
        "crea",
        "cambia",
        "agrega",
    ];
    query
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|term| term.len() > 2 && !STOP_WORDS.contains(term))
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn node_score(
    terms: &[String],
    kind: &NodeKind,
    path: Option<&str>,
    name: &str,
    owner: Option<&str>,
) -> usize {
    let path = path.unwrap_or_default().to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    let owner = owner.unwrap_or_default().to_ascii_lowercase();
    let mut score = terms
        .iter()
        .map(|term| {
            usize::from(path.contains(term)) * 8
                + usize::from(name.contains(term)) * 6
                + usize::from(owner.contains(term)) * 3
        })
        .sum();
    if matches!(path.as_str(), "main.dowe" | "theme.dowe") {
        score += 1;
    }
    if matches!(
        kind,
        NodeKind::Spec | NodeKind::Contract | NodeKind::Acceptance
    ) && terms
        .iter()
        .any(|term| matches!(term.as_str(), "spec" | "contract" | "acceptance"))
    {
        score += 2;
    }
    score
}

fn relevant_node(kind: &NodeKind, path: Option<&str>) -> bool {
    match kind {
        NodeKind::Crate | NodeKind::Spec | NodeKind::Contract | NodeKind::Acceptance => true,
        NodeKind::File => path.is_some_and(|path| {
            path.ends_with(".dowe")
                || path.contains("agent")
                || path.contains("codegraph")
                || path.contains("cli/src/agent")
                || path.contains("docs/development")
                || path.contains("docs/server")
                || path.contains("dowe-llm")
        }),
        _ => false,
    }
}
