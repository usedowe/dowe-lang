mod baseline;
mod build;
mod check;
mod duplicate;
mod error;
mod metrics;
mod mode;
mod model;
mod paths;
mod reports;
mod waivers;

pub use error::{CodeGraphError, CodeGraphResult};
pub use model::*;

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

pub fn write_codegraph_reports(
    root: impl AsRef<std::path::Path>,
    graph: &CodeGraph,
    report: &CheckReport,
) -> CodeGraphResult<WrittenReports> {
    reports::write_codegraph_reports(root.as_ref(), graph, report)
}

pub fn write_codegraph_baseline(
    root: impl AsRef<std::path::Path>,
    report: &CheckReport,
) -> CodeGraphResult<String> {
    baseline::write_codegraph_baseline(root.as_ref(), report)
}
