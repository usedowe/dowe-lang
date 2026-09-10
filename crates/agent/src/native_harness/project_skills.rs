use crate::{AgentError, AgentResult};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const MAX_PROJECT_SKILL_FILE_BYTES: usize = 64 * 1024;
pub const MAX_PROJECT_SKILL_AGGREGATE_BYTES: usize = 256 * 1024;
pub const MAX_PROJECT_SKILL_COUNT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSkillSummary {
    pub id: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSkill {
    pub summary: ProjectSkillSummary,
    pub content: String,
}

/// Discover only the immediate, regular project-local skill files.
///
/// This function performs no writes, network access, caching, or provider work. The
/// returned summaries intentionally do not contain skill bodies.
pub fn discover_project_skills(root: impl AsRef<Path>) -> AgentResult<Vec<ProjectSkillSummary>> {
    let root = canonical_project_root(root.as_ref())?;
    let skills_dir = root.join(".agents").join("skills");
    if !checked_directory(&root.join(".agents"), &root)? {
        return Ok(Vec::new());
    }
    if !checked_directory(&skills_dir, &root)? {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&skills_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    if entries.len() > MAX_PROJECT_SKILL_COUNT {
        return Err(AgentError::at_path(&skills_dir, format!("project skill count exceeds {MAX_PROJECT_SKILL_COUNT}")));
    }

    let mut result = Vec::with_capacity(entries.len());
    let mut total = 0;
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        validate_id(&name)?;
        let directory = entry.path();
        if !checked_directory(&directory, &root)? {
            return Err(AgentError::at_path(&directory, "project skill directory must be a regular directory"));
        }
        let skill = load_project_skill_from_canonical(&root, &name, total)?;
        total += skill.summary.bytes;
        result.push(skill.summary);
    }
    Ok(result)
}

/// Load one project-local skill body after applying the same confinement checks as discovery.
pub fn load_project_skill(root: impl AsRef<Path>, id: &str) -> AgentResult<ProjectSkill> {
    let root = canonical_project_root(root.as_ref())?;
    validate_id(id)?;
    load_project_skill_from_canonical(&root, id, 0)
}

fn load_project_skill_from_canonical(root: &Path, id: &str, total: usize) -> AgentResult<ProjectSkill> {
    let directory = root.join(".agents").join("skills").join(id);
    if !checked_directory(&directory, root)? {
        return Err(AgentError::at_path(&directory, "project skill directory is not a regular directory"));
    }
    let path = directory.join("SKILL.md");
    ensure_confined(&path, root)?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| AgentError::at_path(&path, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AgentError::at_path(&path, "project skill must be a regular, non-symlink file"));
    }
    let bytes = fs::read(&path)?;
    if bytes.len() > MAX_PROJECT_SKILL_FILE_BYTES {
        return Err(AgentError::at_path(&path, format!("file exceeds {MAX_PROJECT_SKILL_FILE_BYTES} byte limit")));
    }
    if total.saturating_add(bytes.len()) > MAX_PROJECT_SKILL_AGGREGATE_BYTES {
        return Err(AgentError::at_path(&path, format!("aggregate skill limit is {MAX_PROJECT_SKILL_AGGREGATE_BYTES} bytes")));
    }
    let content = String::from_utf8(bytes.clone())
        .map_err(|_| AgentError::at_path(&path, "project skill is not valid UTF-8"))?;
    Ok(ProjectSkill {
        summary: ProjectSkillSummary {
            id: id.into(),
            path: format!(".agents/skills/{id}/SKILL.md"),
            hash: super::digest(&bytes),
            bytes: bytes.len(),
        },
        content,
    })
}

fn canonical_project_root(root: &Path) -> AgentResult<PathBuf> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| AgentError::at_path(root, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AgentError::at_path(root, "project root must be a regular directory"));
    }
    root.canonicalize().map_err(|error| AgentError::at_path(root, error.to_string()))
}

fn checked_directory(path: &Path, root: &Path) -> AgentResult<bool> {
    ensure_confined(path, root)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(AgentError::at_path(path, error.to_string())),
    };
    if metadata.file_type().is_symlink() {
        return Err(AgentError::at_path(path, "symlink ancestors are not allowed"));
    }
    Ok(metadata.is_dir())
}

fn ensure_confined(path: &Path, root: &Path) -> AgentResult<()> {
    if !path.starts_with(root) {
        return Err(AgentError::at_path(path, "path escapes project root"));
    }
    Ok(())
}

fn validate_id(id: &str) -> AgentResult<()> {
    if id.is_empty() || id == "." || id == ".." || id.starts_with('.') || id.contains('/') || id.contains('\\')
        || id.chars().any(|character| character.is_control())
        || Path::new(id).is_absolute()
        || Path::new(id).components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AgentError::new("invalid project skill id"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_sorted_summaries_and_loads_exact_bytes() {
        let root = tempfile::tempdir().unwrap();
        for id in ["zeta", "alpha"] {
            let dir = root.path().join(".agents/skills").join(id);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), id.as_bytes()).unwrap();
        }
        let skills = discover_project_skills(root.path()).unwrap();
        assert_eq!(skills.iter().map(|skill| skill.id.as_str()).collect::<Vec<_>>(), ["alpha", "zeta"]);
        assert_eq!(load_project_skill(root.path(), "alpha").unwrap().content, "alpha");
        assert!(load_project_skill(root.path(), "../alpha").is_err());
        assert!(load_project_skill(root.path(), "/alpha").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_skill_directory() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(".agents/skills")).unwrap();
        fs::create_dir(outside.path().join("skill")).unwrap();
        fs::write(outside.path().join("skill/SKILL.md"), "outside").unwrap();
        symlink(outside.path().join("skill"), root.path().join(".agents/skills/link")).unwrap();
        assert!(discover_project_skills(root.path()).is_err());
    }
}
