use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::SourceImport;
use std::path::{Component, Path, PathBuf};

pub fn resolve_import(root: &Path, from_file: &Path, import: &SourceImport) -> DoweResult<PathBuf> {
    let value = import.path.as_str();
    if value.contains("://") || value.starts_with("http:") || value.starts_with("https:") {
        return Err(import_error(import, "imports cannot use URLs"));
    }
    if !value.starts_with("./") && !value.starts_with("../") {
        return Err(import_error(
            import,
            "imports must be static relative paths inside `/src`",
        ));
    }

    let raw = Path::new(value);
    if raw.is_absolute() {
        return Err(import_error(import, "imports cannot use absolute paths"));
    }

    let mut resolved = normalize_path(from_file.parent().unwrap_or(root).join(raw));
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

    let src = normalize_path(root.join("src"));
    if !resolved.starts_with(&src) {
        return Err(import_error(import, "imports cannot leave `/src`"));
    }
    if resolved == src.join("config.dowe") {
        return Err(import_error(
            import,
            "`src/config.dowe` is project configuration and cannot be imported",
        ));
    }
    if resolved.starts_with(root.join(".dowe")) {
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
