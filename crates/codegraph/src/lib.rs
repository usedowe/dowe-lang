mod baseline;
mod build;
mod check;
pub mod clean;
mod duplicate;
mod error;
mod metrics;
mod mode;
mod model;
mod paths;
mod persistence;
#[cfg(test)]
mod persistence_edge_tests;
mod reports;
mod unified;
mod waivers;

pub use error::{CodeGraphError, CodeGraphResult};
pub use model::*;
pub use persistence::{
    CodeGraphBinding, CodeGraphQuery, CodeGraphSnapshot, GraphFreshness, GraphManifest,
    GraphQueryResult, ensure_persistent_codegraph, query_persistent_codegraph,
    read_persistent_codegraph, refresh_persistent_codegraph,
};
pub use unified::{
    EvidenceStatus, KnowledgeEdge, KnowledgeNamespace, KnowledgeNode, KnowledgeRegistry,
    KnowledgeRelation, UnifiedCodeGraph, UnifiedGraphQuery, UnifiedGraphQueryResult,
    UnifiedNodeKind, UnifiedScope, load_knowledge_registry, load_knowledge_registry_into_clean,
    unified_graph_from_codegraph,
};

pub fn check_bound_persistent_codegraph(
    root: impl AsRef<std::path::Path>,
    binding: &CodeGraphBinding,
    options: CheckOptions,
) -> CodeGraphResult<CheckReport> {
    check::check_bound_persistent_codegraph(root.as_ref(), binding, options)
}

pub fn detect_codegraph_mode(root: impl AsRef<std::path::Path>) -> CodeGraphResult<CodeGraphMode> {
    mode::detect_codegraph_mode(root.as_ref())
}

pub fn build_codegraph(
    root: impl AsRef<std::path::Path>,
    options: BuildOptions,
) -> CodeGraphResult<CodeGraph> {
    build::build_codegraph(root.as_ref(), options)
}

pub fn check_codegraph(
    root: impl AsRef<std::path::Path>,
    options: CheckOptions,
) -> CodeGraphResult<CheckReport> {
    check::check_codegraph(root.as_ref(), options)
}

pub fn explain_node(
    root: impl AsRef<std::path::Path>,
    selector: &str,
    options: BuildOptions,
) -> CodeGraphResult<NodeExplanation> {
    build::explain_node(root.as_ref(), selector, options)
}

pub fn explain_clean_node(
    root: impl AsRef<std::path::Path>,
    selector: &str,
    options: BuildOptions,
) -> CodeGraphResult<clean::CleanExplanation> {
    let graph = clean::build_clean_graph(root.as_ref(), options)?;
    clean::explain_node(&graph, selector)
        .ok_or_else(|| CodeGraphError::new(format!("CodeGraph node `{selector}` was not found")))
}

pub fn write_codegraph_reports(
    root: impl AsRef<std::path::Path>,
    graph: &CodeGraph,
    report: &CheckReport,
) -> CodeGraphResult<WrittenReports> {
    reports::write_codegraph_reports(root.as_ref(), graph, report)
}

pub fn write_clean_codegraph_reports(
    root: impl AsRef<std::path::Path>,
    graph: &clean::CleanGraph,
    report: &CheckReport,
) -> CodeGraphResult<WrittenReports> {
    reports::write_clean_codegraph_reports(root.as_ref(), graph, report)
}

pub fn write_codegraph_baseline(
    root: impl AsRef<std::path::Path>,
    report: &CheckReport,
) -> CodeGraphResult<String> {
    baseline::write_codegraph_baseline(root.as_ref(), report)
}
