use crate::error::{CodeGraphError, CodeGraphResult};
use crate::model::{Baseline, BaselineDiagnostic, CheckReport, Diagnostic, DiagnosticSeverity};
use crate::paths::safe_generated_path;
use std::fs;
use std::path::Path;

pub(crate) fn apply_baseline(root: &Path, report: CheckReport) -> CodeGraphResult<CheckReport> {
    let Some(baseline) = load_baseline(root)? else {
        return Ok(report);
    };
    let mut filtered = CheckReport::new();

    for diagnostic in report.diagnostics {
        let matching = baseline
            .diagnostics
            .iter()
            .find(|entry| entry.code == diagnostic.code && entry.path == diagnostic.path);

        match matching {
            Some(entry)
                if diagnostic
                    .metric
                    .zip(entry.metric)
                    .is_some_and(|(current, baseline)| current > baseline) =>
            {
                filtered.diagnostics.push(Diagnostic {
                    code: "baseline_violation_grew".to_string(),
                    severity: DiagnosticSeverity::Error,
                    path: diagnostic.path,
                    message: "A baseline CodeGraph violation grew.".to_string(),
                    action:
                        "Refactor the file or update the related spec before changing the baseline."
                            .to_string(),
                    owner: diagnostic.owner,
                    metric: diagnostic.metric,
                });
            }
            Some(_) => {}
            None => filtered.diagnostics.push(diagnostic),
        }
    }

    Ok(filtered)
}

pub(crate) fn write_codegraph_baseline(
    root: &Path,
    report: &CheckReport,
) -> CodeGraphResult<String> {
    let baseline = Baseline {
        diagnostics: report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .map(|diagnostic| BaselineDiagnostic {
                code: diagnostic.code.clone(),
                path: diagnostic.path.clone(),
                metric: diagnostic.metric,
            })
            .collect(),
    };
    let path = safe_generated_path(root, Path::new(".dowe/codegraph/baseline.json"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| CodeGraphError::at_path(parent, error.to_string()))?;
    }
    fs::write(&path, json(&baseline)?)
        .map_err(|error| CodeGraphError::at_path(&path, error.to_string()))?;
    Ok(".dowe/codegraph/baseline.json".to_string())
}

fn load_baseline(root: &Path) -> CodeGraphResult<Option<Baseline>> {
    let path = root.join(".dowe/codegraph/baseline.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| CodeGraphError::at_path(&path, error.to_string()))?;
    let baseline = serde_json::from_str(&content)?;
    Ok(Some(baseline))
}

fn json<T: serde::Serialize>(value: &T) -> CodeGraphResult<String> {
    let mut content = serde_json::to_string_pretty(value)?;
    content.push('\n');
    Ok(content)
}
