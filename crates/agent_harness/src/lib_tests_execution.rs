    #[test]
    fn task_packet_round_trips_and_rejects_unbounded_evidence() {
        let packet = TaskPacket {
            task_id: "task-1".into(),
            role: "review".into(),
            objective: "inspect the focused change".into(),
            acceptance: vec!["tests pass".into()],
            constraints: vec!["read_only_stage".into()],
            files: vec!["src/lib.rs".into()],
            evidence: vec![EvidenceRef {
                path: "src/lib.rs".into(),
                start_line: Some(1),
                end_line: Some(4),
                fingerprint: "abc".into(),
                source: "operation_receipt".into(),
            }],
            change_set: Some(ChangeSet {
                baseline: Some("baseline-1".into()),
                files: vec![ChangeSetEntry {
                    path: "src/lib.rs".into(),
                    kind: ChangeKind::Modified,
                    before_fingerprint: Some("old".into()),
                    after_fingerprint: Some("new".into()),
                }],
            }),
            validation: vec!["cargo test".into()],
            budget_remaining_tokens: 100,
        };
        packet.validate().expect("bounded packet");
        let mut multiline = packet.clone();
        multiline.objective = "inspect line one\ninspect line two".into();
        multiline.validate().expect("multiline objective");
        let decoded: TaskPacket = serde_json::from_value(serde_json::to_value(&packet).unwrap()).unwrap();
        assert_eq!(decoded, packet);
        let mut oversized = packet;
        oversized.files = (0..65).map(|index| format!("src/{index}.rs")).collect();
        assert!(oversized.validate().is_err());
    }

    #[test]
    fn task_packet_rejects_control_text_empty_evidence_and_invalid_ranges() {
        let mut packet = TaskPacket {
            task_id: "task-1".into(),
            role: "research".into(),
            objective: "inspect".into(),
            acceptance: vec!["facts".into()],
            constraints: vec!["read only".into()],
            files: vec!["src/lib.rs".into()],
            evidence: vec![EvidenceRef {
                path: "src/lib.rs".into(),
                start_line: Some(4),
                end_line: Some(2),
                fingerprint: "sha256:abc".into(),
                source: "local".into(),
            }],
            change_set: None,
            validation: vec!["not run".into()],
            budget_remaining_tokens: 1,
        };
        assert!(packet.validate().is_err());
        packet.evidence[0].end_line = Some(4);
        packet.evidence[0].fingerprint.clear();
        assert!(packet.validate().is_err());
        packet.evidence[0].fingerprint = "sha256:abc".into();
        packet.objective = "bad\u{0000}text".into();
        assert!(packet.validate().is_err());
    }

    #[test]
    fn validate_plan_persists_validation_without_review_or_delivery_authority() {
            let temp = TempDir::new().expect("tempdir");
            write_spec_fixture(temp.path(), true);
            init_project_harness(temp.path(), InitOptions::default()).expect("init");
            plan_from_spec(
                temp.path(),
                Path::new("specs/features/00001-example-feature"),
                PlanOptions::default(),
            )
            .expect("plan");

            validate_plan(temp.path(), "00001-example-feature").expect("validate");
            let state = read_plan_state(temp.path(), "00001-example-feature").expect("reloaded state");
            let task = state.governance_task.expect("governance task");
            assert_eq!(task.validation_state, crate::ValidationState::Recorded);
            assert!(task.validation_evidence.is_some());
            assert_eq!(task.review_state, crate::ReviewState::Pending);
            assert_eq!(task.acknowledgement_state, crate::AcknowledgementState::Pending);
            assert_eq!(task.delivery_state, crate::DeliveryState::NotRequested);
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
