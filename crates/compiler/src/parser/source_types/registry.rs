use super::resolution::{parse_type_reference, resolve_type};
use super::{node_error, validate_identifier};
use crate::error::{DoweError, DoweResult};
use crate::model::DoweType;
use crate::parser::source_ast::{SourceFile, SourceImport, SourceNode, SourceValue};
use crate::parser::source_imports::resolve_import;
use crate::parser::source_parser::parse_source_file;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct TypeRegistry {
    definitions: HashMap<String, DoweType>,
}

impl TypeRegistry {
    pub fn parse(path: &Path, nodes: &[SourceNode]) -> DoweResult<Self> {
        Self::parse_nodes(path, nodes, HashMap::new())
    }

    pub fn parse_file(root: &Path, file: &SourceFile) -> DoweResult<Self> {
        Self::parse_file_with_import_filter(root, file, &|_, _| false)
    }

    pub fn parse_file_with_import_filter(
        root: &Path,
        file: &SourceFile,
        skip_import: &dyn Fn(&SourceFile, &SourceImport) -> bool,
    ) -> DoweResult<Self> {
        parse_file_with_imports(root, file, &mut Vec::new(), skip_import)
    }

    fn parse_nodes(
        path: &Path,
        nodes: &[SourceNode],
        mut definitions: HashMap<String, DoweType>,
    ) -> DoweResult<Self> {
        let mut declarations = HashMap::<String, SourceNode>::new();
        for node in nodes.iter().filter(|node| node.name == "type") {
            let name = node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .ok_or_else(|| node_error(node, "type must declare a name"))?;
            validate_identifier(node, &name, "type")?;
            if node.args.len() != 1 || !node.props.is_empty() {
                return Err(node_error(node, "type only accepts one name"));
            }
            if node.children.is_empty() {
                return Err(node_error(
                    node,
                    format!("type `{name}` must declare fields"),
                ));
            }
            if definitions.contains_key(&name) {
                return Err(node_error(node, format!("duplicate type `{name}`")));
            }
            if declarations.insert(name.clone(), node.clone()).is_some() {
                return Err(node_error(node, format!("duplicate type `{name}`")));
            }
        }

        let names = declarations.keys().cloned().collect::<HashSet<_>>();
        for name in names.iter() {
            resolve_type(
                name,
                &declarations,
                &names,
                &mut definitions,
                &mut Vec::new(),
            )?;
        }

        for node in nodes {
            if node.name == "type"
                || matches!(node.name.as_str(), "main" | "page" | "layout" | "store")
            {
                continue;
            }
            if declarations.is_empty() {
                continue;
            }
            if node.location.indent == 0
                && !matches!(
                    node.name.as_str(),
                    "fn" | "handler"
                        | "middleware"
                        | "entity"
                        | "seeder"
                        | "database"
                        | "endpoints"
                )
                && !node.name.starts_with("type")
            {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "{}:{}: unsupported top-level block `{}`",
                        node.location.line, node.location.column, node.name
                    ),
                ));
            }
        }

        Ok(Self { definitions })
    }

    pub fn empty() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn resolve(&self, node: &SourceNode, name: &str) -> DoweResult<DoweType> {
        parse_type_reference(node, name, &self.definitions)
    }
}

fn parse_file_with_imports(
    root: &Path,
    file: &SourceFile,
    stack: &mut Vec<PathBuf>,
    skip_import: &dyn Fn(&SourceFile, &SourceImport) -> bool,
) -> DoweResult<TypeRegistry> {
    let shared_type_file = is_shared_type_file(file);
    if shared_type_file {
        validate_shared_type_nodes(file)?;
        if stack.iter().any(|path| path == &file.path) {
            return Err(DoweError::at_path(
                &file.path,
                "cyclic shared type import detected",
            ));
        }
        stack.push(file.path.clone());
    }

    let mut imported = HashMap::new();
    for import in &file.imports {
        if skip_import(file, import) {
            continue;
        }
        let path = resolve_import(root, &file.path, import)?;
        if !is_shared_type_path(root, &path) {
            if shared_type_file {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!(
                        "{}:{}: shared type modules can only import shared type modules",
                        import.location.line, import.location.column
                    ),
                ));
            }
            continue;
        }
        if stack.iter().any(|value| value == &path) {
            return Err(DoweError::at_path(
                &import.location.path,
                format!(
                    "{}:{}: cyclic shared type import detected",
                    import.location.line, import.location.column
                ),
            ));
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let imported_file = parse_source_file(root, &path, source)?;
        let registry = parse_file_with_imports(root, &imported_file, stack, skip_import)?;
        let value = registry
            .definitions
            .get(&import.local)
            .cloned()
            .ok_or_else(|| {
                DoweError::at_path(
                    &import.location.path,
                    format!(
                        "{}:{}: shared type module does not export `{}`",
                        import.location.line, import.location.column, import.local
                    ),
                )
            })?;
        if imported.insert(import.local.clone(), value).is_some() {
            return Err(DoweError::at_path(
                &import.location.path,
                format!(
                    "{}:{}: duplicate type `{}`",
                    import.location.line, import.location.column, import.local
                ),
            ));
        }
    }

    let registry = TypeRegistry::parse_nodes(&file.path, &file.nodes, imported);
    if shared_type_file {
        stack.pop();
    }
    registry
}

pub(crate) fn validate_shared_type_source(root: &Path, file: &SourceFile) -> DoweResult<()> {
    validate_shared_type_nodes(file)?;
    TypeRegistry::parse_file(root, file).map(|_| ())
}

pub(crate) fn is_shared_type_file(file: &SourceFile) -> bool {
    !file.nodes.is_empty() && file.nodes.iter().all(|node| node.name == "type")
}

pub(crate) fn is_shared_type_path(root: &Path, path: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| parse_source_file(root, path, source).ok())
        .is_some_and(|file| is_shared_type_file(&file))
}

fn validate_shared_type_nodes(file: &SourceFile) -> DoweResult<()> {
    if file.nodes.is_empty() {
        return Err(DoweError::at_path(
            &file.path,
            "shared type modules must declare at least one type",
        ));
    }
    for node in &file.nodes {
        if node.name != "type" {
            return Err(node_error(
                node,
                "shared type modules only accept `type` declarations",
            ));
        }
    }
    Ok(())
}
