use dowe_codegraph::{
    BuildOptions, CheckOptions, CodeGraphMode, DiagnosticSeverity, NodeKind, build_codegraph,
    check_codegraph, detect_codegraph_mode, explain_node, write_codegraph_baseline,
    write_codegraph_reports,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn builds_graph_for_rust_workspace() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/lib.rs"),
        "pub fn sample() {}\n",
    );

    let graph = build_codegraph(temp.path(), BuildOptions::default()).expect("graph");

    assert_eq!(graph.mode, CodeGraphMode::Project);
    assert!(graph.nodes.iter().any(|node| node.kind == NodeKind::Crate));
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("crates/sample/src/lib.rs"))
    );
    assert!(
        graph
            .edges
            .iter()
            .any(|edge| matches!(edge.kind, dowe_codegraph::EdgeKind::Contains))
    );
}

#[test]
fn detects_dowe_mode_from_repository_markers() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("agents")).expect("agents");
    fs::write(temp.path().join("AGENTS.md"), "# AGENTS\n").expect("agents");
    fs::write(temp.path().join("agents/README.md"), "# Agents\n").expect("readme");

    let mode = detect_codegraph_mode(temp.path()).expect("mode");

    assert_eq!(mode, CodeGraphMode::Dowe);
}

#[test]
fn builds_dowe_graph_from_private_workspace_layout() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("agents")).expect("agents");
    fs::write(temp.path().join("AGENTS.md"), "# AGENTS\n").expect("agents");
    fs::write(temp.path().join("agents/README.md"), "# Agents\n").expect("readme");
    fs::create_dir_all(temp.path().join("dowe-lang/crates/sample/src")).expect("crate");
    fs::write(
        temp.path().join("dowe-lang/crates/sample/Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("crate cargo");
    write_rust_file(
        temp.path().join("dowe-lang/crates/sample/src/lib.rs"),
        "pub fn sample() {}\n",
    );
    fs::create_dir_all(temp.path().join("docs")).expect("docs");
    fs::write(temp.path().join("docs/README.md"), "# Docs\n").expect("docs");
    fs::create_dir_all(temp.path().join("specs/features/00001-private")).expect("spec");
    fs::write(
        temp.path().join("specs/features/00001-private/spec.md"),
        "# Private Spec\n",
    )
    .expect("spec");

    let graph = build_codegraph(temp.path(), BuildOptions::default()).expect("graph");

    assert_eq!(graph.mode, CodeGraphMode::Dowe);
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("dowe-lang/crates/sample/src/lib.rs"))
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("docs/README.md"))
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("specs/features/00001-private/spec.md"))
    );
}

#[test]
fn builds_project_graph_without_agents() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src");
    fs::write(temp.path().join("src/views.dowe"), "views\n").expect("src");

    let graph = build_codegraph(temp.path(), BuildOptions::default()).expect("graph");

    assert_eq!(graph.mode, CodeGraphMode::Project);
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("src/views.dowe"))
    );
}

#[test]
fn builds_project_graph_with_agents() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join(".agents/harnesses")).expect("agents");
    fs::write(temp.path().join(".agents/AGENTS.md"), "# Agents\n").expect("agents");
    fs::write(temp.path().join(".agents/harnesses/tdd.md"), "# TDD\n").expect("tdd");

    let graph = build_codegraph(temp.path(), BuildOptions::default()).expect("graph");

    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.kind == NodeKind::AgentHarness)
    );
}

#[test]
fn reports_modular_warning_and_error_thresholds() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/large.rs"),
        &line_file(301, "pub fn a() {}\n"),
    );
    write_rust_file(
        temp.path().join("crates/sample/src/huge.rs"),
        &line_file(501, "pub fn b() {}\n"),
    );

    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "file_over_300_lines"
                && diagnostic.severity == DiagnosticSeverity::Warning)
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "file_over_500_lines"
                && diagnostic.severity == DiagnosticSeverity::Error)
    );
}

#[test]
fn rejects_anonymous_split_files() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/lib_parts/part_001.rs"),
        "pub fn sample() {}\n",
    );

    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "anonymous_partition_file"
            && diagnostic.severity == DiagnosticSeverity::Error
    }));
}

#[test]
fn accepts_valid_waiver_declared_in_spec() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/huge.rs"),
        &line_file(501, "pub fn b() {}\n"),
    );
    let spec = temp.path().join("specs/features/00001-waiver");
    fs::create_dir_all(&spec).expect("spec");
    fs::write(
        spec.join("compiler.md"),
        "# Contract\n\n## Waivers\n\n| path | limit | reason | spec | expires | owner |\n| --- | --- | --- | --- | --- | --- |\n| crates/sample/src/huge.rs | 500 | migration | specs/features/00001-waiver | split file | crates/sample |\n",
    )
    .expect("waiver");

    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "file_over_500_lines")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "modular_waiver_debt")
    );
}

#[test]
fn rejects_waiver_that_exists_only_under_dowe() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/huge.rs"),
        &line_file(501, "pub fn b() {}\n"),
    );
    fs::create_dir_all(temp.path().join(".dowe/codegraph")).expect("dowe");
    fs::write(
        temp.path().join(".dowe/codegraph/baseline.json"),
        r#"{"diagnostics":[{"path":"crates/sample/src/huge.rs"}]}"#,
    )
    .expect("baseline");

    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "file_over_500_lines")
    );
}

#[test]
fn compares_existing_baseline_metrics() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/huge.rs"),
        &line_file(501, "pub fn b() {}\n"),
    );
    let first = check_codegraph(temp.path(), CheckOptions::default()).expect("check");
    write_codegraph_baseline(temp.path(), &first).expect("baseline");
    write_rust_file(
        temp.path().join("crates/sample/src/huge.rs"),
        &line_file(510, "pub fn b() {}\n"),
    );

    let report = check_codegraph(
        temp.path(),
        CheckOptions {
            use_baseline: true,
            ..CheckOptions::default()
        },
    )
    .expect("check");

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "baseline_violation_grew")
    );
}

#[test]
fn detects_exact_and_normalized_duplicate_functions() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/lib.rs"),
        "pub fn first(value: i32) -> i32 {\nlet total = value + 1;\ntotal * 2\n}\n\npub fn second(input: i32) -> i32 {\nlet result = input + 1;\nresult * 2\n}\n",
    );

    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "duplicate_function")
    );
}

#[test]
fn writes_reports_under_dowe_codegraph() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/lib.rs"),
        "pub fn sample() {}\n",
    );
    let graph = build_codegraph(temp.path(), BuildOptions::default()).expect("graph");
    let report = check_codegraph(temp.path(), CheckOptions::default()).expect("check");

    let written = write_codegraph_reports(temp.path(), &graph, &report).expect("reports");

    assert_eq!(written.graph_path, ".dowe/codegraph/graph.json");
    assert!(temp.path().join(".dowe/codegraph/graph.json").exists());
    assert!(temp.path().join(".dowe/codegraph/report.md").exists());
    assert!(!temp.path().join("codegraph/report.json").exists());
}

#[test]
fn explains_node_by_path() {
    let temp = TempDir::new().expect("tempdir");
    write_workspace(&temp);
    write_rust_file(
        temp.path().join("crates/sample/src/lib.rs"),
        "pub fn sample() {}\n",
    );

    let explanation = explain_node(
        temp.path(),
        "crates/sample/src/lib.rs",
        BuildOptions::default(),
    )
    .expect("explanation");

    assert_eq!(
        explanation.node.path.as_deref(),
        Some("crates/sample/src/lib.rs")
    );
    assert!(!explanation.incoming.is_empty());
}

fn write_workspace(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("crates/sample/src")).expect("crate");
    fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/sample\"]\n",
    )
    .expect("workspace");
    fs::write(
        temp.path().join("crates/sample/Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("crate cargo");
}

fn write_rust_file(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent");
    }
    fs::write(path, content).expect("rust file");
}

fn line_file(lines: usize, line: &str) -> String {
    (0..lines).map(|_| line).collect()
}
