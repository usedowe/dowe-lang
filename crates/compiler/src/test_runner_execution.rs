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

