use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantRule {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvariantViolation {
    pub id: String,
    pub path: String,
    pub rule: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvariantReport {
    pub status: &'static str,
    pub checked: usize,
    pub violations: Vec<InvariantViolation>,
}

pub fn check_invariants(root: impl AsRef<Path>) -> HarnessResult<InvariantReport> {
    let root = root.as_ref();
    let path = root.join(".agent/invariants.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(InvariantReport {
                status: "not_configured",
                checked: 0,
                violations: Vec::new(),
            });
        }
        Err(error) => return Err(HarnessError::at_path(&path, error.to_string())),
    };
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(HarnessError::at_path(
            &path,
            "invariant registry exceeds 2 MiB",
        ));
    }
    let rules: Vec<InvariantRule> = serde_json::from_slice(&bytes).map_err(|error| {
        HarnessError::at_path(&path, format!("invalid invariant registry: {error}"))
    })?;
    if rules.len() > 1024 {
        return Err(HarnessError::at_path(
            &path,
            "invariant registry has too many rules",
        ));
    }
    let mut violations = Vec::new();
    for rule in &rules {
        if rule.id.is_empty() || rule.id.len() > 128 || rule.description.len() > 2048 {
            return Err(HarnessError::new(
                "invariant id or description is out of bounds",
            ));
        }
        if rule.paths.len() > 128 || rule.forbidden.len() > 128 || rule.required.len() > 128 {
            return Err(HarnessError::new("invariant rule has too many clauses"));
        }
        for path in &rule.paths {
            let relative = safe_path(path)?;
            let absolute = root.join(&relative);
            let content = fs::read_to_string(&absolute).map_err(|error| {
                HarnessError::at_path(
                    &absolute,
                    format!("invariant source cannot be read: {error}"),
                )
            })?;
            for pattern in &rule.forbidden {
                if pattern.is_empty() || pattern.len() > 1024 {
                    return Err(HarnessError::new("invariant pattern is out of bounds"));
                }
                if content.contains(pattern) {
                    violations.push(InvariantViolation {
                        id: rule.id.clone(),
                        path: path.clone(),
                        rule: "forbidden".into(),
                        evidence: pattern.clone(),
                    });
                }
            }
            for pattern in &rule.required {
                if pattern.is_empty() || pattern.len() > 1024 {
                    return Err(HarnessError::new("invariant pattern is out of bounds"));
                }
                if !content.contains(pattern) {
                    violations.push(InvariantViolation {
                        id: rule.id.clone(),
                        path: path.clone(),
                        rule: "required".into(),
                        evidence: pattern.clone(),
                    });
                }
            }
        }
    }
    Ok(InvariantReport {
        status: if violations.is_empty() {
            "passed"
        } else {
            "failed"
        },
        checked: rules.len(),
        violations,
    })
}

fn safe_path(path: &str) -> HarnessResult<&Path> {
    let value = Path::new(path);
    if path.is_empty()
        || value
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(HarnessError::new(
            "invariant paths must be project-relative",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariant_report_blocks_forbidden_content_and_accepts_required_content() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::create_dir_all(root.path().join(".agent")).unwrap();
        fs::write(root.path().join("src/main.dowe"), "page Home\n").unwrap();
        fs::write(
            root.path().join(".agent/invariants.json"),
            r#"[{"id":"no-secret","description":"no secret","paths":["src/main.dowe"],"forbidden":["SECRET"],"required":["page Home"]}]"#,
        )
        .unwrap();
        let report = check_invariants(root.path()).unwrap();
        assert_eq!(report.status, "passed");
        assert_eq!(report.checked, 1);
    }

    #[test]
    fn invariant_paths_cannot_escape_the_project() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(".agent")).unwrap();
        fs::write(
            root.path().join(".agent/invariants.json"),
            r#"[{"id":"escape","description":"bad","paths":["../outside"],"forbidden":[],"required":[]}]"#,
        )
        .unwrap();
        assert!(check_invariants(root.path()).is_err());
    }
}
