use super::*;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) fn workflow_summary(&self, budget: budget::BudgetSnapshot) -> WorkflowSummary {
        let task_attempts = self.task_attempts.values().copied().sum::<u32>();
        WorkflowSummary {
            task_count: self.plan.tasks.len() as u32,
            task_attempts,
            retry_count: task_attempts.saturating_sub(self.task_attempts.len() as u32),
            session_count: self.sessions.len() as u32,
            review_sessions: self.review_sessions,
            charged_tokens: budget.tokens,
            cost_usd: budget.cost,
            cost_unknown: budget.unknown_cost,
            estimated: budget.estimated,
            provider_attempts: budget.attempts,
            budget_exhausted: budget.exhausted,
        }
    }
}
