    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn init_project_harness_creates_project_agent_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("src")).expect("src");

        let report =
            init_project_harness(temp.path(), InitOptions::default()).expect("init report");

        assert_eq!(report.created.len(), 4);
        assert!(temp.path().join(".agents/AGENTS.md").exists());
        assert!(temp.path().join(".agents/manifest.json").exists());
        assert!(temp.path().join(".agents/harnesses/tdd.md").exists());
        assert!(temp.path().join(".agents/plans").exists());
        assert!(!temp.path().join("agents").exists());
        assert!(!temp.path().join(".dowe").exists());

        let manifest = read_manifest(temp.path()).expect("manifest");

        assert_eq!(manifest.schema_version, "1");
        assert_eq!(manifest.mode, HarnessMode::Project);
        assert_eq!(manifest.agent_root, ".agents");
        assert_eq!(manifest.generated_evidence_root, ".dowe/agent-harnesses");
        assert!(manifest.tdd_required);
        assert!(
            manifest
                .validation_commands
                .iter()
                .any(|command| command.id == "harness-check")
        );
        assert!(
            manifest
                .validation_commands
                .iter()
                .any(|command| command.id == "codegraph-check")
        );
    }

    #[test]
    fn check_harness_dowe_mode_does_not_require_project_agents() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("agents")).expect("agents");
        fs::write(temp.path().join("AGENTS.md"), "# AGENTS.md\n").expect("root agents");
        fs::write(
            temp.path().join("agents/README.md"),
            "# Dowe Agent System\n",
        )
        .expect("agents");

        let report = check_harness(temp.path()).expect("check");

        assert!(!report.has_errors());
        assert!(!temp.path().join(".agents").exists());
    }

    #[test]
    fn init_project_harness_preserves_existing_user_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join(".agents/harnesses")).expect("agents");
        fs::write(temp.path().join(".agents/AGENTS.md"), "user instructions").expect("agents");

        let report =
            init_project_harness(temp.path(), InitOptions::default()).expect("init report");
        let content = fs::read_to_string(temp.path().join(".agents/AGENTS.md")).expect("agents");

        assert_eq!(content, "user instructions");
        assert!(
            report
                .preserved
                .iter()
                .any(|file| file.path == ".agents/AGENTS.md")
        );
    }

    #[test]
    fn plan_from_spec_creates_tdd_plan_and_state() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");

        let report = plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");

        assert_eq!(report.plan_id, "00001-example-feature");
        assert!(
            temp.path()
                .join(".agents/plans/00001-example-feature/plan.md")
                .exists()
        );
        assert!(
            temp.path()
                .join(".agents/plans/00001-example-feature/state.json")
                .exists()
        );

        let state = read_plan_state(temp.path(), "00001-example-feature").expect("state");

        assert_eq!(
            state.spec_path,
            "specs/features/00001-example-feature/spec.md"
        );
        assert_eq!(state.contracts.len(), 1);
        assert_eq!(state.acceptance_criteria.len(), 2);
        assert!(!state.test_plan.is_empty());
        assert!(
            state
                .documentation_targets
                .iter()
                .any(|target| target == "docs/development")
        );
        assert!(!state.documentation_actions.is_empty());
        assert!(
            state
                .skill_actions
                .iter()
                .any(|action| action.contains("dowe-document-feature"))
        );
        assert_eq!(state.state, TddState::TestsPlanned);
        assert!(!state.implementation_allowed());
    }

    #[test]
    fn complex_spec_plan_includes_codegraph_readiness() {
        let temp = TempDir::new().expect("tempdir");
        write_complex_spec_fixture(temp.path());
        init_project_harness(temp.path(), InitOptions::default()).expect("init");

        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00002-complex-feature"),
            PlanOptions::default(),
        )
        .expect("plan");

        let state = read_plan_state(temp.path(), "00002-complex-feature").expect("state");

        assert!(
            state
                .validation_commands
                .iter()
                .any(|command| command == "codegraph-check")
        );
        assert!(
            state
                .implementation_scope
                .iter()
                .any(|scope| scope.contains("CodeGraph"))
        );
        assert!(state.incomplete_reasons.is_empty());
    }

    #[test]
    fn plan_from_spec_derives_documentation_and_skill_actions() {
        let temp = TempDir::new().expect("tempdir");
        write_server_views_spec_fixture(temp.path());
        init_project_harness(temp.path(), InitOptions::default()).expect("init");

        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00003-server-views-feature"),
            PlanOptions::default(),
        )
        .expect("plan");

        let state = read_plan_state(temp.path(), "00003-server-views-feature").expect("state");
        let plan = fs::read_to_string(
            temp.path()
                .join(".agents/plans/00003-server-views-feature/plan.md"),
        )
        .expect("plan markdown");

        assert!(
            state
                .documentation_targets
                .iter()
                .any(|target| target == "docs/server")
        );
        assert!(
            state
                .documentation_targets
                .iter()
                .any(|target| target == "docs/views")
        );
        assert!(
            state
                .skill_actions
                .iter()
                .any(|action| action.contains("dowe-server-runtime"))
        );
        assert!(
            state
                .skill_actions
                .iter()
                .any(|action| action.contains("dowe-views-generation"))
        );
        assert!(plan.contains("## Documentation Actions"));
        assert!(plan.contains("## Skill Actions"));
    }

    #[test]
    fn plan_from_missing_spec_fails() {
        let temp = TempDir::new().expect("tempdir");
        init_project_harness(temp.path(), InitOptions::default()).expect("init");

        let error = plan_from_spec(
            temp.path(),
            Path::new("specs/features/00009-missing"),
            PlanOptions::default(),
        )
        .expect_err("error");

        assert!(error.to_string().contains("spec does not exist"));
    }

    #[test]
    fn check_harness_detects_outdated_plan_after_spec_change() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");
        fs::write(
            temp.path()
                .join("specs/features/00001-example-feature/spec.md"),
            "# Example\n\nchanged\n",
        )
        .expect("spec");

        let report = check_harness(temp.path()).expect("check");

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "plan_outdated")
        );
    }

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
    fn validate_plan_writes_redacted_evidence_under_dowe() {
        let temp = TempDir::new().expect("tempdir");
        write_spec_fixture(temp.path(), true);
        init_project_harness(temp.path(), InitOptions::default()).expect("init");
        plan_from_spec(
            temp.path(),
            Path::new("specs/features/00001-example-feature"),
            PlanOptions::default(),
        )
        .expect("plan");

        let report = validate_plan(temp.path(), "00001-example-feature").expect("validate");
        let evidence = fs::read_to_string(
            temp.path()
                .join(".dowe/agent-harnesses/00001-example-feature/validation.json"),
        )
        .expect("evidence");

        assert!(report.success);
        assert!(evidence.contains("harness-check"));
        assert!(evidence.contains("codegraph-check"));
        assert!(!evidence.contains("SECRET"));
        assert!(!temp.path().join(".agents/validation.json").exists());
    }
