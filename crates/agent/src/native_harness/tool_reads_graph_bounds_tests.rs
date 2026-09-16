#[cfg(test)]
mod graph_bounds_tests {
    use super::*;

    #[test]
    fn graph_tools_preserve_incomplete_evidence() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(".agent")).unwrap();
        let nodes = ["a", "b", "c"].map(|id| {
            json!({
                "id": format!("knowledge:{id}"), "name": id,
                "namespace": "backend", "kind": "entity", "evidence": "assumed"
            })
        });
        let edges = [("b", "a"), ("c", "b"), ("c", "a")].map(|(from, to)| {
            json!({
                "from": format!("knowledge:{from}"), "to": format!("knowledge:{to}"),
                "relation": "uses", "evidence": "assumed"
            })
        });
        fs::write(
            root.path().join(".agent/knowledge.json"),
            serde_json::to_vec(&json!({
                "schema": 1, "nodes": nodes, "edges": edges
            }))
            .unwrap(),
        )
        .unwrap();
        let tools =
            HarnessTools::new(root.path(), "graph-bounds", HarnessConfig::default()).unwrap();
        for (name, id) in [
            ("get_dependencies", "c"),
            ("get_consumers", "a"),
            ("get_impact", "a"),
        ] {
            for (limit, truncated) in [(1, true), (10, false)] {
                let result = tools
                    .execute_read(&ToolCall::new(
                        "bounds",
                        name,
                        json!({
                            "id": format!("knowledge:{id}"), "limit": limit, "depth": 8
                        }),
                    ))
                    .unwrap();
                assert_eq!(result["truncated"], truncated, "{name}");
                assert!(result["nodes"].as_array().unwrap().len() <= limit);
            }
        }
        let result = tools
            .execute_read(&ToolCall::new(
                "depth",
                "get_impact",
                json!({
                    "id": "knowledge:a", "limit": 10, "depth": 0
                }),
            ))
            .unwrap();
        assert_eq!(result["truncated"], true);
        assert!(!root.path().join(".dowe").exists());
    }
}
