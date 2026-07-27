use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::SourceImport;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn resolve_import(root: &Path, from_file: &Path, import: &SourceImport) -> DoweResult<PathBuf> {
    let value = import.path.as_str();
    if value.contains("://") || value.starts_with("http:") || value.starts_with("https:") {
        return Err(import_error(import, "imports cannot use URLs"));
    }
    if Path::new(value).is_absolute() {
        return Err(import_error(import, "imports cannot use absolute paths"));
    }

    let project_root = normalize_path(root.to_path_buf());
    let mut resolved = if let Some(value) = value.strip_prefix("@/") {
        if value.is_empty() {
            return Err(import_error(import, "`@/` must name a source module"));
        }
        normalize_path(project_root.join(value))
    } else if value.starts_with("./") || value.starts_with("../") {
        normalize_path(from_file.parent().unwrap_or(root).join(value))
    } else {
        return Err(import_error(
            import,
            "imports must use `./`, `../`, or the `@/` project-root alias",
        ));
    };
    match resolved.extension().and_then(|value| value.to_str()) {
        Some("dowe") => {}
        Some(_) => {
            return Err(import_error(
                import,
                "imports must resolve to `.dowe` files",
            ));
        }
        None => {
            resolved.set_extension("dowe");
        }
    }

    if !resolved.starts_with(&project_root) {
        return Err(import_error(
            import,
            "imports cannot leave the project root",
        ));
    }
    if matches!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some("config.dowe" | "theme.dowe" | "main.dowe" | "views.dowe")
    ) && resolved.parent() == Some(project_root.as_path())
    {
        return Err(import_error(
            import,
            "project entry files cannot be imported",
        ));
    }
    if resolved.starts_with(project_root.join(".dowe")) {
        return Err(import_error(
            import,
            "imports cannot load generated `.dowe` files",
        ));
    }
    if !resolved.is_file() {
        return Err(import_error(
            import,
            format!("import target `{}` does not exist", import.path),
        ));
    }
    let canonical_root = fs::canonicalize(&project_root).unwrap_or(project_root);
    let canonical_resolved = fs::canonicalize(&resolved)
        .map_err(|error| import_error(import, format!("cannot resolve import target: {error}")))?;
    if !canonical_resolved.starts_with(&canonical_root) {
        return Err(import_error(
            import,
            "imports cannot leave the project root",
        ));
    }

    Ok(resolved)
}

fn import_error(import: &SourceImport, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &import.location.path,
        format!(
            "{}:{}: unsupported import `{}`: {}",
            import.location.line,
            import.location.column,
            import.path,
            message.as_ref()
        ),
    )
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                output.pop();
            }
            _ => output.push(component.as_os_str()),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::source_ast::SourceLocation;
    use std::fs;
    use tempfile::tempdir;

    fn source_import(path: &Path, value: &str) -> SourceImport {
        SourceImport {
            local: "LabLayout".to_string(),
            path: value.to_string(),
            location: SourceLocation {
                path: path.to_path_buf(),
                relative_path: PathBuf::from("views/routes/view.dowe"),
                line: 1,
                column: 1,
                indent: 0,
            },
        }
    }

    #[test]
    fn resolves_project_root_alias_from_nested_modules() {
        let root = tempdir().expect("root");
        let from = root.path().join("views/routes/admin/view.dowe");
        let target = root.path().join("views/layouts/lab.dowe");
        fs::create_dir_all(from.parent().expect("from parent")).expect("from directory");
        fs::create_dir_all(target.parent().expect("target parent")).expect("target directory");
        fs::write(&target, "layout LabLayout\n  children\n").expect("target");

        let resolved = resolve_import(
            root.path(),
            &from,
            &source_import(&from, "@/views/layouts/lab"),
        )
        .expect("alias import");

        assert_eq!(resolved, target);
    }

    #[test]
    fn resolves_server_modules_from_project_root_alias() {
        let root = tempdir().expect("root");
        let from = root.path().join("main.dowe");
        let target = root.path().join("server/handlers/status.dowe");
        fs::create_dir_all(target.parent().expect("target parent")).expect("target directory");
        fs::write(&target, "handler getStatus req\n  return text:\"OK\"\n").expect("target");

        let resolved = resolve_import(
            root.path(),
            &from,
            &source_import(&from, "@/server/handlers/status"),
        )
        .expect("alias import");

        assert_eq!(resolved, target);
    }

    #[test]
    fn preserves_relative_import_resolution() {
        let root = tempdir().expect("root");
        let from = root.path().join("views/routes/view.dowe");
        let target = root.path().join("views/layouts/lab.dowe");
        fs::create_dir_all(from.parent().expect("from parent")).expect("from directory");
        fs::create_dir_all(target.parent().expect("target parent")).expect("target directory");
        fs::write(&target, "layout LabLayout\n  children\n").expect("target");

        let resolved = resolve_import(root.path(), &from, &source_import(&from, "../layouts/lab"))
            .expect("relative import");

        assert_eq!(resolved, target);
    }

    #[test]
    fn rejects_alias_escape_from_project_root() {
        let root = tempdir().expect("root");
        let from = root.path().join("views/routes/view.dowe");
        fs::create_dir_all(from.parent().expect("from parent")).expect("from directory");

        let error = resolve_import(
            root.path(),
            &from,
            &source_import(&from, "@/../assets/model"),
        )
        .expect_err("asset escape");

        assert!(
            error
                .to_string()
                .contains("imports cannot leave the project root")
        );
    }
}
