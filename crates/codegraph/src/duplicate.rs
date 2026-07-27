use crate::metrics::function_fingerprints;
use crate::model::{CheckReport, CodeGraphMode, Diagnostic, DiagnosticSeverity};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn detect_duplicates(root: &Path, mode: CodeGraphMode, report: &mut CheckReport) {
    let mut groups: BTreeMap<String, Vec<crate::model::FunctionFingerprint>> = BTreeMap::new();

    for diagnostic_file in rust_files(root, mode) {
        let Ok(relative) = diagnostic_file.strip_prefix(root) else {
            continue;
        };
        let relative = crate::paths::slash_path(relative);
        let Ok(content) = fs::read_to_string(&diagnostic_file) else {
            continue;
        };
        for fingerprint in function_fingerprints(&relative, &content) {
            groups
                .entry(fingerprint.normalized.clone())
                .or_default()
                .push(fingerprint);
        }
    }

    for functions in groups.values() {
        if functions.len() < 2 {
            continue;
        }
        let first = &functions[0];
        let second = &functions[1];
        report.diagnostics.push(Diagnostic {
            code: "duplicate_function".to_string(),
            severity: DiagnosticSeverity::Warning,
            path: second.path.clone(),
            message: format!(
                "Function `{}` has a normalized duplicate of `{}`.",
                second.name, first.name
            ),
            action: "Extract shared behavior or confirm ownership before adding more copies."
                .to_string(),
            owner: None,
            metric: Some(second.end_line.saturating_sub(second.start_line) + 1),
        });
    }
}

pub(crate) fn detect_ownership_duplication(
    root: &Path,
    mode: CodeGraphMode,
    report: &mut CheckReport,
) {
    for path in rust_files(root, mode) {
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative = crate::paths::slash_path(relative);
        if !relative.contains("generator_") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if content.contains("unknown_component") || content.contains("ComponentError") {
            let owner = if relative.starts_with("dowe-lang/") {
                "dowe-lang/crates/components"
            } else {
                "crates/components"
            };
            report.diagnostics.push(Diagnostic {
                code: "duplicated_component_semantics".to_string(),
                severity: DiagnosticSeverity::Warning,
                path: relative,
                message: "A generator appears to define component semantic validation.".to_string(),
                action: "Move component semantics to the shared components crate and consume the shared API."
                    .to_string(),
                owner: Some(owner.to_string()),
                metric: None,
            });
        }
    }
}

fn rust_files(root: &Path, mode: CodeGraphMode) -> Vec<std::path::PathBuf> {
    let Ok(files) = crate::paths::discover_files(root, mode) else {
        return Vec::new();
    };
    files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect()
}
