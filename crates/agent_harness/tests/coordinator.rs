use dowe_agent_harness::coordinator::*;

fn task(id: &str, dependencies: &[&str], scope: &str) -> ScheduledTask {
    ScheduledTask::new(
        id,
        dependencies.iter().map(|s| s.to_string()).collect(),
        vec![scope.into()],
    )
    .unwrap()
}

#[test]
fn requirements_and_revision_bound_approval_control_build() {
    let mut coordinator = Coordinator::new("build-login", Intent::Build, "Build login").unwrap();
    coordinator
        .add_requirement(Requirement::blocking("auth", "Which identity provider?"))
        .unwrap();
    assert!(
        coordinator
            .propose(
                vec![task("backend", &[], "server")],
                vec!["login works".into()]
            )
            .is_err()
    );
    coordinator
        .answer("auth", "Existing identity service")
        .unwrap();
    coordinator
        .propose(
            vec![task("backend", &[], "server")],
            vec!["login works".into()],
        )
        .unwrap();
    assert!(coordinator.start().is_err());
    let revision = coordinator.revision();
    coordinator.approve(revision).unwrap();
    coordinator
        .answer("auth", "Changed identity service")
        .unwrap();
    assert!(coordinator.approve(revision).is_err());
    assert!(coordinator.start().is_err());
}

#[test]
fn scheduler_blocks_cycles_dependencies_and_overlapping_writes() {
    assert!(Schedule::new(vec![task("a", &["b"], "a"), task("b", &["a"], "b")]).is_err());
    assert!(Schedule::new(vec![task("a", &["missing"], "a")]).is_err());
    let mut schedule = Schedule::new(vec![
        task("a", &[], "views"),
        task("b", &[], "views/page"),
        task("c", &[], "server"),
        task("d", &["a"], "types"),
    ])
    .unwrap();
    assert_eq!(schedule.claim_ready(4).unwrap(), ["a", "c"]);
    schedule.finish("a", TaskResult::Succeeded).unwrap();
    assert_eq!(schedule.claim_ready(4).unwrap(), ["b", "d"]);
}

#[test]
fn uncertain_effects_are_never_retried_automatically() {
    let mut schedule = Schedule::new(vec![task("a", &[], "views")]).unwrap();
    schedule.claim_ready(1).unwrap();
    schedule
        .finish("a", TaskResult::Failed(FailureClass::UncertainEffect))
        .unwrap();
    assert!(schedule.claim_ready(1).unwrap().is_empty());
    assert!(!schedule.succeeded());
    assert!(schedule.retry("a").is_err());
}

#[test]
fn verification_requires_every_acceptance_and_an_independent_reviewer() {
    let mut coordinator = Coordinator::new("build", Intent::Build, "Build feature").unwrap();
    coordinator
        .propose(
            vec![task("code", &[], "src")],
            vec!["works".into(), "rejects invalid".into()],
        )
        .unwrap();
    coordinator.approve(coordinator.revision()).unwrap();
    coordinator.start().unwrap();
    coordinator.claim_ready(1).unwrap();
    coordinator
        .finish_task("code", TaskResult::Succeeded)
        .unwrap();
    assert!(
        coordinator
            .verify(vec![CheckEvidence::passed("works", "test:works")])
            .is_err()
    );
    coordinator
        .verify(vec![
            CheckEvidence::passed("works", "test:works"),
            CheckEvidence::passed("rejects invalid", "test:invalid"),
        ])
        .unwrap();
    assert!(coordinator.review("code", true).is_err());
    coordinator.review("independent-review", true).unwrap();
    coordinator.deliver().unwrap();
    assert_eq!(coordinator.phase(), Phase::Completed);
}

#[test]
fn ask_and_plan_never_acquire_build_authority() {
    for intent in [Intent::Ask, Intent::Plan] {
        let mut coordinator = Coordinator::new("readonly", intent, "Explain feature").unwrap();
        coordinator
            .propose(vec![task("code", &[], "src")], vec!["works".into()])
            .unwrap();
        coordinator.approve(coordinator.revision()).unwrap();
        assert!(coordinator.start().is_err());
    }
}
