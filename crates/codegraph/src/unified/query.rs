use super::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const MAX_NODES: usize = 128;
const MAX_EDGES: usize = 512;
const MAX_VISITED: usize = 4096;

impl UnifiedCodeGraph {
    pub fn get_node(&self, id: &str) -> Option<&KnowledgeNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn search(
        &self,
        query: &str,
        scope: UnifiedScope,
        limit: usize,
    ) -> UnifiedGraphQueryResult {
        self.query(&UnifiedGraphQuery {
            query: query.into(),
            scope,
            limit,
            depth: 0,
        })
    }

    pub fn query(&self, request: &UnifiedGraphQuery) -> UnifiedGraphQueryResult {
        let terms: Vec<_> = request
            .query
            .split_whitespace()
            .take(32)
            .map(str::to_lowercase)
            .collect();
        let mut matches: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| scope_matches(request.scope, n.namespace))
            .filter_map(|node| {
                let text = format!(
                    "{} {} {} {}",
                    node.id,
                    node.name,
                    node.path.as_deref().unwrap_or(""),
                    node.summary.as_deref().unwrap_or("")
                )
                .to_lowercase();
                let score = terms
                    .iter()
                    .filter(|term| text.contains(term.as_str()))
                    .count();
                (terms.is_empty() || score > 0).then_some((score, node))
            })
            .collect();
        matches.sort_by(|(a, x), (b, y)| b.cmp(a).then(x.id.cmp(&y.id)));
        let limit = request.limit.clamp(1, MAX_NODES);
        let truncated = matches.len() > limit;
        let nodes = matches
            .into_iter()
            .take(limit)
            .map(|(_, n)| n.clone())
            .collect();
        self.result(
            nodes,
            request.depth,
            limit.saturating_mul(4).min(MAX_NODES),
            false,
            truncated,
            request.scope,
        )
    }

    pub fn get_related(&self, id: &str, depth: usize, limit: usize) -> UnifiedGraphQueryResult {
        let nodes = self.get_node(id).cloned().into_iter().collect();
        self.result(
            nodes,
            depth,
            limit.clamp(1, MAX_NODES),
            false,
            false,
            UnifiedScope::All,
        )
    }

    pub fn get_dependencies(&self, id: &str, limit: usize) -> Vec<KnowledgeNode> {
        self.query_dependencies(id, limit).nodes
    }

    pub fn get_consumers(&self, id: &str, limit: usize) -> Vec<KnowledgeNode> {
        self.query_consumers(id, limit).nodes
    }

    pub fn get_impact(&self, id: &str, depth: usize, limit: usize) -> Vec<KnowledgeNode> {
        self.query_impact(id, depth, limit).nodes
    }

    pub fn query_dependencies(&self, id: &str, limit: usize) -> UnifiedNodeQueryResult {
        self.neighbors(id, false, limit)
    }

    pub fn query_consumers(&self, id: &str, limit: usize) -> UnifiedNodeQueryResult {
        self.neighbors(id, true, limit)
    }

    pub fn query_impact(&self, id: &str, depth: usize, limit: usize) -> UnifiedNodeQueryResult {
        let nodes = self.get_node(id).cloned().into_iter().collect();
        let result = self.result(
            nodes,
            depth,
            limit.clamp(1, MAX_NODES),
            true,
            false,
            UnifiedScope::All,
        );
        UnifiedNodeQueryResult {
            nodes: result.impact,
            truncated: result.truncated,
        }
    }

    fn neighbors(&self, id: &str, reverse: bool, limit: usize) -> UnifiedNodeQueryResult {
        if self.get_node(id).is_none() {
            return UnifiedNodeQueryResult {
                nodes: Vec::new(),
                truncated: false,
            };
        }
        let ids: BTreeSet<_> = self
            .edges
            .iter()
            .filter(|edge| is_dependency(edge.relation))
            .filter_map(|e| match reverse {
                true if e.to == id => Some(e.from.as_str()),
                false if e.from == id => Some(e.to.as_str()),
                _ => None,
            })
            .collect();
        let limit = limit.clamp(1, MAX_NODES);
        let mut nodes: Vec<_> = ids
            .into_iter()
            .filter_map(|id| self.get_node(id))
            .take(limit + 1)
            .cloned()
            .collect();
        let truncated = nodes.len() > limit;
        nodes.truncate(limit);
        UnifiedNodeQueryResult { nodes, truncated }
    }

    fn result(
        &self,
        nodes: Vec<KnowledgeNode>,
        depth: usize,
        limit: usize,
        reverse: bool,
        mut truncated: bool,
        scope: UnifiedScope,
    ) -> UnifiedGraphQueryResult {
        let lookup: BTreeMap<_, _> = self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &self.edges {
            if reverse && !is_dependency(edge.relation) {
                continue;
            }
            if !lookup.contains_key(edge.from.as_str()) || !lookup.contains_key(edge.to.as_str()) {
                continue;
            }
            adjacency.entry(&edge.to).or_default().push(&edge.from);
            if !reverse {
                adjacency.entry(&edge.from).or_default().push(&edge.to);
            }
        }
        for values in adjacency.values_mut() {
            values.sort();
            values.dedup();
        }
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        for node in &nodes {
            seen.insert(node.id.as_str());
            queue.push_back((node.id.as_str(), 0));
        }
        let mut impact = Vec::new();
        let mut frontier = BTreeSet::new();
        while let Some((id, level)) = queue.pop_front() {
            if impact.len() == limit {
                truncated = true;
                break;
            }
            if let Some(node) = lookup.get(id) {
                if scope_matches(scope, node.namespace) {
                    impact.push((*node).clone());
                }
            }
            if level >= depth.min(8) {
                frontier.extend(adjacency.get(id).into_iter().flatten().copied());
                continue;
            }
            for next in adjacency.get(id).into_iter().flatten() {
                if seen.len() >= MAX_VISITED {
                    truncated = true;
                    break;
                }
                if lookup
                    .get(next)
                    .is_some_and(|node| scope_matches(scope, node.namespace))
                    && seen.insert(next)
                {
                    queue.push_back((next, level + 1));
                }
            }
        }
        truncated |= (reverse || depth > 0) && frontier.iter().any(|id| !seen.contains(id));
        let selected: BTreeSet<_> = nodes.iter().map(|n| n.id.as_str()).collect();
        let mut edges: Vec<_> = self
            .edges
            .iter()
            .filter(|e| selected.contains(e.from.as_str()) || selected.contains(e.to.as_str()))
            .take(MAX_EDGES + 1)
            .cloned()
            .collect();
        truncated |= edges.len() > MAX_EDGES || depth > 8;
        edges.truncate(MAX_EDGES);
        UnifiedGraphQueryResult {
            nodes,
            edges,
            impact,
            truncated,
        }
    }
}

fn is_dependency(relation: KnowledgeRelation) -> bool {
    !matches!(
        relation,
        KnowledgeRelation::Contains | KnowledgeRelation::Provides
    )
}

fn scope_matches(scope: UnifiedScope, namespace: KnowledgeNamespace) -> bool {
    matches!(scope, UnifiedScope::All)
        || matches!(
            (scope, namespace),
            (UnifiedScope::Frontend, KnowledgeNamespace::Frontend)
                | (UnifiedScope::Backend, KnowledgeNamespace::Backend)
                | (UnifiedScope::Shared, KnowledgeNamespace::Shared)
                | (UnifiedScope::Product, KnowledgeNamespace::Product)
                | (UnifiedScope::Design, KnowledgeNamespace::Design)
                | (UnifiedScope::Asset, KnowledgeNamespace::Asset)
                | (UnifiedScope::I18n, KnowledgeNamespace::I18n)
                | (UnifiedScope::Contract, KnowledgeNamespace::Contract)
                | (UnifiedScope::Verification, KnowledgeNamespace::Verification)
        )
}
