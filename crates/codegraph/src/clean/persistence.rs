use super::{CleanGraph, build_clean_graph};
use crate::{
    BuildOptions, CodeGraphBinding, CodeGraphResult, GraphFreshness, GraphManifest,
    read_persistent_codegraph, refresh_persistent_codegraph,
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CleanGraphSnapshot {
    pub graph: CleanGraph,
    pub manifest: GraphManifest,
    pub freshness: GraphFreshness,
    pub generation: Option<String>,
}

pub fn refresh_persistent_clean_codegraph(
    root: impl AsRef<Path>,
) -> CodeGraphResult<CleanGraphSnapshot> {
    let root = root.as_ref();
    let snapshot = refresh_persistent_codegraph(root)?;
    let graph = snapshot.clean_graph.clone().unwrap_or(build_clean_graph(
        root,
        BuildOptions {
            mode: Some(snapshot.manifest.mode),
        },
    )?);
    Ok(CleanGraphSnapshot {
        graph,
        manifest: snapshot.manifest,
        freshness: snapshot.freshness,
        generation: snapshot.generation,
    })
}

pub fn read_persistent_clean_codegraph(
    root: impl AsRef<Path>,
) -> CodeGraphResult<CleanGraphSnapshot> {
    let root = root.as_ref();
    let snapshot = read_persistent_codegraph(root)?;
    let graph = snapshot.clean_graph.clone().unwrap_or(build_clean_graph(
        root,
        BuildOptions {
            mode: Some(snapshot.manifest.mode),
        },
    )?);
    Ok(CleanGraphSnapshot {
        graph,
        manifest: snapshot.manifest,
        freshness: snapshot.freshness,
        generation: snapshot.generation,
    })
}

pub fn clean_binding(snapshot: &CleanGraphSnapshot) -> CodeGraphBinding {
    CodeGraphBinding {
        generation: snapshot.generation.clone().unwrap_or_default(),
        revision: snapshot.manifest.revision,
        root: snapshot.manifest.root.clone(),
        mode: snapshot.manifest.mode,
    }
}
