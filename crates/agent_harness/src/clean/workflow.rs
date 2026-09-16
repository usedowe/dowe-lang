use super::{
    Approval, Budget, BudgetUsage, Failure, FailureClass, Intent, Requirement, Task, TaskState,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Intake,
    Requirements,
    Approval,
    Execution,
    Verification,
    Review,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowEvent {
    PhaseChanged(Phase),
    RequirementNeeded(String),
    ApprovalNeeded,
    TaskReady(String),
    Failure(Failure),
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowResult {
    Waiting,
    Ready,
    Completed,
    Blocked(Failure),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub intent: Intent,
    pub phase: Phase,
    pub requirements: Vec<Requirement>,
    pub tasks: Vec<Task>,
    pub approval: Option<Approval>,
    pub budget: Budget,
    pub usage: BudgetUsage,
    pub events: Vec<WorkflowEvent>,
}

impl Workflow {
    pub fn new(id: impl Into<String>, intent: Intent, budget: Budget) -> Self {
        Self {
            id: id.into(),
            intent,
            phase: Phase::Intake,
            requirements: Vec::new(),
            tasks: Vec::new(),
            approval: None,
            budget,
            usage: BudgetUsage::default(),
            events: Vec::new(),
        }
    }

    pub fn resolve_requirements(&mut self) -> WorkflowResult {
        self.phase = Phase::Requirements;
        if let Some(item) = self
            .requirements
            .iter()
            .find(|item| item.blocking && item.answer.is_none())
        {
            self.events
                .push(WorkflowEvent::RequirementNeeded(item.id.clone()));
            return WorkflowResult::Waiting;
        }
        self.phase = Phase::Approval;
        self.events.push(WorkflowEvent::PhaseChanged(self.phase));
        WorkflowResult::Ready
    }

    pub fn approve(&mut self, approval: Approval, plan_digest: &str) -> WorkflowResult {
        if approval.plan_digest != plan_digest || !approval.approved_by_host {
            return self.block(Failure::new(
                FailureClass::Permission,
                "approval does not match the plan",
                false,
            ));
        }
        self.approval = Some(approval);
        self.phase = if self.intent == Intent::Build {
            Phase::Execution
        } else {
            Phase::Completed
        };
        self.events.push(WorkflowEvent::PhaseChanged(self.phase));
        if self.phase == Phase::Completed {
            WorkflowResult::Completed
        } else {
            WorkflowResult::Ready
        }
    }

    pub fn admit_ready_tasks(&mut self) -> Vec<String> {
        if self.phase != Phase::Execution {
            return Vec::new();
        }
        let completed = self
            .tasks
            .iter()
            .filter(|task| task.state == TaskState::Completed)
            .map(|task| task.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let mut ready = Vec::new();
        for task in &mut self.tasks {
            if task.state == TaskState::Pending
                && task.depends_on.iter().all(|id| completed.contains(id))
            {
                task.state = TaskState::Ready;
                ready.push(task.id.clone());
                self.events.push(WorkflowEvent::TaskReady(task.id.clone()));
            }
        }
        ready
    }

    pub fn reserve(&mut self, usage: BudgetUsage) -> WorkflowResult {
        if !self.budget.accepts(self.usage, usage) {
            self.events.push(WorkflowEvent::BudgetExhausted);
            return self.block(Failure::new(
                FailureClass::Budget,
                "workflow budget exhausted",
                false,
            ));
        }
        self.usage = self.usage.add(usage);
        WorkflowResult::Ready
    }

    fn block(&mut self, failure: Failure) -> WorkflowResult {
        self.phase = Phase::Blocked;
        self.events.push(WorkflowEvent::Failure(failure.clone()));
        WorkflowResult::Blocked(failure)
    }
}
