use crate::baseline::apply_baseline;
use crate::build::build_codegraph;
use crate::duplicate::{detect_duplicates, detect_ownership_duplication};
use crate::error::CodeGraphResult;
use crate::mode::detect_codegraph_mode;
use crate::model::{CheckOptions, CheckReport, Diagnostic, DiagnosticSeverity};
use crate::waivers::{collect_waivers, waiver_for};
use std::path::Path;

pub(crate) fn check_codegraph(root: &Path, options: CheckOptions) -> CodeGraphResult<CheckReport> {
    let mode = options
        .mode
        .map(Ok)
        .unwrap_or_else(|| detect_codegraph_mode(root))?;
    let graph = build_codegraph(root, crate::model::BuildOptions { mode: Some(mode) })?;
    let waivers = collect_waivers(root)?;
    let mut report = CheckReport::new();

    for node in &graph.nodes {
        let Some(path) = &node.path else {
            continue;
        };
        if !path.ends_with(".rs") {
            continue;
        }
        if is_anonymous_partition_path(path) {
            report.diagnostics.push(Diagnostic {
                code: "anonymous_partition_file".to_string(),
                severity: DiagnosticSeverity::Error,
                path: path.clone(),
                message: "Source file uses an anonymous split name instead of a functional responsibility."
                    .to_string(),
                action: "Rename split files and directories by domain responsibility, such as parsing, rendering, validation, or artifacts."
                    .to_string(),
                owner: node.owner.clone(),
                metric: None,
            });
        }
        let Some(metrics) = &node.metrics else {
            continue;
        };

        if metrics.total_lines > 300 {
            report.diagnostics.push(Diagnostic {
                code: "file_over_300_lines".to_string(),
                severity: DiagnosticSeverity::Warning,
                path: path.clone(),
                message: "Source file exceeds the preferred 300 line modularity budget."
                    .to_string(),
                action: "Split the file by responsibility before adding more behavior.".to_string(),
                owner: node.owner.clone(),
                metric: Some(metrics.total_lines),
            });
        }

        if metrics.total_lines > 500 {
            if let Some(waiver) = waiver_for(&waivers, path, 500) {
                report.diagnostics.push(Diagnostic {
                    code: "modular_waiver_debt".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    path: path.clone(),
                    message: format!("File exceeds 500 lines under waiver `{}`.", waiver.spec),
                    action: format!("Remove the waiver when `{}` is complete.", waiver.expires),
                    owner: Some(waiver.owner.clone()),
                    metric: Some(metrics.total_lines),
                });
            } else {
                report.diagnostics.push(Diagnostic {
                    code: "file_over_500_lines".to_string(),
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    message: "Source file exceeds the blocking 500 line modularity budget."
                        .to_string(),
                    action:
                        "Refactor into smaller modules or declare a temporary waiver in a spec."
                            .to_string(),
                    owner: node.owner.clone(),
                    metric: Some(metrics.total_lines),
                });
            }
        }

        if path.ends_with("/lib.rs") && metrics.total_lines > 200 {
            report.diagnostics.push(Diagnostic {
                code: "lib_rs_over_200_lines".to_string(),
                severity: DiagnosticSeverity::Warning,
                path: path.clone(),
                message: "`lib.rs` exceeds the preferred 200 line API surface budget.".to_string(),
                action: "Keep lib.rs focused on modules and reexports.".to_string(),
                owner: node.owner.clone(),
                metric: Some(metrics.total_lines),
            });
            if metrics.functions > 10 || metrics.inline_tests > 0 {
                report.diagnostics.push(Diagnostic {
                    code: "lib_rs_monolith".to_string(),
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    message: "`lib.rs` contains implementation or test responsibilities."
                        .to_string(),
                    action: "Move implementation and tests to focused modules.".to_string(),
                    owner: node.owner.clone(),
                    metric: Some(metrics.total_lines),
                });
            }
        }
    }

    detect_duplicates(root, mode, &mut report);
    detect_ownership_duplication(root, mode, &mut report);

    if options.use_baseline {
        apply_baseline(root, report)
    } else {
        Ok(report)
    }
}

fn is_anonymous_partition_path(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let numbered_part = file_name
        .strip_prefix("part_")
        .is_some_and(starts_with_digit);
    let numbered_test = file_name
        .strip_prefix("tests_")
        .is_some_and(starts_with_digit);
    let anonymous_dir = path.split('/').any(|segment| segment.ends_with("_parts"));
    numbered_part || numbered_test || anonymous_dir
}

fn starts_with_digit(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|value| value.is_ascii_digit())
}

pub(crate) fn ownership_areas() -> [(&'static str, &'static str); 13] {
    [
        ("components_model", "dowe-lang/crates/components"),
        ("components_semantics", "dowe-lang/crates/components"),
        ("web_generation", "dowe-lang/crates/generator_web"),
        ("desktop_generation", "dowe-lang/crates/generator_desktop"),
        ("android_generation", "dowe-lang/crates/generator_android"),
        ("ios_generation", "dowe-lang/crates/generator_ios"),
        ("source_parse_lowering", "dowe-lang/crates/compiler"),
        ("dowe_artifact_writing", "dowe-lang/crates/compiler"),
        ("development_runtime", "dowe-lang/crates/runtime"),
        ("spawn_runtime", "dowe-lang/crates/spawn"),
        ("agent_harnesses", "dowe-lang/crates/agent_harness"),
        ("codegraph", "dowe-lang/crates/codegraph"),
        ("cli_ipc_adapters", "dowe-lang/crates/cli"),
    ]
}
