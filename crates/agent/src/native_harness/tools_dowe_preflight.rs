const PREFLIGHT_SOURCE_LIMIT: u64 = 8 * 1024 * 1024;
const PREFLIGHT_ERROR_LIMIT: usize = 16 * 1024;

impl HarnessTools {
    fn preflight_dowe_write_batch(
        &self,
        virtual_files: &BTreeMap<PathBuf, Option<String>>,
    ) -> AgentResult<()> {
        let has_dowe_write = virtual_files.keys().any(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dowe"))
        });
        if !has_dowe_write {
            return Ok(());
        }
        let has_candidate_main = virtual_files
            .keys()
            .any(|path| path == &self.root.join("main.dowe"));
        if !is_dowe_project_root(&self.root) && !has_candidate_main {
            return Ok(());
        }

        let staging = PreflightProject::new()?;
        copy_dowe_sources(&self.root, staging.path())?;
        apply_virtual_dowe_sources(staging.path(), &self.root, virtual_files)?;
        match compile_dowe_preflight(staging.path())? {
            Ok(_) => Ok(()),
            Err(error) => Err(AgentError::new(format!(
                "Dowe preflight rejected this write batch; no files were changed. Repair the compiler diagnostics and submit the related source files together:\n{}",
                bounded_preflight_error(staging.path(), &error.to_string())
            ))),
        }
    }
}

fn compile_dowe_preflight(
    root: &Path,
) -> AgentResult<Result<dowe_compiler::CompiledProject, dowe_compiler::DoweError>> {
    let root = root.to_path_buf();
    std::thread::Builder::new()
        .name("dowe-agent-preflight".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || dowe_compiler::compile_dev(root))
        .map_err(|error| {
            AgentError::new(format!("Dowe preflight thread could not start: {error}"))
        })?
        .join()
        .map_err(|_| AgentError::new("Dowe preflight compiler thread panicked"))
}

struct PreflightProject {
    path: PathBuf,
}

impl PreflightProject {
    fn new() -> AgentResult<Self> {
        let path = std::env::temp_dir().join(format!("dowe-agent-preflight-{}", identifier()));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PreflightProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn copy_dowe_sources(source: &Path, destination: &Path) -> AgentResult<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        if file_type.is_symlink() || skipped_preflight_directory(&name) {
            continue;
        }
        let destination_path = destination.join(&name);
        if file_type.is_dir() {
            copy_dowe_sources(&source_path, &destination_path)?;
        } else if file_type.is_file() && is_dowe_source(&source_path) {
            let size = file_type
                .is_file()
                .then(|| fs::metadata(&source_path))
                .transpose()?
                .map(|metadata| metadata.len())
                .unwrap_or_default();
            if size > PREFLIGHT_SOURCE_LIMIT {
                return Err(AgentError::new(format!(
                    "Dowe preflight source exceeds {} MiB: {}",
                    PREFLIGHT_SOURCE_LIMIT / 1024 / 1024,
                    source_path
                        .strip_prefix(source)
                        .unwrap_or(&source_path)
                        .display()
                )));
            }
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

fn skipped_preflight_directory(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(".git" | ".dowe" | ".agents" | ".pi" | "node_modules" | "target")
    )
}

fn is_dowe_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dowe"))
}

fn apply_virtual_dowe_sources(
    staging: &Path,
    root: &Path,
    virtual_files: &BTreeMap<PathBuf, Option<String>>,
) -> AgentResult<()> {
    for (path, content) in virtual_files {
        if !is_dowe_source(path) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| AgentError::new("Dowe preflight candidate escaped the project root"))?;
        let destination = staging.join(relative);
        match content {
            Some(content) => {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(destination, content)?;
            }
            None => {
                if destination.is_file() {
                    fs::remove_file(destination)?;
                }
            }
        }
    }
    Ok(())
}

fn bounded_preflight_error(staging: &Path, error: &str) -> String {
    let staging = staging.to_string_lossy();
    let mut message = error.replace(staging.as_ref(), "");
    message = message.replace('\\', "/");
    let message = message.trim_start_matches('/');
    message.chars().take(PREFLIGHT_ERROR_LIMIT).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_call(id: &str, path: &str, content: &str) -> ToolCall {
        let skill = if path == "main.dowe" {
            "core/configuration"
        } else {
            "views"
        };
        ToolCall::new(
            id,
            "write_file",
            json!({
                "path": path,
                "content": content,
                "skill": skill,
                "reason": "author the Dowe view source"
            }),
        )
    }

    fn view_sources(main: &str, routes: &str, layout: &str, page: &str) -> Vec<ToolCall> {
        vec![
            write_call("main", "main.dowe", main),
            write_call("routes", "views/routes/view.dowe", routes),
            write_call("layout", "views/layouts/app.dowe", layout),
            write_call("page", "views/pages/home.dowe", page),
        ]
    }

    #[test]
    fn rejects_route_export_mismatch_before_any_file_is_written() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        let calls = view_sources(
            "import viewRoutes from \"@/views/routes/view\"\n\nmain\n  views:viewRoutes\n",
            "import AppLayout from \"@/views/layouts/app\"\nimport HomePage from \"@/views/pages/home\"\n\nviews siteRoutes\n  group path:\"/\" layout:AppLayout\n    route path:\"\" page:HomePage\n",
            "layout AppLayout\n  Box\n    children\n",
            "page HomePage\n  Text\n    \"Home\"\n",
        );

        let error = tools
            .prepare_text_write_batch(&calls, HarnessRole::Execute)
            .expect_err("route export mismatch");

        assert!(
            error
                .to_string()
                .contains("export `siteRoutes` does not match import `viewRoutes`")
        );
        assert!(!root.path().join("main.dowe").exists());
        assert!(tools.pending.is_empty());
    }

    #[test]
    fn rejects_invalid_component_scheme_before_any_file_is_written() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        let calls = view_sources(
            "import viewRoutes from \"@/views/routes/view\"\n\nmain\n  views:viewRoutes\n",
            "import AppLayout from \"@/views/layouts/app\"\nimport HomePage from \"@/views/pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:AppLayout\n    route path:\"\" page:HomePage\n",
            "layout AppLayout\n  Box\n    children\n",
            "page HomePage\n  Button scheme:\"surface\"\n    \"Home\"\n",
        );

        let error = tools
            .prepare_text_write_batch(&calls, HarnessRole::Execute)
            .expect_err("invalid scheme");

        assert!(
            error
                .to_string()
                .contains("invalid value for prop `scheme`")
        );
        assert!(
            error
                .to_string()
                .contains("expected primary, secondary, accent")
        );
        assert!(!root.path().join("views/pages/home.dowe").exists());
        assert!(tools.pending.is_empty());
    }

    #[test]
    fn accepts_a_coherent_dowe_batch_without_creating_preflight_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        let calls = view_sources(
            "import viewRoutes from \"@/views/routes/view\"\n\nmain\n  views:viewRoutes\n",
            "import AppLayout from \"@/views/layouts/app\"\nimport HomePage from \"@/views/pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:AppLayout\n    route path:\"\" page:HomePage\n",
            "layout AppLayout\n  Box\n    children\n",
            "page HomePage\n  Button scheme:\"primary\"\n    \"Home\"\n",
        );

        let approvals = tools
            .prepare_text_write_batch(&calls, HarnessRole::Execute)
            .expect("coherent Dowe batch");

        assert_eq!(approvals.len(), calls.len());
        assert!(!root.path().join("main.dowe").exists());
        assert_eq!(tools.pending.len(), calls.len());
    }
}
