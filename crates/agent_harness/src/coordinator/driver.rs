use super::*;

pub trait CoordinatorHost {
    fn approve(
        &mut self,
        coordinator: &Coordinator,
    ) -> impl std::future::Future<Output = CoordinatorResult<bool>>;
    fn execute_batch(
        &mut self,
        tasks: Vec<ScheduledTask>,
    ) -> impl std::future::Future<Output = CoordinatorResult<Vec<(String, TaskResult)>>>;
    fn verify(
        &mut self,
        criteria: &[String],
    ) -> impl std::future::Future<Output = CoordinatorResult<Vec<CheckEvidence>>>;
    fn review(
        &mut self,
        coordinator: &Coordinator,
    ) -> impl std::future::Future<Output = CoordinatorResult<(String, bool)>>;
    fn integrate(
        &mut self,
        _coordinator: &mut Coordinator,
    ) -> impl std::future::Future<Output = CoordinatorResult<bool>> {
        async { Ok(true) }
    }
    fn checkpoint(&mut self, coordinator: &Coordinator) -> CoordinatorResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveOutcome {
    AwaitingRequirements,
    AwaitingApproval,
    Blocked,
    Completed,
    ReviewRejected,
    BudgetExhausted,
}

pub async fn drive(
    coordinator: &mut Coordinator,
    host: &mut impl CoordinatorHost,
    capacity: usize,
) -> CoordinatorResult<DriveOutcome> {
    if capacity == 0 || capacity > 32 {
        return Err("capacity must be 1..32".into());
    }
    if coordinator.phase() == Phase::AwaitingApproval {
        host.checkpoint(coordinator)?;
        if !host.approve(coordinator).await? {
            return Ok(DriveOutcome::AwaitingApproval);
        }
        coordinator.approve(coordinator.revision())?;
        host.checkpoint(coordinator)?;
    }
    if coordinator.phase() == Phase::Ready {
        coordinator.start()?;
        host.checkpoint(coordinator)?;
    }
    while coordinator.phase() == Phase::Executing {
        let ids = coordinator.claim_ready(capacity)?;
        if ids.is_empty() {
            return Ok(DriveOutcome::Blocked);
        }
        host.checkpoint(coordinator)?;
        let tasks = coordinator
            .schedule()
            .ok_or("missing schedule")?
            .tasks()
            .filter(|t| ids.iter().any(|id| id == t.id()))
            .cloned()
            .collect();
        let results = host.execute_batch(tasks).await;
        let results = match results {
            Ok(results) => results,
            Err(error) => {
                for id in &ids {
                    coordinator
                        .finish_task(id, TaskResult::Failed(FailureClass::UncertainEffect))?;
                }
                host.checkpoint(coordinator)?;
                return Err(error);
            }
        };
        let unique: std::collections::BTreeSet<_> = results.iter().map(|(id, _)| id).collect();
        if results.len() != ids.len()
            || unique.len() != ids.len()
            || ids.iter().any(|id| !unique.contains(id))
        {
            for id in &ids {
                coordinator.finish_task(id, TaskResult::Failed(FailureClass::UncertainEffect))?;
            }
            host.checkpoint(coordinator)?;
            return Err("worker batch returned missing, duplicate or foreign results".into());
        }
        for (id, result) in results {
            coordinator.finish_task(&id, result)?;
            if result == TaskResult::Failed(FailureClass::Transient) {
                let _ = coordinator.retry_task(&id);
            }
        }
        host.checkpoint(coordinator)?;
    }
    if coordinator.phase() == Phase::Verification {
        let evidence = host.verify(&coordinator.acceptance).await?;
        coordinator.verify(evidence)?;
        host.checkpoint(coordinator)?;
    }
    if coordinator.phase() == Phase::Review {
        let (reviewer, approved) = host.review(coordinator).await?;
        coordinator.review(&reviewer, approved)?;
        host.checkpoint(coordinator)?;
        if !approved {
            return Ok(DriveOutcome::ReviewRejected);
        }
    }
    if coordinator.phase() == Phase::Deliverable {
        let integration = host.integrate(coordinator).await;
        host.checkpoint(coordinator)?;
        if !integration? {
            return Ok(DriveOutcome::Blocked);
        }
        coordinator.deliver()?;
        host.checkpoint(coordinator)?;
    }
    Ok(if coordinator.phase() == Phase::Completed {
        DriveOutcome::Completed
    } else {
        DriveOutcome::Blocked
    })
}
