use crate::clean;
use crate::error::{CodeGraphError, CodeGraphResult};
use crate::metrics::fingerprint_bytes;
use crate::paths::{discover_files, slash_path};
use crate::{
    BuildOptions, CodeGraph, CodeGraphMode, Edge, Node,
    clean::{build_clean_graph, legacy_codegraph_from_clean},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

mod lock;
mod storage;
use storage::*;

const VERSION: u32 = 1;
const STORE: &str = ".dowe/codegraph";
const LEGACY_STORE: &str = ".agents/codegraph";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphManifest {
    pub version: u32,
    pub schema: u32,
    pub root: String,
    pub mode: CodeGraphMode,
    pub revision: u64,
    pub fingerprints: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphFreshness {
    Fresh,
    Initialized,
    Refreshed,
    Stale,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraphSnapshot {
    pub graph: CodeGraph,
    #[serde(default)]
    pub clean_graph: Option<clean::CleanGraph>,
    pub manifest: GraphManifest,
    pub freshness: GraphFreshness,
    pub changed: Vec<String>,
    pub deleted: Vec<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub generation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraphBinding {
    pub generation: String,
    pub revision: u64,
    pub root: String,
    pub mode: CodeGraphMode,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraphQuery {
    pub text: String,
    pub limit: usize,
    pub depth: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQueryResult {
    pub nodes: Vec<Node>,
    pub incoming: Vec<Edge>,
    pub outgoing: Vec<Edge>,
    pub impact: Vec<Node>,
}

pub fn ensure_persistent_codegraph(root: impl AsRef<Path>) -> CodeGraphResult<CodeGraphSnapshot> {
    refresh_persistent_codegraph(root)
}
pub fn read_persistent_codegraph(root: impl AsRef<Path>) -> CodeGraphResult<CodeGraphSnapshot> {
    let root = root
        .as_ref()
        .canonicalize()
        .map_err(|e| CodeGraphError::at_path(root.as_ref(), e.to_string()))?;
    let mode = crate::detect_codegraph_mode(&root)?;
    let _lock = lock::acquire(&root, false)?;
    let dir = [STORE, LEGACY_STORE]
        .iter()
        .map(|store| root.join(store))
        .find(|dir| dir.join("CURRENT").is_file())
        .ok_or_else(|| CodeGraphError::new("persisted CodeGraph generation is missing"))?;
    let mut snapshot = read_current(&dir, &root, mode)?;
    snapshot.freshness = if current_fingerprints(&root, mode)? == snapshot.manifest.fingerprints {
        GraphFreshness::Fresh
    } else {
        GraphFreshness::Stale
    };
    Ok(snapshot)
}

fn current_fingerprints(
    root: &Path,
    mode: CodeGraphMode,
) -> CodeGraphResult<BTreeMap<String, String>> {
    Ok(discover_files(root, mode)?
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).ok().map(slash_path)?;
            if relative.starts_with(".agents/plans/")
                || relative.starts_with(".agents/capabilities/")
                || relative.starts_with(".dowe/")
            {
                return None;
            }
            let bytes = fs::read(path).ok()?;
            Some((relative, fingerprint_bytes(&bytes)))
        })
        .collect())
}
pub fn refresh_persistent_codegraph(root: impl AsRef<Path>) -> CodeGraphResult<CodeGraphSnapshot> {
    let root = root
        .as_ref()
        .canonicalize()
        .map_err(|e| CodeGraphError::at_path(root.as_ref(), e.to_string()))?;
    let mode = crate::detect_codegraph_mode(&root)?;
    let current = store_dir(&root)?;
    let _lock = lock::acquire(&root, true)?;
    let had_current_generation = current.join("CURRENT").is_file();
    let files = discover_files(&root, mode)?;
    let fingerprints = files
        .iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(&root).ok().map(slash_path)?;
            if rel.starts_with(".agents/plans/") || rel.starts_with(".dowe/") {
                return None;
            }
            let bytes = fs::read(p).ok()?;
            Some((rel, fingerprint_bytes(&bytes)))
        })
        .collect::<BTreeMap<_, _>>();
    let old = if current.join("CURRENT").exists() {
        Some(read_current(&current, &root, mode)?)
    } else {
        [LEGACY_STORE]
            .iter()
            .map(|store| root.join(store))
            .find(|dir| dir.join("CURRENT").is_file())
            .map(|dir| read_current(&dir, &root, mode))
            .transpose()?
    };
    if let Some(snapshot) = old.as_ref() {
        if had_current_generation && snapshot.manifest.fingerprints == fingerprints {
            if let Some(generation) = snapshot.generation.as_deref() {
                prune_generations(&current, generation)?;
            }
            return Ok(CodeGraphSnapshot {
                freshness: GraphFreshness::Fresh,
                changed: vec![],
                deleted: vec![],
                ..snapshot.clone()
            });
        }
    }
    let previous = old.as_ref().map(|s| &s.manifest.fingerprints);
    let changed = fingerprints
        .iter()
        .filter(|(p, f)| previous.is_none_or(|o| o.get(*p) != Some(f)))
        .map(|(p, _)| p.clone())
        .collect::<Vec<_>>();
    let deleted: Vec<String> = previous
        .map(|o| {
            o.keys()
                .filter(|p| !fingerprints.contains_key(*p))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let extracted_clean = build_clean_graph(&root, BuildOptions { mode: Some(mode) })?;
    let extracted = legacy_codegraph_from_clean(&extracted_clean, mode, &root);
    let graph = old
        .as_ref()
        .map(|s| replace_file_contributions(s.graph.clone(), &extracted, &changed, &deleted))
        .unwrap_or(extracted);
    let revision = old.as_ref().map_or(1, |s| s.manifest.revision + 1);
    let manifest = GraphManifest {
        version: VERSION,
        schema: VERSION,
        root: root.to_string_lossy().into_owned(),
        mode,
        revision,
        fingerprints,
    };
    let generation = write_generation(&current, &manifest, &graph, &extracted_clean)?;
    Ok(CodeGraphSnapshot {
        generation: Some(generation),
        graph,
        clean_graph: Some(extracted_clean),
        manifest,
        freshness: if old.is_none() {
            GraphFreshness::Initialized
        } else {
            GraphFreshness::Refreshed
        },
        changed,
        deleted,
        error: None,
    })
}
pub fn replace_file_contributions(
    mut base: CodeGraph,
    replacement: &CodeGraph,
    changed: &[String],
    deleted: &[String],
) -> CodeGraph {
    let paths = changed.iter().chain(deleted).collect::<BTreeSet<_>>();
    let removed = base
        .nodes
        .iter()
        .filter(|n| n.path.as_ref().is_some_and(|p| paths.contains(p)))
        .map(|n| n.id.clone())
        .collect::<BTreeSet<_>>();
    base.nodes.retain(|n| !removed.contains(&n.id));
    base.edges
        .retain(|e| !removed.contains(&e.from) && !removed.contains(&e.to));
    let additions = replacement
        .nodes
        .iter()
        .filter(|n| n.path.as_ref().is_some_and(|p| paths.contains(p)))
        .cloned()
        .collect::<Vec<_>>();
    let ids = base
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .chain(additions.iter().map(|n| n.id.clone()))
        .collect::<BTreeSet<_>>();
    let affected = removed
        .iter()
        .cloned()
        .chain(additions.iter().map(|n| n.id.clone()))
        .collect::<BTreeSet<_>>();
    base.nodes.extend(additions);
    base.edges.extend(
        replacement
            .edges
            .iter()
            .filter(|e| {
                ids.contains(&e.from)
                    && ids.contains(&e.to)
                    && (affected.contains(&e.from) || affected.contains(&e.to))
            })
            .cloned(),
    );
    base.nodes.sort_by(|a, b| a.id.cmp(&b.id));
    base.edges
        .sort_by(|a, b| (a.from.clone(), a.to.clone()).cmp(&(b.from.clone(), b.to.clone())));
    base
}

pub fn query_persistent_codegraph(
    root: impl AsRef<Path>,
    query: CodeGraphQuery,
) -> CodeGraphResult<GraphQueryResult> {
    let root = root.as_ref().canonicalize()?;
    let _snapshot = refresh_persistent_codegraph(&root)?;
    let clean = crate::clean::build_clean_graph(&root, BuildOptions::default())?;
    let result = crate::clean::GraphQuery {
        kind: crate::clean::QueryKind::Search,
        value: query.text,
        namespace: None,
        max_nodes: query.limit.min(128),
        max_depth: query.depth.min(8),
    }
    .execute(&clean);
    let legacy = crate::clean::legacy_codegraph_from_clean(
        &clean,
        crate::detect_codegraph_mode(&root)?,
        &root,
    );
    let selected = result
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let nodes = result
        .nodes
        .iter()
        .filter_map(|node| legacy.nodes.iter().find(|legacy| legacy.id == node.id))
        .cloned()
        .collect();
    let incoming = legacy
        .edges
        .iter()
        .filter(|edge| selected.contains(edge.to.as_str()))
        .cloned()
        .collect();
    let outgoing = legacy
        .edges
        .iter()
        .filter(|edge| selected.contains(edge.from.as_str()))
        .cloned()
        .collect();
    let impact_result = crate::clean::GraphQuery {
        kind: crate::clean::QueryKind::Impact,
        value: result
            .nodes
            .first()
            .map(|node| node.id.clone())
            .unwrap_or_default(),
        namespace: None,
        max_nodes: query.limit.min(128),
        max_depth: query.depth.min(8),
    }
    .execute(&clean);
    let impact = impact_result
        .nodes
        .iter()
        .filter_map(|node| legacy.nodes.iter().find(|legacy| legacy.id == node.id))
        .cloned()
        .collect();
    Ok(GraphQueryResult {
        nodes,
        incoming,
        outgoing,
        impact,
    })
}
