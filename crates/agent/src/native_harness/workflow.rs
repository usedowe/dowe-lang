use super::*;
use crate::{AgentError, AgentResult};
use dowe_agent_harness::coordinator::*;
use dowe_agent_harness::{AutonomyLevel, RiskLevel, autonomy_allowed};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

mod backend;
mod budget;
mod budget_host;
#[cfg(test)]
mod budget_tests;
mod checkpoint;
mod component_planning;
mod continuation;
mod contracts;
mod execution;
mod integration;
mod parallel_execution;
mod plan_artifact;
mod plan_io;
mod planning;
mod requirements;
mod review;
mod reviewer;
mod summary;
mod visual;
mod workers;
pub use backend::{BackendArchitecturePlan, BackendEntity, BackendHandler, BackendRoute};
pub use continuation::{WorkflowContinuation, resume_workflow};
use execution::WorkflowAdapter;
pub use plan_io::read_workflow_plan;
pub use planning::{draft_workflow_plan, draft_workflow_plan_with_images};
#[allow(unused_imports)]
pub use visual::{
    PageArchitecturePlan, VisualBounds, VisualNode, VisualPage, VisualSection, VisualViewport,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPlan {
    pub id: String,
    pub objective: String,
    #[serde(default)]
    pub product_discovery: Option<dowe_agent_harness::ProductDiscovery>,
    #[serde(default)]
    pub product_use_cases: Vec<dowe_agent_harness::VerifiedProductUseCase>,
    #[serde(default)]
    pub view_components: Vec<crate::component_contracts::ComponentContract>,
    #[serde(default = "default_risk")]
    pub risk: RiskLevel,
    #[serde(default = "default_autonomy")]
    pub autonomy: AutonomyLevel,
    #[serde(default)]
    pub visual_plan: Option<PageArchitecturePlan>,
    #[serde(default)]
    pub backend_plan: Option<BackendArchitecturePlan>,
    #[serde(default)]
    pub verification_graph: Option<dowe_agent_harness::VerificationGraph>,
    #[serde(default)]
    pub specification: Option<WorkflowSourceBinding>,
    #[serde(default)]
    pub contracts: Vec<WorkflowSourceBinding>,
    #[serde(default)]
    pub codegraph_binding: Option<dowe_codegraph::CodeGraphBinding>,
    pub requirements: Vec<Requirement>,
    pub tasks: Vec<WorkflowTask>,
    pub checks: Vec<WorkflowCheck>,
    #[serde(default)]
    pub isolation: WorkflowIsolation,
    #[serde(default)]
    pub cleanup_after_integration: bool,
}

fn default_risk() -> RiskLevel {
    RiskLevel::Medium
}
fn default_autonomy() -> AutonomyLevel {
    AutonomyLevel::Feature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSourceBinding {
    pub path: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowIsolation {
    #[default]
    SharedCheckout,
    DetachedWorktrees,
    IsolatedWorkers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowTask {
    pub id: String,
    pub objective: String,
    pub dependencies: Vec<String>,
    pub write_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCheck {
    pub criterion: String,
    pub test_paths: Vec<String>,
}

impl WorkflowPlan {
    pub fn validate_draft(&self) -> AgentResult<()> {
        let mut requirements =
            Coordinator::new(&self.id, Intent::Build, &self.objective).map_err(AgentError::new)?;
        for requirement in &self.requirements {
            requirements
                .add_requirement(requirement.clone())
                .map_err(AgentError::new)?;
        }
        let mut shape = self.clone();
        shape.requirements.clear();
        shape.coordinator()?;
        if let Some(visual) = &self.visual_plan {
            visual.validate()?
        }
        if let Some(backend) = &self.backend_plan {
            backend.validate()?
        }
        if let Some(graph) = &self.verification_graph {
            graph
                .validate()
                .map_err(|error| AgentError::new(error.to_string()))?
        }
        Ok(())
    }

    pub fn coordinator(&self) -> AgentResult<Coordinator> {
        let mut coordinator =
            Coordinator::new(&self.id, Intent::Build, &self.objective).map_err(AgentError::new)?;
        for requirement in &self.requirements {
            coordinator
                .add_requirement(requirement.clone())
                .map_err(AgentError::new)?;
        }
        for task in &self.tasks {
            if task.objective.trim().is_empty() || task.objective.len() > 8192 {
                return Err(AgentError::new(
                    "each workflow task requires a bounded objective",
                ));
            }
        }
        for check in &self.checks {
            if check.criterion.trim().is_empty() || check.criterion.len() > 1024 {
                return Err(AgentError::new("each criterion requires a bounded name"));
            }
            if check.test_paths.len() > 32 {
                return Err(AgentError::new(
                    "each criterion accepts at most 32 test paths",
                ));
            }
        }
        let tasks = self
            .tasks
            .iter()
            .map(|task| {
                ScheduledTask::new(
                    &task.id,
                    task.dependencies.clone(),
                    task.write_scopes.clone(),
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(AgentError::new)?;
        coordinator
            .propose(
                tasks,
                self.checks
                    .iter()
                    .map(|check| check.criterion.clone())
                    .collect(),
            )
            .map_err(AgentError::new)?;
        Ok(coordinator)
    }
}

pub async fn run_workflow(
    store: &HarnessStore,
    config: &HarnessConfig,
    active: &ModelSelection,
    plan: &WorkflowPlan,
    host: &mut impl HarnessHost,
) -> AgentResult<DriveOutcome> {
    config.validate()?;
    let mut public_plan = serde_json::to_value(plan)?;
    let mut redactor = Redactor::for_project(store.root());
    for secret in host.secrets() {
        redactor.add(&secret);
    }
    redactor.value(&mut public_plan);
    let plan: WorkflowPlan = serde_json::from_value(public_plan)?;
    plan.validate_draft()?;
    validate_source_bindings(store.root(), &plan)?;
    if let Some(verified) = contracts::validate_registry(store.root(), &plan)? {
        host.event(
            &json!({"event":"workflow_contract_registry","status":"verified","contracts":verified}),
        )?;
    }
    validate_plan_codegraph_binding(store.root(), &plan)?;
    for check in &plan.checks {
        for path in &check.test_paths {
            HarnessTools::checked_path(store.root(), path)?;
        }
    }
    let _lease = store.lease_workflow(&plan.id)?;
    if store.workflow_path(&plan.id)?.exists() {
        return Err(AgentError::new(
            "workflow already exists; use explicit continuation",
        ));
    }
    let resolution = tokio::time::timeout(
        std::time::Duration::from_secs(config.duration_seconds),
        requirements::resolve(store, &plan, &redactor, host),
    )
    .await;
    let plan = match resolution {
        Ok(Ok(Some(plan))) => plan,
        Ok(Err(error)) => return Err(error),
        _ => return Ok(DriveOutcome::AwaitingRequirements),
    };
    let coordinator = plan.coordinator()?;
    execute_workflow(store, config, active, &plan, coordinator, host, None).await
}

fn validate_source_bindings(root: &std::path::Path, plan: &WorkflowPlan) -> AgentResult<()> {
    let mut bindings = plan.contracts.iter().collect::<Vec<_>>();
    if let Some(specification) = &plan.specification {
        bindings.push(specification);
    }
    if bindings.len() > 128 {
        return Err(AgentError::new(
            "workflow has too many specification bindings",
        ));
    }
    for binding in bindings {
        HarnessTools::checked_path(root, &binding.path)?;
        let bytes = std::fs::read(root.join(&binding.path))
            .map_err(|error| AgentError::new(format!("cannot read workflow binding: {error}")))?;
        if digest(&bytes) != binding.fingerprint {
            return Err(AgentError::new(format!(
                "workflow source binding is stale: {}",
                binding.path
            )));
        }
    }
    Ok(())
}

pub(crate) async fn run_direct_build_task(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    let plan = WorkflowPlan {
        id: format!("direct-{}", session.id),
        objective: task.prompt.to_string(),
        product_discovery: None,
        product_use_cases: Vec::new(),
        view_components: Vec::new(),
        risk: RiskLevel::Medium,
        autonomy: AutonomyLevel::Feature,
        visual_plan: None,
        backend_plan: None,
        verification_graph: None,
        specification: None,
        contracts: Vec::new(),
        codegraph_binding: None,
        requirements: Vec::new(),
        tasks: vec![WorkflowTask {
            id: "implementation".into(),
            objective: task.prompt.to_string(),
            dependencies: Vec::new(),
            write_scopes: vec![".".into()],
        }],
        checks: vec![WorkflowCheck {
            criterion: "Project compiles and passes declared validation".into(),
            test_paths: Vec::new(),
        }],
        isolation: WorkflowIsolation::SharedCheckout,
        cleanup_after_integration: false,
    };
    host.event(&json!({
        "event":"direct_build_promoted_to_workflow",
        "session":session.id,
        "workflow":plan.id
    }))?;
    let outcome = run_workflow(
        store,
        config,
        task.explicit.unwrap_or(task.active),
        &plan,
        host,
    )
    .await?;
    session.events.push(json!({
        "event":"direct_build_workflow_finished",
        "workflow":plan.id,
        "outcome":format!("{outcome:?}")
    }));
    store.save_session(session)?;
    Ok(match outcome {
        DriveOutcome::Completed => HarnessOutcome::Completed,
        DriveOutcome::AwaitingApproval => HarnessOutcome::ApprovalRequired,
        DriveOutcome::AwaitingRequirements => HarnessOutcome::ClarificationRequired,
        DriveOutcome::BudgetExhausted => HarnessOutcome::BudgetExhausted,
        DriveOutcome::Blocked | DriveOutcome::ReviewRejected => HarnessOutcome::ValidationFailed,
    })
}

fn validate_plan_codegraph_binding(root: &std::path::Path, plan: &WorkflowPlan) -> AgentResult<()> {
    let Some(expected) = plan.codegraph_binding.as_ref() else {
        return Ok(());
    };
    let snapshot =
        dowe_codegraph::clean::read_persistent_clean_codegraph(root).map_err(|error| {
            AgentError::new(format!(
                "workflow CodeGraph binding cannot be verified: {error}"
            ))
        })?;
    let actual = dowe_codegraph::clean::clean_binding(&snapshot);
    if &actual != expected {
        return Err(AgentError::new(
            "workflow plan CodeGraph binding was changed or no longer matches the project",
        ));
    }
    if matches!(
        snapshot.freshness,
        dowe_codegraph::GraphFreshness::Stale | dowe_codegraph::GraphFreshness::Error
    ) {
        return Err(AgentError::new(
            "workflow plan CodeGraph snapshot is stale; refresh and create a new plan",
        ));
    }
    Ok(())
}

async fn execute_workflow(
    store: &HarnessStore,
    config: &HarnessConfig,
    active: &ModelSelection,
    plan: &WorkflowPlan,
    mut coordinator: Coordinator,
    host: &mut impl HarnessHost,
    continuation: Option<(u64, WorkflowContinuation, budget::WorkflowBudget)>,
) -> AgentResult<DriveOutcome> {
    let mut path_tools = HarnessTools::new(store.root(), &plan.id, config.clone())?;
    for secret in host.secrets() {
        path_tools.redactor.add(&secret);
    }
    for check in &plan.checks {
        for path in &check.test_paths {
            HarnessTools::checked_path(store.root(), path)?;
        }
    }
    let mut execution_config = config.clone();
    if execution_config
        .roles
        .remove(&HarnessRole::Codegraph)
        .is_some()
    {
        host.event(&json!({"event":"workflow_enrichment_disabled","reason":"unmetered auxiliary transport"}))?;
    }
    let (sequence, state, budget) = if let Some((sequence, state, budget)) = continuation {
        (Some(sequence), Some(state), budget)
    } else {
        (None, None, budget::WorkflowBudget::new(config))
    };
    let codegraph_binding = state
        .as_ref()
        .and_then(|state| state.codegraph_binding.clone())
        .or_else(|| plan.codegraph_binding.clone())
        .or_else(|| {
            if !matches!(plan.isolation, WorkflowIsolation::SharedCheckout) {
                return None;
            }
            let snapshot =
                dowe_codegraph::clean::read_persistent_clean_codegraph(store.root()).ok()?;
            (snapshot.freshness == dowe_codegraph::GraphFreshness::Fresh)
                .then(|| dowe_codegraph::clean::clean_binding(&snapshot))
        });
    let trace = if state.is_some() {
        observability::load_trace(store.root(), &plan.id)?
            .unwrap_or_else(|| dowe_agent_harness::ObservabilityTrace::new(plan.id.clone()))
    } else {
        dowe_agent_harness::ObservabilityTrace::new(plan.id.clone())
    };
    let mut budget_host = budget_host::BudgetHost {
        inner: host,
        budget,
        parallel_budget: None,
    };
    let mut adapter = WorkflowAdapter {
        store,
        config: &execution_config,
        active,
        plan,
        host: &mut budget_host,
        sequence,
        sessions: BTreeMap::new(),
        path_tools,
        isolation: plan.isolation,
        worktrees: BTreeMap::new(),
        worker_worktrees: BTreeMap::new(),
        worker_results: BTreeMap::new(),
        cleanup_after_integration: plan.cleanup_after_integration,
        task_attempts: BTreeMap::new(),
        review_sessions: 0,
        verified_manifest: None,
        configuration: continuation::configuration(config, active)?,
        elapsed_ms: 0,
        resumable: false,
        codegraph_binding,
        trace,
    };
    if let Some(state) = state {
        adapter.elapsed_ms = state.elapsed_ms;
        adapter.worktrees = state.worktrees;
        adapter.worker_worktrees = state.worker_worktrees;
        adapter.task_attempts = state.task_attempts;
        adapter.review_sessions = state.review_sessions;
        let isolated = adapter
            .worktrees
            .values()
            .next()
            .map(|candidate| store.for_project_root(&candidate.path))
            .transpose()?;
        let session_store = isolated.as_ref().unwrap_or(store);
        for (task, session) in state.sessions {
            let worker_store = adapter
                .worker_worktrees
                .get(&task)
                .map(|worker| store.for_project_root(&worker.path))
                .transpose()?;
            adapter.sessions.insert(
                task,
                worker_store
                    .as_ref()
                    .unwrap_or(session_store)
                    .load_session(&session)?,
            );
        }
    }
    let started = std::time::Instant::now();
    let duration = std::time::Duration::from_millis(
        config
            .duration_seconds
            .saturating_mul(1000)
            .saturating_sub(adapter.elapsed_ms),
    );
    let result = tokio::time::timeout(
        duration,
        drive(&mut coordinator, &mut adapter, config.worker_capacity),
    )
    .await;
    let timed_out = result.is_err() || started.elapsed() >= duration;
    adapter.elapsed_ms = adapter
        .elapsed_ms
        .saturating_add(started.elapsed().as_millis().min(u64::MAX as u128) as u64);
    adapter.host.sync_parallel_budget()?;
    if timed_out || adapter.host.budget.exhausted {
        if timed_out {
            adapter.host.budget.interrupt();
        }
        coordinator.recover();
        coordinator.record_summary(adapter.workflow_summary(adapter.host.budget.snapshot()));
        if adapter.sequence.is_some() {
            adapter.checkpoint(&coordinator).map_err(AgentError::new)?;
        }
        adapter.host.budget.exhausted = true;
        adapter.host.inner.event(&adapter.host.budget.event())?;
        return Ok(DriveOutcome::BudgetExhausted);
    }
    let outcome = result.unwrap().map_err(AgentError::new);
    coordinator.record_summary(adapter.workflow_summary(adapter.host.budget.snapshot()));
    adapter.resumable = coordinator.clone().prepare_continuation().is_ok()
        && (adapter.sessions.is_empty() || adapter.worktrees.len() == 1)
        && (outcome.is_ok() || matches!(coordinator.phase(), Phase::Verification | Phase::Review))
        && !adapter.host.budget.exhausted;
    if adapter.sequence.is_some() {
        adapter.checkpoint(&coordinator).map_err(AgentError::new)?;
    }
    outcome
}
