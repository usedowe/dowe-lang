use dowe_agent_harness::clean::*;

fn budget() -> Budget {
    Budget {
        input_tokens: 100,
        output_tokens: 100,
        cost_micros: 10,
        duration_ms: 1000,
    }
}

#[test]
fn build_waits_for_blocking_requirements_before_approval() {
    let mut workflow = Workflow::new("task-1", Intent::Build, budget());
    workflow.requirements.push(Requirement {
        id: "limit".into(),
        question: "What limit?".into(),
        blocking: true,
        answer: None,
    });
    assert_eq!(workflow.resolve_requirements(), WorkflowResult::Waiting);
    assert_eq!(workflow.phase, Phase::Requirements);
    workflow.requirements[0].answer = Some(RequirementAnswer {
        value: "5".into(),
        source: "user".into(),
    });
    assert_eq!(workflow.resolve_requirements(), WorkflowResult::Ready);
    assert_eq!(workflow.phase, Phase::Approval);
}

#[test]
fn approval_is_bound_to_exact_plan_and_host() {
    let mut workflow = Workflow::new("task-1", Intent::Build, budget());
    assert!(matches!(
        workflow.approve(
            Approval {
                plan_digest: "old".into(),
                scopes: vec![],
                approved_by_host: true
            },
            "new"
        ),
        WorkflowResult::Blocked(_)
    ));
    let mut workflow = Workflow::new("task-2", Intent::Build, budget());
    assert_eq!(
        workflow.approve(
            Approval {
                plan_digest: "plan".into(),
                scopes: vec!["src".into()],
                approved_by_host: true
            },
            "plan"
        ),
        WorkflowResult::Ready
    );
    assert_eq!(workflow.phase, Phase::Execution);
}

#[test]
fn budget_blocks_before_reserving_over_limit_work() {
    let mut workflow = Workflow::new("task-1", Intent::Ask, budget());
    assert!(matches!(
        workflow.reserve(BudgetUsage {
            input_tokens: 101,
            ..BudgetUsage::default()
        }),
        WorkflowResult::Blocked(_)
    ));
    assert_eq!(workflow.phase, Phase::Blocked);
}

#[test]
fn context_slice_is_bounded_and_reports_truncation() {
    let request = ContextRequest {
        objective: "inspect".into(),
        node_ids: vec!["a".into()],
        max_bytes: 8,
    };
    let slice = ContextSlice::from_evidence(
        &request,
        [
            Evidence {
                reference: "a".into(),
                kind: EvidenceKind::Compiler,
                summary: "1234".into(),
            },
            Evidence {
                reference: "b".into(),
                kind: EvidenceKind::Inferred,
                summary: "5678".into(),
            },
        ],
    );
    assert_eq!(slice.evidence.len(), 1);
    assert!(slice.truncated);
}
