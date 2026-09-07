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
