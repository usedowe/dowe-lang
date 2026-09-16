use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContinuation {
    pub(crate) plan: WorkflowPlan,
    pub(crate) configuration: String,
    pub(crate) sessions: BTreeMap<String, String>,
    pub(crate) worktrees: BTreeMap<String, dowe_agent_harness::IsolatedWorktree>,
    #[serde(default)]
    pub(crate) worker_worktrees: BTreeMap<String, dowe_agent_harness::IsolatedWorktree>,
    pub(crate) task_attempts: BTreeMap<String, u32>,
    pub(crate) review_sessions: u32,
    pub(crate) elapsed_ms: u64,
    pub(crate) resumable: bool,
    pub(crate) candidate: Option<String>,
    #[serde(default)]
    pub(crate) codegraph_binding: Option<dowe_codegraph::CodeGraphBinding>,
}

pub(super) fn configuration(
    config: &HarnessConfig,
    active: &ModelSelection,
) -> AgentResult<String> {
    Ok(super::super::digest(&serde_json::to_vec(
        &json!({"config":config,"active":active}),
    )?))
}

pub(super) fn candidate_digest(
    store: &HarnessStore,
    worktrees: &BTreeMap<String, dowe_agent_harness::IsolatedWorktree>,
) -> AgentResult<Option<String>> {
    if worktrees.is_empty() {
        return Ok(None);
    }
    let prepared = dowe_agent_harness::prepare_isolated_worktrees(
        store.root(),
        &worktrees.values().cloned().collect::<Vec<_>>(),
    )
    .map_err(|e| AgentError::new(e.to_string()))?;
    Ok(Some(super::super::digest(&serde_json::to_vec(
        &prepared.approval_manifest(),
    )?)))
}

pub async fn resume_workflow(
    store: &HarnessStore,
    config: &HarnessConfig,
    active: &ModelSelection,
    id: &str,
    expected_sequence: u64,
    host: &mut impl HarnessHost,
) -> AgentResult<DriveOutcome> {
    let _lease = store.lease_workflow(id)?;
    let mut checkpoint = store.load_workflow(id)?;
    if checkpoint.sequence != expected_sequence {
        return Err(AgentError::new(
            "workflow changed; inspect its current sequence before resuming",
        ));
    }
    if checkpoint.coordinator.phase() == Phase::Completed {
        return Ok(DriveOutcome::Completed);
    }
    config.validate()?;
    let state = checkpoint.continuation.take().ok_or_else(|| {
        AgentError::new("checkpoint has no continuation evidence; inspect without replaying")
    })?;
    if !state.resumable || state.configuration != configuration(config, active)? {
        return Err(AgentError::new(
            "workflow was interrupted or configuration changed; inspection is required",
        ));
    }
    if state.plan.id != id || candidate_digest(store, &state.worktrees)? != state.candidate {
        return Err(AgentError::new(
            "retained workflow candidate changed; continuation rejected",
        ));
    }
    if !checkpoint
        .coordinator
        .has_same_plan(&state.plan.coordinator()?)
    {
        return Err(AgentError::new(
            "stored plan and coordinator disagree; continuation rejected",
        ));
    }
    if state.elapsed_ms >= config.duration_seconds.saturating_mul(1000) {
        return Ok(DriveOutcome::BudgetExhausted);
    }
    if store
        .root()
        .join(".dowe/integration-pending")
        .try_exists()?
    {
        return Err(AgentError::new(
            "pending integration requires inspection before continuation",
        ));
    }
    if !state.sessions.is_empty() && state.worktrees.len() != 1 {
        return Err(AgentError::new(
            "continuation of prior effects requires one isolated candidate",
        ));
    }
    for (task, worker) in &state.worker_worktrees {
        if !state.sessions.contains_key(task) {
            return Err(AgentError::new(
                "worker has no corresponding source session",
            ));
        }
        dowe_agent_harness::validate_isolated_worktree(store.root(), worker)
            .map_err(|e| AgentError::new(e.to_string()))?;
    }
    if matches!(state.plan.isolation, WorkflowIsolation::IsolatedWorkers)
        && state.worker_worktrees.len() != state.sessions.len()
    {
        return Err(AgentError::new(
            "isolated worker session ownership is incomplete",
        ));
    }
    let summary = checkpoint
        .coordinator
        .summary()
        .cloned()
        .ok_or_else(|| AgentError::new("workflow usage evidence is missing"))?;
    let budget = budget::WorkflowBudget::restore(config, &summary)?;
    checkpoint
        .coordinator
        .prepare_continuation()
        .map_err(AgentError::new)?;
    execute_workflow(
        store,
        config,
        active,
        &state.plan.clone(),
        checkpoint.coordinator,
        host,
        Some((checkpoint.sequence, state, budget)),
    )
    .await
}

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) fn continuation(&self) -> AgentResult<WorkflowContinuation> {
        let candidate = if self.resumable {
            candidate_digest(self.store, &self.worktrees)?
        } else {
            None
        };
        Ok(WorkflowContinuation {
            plan: self.plan.clone(),
            configuration: self.configuration.clone(),
            sessions: self
                .sessions
                .iter()
                .map(|(task, session)| (task.clone(), session.id.clone()))
                .collect(),
            worktrees: self.worktrees.clone(),
            worker_worktrees: self.worker_worktrees.clone(),
            task_attempts: self.task_attempts.clone(),
            review_sessions: self.review_sessions,
            elapsed_ms: self.elapsed_ms,
            resumable: self.resumable,
            candidate,
            codegraph_binding: self.codegraph_binding.clone(),
        })
    }
}
