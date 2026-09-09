use crate::error::{CodeGraphError, CodeGraphResult};
use crate::metrics::fingerprint_bytes;
use crate::paths::{discover_files, slash_path};
use crate::{build_codegraph, BuildOptions, CodeGraph, CodeGraphMode, Edge, Node};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

const VERSION: u32 = 1;
const STORE: &str = ".agents/codegraph";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphManifest { pub version: u32, pub schema: u32, pub root: String, pub mode: CodeGraphMode, pub revision: u64, pub fingerprints: BTreeMap<String, String> }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphFreshness { Fresh, Initialized, Refreshed, Stale, Error }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraphSnapshot { pub graph: CodeGraph, pub manifest: GraphManifest, pub freshness: GraphFreshness, pub changed: Vec<String>, pub deleted: Vec<String>, pub error: Option<String> }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraphQuery { pub text: String, pub limit: usize, pub depth: usize }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQueryResult { pub nodes: Vec<Node>, pub incoming: Vec<Edge>, pub outgoing: Vec<Edge>, pub impact: Vec<Node> }

pub fn ensure_persistent_codegraph(root: impl AsRef<Path>) -> CodeGraphResult<CodeGraphSnapshot> { refresh_persistent_codegraph(root) }
pub fn refresh_persistent_codegraph(root: impl AsRef<Path>) -> CodeGraphResult<CodeGraphSnapshot> {
    let root = root.as_ref().canonicalize().map_err(|e| CodeGraphError::at_path(root.as_ref(), e.to_string()))?;
    let mode = crate::detect_codegraph_mode(&root)?;
    let current = store_dir(&root)?;
    let files = discover_files(&root, mode)?;
    let fingerprints = files.iter().filter_map(|p| { let rel = p.strip_prefix(&root).ok().map(slash_path)?; let bytes = fs::read(p).ok()?; Some((rel, fingerprint_bytes(&bytes))) }).collect::<BTreeMap<_, _>>();
    let old = if current.join("CURRENT").exists() {
        Some(read_current(&current, &root, mode)?)
    } else {
        None
    };
    if let Some(snapshot) = old.as_ref() {
        if snapshot.manifest.fingerprints == fingerprints { return Ok(CodeGraphSnapshot { freshness: GraphFreshness::Fresh, changed: vec![], deleted: vec![], ..snapshot.clone() }); }
    }
    let previous = old.as_ref().map(|s| &s.manifest.fingerprints);
    let changed = fingerprints.iter().filter(|(p,f)| previous.is_none_or(|o| o.get(*p) != Some(f))).map(|(p,_)| p.clone()).collect::<Vec<_>>();
    let deleted: Vec<String> = previous.map(|o| o.keys().filter(|p| !fingerprints.contains_key(*p)).cloned().collect()).unwrap_or_default();
    // Extraction is performed once for the changed source set, then only those file contributions
    // are replaced in the persisted graph. Unchanged refreshes return before extraction.
    let extracted = build_codegraph(&root, BuildOptions { mode: Some(mode) })?;
    let graph = old.as_ref().map(|s| replace_file_contributions(s.graph.clone(), &extracted, &changed, &deleted)).unwrap_or(extracted);
    let revision = old.as_ref().map_or(1, |s| s.manifest.revision + 1);
    let manifest = GraphManifest { version: VERSION, schema: VERSION, root: root.to_string_lossy().into_owned(), mode, revision, fingerprints };
    write_generation(&current, &manifest, &graph)?;
    Ok(CodeGraphSnapshot { graph, manifest, freshness: if old.is_none() { GraphFreshness::Initialized } else { GraphFreshness::Refreshed }, changed, deleted, error: None })
}
pub fn replace_file_contributions(mut base: CodeGraph, replacement: &CodeGraph, changed: &[String], deleted: &[String]) -> CodeGraph {
    let paths = changed.iter().chain(deleted).collect::<BTreeSet<_>>();
    let removed = base.nodes.iter().filter(|n| n.path.as_ref().is_some_and(|p| paths.contains(p))).map(|n| n.id.clone()).collect::<BTreeSet<_>>();
    base.nodes.retain(|n| !removed.contains(&n.id));
    base.edges.retain(|e| !removed.contains(&e.from) && !removed.contains(&e.to));
    let additions = replacement.nodes.iter().filter(|n| n.path.as_ref().is_some_and(|p| paths.contains(p))).cloned().collect::<Vec<_>>();
    let ids = base.nodes.iter().map(|n| n.id.clone()).chain(additions.iter().map(|n| n.id.clone())).collect::<BTreeSet<_>>();
    base.nodes.extend(additions);
    base.edges.extend(replacement.edges.iter().filter(|e| ids.contains(&e.from) && ids.contains(&e.to) && (removed.contains(&e.from) || removed.contains(&e.to))).cloned());
    base.nodes.sort_by(|a,b| a.id.cmp(&b.id)); base.edges.sort_by(|a,b| (a.from.clone(),a.to.clone()).cmp(&(b.from.clone(),b.to.clone()))); base
}

pub fn query_persistent_codegraph(root: impl AsRef<Path>, query: CodeGraphQuery) -> CodeGraphResult<GraphQueryResult> {
    let root = root.as_ref().canonicalize()?; let snapshot = refresh_persistent_codegraph(&root)?; let limit = query.limit.min(128); let index = read_index(&root, &snapshot.manifest, &snapshot.graph)?;
    let text = query.text.to_ascii_lowercase(); let terms = text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_').filter(|t| t.len()>2).collect::<Vec<_>>();
    let ids = index.iter().filter(|(path,_id)| terms.is_empty() || terms.iter().any(|t| path.to_ascii_lowercase().contains(t))).map(|(_,id)| id).collect::<BTreeSet<_>>();
    let mut nodes = snapshot.graph.nodes.iter().filter(|n| ids.contains(&n.id) || (terms.is_empty() && n.path.is_some())).cloned().collect::<Vec<_>>(); nodes.sort_by(|a,b| a.id.cmp(&b.id)); nodes.truncate(limit);
    let selected = nodes.iter().map(|n| n.id.clone()).collect::<BTreeSet<_>>(); let incoming = snapshot.graph.edges.iter().filter(|e| selected.contains(&e.to)).cloned().collect(); let outgoing = snapshot.graph.edges.iter().filter(|e| selected.contains(&e.from)).cloned().collect();
    let impact_ids = traverse(&snapshot.graph, &selected, query.depth.min(4)); let mut impact = snapshot.graph.nodes.iter().filter(|n| impact_ids.contains(&n.id)).cloned().collect::<Vec<_>>(); impact.sort_by(|a,b| a.id.cmp(&b.id)); impact.truncate(limit); Ok(GraphQueryResult { nodes, incoming, outgoing, impact })
}
fn traverse(graph: &CodeGraph, start: &BTreeSet<String>, depth: usize) -> BTreeSet<String> { let mut seen=start.clone(); let mut q=start.iter().map(|x|(x.clone(),0)).collect::<VecDeque<_>>(); while let Some((id,d))=q.pop_front() { if d>=depth {continue;} for e in graph.edges.iter().filter(|e|e.from==id || e.to==id) { let next=if e.from==id {&e.to} else {&e.from}; if seen.insert(next.clone()) {q.push_back((next.clone(),d+1));} } } seen }
fn store_dir(root: &Path) -> CodeGraphResult<PathBuf> { let dir=root.join(STORE); reject_path(root,&dir)?; fs::create_dir_all(&dir)?; Ok(dir) }
fn reject_path(root:&Path,path:&Path)->CodeGraphResult<()> { let rel=path.strip_prefix(root).map_err(|_|CodeGraphError::new("CodeGraph path escapes project root"))?; if rel.components().any(|c|matches!(c,Component::ParentDir|Component::RootDir|Component::Prefix(_))){return Err(CodeGraphError::new("CodeGraph path traversal rejected"));} let mut p=root.to_path_buf(); for c in rel.components(){p.push(c); if p.exists() && fs::symlink_metadata(&p)?.file_type().is_symlink(){return Err(CodeGraphError::at_path(&p,"CodeGraph path must not contain symlinks"));}} Ok(()) }
fn read_current(dir:&Path,root:&Path,mode:CodeGraphMode)->CodeGraphResult<CodeGraphSnapshot>{ let name=fs::read_to_string(dir.join("CURRENT"))?; let generation=dir.join(name.trim()); reject_path(root,&generation)?; let manifest:GraphManifest=serde_json::from_slice(&fs::read(generation.join("manifest.json"))?)?; if manifest.version!=VERSION || manifest.schema!=VERSION || manifest.root!=root.to_string_lossy() || manifest.mode!=mode {return Err(CodeGraphError::new("CodeGraph generation is incompatible"));} let nodes=serde_json::from_slice(&fs::read(generation.join("nodes.json"))?)?; let edges=serde_json::from_slice(&fs::read(generation.join("edges.json"))?)?; Ok(CodeGraphSnapshot{graph:CodeGraph{mode,root:".".into(),nodes,edges},manifest,freshness:GraphFreshness::Stale,changed:vec![],deleted:vec![],error:None}) }
fn read_index(root:&Path,manifest:&GraphManifest,graph:&CodeGraph)->CodeGraphResult<BTreeMap<String,String>> { if manifest.root!=root.to_string_lossy() || graph.mode!=manifest.mode {return Err(CodeGraphError::new("CodeGraph compatibility mismatch"));} let dir=store_dir(root)?; let name=fs::read_to_string(dir.join("CURRENT"))?; let value=serde_json::from_slice(&fs::read(dir.join(name.trim()).join("index.json"))?)?; Ok(value) }
fn write_generation(dir:&Path,manifest:&GraphManifest,graph:&CodeGraph)->CodeGraphResult<()> { let name=format!("generation-{}-{}",manifest.revision,std::process::id()); let tmp=dir.join(format!(".{name}.tmp")); let generation=dir.join(&name); fs::create_dir_all(&tmp)?; for (file,value) in [("manifest.json",serde_json::to_vec_pretty(manifest)?),("nodes.json",serde_json::to_vec_pretty(&graph.nodes)?),("edges.json",serde_json::to_vec_pretty(&graph.edges)?),("index.json",serde_json::to_vec_pretty(&graph.nodes.iter().filter_map(|n|n.path.as_ref().map(|p|(p,n.id.clone()))).collect::<BTreeMap<_,_>>())?)] { fs::write(tmp.join(file),value)?; } fs::rename(&tmp,&generation)?; let pointer=dir.join("CURRENT.tmp"); fs::write(&pointer,format!("{name}\n"))?; fs::rename(pointer,dir.join("CURRENT"))?; Ok(()) }
#[allow(dead_code)] fn _fingerprint(bytes:&[u8])->String{fingerprint_bytes(bytes)}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    #[test]
    fn generation_round_trip_and_incremental_reuse() {
        let d=TempDir::new().unwrap(); fs::create_dir_all(d.path().join("src")).unwrap(); fs::write(d.path().join("src/a.rs"),"pub fn a() {}\n").unwrap();
        let first=ensure_persistent_codegraph(d.path()).unwrap(); assert_eq!(first.freshness,GraphFreshness::Initialized);
        let second=refresh_persistent_codegraph(d.path()).unwrap(); assert_eq!(second.freshness,GraphFreshness::Fresh); assert_eq!(second.manifest.revision,first.manifest.revision);
        let current=fs::read_to_string(d.path().join(STORE).join("CURRENT")).unwrap(); let generation=d.path().join(STORE).join(current.trim()); assert!(generation.join("manifest.json").is_file()); assert!(generation.join("nodes.json").is_file()); assert!(generation.join("edges.json").is_file()); assert!(generation.join("index.json").is_file());
        fs::write(generation.join("manifest.json"),"{}").unwrap(); assert!(refresh_persistent_codegraph(d.path()).is_err());
    }
}
