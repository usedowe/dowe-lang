use super::*;
use std::collections::BTreeMap;

#[test]
fn dependency_graph_rejects_transitive_cycles_and_missing_dependencies() {
    let mut graph = BTreeMap::from([
        ("core".into(), vec![]),
        ("page".into(), vec!["layout".into()]),
        ("layout".into(), vec!["core".into()]),
    ]);
    assert!(validate_dependencies(&graph).is_ok());
    graph.insert("core".into(), vec!["page".into()]);
    assert!(
        validate_dependencies(&graph)
            .unwrap_err()
            .to_string()
            .contains("cyclic")
    );
    graph.insert("core".into(), vec!["missing".into()]);
    assert!(validate_dependencies(&graph).is_err());
}

#[test]
fn invalid_project_skill_is_diagnostic_instead_of_request_failure() {
    let root = tempfile::tempdir().expect("root");
    std::fs::create_dir_all(root.path().join(".agents/skills")).expect("skills");
    std::fs::write(root.path().join(".agents/skills/broken"), "not a directory")
        .expect("broken skill");
    let context = project_skill_context(root.path()).expect("diagnostic context");
    assert!(context.contains("invalid or unavailable skills were excluded"));
}
