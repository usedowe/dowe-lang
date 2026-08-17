use crate::error::{DoweError, DoweResult};
use crate::model::{CompileEnvironment, CompiledProject, ViewPlatform};
use crate::parser::ViewModuleCache;
use crate::pipeline::{compile_project, complete_dev_app_outputs};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DevCompilerSessionStats {
    pub module_cache_entries: usize,
    pub module_cache_hits: usize,
    pub module_cache_misses: usize,
}

#[derive(Clone)]
pub struct DevCompilerSession {
    root: PathBuf,
    platforms: BTreeSet<ViewPlatform>,
    module_cache: ViewModuleCache,
}

impl DevCompilerSession {
    pub fn new(
        root: impl AsRef<Path>,
        platforms: impl IntoIterator<Item = ViewPlatform>,
    ) -> DoweResult<Self> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|error| DoweError::at_path(root.as_ref(), error.to_string()))?;
        Ok(Self {
            root,
            platforms: platforms.into_iter().collect(),
            module_cache: ViewModuleCache::default(),
        })
    }

    pub fn compile_initial(&mut self, compile_server: bool) -> DoweResult<CompiledProject> {
        self.compile(compile_server, None, true)
    }

    pub fn compile_initial_web(&mut self, compile_server: bool) -> DoweResult<CompiledProject> {
        self.compile(compile_server, None, false)
    }

    pub fn complete_dev_app_outputs(&self, project: &mut CompiledProject) -> DoweResult<()> {
        complete_dev_app_outputs(project, &self.platforms)
    }

    pub fn rebuild(
        &mut self,
        paths: &[String],
        compile_server: bool,
    ) -> DoweResult<CompiledProject> {
        self.invalidate(paths);
        self.compile(compile_server, None, true)
    }

    pub fn rebuild_from(
        &mut self,
        paths: &[String],
        compile_server: bool,
        previous: &CompiledProject,
    ) -> DoweResult<CompiledProject> {
        self.invalidate(paths);
        self.compile(compile_server, Some(previous), true)
    }

    pub fn rebuild_snapshot_from(
        &mut self,
        paths: &[String],
        compile_server: bool,
        previous: &CompiledProject,
    ) -> DoweResult<CompiledProject> {
        self.invalidate(paths);
        self.compile(compile_server, Some(previous), false)
    }

    pub fn stats(&self) -> DevCompilerSessionStats {
        DevCompilerSessionStats {
            module_cache_entries: self.module_cache.len(),
            module_cache_hits: self.module_cache.hits(),
            module_cache_misses: self.module_cache.misses(),
        }
    }

    fn compile(
        &mut self,
        compile_server: bool,
        previous: Option<&CompiledProject>,
        compile_apps: bool,
    ) -> DoweResult<CompiledProject> {
        compile_project(
            &self.root,
            CompileEnvironment::Development,
            false,
            compile_server,
            !self.platforms.is_empty(),
            compile_apps,
            Some(self.platforms.clone()),
            Some(&mut self.module_cache),
            previous,
        )
    }

    fn invalidate(&mut self, paths: &[String]) {
        let mut direct = Vec::new();
        for path in paths {
            match cache_invalidation(&self.root, path) {
                CacheInvalidation::Module(path) => direct.push(path),
                CacheInvalidation::RouteGraph => {}
                CacheInvalidation::All => {
                    self.module_cache.clear();
                    return;
                }
            }
        }
        for path in direct {
            self.module_cache.remove(&path);
        }
    }
}

enum CacheInvalidation {
    Module(PathBuf),
    RouteGraph,
    All,
}

fn cache_invalidation(root: &Path, path: &str) -> CacheInvalidation {
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative.extension().and_then(|value| value.to_str()) != Some("dowe")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return CacheInvalidation::All;
    }
    let absolute = root.join(relative);
    let Ok(source) = fs::read_to_string(&absolute) else {
        return CacheInvalidation::All;
    };
    match source_declaration(&source) {
        Some("page" | "layout") => CacheInvalidation::Module(absolute),
        Some("views") => CacheInvalidation::RouteGraph,
        _ => CacheInvalidation::All,
    }
}

fn source_declaration(source: &str) -> Option<&str> {
    source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with([' ', '\t']))
        .map(str::trim)
        .filter(|line| !line.starts_with("import "))
        .find_map(|line| line.split_whitespace().next())
}

#[cfg(test)]
mod tests {
    use super::DevCompilerSession;
    use crate::ViewPlatform;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn reuses_unchanged_view_modules_after_a_page_change() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let mut session =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("session");

        session.compile_initial(false).expect("initial compile");
        let initial = session.stats();
        assert_eq!(initial.module_cache_entries, 3);
        assert_eq!(initial.module_cache_hits, 0);
        assert_eq!(initial.module_cache_misses, 3);

        fs::write(
            temp.path().join("pages/login.dowe"),
            "page LoginPage\n  Text\n    \"Changed\"\n",
        )
        .expect("page");
        let project = session
            .rebuild(&["pages/login.dowe".to_string()], false)
            .expect("incremental rebuild");
        let rebuilt = session.stats();

        assert_eq!(rebuilt.module_cache_entries, 3);
        assert_eq!(rebuilt.module_cache_hits - initial.module_cache_hits, 2);
        assert_eq!(rebuilt.module_cache_misses - initial.module_cache_misses, 1);
        assert!(project.web.pages[0].body_html.contains("Changed"));
    }

    #[test]
    fn shared_module_changes_clear_the_module_cache() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let mut session =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("session");
        session.compile_initial(false).expect("initial compile");
        let initial = session.stats();

        fs::create_dir_all(temp.path().join("components")).expect("components");
        fs::write(
            temp.path().join("components/card.dowe"),
            "component Card\n  Text\n    \"Card\"\n",
        )
        .expect("component");
        let error = session
            .rebuild(&["components/card.dowe".to_string()], false)
            .expect("conservative rebuild");
        let rebuilt = session.stats();

        assert_eq!(rebuilt.module_cache_entries, 3);
        assert_eq!(rebuilt.module_cache_hits, initial.module_cache_hits);
        assert_eq!(rebuilt.module_cache_misses - initial.module_cache_misses, 3);
        assert_eq!(error.web.pages.len(), 2);
    }

    #[test]
    fn layout_changes_rebuild_dependent_routes_and_reuse_pages() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let mut session =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("session");
        session.compile_initial(false).expect("initial compile");
        let initial = session.stats();

        fs::write(
            temp.path().join("layouts/app.dowe"),
            "layout AppLayout\n  Box\n    Text\n      \"Changed layout\"\n    children\n",
        )
        .expect("layout");
        let project = session
            .rebuild(&["layouts/app.dowe".to_string()], false)
            .expect("incremental rebuild");
        let rebuilt = session.stats();

        assert_eq!(rebuilt.module_cache_hits - initial.module_cache_hits, 2);
        assert_eq!(rebuilt.module_cache_misses - initial.module_cache_misses, 1);
        assert!(
            project
                .web
                .pages
                .iter()
                .all(|page| page.body_html.contains("Changed layout"))
        );
    }

    #[test]
    fn rebuild_snapshot_publishes_web_without_generating_native_apps() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let mut session = DevCompilerSession::new(
            temp.path(),
            [ViewPlatform::Web, ViewPlatform::Android, ViewPlatform::Ios],
        )
        .expect("session");
        let initial = session.compile_initial(false).expect("initial compile");
        assert!(!initial.apps.files.is_empty());

        fs::write(
            temp.path().join("pages/login.dowe"),
            "page LoginPage\n  Text\n    \"Changed\"\n",
        )
        .expect("page");
        let project = session
            .rebuild_snapshot_from(&["pages/login.dowe".to_string()], false, &initial)
            .expect("snapshot rebuild");

        assert!(project.web.pages[0].body_html.contains("Changed"));
        assert!(project.apps.files.is_empty());
    }

    fn write_fixture(root: &std::path::Path) {
        fs::create_dir_all(root.join("routes")).expect("routes");
        fs::create_dir_all(root.join("layouts")).expect("layouts");
        fs::create_dir_all(root.join("pages")).expect("pages");
        fs::write(
            root.join("main.dowe"),
            "import AppViews from \"@/routes/app\"\n\nmain\n  views:AppViews\n",
        )
        .expect("main");
        fs::write(
            root.join("routes/app.dowe"),
            "import AppLayout from \"@/layouts/app\"\nimport LoginPage from \"@/pages/login\"\nimport AboutPage from \"@/pages/about\"\n\nviews AppViews\n  group path:\"/\" layout:AppLayout\n    route path:\"\" page:LoginPage\n    route path:\"about\" page:AboutPage\n",
        )
        .expect("routes");
        fs::write(
            root.join("layouts/app.dowe"),
            "layout AppLayout\n  Box\n    children\n",
        )
        .expect("layout");
        fs::write(
            root.join("pages/login.dowe"),
            "page LoginPage\n  Text\n    \"Login\"\n",
        )
        .expect("login");
        fs::write(
            root.join("pages/about.dowe"),
            "page AboutPage\n  Text\n    \"About\"\n",
        )
        .expect("about");
    }
}
