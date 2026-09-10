    #[test]
    fn check_harness_blocks_validated_plan_without_docs_updated() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        let mut state = read_plan_state(temp.path(), "00001-example-feature").expect("state");
        state.expected_initial_failures.push("test failed first".to_string());
        state.state = TddState::Validated;
        write_plan_state(temp.path(), "00001-example-feature", &state).expect("state");

        let report = check_harness(temp.path()).expect("check");

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "documentation_not_updated")
        );
    }

    #[test]
    fn check_harness_blocks_missing_post_implementation_actions() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        let mut state = read_plan_state(temp.path(), "00001-example-feature").expect("state");
        state.documentation_actions.clear();
        state.skill_actions.clear();
        write_plan_state(temp.path(), "00001-example-feature", &state).expect("state");

        let report = check_harness(temp.path()).expect("check");

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "documentation_actions_missing")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "skill_actions_missing")
        );
    }

    #[test]
    fn check_harness_blocks_docs_updated_when_target_is_missing() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        let mut state = read_plan_state(temp.path(), "00001-example-feature").expect("state");
        state.expected_initial_failures.push("test failed first".to_string());
        state.state = TddState::DocsUpdated;
        write_plan_state(temp.path(), "00001-example-feature", &state).expect("state");

        let report = check_harness(temp.path()).expect("check");

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "documentation_target_missing")
        );
    }

    #[test]
    fn check_harness_accepts_docs_updated_with_existing_target() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        fs::create_dir_all(temp.path().join("docs/development")).expect("docs");
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        let mut state = read_plan_state(temp.path(), "00001-example-feature").expect("state");
        state.expected_initial_failures.push("test failed first".to_string());
        state.state = TddState::DocsUpdated;
        write_plan_state(temp.path(), "00001-example-feature", &state).expect("state");

        let report = check_harness(temp.path()).expect("check");

        assert!(!report.has_errors());
    }

    #[test]
    fn tdd_state_rejects_implementation_before_tests_are_written() {
        assert!(
            transition_tdd_state(TddState::TestsPlanned, TddState::ImplementationAllowed).is_err()
        );
        assert!(
            transition_tdd_state(
                TddState::ExpectedFailureRecorded,
                TddState::ImplementationAllowed
            )
            .is_ok()
        );
    }

    #[test]
    fn safe_agent_paths_reject_traversal() {
        let temp = TempDir::new().expect("tempdir");
        init_project_harness(temp.path(), InitOptions::default()).expect("init");

        let error = write_agent_file(
            temp.path(),
            Path::new("../outside.md"),
            "bad",
            WriteMode::Preserve,
        )
        .expect_err("error");

        assert!(error.to_string().contains("path must stay under .agents"));
        assert!(!temp.path().join("outside.md").exists());
    }

    #[test]
    fn check_harness_detects_invalid_tdd_state() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        let mut state = read_plan_state(temp.path(), "00001-example-feature").expect("state");
        state.state = TddState::ImplementationAllowed;
        write_plan_state(temp.path(), "00001-example-feature", &state).expect("state");

        let report = check_harness(temp.path()).expect("check");

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "invalid_tdd_state")
        );
    }

    #[test]
    fn legacy_plan_state_without_codegraph_binding_deserializes() {
        let value = serde_json::json!({"planId":"legacy","specPath":"spec.md","specFingerprint":"x","contracts":[],"acceptanceCriteria":[],"testPlan":[],"expectedInitialFailures":[],"expectedFailureJustification":null,"implementationScope":[],"validationCommands":[],"documentationTargets":[],"state":"tests_planned","incompleteReasons":[],"tddRequired":true});
        let state: PlanState = serde_json::from_value(value).expect("legacy state");
        assert!(state.codegraph_binding.is_none());
    }

    #[test]
    fn bootstrap_sdd_change_is_bounded_preserves_existing_files_and_respects_canonical_surfaces() {
        let root = tempfile::tempdir().expect("root");
        let report = bootstrap_sdd_change(root.path(), "add-search", "Add search").expect("bootstrap");
        assert_eq!(report.created.len(), 3);
        assert!(root.path().join(".agents/changes/add-search/proposal.md").is_file());
        let proposal = root.path().join(".agents/changes/add-search/proposal.md");
        fs::write(&proposal, "# user-owned proposal\n").expect("customize proposal");
        let rerun = bootstrap_sdd_change(root.path(), "add-search", "Changed title").expect("rerun");
        assert!(rerun.created.is_empty());
        assert_eq!(fs::read_to_string(&proposal).unwrap(), "# user-owned proposal\n");

        fs::create_dir(root.path().join("openspec")).expect("canonical marker");
        assert!(bootstrap_sdd_change(root.path(), "another", "Another").is_err());
    }

