#[cfg(test)]
mod skill_normalization_tests {
    use super::*;

    #[test]
    fn embedded_skill_pages_keep_large_references_out_of_one_request() {
        let root = tempfile::tempdir().unwrap();
        let tools = HarnessTools::new(root.path(), "skill-page", HarnessConfig::default()).unwrap();
        let result = tools
            .execute_skill(&ToolCall::new("skill", "get_skill", json!({"id":"views"})))
            .unwrap();
        assert!(result["content"].as_str().unwrap().len() <= MAX_SKILL_PAGE_BYTES);
        assert_eq!(result["truncated"], true);
        assert!(result["next_offset"].as_u64().unwrap() > 1);
    }

    #[test]
    fn normalizes_legacy_embedded_skill_forms() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-views", Some("views")).unwrap(),
            ("views".to_string(), Some("references/views.md".to_string()))
        );
        assert_eq!(
            normalize_embedded_skill_request("dowe-core", Some("core")).unwrap(),
            ("core".to_string(), Some("references/main.md".to_string()))
        );
        assert_eq!(
            normalize_embedded_skill_request("references/composition.md", None).unwrap(),
            (
                "views".to_string(),
                Some("references/composition.md".to_string())
            )
        );
    }

    #[test]
    fn normalizes_swapped_resource_and_bundle_forms() {
        for resource in ["composition.md", "components.md"] {
            let id = format!("references/{resource}");
            assert_eq!(
                normalize_embedded_skill_request(&id, Some("views")).unwrap(),
                ("views".to_string(), Some(id))
            );
        }
    }

    #[test]
    fn infers_views_owner_from_declared_reference_ids() {
        for (id, resource) in [
            ("composition", "references/composition.md"),
            ("components", "references/components.md"),
            ("reference-ui", "references/reference-ui.md"),
        ] {
            assert_eq!(
                normalize_embedded_skill_request(id, Some(resource)).unwrap(),
                ("views".to_string(), Some(resource.to_string()))
            );
        }
    }

    #[test]
    fn rejects_declared_reference_id_mismatch() {
        let error =
            normalize_embedded_skill_request("composition", Some("references/components.md"))
                .unwrap_err()
                .to_string();
        assert!(error.contains("does not match skill id `composition`"));
    }

    #[test]
    fn rejects_swapped_resource_bundle_mismatch() {
        let error = normalize_embedded_skill_request("references/composition.md", Some("server"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("does not match bundle `server`"));
    }

    #[test]
    fn normalizes_matching_legacy_bundle_alias_to_compact_unit() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-core", Some("bundles/core")).unwrap(),
            ("core".to_string(), None)
        );
    }

    #[test]
    fn normalizes_nested_legacy_bundle_alias_to_compact_unit() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-views-pages", Some("bundles/views/pages"))
                .unwrap(),
            ("views-pages".to_string(), None)
        );
    }

    #[test]
    fn normalizes_hierarchical_legacy_bundle_aliases() {
        for (id, resource) in [
            ("core/validation", "bundles/core/validation"),
            ("core/validation", "bundles/core"),
            ("views/pages", "bundles/views"),
        ] {
            assert_eq!(
                normalize_embedded_skill_request(id, Some(resource)).unwrap(),
                (id.to_string(), None)
            );
        }
    }

    #[test]
    fn rejects_mismatched_legacy_bundle_alias() {
        let error = normalize_embedded_skill_request("core", Some("bundles/views"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("legacy skill bundle alias `bundles/views`"));
        assert!(error.contains("logical skill id `core`"));

        let error = normalize_embedded_skill_request(
            "core/validation",
            Some("bundles/core-validation-extra"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("legacy skill bundle alias"));

        let error = normalize_embedded_skill_request("core/validation", Some("bundles/corex"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("legacy skill bundle alias"));
    }

    #[test]
    fn rejects_ambiguous_or_unknown_resource_shorthand() {
        let error = normalize_embedded_skill_request("references/workflow.md", None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("ambiguous"));
        let error =
            normalize_embedded_skill_request("dowe-domain-modeling", Some("domain-modeling"))
                .unwrap_err()
                .to_string();
        assert!(error.contains("not unambiguous"));
    }

    #[test]
    fn unknown_skill_diagnostic_explains_the_recovery_path() {
        let error = normalize_embedded_skill_request("layoutz", Some("layoutz"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown public Dowe skill `layoutz`"));
        assert!(error.contains("exact logical id"));
        assert!(error.contains("views"));
    }
}
