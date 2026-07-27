use crate::authoring::{PublicSkill, public_skills};
use crate::context::{AgentCodeGraphSummary, summarize_codegraph};
use crate::error::{AgentError, AgentResult};
use dowe_agent_harness::{DetectedMode, read_status};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_FILE_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectContext {
    pub root: String,
    pub dowe_version: String,
    pub mode: String,
    pub markers: Vec<String>,
    pub source_files: Vec<String>,
    pub source_file_count: usize,
    pub skills: Vec<PublicSkill>,
    pub harness: AgentHarnessSummary,
    pub codegraph: AgentCodeGraphSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHarnessSummary {
    pub mode: String,
    pub plan_count: usize,
    pub error: Option<String>,
}

pub fn project_context(root: impl AsRef<Path>) -> AgentResult<ProjectContext> {
    let root = normalize_root(root.as_ref())?;
    let markers = project_markers(&root);
    let (source_file_count, source_files) = discover_source_files(&root)?;
    let harness = harness_summary(&root);
    let mode = match harness.mode.as_str() {
        "dowe" => "dowe-workspace",
        "project" => "project",
        _ if markers.iter().any(|marker| marker == "main.dowe") => "project",
        _ => "unknown",
    }
    .to_string();

    Ok(ProjectContext {
        root: slash_path(&root),
        dowe_version: env!("CARGO_PKG_VERSION").to_string(),
        mode,
        markers,
        source_files,
        source_file_count,
        skills: public_skills(),
        harness,
        codegraph: summarize_codegraph(&root, 16)?,
    })
}

fn normalize_root(root: &Path) -> AgentResult<PathBuf> {
    root.canonicalize()
        .map_err(|error| AgentError::at_path(root, error.to_string()))
}

fn project_markers(root: &Path) -> Vec<String> {
    [
        "main.dowe",
        "theme.dowe",
        ".env.example",
        ".agents/manifest.json",
        "AGENTS.md",
        "CLAUDE.md",
    ]
    .into_iter()
    .filter(|marker| root.join(marker).is_file())
    .map(str::to_string)
    .collect()
}

fn discover_source_files(root: &Path) -> AgentResult<(usize, Vec<String>)> {
    let mut files = Vec::new();
    walk_source_files(root, root, &mut files)?;
    files.sort();
    let count = files.len();
    files.truncate(SOURCE_FILE_LIMIT);
    Ok((count, files))
}

fn walk_source_files(root: &Path, current: &Path, files: &mut Vec<String>) -> AgentResult<()> {
    let entries =
        fs::read_dir(current).map_err(|error| AgentError::at_path(current, error.to_string()))?;
    let mut entries = entries
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AgentError::at_path(current, error.to_string()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| AgentError::at_path(&path, error.to_string()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if !ignored_directory(&entry.file_name().to_string_lossy()) {
                walk_source_files(root, &path, files)?;
            }
        } else if file_type.is_file()
            && (path
                .extension()
                .is_some_and(|extension| extension == "dowe")
                || path.file_name().is_some_and(|name| name == ".env.example"))
        {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| AgentError::at_path(&path, error.to_string()))?;
            files.push(slash_path(relative));
        }
    }
    Ok(())
}

fn ignored_directory(name: &str) -> bool {
    matches!(
        name,
        ".agents" | ".dowe" | ".git" | "node_modules" | "target" | "dist" | "build" | ".release"
    )
}

fn harness_summary(root: &Path) -> AgentHarnessSummary {
    match read_status(root) {
        Ok(status) => AgentHarnessSummary {
            mode: match status.mode {
                DetectedMode::Dowe => "dowe",
                DetectedMode::Project => "project",
                DetectedMode::Unknown => "unknown",
            }
            .to_string(),
            plan_count: status.plans.len(),
            error: None,
        },
        Err(error) => AgentHarnessSummary {
            mode: "unknown".to_string(),
            plan_count: 0,
            error: Some(error.to_string()),
        },
    }
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
