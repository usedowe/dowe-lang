use crate::error::{RuntimeError, RuntimeResult};
use crate::init_templates::files_for_template;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectTemplate {
    Blank,
    ClinicDesk,
    CommerceOps,
    SupportConsole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectTemplateKind {
    Blank,
    Example,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TemplateFile {
    path: &'static str,
    content: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitProjectOptions {
    template: ProjectTemplate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitProjectReport {
    template: ProjectTemplate,
    created: Vec<PathBuf>,
}

struct PlannedFile {
    relative: PathBuf,
    absolute: PathBuf,
    content: &'static str,
}

impl ProjectTemplate {
    pub fn canonical() -> &'static [Self] {
        &[
            Self::Blank,
            Self::ClinicDesk,
            Self::CommerceOps,
            Self::SupportConsole,
        ]
    }

    pub fn examples() -> &'static [Self] {
        &[Self::ClinicDesk, Self::CommerceOps, Self::SupportConsole]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::ClinicDesk => "clinic-desk",
            Self::CommerceOps => "commerce-ops",
            Self::SupportConsole => "support-console",
        }
    }

    pub fn kind(self) -> ProjectTemplateKind {
        match self {
            Self::Blank => ProjectTemplateKind::Blank,
            Self::ClinicDesk | Self::CommerceOps | Self::SupportConsole => {
                ProjectTemplateKind::Example
            }
        }
    }

    pub fn is_example(self) -> bool {
        self.kind() == ProjectTemplateKind::Example
    }
}

impl Display for ProjectTemplate {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProjectTemplate {
    type Err = RuntimeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "blank" => Ok(Self::Blank),
            "clinic-desk" => Ok(Self::ClinicDesk),
            "commerce-ops" => Ok(Self::CommerceOps),
            "support-console" => Ok(Self::SupportConsole),
            _ => Err(RuntimeError::new(format!(
                "unknown init template `{value}`"
            ))),
        }
    }
}

impl TemplateFile {
    pub(crate) const fn new(path: &'static str, content: &'static str) -> Self {
        Self { path, content }
    }
}

impl InitProjectOptions {
    pub fn new(template: ProjectTemplate) -> Self {
        Self { template }
    }

    pub fn template(&self) -> ProjectTemplate {
        self.template
    }
}

impl Default for InitProjectOptions {
    fn default() -> Self {
        Self::new(ProjectTemplate::Blank)
    }
}

impl InitProjectReport {
    fn new(template: ProjectTemplate, created: Vec<PathBuf>) -> Self {
        Self { template, created }
    }

    pub fn template(&self) -> ProjectTemplate {
        self.template
    }

    pub fn created(&self) -> &[PathBuf] {
        &self.created
    }
}

pub fn available_project_templates() -> &'static [ProjectTemplate] {
    ProjectTemplate::canonical()
}

pub fn available_project_examples() -> &'static [ProjectTemplate] {
    ProjectTemplate::examples()
}

pub fn init_project(
    root: impl AsRef<Path>,
    options: InitProjectOptions,
) -> RuntimeResult<InitProjectReport> {
    write_project_files(
        root.as_ref(),
        options.template(),
        files_for_template(options.template()),
    )
}

pub(crate) fn write_project_files(
    root: &Path,
    template: ProjectTemplate,
    files: &[TemplateFile],
) -> RuntimeResult<InitProjectReport> {
    let planned = plan_files(root, files)?;
    reject_duplicate_destinations(&planned)?;
    reject_existing_destinations(&planned)?;
    write_planned_files(&planned)?;
    Ok(InitProjectReport::new(
        template,
        planned.into_iter().map(|file| file.relative).collect(),
    ))
}

fn plan_files(root: &Path, files: &[TemplateFile]) -> RuntimeResult<Vec<PlannedFile>> {
    files
        .iter()
        .map(|file| {
            let relative = safe_relative_path(file.path)?;
            Ok(PlannedFile {
                absolute: root.join(&relative),
                relative,
                content: file.content,
            })
        })
        .collect()
}

fn reject_duplicate_destinations(files: &[PlannedFile]) -> RuntimeResult<()> {
    let mut seen = BTreeSet::new();
    let mut duplicate = None;

    for file in files {
        if !seen.insert(file.relative.clone()) {
            duplicate = Some(slash_path(&file.relative));
            break;
        }
    }

    if let Some(path) = duplicate {
        Err(RuntimeError::new(format!(
            "init template contains duplicate file `{path}`"
        )))
    } else {
        Ok(())
    }
}

fn reject_existing_destinations(files: &[PlannedFile]) -> RuntimeResult<()> {
    let conflicts = files
        .iter()
        .filter(|file| file.absolute.exists())
        .map(|file| slash_path(&file.relative))
        .collect::<Vec<_>>();

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::new(format!(
            "cannot initialize project because these files already exist: {}",
            conflicts.join(", ")
        )))
    }
}

fn write_planned_files(files: &[PlannedFile]) -> RuntimeResult<()> {
    for file in files {
        if let Some(parent) = file.absolute.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file.absolute, file.content)?;
    }

    Ok(())
}

fn safe_relative_path(value: &str) -> RuntimeResult<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(unsafe_template_path_error(value));
    }

    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => safe.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(unsafe_template_path_error(value));
            }
        }
    }

    if safe.as_os_str().is_empty() {
        Err(unsafe_template_path_error(value))
    } else {
        Ok(safe)
    }
}

fn unsafe_template_path_error(path: &str) -> RuntimeError {
    RuntimeError::new(format!("unsafe init template path `{path}`"))
}

fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
