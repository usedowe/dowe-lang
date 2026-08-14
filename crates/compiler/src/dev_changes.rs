use std::fs;
use std::path::{Component, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevChangeScope {
    Views,
    Project,
}

pub fn classify_dev_changes(root: &Path, paths: &[String]) -> DevChangeScope {
    if !paths.is_empty() && paths.iter().all(|path| is_view_change(root, path)) {
        DevChangeScope::Views
    } else {
        DevChangeScope::Project
    }
}

fn is_view_change(root: &Path, path: &str) -> bool {
    if path == "theme.dowe" || path.starts_with("i18n/") || path.starts_with("icons/") {
        return true;
    }

    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("dowe")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return false;
    }

    fs::read_to_string(root.join(relative))
        .ok()
        .and_then(|source| source_declaration(&source).map(str::to_string))
        .is_some_and(|declaration| {
            matches!(
                declaration.as_str(),
                "views" | "layout" | "page" | "component" | "store"
            )
        })
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
    use super::{DevChangeScope, classify_dev_changes};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn classifies_view_graph_changes_without_server_invalidation() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("views/routes")).expect("routes");
        fs::create_dir_all(temp.path().join("layouts")).expect("layouts");
        fs::create_dir_all(temp.path().join("pages")).expect("pages");
        fs::write(
            temp.path().join("views/routes/home.dowe"),
            "import HomePage from \"@/pages/home\"\nviews HomeViews",
        )
        .expect("views");
        fs::write(temp.path().join("layouts/app.dowe"), "layout AppLayout").expect("layout");
        fs::write(temp.path().join("pages/home.dowe"), "page HomePage").expect("page");
        let paths = [
            "views/routes/home.dowe".to_string(),
            "layouts/app.dowe".to_string(),
            "pages/home.dowe".to_string(),
            "i18n/en.dowe".to_string(),
            "icons/web/favicon.png".to_string(),
            "theme.dowe".to_string(),
        ];

        assert_eq!(
            classify_dev_changes(temp.path(), &paths),
            DevChangeScope::Views
        );
    }

    #[test]
    fn conservatively_classifies_shared_and_server_changes() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("server")).expect("server");
        fs::create_dir_all(temp.path().join("types")).expect("types");
        fs::write(temp.path().join("main.dowe"), "main").expect("main");
        fs::write(
            temp.path().join("server/endpoints.dowe"),
            "endpoints ApiRoutes",
        )
        .expect("server");
        fs::write(temp.path().join("types/user.dowe"), "type User").expect("type");
        for path in [
            "main.dowe",
            "server/endpoints.dowe",
            "types/user.dowe",
            ".env",
            ".env.example",
        ] {
            assert_eq!(
                classify_dev_changes(temp.path(), &[path.to_string()]),
                DevChangeScope::Project
            );
        }
    }

    #[test]
    fn classifies_deleted_and_escaping_paths_conservatively() {
        let temp = TempDir::new().expect("tempdir");

        for path in ["views/pages/deleted.dowe", "../outside.dowe"] {
            assert_eq!(
                classify_dev_changes(temp.path(), &[path.to_string()]),
                DevChangeScope::Project
            );
        }
        assert_eq!(
            classify_dev_changes(temp.path(), &[]),
            DevChangeScope::Project
        );
    }
}
