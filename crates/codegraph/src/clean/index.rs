use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Namespace {
    Frontend,
    Backend,
    Shared,
    Product,
    Design,
    Asset,
    I18n,
    Contract,
    Verification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Compiler,
    Durable,
    Inferred,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanNode {
    pub id: String,
    pub namespace: Namespace,
    pub kind: String,
    pub name: String,
    pub path: Option<String>,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanEdge {
    pub source: String,
    pub relation: String,
    pub target: String,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanGraph {
    nodes: BTreeMap<String, CleanNode>,
    outgoing: BTreeMap<String, Vec<CleanEdge>>,
    incoming: BTreeMap<String, Vec<CleanEdge>>,
}

impl CleanGraph {
    pub fn insert_node(&mut self, node: CleanNode) -> bool {
        self.nodes.insert(node.id.clone(), node).is_none()
    }

    pub fn insert_edge(&mut self, edge: CleanEdge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.source) || !self.nodes.contains_key(&edge.target) {
            return Err("edge endpoints must already exist".into());
        }
        if self
            .outgoing
            .get(&edge.source)
            .is_some_and(|edges| edges.iter().any(|existing| existing == &edge))
        {
            return Ok(());
        }
        self.outgoing
            .entry(edge.source.clone())
            .or_default()
            .push(edge.clone());
        self.incoming
            .entry(edge.target.clone())
            .or_default()
            .push(edge);
        Ok(())
    }

    pub fn node(&self, id: &str) -> Option<&CleanNode> {
        self.nodes.get(id)
    }
    pub fn nodes(&self) -> impl Iterator<Item = &CleanNode> {
        self.nodes.values()
    }
    pub fn edges_from(&self, id: &str) -> impl Iterator<Item = &CleanEdge> {
        self.outgoing.get(id).into_iter().flatten()
    }
    pub fn edges_to(&self, id: &str) -> impl Iterator<Item = &CleanEdge> {
        self.incoming.get(id).into_iter().flatten()
    }
    pub fn edges(&self) -> impl Iterator<Item = &CleanEdge> {
        self.outgoing.values().flatten()
    }
    pub fn ids(&self) -> BTreeSet<String> {
        self.nodes.keys().cloned().collect()
    }
}
