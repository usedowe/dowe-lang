use crate::error::{DoweError, DoweResult};
use crate::parser::{SourceFile, SourceNode, SourceValue, parse_source_file};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseResult {
    pub name: String,
    pub path: String,
    pub line: usize,
    pub assertions: usize,
    pub status: TestStatus,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestReport {
    pub discovered: usize,
    pub passed: usize,
    pub failed: usize,
    pub cases: Vec<TestCaseResult>,
}

impl TestReport {
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[derive(Clone)]
struct ParsedTestCase {
    name: String,
    path: String,
    line: usize,
    assertions: Vec<ParsedAssertion>,
}

#[derive(Clone)]
struct ParsedAssertion {
    line: usize,
    kind: AssertionKind,
}

#[derive(Clone)]
enum AssertionKind {
    True(SourceValue),
    False(SourceValue),
    Equal {
        actual: SourceValue,
        expected: SourceValue,
    },
}

pub fn run_project_tests(root: &Path, selectors: &[PathBuf]) -> DoweResult<TestReport> {
    let root = fs::canonicalize(root)
        .map_err(|error| DoweError::at_path(root, format!("cannot read test root: {error}")))?;
    if !root.is_dir() {
        return Err(DoweError::at_path(&root, "test root must be a directory"));
    }

    let files = discover_test_files(&root, selectors)?;
    let mut cases = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)?;
        let file = parse_source_file(&root, &path, source)?;
        let Some(mut parsed) = parse_test_file(&file)? else {
            continue;
        };
        cases.append(&mut parsed);
    }

    let cases = cases.into_iter().map(run_case).collect::<Vec<_>>();
    let discovered = cases.len();
    let passed = cases
        .iter()
        .filter(|case| case.status == TestStatus::Passed)
        .count();
    Ok(TestReport {
        discovered,
        passed,
        failed: discovered - passed,
        cases,
    })
}

pub(crate) fn validate_test_file(file: &SourceFile) -> DoweResult<bool> {
    parse_test_file(file).map(|tests| tests.is_some())
}

fn discover_test_files(root: &Path, selectors: &[PathBuf]) -> DoweResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if selectors.is_empty() {
        collect_test_files(root, root, &mut files)?;
    } else {
        for selector in selectors {
            let path = resolve_selector(root, selector)?;
            let file_type = fs::symlink_metadata(&path)?.file_type();
            if file_type.is_dir() {
                if ignored_directory(&path) {
                    continue;
                }
                collect_test_files(root, &path, &mut files)?;
            } else if file_type.is_file() {
                if path.extension().is_none_or(|extension| extension != "dowe") {
                    return Err(DoweError::at_path(
                        selector,
                        "test file selectors must end in `.dowe`",
                    ));
                }
                files.push(path);
            } else {
                return Err(DoweError::at_path(
                    selector,
                    "test selector must be a file or directory",
                ));
            }
        }
    }
    files.sort_by_key(|path| normalized_relative_path(root, path));
    files.dedup();
    Ok(files)
}

fn resolve_selector(root: &Path, selector: &Path) -> DoweResult<PathBuf> {
    if selector.is_absolute()
        || selector.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(DoweError::at_path(
            selector,
            "test selectors must be relative paths below the project root",
        ));
    }
    let path = root.join(selector);
    if fs::symlink_metadata(&path)
        .map_err(|error| {
            DoweError::at_path(selector, format!("cannot resolve test selector: {error}"))
        })?
        .file_type()
        .is_symlink()
    {
        return Err(DoweError::at_path(
            selector,
            "test selectors cannot be symbolic links",
        ));
    }
    let resolved = fs::canonicalize(&path).map_err(|error| {
        DoweError::at_path(selector, format!("cannot resolve test selector: {error}"))
    })?;
    if !resolved.starts_with(root) {
        return Err(DoweError::at_path(
            selector,
            "test selectors must stay below the project root",
        ));
    }
    Ok(resolved)
}

fn collect_test_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> DoweResult<()> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            if !ignored_directory(&path) {
                collect_test_files(root, &path, files)?;
            }
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "dowe")
            && path.starts_with(root)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn ignored_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".agents" | ".dowe" | ".git" | "target" | "node_modules")
    )
}

fn parse_test_file(file: &SourceFile) -> DoweResult<Option<Vec<ParsedTestCase>>> {
    if !file.nodes.iter().any(|node| node.name == "test") {
        return Ok(None);
    }
    if !file.imports.is_empty() {
        let import = &file.imports[0];
        return Err(DoweError::at_path(
            &file.path,
            format!(
                "{}:{}: test files do not support imports",
                import.location.line, import.location.column
            ),
        ));
    }
    let mut tests = Vec::new();
    for node in &file.nodes {
        if node.name != "test" {
            return Err(node_error(
                node,
                "test files can only contain top-level `test` declarations",
            ));
        }
        tests.push(parse_test_case(node)?);
    }
    Ok(Some(tests))
}

fn parse_test_case(node: &SourceNode) -> DoweResult<ParsedTestCase> {
    let Some(SourceValue::String(name)) = node.args.first() else {
        return Err(node_error(node, "`test` requires one quoted string name"));
    };
    if node.args.len() != 1 || name.trim().is_empty() {
        return Err(node_error(
            node,
            "`test` requires one non-empty quoted string name",
        ));
    }
    if !node.props.is_empty() {
        return Err(node_error(node, "`test` does not support properties"));
    }
    if node.children.is_empty() {
        return Err(node_error(node, "`test` requires at least one `assert`"));
    }
    let assertions = node
        .children
        .iter()
        .map(parse_assertion)
        .collect::<DoweResult<Vec<_>>>()?;
    Ok(ParsedTestCase {
        name: name.clone(),
        path: normalized_relative_path_from_location(node),
        line: node.location.line,
        assertions,
    })
}

fn parse_assertion(node: &SourceNode) -> DoweResult<ParsedAssertion> {
    if node.name != "assert" {
        return Err(node_error(
            node,
            "`test` children must be `assert` declarations",
        ));
    }
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            "`assert` declarations cannot contain nested nodes",
        ));
    }
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`assert` requires exactly one assertion kind",
        ));
    }
    let kind = match node.args.first() {
        Some(SourceValue::Boolean(true)) => {
            let value = required_assertion_prop(node, "value", &["value"])?;
            AssertionKind::True(value.clone())
        }
        Some(SourceValue::Boolean(false)) => {
            let value = required_assertion_prop(node, "value", &["value"])?;
            AssertionKind::False(value.clone())
        }
        Some(SourceValue::Bareword(kind)) if kind == "equal" => {
            let actual = required_assertion_prop(node, "actual", &["actual", "expected"])?;
            let expected = required_assertion_prop(node, "expected", &["actual", "expected"])?;
            AssertionKind::Equal {
                actual: actual.clone(),
                expected: expected.clone(),
            }
        }
        _ => {
            return Err(node_error(
                node,
                "`assert` kind must be `true`, `false`, or `equal`",
            ));
        }
    };
    Ok(ParsedAssertion {
        line: node.location.line,
        kind,
    })
}

fn required_assertion_prop<'a>(
    node: &'a SourceNode,
    required: &str,
    allowed: &[&str],
) -> DoweResult<&'a SourceValue> {
    if let Some(prop) = node
        .props
        .iter()
        .find(|prop| !allowed.contains(&prop.name.as_str()))
    {
        return Err(node_error(
            node,
            format!("`assert` does not support `{}`", prop.name),
        ));
    }
    node.prop(required)
        .map(|prop| &prop.value)
        .ok_or_else(|| node_error(node, format!("`assert` requires `{required}`")))
}

