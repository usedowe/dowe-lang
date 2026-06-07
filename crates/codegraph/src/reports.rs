use crate::error::{CodeGraphError, CodeGraphResult};
use crate::model::{CheckReport, CodeGraph, DiagnosticSeverity, NodeKind, WrittenReports};
use crate::paths::safe_generated_path;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub(crate) fn write_codegraph_reports(
    root: &Path,
    graph: &CodeGraph,
    report: &CheckReport,
) -> CodeGraphResult<WrittenReports> {
    write_json(root, ".dowe/codegraph/graph.json", graph)?;
    write_json(root, ".dowe/codegraph/report.json", report)?;
    write_text(root, ".dowe/codegraph/report.md", &markdown_report(report))?;
    write_json(
        root,
        ".dowe/codegraph/ownership.json",
        &ownership_report(graph),
    )?;
    write_json(
        root,
        ".dowe/codegraph/duplication.json",
        &duplication_report(report),
    )?;

    Ok(WrittenReports {
        graph_path: ".dowe/codegraph/graph.json".to_string(),
        report_path: ".dowe/codegraph/report.json".to_string(),
        markdown_path: ".dowe/codegraph/report.md".to_string(),
        ownership_path: ".dowe/codegraph/ownership.json".to_string(),
        duplication_path: ".dowe/codegraph/duplication.json".to_string(),
    })
}

fn markdown_report(report: &CheckReport) -> String {
    let mut content = String::from("# CodeGraph Report\n\n");
    let errors = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .count();
    let warnings = report.diagnostics.len().saturating_sub(errors);
    content.push_str(&format!("Errors: {errors}\n\nWarnings: {warnings}\n\n"));
    content.push_str("## Diagnostics\n\n");
    if report.diagnostics.is_empty() {
        content.push_str("- No diagnostics.\n");
    } else {
        for diagnostic in &report.diagnostics {
            content.push_str(&format!(
                "- {:?} {} `{}`: {} Action: {}\n",
                diagnostic.severity,
                diagnostic.code,
                diagnostic.path,
                diagnostic.message,
                diagnostic.action
            ));
        }
    }
    content
}

fn ownership_report(graph: &CodeGraph) -> Vec<serde_json::Value> {
    graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::OwnershipArea)
        .map(|node| {
            serde_json::json!({
                "name": node.name,
                "owner": node.owner,
                "path": node.path
            })
        })
        .collect()
}

fn duplication_report(report: &CheckReport) -> Vec<serde_json::Value> {
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.contains("duplicate"))
        .map(|diagnostic| {
            serde_json::json!({
                "code": diagnostic.code,
                "path": diagnostic.path,
                "message": diagnostic.message,
                "action": diagnostic.action,
                "owner": diagnostic.owner,
                "metric": diagnostic.metric
            })
        })
        .collect()
}

fn write_json<T: Serialize>(root: &Path, relative: &str, value: &T) -> CodeGraphResult<()> {
    let mut content = serde_json::to_string_pretty(value)?;
    content.push('\n');
    write_text(root, relative, &content)
}

fn write_text(root: &Path, relative: &str, content: &str) -> CodeGraphResult<()> {
    let path = safe_generated_path(root, Path::new(relative))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| CodeGraphError::at_path(parent, error.to_string()))?;
    }
    fs::write(&path, content).map_err(|error| CodeGraphError::at_path(&path, error.to_string()))
}
