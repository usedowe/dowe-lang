use super::*;

pub fn read_workflow_plan(root: &std::path::Path, relative: &str) -> AgentResult<WorkflowPlan> {
    use std::path::{Component, Path};
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AgentError::new(
            "plan must be a project-relative path without traversal",
        ));
    }
    let path = root.join(relative);
    for ancestor in path.ancestors() {
        if std::fs::symlink_metadata(ancestor)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(AgentError::new("plan must not traverse symlinks"));
        }
    }
    let metadata = std::fs::metadata(&path)?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Err(AgentError::new(
            "plan must be a regular JSON file of at most 1 MiB",
        ));
    }
    let plan: WorkflowPlan = serde_json::from_slice(&std::fs::read(path)?)?;
    plan.validate_draft()?;
    Ok(plan)
}
