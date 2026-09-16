use super::model::text;
use super::*;
use std::collections::BTreeSet;

impl Coordinator {
    pub fn new(id: &str, intent: Intent, objective: &str) -> CoordinatorResult<Self> {
        text(id, 128)?;
        text(objective, 8192)?;
        Ok(Self {
            id: id.into(),
            intent,
            objective: objective.into(),
            phase: Phase::Requirements,
            revision: 0,
            requirements: vec![],
            schedule: None,
            acceptance: vec![],
            approved_revision: None,
            evidence: vec![],
            reviewer: None,
            integration: None,
            summary: None,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }
    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    pub fn integration(&self) -> Option<&IntegrationSummary> {
        self.integration.as_ref()
    }

    pub fn record_integration(&mut self, summary: IntegrationSummary) {
        self.integration = Some(summary);
    }

    pub fn summary(&self) -> Option<&WorkflowSummary> {
        self.summary.as_ref()
    }

    pub fn record_summary(&mut self, summary: WorkflowSummary) {
        self.summary = Some(summary);
    }

    pub fn add_requirement(&mut self, requirement: Requirement) -> CoordinatorResult<()> {
        self.editable()?;
        text(&requirement.id, 128)?;
        text(&requirement.question, 2048)?;
        if let Some(answer) = &requirement.answer {
            text(answer, 8192)?;
        }
        if self.requirements.len() >= 128
            || self.requirements.iter().any(|r| r.id == requirement.id)
        {
            return Err("too many requirements or duplicate id".into());
        }
        self.requirements.push(requirement);
        self.invalidate();
        Ok(())
    }

    pub fn answer(&mut self, id: &str, answer: &str) -> CoordinatorResult<()> {
        self.editable()?;
        text(answer, 8192)?;
        let requirement = self
            .requirements
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("unknown requirement")?;
        requirement.answer = Some(answer.into());
        self.invalidate();
        Ok(())
    }

    pub fn propose(
        &mut self,
        tasks: Vec<ScheduledTask>,
        acceptance: Vec<String>,
    ) -> CoordinatorResult<()> {
        self.editable()?;
        if self
            .requirements
            .iter()
            .any(|r| r.blocking && r.answer.is_none())
        {
            return Err("blocking requirements remain unresolved".into());
        }
        if acceptance.is_empty() || acceptance.len() > 128 {
            return Err("plan requires 1..128 acceptance criteria".into());
        }
        for criterion in &acceptance {
            text(criterion, 2048)?;
        }
        if acceptance.iter().collect::<BTreeSet<_>>().len() != acceptance.len() {
            return Err("duplicate acceptance criterion".into());
        }
        let schedule = Schedule::new(tasks)?;
        self.invalidate();
        self.schedule = Some(schedule);
        self.acceptance = acceptance;
        self.phase = Phase::AwaitingApproval;
        Ok(())
    }

    pub fn approve(&mut self, revision: u64) -> CoordinatorResult<()> {
        if self.phase != Phase::AwaitingApproval || revision != self.revision {
            return Err("plan revision is not awaiting approval".into());
        }
        self.approved_revision = Some(revision);
        self.phase = Phase::Ready;
        Ok(())
    }

    pub fn start(&mut self) -> CoordinatorResult<()> {
        if self.intent != Intent::Build
            || self.phase != Phase::Ready
            || self.approved_revision != Some(self.revision)
        {
            return Err("BUILD requires approval for the current plan revision".into());
        }
        self.phase = if self.schedule.as_ref().is_some_and(Schedule::succeeded) {
            Phase::Verification
        } else {
            Phase::Executing
        };
        Ok(())
    }

    pub fn prepare_continuation(&mut self) -> CoordinatorResult<()> {
        if !matches!(
            self.phase,
            Phase::AwaitingApproval | Phase::Verification | Phase::Review | Phase::Deliverable
        ) || self
            .integration
            .as_ref()
            .is_some_and(|summary| summary.applied)
        {
            return Err("workflow is not at a resumable boundary".into());
        }
        let schedule = self.schedule.as_ref().ok_or("missing schedule")?;
        if !schedule.succeeded()
            && !schedule
                .tasks()
                .all(|task| task.state() == ExecutionState::Pending)
        {
            return Err(
                "partial, running or failed tasks require inspection before continuation".into(),
            );
        }
        self.revision = self.revision.checked_add(1).ok_or("revision exhausted")?;
        self.approved_revision = None;
        self.evidence.clear();
        self.reviewer = None;
        self.integration = None;
        self.phase = Phase::AwaitingApproval;
        Ok(())
    }

    pub fn has_same_plan(&self, other: &Self) -> bool {
        self.id == other.id
            && self.intent == other.intent
            && self.objective == other.objective
            && self.requirements == other.requirements
            && self.acceptance == other.acceptance
            && match (&self.schedule, &other.schedule) {
                (Some(left), Some(right)) => {
                    let left = left.tasks().collect::<Vec<_>>();
                    let right = right.tasks().collect::<Vec<_>>();
                    left.len() == right.len()
                        && left.iter().zip(right).all(|(a, b)| {
                            a.id == b.id
                                && a.dependencies == b.dependencies
                                && a.write_scopes == b.write_scopes
                        })
                }
                _ => false,
            }
    }

    pub fn claim_ready(&mut self, capacity: usize) -> CoordinatorResult<Vec<String>> {
        self.executing()?;
        self.schedule
            .as_mut()
            .ok_or("missing schedule")?
            .claim_ready(capacity)
    }

    pub fn finish_task(&mut self, id: &str, result: TaskResult) -> CoordinatorResult<()> {
        self.executing()?;
        let schedule = self.schedule.as_mut().ok_or("missing schedule")?;
        schedule.finish(id, result)?;
        if schedule.succeeded() {
            self.phase = Phase::Verification;
        }
        Ok(())
    }

    pub fn retry_task(&mut self, id: &str) -> CoordinatorResult<()> {
        self.executing()?;
        self.schedule.as_mut().ok_or("missing schedule")?.retry(id)
    }

    pub fn verify(&mut self, evidence: Vec<CheckEvidence>) -> CoordinatorResult<()> {
        if self.phase != Phase::Verification {
            return Err("tasks must succeed before verification".into());
        }
        if evidence.len() != self.acceptance.len() {
            return Err("evidence must cover every acceptance criterion".into());
        }
        for criterion in &self.acceptance {
            let matches: Vec<_> = evidence
                .iter()
                .filter(|e| &e.criterion == criterion)
                .collect();
            if matches.len() != 1 || !matches[0].passed {
                return Err("missing, duplicate or failed verification".into());
            }
            text(&matches[0].source, 2048)?;
        }
        self.evidence = evidence;
        self.phase = Phase::Review;
        Ok(())
    }

    pub fn review(&mut self, reviewer: &str, approved: bool) -> CoordinatorResult<()> {
        text(reviewer, 128)?;
        if self.phase != Phase::Review {
            return Err("verification is required before review".into());
        }
        if self
            .schedule
            .as_ref()
            .is_some_and(|s| s.tasks().any(|t| t.id() == reviewer))
        {
            return Err("reviewer must be independent from implementation tasks".into());
        }
        if !approved {
            self.invalidate();
            return Ok(());
        }
        self.reviewer = Some(reviewer.into());
        self.phase = Phase::Deliverable;
        Ok(())
    }

    pub fn deliver(&mut self) -> CoordinatorResult<()> {
        if self.phase != Phase::Deliverable {
            return Err("successful verification and review are required".into());
        }
        self.phase = Phase::Completed;
        Ok(())
    }

    pub fn recover(&mut self) {
        if let Some(schedule) = &mut self.schedule {
            schedule.interrupt();
        }
        self.approved_revision = None;
        self.evidence.clear();
        self.reviewer = None;
        self.integration = None;
        self.summary = None;
        self.revision = self.revision.saturating_add(1);
        self.phase = Phase::Planning;
    }

    fn executing(&self) -> CoordinatorResult<()> {
        if self.phase == Phase::Executing {
            Ok(())
        } else {
            Err("coordinator is not executing".into())
        }
    }

    fn editable(&self) -> CoordinatorResult<()> {
        if matches!(
            self.phase,
            Phase::Executing
                | Phase::Verification
                | Phase::Review
                | Phase::Deliverable
                | Phase::Completed
        ) {
            Err("active or completed plan cannot be changed without recovery".into())
        } else {
            Ok(())
        }
    }

    fn invalidate(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.approved_revision = None;
        self.evidence.clear();
        self.reviewer = None;
        self.integration = None;
        self.summary = None;
        self.schedule = None;
        self.phase = Phase::Planning;
    }
}
