use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{HarnessError, HarnessResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationTask {
    pub task_id: String,
    pub baseline: BTreeMap<String, String>,
    pub changes: Vec<IntegrationChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationChange {
    pub path: String,
    pub before_fingerprint: Option<String>,
    pub after_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationBlock {
    InvalidTask,
    StaleBaseline,
    OverlappingWrites,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationIssue {
    pub kind: IntegrationBlock,
    pub path: String,
    pub tasks: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationReport {
    pub ready: bool,
    pub issues: Vec<IntegrationIssue>,
    pub files: BTreeMap<String, String>,
}

/// Compares independent task results against one host baseline.
///
/// This function deliberately produces a decision, not a merge. A ready report
/// is the only safe input for a future worktree integrator; conflicts and stale
/// baselines remain visible and require an explicit host-side resolution.
pub fn analyze_integration(
    baseline: &BTreeMap<String, String>,
    tasks: &[IntegrationTask],
) -> HarnessResult<IntegrationReport> {
    if tasks.len() > 128 {
        return Err(HarnessError::new("integration accepts at most 128 tasks"));
    }
    let mut issues = Vec::new();
    let mut owners: BTreeMap<String, Vec<(String, Option<String>)>> = BTreeMap::new();
    let mut files = BTreeMap::new();

    for task in tasks {
        validate_task(task)?;
        if !task
            .baseline
            .iter()
            .all(|(path, fingerprint)| baseline.get(path) == Some(fingerprint))
        {
            for path in task.baseline.keys() {
                if baseline.get(path) != task.baseline.get(path) {
                    issues.push(IntegrationIssue {
                        kind: IntegrationBlock::StaleBaseline,
                        path: path.clone(),
                        tasks: vec![task.task_id.clone()],
                        detail: "task baseline differs from the host baseline".into(),
                    });
                }
            }
        }
        for change in &task.changes {
            if task.baseline.get(&change.path) != baseline.get(&change.path) {
                issues.push(IntegrationIssue {
                    kind: IntegrationBlock::StaleBaseline,
                    path: change.path.clone(),
                    tasks: vec![task.task_id.clone()],
                    detail: "changed file was not based on the current host fingerprint".into(),
                });
            }
            if let Some(after) = &change.after_fingerprint {
                files.insert(change.path.clone(), after.clone());
            }
            owners
                .entry(change.path.clone())
                .or_default()
                .push((task.task_id.clone(), change.after_fingerprint.clone()));
        }
    }

    for (path, entries) in owners {
        let unique = entries
            .iter()
            .map(|(_, fingerprint)| fingerprint)
            .collect::<std::collections::BTreeSet<_>>();
        if entries.len() > 1 && unique.len() > 1 {
            issues.push(IntegrationIssue {
                kind: IntegrationBlock::OverlappingWrites,
                path,
                tasks: entries.into_iter().map(|(task, _)| task).collect(),
                detail: "independent tasks propose different final contents".into(),
            });
        }
    }

    issues.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| format!("{:?}", a.kind).cmp(&format!("{:?}", b.kind)))
    });
    issues.dedup();
    Ok(IntegrationReport {
        ready: issues.is_empty(),
        issues,
        files,
    })
}

fn validate_task(task: &IntegrationTask) -> HarnessResult<()> {
    if task.task_id.trim().is_empty() || task.task_id.len() > 128 {
        return Err(HarnessError::new("integration task ID is invalid"));
    }
    if task.changes.len() > 256 || task.baseline.len() > 512 {
        return Err(HarnessError::new("integration task exceeds its bounds"));
    }
    for change in &task.changes {
        validate_path(&change.path)?;
        for fingerprint in [&change.before_fingerprint, &change.after_fingerprint]
            .into_iter()
            .flatten()
        {
            if fingerprint.is_empty() || fingerprint.len() > 128 {
                return Err(HarnessError::new("integration fingerprint is invalid"));
            }
        }
    }
    for path in task.baseline.keys() {
        validate_path(path)?;
    }
    Ok(())
}

fn validate_path(path: &str) -> HarnessResult<()> {
    let value = std::path::Path::new(path);
    if path.is_empty()
        || path.len() > 512
        || value.is_absolute()
        || value.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(HarnessError::new(
            "integration path must be project-relative",
        ));
    }
    Ok(())
}
