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

fn run_case(case: ParsedTestCase) -> TestCaseResult {
    let mut assertions = 0usize;
    for assertion in case.assertions {
        assertions += 1;
        let failure = match assertion.kind {
            AssertionKind::True(value) if value != SourceValue::Boolean(true) => Some(format!(
                "{}: assert true received {}",
                assertion.line,
                value.to_source()
            )),
            AssertionKind::False(value) if value != SourceValue::Boolean(false) => Some(format!(
                "{}: assert false received {}",
                assertion.line,
                value.to_source()
            )),
            AssertionKind::Equal { actual, expected } if actual != expected => Some(format!(
                "{}: assert equal expected {} but received {}",
                assertion.line,
                expected.to_source(),
                actual.to_source()
            )),
            _ => None,
        };
        if let Some(message) = failure {
            return TestCaseResult {
                name: case.name,
                path: case.path,
                line: case.line,
                assertions,
                status: TestStatus::Failed,
                message: Some(message),
            };
        }
    }
    TestCaseResult {
        name: case.name,
        path: case.path,
        line: case.line,
        assertions,
        status: TestStatus::Passed,
        message: None,
    }
}

fn normalized_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn normalized_relative_path_from_location(node: &SourceNode) -> String {
    node.location
        .relative_path
        .to_string_lossy()
        .replace('\\', "/")
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{TestStatus, run_project_tests};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    const PASSING_TEST: &str = r#"test "literal assertions"
  assert true value:true
  assert false value:false
  assert equal actual:{ name:"dowe" versions:[1 2] } expected:{ name:"dowe" versions:[1 2] }
"#;

    #[test]
    fn discovers_tests_from_named_and_arbitrary_directories() {
        let root = tempdir().expect("root");
        fs::create_dir_all(root.path().join("test")).expect("test dir");
        fs::create_dir_all(root.path().join("checks/release")).expect("checks dir");
        fs::write(root.path().join("main.dowe"), "main\n").expect("main");
        fs::write(root.path().join("test/basic.dowe"), PASSING_TEST).expect("test source");
        fs::write(
            root.path().join("checks/release/metadata.dowe"),
            PASSING_TEST,
        )
        .expect("checks source");

        let report = run_project_tests(root.path(), &[]).expect("report");

        assert_eq!(report.discovered, 2);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 0);
        assert_eq!(report.cases[0].path, "checks/release/metadata.dowe");
        assert_eq!(report.cases[1].path, "test/basic.dowe");
    }

    #[test]
    fn selectors_limit_test_discovery_to_a_directory_or_file() {
        let root = tempdir().expect("root");
        fs::create_dir_all(root.path().join("verification")).expect("verification");
        fs::write(root.path().join("test.dowe"), PASSING_TEST).expect("root test");
        fs::write(root.path().join("verification/check.dowe"), PASSING_TEST).expect("nested test");

        let directory = run_project_tests(root.path(), &[PathBuf::from("verification")])
            .expect("directory report");
        let file =
            run_project_tests(root.path(), &[PathBuf::from("test.dowe")]).expect("file report");

        assert_eq!(directory.discovered, 1);
        assert_eq!(directory.cases[0].path, "verification/check.dowe");
        assert_eq!(file.discovered, 1);
        assert_eq!(file.cases[0].path, "test.dowe");
    }

    #[test]
    fn reports_failures_without_aborting_other_tests() {
        let root = tempdir().expect("root");
        fs::write(
            root.path().join("checks.dowe"),
            "test \"failure\"\n  assert equal actual:\"actual\" expected:\"expected\"\n\ntest \"pass\"\n  assert true value:true\n",
        )
        .expect("test source");

        let report = run_project_tests(root.path(), &[]).expect("report");

        assert_eq!(report.discovered, 2);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.cases[0].status, TestStatus::Failed);
        assert!(
            report.cases[0]
                .message
                .as_deref()
                .is_some_and(|message| message.contains("expected \"expected\""))
        );
        assert_eq!(report.cases[1].status, TestStatus::Passed);
    }

    #[test]
    fn rejects_invalid_test_source_and_unsafe_selectors() {
        let root = tempdir().expect("root");
        fs::write(
            root.path().join("bad.dowe"),
            "test \"bad\"\n  assert true\n",
        )
        .expect("bad source");

        let invalid = run_project_tests(root.path(), &[]).expect_err("invalid test");
        let selector = run_project_tests(root.path(), &[PathBuf::from("../outside")])
            .expect_err("unsafe selector");

        assert!(invalid.message().contains("requires `value`"));
        assert!(selector.message().contains("relative paths below"));
    }

    #[test]
    fn skips_managed_and_generated_directories() {
        let root = tempdir().expect("root");
        fs::create_dir_all(root.path().join(".agents/skills")).expect("agents");
        fs::create_dir_all(root.path().join(".dowe/generated")).expect("dowe");
        fs::write(
            root.path().join(".agents/skills/ignored.dowe"),
            PASSING_TEST,
        )
        .expect("agent test");
        fs::write(
            root.path().join(".dowe/generated/ignored.dowe"),
            PASSING_TEST,
        )
        .expect("generated test");
        fs::write(root.path().join("included.dowe"), PASSING_TEST).expect("included test");

        let report = run_project_tests(root.path(), &[]).expect("report");

        assert_eq!(report.discovered, 1);
        assert_eq!(report.cases[0].path, "included.dowe");
    }
}
