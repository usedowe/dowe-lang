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
