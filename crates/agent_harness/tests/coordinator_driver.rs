use dowe_agent_harness::coordinator::*;
use std::future::Future;
use std::task::{Context, Poll, Waker};

struct Host {
    approve: bool,
    malformed: bool,
    fail_checkpoint: bool,
    attempts: usize,
    transient: bool,
    integrate: bool,
    integration_error: bool,
    integration_checkpoints: usize,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            approve: true,
            malformed: false,
            fail_checkpoint: false,
            attempts: 0,
            transient: false,
            integrate: true,
            integration_error: false,
            integration_checkpoints: 0,
        }
    }
}

impl CoordinatorHost for Host {
    async fn approve(&mut self, _: &Coordinator) -> CoordinatorResult<bool> {
        Ok(self.approve)
    }

    async fn execute_batch(
        &mut self,
        tasks: Vec<ScheduledTask>,
    ) -> CoordinatorResult<Vec<(String, TaskResult)>> {
        self.attempts += 1;
        if self.malformed {
            return Ok(vec![]);
        }
        Ok(tasks
            .iter()
            .map(|task| {
                (
                    task.id().into(),
                    if self.transient {
                        TaskResult::Failed(FailureClass::Transient)
                    } else {
                        TaskResult::Succeeded
                    },
                )
            })
            .collect())
    }

    async fn verify(&mut self, criteria: &[String]) -> CoordinatorResult<Vec<CheckEvidence>> {
        Ok(criteria
            .iter()
            .map(|c| CheckEvidence::passed(c, "host test receipt"))
            .collect())
    }

    async fn review(&mut self, _: &Coordinator) -> CoordinatorResult<(String, bool)> {
        Ok(("reviewer".into(), true))
    }

    async fn integrate(&mut self, coordinator: &mut Coordinator) -> CoordinatorResult<bool> {
        if self.integration_error {
            coordinator.record_integration(IntegrationSummary {
                applied: true,
                ..Default::default()
            });
            return Err("failure after effect".into());
        }
        Ok(self.integrate)
    }

    fn checkpoint(&mut self, coordinator: &Coordinator) -> CoordinatorResult<()> {
        if coordinator.integration().is_some() {
            self.integration_checkpoints += 1;
        }
        if self.fail_checkpoint {
            Err("storage unavailable".into())
        } else {
            Ok(())
        }
    }
}

fn coordinator() -> Coordinator {
    let mut c = Coordinator::new("feature", Intent::Build, "Build feature").unwrap();
    c.propose(
        vec![ScheduledTask::new("code", vec![], vec!["src".into()]).unwrap()],
        vec!["works".into()],
    )
    .unwrap();
    c
}

fn run(c: &mut Coordinator, host: &mut Host) -> CoordinatorResult<DriveOutcome> {
    let mut future = std::pin::pin!(drive(c, host, 2));
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("in-memory test host must complete without suspension"),
    }
}

#[test]
fn host_denial_or_checkpoint_failure_prevents_execution() {
    let mut denied = Host {
        approve: false,
        ..Default::default()
    };
    assert_eq!(
        run(&mut coordinator(), &mut denied).unwrap(),
        DriveOutcome::AwaitingApproval
    );
    assert_eq!(denied.attempts, 0);
    let mut broken = Host {
        fail_checkpoint: true,
        ..Default::default()
    };
    assert!(run(&mut coordinator(), &mut broken).is_err());
    assert_eq!(broken.attempts, 0);
}

#[test]
fn malformed_worker_receipts_preserve_uncertain_effects() {
    let mut c = coordinator();
    let mut host = Host {
        malformed: true,
        ..Default::default()
    };
    assert!(run(&mut c, &mut host).is_err());
    assert_eq!(
        c.schedule().unwrap().tasks().next().unwrap().state(),
        ExecutionState::Finished(TaskResult::Failed(FailureClass::UncertainEffect))
    );
    assert_eq!(run(&mut c, &mut host).unwrap(), DriveOutcome::Blocked);
    assert_eq!(host.attempts, 1);
}

#[test]
fn transient_retry_is_bounded_and_cannot_complete() {
    let mut host = Host {
        transient: true,
        ..Default::default()
    };
    let mut c = coordinator();
    assert_eq!(run(&mut c, &mut host).unwrap(), DriveOutcome::Blocked);
    assert_eq!(host.attempts, 3);
    assert_ne!(c.phase(), Phase::Completed);
}

#[test]
fn delivery_requires_all_host_stages() {
    let mut c = coordinator();
    assert_eq!(
        run(&mut c, &mut Host::default()).unwrap(),
        DriveOutcome::Completed
    );
    assert_eq!(c.phase(), Phase::Completed);
}

#[test]
fn integration_gate_blocks_delivery_until_host_accepts_results() {
    let mut host = Host {
        integrate: false,
        ..Default::default()
    };
    let mut c = coordinator();
    assert_eq!(run(&mut c, &mut host).unwrap(), DriveOutcome::Blocked);
    assert_eq!(c.phase(), Phase::Deliverable);
    host.integrate = true;
    assert_eq!(run(&mut c, &mut host).unwrap(), DriveOutcome::Completed);
}

#[test]
fn ambiguous_scope_aliases_are_rejected() {
    for scope in [
        "src/./views",
        "src/../views",
        "src//views",
        "/src",
        "src\\views",
    ] {
        assert!(
            ScheduledTask::new("code", vec![], vec![scope.into()]).is_err(),
            "{scope}"
        );
    }
}

#[test]
fn integration_error_checkpoints_effects_without_delivery() {
    let mut host = Host {
        integration_error: true,
        ..Default::default()
    };
    let mut coordinator = coordinator();
    assert!(run(&mut coordinator, &mut host).is_err());
    assert_eq!(coordinator.phase(), Phase::Deliverable);
    assert_eq!(host.integration_checkpoints, 1);
}
