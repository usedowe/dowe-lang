use crate::authoring::managed_agent_skills;
use crate::{AgentError, AgentResult};
use dowe_agent_harness::{
    HarnessError, HarnessResult, InitOptions, InitReport, init_agent_project_with_skills,
};
use dowe_runtime::{InitProjectOptions, InitProjectReport, init_project};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoweProjectInitReport {
    pub project: InitProjectReport,
    pub agent: InitReport,
}

pub fn init_dowe_project(
    root: impl AsRef<Path>,
    options: InitProjectOptions,
) -> AgentResult<DoweProjectInitReport> {
    let root = root.as_ref();
    let staging = tempfile::tempdir().map_err(AgentError::from)?;
    let project = init_project(staging.path(), options)
        .map_err(|error| AgentError::new(error.to_string()))?;
    let agent = init_external_agent_project(staging.path())
        .map_err(|error| AgentError::new(error.to_string()))?;
    copy_staged_project(staging.path(), root, options.reinstall_enabled())?;
    Ok(DoweProjectInitReport { project, agent })
}

pub fn init_external_agent_project(root: impl AsRef<Path>) -> HarnessResult<InitReport> {
    init_agent_project_with_skills(root, InitOptions::default(), &managed_agent_skills())
}

pub fn update_external_agent_project(root: impl AsRef<Path>) -> HarnessResult<InitReport> {
    let root = root.as_ref();
    if !root.join(".agents/manifest.json").is_file() {
        return Err(HarnessError::new(
            "Dowe project agent is not initialized; run `dowe agent init` first",
        ));
    }
    init_agent_project_with_skills(
        root,
        InitOptions {
            update_existing: true,
        },
        &managed_agent_skills(),
    )
}

fn copy_staged_project(staging: &Path, root: &Path, replace_existing: bool) -> AgentResult<()> {
    if fs::symlink_metadata(root)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(AgentError::at_path(
            root,
            "project initialization root cannot be a symbolic link",
        ));
    }
    let mut directories = Vec::new();
    let mut files = Vec::new();
    collect_staged_entries(staging, staging, &mut directories, &mut files)?;
    directories.sort();
    files.sort();
    for relative in &directories {
        let destination = root.join(relative);
        if let Ok(metadata) = fs::symlink_metadata(&destination) {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(conflict_error(relative));
            }
        }
    }
    for relative in &files {
        match fs::symlink_metadata(root.join(relative)) {
            Ok(metadata)
                if replace_existing && metadata.is_file() && !metadata.file_type().is_symlink() => {
            }
            Ok(_) => return Err(conflict_error(relative)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AgentError::from(error)),
        }
    }
    fs::create_dir_all(root).map_err(AgentError::from)?;
    for relative in directories {
        fs::create_dir_all(root.join(relative)).map_err(AgentError::from)?;
    }
    for relative in files {
        fs::copy(staging.join(&relative), root.join(&relative)).map_err(AgentError::from)?;
    }
    Ok(())
}

fn collect_staged_entries(
    staging: &Path,
    directory: &Path,
    directories: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> AgentResult<()> {
    let entries = fs::read_dir(directory).map_err(AgentError::from)?;
    for entry in entries {
        let entry = entry.map_err(AgentError::from)?;
        let file_type = entry.file_type().map_err(AgentError::from)?;
        let relative = entry
            .path()
            .strip_prefix(staging)
            .map_err(|error| AgentError::new(error.to_string()))?
            .to_path_buf();
        if file_type.is_symlink() {
            return Err(AgentError::at_path(
                &relative,
                "project initialization does not copy symbolic links",
            ));
        }
        if file_type.is_dir() {
            directories.push(relative.clone());
            collect_staged_entries(staging, &entry.path(), directories, files)?;
        } else if file_type.is_file() {
            files.push(relative);
        }
    }
    Ok(())
}

fn conflict_error(path: &Path) -> AgentError {
    AgentError::new(format!(
        "cannot initialize project because `{}` already exists",
        path.to_string_lossy().replace('\\', "/")
    ))
}
