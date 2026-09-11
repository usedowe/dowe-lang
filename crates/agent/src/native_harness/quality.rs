use crate::AgentResult;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 2048;
const MAX_FINDINGS: usize = 128;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_DEPTH: usize = 64;

#[derive(Debug, Default)]
struct FileInventory {
    files: Vec<PathBuf>,
    skipped: usize,
    truncated: bool,
}

#[derive(Debug, Clone)]
struct Finding {
    path: String,
    line: usize,
    component: String,
    prop: String,
    value: String,
}

fn scalar(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches(',');
    if value.is_empty() || value.starts_with('{') || value.starts_with('[') {
        return None;
    }
    Some(value.trim_matches(['"', '\'']).to_string())
}

fn props(line: &str) -> Option<(String, BTreeMap<String, String>)> {
    let mut tokens = line.split_whitespace();
    let component = tokens.next()?.trim();
    if component.is_empty()
        || !component
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
        || !component
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    let mut values = BTreeMap::new();
    for token in tokens {
        let Some((key, value)) = token.split_once(':') else {
            continue;
        };
        if let Some(value) = scalar(value) {
            values.insert(key.to_string(), value);
        }
    }
    Some((component.to_string(), values))
}

fn is_ignored_directory(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".dowe" | ".agents" | "target" | "node_modules"
    )
}

fn collect_dowe_files(root: &Path) -> AgentResult<FileInventory> {
    let mut inventory = FileInventory::default();
    collect_dowe_files_at(root, &mut inventory, 0)?;
    Ok(inventory)
}

fn collect_dowe_files_at(
    root: &Path,
    inventory: &mut FileInventory,
    depth: usize,
) -> AgentResult<()> {
    if inventory.files.len() >= MAX_FILES {
        inventory.truncated = true;
        return Ok(());
    }
    if depth >= MAX_DEPTH {
        inventory.truncated = true;
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_ignored_directory)
            {
                collect_dowe_files_at(&path, inventory, depth + 1)?;
            }
        } else if metadata.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("dowe")
        {
            if metadata.len() > MAX_FILE_BYTES {
                inventory.skipped += 1;
                continue;
            }
            inventory.files.push(path);
            if inventory.files.len() >= MAX_FILES {
                inventory.truncated = true;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn parse_theme_defaults(root: &Path) -> AgentResult<BTreeMap<String, BTreeMap<String, String>>> {
    let path = root.join("theme.dowe");
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return Ok(BTreeMap::new());
    };
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Ok(BTreeMap::new());
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(BTreeMap::new());
    };
    let mut defaults = BTreeMap::new();
    let mut design_indent = None;
    let mut component_indent = None;
    for line in content.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.split_whitespace().next() == Some("design") {
            design_indent = Some(indent);
            component_indent = None;
            continue;
        }
        let Some(parent_indent) = design_indent else {
            // Keep accepting the compact fixture shape used by older local
            // projects while preferring the canonical `theme -> design`
            // nesting above.
            if indent != 4 {
                continue;
            }
            let Some((component, values)) = props(trimmed) else {
                continue;
            };
            if !values.is_empty() && component != "theme" && component != "fonts" {
                defaults.insert(component, values);
            }
            continue;
        };
        if indent <= parent_indent {
            design_indent = None;
            component_indent = None;
            continue;
        }
        let Some((component, values)) = props(trimmed) else {
            continue;
        };
        let component_indent = *component_indent.get_or_insert(indent);
        if indent == component_indent && !values.is_empty() {
            defaults.insert(component, values);
        }
    }
    Ok(defaults)
}

fn style_prop(prop: &str) -> bool {
    matches!(
        prop,
        "variant"
            | "scheme"
            | "radius"
            | "rounded"
            | "shadow"
            | "shadowColor"
            | "font"
            | "weight"
            | "spacing"
            | "size"
            | "bg"
            | "background"
            | "color"
            | "textColor"
            | "opacity"
            | "p"
            | "px"
            | "py"
            | "pt"
            | "pb"
            | "pl"
            | "pr"
            | "padding"
            | "border"
            | "borderColor"
            | "borderRadius"
            | "lineHeight"
            | "letterSpacing"
            | "fill"
    )
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn audit_dowe_project(root: &Path) -> AgentResult<Value> {
    if !root.join("main.dowe").is_file() {
        return Ok(json!({
            "status": "not_run",
            "reason": "project is not a Dowe application",
            "findings": [],
        }));
    }
    let defaults = parse_theme_defaults(root)?;
    let inventory = collect_dowe_files(root)?;
    let mut files = inventory.files;
    files.sort();
    let mut findings = Vec::new();
    for path in &files {
        if path.file_name().and_then(|name| name.to_str()) == Some("theme.dowe") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        for (line_index, line) in content.lines().enumerate() {
            let Some((component, values)) = props(line.trim()) else {
                continue;
            };
            let Some(component_defaults) = defaults.get(&component) else {
                continue;
            };
            for (prop, value) in values {
                if findings.len() >= MAX_FINDINGS || !style_prop(&prop) {
                    continue;
                }
                if component_defaults.get(&prop) == Some(&value) {
                    findings.push(Finding {
                        path: relative(root, path),
                        line: line_index + 1,
                        component: component.clone(),
                        prop,
                        value,
                    });
                }
            }
        }
    }
    let finding_values = findings
        .iter()
        .map(|finding| {
            json!({
                "path": finding.path,
                "line": finding.line,
                "component": finding.component,
                "prop": finding.prop,
                "value": finding.value,
                "message": "prop matches the component design default; omit it unless the difference is intentional",
            })
        })
        .collect::<Vec<_>>();
    let limits_hit = inventory.skipped > 0 || inventory.truncated || findings.len() >= MAX_FINDINGS;
    let status = if defaults.is_empty() {
        "not_run"
    } else if limits_hit {
        "not_run"
    } else if finding_values.is_empty() {
        "passed"
    } else {
        "failed"
    };
    Ok(json!({
        "status": status,
        "files_scanned": files.len(),
        "defaults_loaded": defaults.len(),
        "findings": finding_values,
        "limits": {"max_files": MAX_FILES, "max_findings": MAX_FINDINGS, "max_file_bytes": MAX_FILE_BYTES, "max_depth": MAX_DEPTH},
        "files_skipped": inventory.skipped,
        "limits_hit": limits_hit,
        "reason": if defaults.is_empty() {
            Some("theme.dowe has no readable design defaults")
        } else if limits_hit {
            Some("quality audit reached its bounded project scan limits")
        } else {
            None::<&str>
        },
    }))
}

pub(crate) fn quality_failed(report: &Value) -> bool {
    report["status"] == "failed"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_redundant_component_defaults_and_allows_differences() {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("main.dowe"),
            "import view from \"views/page.dowe\"\nmain {}\n",
        )
        .unwrap();
        fs::write(
            root.path().join("theme.dowe"),
            "theme\n    Card variant:\"solid\" scheme:\"surface\"\n",
        )
        .unwrap();
        fs::write(
            root.path().join("views.dowe"),
            "Card variant:\"solid\" scheme:\"primary\"\n",
        )
        .unwrap();
        let report = audit_dowe_project(root.path()).unwrap();
        assert_eq!(report["status"], "failed");
        assert_eq!(report["findings"].as_array().unwrap().len(), 1);
        assert_eq!(report["findings"][0]["prop"], "variant");
    }

    #[test]
    fn reads_canonical_two_space_design_defaults() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
        fs::write(
            root.path().join("theme.dowe"),
            "theme\n  design defaultTheme:\"reference\"\n    Card variant:\"solid\" scheme:\"surface\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.path().join("views")).unwrap();
        fs::write(
            root.path().join("views/page.dowe"),
            "page Page\n  Card variant:\"solid\" scheme:\"primary\"\n",
        )
        .unwrap();
        let report = audit_dowe_project(root.path()).unwrap();
        assert_eq!(report["defaults_loaded"], 1);
        assert_eq!(report["status"], "failed");
        assert_eq!(report["findings"].as_array().unwrap().len(), 1);
        assert_eq!(report["findings"][0]["prop"], "variant");
    }

    #[test]
    fn reports_not_run_when_no_design_defaults_exist() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
        let report = audit_dowe_project(root.path()).unwrap();
        assert_eq!(report["status"], "not_run");
        assert_eq!(report["defaults_loaded"], 0);
    }
}
