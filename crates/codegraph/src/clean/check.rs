use super::build_clean_graph;
use crate::duplicate::{detect_duplicates, detect_ownership_duplication};
use crate::{
    BuildOptions, CheckReport, CodeGraphResult, Diagnostic, DiagnosticSeverity,
    detect_codegraph_mode,
};
use std::fs;
use std::path::Path;

pub fn check_clean_codegraph(
    root: &Path,
    options: crate::CheckOptions,
) -> CodeGraphResult<CheckReport> {
    let graph = build_clean_graph(root, BuildOptions { mode: options.mode })?;
    let root = root.canonicalize()?;
    let mut report = CheckReport::new();
    for node in graph.nodes().filter(|node| node.kind == "file") {
        let Some(path) = node.path.as_deref() else {
            continue;
        };
        if anonymous_partition(path) {
            report.diagnostics.push(diagnostic(
                "anonymous_partition_file",
                DiagnosticSeverity::Error,
                path,
                "Source file uses an anonymous split name instead of a functional responsibility.",
                "Rename the split by domain responsibility.",
                None,
            ));
        }
        let is_rust = path.ends_with(".rs");
        let is_router_runtime = path.ends_with(".js") && path.contains("/router_runtime/");
        if !is_rust && !is_router_runtime {
            continue;
        }
        let line_count = fs::read_to_string(root.join(path))?.lines().count();
        if line_count > 300 {
            report.diagnostics.push(diagnostic(
                "file_over_300_lines",
                DiagnosticSeverity::Warning,
                path,
                "Source file exceeds the preferred 300 line modularity budget.",
                "Split the file by responsibility.",
                Some(line_count),
            ));
        }
        if line_count > 500 {
            report.diagnostics.push(diagnostic(
                "file_over_500_lines",
                DiagnosticSeverity::Error,
                path,
                "Source file exceeds the blocking 500 line modularity budget.",
                "Refactor the file into smaller modules.",
                Some(line_count),
            ));
        }
        if path.ends_with("/lib.rs") && line_count > 200 {
            report.diagnostics.push(diagnostic(
                "lib_rs_over_200_lines",
                DiagnosticSeverity::Warning,
                path,
                "lib.rs exceeds the preferred API surface budget.",
                "Keep lib.rs focused on modules and reexports.",
                Some(line_count),
            ));
        }
    }
    let mode = options
        .mode
        .map(Ok)
        .unwrap_or_else(|| detect_codegraph_mode(&root))?;
    detect_duplicates(&root, mode, &mut report);
    detect_ownership_duplication(&root, mode, &mut report);
    Ok(report)
}

fn diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    path: &str,
    message: &str,
    action: &str,
    metric: Option<usize>,
) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        severity,
        path: path.into(),
        message: message.into(),
        action: action.into(),
        owner: None,
        metric,
    }
}

fn anonymous_partition(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let numbered = |prefix: &str| {
        file_name
            .strip_prefix(prefix)
            .is_some_and(|value| value.chars().next().is_some_and(|c| c.is_ascii_digit()))
    };
    numbered("part_")
        || numbered("tests_")
        || path.split('/').any(|segment| segment.ends_with("_parts"))
        || (path.contains("/router_runtime/")
            && file_name.ends_with(".js")
            && file_name
                .strip_suffix(".js")
                .and_then(|name| name.rsplit_once('_'))
                .is_some_and(|(_, suffix)| {
                    !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
                }))
}
