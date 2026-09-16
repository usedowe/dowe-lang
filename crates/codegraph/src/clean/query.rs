use super::{CleanEdge, CleanGraph, CleanNode, Namespace};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryKind {
    Search,
    Exact,
    Dependencies,
    Consumers,
    Related,
    Impact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQuery {
    pub kind: QueryKind,
    pub value: String,
    pub namespace: Option<Namespace>,
    pub max_nodes: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphResult {
    pub nodes: Vec<CleanNode>,
    pub edges: Vec<CleanEdge>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanExplanation {
    pub node: CleanNode,
    pub incoming: Vec<CleanEdge>,
    pub outgoing: Vec<CleanEdge>,
}

pub fn explain_node(graph: &CleanGraph, selector: &str) -> Option<CleanExplanation> {
    let node = graph
        .nodes()
        .find(|node| {
            node.id == selector || node.name == selector || node.path.as_deref() == Some(selector)
        })?
        .clone();
    Some(CleanExplanation {
        incoming: graph.edges_to(&node.id).cloned().collect(),
        outgoing: graph.edges_from(&node.id).cloned().collect(),
        node,
    })
}

impl GraphQuery {
    pub fn execute(&self, graph: &CleanGraph) -> GraphResult {
        let limit = self.max_nodes.max(1);
        match self.kind {
            QueryKind::Exact => GraphResult {
                nodes: graph.node(&self.value).cloned().into_iter().collect(),
                edges: Vec::new(),
                truncated: false,
            },
            QueryKind::Search => {
                let needle = self.value.to_lowercase();
                let mut nodes = Vec::new();
                for node in graph.nodes().filter(|node| {
                    self.namespace
                        .is_none_or(|namespace| namespace == node.namespace)
                }) {
                    if node.id.to_lowercase().contains(&needle)
                        || node.name.to_lowercase().contains(&needle)
                        || node.kind.to_lowercase().contains(&needle)
                    {
                        nodes.push(node.clone());
                        if nodes.len() == limit {
                            break;
                        }
                    }
                }
                let total = graph
                    .nodes()
                    .filter(|node| {
                        self.namespace
                            .is_none_or(|namespace| namespace == node.namespace)
                            && (node.id.to_lowercase().contains(&needle)
                                || node.name.to_lowercase().contains(&needle)
                                || node.kind.to_lowercase().contains(&needle))
                    })
                    .count();
                GraphResult {
                    nodes,
                    edges: Vec::new(),
                    truncated: total > limit,
                }
            }
            QueryKind::Dependencies => {
                traverse(graph, &self.value, self.max_depth.max(1), limit, false)
            }
            QueryKind::Consumers => {
                traverse(graph, &self.value, self.max_depth.max(1), limit, true)
            }
            QueryKind::Related => {
                traverse_related(graph, &self.value, self.max_depth.max(1), limit)
            }
            QueryKind::Impact => traverse(graph, &self.value, self.max_depth, limit, true),
        }
    }
}

fn traverse_related(graph: &CleanGraph, root: &str, max_depth: usize, limit: usize) -> GraphResult {
    let mut queue = VecDeque::from([(root.to_string(), 0usize)]);
    let mut seen = BTreeSet::from([root.to_string()]);
    let mut nodes = Vec::new();
    let mut selected_edges = Vec::new();
    let mut truncated = false;
    while let Some((id, depth)) = queue.pop_front() {
        if depth >= max_depth {
            truncated |=
                graph.edges_from(&id).next().is_some() || graph.edges_to(&id).next().is_some();
            continue;
        }
        let neighbors = graph
            .edges_from(&id)
            .chain(graph.edges_to(&id))
            .collect::<Vec<_>>();
        for edge in neighbors {
            let next = if edge.source == id {
                &edge.target
            } else {
                &edge.source
            };
            if !seen.insert(next.clone()) {
                continue;
            }
            let Some(node) = graph.node(next) else {
                continue;
            };
            if nodes.len() == limit {
                truncated = true;
                break;
            }
            nodes.push(node.clone());
            selected_edges.push(edge.clone());
            queue.push_back((next.clone(), depth + 1));
        }
        if truncated {
            break;
        }
    }
    GraphResult {
        nodes,
        edges: selected_edges,
        truncated,
    }
}

fn traverse(
    graph: &CleanGraph,
    root: &str,
    max_depth: usize,
    limit: usize,
    reverse: bool,
) -> GraphResult {
    let mut queue = VecDeque::from([(root.to_string(), 0usize)]);
    let mut seen = BTreeSet::from([root.to_string()]);
    let mut nodes = Vec::new();
    let mut selected_edges = Vec::new();
    let mut truncated = false;
    while let Some((id, depth)) = queue.pop_front() {
        if depth >= max_depth {
            let has_more = if reverse {
                graph.edges_to(&id).next().is_some()
            } else {
                graph.edges_from(&id).next().is_some()
            };
            truncated |= has_more;
            continue;
        }
        let edges = if reverse {
            graph.edges_to(&id).collect::<Vec<_>>()
        } else {
            graph.edges_from(&id).collect::<Vec<_>>()
        };
        for edge in edges {
            let next = if reverse { &edge.source } else { &edge.target };
            if !seen.insert(next.clone()) {
                continue;
            }
            if let Some(node) = graph.node(next) {
                if nodes.len() == limit {
                    truncated = true;
                    break;
                }
                nodes.push(node.clone());
                selected_edges.push(edge.clone());
                queue.push_back((next.clone(), depth + 1));
            }
        }
        if truncated {
            break;
        }
    }
    GraphResult {
        nodes,
        edges: selected_edges,
        truncated,
    }
}
