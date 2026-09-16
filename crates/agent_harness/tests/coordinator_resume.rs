use dowe_agent_harness::coordinator::*;

fn planned() -> Coordinator {
    let mut coordinator = Coordinator::new("resume", Intent::Build, "Implement changes").unwrap();
    coordinator
        .propose(
            vec![ScheduledTask::new("code", vec![], vec!["src".into()]).unwrap()],
            vec!["works".into()],
        )
        .unwrap();
    coordinator
}

#[test]
fn continuation_reapproves_and_reverifies_without_reexecuting_success() {
    let mut c = planned();
    c.approve(c.revision()).unwrap();
    c.start().unwrap();
    c.claim_ready(1).unwrap();
    c.finish_task("code", TaskResult::Succeeded).unwrap();
    c.verify(vec![CheckEvidence::passed("works", "tests")])
        .unwrap();
    c.review("reviewer", true).unwrap();
    let previous = c.revision();
    c.prepare_continuation().unwrap();
    assert_eq!(c.phase(), Phase::AwaitingApproval);
    assert!(c.approve(previous).is_err());
    c.approve(c.revision()).unwrap();
    c.start().unwrap();
    assert_eq!(c.phase(), Phase::Verification);
    assert!(c.claim_ready(1).is_err());
}

#[test]
fn running_or_uncertain_work_and_applied_integration_cannot_continue() {
    let mut c = planned();
    c.approve(c.revision()).unwrap();
    c.start().unwrap();
    c.claim_ready(1).unwrap();
    assert!(c.prepare_continuation().is_err());
    c.finish_task("code", TaskResult::Failed(FailureClass::UncertainEffect))
        .unwrap();
    assert!(c.prepare_continuation().is_err());
    let mut c = planned();
    c.record_integration(IntegrationSummary {
        applied: true,
        ..Default::default()
    });
    assert!(c.prepare_continuation().is_err());
}

#[test]
fn continuation_checks_plan_identity_independently_of_execution_state() {
    let expected = planned();
    let mut active = expected.clone();
    active.approve(active.revision()).unwrap();
    active.start().unwrap();
    assert!(active.has_same_plan(&expected));
    let mut different = planned();
    different
        .propose(
            vec![ScheduledTask::new("code", vec![], vec!["other".into()]).unwrap()],
            vec!["works".into()],
        )
        .unwrap();
    assert!(!active.has_same_plan(&different));
}
