use crate::error::AgentResult;
use dowe_codegraph::{BuildOptions, CodeGraphMode, NodeKind, build_codegraph};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCodeGraphSummary {
    pub mode: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub relevant_nodes: Vec<AgentCodeGraphNodeSummary>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCodeGraphNodeSummary {
    pub kind: String,
    pub path: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    pub total_lines: Option<usize>,
}

pub fn summarize_codegraph(
    root: impl AsRef<Path>,
    max_nodes: usize,
) -> AgentResult<AgentCodeGraphSummary> {
    let root = root.as_ref();
    match build_codegraph(root, BuildOptions::default()) {
        Ok(graph) => {
            let mode = match graph.mode {
                CodeGraphMode::Dowe => "dowe",
                CodeGraphMode::Project => "project",
            }
            .to_string();
            let mut nodes = graph
                .nodes
                .iter()
                .filter(|node| relevant_node(&node.kind, node.path.as_deref()))
                .map(|node| AgentCodeGraphNodeSummary {
                    kind: format!("{:?}", node.kind).to_ascii_lowercase(),
                    path: node.path.clone(),
                    name: node.name.clone(),
                    owner: node.owner.clone(),
                    total_lines: node.metrics.as_ref().map(|metrics| metrics.total_lines),
                })
                .collect::<Vec<_>>();
            nodes.sort_by(|left, right| {
                (
                    left.owner.as_deref().unwrap_or_default(),
                    left.path.as_deref().unwrap_or_default(),
                    &left.name,
                )
                    .cmp(&(
                        right.owner.as_deref().unwrap_or_default(),
                        right.path.as_deref().unwrap_or_default(),
                        &right.name,
                    ))
            });
            nodes.truncate(max_nodes);

            Ok(AgentCodeGraphSummary {
                mode,
                node_count: graph.nodes.len(),
                edge_count: graph.edges.len(),
                relevant_nodes: nodes,
                error: None,
            })
        }
        Err(error) => Ok(AgentCodeGraphSummary {
            mode: "unknown".to_string(),
            node_count: 0,
            edge_count: 0,
            relevant_nodes: Vec::new(),
            error: Some(error.to_string()),
        }),
    }
}

fn relevant_node(kind: &NodeKind, path: Option<&str>) -> bool {
    match kind {
        NodeKind::Crate | NodeKind::Spec | NodeKind::Contract | NodeKind::Acceptance => true,
        NodeKind::File => path.is_some_and(|path| {
            path.contains("agent")
                || path.contains("codegraph")
                || path.contains("cli/src/agent")
                || path.contains("docs/development")
                || path.contains("docs/server")
                || path.contains("dowe-llm")
        }),
        _ => false,
    }
}
