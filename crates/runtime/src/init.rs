use crate::error::{RuntimeError, RuntimeResult};
use crate::init_templates::files_for_options;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectTemplate {
    Crud,
    Blank,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TemplateFile {
    path: &'static str,
    content: Cow<'static, str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitProjectOptions {
    template: ProjectTemplate,
    i18n: bool,
    reinstall: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitProjectReport {
    template: ProjectTemplate,
    i18n: bool,
    reinstalled: bool,
    created: Vec<PathBuf>,
}

struct PlannedFile {
    relative: PathBuf,
    absolute: PathBuf,
    content: Cow<'static, str>,
}

impl ProjectTemplate {
    pub fn canonical() -> &'static [Self] {
        &[Self::Crud, Self::Blank]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Crud => "crud",
            Self::Blank => "blank",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Crud => "CRUD",
            Self::Blank => "blank",
        }
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
            "crud" => Ok(Self::Crud),
            "blank" => Ok(Self::Blank),
            _ => Err(RuntimeError::new(format!(
                "unknown init template `{value}`"
            ))),
        }
    }
}

impl TemplateFile {
    pub(crate) const fn new(path: &'static str, content: &'static str) -> Self {
        Self {
            path,
            content: Cow::Borrowed(content),
        }
    }

    pub(crate) fn owned(path: &'static str, content: String) -> Self {
        Self {
            path,
            content: Cow::Owned(content),
        }
    }

    pub(crate) fn path(&self) -> &'static str {
        self.path
    }

    pub(crate) fn content(&self) -> &str {
        &self.content
    }
}

impl InitProjectOptions {
    pub fn new(template: ProjectTemplate) -> Self {
        Self {
            template,
            i18n: false,
            reinstall: false,
        }
    }

    pub fn template(&self) -> ProjectTemplate {
        self.template
    }

    pub fn with_i18n(mut self, enabled: bool) -> Self {
        self.i18n = enabled;
        self
    }

    pub fn i18n_enabled(&self) -> bool {
        self.i18n
    }

    pub fn with_reinstall(mut self, enabled: bool) -> Self {
        self.reinstall = enabled;
        self
    }

    pub fn reinstall_enabled(&self) -> bool {
        self.reinstall
    }

    fn validate(&self) -> RuntimeResult<()> {
        Ok(())
    }
}

impl Default for InitProjectOptions {
    fn default() -> Self {
        Self::new(ProjectTemplate::Blank)
    }
}

impl InitProjectReport {
    fn new(options: InitProjectOptions, created: Vec<PathBuf>) -> Self {
        Self {
            template: options.template,
            i18n: options.i18n,
            reinstalled: options.reinstall,
            created,
        }
    }

    pub fn template(&self) -> ProjectTemplate {
        self.template
    }

    pub fn i18n_enabled(&self) -> bool {
        self.i18n
    }

    pub fn reinstalled(&self) -> bool {
        self.reinstalled
    }

    pub fn created(&self) -> &[PathBuf] {
        &self.created
    }
}

pub fn available_project_templates() -> &'static [ProjectTemplate] {
    ProjectTemplate::canonical()
}

pub fn has_dowe_project_marker(root: impl AsRef<Path>) -> bool {
    fs::symlink_metadata(root.as_ref().join("main.dowe")).is_ok()
}

pub fn init_project(
    root: impl AsRef<Path>,
    options: InitProjectOptions,
) -> RuntimeResult<InitProjectReport> {
    options.validate()?;
    let files = files_for_options(options);
    write_project_files(root.as_ref(), options, &files)
}

pub(crate) fn write_project_files(
    root: &Path,
    options: InitProjectOptions,
    files: &[TemplateFile],
) -> RuntimeResult<InitProjectReport> {
    let planned = plan_files(root, files)?;
    reject_duplicate_destinations(&planned)?;
    reject_existing_destinations(&planned, options.reinstall)?;
    write_planned_files(&planned)?;
    Ok(InitProjectReport::new(
        options,
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
                content: file.content.clone(),
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

fn reject_existing_destinations(
    files: &[PlannedFile],
    replace_existing: bool,
) -> RuntimeResult<()> {
    let mut conflicts = Vec::new();

    for file in files {
        match fs::symlink_metadata(&file.absolute) {
            Ok(metadata)
                if replace_existing && metadata.is_file() && !metadata.file_type().is_symlink() => {
            }
            Ok(_) => conflicts.push(slash_path(&file.relative)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

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
        fs::write(&file.absolute, file.content.as_bytes())?;
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
