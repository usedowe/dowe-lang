use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationNode {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct VerificationGraph {
    #[serde(default)]
    pub nodes: Vec<VerificationNode>,
}

impl VerificationGraph {
    pub fn validate(&self) -> HarnessResult<()> {
        if self.nodes.len() > 1024 {
            return Err(HarnessError::new("verification graph has too many nodes"));
        }
        let mut ids = BTreeSet::new();
        for node in &self.nodes {
            if node.id.is_empty() || node.id.len() > 128 || !ids.insert(node.id.clone()) {
                return Err(HarnessError::new(
                    "verification node ids must be unique and bounded",
                ));
            }
            if node.kind.is_empty() || node.kind.len() > 64 || node.evidence.len() > 2048 {
                return Err(HarnessError::new(
                    "verification node metadata is out of bounds",
                ));
            }
        }
        let mut edges = BTreeMap::new();
        for node in &self.nodes {
            if node.depends_on.len() > 64 || node.depends_on.iter().any(|id| !ids.contains(id)) {
                return Err(HarnessError::new(
                    "verification dependency references an unknown node",
                ));
            }
            edges.insert(node.id.clone(), node.depends_on.clone());
        }
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for id in &ids {
            visit(id, &edges, &mut visiting, &mut visited)?;
        }
        Ok(())
    }

    pub fn ordered_nodes(&self) -> HarnessResult<Vec<&VerificationNode>> {
        self.validate()?;
        let mut nodes = BTreeMap::new();
        for node in &self.nodes {
            nodes.insert(node.id.as_str(), node);
        }
        let mut ordered = Vec::with_capacity(self.nodes.len());
        let mut visited = BTreeSet::new();
        for node in &self.nodes {
            visit_order(node.id.as_str(), &nodes, &mut visited, &mut ordered)?;
        }
        Ok(ordered)
    }
}

fn visit_order<'a>(
    id: &str,
    nodes: &BTreeMap<&str, &'a VerificationNode>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<&'a VerificationNode>,
) -> HarnessResult<()> {
    if visited.contains(id) {
        return Ok(());
    }
    let node = nodes
        .get(id)
        .ok_or_else(|| HarnessError::new("verification dependency references an unknown node"))?;
    for dependency in &node.depends_on {
        visit_order(dependency, nodes, visited, ordered)?;
    }
    visited.insert(id.to_string());
    ordered.push(*node);
    Ok(())
}

fn visit(
    id: &str,
    edges: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> HarnessResult<()> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id.to_string()) {
        return Err(HarnessError::new("verification graph contains a cycle"));
    }
    if let Some(dependencies) = edges.get(id) {
        for dependency in dependencies {
            visit(dependency, edges, visiting, visited)?;
        }
    }
    visiting.remove(id);
    visited.insert(id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verification_graph_rejects_cycles() {
        let graph = VerificationGraph {
            nodes: vec![
                VerificationNode {
                    id: "a".into(),
                    kind: "test".into(),
                    depends_on: vec!["b".into()],
                    evidence: "a".into(),
                },
                VerificationNode {
                    id: "b".into(),
                    kind: "review".into(),
                    depends_on: vec!["a".into()],
                    evidence: "b".into(),
                },
            ],
        };
        assert!(graph.validate().is_err());
    }
}
