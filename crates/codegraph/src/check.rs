use crate::baseline::apply_baseline;
use crate::build::build_codegraph;
use crate::duplicate::{detect_duplicates, detect_ownership_duplication};
use crate::error::CodeGraphResult;
use crate::metrics::fingerprint_bytes;
use crate::mode::detect_codegraph_mode;
use crate::model::{CheckOptions, CheckReport, Diagnostic, DiagnosticSeverity};
use crate::paths::{discover_files, slash_path};
use crate::persistence::{read_persistent_codegraph, CodeGraphBinding};
use crate::waivers::{collect_waivers, waiver_for};
use std::collections::BTreeMap;
use std::fs;
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
        let is_rust = path.ends_with(".rs");
        let is_router_runtime_javascript = path.ends_with(".js")
            && path.contains("/router_runtime/");
        if !is_rust && !is_router_runtime_javascript {
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
        if !is_rust {
            continue;
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

pub(crate) fn check_bound_persistent_codegraph(
    root: &Path,
    binding: &CodeGraphBinding,
    _options: CheckOptions,
) -> CodeGraphResult<CheckReport> {
    let snapshot = read_persistent_codegraph(root)?;
    let files = discover_files(root, snapshot.manifest.mode)?;
    let fingerprints = files
        .iter()
        .filter_map(|path| {
            let relative = slash_path(path.strip_prefix(root).ok()?);
            if relative.starts_with(".agents/plans/") || relative.starts_with(".dowe/") {
                return None;
            }
            Some((relative, fingerprint_bytes(&fs::read(path).ok()?)))
        })
        .collect::<BTreeMap<_, _>>();
    let persisted_fingerprints = snapshot
        .manifest
        .fingerprints
        .iter()
        .filter(|(path, _)| !path.starts_with(".agents/plans/") && !path.starts_with(".dowe/"))
        .map(|(path, fingerprint)| (path.clone(), fingerprint.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut report = CheckReport::new();
    if snapshot.generation.as_deref() != Some(binding.generation.as_str())
        || snapshot.manifest.revision != binding.revision
        || snapshot.manifest.root != binding.root
        || snapshot.manifest.mode != binding.mode
    {
        report.diagnostics.push(drift_diagnostic(
            "codegraph_binding_mismatch",
            ".agents/codegraph/CURRENT",
            "Persisted CodeGraph generation does not match the plan binding.",
        ));
    }
    for path in persisted_fingerprints
        .keys()
        .filter(|path| !fingerprints.contains_key(*path))
    {
        report.diagnostics.push(drift_diagnostic(
            "codegraph_source_deleted",
            path,
            "A source file recorded by the persisted CodeGraph was deleted.",
        ));
    }
    for path in fingerprints
        .keys()
        .filter(|path| persisted_fingerprints.get(*path) != Some(fingerprints.get(*path).unwrap()))
    {
        let code = if persisted_fingerprints.contains_key(path) {
            "codegraph_source_changed"
        } else {
            "codegraph_source_added"
        };
        report.diagnostics.push(drift_diagnostic(
            code,
            path,
            "Current source differs from the persisted CodeGraph manifest.",
        ));
    }
    validate_persisted_graph(&snapshot.graph, &snapshot.manifest, &mut report);
    if persisted_fingerprints != fingerprints {
        report.diagnostics.push(drift_diagnostic(
            "codegraph_manifest_drift",
            ".agents/codegraph/CURRENT",
            "Persisted CodeGraph manifest differs from current project files.",
        ));
    }
    Ok(report)
}

fn drift_diagnostic(code: &str, path: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Error,
        path: path.to_string(),
        message: message.to_string(),
        action: "Regenerate the persisted CodeGraph, then re-plan the feature.".to_string(),
        owner: None,
        metric: None,
    }
}

fn validate_persisted_graph(
    graph: &crate::model::CodeGraph,
    manifest: &crate::persistence::GraphManifest,
    report: &mut CheckReport,
) {
    let ids = graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != graph.nodes.len() {
        report.diagnostics.push(drift_diagnostic(
            "codegraph_graph_invalid",
            ".agents/codegraph",
            "Persisted CodeGraph contains duplicate node IDs.",
        ));
    }
    for edge in &graph.edges {
        if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) {
            report.diagnostics.push(drift_diagnostic(
                "codegraph_graph_invalid",
                ".agents/codegraph",
                "Persisted CodeGraph contains an edge to a missing node.",
            ));
            break;
        }
    }
    for node in &graph.nodes {
        if let Some(path) = &node.path {
            if manifest
                .fingerprints
                .get(path)
                .is_some_and(|fingerprint| fingerprint != &node.fingerprint)
            {
                report.diagnostics.push(drift_diagnostic(
                    "codegraph_graph_invalid",
                    path,
                    "Persisted CodeGraph node fingerprint disagrees with its manifest.",
                ));
            }
        }
    }
}

fn is_anonymous_partition_path(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let runtime_partition = path.contains("/router_runtime/")
        && file_name.ends_with(".js")
        && file_name
            .strip_suffix(".js")
            .and_then(|name| name.rsplit_once('_'))
            .is_some_and(|(_, suffix)| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()));
    let numbered_part = file_name
        .strip_prefix("part_")
        .is_some_and(starts_with_digit);
    let numbered_test = file_name
        .strip_prefix("tests_")
        .is_some_and(starts_with_digit);
    let anonymous_dir = path.split('/').any(|segment| segment.ends_with("_parts"));
    numbered_part || numbered_test || anonymous_dir || runtime_partition
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
